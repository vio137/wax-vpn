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

fn split_list(value: &str) -> Option<Vec<&str>> {
    let values: Vec<_> = value.split(',').map(str::trim).collect();
    if values.iter().any(|item| item.is_empty()) { None } else { Some(values) }
}

// Bare numeric addresses represent a single host, never a wider network.
fn parse_ip_prefix(value: &str) -> Option<IpNet> {
    IpNet::from_str(value).ok().or_else(|| {
        let ip = IpAddr::from_str(value).ok()?;
        IpNet::new(ip, if ip.is_ipv4() { 32 } else { 128 }).ok()
    })
}

// Import-only syntax validation. Existing saved profiles retain their load path.
// This never resolves a host or changes the endpoint string.
fn import_endpoint_is_valid(value: &str) -> bool {
    let Some((host, port)) = value.rsplit_once(':') else { return false; };
    if host.is_empty() || port.parse::<u16>().is_err() { return false; }
    if host.starts_with('[') || host.ends_with(']') || host.contains(':') {
        let Some(inner) = host.strip_prefix('[').and_then(|h| h.strip_suffix(']')) else { return false; };
        // WireGuard accepts an IPv6 zone suffix. Keep it verbatim, without lookup.
        let address = inner.rsplit_once('%').map_or(inner, |(address, _)| address);
        return address.parse::<std::net::Ipv6Addr>().is_ok();
    }
    !host.contains(['[', ']'])
}

impl WireGuardConfig {
    pub(crate) fn validate_import_endpoints(&self) -> Result<(), CoreError> {
        for (index, peer) in self.peers.iter().enumerate() {
            if peer.endpoint.as_deref().is_some_and(|value| !import_endpoint_is_valid(value)) {
                return Err(CoreError::Parse(format!("peer {}: invalid Endpoint; expected host:port or [IPv6]:port with port 0-65535", index + 1)));
            }
        }
        Ok(())
    }

