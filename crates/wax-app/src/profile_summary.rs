//! Explicitly allowlisted loaded-config values. No I/O, key formatting or live claims.
use wax_core::conf::WireGuardConfig;

pub fn rows(config: &WireGuardConfig) -> Vec<(&'static str, String)> {
    fn joined(items: impl Iterator<Item = String>, empty: &str) -> String {
        let text = items.collect::<Vec<_>>().join(", ");
        if text.is_empty() { empty.into() } else { text }
    }
    vec![
        ("Addresses", joined(config.interface.addresses.iter().map(ToString::to_string), "Not set")),
        ("DNS", joined(config.interface.dns.iter().map(ToString::to_string), "Not set")),
        ("MTU", config.interface.mtu.map(|v| v.to_string()).unwrap_or_else(|| "Automatic".into())),
        ("Peers", config.peers.len().to_string()),
        ("Route entries", config.peers.iter().map(|p| p.allowed_ips.len()).sum::<usize>().to_string()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn configured_values_are_explicit_and_never_include_keys() {
        let config = WireGuardConfig::parse("[Interface]\nPrivateKey = AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=\nAddress = 10.4.0.2/32, fd00::2/128\nDNS = 1.1.1.1, 2606:4700:4700::1111\nMTU = 1380\n[Peer]\nPublicKey = AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI=\nPresharedKey = AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM=\nAllowedIPs = 0.0.0.0/0, ::/0\nEndpoint = example.invalid:51820\n").unwrap();
        assert_eq!(rows(&config), vec![("Addresses", "10.4.0.2/32, fd00::2/128".into()), ("DNS", "1.1.1.1, 2606:4700:4700::1111".into()), ("MTU", "1380".into()), ("Peers", "1".into()), ("Route entries", "2".into())]);
        let text = format!("{:?}", rows(&config));
        for hidden in ["AQEBAQE", "AgICAgI", "AwMDAwM", "PrivateKey", "PublicKey", "PresharedKey", "example.invalid"] { assert!(!text.contains(hidden)); }
        let mut unset = config.clone();
        unset.interface.addresses.clear();
        unset.interface.dns.clear();
        unset.interface.mtu = None;
        unset.peers.clear();
        assert_eq!(rows(&unset), vec![("Addresses", "Not set".into()), ("DNS", "Not set".into()), ("MTU", "Automatic".into()), ("Peers", "0".into()), ("Route entries", "0".into())]);
        let mut multiple = config.clone();
        multiple.peers.push(config.peers[0].clone());
        assert_eq!(rows(&multiple)[3..], [("Peers", "2".into()), ("Route entries", "4".into())]);
        assert_eq!(config, WireGuardConfig::parse(&config.to_conf_string()).unwrap());
    }
}
