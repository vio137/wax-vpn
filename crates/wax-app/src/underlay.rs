//! Read-only route fingerprint groundwork. Test-only until a complete local
//! underlay policy is ready. Private in-memory values, never diagnostic output.
use windows_sys::Win32::{NetworkManagement::IpHelper::{FreeMibTable, GetIpForwardTable2, GetUnicastIpAddressTable, MIB_IPFORWARD_ROW2, MIB_IPFORWARD_TABLE2, MIB_UNICASTIPADDRESS_ROW, MIB_UNICASTIPADDRESS_TABLE}, Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC}};

// Deliberately no Debug or Serialize: this holds local gateway addresses.
#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct DefaultRoute {
    family: u16,
    luid: u64,
    metric: u32,
    gateway: [u8;16],
    scope: u32,
}
// Also intentionally lacks Debug/Serialize. Only addresses on a current
// non-waX default-route interface count, not unrelated virtual adapters.
#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct Address {
    family: u16,
    luid: u64,
    bytes: [u8;16],
    scope: u32,
    prefix: u8,
    dad: i32,
    skip_source: bool,
}
#[derive(Eq, PartialEq)]
pub struct Snapshot(Vec<DefaultRoute>, Vec<Address>);
struct Table(*mut MIB_IPFORWARD_TABLE2);
impl Drop for Table { fn drop(&mut self) { unsafe { FreeMibTable(self.0.cast()); } } }

fn fingerprint(rows: &[MIB_IPFORWARD_ROW2], own_luid: u64) -> Snapshot {
    let mut routes = Vec::new();
    for row in rows {
        let luid = unsafe { row.InterfaceLuid.Value };
        if luid == 0 || luid == own_luid || row.DestinationPrefix.PrefixLength != 0 || row.Loopback != 0 { continue; }
        let family = unsafe { row.NextHop.si_family };
        let mut gateway = [0;16];
        let scope = match family {
            AF_INET => { gateway[..4].copy_from_slice(&unsafe { row.NextHop.Ipv4.sin_addr.S_un.S_addr }.to_ne_bytes()); 0 },
            AF_INET6 => { gateway = unsafe { row.NextHop.Ipv6.sin6_addr.u.Byte }; unsafe { row.NextHop.Ipv6.Anonymous.sin6_scope_id } },
            _ => continue,
        };
        routes.push(DefaultRoute { family, luid, metric:row.Metric, gateway, scope });
    }
    routes.sort_unstable(); routes.dedup(); Snapshot(routes,Vec::new())
}

fn addresses(rows: &[MIB_UNICASTIPADDRESS_ROW], defaults: &[DefaultRoute]) -> Vec<Address> {
    let mut result=Vec::new();
    for row in rows {
        let luid=unsafe { row.InterfaceLuid.Value };
        let family=unsafe { row.Address.si_family };
        if !defaults.iter().any(|r| r.luid==luid && r.family==family) { continue; }
        let mut bytes=[0;16];
        let scope=match family {
            AF_INET => { bytes[..4].copy_from_slice(&unsafe { row.Address.Ipv4.sin_addr.S_un.S_addr }.to_ne_bytes()); 0 },
            AF_INET6 => { bytes=unsafe { row.Address.Ipv6.sin6_addr.u.Byte }; unsafe { row.Address.Ipv6.Anonymous.sin6_scope_id } },
            _ => continue,
        };
        result.push(Address { family,luid,bytes,scope,prefix:row.OnLinkPrefixLength,dad:row.DadState,skip_source:row.SkipAsSource!=0 });
    }
    result.sort_unstable(); result.dedup(); result
}