    pub fn parse(text: &str) -> Result<WireGuardConfig, CoreError> {
        enum Section { None, Interface, Peer }

        let mut section = Section::None;
        let mut interface_header_line: Option<usize> = None;
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
        let mut peer_header_line = 0usize;

        let finish_peer = |in_peer: &mut bool, peer_header_line: usize,
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
                    .ok_or_else(|| CoreError::Parse(format!("line {peer_header_line}: [Peer] missing PublicKey")))?;
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
            // Inline hash comments apply to headers and values alike.
            // Normalized serialization intentionally does not retain comments.
            let line = raw.split_once('#').map_or(raw, |(code, _)| code).trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            let err = |m: &str| CoreError::Parse(format!("line {}: {}", lineno + 1, m));
            if line.starts_with('[') {
                finish_peer(&mut in_peer, peer_header_line, &mut cur_public, &mut cur_psk, &mut cur_allowed,
                            &mut cur_endpoint, &mut cur_keepalive, &mut peers)?;
                section = match line {
                    name if name.eq_ignore_ascii_case("[Interface]") => { interface_header_line.get_or_insert(lineno + 1); Section::Interface },
                    name if name.eq_ignore_ascii_case("[Peer]") => { in_peer = true; peer_header_line = lineno + 1; Section::Peer }
                    other => return Err(err(&format!("unknown section {}", other))),
                };
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| err("expected Key = Value"))?;
            // Normalize known names only. Values and unknown names stay untouched.
            let raw_key = key.trim();
            let key = ["PrivateKey", "Address", "DNS", "MTU", "ListenPort", "PublicKey",
                "PresharedKey", "AllowedIPs", "Endpoint", "PersistentKeepalive", "PreUp",
                "PostUp", "PreDown", "PostDown", "Table", "SaveConfig", "FwMark"]
                .into_iter().find(|known| known.eq_ignore_ascii_case(raw_key)).unwrap_or(raw_key);
            let value = value.trim();
            // Repeated scalar keys used to silently replace the earlier value.
            // Reject them before parsing or echoing any potentially secret value.
            let duplicate_section = match section {
                Section::Interface if match key {
                    "PrivateKey" => private_key.is_some(),
                    "MTU" => mtu.is_some(),
                    "ListenPort" => listen_port.is_some(),
                    _ => false,
                } => Some("Interface"),
                Section::Peer if match key {
                    "PublicKey" => cur_public.is_some(),
                    "PresharedKey" => cur_psk.is_some(),
                    "Endpoint" => cur_endpoint.is_some(),
                    "PersistentKeepalive" => cur_keepalive.is_some(),
                    _ => false,
                } => Some("Peer"),
                _ => None,
            };
            if let Some(section_name) = duplicate_section {
                return Err(err(&format!("duplicate {key} in [{section_name}]; keep only one value")));
            }
            match section {
                Section::Interface => match key {
                    "PrivateKey" => private_key = Some(keys::key_from_base64(value).map_err(|_| err("bad PrivateKey; expected a base64-encoded 32-byte key"))?),
                    "Address" => {
                        for a in split_list(value).ok_or_else(|| err("empty Address list entry"))? {
                            addresses.push(parse_ip_prefix(a).ok_or_else(|| err("bad Address; expected an IP address or CIDR prefix"))?);
                        }
                    }
                    "DNS" => {
                        for d in split_list(value).ok_or_else(|| err("empty DNS list entry"))? {
                            let ip = IpAddr::from_str(d).map_err(|_| err("DNS supports only IP addresses; search domains and hostnames are not supported"))?;
                            dns.push(ip);
                        }
                    }
                    "MTU" => mtu = Some(value.parse().map_err(|_| err("bad MTU; expected an integer from 576 to 65535 (1280 minimum with an IPv6 interface address)"))?),
                    "ListenPort" => listen_port = Some(value.parse().map_err(|_| err("bad ListenPort; expected an integer from 0 to 65535"))?),
                    // Never silently strip instructions that can change a
                    // profile's routing, firewall, DNS or lifecycle behavior.
                    "PreUp" | "PostUp" | "PreDown" | "PostDown" | "Table" | "SaveConfig" | "FwMark" => {
                        return Err(err(&format!("unsupported wg-quick directive {key}; waX VPN does not run scripts or apply this setting")));
                    }
                    other => return Err(err(&format!("unknown Interface key {other}"))),
                },
                Section::Peer => match key {
                    "PublicKey" => cur_public = Some(keys::key_from_base64(value).map_err(|_| err("bad PublicKey; expected a base64-encoded 32-byte key"))?),
                    "PresharedKey" => cur_psk = Some(keys::key_from_base64(value).map_err(|_| err("bad PresharedKey; expected a base64-encoded 32-byte key"))?),
                    "AllowedIPs" => {
                        // Older waX normalized files wrote an empty line for no routes.
                        if value.is_empty() { continue; }
                        for a in split_list(value).ok_or_else(|| err("empty AllowedIPs list entry"))? {
                            cur_allowed.push(parse_ip_prefix(a).ok_or_else(|| err("bad AllowedIP; expected an IP address or CIDR prefix"))?);
                        }
                    }
                    "Endpoint" => cur_endpoint = Some(value.to_string()),
                    "PersistentKeepalive" => {
                        // The explicit disabled form used by WireGuard configs.
                        cur_keepalive = Some(if value == "off" { 0 } else {
                            value.parse().map_err(|_| err("bad PersistentKeepalive; expected off or an integer from 0 to 65535 seconds"))?
                        })
                    }
                    other => return Err(err(&format!("unknown Peer key {other}"))),
                },
                Section::None => return Err(err("key outside of any section")),
            }
        }
        finish_peer(&mut in_peer, peer_header_line, &mut cur_public, &mut cur_psk, &mut cur_allowed,
                    &mut cur_endpoint, &mut cur_keepalive, &mut peers)?;

