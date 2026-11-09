//! Split tunneling: subtract excluded CIDRs from a peer's AllowedIPs.
//!
//! WireGuard has no "exclude this range" concept; the standard approach is
//! to compute the AllowedIPs set that covers everything EXCEPT the excluded
//! prefixes. Subtraction splits base prefixes at bit boundaries, so the
//! result is always an exact, minimal cover.

use std::net::IpAddr;

use ipnet::IpNet;

use crate::CoreError;

fn bits_and_len(net: IpNet) -> (u128, u8, u8) {
    match net.network() {
        IpAddr::V4(a) => (u32::from(a) as u128, net.prefix_len(), 32),
        IpAddr::V6(a) => (u128::from(a), net.prefix_len(), 128),
    }
}

fn from_bits(v: u128, plen: u8, v6: bool) -> Option<IpNet> {
    if v6 {
        IpNet::new(IpAddr::V6(std::net::Ipv6Addr::from(v)), plen).ok()
    } else {
        IpNet::new(IpAddr::V4(std::net::Ipv4Addr::from(v as u32)), plen).ok()
    }
}

fn mask(v: u128, plen: u8, max: u8) -> u128 {
    if plen == 0 {
        0
    } else {
        v & (u128::MAX << (max - plen))
    }
}

/// Subtract `excl` from `base`. Both must be the same address family.
fn subtract_one(base: IpNet, excl: IpNet) -> Vec<IpNet> {
    let (b_addr, b_plen, b_max) = bits_and_len(base);
    let (e_addr, e_plen, e_max) = bits_and_len(excl);
    if b_max != e_max {
        return vec![base]; // different families never overlap
    }
    let max = b_max;
    let check = b_plen.min(e_plen);
    if mask(b_addr, check, max) != mask(e_addr, check, max) {
        return vec![base]; // disjoint
    }
    if e_plen <= b_plen {
        return Vec::new(); // exclusion covers the whole base
    }
    // excl sits strictly inside base: split base and recurse.
    let half = b_plen + 1;
    let lo = from_bits(b_addr, half, max == 128);
    let hi_bit = 1u128 << (max - half);
    let hi = from_bits(b_addr | hi_bit, half, max == 128);
    let mut out = Vec::new();
    for part in [lo, hi].into_iter().flatten() {
        out.extend(subtract_one(part, excl));
    }
    out
}

/// Remove every exclusion from each allowed prefix. Mixed families are fine;
/// exclusions only affect their own family.
pub fn apply_exclusions(allowed: &[IpNet], exclusions: &[IpNet]) -> Vec<IpNet> {
    let mut nets: Vec<IpNet> = allowed.to_vec();
    for excl in exclusions {
        let mut next = Vec::new();
        for net in nets {
            next.extend(subtract_one(net, *excl));
        }
        nets = next;
    }
    nets
}

/// Parse a comma- or whitespace-separated CIDR list.
pub fn parse_exclusions(text: &str) -> Result<Vec<IpNet>, CoreError> {
    let mut out = Vec::new();
    for part in text.split([',', ' ', '\n', '\t']) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let net: IpNet = part
            .parse()
            .map_err(|_| CoreError::Parse(format!("invalid CIDR '{part}'")))?;
        out.push(net);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn covers(nets: &[IpNet], ip: &str) -> bool {
        let addr: IpAddr = ip.parse().unwrap();
        nets.iter().any(|n| n.contains(&addr))
    }

    #[test]
    fn subtract_disjoint_keeps_base() {
        let base: IpNet = "10.0.0.0/8".parse().unwrap();
        let excl: IpNet = "192.168.0.0/16".parse().unwrap();
        assert_eq!(subtract_one(base, excl), vec![base]);
    }

    #[test]
    fn subtract_covering_returns_empty() {
        let base: IpNet = "10.0.0.0/8".parse().unwrap();
        let excl: IpNet = "0.0.0.0/0".parse().unwrap();
        assert!(subtract_one(base, excl).is_empty());
        assert!(subtract_one(base, base).is_empty());
    }

    #[test]
    fn subtract_inner_prefix_covers_rest() {
        let base: IpNet = "0.0.0.0/0".parse().unwrap();
        let excl: IpNet = "10.0.0.0/8".parse().unwrap();
        let out = subtract_one(base, excl);
        assert!(covers(&out, "9.255.255.255"));
        assert!(covers(&out, "11.0.0.1"));
        assert!(covers(&out, "1.2.3.4"));
        assert!(!covers(&out, "10.0.0.1"));
        assert!(!covers(&out, "10.255.255.254"));
        // no overlaps: total covered space must equal 2^32 - 2^24
        let total: u128 = out.iter().map(|n| 1u128 << (32 - n.prefix_len())).sum();
        assert_eq!(total, (1u128 << 32) - (1u128 << 24));
    }

    #[test]
    fn subtract_multiple_exclusions() {
        let base: IpNet = "0.0.0.0/0".parse().unwrap();
        let excls = parse_exclusions("192.168.0.0/16, 10.0.0.0/8").unwrap();
        let out = apply_exclusions(&[base], &excls);
        assert!(!covers(&out, "192.168.1.10"));
        assert!(!covers(&out, "10.4.5.6"));
        assert!(covers(&out, "172.16.0.5"));
        assert!(covers(&out, "8.8.8.8"));
    }

    #[test]
    fn subtract_v6() {
        let base: IpNet = "::/0".parse().unwrap();
        let excl: IpNet = "2001:db8::/32".parse().unwrap();
        let out = subtract_one(base, excl);
        assert!(!covers(&out, "2001:db8::1"));
        assert!(covers(&out, "2001:db9::1"));
        assert!(covers(&out, "fd00::1"));
    }

    #[test]
    fn mixed_families_do_not_interfere() {
        let v4: IpNet = "0.0.0.0/0".parse().unwrap();
        let v6: IpNet = "::/0".parse().unwrap();
        let excl: IpNet = "10.0.0.0/8".parse().unwrap();
        let out = apply_exclusions(&[v4, v6], &[excl]);
        assert!(out.contains(&v6));
        assert!(covers(&out, "11.0.0.1"));
        assert!(!covers(&out, "10.1.1.1"));
    }

    #[test]
    fn parse_bad_cidr_errors() {
        assert!(parse_exclusions("not-a-cidr").is_err());
        assert!(parse_exclusions("").unwrap().is_empty());
    }
}
