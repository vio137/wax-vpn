//! Local persistence of the WARP registration plus profile generation.

use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use ipnet::IpNet;
use serde::{Deserialize, Serialize};

use wax_core::conf::WireGuardConfig;
use wax_core::{keys, InterfaceConfig, PeerConfig, Profile};

use crate::model::Registration;
use crate::WarpError;

#[derive(Clone, Serialize, Deserialize)]
pub struct WarpAccount {
    pub device_id: String,
    pub token: String,
    pub license: String,
    /// Base64 WireGuard private key generated at registration time.
    pub private_key: String,
    pub model: String,
    pub created_at: String,
    /// Raw registration payload; holds interface addresses and peer config.
    pub registration: Registration,
}

fn account_path() -> PathBuf {
    wax_core::profile::config_dir().join("warp-account.json")
}

pub fn save_account(account: &WarpAccount) -> Result<(), WarpError> {
    let path = account_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(account)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn load_account() -> Result<Option<WarpAccount>, WarpError> {
    let path = account_path();
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&text)?))
}

pub fn delete_account() -> Result<(), WarpError> {
    let path = account_path();
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// Default WARP port used by wgcf when the API only supplies a host.
const DEFAULT_WARP_PORT: u16 = 2408;

/// Build a WireGuard profile (wgcf `generate` equivalent) from a stored
/// registration.
pub fn warp_profile(account: &WarpAccount, name: &str) -> Result<Profile, WarpError> {
    let reg = &account.registration;
    let private_key = keys::key_from_base64(&account.private_key)?;
    let peer = reg
        .config
        .peers
        .first()
        .ok_or_else(|| WarpError::BadResponse("registration has no peer".into()))?;
    let peer_public = keys::key_from_base64(&peer.public_key)?;

    let raw_host = peer
        .endpoint
        .host
        .clone()
        .filter(|h| !h.is_empty())
        .or_else(|| {
            let v4 = peer.endpoint.v4.trim().to_string();
            if v4.is_empty() { None } else { Some(v4) }
        })
        .or_else(|| {
            let v6 = peer.endpoint.v6.trim().to_string();
            if v6.is_empty() { None } else { Some(v6) }
        })
        .ok_or_else(|| WarpError::BadResponse("registration has no endpoint".into()))?;
    // Strip any port the API may have attached; wgcf then appends its default.
    let host = raw_host
        .rsplit_once(':')
        .filter(|(_, tail)| tail.chars().all(|c| c.is_ascii_digit()))
        .map(|(h, _)| h.to_string())
        .unwrap_or(raw_host);
    let endpoint = format!("{host}:{DEFAULT_WARP_PORT}");

    let mut addresses = Vec::new();
    let v4 = reg.config.interface.addresses.v4.trim();
    if !v4.is_empty() {
        let ip = v4.strip_suffix("/32").unwrap_or(v4);
        addresses.push(
            IpNet::from_str(&format!("{ip}/32"))
                .map_err(|_| WarpError::BadResponse("bad v4 address; expected an IPv4 address".into()))?,
        );
    }
    let v6 = reg.config.interface.addresses.v6.trim();
    if !v6.is_empty() {
        let ip = v6.strip_suffix("/128").unwrap_or(v6);
        addresses.push(
            IpNet::from_str(&format!("{ip}/128"))
                .map_err(|_| WarpError::BadResponse("bad v6 address; expected an IPv6 address".into()))?,
        );
    }

    let dns = vec![
        "1.1.1.1".parse().unwrap(),
        "1.0.0.1".parse().unwrap(),
        "2606:4700:4700::1111".parse().unwrap(),
        "2606:4700:4700::1001".parse().unwrap(),
    ];

    Ok(Profile {
        name: name.to_string(),
        config: WireGuardConfig {
            interface: InterfaceConfig {
                private_key,
                addresses,
                dns,
                mtu: Some(1280),
                listen_port: None,
            },
            peers: vec![PeerConfig {
                public_key: peer_public,
                preshared_key: None,
                allowed_ips: vec![
                    "0.0.0.0/0".parse().unwrap(),
                    "::/0".parse().unwrap(),
                ],
                endpoint: Some(endpoint),
                persistent_keepalive: None,
            }],
        },
    })
}

impl std::fmt::Debug for WarpAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WarpAccount")
            .field("device_id", &self.device_id)
            .field("token", &"<redacted>")
            .field("license", &"<redacted>")
            .field("private_key", &"<redacted>")
            .field("model", &self.model)
            .field("created_at", &self.created_at)
            .field("registration", &self.registration)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_account_debug_redacts_credentials_without_changing_json() {
        let account = WarpAccount {
            device_id: "stored-device-fixture".into(), token: "stored-token-sentinel".into(),
            license: "stored-license-sentinel".into(), private_key: "stored-private-sentinel".into(),
            model: "PC-fixture".into(), created_at: "synthetic-date".into(),
            registration: Registration { token: "nested-token-sentinel".into(),
                account: crate::model::Account { license: Some("nested-license-sentinel".into()), ..Default::default() },
                ..Default::default() },
        };
        for output in [format!("{account:?}"), format!("{account:#?}")] {
            assert!(output.contains("<redacted>"));
            for secret in ["stored-token-sentinel", "stored-license-sentinel", "stored-private-sentinel", "nested-token-sentinel", "nested-license-sentinel"] {
                assert!(!output.contains(secret));
            }
            for field in ["stored-device-fixture", "PC-fixture", "synthetic-date", "registration:"] {
                assert!(output.contains(field));
            }
        }
        let json = serde_json::to_string(&account).unwrap();
        for secret in ["stored-token-sentinel", "stored-license-sentinel", "stored-private-sentinel", "nested-token-sentinel", "nested-license-sentinel"] {
            assert!(json.contains(secret));
        }
        let read: WarpAccount = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_value(&read).unwrap(), serde_json::to_value(&account).unwrap());
    }

    #[test]
    fn warp_profile_address_errors_use_fixed_guidance_without_echoing_payloads() {
        fn fixture(v4: &str, v6: &str) -> WarpAccount {
            WarpAccount {
                device_id: "offline-device".into(), token: "offline-token".into(),
                license: "offline-license".into(), private_key: keys::key_to_base64(&[1; 32]),
                model: "offline-PC".into(), created_at: "synthetic-date".into(),
                registration: Registration {
                    config: crate::model::Config {
                        interface: crate::model::ConfigInterface {
                            addresses: crate::model::NetworkAddress { v4: v4.into(), v6: v6.into() },
                        },
                        peers: vec![crate::model::Peer {
                            public_key: keys::key_to_base64(&[2; 32]),
                            endpoint: crate::model::Endpoint { host: Some("example.invalid:2408".into()), ..Default::default() },
                        }],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }
        }
        let sentinel = "PRIVATE_ADDRESS_SENTINEL_84b1";
        for (v4, v6, expected) in [
            (sentinel, "fd01::2", "bad v4 address; expected an IPv4 address"),
            ("172.16.0.2", sentinel, "bad v6 address; expected an IPv6 address"),
        ] {
            let err = warp_profile(&fixture(v4, v6), "offline").unwrap_err();
            assert_eq!(err.to_string(), format!("unexpected response: {expected}"));
            assert!(!format!("{err:?}").contains(sentinel));
        }
        for (v4, v6) in [
            ("172.16.0.2", "fd01::2"),
            ("172.16.0.2/32", "fd01::2/128"),
            (" 172.16.0.2/32 ", " fd01::2/128 "),
        ] {
            let profile = warp_profile(&fixture(v4, v6), "offline").unwrap();
            assert_eq!(profile.config.interface.addresses, vec!["172.16.0.2/32".parse::<IpNet>().unwrap(), "fd01::2/128".parse::<IpNet>().unwrap()]);
            assert_eq!(profile.config.interface.private_key, [1; 32]);
            assert_eq!(profile.config.interface.mtu, Some(1280));
            assert_eq!(profile.config.interface.dns, vec!["1.1.1.1".parse::<std::net::IpAddr>().unwrap(), "1.0.0.1".parse::<std::net::IpAddr>().unwrap(), "2606:4700:4700::1111".parse::<std::net::IpAddr>().unwrap(), "2606:4700:4700::1001".parse::<std::net::IpAddr>().unwrap()]);
            assert_eq!(profile.config.peers.len(), 1);
            let peer = &profile.config.peers[0];
            assert_eq!(peer.public_key, [2; 32]);
            assert_eq!(peer.endpoint.as_deref(), Some("example.invalid:2408"));
            assert_eq!(peer.allowed_ips, vec!["0.0.0.0/0".parse::<IpNet>().unwrap(), "::/0".parse::<IpNet>().unwrap()]);
        }
        let empty = warp_profile(&fixture("", ""), "offline").unwrap();
        assert!(empty.config.interface.addresses.is_empty());
        // Keep the current rejection of non-host prefixes; this is wording only.
        assert!(warp_profile(&fixture("172.16.0.2/24", ""), "offline").is_err());
        assert!(warp_profile(&fixture("", "fd01::2/64"), "offline").is_err());
    }

}
