//! Non-Windows stub so the workspace type-checks everywhere.

use wax_core::Profile;

use crate::{ConnectOptions, TunnelError, TunnelStats};

pub struct Tunnel;

impl Tunnel {
    pub fn connect(_profile: &Profile, _opts: &ConnectOptions) -> Result<Tunnel, TunnelError> {
        Err(TunnelError::Message("waX VPN tunnel is Windows-only for now".into()))
    }
    pub fn connect_progress(
        _profile: &Profile,
        _opts: &ConnectOptions,
        _progress: impl FnMut(&'static str),
    ) -> Result<Tunnel, TunnelError> {
        Err(TunnelError::Message("waX VPN tunnel is Windows-only for now".into()))
    }
    pub fn stats(&self) -> TunnelStats {
        TunnelStats::default()
    }
    pub fn is_up(&self) -> bool {
        false
    }
    pub fn shutdown(self) -> Result<(), TunnelError> {
        Ok(())
    }
    pub fn force_disconnect() -> Result<(), TunnelError> {
        Ok(())
    }
    pub fn existing_status() -> Result<Option<TunnelStats>, TunnelError> {
        Ok(None)
    }
}
