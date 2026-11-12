//! Explicitly allowlisted loaded-config values. No I/O, key formatting or live claims.
use wax_core::conf::WireGuardConfig;

pub fn rows(config: &WireGuardConfig) -> Vec<(&'static str, String)> {
    fn joined(items: impl Iterator<Item = String>, empty: &str) -> String {
        let text = items.collect::<Vec<_>>().join(", ");
        if text.is_empty() { empty.into() } else { text }
    }
    let routes = config.peers.iter().flat_map(|p| &p.allowed_ips);
    let ipv4 = routes.clone().any(|ip| ip.addr().is_ipv4() && ip.prefix_len() == 0);
    let ipv6 = routes.clone().any(|ip| ip.addr().is_ipv6() && ip.prefix_len() == 0);
    let defaults = match (ipv4, ipv6) {
        (true, true) => "IPv4 and IPv6",
        (true, false) => "IPv4 only",
        (false, true) => "IPv6 only",
        (false, false) => "None",
    };
    vec![
        ("Addresses", joined(config.interface.addresses.iter().map(ToString::to_string), "Not set")),
        ("DNS", joined(config.interface.dns.iter().map(ToString::to_string), "Not set")),
        ("MTU", config.interface.mtu.map(|v| v.to_string()).unwrap_or_else(|| "Automatic".into())),
        ("Peers", config.peers.len().to_string()),
        ("Route entries", config.peers.iter().map(|p| p.allowed_ips.len()).sum::<usize>().to_string()),
        ("Explicit /0 defaults", defaults.into()),
    ]
}

/// Bounded configured route preview. Peer positions are local labels, not keys.
pub fn route_preview(config: &WireGuardConfig) -> (Vec<String>, usize) {
    const LIMIT: usize = 64;
    let total = config.peers.iter().map(|peer| peer.allowed_ips.len()).sum::<usize>();
    let routes = config.peers.iter().enumerate()
        .flat_map(|(peer, config)| config.allowed_ips.iter().map(move |cidr| format!("Peer {}: {cidr}", peer + 1)))
        .take(LIMIT).collect();
    (routes, total.saturating_sub(LIMIT))
}

/// Clipboard export is explicit and includes the same allowlist and limits.
pub fn clipboard_text(name: &str, rows: &[(&str, String)]) -> String {
    let mut text = format!("Loaded config: {name}\nConfigured values only. Not a live connection, routing or leak check. Keys are not shown.\n");
    for (label, value) in rows {
        text.push_str(&format!("{label}: {value}\n"));
    }
    text
}

