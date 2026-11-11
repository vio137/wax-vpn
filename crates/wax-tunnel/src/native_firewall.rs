//! Remove only the retired waX rule-name prefix. Never change firewall policy.
use crate::TunnelError;
use windows::{
    core::{Interface, BSTR},
    Win32::{
        NetworkManagement::WindowsFirewall::{INetFwPolicy2, INetFwRule, NetFwPolicy2},
        System::{
            Com::{
                CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
                COINIT_APARTMENTTHREADED,
            },
            Ole::IEnumVARIANT,
            Variant::{VariantClear, VARIANT, VT_DISPATCH, VT_UNKNOWN},
        },
    },
};
const PREFIX: &str = "waX VPN KillSwitch";
fn legacy_name(name: &str) -> bool {
    name.to_ascii_lowercase()
        .starts_with(&PREFIX.to_ascii_lowercase())
}
pub(crate) fn cleanup() -> Result<(), TunnelError> {
    cleanup_inner()
        .map_err(|error| TunnelError::Message(format!("Legacy firewall cleanup failed: {error}")))
}
fn cleanup_inner() -> windows::core::Result<()> {
    struct Apartment(bool);
    impl Drop for Apartment {
        fn drop(&mut self) {
            if self.0 {
                unsafe {
                    CoUninitialize();
                }
            }
        }
    }
    let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    // RPC_E_CHANGED_MODE means COM is already initialized in another apartment.
    if hr.is_err() && hr.0 != 0x80010106u32 as i32 {
        return Err(hr.into());
    }
    let _apartment = Apartment(hr.is_ok());
    let policy: INetFwPolicy2 =
        unsafe { CoCreateInstance(&NetFwPolicy2, None, CLSCTX_INPROC_SERVER)? };
    let rules = unsafe { policy.Rules()? };
    let enumerator: IEnumVARIANT = unsafe { rules._NewEnum()?.cast()? };
    let mut names = Vec::new();
    loop {
        struct Value(VARIANT);
        impl Drop for Value {
            fn drop(&mut self) {
                unsafe {
                    let _ = VariantClear(&mut self.0);
                }
            }
        }
        let mut value = Value(VARIANT::default());
        let mut fetched = 0;
        unsafe {
            enumerator
                .Next(std::slice::from_mut(&mut value.0), &mut fetched)
                .ok()?;
        }
        if fetched == 0 {
            break;
        }
        let rule: INetFwRule = unsafe {
            let inner = &value.0.Anonymous.Anonymous;
            if inner.vt == VT_DISPATCH {
                inner
                    .Anonymous
                    .pdispVal
                    .as_ref()
                    .ok_or_else(windows::core::Error::from_thread)?
                    .cast()?
            } else if inner.vt == VT_UNKNOWN {
                inner
                    .Anonymous
                    .punkVal
                    .as_ref()
                    .ok_or_else(windows::core::Error::from_thread)?
                    .cast()?
            } else {
                return Err(windows::core::Error::from_hresult(windows::core::HRESULT(
                    0x80070057u32 as i32,
                )));
            }
        };
        let name = unsafe { rule.Name()? }.to_string();
        if legacy_name(&name) {
            names.push(name);
        }
    }
    // Collect first so mutation cannot invalidate enumeration.
    for name in names {
        unsafe {
            rules.Remove(&BSTR::from(name))?;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_retired_prefix_is_selected() {
        assert!(legacy_name("waX VPN KillSwitch IPv4"));
        assert!(legacy_name("WAX VPN KILLSWITCH"));
        for name in [
            "waX VPN",
            "Other VPN KillSwitch",
            "prefix waX VPN KillSwitch",
            "Windows Firewall",
        ] {
            assert!(!legacy_name(name));
        }
    }
}

#[cfg(test)]
mod runner_tests {
    use super::*;
    #[test]
    #[ignore = "isolated elevated Windows runner only"]
    fn native_runner_legacy_cleanup_preserves_unrelated_rule() {
        use windows::Win32::{
            Foundation::VARIANT_FALSE,
            NetworkManagement::WindowsFirewall::{
                NetFwRule, NET_FW_ACTION_ALLOW, NET_FW_RULE_DIR_OUT,
            },
        };
        assert_eq!(std::env::var("GITHUB_ACTIONS").as_deref(), Ok("true"));
        assert!(crate::is_elevated());
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().unwrap();
        }
        struct Com;
        impl Drop for Com {
            fn drop(&mut self) {
                unsafe {
                    CoUninitialize();
                }
            }
        }
        let _com = Com;
        let policy: INetFwPolicy2 =
            unsafe { CoCreateInstance(&NetFwPolicy2, None, CLSCTX_INPROC_SERVER).unwrap() };
        let rules = unsafe { policy.Rules().unwrap() };
        let legacy = BSTR::from(format!("{PREFIX} Test {}", std::process::id()));
        let sentinel = BSTR::from(format!("waX Native Test Sentinel {}", std::process::id()));
        struct Cleanup(
            windows::Win32::NetworkManagement::WindowsFirewall::INetFwRules,
            BSTR,
            BSTR,
        );
        impl Drop for Cleanup {
            fn drop(&mut self) {
                unsafe {
                    let _ = self.0.Remove(&self.1);
                    let _ = self.0.Remove(&self.2);
                }
            }
        }
        let _cleanup = Cleanup(rules.clone(), legacy.clone(), sentinel.clone());
        for name in [&legacy, &sentinel] {
            let rule: INetFwRule =
                unsafe { CoCreateInstance(&NetFwRule, None, CLSCTX_INPROC_SERVER).unwrap() };
            unsafe {
                rule.SetName(name).unwrap();
                rule.SetProtocol(256).unwrap();
                rule.SetDirection(NET_FW_RULE_DIR_OUT).unwrap();
                rule.SetAction(NET_FW_ACTION_ALLOW).unwrap();
                rule.SetEnabled(VARIANT_FALSE).unwrap();
                rules.Add(&rule).unwrap();
            }
        }
        cleanup().expect("native legacy cleanup");
        assert!(
            unsafe { rules.Item(&legacy) }.is_err(),
            "legacy test rule removed"
        );
        assert!(
            unsafe { rules.Item(&sentinel) }.is_ok(),
            "unrelated disabled test rule preserved"
        );
    }
}
