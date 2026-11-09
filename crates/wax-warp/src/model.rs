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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
