//! WireGuard configuration parsing and serialization (wg-quick .conf format).

use std::net::IpAddr;
use std::str::FromStr;

use ipnet::IpNet;

use crate::{keys, CoreError};

#[derive(Clone, Debug, PartialEq)]
pub struct InterfaceConfig {
    pub private_key: [u8; 32],
    pub addresses: Vec<IpNet>,
    pub dns: Vec<IpAddr>,
    pub mtu: Option<u16>,
    pub listen_port: Option<u16>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PeerConfig {
    pub public_key: [u8; 32],
    pub preshared_key: Option<[u8; 32]>,
    pub allowed_ips: Vec<IpNet>,
    /// Endpoint as written in the config: `host:port`, `[v6]:port` or bare host.
    pub endpoint: Option<String>,
    pub persistent_keepalive: Option<u16>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WireGuardConfig {
    pub interface: InterfaceConfig,
    pub peers: Vec<PeerConfig>,
}

fn split_list(value: &str) -> impl Iterator<Item = &str> {
    value.split(',').map(|s| s.trim()).filter(|s| !s.is_empty())
}

impl WireGuardConfig {
    pub fn parse(text: &str) -> Result<WireGuardConfig, CoreError> {
        enum Section { None, Interface, Peer }

        let mut section = Section::None;
        let mut private_key: Option<[u8; 32]> = None;
        let mut addresses: Vec<IpNet> = Vec::new();
        let mut dns: Vec<IpAddr> = Vec::new();
        let mut mtu: Option<u16> = None;
        let mut listen_port: Option<u16> = None;

        let mut peers: Vec<PeerConfig> = Vec::new();
        let mut cur_public: Option<[u8; 32]> = None;
        let mut cur_psk: Option<[u8; 32]> = None;
        let mut cur_allowed: Vec<IpNet> = Vec::new();
        let mut cur_endpoint: Option<String> = None;
        let mut cur_keepalive: Option<u16> = None;
        let mut in_peer = false;

        let finish_peer = |in_peer: &mut bool,
                           cur_public: &mut Option<[u8; 32]>,
                           cur_psk: &mut Option<[u8; 32]>,
                           cur_allowed: &mut Vec<IpNet>,
                           cur_endpoint: &mut Option<String>,
                           cur_keepalive: &mut Option<u16>,
                           peers: &mut Vec<PeerConfig>|
         -> Result<(), CoreError> {
            if *in_peer {
                let public_key = cur_public
                    .take()
                    .ok_or_else(|| CoreError::Parse("peer missing PublicKey".into()))?;
                peers.push(PeerConfig {
                    public_key,
                    preshared_key: cur_psk.take(),
                    allowed_ips: std::mem::take(cur_allowed),
                    endpoint: cur_endpoint.take(),
                    persistent_keepalive: cur_keepalive.take(),
                });
                *in_peer = false;
            }
            Ok(())
        };

        // Windows editors may write a UTF-8 BOM before the first section.
        let text = text.trim_start_matches('\u{feff}');
        for (lineno, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            let err = |m: &str| CoreError::Parse(format!("line {}: {}", lineno + 1, m));
            if line.starts_with('[') {
                finish_peer(&mut in_peer, &mut cur_public, &mut cur_psk, &mut cur_allowed,
                            &mut cur_endpoint, &mut cur_keepalive, &mut peers)?;
                section = match line {
                    "[Interface]" => Section::Interface,
                    "[Peer]" => { in_peer = true; Section::Peer }
                    other => return Err(err(&format!("unknown section {}", other))),
                };
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| err("expected Key = Value"))?;
            let key = key.trim();
            let value = value.trim();
            match section {
                Section::Interface => match key {
                    "PrivateKey" => private_key = Some(keys::key_from_base64(value)?),
                    "Address" => {
                        for a in split_list(value) {
                            addresses.push(IpNet::from_str(a).map_err(|_| err(&format!("bad address {a}")))?);
                        }
                    }
                    "DNS" => {
                        for d in split_list(value) {
                            if let Ok(ip) = IpAddr::from_str(d) {
                                dns.push(ip);
                            }
                        }
                    }
                    "MTU" => mtu = Some(value.parse().map_err(|_| err("bad MTU"))?),
                    "ListenPort" => listen_port = Some(value.parse().map_err(|_| err("bad ListenPort"))?),
                    // wg-quick convenience keys we don't action but tolerate
                    "PreUp" | "PostUp" | "PreDown" | "PostDown" | "Table" | "SaveConfig" | "FwMark" => {}
                    other => return Err(err(&format!("unknown Interface key {other}"))),
                },
                Section::Peer => match key {
                    "PublicKey" => cur_public = Some(keys::key_from_base64(value)?),
                    "PresharedKey" => cur_psk = Some(keys::key_from_base64(value)?),
                    "AllowedIPs" => {
                        for a in split_list(value) {
                            cur_allowed.push(IpNet::from_str(a).map_err(|_| err(&format!("bad AllowedIP {a}")))?);
                        }
                    }
                    "Endpoint" => cur_endpoint = Some(value.to_string()),
                    "PersistentKeepalive" => {
                        cur_keepalive = Some(value.parse().map_err(|_| err("bad PersistentKeepalive"))?)
                    }
                    other => return Err(err(&format!("unknown Peer key {other}"))),
                },
                Section::None => return Err(err("key outside of any section")),
            }
        }
        finish_peer(&mut in_peer, &mut cur_public, &mut cur_psk, &mut cur_allowed,
                    &mut cur_endpoint, &mut cur_keepalive, &mut peers)?;

        let private_key = private_key.ok_or_else(|| CoreError::Parse("interface missing PrivateKey".into()))?;
        if peers.is_empty() {
            return Err(CoreError::Parse("no [Peer] section".into()));
        }
        Ok(WireGuardConfig {
            interface: InterfaceConfig { private_key, addresses, dns, mtu, listen_port },
            peers,
        })
    }

    pub fn to_conf_string(&self) -> String {
        let mut out = String::new();
        out.push_str("[Interface]\n");
        out.push_str(&format!("PrivateKey = {}\n", keys::key_to_base64(&self.interface.private_key)));
        if !self.interface.addresses.is_empty() {
            let list: Vec<String> = self.interface.addresses.iter().map(|a| a.to_string()).collect();
            out.push_str(&format!("Address = {}\n", list.join(", ")));
        }
        if !self.interface.dns.is_empty() {
            let list: Vec<String> = self.interface.dns.iter().map(|d| d.to_string()).collect();
            out.push_str(&format!("DNS = {}\n", list.join(", ")));
        }
        if let Some(mtu) = self.interface.mtu {
            out.push_str(&format!("MTU = {mtu}\n"));
        }
        if let Some(port) = self.interface.listen_port {
            out.push_str(&format!("ListenPort = {port}\n"));
        }
        for peer in &self.peers {
            out.push_str("\n[Peer]\n");
            out.push_str(&format!("PublicKey = {}\n", keys::key_to_base64(&peer.public_key)));
            if let Some(psk) = &peer.preshared_key {
                out.push_str(&format!("PresharedKey = {}\n", keys::key_to_base64(psk)));
            }
            let list: Vec<String> = peer.allowed_ips.iter().map(|a| a.to_string()).collect();
            out.push_str(&format!("AllowedIPs = {}\n", list.join(", ")));
            if let Some(ep) = &peer.endpoint {
                out.push_str(&format!("Endpoint = {ep}\n"));
            }
            if let Some(ka) = peer.persistent_keepalive {
                out.push_str(&format!("PersistentKeepalive = {ka}\n"));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys;

    fn sample() -> String {
        let priv_b64 = keys::key_to_base64(&keys::generate_private_key());
        let pub_b64 = keys::key_to_base64(&keys::generate_private_key());
        format!(
            "[Interface]\nPrivateKey = {priv_b64}\nAddress = 10.0.0.2/32, fd00::2/128\nDNS = 1.1.1.1, 2606:4700:4700::1111\nMTU = 1280\n\n[Peer]\nPublicKey = {pub_b64}\nAllowedIPs = 0.0.0.0/0, ::/0\nEndpoint = engage.cloudflareclient.com:2408\nPersistentKeepalive = 25\n"
        )
    }

    #[test]
    fn parse_roundtrip() {
        let text = sample();
        let cfg = WireGuardConfig::parse(&text).unwrap();
        assert_eq!(cfg.interface.dns.len(), 2);
        assert_eq!(cfg.interface.mtu, Some(1280));
        assert_eq!(cfg.peers.len(), 1);
        assert_eq!(cfg.peers[0].endpoint.as_deref(), Some("engage.cloudflareclient.com:2408"));
        let rendered = cfg.to_conf_string();
        let cfg2 = WireGuardConfig::parse(&rendered).unwrap();
        assert_eq!(cfg, cfg2);
    }

    #[test]
    fn accepts_utf8_bom_and_windows_line_endings() {
        let plain = sample();
        let windows = format!("\u{feff}{}", plain.replace('\n', "\r\n"));
        assert_eq!(
            WireGuardConfig::parse(&plain).unwrap(),
            WireGuardConfig::parse(&windows).unwrap(),
        );
    }

    #[test]
    fn rejects_missing_peer() {
        let priv_b64 = keys::key_to_base64(&keys::generate_private_key());
        let text = format!("[Interface]\nPrivateKey = {priv_b64}\nAddress = 10.0.0.2/32\n");
        assert!(WireGuardConfig::parse(&text).is_err());
    }
}
