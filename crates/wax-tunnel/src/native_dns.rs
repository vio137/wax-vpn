//! Per-interface DNS through the OS API, resolved dynamically for older Windows 10.
use crate::TunnelError;
use std::net::IpAddr;
use windows_sys::Win32::Foundation::FreeLibrary;
use windows_sys::Win32::{
    NetworkManagement::{
        IpHelper::{
            ConvertInterfaceAliasToLuid, ConvertInterfaceLuidToGuid, DNS_INTERFACE_SETTINGS,
            DNS_INTERFACE_SETTINGS_VERSION1,
        },
        Ndis::NET_LUID_LH,
    },
    System::LibraryLoader::{GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32},
};

type SetDns =
    unsafe extern "system" fn(windows_sys::core::GUID, *const DNS_INTERFACE_SETTINGS) -> u32;
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

/// None means the OS API is absent. API errors must never silently select netsh.
pub(crate) fn apply(
    adapter: &str,
    servers: &[IpAddr],
    reset: bool,
) -> Option<Result<(), TunnelError>> {
    let dll = wide("iphlpapi.dll");
    let module = unsafe { LoadLibraryExW(dll.as_ptr(), 0, LOAD_LIBRARY_SEARCH_SYSTEM32) };
    if module == 0 {
        return Some(Err(TunnelError::Message(
            "Could not load Windows IP Helper for DNS".into(),
        )));
    }
    struct Library(isize);
    impl Drop for Library {
        fn drop(&mut self) {
            unsafe {
                FreeLibrary(self.0);
            }
        }
    }
    let library = Library(module);
    let address = unsafe { GetProcAddress(library.0, b"SetInterfaceDnsSettings\0".as_ptr()) }?;
    let set: SetDns = unsafe { std::mem::transmute(address) };
    Some((|| {
        let mut luid: NET_LUID_LH = unsafe { std::mem::zeroed() };
        let mut guid = windows_sys::core::GUID::from_u128(0);
        let result = unsafe { ConvertInterfaceAliasToLuid(wide(adapter).as_ptr(), &mut luid) };
        if result != 0 {
            return Err(TunnelError::Message(format!(
                "Could not find DNS interface: {result}"
            )));
        }
        let result = unsafe { ConvertInterfaceLuidToGuid(&luid, &mut guid) };
        if result != 0 {
            return Err(TunnelError::Message(format!(
                "Could not identify DNS interface: {result}"
            )));
        }
        let mut errors = Vec::new();
        for ipv6 in [false, true] {
            let names = nameservers(servers, ipv6);
            if !reset && names.is_empty() {
                continue;
            }
            // Empty static list resets this address family's DNS to automatic.
            // Runner verification must confirm the resulting DNS state.
            let mut names = wide(&names);
            let mut settings: DNS_INTERFACE_SETTINGS = unsafe { std::mem::zeroed() };
            settings.Version = DNS_INTERFACE_SETTINGS_VERSION1;
            settings.Flags = 2 | if ipv6 { 1 } else { 0 };
            settings.NameServer = names.as_mut_ptr();
            let result = unsafe { set(guid, &settings) };
            if result != 0 {
                errors.push(format!(
                    "{} DNS settings failed: {result}",
                    if ipv6 { "IPv6" } else { "IPv4" }
                ));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(TunnelError::Message(errors.join("; ")))
        }
    })())
}
fn nameservers(servers: &[IpAddr], ipv6: bool) -> String {
    servers
        .iter()
        .filter(|ip| ip.is_ipv6() == ipv6)
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dns_families_keep_order_and_do_not_mix() {
        let ips = ["1.1.1.1", "2606:4700:4700::1111", "8.8.8.8"].map(|s| s.parse().unwrap());
        assert_eq!(nameservers(&ips, false), "1.1.1.1,8.8.8.8");
        assert_eq!(nameservers(&ips, true), "2606:4700:4700::1111");
        assert_eq!(nameservers(&[], false), "");
    }
}

#[cfg(test)]
pub(crate) fn assert_configured(adapter: &str, ipv6: bool, expected: &str) {
    use windows_sys::Win32::NetworkManagement::IpHelper::GetInterfaceDnsSettings;
    let mut luid: NET_LUID_LH = unsafe { std::mem::zeroed() };
    let mut guid = windows_sys::core::GUID::from_u128(0);
    assert_eq!(
        unsafe { ConvertInterfaceAliasToLuid(wide(adapter).as_ptr(), &mut luid) },
        0
    );
    assert_eq!(unsafe { ConvertInterfaceLuidToGuid(&luid, &mut guid) }, 0);
    let mut settings: DNS_INTERFACE_SETTINGS = unsafe { std::mem::zeroed() };
    settings.Version = DNS_INTERFACE_SETTINGS_VERSION1;
    settings.Flags = if ipv6 { 1 } else { 0 };
    assert_eq!(unsafe { GetInterfaceDnsSettings(guid, &mut settings) }, 0);
    let actual = if settings.NameServer.is_null() {
        String::new()
    } else {
        unsafe {
            let mut length = 0;
            while *settings.NameServer.add(length) != 0 {
                length += 1;
            }
            String::from_utf16_lossy(std::slice::from_raw_parts(settings.NameServer, length))
        }
    };
    // windows-sys 0.52 incorrectly declares this VOID API as WIN32_ERROR.
    // Use the documented signature; there is no return value to assert.
    #[link(name = "iphlpapi")]
    extern "system" {
        #[link_name = "FreeInterfaceDnsSettings"]
        fn free_dns_settings(settings: *mut DNS_INTERFACE_SETTINGS);
    }
    unsafe {
        free_dns_settings(&mut settings);
    }
    let normalized = actual
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(normalized, expected, "DNS family ipv6={ipv6}");
}
