//! Installer-only process cleanup, limited to the exact installed GUI path.
use std::{
    os::windows::ffi::OsStrExt,
    path::Path,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{
        CloseHandle, GetLastError, ERROR_INVALID_PARAMETER, ERROR_NO_MORE_FILES, HANDLE,
        INVALID_HANDLE_VALUE, WAIT_OBJECT_0,
    },
    Globalization::{CompareStringOrdinal, CSTR_EQUAL},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        },
        Threading::{
            OpenEventW, OpenProcess, QueryFullProcessImageNameW, SetEvent, TerminateProcess, WaitForSingleObject,
            EVENT_MODIFY_STATE,
            PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
        },
    },
};
struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
fn same_path(a: &[u16], b: &[u16]) -> bool {
    unsafe {
        CompareStringOrdinal(a.as_ptr(), a.len() as i32, b.as_ptr(), b.len() as i32, 1)
            == CSTR_EQUAL
    }
}
fn path_units(path: &Path) -> Result<Vec<u16>, String> {
    let path = std::fs::canonicalize(path)
        .map_err(|e| format!("Could not identify installed GUI: {e}"))?;
    let path = path.as_os_str().encode_wide().collect::<Vec<_>>();
    // QueryFullProcessImageNameW returns the Win32 DOS path, not the extended prefix.
    let prefix: Vec<u16> = r"\\?\".encode_utf16().collect();
    let unc: Vec<u16> = r"\\?\UNC\".encode_utf16().collect();
    Ok(if path.starts_with(&unc) {
        let mut plain: Vec<u16> = r"\\".encode_utf16().collect();
        plain.extend_from_slice(&path[unc.len()..]);
        plain
    } else if path.starts_with(&prefix) {
        path[4..].to_vec()
    } else {
        path
    })
}
pub(crate) fn stop(directory: &str) -> Result<(), String> {
    let target = Path::new(directory).join("waX VPN.exe");
    if !target
        .try_exists()
        .map_err(|e| format!("Could not inspect installed GUI: {e}"))?
    {
        return Ok(());
    }
    let target_path = path_units(&target)?;
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(format!(
            "Could not enumerate installed app processes: {}",
            std::io::Error::last_os_error()
        ));
    }
    let snapshot = Handle(snapshot);
    let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
    let name: Vec<u16> = "waX VPN.exe".encode_utf16().collect();
    let mut result = unsafe { Process32FirstW(snapshot.0, &mut entry) };
    let mut processes = Vec::new();
    while result != 0 {
        let end = entry
            .szExeFile
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(entry.szExeFile.len());
        if same_path(&entry.szExeFile[..end], &name) {
            let raw = unsafe {
                OpenProcess(
                    PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE | PROCESS_SYNCHRONIZE,
                    0,
                    entry.th32ProcessID,
                )
            };
            if raw == 0 {
                let error = unsafe { GetLastError() };
                if error != ERROR_INVALID_PARAMETER {
                    return Err(format!("Could not inspect matching app process: {error}"));
                }
            } else {
                let process = Handle(raw);
                let mut path = vec![0u16; 32768];
                let mut length = path.len() as u32;
                if unsafe {
                    QueryFullProcessImageNameW(process.0, 0, path.as_mut_ptr(), &mut length)
                } == 0
                {
                    if unsafe { WaitForSingleObject(process.0, 0) } != WAIT_OBJECT_0 {
                        return Err("Could not verify matching app process path".into());
                    }
                } else if same_path(&path[..length as usize], &target_path) {
                    processes.push((process, entry.th32ProcessID));
                }
            }
        }
        result = unsafe { Process32NextW(snapshot.0, &mut entry) };
    }
    if unsafe { GetLastError() } != ERROR_NO_MORE_FILES {
        return Err("Process enumeration did not complete".into());
    }
    // A PID-scoped graceful signal cannot quit another installation. Older
    // versions have no such event; use the exact-handle fallback for them.
    for (process, pid) in &processes {
        if unsafe { WaitForSingleObject(process.0, 0) } == WAIT_OBJECT_0 { continue; }
        let name: Vec<u16> = format!(r"Local\waXVPN.Quit.{pid}").encode_utf16().chain(Some(0)).collect();
        let event = unsafe { OpenEventW(EVENT_MODIFY_STATE, 0, name.as_ptr()) };
        if event != 0 {
            let event = Handle(event);
            unsafe { SetEvent(event.0); }
        }
    }
    let until = Instant::now() + Duration::from_secs(5);
    while Instant::now() < until
        && processes
            .iter()
            .any(|(p, _)| unsafe { WaitForSingleObject(p.0, 0) } != WAIT_OBJECT_0)
    {
        std::thread::sleep(Duration::from_millis(25));
    }
    for (process, _) in processes {
        if unsafe { WaitForSingleObject(process.0, 0) } == WAIT_OBJECT_0 {
            continue;
        }
        if unsafe { TerminateProcess(process.0, 0) } == 0
            && unsafe { WaitForSingleObject(process.0, 0) } != WAIT_OBJECT_0
        {
            return Err("Could not stop the verified installed app".into());
        }
        if unsafe { WaitForSingleObject(process.0, 5000) } != WAIT_OBJECT_0 {
            return Err("Installed app did not exit before the deadline".into());
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinal_path_match_is_exact_not_basename() {
        let w = |s: &str| s.encode_utf16().collect::<Vec<_>>();
        assert!(same_path(
            &w(r"C:\Apps\waX VPN.exe"),
            &w(r"c:\apps\WAX VPN.EXE")
        ));
        assert!(!same_path(
            &w(r"C:\Other\waX VPN.exe"),
            &w(r"C:\Apps\waX VPN.exe")
        ));
    }
}
