//! Tunnel lifecycle for waX VPN.
//!
//! On Windows this drives the official WireGuardNT kernel driver
//! (wireguard.dll, loaded side-by-side) through the `wireguard-nt` crate.
//! DNS uses Windows APIs, with netsh only when the OS lacks the DNS API.
//! Traffic is not blocked if the tunnel drops.

#[cfg(windows)]
mod native_dns;
#[cfg(windows)]
mod native_firewall;

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

#[derive(Clone, Debug, Default)]
pub struct ConnectOptions;

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
    use windows_sys::Win32::Security::{CheckTokenMembership, CreateWellKnownSid, WinBuiltinAdministratorsSid};
    // A filtered UAC token is not an enabled Administrators member.
    // Query the effective token, not the Server service used by `net session`.
    let mut sid = [0u64; 9];
    let mut bytes = std::mem::size_of_val(&sid) as u32;
    let mut member = 0;
    unsafe {
        if CreateWellKnownSid(WinBuiltinAdministratorsSid, std::ptr::null_mut(), sid.as_mut_ptr().cast(), &mut bytes) == 0 {
            return false;
        }
        CheckTokenMembership(0, sid.as_mut_ptr().cast(), &mut member) != 0 && member != 0
    }
}

/// True when the current process holds administrator rights.
#[cfg(not(windows))]
pub fn is_elevated() -> bool {
    false
}