impl Snapshot {
    pub fn read(own_luid: u64) -> Result<Self, &'static str> {
        if own_luid == 0 { return Err("underlay_identity"); }
        let mut table = std::ptr::null_mut();
        if unsafe { GetIpForwardTable2(AF_UNSPEC, &mut table) } != 0 || table.is_null() { return Err("underlay_routes"); }
        let table = Table(table);
        // Typed field address handles SDK padding before the first row.
        // Rows are the API's variable-length allocation, not the one-row Rust
        // placeholder array. Never create a reference to an out-of-bounds array.
        let rows = unsafe {
            std::slice::from_raw_parts(std::ptr::addr_of!((*table.0).Table).cast::<MIB_IPFORWARD_ROW2>(), (*table.0).NumEntries as usize)
        };
        let mut snapshot=fingerprint(rows, own_luid);
        let mut raw=std::ptr::null_mut();
        if unsafe { GetUnicastIpAddressTable(AF_UNSPEC,&mut raw) } != 0 || raw.is_null() { return Err("underlay_addresses"); }
        struct Addresses(*mut MIB_UNICASTIPADDRESS_TABLE);
        impl Drop for Addresses { fn drop(&mut self) { unsafe { FreeMibTable(self.0.cast()); } } }
        let address_table=Addresses(raw);
        let rows=unsafe { std::slice::from_raw_parts(std::ptr::addr_of!((*address_table.0).Table).cast::<MIB_UNICASTIPADDRESS_ROW>(),(*address_table.0).NumEntries as usize) };
        snapshot.1=addresses(rows,&snapshot.0);
        Ok(snapshot)
    }
    // This only describes default-route changes, NOT tunnel loss or Internet
    // reachability. Interface metrics/endpoint-specific routes remain future work.
    pub fn differs(&self, other: &Self) -> bool { self != other }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn route(luid:u64, metric:u32, gateway:u32) -> MIB_IPFORWARD_ROW2 {
        let mut r:MIB_IPFORWARD_ROW2=unsafe { std::mem::zeroed() };
        r.InterfaceLuid.Value=luid; r.Metric=metric;
        r.NextHop.Ipv4.sin_family=AF_INET; r.NextHop.Ipv4.sin_addr.S_un.S_addr=gateway;
        r
    }
    #[test]
    fn own_and_nondefault_routes_do_not_change_fingerprint() {
        let base=route(22,5,7); let own=route(11,1,8);
        let mut specific=route(33,8,9); specific.DestinationPrefix.PrefixLength=24;
        assert!(!fingerprint(&[base],11).differs(&fingerprint(&[specific,own,base],11)));
    }
    #[test]
    fn reorder_duplicate_and_transient_age_are_ignored_but_gateway_metric_and_luid_change() {
        let a=route(22,5,7); let b=route(33,8,9); let mut aged=a; aged.Age=999;
        let base=fingerprint(&[a,b],11);
        assert!(!base.differs(&fingerprint(&[b,aged,a],11)));
        for changed in [route(22,5,8),route(22,6,7),route(44,5,7)] {
            assert!(base.differs(&fingerprint(&[changed,b],11)));
        }
    }
    #[test]
    fn address_comparison_ignores_other_interfaces_and_lifetimes_but_tracks_source_changes() {
        let defaults=fingerprint(&[route(22,5,7)],11);
        let mut a:MIB_UNICASTIPADDRESS_ROW=unsafe { std::mem::zeroed() };
        a.InterfaceLuid.Value=22; a.Address.Ipv4.sin_family=AF_INET;
        a.Address.Ipv4.sin_addr.S_un.S_addr=1; a.OnLinkPrefixLength=24;
        let baseline=addresses(&[a],&defaults.0);
        let mut aged=a; aged.ValidLifetime=555; aged.PreferredLifetime=444;
        let mut unrelated=a; unrelated.InterfaceLuid.Value=33;
        assert!(baseline==addresses(&[unrelated,aged,a],&defaults.0));
        for field in 0..4 {
            let mut changed=a;
            match field { 0 => changed.Address.Ipv4.sin_addr.S_un.S_addr=2,
                1 => changed.OnLinkPrefixLength=25, 2 => changed.DadState=3,
                _ => changed.SkipAsSource=1 }
            assert!(baseline!=addresses(&[changed],&defaults.0));
        }
    }
    #[test]
    fn windows_snapshot_read_has_no_route_mutation_or_network_probe() {
        let first=Snapshot::read(1).expect("local route read");
        let second=Snapshot::read(1).expect("repeated local route read");
        let _=first.differs(&second); // Runner activity can change routes.
        assert!(Snapshot::read(0).is_err());
    }
}
