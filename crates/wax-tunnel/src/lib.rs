//! Tunnel lifecycle for waX VPN.
//!
//! On Windows this drives the official WireGuardNT kernel driver
//! (wireguard.dll, loaded side-by-side) through the `wireguard-nt` crate.
//! DNS is configured with netsh and the optional kill switch uses Windows
//! Firewall rules (PowerShell) scoped to the tunnel interface alias.

use std::time::SystemTime;

#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::*;

#[cfg(not(windows))]
mod stub;
#[cfg(not(windows))]
pub use stub::*;

pub const ADAPTER_NAME: &str = "waX VPN";
pub const ADAPTER_POOL: &str = "waX VPN";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TunnelStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub last_handshake: Option<SystemTime>,
}

#[derive(Clone, Debug)]
pub struct ConnectOptions {
    /// Block all traffic outside the tunnel (firewall based).
    pub kill_switch: bool,
    /// While the kill switch is on, still allow local-subnet traffic.
    pub allow_lan: bool,
}

impl Default for ConnectOptions {
    fn default() -> Self {
        Self { kill_switch: false, allow_lan: true }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TunnelError {
    #[error("{0}")]
    Message(String),
    #[error("wireguard driver error: {0}")]
    Driver(String),
    #[error("failed to resolve endpoint {0}: {1}")]
    Resolve(String, String),
    #[error("command failed: {0}")]
    Command(String),
    #[error("core: {0}")]
    Core(#[from] wax_core::CoreError),
}

/// True when the current process holds administrator rights.
#[cfg(windows)]
pub fn is_elevated() -> bool {
    std::process::Command::new("net")
        .args(["session"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// True when the current process holds administrator rights.
#[cfg(not(windows))]
pub fn is_elevated() -> bool {
    false
}
