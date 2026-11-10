//! CLI and GUI wording reflects WireGuard evidence, not an Internet connectivity probe.
use std::time::{Duration, SystemTime};

#[derive(Debug, PartialEq, Eq)]
pub enum Health {
    AwaitingHandshake,
    RecentHandshake,
    OlderHandshake,
}

pub fn health(handshake: Option<SystemTime>, now: SystemTime) -> Health {
    match handshake.and_then(|h| now.duration_since(h).ok()) {
        None => Health::AwaitingHandshake,
        Some(age) if age <= Duration::from_secs(180) => Health::RecentHandshake,
        Some(_) => Health::OlderHandshake,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn handshake_evidence_has_explicit_boundaries() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
        assert_eq!(health(None, now), Health::AwaitingHandshake);
        assert_eq!(health(Some(now), now), Health::RecentHandshake);
        assert_eq!(health(Some(now - Duration::from_secs(180)), now), Health::RecentHandshake);
        assert_eq!(health(Some(now - Duration::from_secs(181)), now), Health::OlderHandshake);
        assert_eq!(health(Some(now + Duration::from_secs(1)), now), Health::AwaitingHandshake);
    }
}