        let private_key = private_key.ok_or_else(|| CoreError::Parse(match interface_header_line {
            Some(line) => format!("line {line}: [Interface] missing PrivateKey"),
            None => "interface missing PrivateKey".into(),
        }))?;
        if peers.is_empty() {
            return Err(CoreError::Parse("no [Peer] section".into()));
        }
        if let Some(mtu) = mtu {
            let minimum = if addresses.iter().any(|a| a.addr().is_ipv6()) { 1280 } else { 576 };
            if mtu < minimum {
                return Err(CoreError::Parse(format!("MTU must be at least {minimum} for this profile's address families")));
            }
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
            if !peer.allowed_ips.is_empty() {
                let list: Vec<String> = peer.allowed_ips.iter().map(|a| a.to_string()).collect();
                out.push_str(&format!("AllowedIPs = {}\n", list.join(", ")));
            }
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
    fn known_names_ignore_ascii_case_without_changing_values_or_validation() {
        let plain = sample().replace("MTU = 1280", "MTU = 1280\nListenPort = 12345")
            .replace("[Peer]", &format!("[Peer]\nPresharedKey = {}", keys::key_to_base64(&[3; 32])))
            .replace("engage.cloudflareclient.com", "MiXeD.Example")
            .replace("PersistentKeepalive = 25", "PersistentKeepalive = off");
        let mixed = plain.lines().map(|line| {
            if line.starts_with('[') { line.to_ascii_lowercase() }
            else if let Some((key, value)) = line.split_once('=') {
                format!("{}={value}", key.to_ascii_lowercase())
            } else { line.to_string() }
        }).collect::<Vec<_>>().join("\n");
        let parsed = WireGuardConfig::parse(&mixed).unwrap();
        assert_eq!(parsed, WireGuardConfig::parse(&plain).unwrap());
        assert_eq!(parsed.peers[0].endpoint.as_deref(), Some("MiXeD.Example:2408"));
        let rendered = parsed.to_conf_string();
        assert!(rendered.contains("[Interface]\nPrivateKey = "));
        assert!(rendered.contains("[Peer]\nPublicKey = "));
        assert_eq!(WireGuardConfig::parse(&rendered).unwrap(), parsed);
        for key in ["PrivateKey", "MTU", "ListenPort", "PublicKey", "PresharedKey", "Endpoint", "PersistentKeepalive"] {
            let source = plain.lines().find(|line| line.starts_with(&format!("{key} ="))).unwrap();
            let duplicate = plain.replace(source, &format!("{source}\n{} = do-not-echo-private-value", key.to_ascii_uppercase()));
            let error = WireGuardConfig::parse(&duplicate).unwrap_err().to_string();
            assert!(error.contains(&format!("duplicate {key} in [")), "{error}");
            assert!(!error.contains("do-not-echo-private-value"));
        }
        for directive in ["PreUp", "PostUp", "PreDown", "PostDown", "Table", "SaveConfig", "FwMark"] {
            let invalid = plain.replace("MTU = 1280", &format!("{} = do-not-echo-private-value", directive.to_ascii_lowercase()));
            let error = WireGuardConfig::parse(&invalid).unwrap_err().to_string();
            assert!(error.contains(&format!("unsupported wg-quick directive {directive}")), "{error}");
            assert!(!error.contains("do-not-echo-private-value"));
        }
        for (old, new) in [("mtu = 1280", "mtu = 0"), ("persistentkeepalive = off", "persistentkeepalive = OFF"),
            ("dns = 1.1.1.1, 2606:4700:4700::1111", "dns = 1.1.1.1,,8.8.8.8"),
            ("listenport = 12345", "unknownfield = do-not-echo-private-value"), ("[peer]", "[unknown]")] {
            assert!(mixed.contains(old), "fixture missing {old}");
            assert!(WireGuardConfig::parse(&mixed.replace(old, new)).is_err(), "accepted {new}");
        }
        let repeated = mixed.replace("[peer]", "aDdReSs = 10.1.0.2/32\ndNs = 8.8.8.8\n[pEeR]")
            .replace("endpoint =", "aLlOwEdIpS = 192.0.2.1\nendpoint =");
        let repeated = WireGuardConfig::parse(&repeated).unwrap();
        assert_eq!(repeated.interface.addresses.len(), 3);
        assert_eq!(repeated.interface.dns.len(), 3);
        assert_eq!(repeated.peers[0].allowed_ips.len(), 3);
    }

    #[test]
    fn key_errors_identify_field_and_line_without_echoing_payload() {
        let plain = sample().replace("[Peer]", &format!("[Peer]\nPresharedKey = {}", keys::key_to_base64(&[3; 32])));
        let malformed = "do-not-echo-private-value!!!";
        let short = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, [1u8; 31]);
        let long = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, [2u8; 33]);
        for field in ["PrivateKey", "PublicKey", "PresharedKey"] {
            for value in [malformed, short.as_str(), long.as_str(), ""] {
                let mut field_line = 0;
                let invalid = plain.lines().enumerate().map(|(i, line)| {
                    if line.starts_with(&format!("{field} =")) {
                        field_line = i + 1;
                        format!("{} = {value}", field.to_ascii_lowercase())
                    } else { line.to_string() }
                }).collect::<Vec<_>>().join("\n");
                let error = WireGuardConfig::parse(&invalid).unwrap_err().to_string();
                assert!(error.contains(&format!("line {field_line}: bad {field}; expected a base64-encoded 32-byte key")), "{error}");
                if !value.is_empty() { assert!(!error.contains(value), "key payload echoed"); }
                assert!(!error.contains("Invalid byte"));
                assert!(!error.contains("decoded key is"));
            }
        }
        let parsed = WireGuardConfig::parse(&plain).unwrap();
        assert_eq!(WireGuardConfig::parse(&parsed.to_conf_string()).unwrap(), parsed);
    }

    #[test]
    fn missing_public_key_identifies_peer_header_without_payloads() {
        let private = keys::key_to_base64(&[1; 32]);
        let public = keys::key_to_base64(&[2; 32]);
        let header = format!("\u{feff}# comment\n[interface] # interface\nPrivateKey = {private}\n");
        let complete = format!("[pEeR] # peer\nPublicKey = {public}\nAllowedIPs = 10.0.0.1/32\n");
        let incomplete = "[pEeR] # missing public key\nEndpoint = never-echo-this.example:1234\n";
        for position in 0..3 {
            for next_section in [false, true] {
                let mut text = header.clone();
                let mut expected_line = 0;
                for peer in 0..3 {
                    if peer == position {
                        expected_line = text.lines().count() + 1;
                        text.push_str(incomplete);
                        if !next_section { break; }
                    } else { text.push_str(&complete); }
                }
                if next_section && position == 2 { text.push_str("[Interface]\n"); }
                let error = WireGuardConfig::parse(&text).unwrap_err().to_string();
                assert!(error.contains(&format!("line {expected_line}: [Peer] missing PublicKey")), "{error}");
                for payload in [private.as_str(), public.as_str(), "never-echo-this.example:1234"] {
                    assert!(!error.contains(payload), "config payload echoed");
                }
            }
        }
        let valid = format!("{header}{complete}{complete}{complete}");
        let parsed = WireGuardConfig::parse(&valid).unwrap();
        assert_eq!(parsed.peers.len(), 3);
        assert_eq!(WireGuardConfig::parse(&parsed.to_conf_string()).unwrap(), parsed);
    }

    #[test]
    fn missing_private_key_identifies_first_interface_header_without_payloads() {
        let public = keys::key_to_base64(&[2; 32]);
        let private = keys::key_to_base64(&[1; 32]);
        let peer = format!("[Peer]\nPublicKey = {public}\nEndpoint = never-echo-this.example:1234\n");
        for (prefix, header, line) in [("", "[Interface]", 1),
            ("\u{feff}# comment\n\n; comment\n", "[iNtErFaCe] # comment", 4)] {
            for repeated in [false, true] {
                let extra = if repeated { "[Interface]\nDNS = 1.1.1.1\n" } else { "" };
                let text = format!("{prefix}{header}\nAddress = 10.0.0.2/32\n{peer}{extra}");
                let error = WireGuardConfig::parse(&text).unwrap_err().to_string();
                assert!(error.contains(&format!("line {line}: [Interface] missing PrivateKey")), "{error}");
                for payload in [public.as_str(), "never-echo-this.example:1234", "10.0.0.2/32"] {
                    assert!(!error.contains(payload), "config payload echoed");
                }
                let valid = text.replacen(header, &format!("{header}\nPrivateKey = {private}"), 1);
                let parsed = WireGuardConfig::parse(&valid).unwrap();
                assert_eq!(WireGuardConfig::parse(&parsed.to_conf_string()).unwrap(), parsed);
            }
        }
        for text in [String::new(), peer] {
            let error = WireGuardConfig::parse(&text).unwrap_err().to_string();
            assert!(error.contains("interface missing PrivateKey"), "{error}");
            assert!(!error.contains("line "), "{error}");
            assert!(!error.contains(&public));
            assert!(!error.contains("never-echo-this.example:1234"));
        }
    }

    #[test]
    fn import_endpoint_syntax_preserves_valid_values_and_legacy_parse() {
        let plain = sample();
        let with_endpoint = |value: &str| plain.lines().map(|line| {
            if line.starts_with("Endpoint =") { format!("eNdPoInT = {value}") }
            else { line.to_string() }
        }).collect::<Vec<_>>().join("\n");
        for value in ["192.0.2.1:0", "vpn.Example:65535", "[2001:db8::1]:51820", "[fe80::1%12]:51820", "[fe80::1%Ethernet 3]:51820", "[::ffff:192.0.2.1]:1234"] {
            let parsed = WireGuardConfig::parse(&with_endpoint(value)).unwrap();
            parsed.validate_import_endpoints().unwrap();
            assert_eq!(parsed.peers[0].endpoint.as_deref(), Some(value));
            assert_eq!(WireGuardConfig::parse(&parsed.to_conf_string()).unwrap(), parsed);
        }
        for value in ["", "never-echo.invalid", ":1234", "host:", "host:bad", "host:65536", "host:-1", "[192.0.2.1]:1234", "[not-v6]:1234", "[::1:1234", "::1:1234", "host]:1234", "[::1]junk:1234"] {
            // Loading existing profiles remains permissive, including odd values.
            let parsed = WireGuardConfig::parse(&with_endpoint(value)).unwrap();
            assert_eq!(parsed.peers[0].endpoint.as_deref(), Some(value));
            let error = parsed.validate_import_endpoints().unwrap_err().to_string();
            assert!(error.contains("peer 1: invalid Endpoint; expected host:port or [IPv6]:port with port 0-65535"));
            assert!(!error.contains("never-echo.invalid"));
        }
        let absent = plain.lines().filter(|line| !line.starts_with("Endpoint =")).collect::<Vec<_>>().join("\n");
        WireGuardConfig::parse(&absent).unwrap().validate_import_endpoints().unwrap();
        let mut multi = WireGuardConfig::parse(&plain).unwrap();
        let mut bad = multi.peers[0].clone();
        bad.endpoint = Some("never-echo.invalid".into());
        multi.peers.push(bad);
        assert!(multi.validate_import_endpoints().unwrap_err().to_string().contains("peer 2: invalid Endpoint"));
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
    fn rejects_duplicate_singleton_keys_with_section_and_no_values() {
        let base = sample();
        let psk = keys::key_to_base64(&[3; 32]);
        for (key, section) in [("PrivateKey", "Interface"), ("MTU", "Interface"),
            ("ListenPort", "Interface"), ("PublicKey", "Peer"), ("PresharedKey", "Peer"),
            ("Endpoint", "Peer"), ("PersistentKeepalive", "Peer")] {
            let first = match key {
                "ListenPort" => format!("{key} = 0"),
                "PresharedKey" => format!("{key} = {psk}"),
                _ => base.lines().find(|line| line.starts_with(&format!("{key} ="))).unwrap().to_string(),
            };
            // Duplicate detection precedes parsing the second value.
            let text = base.replace(&format!("[{section}]"),
                &format!("[{section}]\n{first}\n{key} = do-not-echo-secret"));
            let error = WireGuardConfig::parse(&text).unwrap_err().to_string();
            assert!(error.contains(&format!("duplicate {key} in [{section}]")), "{error}");
            assert!(error.contains("line "));
            assert!(!error.contains("do-not-echo-secret"));
            assert!(!error.contains(&psk));
        }
        // Identical repeated values are ambiguous too, not just conflicting ones.
        let text = base.replace("MTU = 1280", "MTU = 1280\nMTU = 1280");
        assert!(WireGuardConfig::parse(&text).unwrap_err().to_string().contains("duplicate MTU"));
        // Reopening Interface must not bypass the scalar check.
        let text = format!("{base}\n[Interface]\nMTU = 1400\n");
        assert!(WireGuardConfig::parse(&text).unwrap_err().to_string().contains("duplicate MTU in [Interface]"));
    }

    #[test]
    fn repeated_lists_and_independent_peer_singletons_still_parse() {
        let base = sample();
        let psk = keys::key_to_base64(&[3; 32]);
        let public = keys::key_to_base64(&[4; 32]);
        let text = base.replace("MTU = 1280", "MTU = 1280\nListenPort = 0\nAddress = 10.1.0.2/32\nDNS = 9.9.9.9")
            .replace("PersistentKeepalive = 25", &format!("PersistentKeepalive = 25\nPresharedKey = {psk}\nAllowedIPs = 10.0.0.0/8"));
        let text = format!("{text}\n[Peer]\nPublicKey = {public}\nPresharedKey = {psk}\nEndpoint = localhost:12345\nPersistentKeepalive = 0\nAllowedIPs = 192.168.0.0/16\n");
        let parsed = WireGuardConfig::parse(&text).unwrap();
        assert_eq!(parsed.interface.addresses.len(), 3);
        assert_eq!(parsed.interface.dns.len(), 3);
        assert_eq!(parsed.interface.listen_port, Some(0));
        assert_eq!(parsed.peers.len(), 2);
        assert_eq!(parsed.peers[0].allowed_ips.len(), 3);
        assert_eq!(parsed.peers[1].persistent_keepalive, Some(0));
        assert_eq!(WireGuardConfig::parse(&parsed.to_conf_string()).unwrap(), parsed);
    }

    #[test]
    fn keepalive_off_disables_and_roundtrips_without_relaxing_validation() {
        let text = sample().replace("PersistentKeepalive = 25", "PersistentKeepalive = off");
        let parsed = WireGuardConfig::parse(&text).unwrap();
        assert_eq!(parsed.peers[0].persistent_keepalive, Some(0));
        let rendered = parsed.to_conf_string();
        assert!(rendered.contains("PersistentKeepalive = 0"));
        assert_eq!(WireGuardConfig::parse(&rendered).unwrap(), parsed);
        for value in ["OFF", "disabled", "on", "-1", "65536", ""] {
            let invalid = text.replace("PersistentKeepalive = off", &format!("PersistentKeepalive = {value}"));
            assert!(WireGuardConfig::parse(&invalid).is_err(), "accepted {value}");
        }
        for value in ["0", "25", "65535"] {
            let numeric = text.replace("PersistentKeepalive = off", &format!("PersistentKeepalive = {value}"));
            assert_eq!(WireGuardConfig::parse(&numeric).unwrap().peers[0].persistent_keepalive,
                Some(value.parse().unwrap()));
        }
        let duplicate = text.replace("PersistentKeepalive = off", "PersistentKeepalive = off\nPersistentKeepalive = 25");
        assert!(WireGuardConfig::parse(&duplicate).unwrap_err().to_string().contains("duplicate PersistentKeepalive in [Peer]"));
    }

    #[test]
    fn inline_hash_comments_parse_without_relaxing_validation() {
        let plain = sample();
        let annotated = plain.lines().map(|line| {
            if line.is_empty() { "# between sections".to_string() }
            else { format!("{line} # comment with = and [Peer]") }
        }).collect::<Vec<_>>().join("\r\n");
        let annotated = format!("\u{feff}# heading\r\n; whole-line semicolon\r\n{annotated}\r\n");
        let parsed = WireGuardConfig::parse(&annotated).unwrap();
        assert_eq!(parsed, WireGuardConfig::parse(&plain).unwrap());
        let rendered = parsed.to_conf_string();
        assert!(!rendered.contains('#'));
        assert_eq!(WireGuardConfig::parse(&rendered).unwrap(), parsed);
        for (old, new, expected) in [
            ("MTU = 1280", "MTU = 1280 # first\nMTU = 1400 # second", "duplicate MTU in [Interface]"),
            ("PersistentKeepalive = 25", "PersistentKeepalive = invalid # 25", "bad PersistentKeepalive"),
            ("PersistentKeepalive = 25", "PersistentKeepalive = # empty", "bad PersistentKeepalive"),
            ("PersistentKeepalive = 25", "PersistentKeepalive = 25 ; not an inline comment", "bad PersistentKeepalive"),
            ("MTU = 1280", "PostUp = do-not-echo-private-value # note", "unsupported wg-quick directive PostUp"),
            ("MTU = 1280", "UnknownSetting = do-not-echo-private-value # note", "unknown Interface key UnknownSetting"),
        ] {
            let error = WireGuardConfig::parse(&plain.replace(old, new)).unwrap_err().to_string();
            assert!(error.contains(expected), "{error}");
            assert!(!error.contains("do-not-echo-private-value"));
        }
    }

    #[test]
    fn bare_ip_values_normalize_to_host_prefixes_without_relaxing_validation() {
        let text = sample()
            .replace("Address = 10.0.0.2/32, fd00::2/128", "Address = 10.0.0.2, fd00::2 # hosts\nAddress = 10.1.0.2/24")
            .replace("AllowedIPs = 0.0.0.0/0, ::/0", "AllowedIPs = 192.0.2.1, 2001:db8::1 # hosts\nAllowedIPs = 10.0.0.0/8");
        let parsed = WireGuardConfig::parse(&text).unwrap();
        assert_eq!(parsed.interface.addresses.iter().map(ToString::to_string).collect::<Vec<_>>(),
            ["10.0.0.2/32", "fd00::2/128", "10.1.0.2/24"]);
        assert_eq!(parsed.peers[0].allowed_ips.iter().map(ToString::to_string).collect::<Vec<_>>(),
            ["192.0.2.1/32", "2001:db8::1/128", "10.0.0.0/8"]);
        let normalized = parsed.to_conf_string();
        assert!(normalized.contains("Address = 10.0.0.2/32, fd00::2/128"));
        assert!(normalized.contains("AllowedIPs = 192.0.2.1/32, 2001:db8::1/128"));
        assert_eq!(WireGuardConfig::parse(&normalized).unwrap(), parsed);
        for value in ["999.1.2.3", "fd00:::2", "private.example", "10.0.0.2/33", "fd00::2/129", "10.0.0.2/", "[fd00::2]"] {
            for key in ["Address", "AllowedIPs"] {
                let invalid = sample().lines().map(|line| {
                    if line.starts_with(&format!("{key} =")) { format!("{key} = {value}") }
                    else { line.to_string() }
                }).collect::<Vec<_>>().join("\n");
                assert!(WireGuardConfig::parse(&invalid).is_err(), "accepted {key} = {value}");
            }
        }
        assert!(WireGuardConfig::parse(&text.replace("MTU = 1280", "MTU = 1279")).is_err());
    }

    #[test]
    fn rejects_list_gaps_but_preserves_legacy_empty_allowed_ips() {
        for (key, good) in [("Address", "10.0.0.2"), ("DNS", "1.1.1.1"), ("AllowedIPs", "10.0.0.0/8")] {
            for value in [format!(", {good}"), format!("{good},"), format!("{good}, ,{good}"), ",".to_string(), " , # hidden".to_string()] {
                let invalid = sample().lines().map(|line| {
                    if line.starts_with(&format!("{key} =")) { format!("{key} = {value}") }
                    else { line.to_string() }
                }).collect::<Vec<_>>().join("\n");
                let error = WireGuardConfig::parse(&invalid).unwrap_err().to_string();
                assert!(error.contains(&format!("empty {key} list entry")), "{error}");
                assert!(error.contains("line "));
                assert!(!error.contains(good));
            }
        }
        for key in ["Address", "DNS"] {
            let invalid = sample().lines().map(|line| {
                if line.starts_with(&format!("{key} =")) { format!("{key} = # empty") }
                else { line.to_string() }
            }).collect::<Vec<_>>().join("\n");
            assert!(WireGuardConfig::parse(&invalid).is_err());
        }
        let old = sample().replace("AllowedIPs = 0.0.0.0/0, ::/0", "AllowedIPs = # legacy no routes");
        let parsed = WireGuardConfig::parse(&old).unwrap();
        assert!(parsed.peers[0].allowed_ips.is_empty());
        let normalized = parsed.to_conf_string();
        assert!(!normalized.contains("AllowedIPs ="));
        assert_eq!(WireGuardConfig::parse(&normalized).unwrap(), parsed);
        let repeated = old.replace("AllowedIPs = # legacy no routes", "AllowedIPs =\nAllowedIPs = 10.0.0.1, fd00::1\nAllowedIPs = 192.0.2.0/24");
        assert_eq!(WireGuardConfig::parse(&repeated).unwrap().peers[0].allowed_ips.len(), 3);
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
    fn rejects_unsupported_directives_without_echoing_values() {
        for key in ["PreUp", "PostUp", "PreDown", "PostDown", "Table", "SaveConfig", "FwMark"] {
            let text = sample().replace("[Interface]", &format!("[Interface]\n{key} = do-not-echo-private-value"));
            let error = WireGuardConfig::parse(&text).unwrap_err().to_string();
            assert!(error.contains("unsupported wg-quick directive"), "{key}: {error}");
            assert!(error.contains(key));
            assert!(!error.contains("do-not-echo-private-value"));
        }
    }

    #[test]
    fn rejects_unsupported_dns_without_silent_partial_import() {
        for dns in ["internal.private", "1.1.1.1, internal.private", "not-an-ip"] {
            let text = sample().replace("DNS = 1.1.1.1, 2606:4700:4700::1111", &format!("DNS = {dns}"));
            let error = WireGuardConfig::parse(&text).unwrap_err().to_string();
            assert!(error.contains("DNS supports only IP addresses"));
            assert!(!error.contains("internal.private"));
            assert!(!error.contains("not-an-ip"));
        }
    }

    #[test]
    fn validates_mtu_for_profile_address_families() {
        assert!(WireGuardConfig::parse(&sample().replace("MTU = 1280", "MTU = 1279")).is_err());
        let ipv4 = sample().replace(", fd00::2/128", "").replace("MTU = 1280", "MTU = 576");
        assert!(WireGuardConfig::parse(&ipv4).is_ok());
        assert!(WireGuardConfig::parse(&ipv4.replace("MTU = 576", "MTU = 575")).is_err());
    }

    #[test]
    fn rejects_missing_peer() {
        let priv_b64 = keys::key_to_base64(&keys::generate_private_key());
        let text = format!("[Interface]\nPrivateKey = {priv_b64}\nAddress = 10.0.0.2/32\n");
        assert!(WireGuardConfig::parse(&text).is_err());
    }

    #[test]
    fn numeric_errors_explain_ranges_without_echoing_values() {
        for (key, existing, expected) in [
            ("MTU", "MTU = 1280", "bad MTU; expected an integer from 576 to 65535 (1280 minimum with an IPv6 interface address)"),
            ("ListenPort", "MTU = 1280", "bad ListenPort; expected an integer from 0 to 65535"),
            ("PersistentKeepalive", "PersistentKeepalive = 25", "bad PersistentKeepalive; expected off or an integer from 0 to 65535 seconds"),
        ] {
            for value in ["-17", "do-not-echo-private-value", "65536", "12.5", ""] {
                let replacement = if key == "ListenPort" { format!("MTU = 1280\nListenPort = {value}") }
                    else { format!("{key} = {value}") };
                let text = sample().replace(existing, &replacement);
                let line = text.lines().position(|line| line.starts_with(&format!("{key} ="))).unwrap() + 1;
                let error = WireGuardConfig::parse(&text).unwrap_err().to_string();
                assert!(error.contains(&format!("line {line}: {expected}")), "{error}");
                if !value.is_empty() { assert!(!error.contains(value), "{error}"); }
            }
        }
        for value in ["0", "65535"] {
            let text = sample().replace("MTU = 1280", &format!("MTU = 1280\nListenPort = {value}"))
                .replace("PersistentKeepalive = 25", &format!("PersistentKeepalive = {value}"));
            let parsed = WireGuardConfig::parse(&text).unwrap();
            assert_eq!(parsed.interface.listen_port, Some(value.parse().unwrap()));
            assert_eq!(parsed.peers[0].persistent_keepalive, Some(value.parse().unwrap()));
        }
        assert_eq!(WireGuardConfig::parse(&sample().replace("PersistentKeepalive = 25", "PersistentKeepalive = off")).unwrap().peers[0].persistent_keepalive, Some(0));
        assert_eq!(WireGuardConfig::parse(&sample().replace("MTU = 1280", "MTU = 65535")).unwrap().interface.mtu, Some(65535));
        for (base, valid, invalid, minimum) in [
            (sample(), "1280", "1279", "1280"),
            (sample().replace(", fd00::2/128", ""), "576", "575", "576"),
        ] {
            assert!(WireGuardConfig::parse(&base.replace("MTU = 1280", &format!("MTU = {valid}"))).is_ok());
            for value in [invalid, "0"] {
                let error = WireGuardConfig::parse(&base.replace("MTU = 1280", &format!("MTU = {value}"))).unwrap_err().to_string();
                assert!(error.contains(&format!("MTU must be at least {minimum} for this profile's address families")));
                assert!(!error.contains("bad MTU;"));
            }
        }
    }


    #[test]
    fn address_errors_explain_syntax_without_echoing_entries() {
        for (key, existing, field) in [
            ("Address", "Address = 10.0.0.2/32, fd00::2/128", "Address"),
            ("AllowedIPs", "AllowedIPs = 0.0.0.0/0, ::/0", "AllowedIP"),
        ] {
            for value in ["do-not-echo-private-entry", "private.internal", "192.0.2.1/33", "2001:db8::1/129", "192.0.2.1/secret", "10.0.0.2, do-not-echo-private-entry"] {
                let text = sample().replace(existing, &format!("{key} = {value}"));
                let line = text.lines().position(|line| line.starts_with(&format!("{key} ="))).unwrap() + 1;
                let error = WireGuardConfig::parse(&text).unwrap_err().to_string();
                assert!(error.contains(&format!("line {line}: bad {field}; expected an IP address or CIDR prefix")), "{error}");
                assert!(!error.contains(value), "{error}");
                for entry in value.split(',').map(str::trim) { assert!(!error.contains(entry), "{error}"); }
            }
            for value in ["192.0.2.7", "2001:db8::7", "192.0.2.7/32", "2001:db8::7/128", "192.0.2.0/24, 2001:db8::/64"] {
                let text = sample().replace(existing, &format!("{key} = {value}"));
                let parsed = WireGuardConfig::parse(&text).unwrap();
                let expected = value.split(',').map(str::trim).map(|item| parse_ip_prefix(item).unwrap()).collect::<Vec<_>>();
                if key == "Address" { assert_eq!(parsed.interface.addresses, expected); }
                else { assert_eq!(parsed.peers[0].allowed_ips, expected); }
                assert_eq!(WireGuardConfig::parse(&parsed.to_conf_string()).unwrap(), parsed);
            }
        }
    }

}
