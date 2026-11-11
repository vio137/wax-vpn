//! WARP API models. Field set mirrors the parts of the Cloudflare WARP
//! "Android" API (v0a5641) that wgcf uses; unknown fields are ignored so
//! Cloudflare-side additions don't break us.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ApiErrorItem {
    #[serde(default)]
    pub code: i64,
    #[serde(default)]
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Envelope<T> {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub errors: Vec<ApiErrorItem>,
    pub result: Option<T>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Account {
    #[serde(default)]
    pub id: String,
    #[serde(default, rename = "account_type")]
    pub account_type: String,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub warp_plus: Option<bool>,
    #[serde(default)]
    pub referral_count: Option<i64>,
    #[serde(default)]
    pub premium_data: Option<i64>,
    #[serde(default)]
    pub quota: Option<i64>,
    #[serde(default)]
    pub usage: Option<i64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Endpoint {
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub v4: String,
    #[serde(default)]
    pub v6: String,
    #[serde(default)]
    pub ports: Vec<u16>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Peer {
    #[serde(default, rename = "public_key")]
    pub public_key: String,
    #[serde(default)]
    pub endpoint: Endpoint,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NetworkAddress {
    #[serde(default)]
    pub v4: String,
    #[serde(default)]
    pub v6: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConfigInterface {
    #[serde(default)]
    pub addresses: NetworkAddress,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default, rename = "client_id")]
    pub client_id: String,
    #[serde(default)]
    pub interface: ConfigInterface,
    #[serde(default)]
    pub peers: Vec<Peer>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Registration {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub account: Account,
    #[serde(default)]
    pub config: Config,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, rename = "warp_enabled")]
    pub warp_enabled: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BoundDevice {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub created: Option<String>,
    #[serde(default)]
    pub active: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RegisterRequest {
    pub key: String,
    pub install_id: String,
    pub fcm_token: String,
    pub tos: String,
    pub model: String,
    pub serial_number: String,
    pub locale: String,
    pub os_version: String,
    pub key_type: String,
    pub tunnel_type: String,
}

impl RegisterRequest {
    pub fn new(public_key_b64: &str, model: &str, tos_rfc3339: &str) -> Self {
        Self {
            key: public_key_b64.to_string(),
            install_id: String::new(),
            fcm_token: String::new(),
            tos: tos_rfc3339.to_string(),
            model: model.to_string(),
            serial_number: String::new(),
            locale: "en_US".to_string(),
            os_version: "16.0.0".to_string(),
            key_type: "curve25519".to_string(),
            tunnel_type: "wireguard".to_string(),
        }
    }
}

// Debug is useful for diagnostics, but credentials never belong in it.
impl std::fmt::Debug for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Account")
            .field("id", &self.id)
            .field("account_type", &self.account_type)
            .field("license", &self.license.as_ref().map(|_| "<redacted>"))
            .field("warp_plus", &self.warp_plus)
            .field("referral_count", &self.referral_count)
            .field("premium_data", &self.premium_data)
            .field("quota", &self.quota)
            .field("usage", &self.usage)
            .finish()
    }
}

impl std::fmt::Debug for Registration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Registration")
            .field("id", &self.id)
            .field("token", &"<redacted>")
            .field("account", &self.account)
            .field("config", &self.config)
            .field("name", &self.name)
            .field("warp_enabled", &self.warp_enabled)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_and_registration_debug_redact_nested_credentials() {
        let account = Account { id: "account-fixture".into(), account_type: "free".into(),
            license: Some("license-secret-sentinel".into()), warp_plus: Some(false),
            referral_count: Some(3), premium_data: Some(4), quota: Some(5), usage: Some(6) };
        let registration = Registration { id: "device-fixture".into(),
            token: "token-secret-sentinel".into(), account: account.clone(),
            name: Some("synthetic fixture".into()), warp_enabled: Some(false), ..Default::default() };
        let envelope = Envelope { success: true, errors: vec![], result: Some(registration.clone()) };
        for pretty in [false, true] {
            let outputs = if pretty {
                vec![format!("{account:#?}"), format!("{registration:#?}"), format!("{envelope:#?}")]
            } else {
                vec![format!("{account:?}"), format!("{registration:?}"), format!("{envelope:?}")]
            };
            for output in outputs {
                assert!(output.contains("<redacted>"));
                assert!(!output.contains("license-secret-sentinel"));
                assert!(!output.contains("token-secret-sentinel"));
                assert!(output.contains("account-fixture"));
                assert!(output.contains("free"));
            }
        }
        let debug = format!("{account:?}");
        for field in ["warp_plus: Some(false)", "referral_count: Some(3)", "premium_data: Some(4)", "quota: Some(5)", "usage: Some(6)"] {
            assert!(debug.contains(field));
        }
        let debug = format!("{registration:?}");
        for field in ["device-fixture", "config:", "synthetic fixture", "warp_enabled: Some(false)"] {
            assert!(debug.contains(field));
        }
        let mut absent = account.clone(); absent.license = None;
        assert!(format!("{absent:?}").contains("license: None"));
        assert!(!format!("{absent:#?}").contains("<redacted>"));
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("license-secret-sentinel"));
        assert!(json.contains("token-secret-sentinel"));
        let read: Envelope<Registration> = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_value(&read).unwrap(), serde_json::to_value(&envelope).unwrap());
        let json = serde_json::to_string(&absent).unwrap();
        let read: Account = serde_json::from_str(&json).unwrap();
        assert!(read.license.is_none());
        assert_eq!(serde_json::to_value(&read).unwrap(), serde_json::to_value(&absent).unwrap());
    }
}
