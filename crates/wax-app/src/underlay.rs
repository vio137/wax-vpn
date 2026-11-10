//! Read-only route fingerprint groundwork. Test-only until a complete local
//! underlay policy is ready. Private in-memory values, never diagnostic output.
use windows_sys::Win32::{NetworkManagement::IpHelper::{FreeMibTable, GetIpForwardTable2, MIB_IPFORWARD_ROW2, MIB_IPFORWARD_TABLE2}, Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC}};

// Deliberately no Debug or Serialize: this holds local gateway addresses.
#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct DefaultRoute {
    family: u16,
    luid: u64,
    metric: u32,
    gateway: [u8;16],
    scope: u32,
}
#[derive(Eq, PartialEq)]
pub struct Snapshot(Vec<DefaultRoute>);
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
    routes.sort_unstable(); routes.dedup(); Snapshot(routes)
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
        Ok(fingerprint(rows, own_luid))
    }
    // This only describes default-route changes, NOT tunnel loss or Internet
    // reachability. Address-only changes/interface metrics remain future work.
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
    fn windows_snapshot_read_has_no_route_mutation_or_network_probe() {
        let first=Snapshot::read(1).expect("local route read");
        let second=Snapshot::read(1).expect("repeated local route read");
        let _=first.differs(&second); // Runner activity can change routes.
        assert!(Snapshot::read(0).is_err());
    }
}
