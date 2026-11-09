//! Single-instance enforcement and cross-instance signals (Windows).
//!
//! The primary instance holds a named mutex; a second launch signals the
//! primary to show its window and exits. `wax-vpn.exe --quit` sets a named
//! event the primary polls, exiting through the same cleanup path as tray
//! Quit. Events are auto-reset so a consumed signal cannot fire twice, and
//! stale signals are drained at startup.

#![cfg(windows)]

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, WAIT_OBJECT_0};
use windows_sys::Win32::System::Threading::{CreateEventW, CreateMutexW, SetEvent, WaitForSingleObject};

/// One-line diagnostics for the cross-instance show path, read by CI on failure.
pub fn diag(msg: &str) {
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(crate::settings::debug_log_path())
    {
        use std::io::Write;
        let _ = writeln!(f, "{msg}");
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

const MUTEX_NAME: &str = r"Local\waXVPN.Instance";
const QUIT_EVENT: &str = r"Local\waXVPN.Quit";
const SHOW_EVENT: &str = r"Local\waXVPN.Show";

pub struct Instance {
    mutex: HANDLE,
    quit_event: HANDLE,
    show_event: HANDLE,
}

pub enum Acquire {
    Primary(Instance),
    AlreadyRunning,
}

fn signal(name: &str) {
    unsafe {
        let ev = CreateEventW(std::ptr::null(), 0, 0, wide(name).as_ptr());
        if ev != 0 {
            let ok = SetEvent(ev);
            diag(&format!("signal {name}: SetEvent -> {ok}"));
            CloseHandle(ev);
        } else {
            diag(&format!("signal {name}: CreateEventW failed ({})", GetLastError()));
        }
    }
}

fn drain(event: HANDLE) {
    unsafe {
        WaitForSingleObject(event, 0);
    }
}

pub fn acquire() -> Acquire {
    unsafe {
        let mutex = CreateMutexW(std::ptr::null(), 1, wide(MUTEX_NAME).as_ptr());
        if mutex == 0 {
            // Could not enforce: run as primary rather than block the user.
            let quit_event = CreateEventW(std::ptr::null(), 0, 0, wide(QUIT_EVENT).as_ptr());
            let show_event = CreateEventW(std::ptr::null(), 0, 0, wide(SHOW_EVENT).as_ptr());
            return Acquire::Primary(Instance { mutex: 0, quit_event, show_event });
        }
        if GetLastError() == ERROR_ALREADY_EXISTS {
            signal(SHOW_EVENT);
            CloseHandle(mutex);
            return Acquire::AlreadyRunning;
        }
        let quit_event = CreateEventW(std::ptr::null(), 0, 0, wide(QUIT_EVENT).as_ptr());
        let show_event = CreateEventW(std::ptr::null(), 0, 0, wide(SHOW_EVENT).as_ptr());
        // A signal sent before we existed must not act on a fresh launch.
        drain(quit_event);
        drain(show_event);
        Acquire::Primary(Instance { mutex, quit_event, show_event })
    }
}

/// Ask the running instance (if any) to quit gracefully. Returns true when a
/// running instance was signaled, false when no instance appears to exist.
pub fn request_quit() -> bool {
    unsafe {
        let mutex = CreateMutexW(std::ptr::null(), 0, wide(MUTEX_NAME).as_ptr());
        if mutex == 0 {
            return false;
        }
        let running = GetLastError() == ERROR_ALREADY_EXISTS;
        CloseHandle(mutex);
        if running {
            signal(QUIT_EVENT);
        }
        running
    }
}

fn signaled(event: HANDLE) -> bool {
    unsafe { event != 0 && WaitForSingleObject(event, 0) == WAIT_OBJECT_0 }
}

impl Instance {
    /// Consumes the quit signal when present (auto-reset event).
    pub fn take_quit(&self) -> bool {
        signaled(self.quit_event)
    }

    /// Consumes the show-window signal when present.
    pub fn take_show(&self) -> bool {
        signaled(self.show_event)
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            for h in [self.mutex, self.quit_event, self.show_event] {
                if h != 0 {
                    CloseHandle(h);
                }
            }
        }
    }
}
