//! Windows notification-owner groundwork. Currently test-only, not part of the
//! production reconnect path. Callbacks perform no work except an atomic flag.
use std::{ffi::c_void, sync::atomic::{AtomicBool, Ordering}};
use windows_sys::Win32::{Foundation::HANDLE, NetworkManagement::IpHelper::{
    CancelMibChangeNotify2, NotifyRouteChange2, NotifyUnicastIpAddressChange,
    MIB_IPFORWARD_ROW2, MIB_UNICASTIPADDRESS_ROW, MIB_NOTIFICATION_TYPE, MibInitialNotification,
}, Networking::WinSock::AF_UNSPEC};

struct Context {
    own_luid: u64,
    changed: AtomicBool,
}
impl Context {
    fn mark(&self, luid: u64) {
        if luid != 0 && luid != self.own_luid { self.changed.store(true, Ordering::Release); }
    }
}
unsafe extern "system" fn route(context: *const c_void, row: *const MIB_IPFORWARD_ROW2, kind: MIB_NOTIFICATION_TYPE) {
    if kind == MibInitialNotification || context.is_null() || row.is_null() { return; }
    // Windows owns row. Copy only its stable LUID; never retain/free the row.
    let context = unsafe { &*(context as *const Context) };
    context.mark(unsafe { (*row).InterfaceLuid.Value });
}
unsafe extern "system" fn address(context: *const c_void, row: *const MIB_UNICASTIPADDRESS_ROW, kind: MIB_NOTIFICATION_TYPE) {
    if kind == MibInitialNotification || context.is_null() || row.is_null() { return; }
    let context = unsafe { &*(context as *const Context) };
    context.mark(unsafe { (*row).InterfaceLuid.Value });
}

pub struct Notifications {
    context: Option<Box<Context>>,
    route: HANDLE,
    address: HANDLE,
}
impl Notifications {
    pub fn register(own_luid: u64) -> Result<Self, &'static str> {
        if own_luid == 0 { return Err("network_notify_identity"); }
        let mut owner = Self { context: Some(Box::new(Context { own_luid, changed: AtomicBool::new(false) })), route: 0, address: 0 };
        let context = owner.context.as_deref().unwrap() as *const Context as *const c_void;
        // Context is heap-stable before either registration can invoke callback.
        // Initial confirmations are requested to exercise the null-row path.
        let result = unsafe { NotifyRouteChange2(AF_UNSPEC, Some(route), context, 1, &mut owner.route) };
        if result != 0 { return Err("network_notify_route"); }
        let result = unsafe { NotifyUnicastIpAddressChange(AF_UNSPEC, Some(address), context, 1, &mut owner.address) };
        if result != 0 { return Err("network_notify_address"); }
        Ok(owner)
    }
    pub fn take_changed(&self) -> bool {
        self.context.as_ref().is_some_and(|c| c.changed.swap(false, Ordering::AcqRel))
    }
    /// Never call from either callback or while owning a callback dependency.
    /// Both cancellations are attempted even when one fails. On failure keep
    /// context alive, so an uncertain registration cannot use freed memory.
    pub fn shutdown(&mut self) -> Result<(), &'static str> {
        let mut failed = false;
        for handle in [&mut self.route, &mut self.address] {
            if *handle != 0 {
                if unsafe { CancelMibChangeNotify2(*handle) } == 0 { *handle = 0; }
                else { failed = true; }
            }
        }
        if failed { Err("network_notify_cancel") }
        else { self.context.take(); Ok(()) }
    }
}
impl Drop for Notifications {
    fn drop(&mut self) {
        if self.shutdown().is_err() {
            // Exceptional cancellation failure: Windows may still call back.
            // Leak this tiny, address-free context rather than risk a UAF.
            if let Some(context) = self.context.take() { let _ = Box::leak(context); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::NetworkManagement::IpHelper::MibParameterNotification;
    #[test]
    fn callbacks_filter_initial_null_own_and_coalesce_physical_identity() {
        let c = Context { own_luid: 11, changed: AtomicBool::new(false) };
        let p = &c as *const Context as *const c_void;
        let mut r: MIB_IPFORWARD_ROW2 = unsafe { std::mem::zeroed() };
        let mut a: MIB_UNICASTIPADDRESS_ROW = unsafe { std::mem::zeroed() };
        r.InterfaceLuid.Value = 11; a.InterfaceLuid.Value = 11;
        unsafe {
            route(p,std::ptr::null(),MibInitialNotification);
            address(p,std::ptr::null(),MibParameterNotification);
            route(p,&r,MibParameterNotification);
            address(p,&a,MibParameterNotification);
        }
        assert!(!c.changed.swap(false,Ordering::AcqRel));
        r.InterfaceLuid.Value = 22; a.InterfaceLuid.Value = 33;
        unsafe { route(p,&r,MibParameterNotification); address(p,&a,MibParameterNotification); }
        assert!(c.changed.swap(false,Ordering::AcqRel));
        assert!(!c.changed.swap(false,Ordering::AcqRel));
    }
    #[test]
    fn windows_registration_and_explicit_cancellation_are_repeatable() {
        // No route/address mutation, tunnel, probe, registration service or upload.
        for _ in 0..3 {
            let mut owner = Notifications::register(1).expect("local notification registration");
            let _ = owner.take_changed(); // External runner activity is allowed.
            owner.shutdown().expect("local notification cancellation");
            assert!(!owner.take_changed());
            owner.shutdown().expect("idempotent cancellation");
        }
    }
}
