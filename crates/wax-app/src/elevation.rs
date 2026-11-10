//! Native UAC restart. Paths are separate UTF-16 arguments, never shell code.
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

const PARENT_FLAG: &str = "--elevation-parent=";

fn parent_pid(args: &[String]) -> Result<Option<u32>, ()> {
    let mut parent = None;
    for arg in args {
        if let Some(value) = arg.strip_prefix(PARENT_FLAG) {
            let pid = value.parse::<u32>().map_err(|_| ())?;
            if pid == 0 || parent.replace(pid).is_some() { return Err(()); }
        }
    }
    Ok(parent)
}

fn launch_succeeded(result: isize) -> bool { result > 32 }

#[cfg(windows)]
fn wide(value: &std::ffi::OsStr) -> Result<Vec<u16>, ()> {
    let mut encoded = value.encode_wide().collect::<Vec<_>>();
    if encoded.contains(&0) { return Err(()); }
    encoded.push(0);
    Ok(encoded)
}

#[cfg(windows)]
pub fn restart() -> Result<(), ()> {
    use windows_sys::Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL};
    let exe = std::env::current_exe().map_err(|_| ())?;
    let path = wide(exe.as_os_str())?;
    let verb = wide(std::ffi::OsStr::new("runas"))?;
    let parameters = wide(std::ffi::OsStr::new(&format!("{PARENT_FLAG}{}", std::process::id())))?;
    // runas retains the Windows UAC consent/credential prompt. A success code
    // means the launch request was accepted, not that the new UI is ready.
    let result = unsafe { ShellExecuteW(0, verb.as_ptr(), path.as_ptr(), parameters.as_ptr(), std::ptr::null(), SW_SHOWNORMAL) };
    if launch_succeeded(result) { Ok(()) } else { Err(()) }
}

#[cfg(windows)]
fn wait_for_parent(pid: u32, timeout_ms: u32) -> Result<(), ()> {
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_INVALID_PARAMETER, WAIT_OBJECT_0};
    use windows_sys::Win32::System::Threading::{OpenProcess, WaitForSingleObject};
    // SYNCHRONIZE only: no termination, token or process-modification rights.
    let handle = unsafe { OpenProcess(0x0010_0000, 0, pid) };
    if handle == 0 {
        return if unsafe { GetLastError() } == ERROR_INVALID_PARAMETER { Ok(()) } else { Err(()) };
    }
    let result = unsafe { WaitForSingleObject(handle, timeout_ms) };
    unsafe { CloseHandle(handle); }
    if result == WAIT_OBJECT_0 { Ok(()) } else { Err(()) }
}

#[cfg(windows)]
pub fn finish_restart() -> Result<(), ()> {
    let args = std::env::args_os().skip(1).map(|arg| arg.into_string().map_err(|_| ())).collect::<Result<Vec<_>, _>>()?;
    if let Some(pid) = parent_pid(&args)? {
        if !wax_tunnel::is_elevated() || pid == std::process::id() { return Err(()); }
        // Do not race the original process's named single-instance mutex.
        // Failure/timeout aborts this child instead of starting a duplicate.
        wait_for_parent(pid, 10_000)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<String> { values.iter().map(|v| v.to_string()).collect() }
    #[test]
    fn restart_parent_parser_rejects_ambiguous_or_invalid_values() {
        assert_eq!(parent_pid(&args(&["wax.exe"])), Ok(None));
        assert_eq!(parent_pid(&args(&["wax.exe", "--elevation-parent=42"])), Ok(Some(42)));
        for values in [vec!["--elevation-parent=0"], vec!["--elevation-parent=-1"], vec!["--elevation-parent="], vec!["--elevation-parent=4294967296"], vec!["--elevation-parent=42", "--elevation-parent=43"]] {
            assert!(parent_pid(&args(&values)).is_err());
        }
    }
    #[test]
    fn shell_result_boundary_keeps_failed_or_cancelled_launches_open() {
        for value in [-1, 0, 2, 5, 31, 32] { assert!(!launch_succeeded(value)); }
        assert!(launch_succeeded(33));
    }
    #[cfg(windows)]
    #[test]
    fn native_path_preserves_apostrophe_spaces_unicode_and_non_unicode() {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        let original = OsString::from("C:\\Users\\O'Brien\\桌面\\waX VPN.exe");
        let encoded = wide(&original).unwrap();
        assert_eq!(encoded.last(), Some(&0));
        assert_eq!(OsString::from_wide(&encoded[..encoded.len()-1]), original);
        let original = OsString::from_wide(&[0x43, 0x3a, 0x5c, 0xd800, 0x2e, 0x65, 0x78, 0x65]);
        let encoded = wide(&original).unwrap();
        assert_eq!(OsString::from_wide(&encoded[..encoded.len()-1]), original);
        assert!(wide(&OsString::from_wide(&[0x41, 0, 0x42])).is_err());
    }
    #[cfg(windows)]
    #[test]
    fn parent_wait_is_bounded_without_terminating_the_process() {
        assert!(wait_for_parent(std::process::id(), 1).is_err());
        assert!(wait_for_parent(u32::MAX, 1).is_ok());
    }
}