/// Holds only allowlisted display strings, never a config or key material.
/// The caller invalidates on every successful loaded-profile replacement.
#[derive(Default)]
pub struct DisplayCache {
    selected: Option<usize>,
    rows: Vec<(&'static str, String)>,
}
impl DisplayCache {
    pub fn invalidate(&mut self) {
        self.selected = None;
        self.rows.clear();
    }
    pub fn get(&mut self, selected: usize, config: &WireGuardConfig) -> &[(&'static str, String)] {
        if self.selected != Some(selected) {
            self.rows = rows(config);
            self.selected = Some(selected);
        }
        &self.rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn route_preview_is_ordered_bounded_and_contains_only_peer_positions_and_cidrs() {
        let mut config = WireGuardConfig::parse("[Interface]\nPrivateKey = AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=\n[Peer]\nPublicKey = AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI=\nAllowedIPs = 0.0.0.0/0, ::/0\nEndpoint = hidden.invalid:51820\n").unwrap();
        let mut second = config.peers[0].clone();
        second.allowed_ips = vec!["10.4.0.0/24".parse().unwrap()];
        config.peers.push(second);
        assert_eq!(route_preview(&config), (vec!["Peer 1: 0.0.0.0/0".into(), "Peer 1: ::/0".into(), "Peer 2: 10.4.0.0/24".into()], 0));
        let text = format!("{:?}", route_preview(&config));
        for secret in ["AQEBAQE", "AgICAgI", "hidden.invalid", "Endpoint", "Key"] { assert!(!text.contains(secret)); }
        config.peers[0].allowed_ips = (0..70).map(|i| format!("10.5.{i}.0/24").parse().unwrap()).collect();
        let (rows, hidden) = route_preview(&config);
        assert_eq!(rows.len(), 64); assert_eq!(hidden, 7);
        assert_eq!(rows.last().unwrap(), "Peer 1: 10.5.63.0/24");
        config.peers.clear(); assert_eq!(route_preview(&config), (vec![], 0));
    }

    #[test]
    fn configured_values_are_explicit_and_never_include_keys() {
        let config = WireGuardConfig::parse("[Interface]\nPrivateKey = AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=\nAddress = 10.4.0.2/32, fd00::2/128\nDNS = 1.1.1.1, 2606:4700:4700::1111\nMTU = 1380\n[Peer]\nPublicKey = AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI=\nPresharedKey = AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM=\nAllowedIPs = 0.0.0.0/0, ::/0\nEndpoint = example.invalid:51820\n").unwrap();
        assert_eq!(rows(&config), vec![("Addresses", "10.4.0.2/32, fd00::2/128".into()), ("DNS", "1.1.1.1, 2606:4700:4700::1111".into()), ("MTU", "1380".into()), ("Peers", "1".into()), ("Route entries", "2".into()), ("Explicit /0 defaults", "IPv4 and IPv6".into())]);
        let text = format!("{:?}", rows(&config));
        for hidden in ["AQEBAQE", "AgICAgI", "AwMDAwM", "PrivateKey", "PublicKey", "PresharedKey", "example.invalid"] { assert!(!text.contains(hidden)); }
        let mut unset = config.clone();
        unset.interface.addresses.clear();
        unset.interface.dns.clear();
        unset.interface.mtu = None;
        unset.peers.clear();
        assert_eq!(rows(&unset), vec![("Addresses", "Not set".into()), ("DNS", "Not set".into()), ("MTU", "Automatic".into()), ("Peers", "0".into()), ("Route entries", "0".into()), ("Explicit /0 defaults", "None".into())]);
        let mut multiple = config.clone();
        multiple.peers.push(config.peers[0].clone());
        assert_eq!(rows(&multiple)[3..5], [("Peers", "2".into()), ("Route entries", "4".into())]);
        assert_eq!(config, WireGuardConfig::parse(&config.to_conf_string()).unwrap());
    }
    #[test]
    fn explicit_default_families_check_all_peers_and_do_not_infer_split_routes() {
        let mut config = WireGuardConfig::parse("[Interface]\nPrivateKey = AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=\n[Peer]\nPublicKey = AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI=\nAllowedIPs = 0.0.0.0/0\n").unwrap();
        assert_eq!(rows(&config).last().unwrap().1, "IPv4 only");
        config.peers[0].allowed_ips = vec!["::/0".parse().unwrap()];
        assert_eq!(rows(&config).last().unwrap().1, "IPv6 only");
        let mut peer = config.peers[0].clone();
        peer.allowed_ips = vec!["0.0.0.0/0".parse().unwrap()];
        config.peers.push(peer);
        assert_eq!(rows(&config).last().unwrap().1, "IPv4 and IPv6");
        config.peers[0].allowed_ips = vec!["0.0.0.0/1".parse().unwrap(), "128.0.0.0/1".parse().unwrap()];
        config.peers[1].allowed_ips = vec!["::/1".parse().unwrap(), "8000::/1".parse().unwrap()];
        assert_eq!(rows(&config).last().unwrap().1, "None");
    }

    #[test]
    fn display_cache_reuses_storage_and_refreshes_on_selection_and_reload() {
        let config = WireGuardConfig::parse("[Interface]\nPrivateKey = AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=\nAddress = 10.4.0.2/32\n[Peer]\nPublicKey = AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI=\nAllowedIPs = 10.4.0.0/24\n").unwrap();
        let mut other = config.clone();
        other.interface.addresses = vec!["fd00::2/128".parse().unwrap()];
        let mut cache = DisplayCache::default();
        assert_eq!(cache.get(0, &config), rows(&config));
        let storage = cache.rows.as_ptr();
        let address = cache.rows[0].1.as_ptr();
        for _ in 0..20 {
            assert_eq!(cache.get(0, &config), rows(&config));
            assert_eq!(cache.rows.as_ptr(), storage);
            assert_eq!(cache.rows[0].1.as_ptr(), address);
        }
        assert_eq!(cache.get(1, &other), rows(&other));
        assert_eq!(cache.get(0, &config), rows(&config));
        cache.invalidate();
        assert!(cache.rows.is_empty());
        assert_eq!(cache.get(0, &other), rows(&other));
        cache.invalidate();
        assert_eq!(cache.get(0, &config), rows(&config));
    }

    #[test]
    fn clipboard_export_uses_allowlisted_rows_and_explicit_limits() {
        let config = WireGuardConfig::parse("[Interface]\nPrivateKey = AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=\nAddress = 10.4.0.2/32\n[Peer]\nPublicKey = AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI=\nAllowedIPs = 0.0.0.0/0\nEndpoint = private-endpoint.invalid:51820\n").unwrap();
        let text = clipboard_text("Office", &rows(&config));
        assert!(text.starts_with("Loaded config: Office\nConfigured values only."));
        assert!(text.contains("Not a live connection, routing or leak check. Keys are not shown."));
        assert!(text.contains("Addresses: 10.4.0.2/32\n"));
        assert!(text.contains("Explicit /0 defaults: IPv4 only\n"));
        for hidden in ["AQEBAQE", "AgICAgI", "PrivateKey", "PublicKey", "PresharedKey", "private-endpoint.invalid"] { assert!(!text.contains(hidden)); }
    }

}
