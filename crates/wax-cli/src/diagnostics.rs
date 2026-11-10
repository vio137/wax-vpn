//! Small allowlisted support report. Never read profiles, settings, logs or accounts.
use wax_tunnel::{TunnelError, TunnelStats};

pub fn report(elevated: bool, state: Result<Option<TunnelStats>, TunnelError>) -> String {
    report_for_platform(elevated, state, cfg!(windows))
}

fn report_for_platform(elevated: bool, state: Result<Option<TunnelStats>, TunnelError>, needs_admin: bool) -> String {
    let (status, code, stats) = if needs_admin && !elevated {
        ("unknown", Some("administrator_not_confirmed"), None)
    } else {
        match state {
            Ok(Some(s)) => ("adapter_up", None, Some(s)),
            Ok(None) => ("disconnected", None, None),
            Err(TunnelError::Driver(_)) => ("unknown", Some("driver_unavailable"), None),
            Err(TunnelError::Command(_)) => ("unknown", Some("system_query_failed"), None),
            Err(_) => ("unknown", Some("status_query_failed"), None),
        }
    };
    // Values are constructed here, never a raw error/config/debug dump.
    let age = stats.and_then(|s| s.last_handshake).and_then(|t| std::time::SystemTime::now().duration_since(t).ok()).map(|d| d.as_secs());
    serde_json::to_string_pretty(&serde_json::json!({
        "schema_version": 1,
        "app_version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "architecture": std::env::consts::ARCH,
        "administrator_check_passed": elevated,
        "tunnel_status": status,
        "status_error_code": code,
        "rx_bytes": stats.map(|s| s.rx_bytes),
        "tx_bytes": stats.map(|s| s.tx_bytes),
        "last_handshake_age_seconds": age,
        "internet_reachability": "not_tested",
        "privacy": "No profiles, keys, tokens, account IDs, endpoint IPs, hostnames, paths or logs collected"
    })).expect("allowlisted diagnostics JSON serializes")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn error_details_never_enter_report() {
        let private = "C:/Users/private-name/private-key-token.conf 203.0.113.8";
        for error in [TunnelError::Driver(private.into()), TunnelError::Command(private.into()), TunnelError::Message(private.into())] {
            let text = report(false, Err(error));
            assert!(!text.contains(private));
            assert!(!text.contains("203.0.113.8"));
            assert!(!text.contains("private-name"));
            let value: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(value["tunnel_status"], "unknown");
            assert!(value["rx_bytes"].is_null());
        }
    }
    #[test]
    fn missing_privilege_is_not_disconnected() {
        let text = report_for_platform(false, Ok(None), true);
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["tunnel_status"], "unknown");
        assert_eq!(value["status_error_code"], "administrator_not_confirmed");
    }
    #[test]
    fn connected_adapter_is_not_internet_claim() {
        let text = report(true, Ok(Some(TunnelStats { rx_bytes: 123, tx_bytes: 456, last_handshake: None })));
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["tunnel_status"], "adapter_up");
        assert_eq!(value["internet_reachability"], "not_tested");
        assert_eq!(value["rx_bytes"], 123);
        assert!(value["last_handshake_age_seconds"].is_null());
    }
}
