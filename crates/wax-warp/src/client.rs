//! Blocking WARP API client. Calls are few and short-lived; callers that
//! need responsiveness (the GUI) run them on a worker thread.

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::model::*;
use crate::WarpError;

/// Mirrors the constants wgcf currently uses for the Cloudflare WARP API.
pub const API_BASE: &str = "https://api.cloudflareclient.com/v0a5641";
pub const USER_AGENT: &str = "1.1.1.1/6.38.9-5641 (Android 16.0.0)";
pub const CF_CLIENT_VERSION: &str = "a-6.38.9-5641";

pub struct WarpClient {
    http: Client,
}

fn decode_result<T: serde::de::DeserializeOwned>(resp: reqwest::blocking::Response) -> Result<T, WarpError> {
    let status = resp.status();
    let body = resp.text().map_err(|e| WarpError::Http(e.to_string()))?;
    let value: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
        WarpError::BadResponse(format!("HTTP {status}, body is not JSON: {e}"))
    })?;
    // Failure: HTTP error status, or an explicit `"success": false` envelope.
    let envelope_failed = value.get("success").and_then(|v| v.as_bool()) == Some(false);
    if !status.is_success() || envelope_failed {
        let first = value
            .get("errors")
            .and_then(|e| e.as_array())
            .and_then(|a| a.first())
            .cloned()
            .unwrap_or_default();
        let code = first.get("code").and_then(|c| c.as_i64()).unwrap_or(i64::from(status.as_u16()));
        let message = first
            .get("message")
            .and_then(|m| m.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| format!("HTTP {status}"));
        return Err(WarpError::Api { code, message });
    }
    // The registration endpoint returns the object directly; other endpoints
    // may wrap it as {"success":true,"result":{...}}. Accept both.
    let payload = if value.get("success").is_some() && value.get("result").is_some() {
        value["result"].clone()
    } else {
        value
    };
    serde_json::from_value(payload)
        .map_err(|e| WarpError::BadResponse(format!("HTTP {status}, unexpected shape: {e}")))
}

impl WarpClient {
    pub fn new() -> Result<Self, WarpError> {
        let mut headers = HeaderMap::new();
        headers.insert("CF-Client-Version", HeaderValue::from_static(CF_CLIENT_VERSION));
        let http = Client::builder()
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .connect_timeout(std::time::Duration::from_secs(15))
            .timeout(std::time::Duration::from_secs(45))
            .build()
            .map_err(|e| WarpError::Http(e.to_string()))?;
        Ok(Self { http })
    }

    fn authed(&self, token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Ok(v) = HeaderValue::from_str(&format!("Bearer {token}")) {
            headers.insert(AUTHORIZATION, v);
        }
        headers
    }

    /// Register a brand-new WARP device/account. `tos_accepted` must be true -
    /// the caller is responsible for having shown Cloudflare's terms.
    pub fn register(
        &self,
        public_key_b64: &str,
        model: &str,
        tos_accepted: bool,
    ) -> Result<Registration, WarpError> {
        if !tos_accepted {
            return Err(WarpError::BadResponse(
                "Cloudflare Terms of Service were not accepted".to_string(),
            ));
        }
        let tos = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());
        let req = RegisterRequest::new(public_key_b64, model, &tos);
        let resp = self
            .http
            .post(format!("{API_BASE}/reg"))
            .json(&req)
            .send()
            .map_err(|e| WarpError::Http(e.to_string()))?;
        decode_result(resp)
    }

    pub fn account(&self, device_id: &str, token: &str) -> Result<Account, WarpError> {
        let resp = self
            .http
            .get(format!("{API_BASE}/reg/{device_id}/account"))
            .headers(self.authed(token))
            .send()
            .map_err(|e| WarpError::Http(e.to_string()))?;
        decode_result(resp)
    }

    pub fn devices(&self, device_id: &str, token: &str) -> Result<Vec<BoundDevice>, WarpError> {
        let resp = self
            .http
            .get(format!("{API_BASE}/reg/{device_id}/account/devices"))
            .headers(self.authed(token))
            .send()
            .map_err(|e| WarpError::Http(e.to_string()))?;
        decode_result(resp)
    }

    pub fn rename_device(&self, device_id: &str, token: &str, name: &str) -> Result<(), WarpError> {
        let body = serde_json::json!({ "name": name });
        let resp = self
            .http
            .patch(format!("{API_BASE}/reg/{device_id}"))
            .headers(self.authed(token))
            .json(&body)
            .send()
            .map_err(|e| WarpError::Http(e.to_string()))?;
        let _: serde_json::Value = decode_result(resp)?;
        Ok(())
    }

    pub fn set_license(&self, device_id: &str, token: &str, license: &str) -> Result<Account, WarpError> {
        let body = serde_json::json!({ "license": license });
        let resp = self
            .http
            .put(format!("{API_BASE}/reg/{device_id}/account/license"))
            .headers(self.authed(token))
            .json(&body)
            .send()
            .map_err(|e| WarpError::Http(e.to_string()))?;
        decode_result(resp)
    }

    pub fn delete_device(&self, device_id: &str, token: &str) -> Result<(), WarpError> {
        let resp = self
            .http
            .delete(format!("{API_BASE}/reg/{device_id}"))
            .headers(self.authed(token))
            .send()
            .map_err(|e| WarpError::Http(e.to_string()))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            Err(WarpError::Http(format!("status {status}: {body}")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registration_object_decodes_without_envelope() {
        let v: serde_json::Value = serde_json::from_str(r#"{"id":"abc","token":"t","account":{"id":"x","account_type":"free","license":"L"},"config":{"client_id":"AQID","interface":{"addresses":{"v4":"172.16.0.2","v6":"fd01::2"}},"peers":[{"public_key":"pk","endpoint":{"host":"h:2408","v4":"1.2.3.4:0","v6":"[::1]:0"}}]}}"#).unwrap();
        assert!(v.get("success").is_none());
        let r: Registration = serde_json::from_value(v).unwrap();
        assert_eq!(r.id, "abc");
        assert_eq!(r.config.peers.len(), 1);
    }
}
