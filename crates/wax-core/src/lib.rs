//! wax-core: keys, WireGuard config parsing/serialization and profile storage.

pub mod conf;
pub mod keys;
pub mod profile;

pub use conf::{InterfaceConfig, PeerConfig, WireGuardConfig};
pub use profile::Profile;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("invalid key: {0}")]
    BadKey(String),
    #[error("config parse error: {0}")]
    Parse(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}
