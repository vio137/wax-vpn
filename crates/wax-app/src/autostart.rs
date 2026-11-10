//! Existing per-user startup option. Never report success before readback.
#[cfg(windows)]
fn set_and_verify(key: &winreg::RegKey, name: &str, desired: Option<&str>) -> std::io::Result<()> {
    match desired {
        Some(command) => key.set_value(name, &command)?,
        None => match key.delete_value(name) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e),
        },
    }
    let actual = match key.get_value::<String, _>(name) {
        Ok(value) => Some(value),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e),
    };
    if actual.as_deref() == desired { Ok(()) }
    else { Err(std::io::Error::other("startup_readback")) }
}

pub fn apply(enabled: bool) -> Result<(), ()> {
    #[cfg(windows)]
    {
        use winreg::{enums::{HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE}, RegKey};
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu.create_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_QUERY_VALUE | KEY_SET_VALUE,
        ).map_err(|_| ())?;
        let command = if enabled {
            let exe = std::env::current_exe().map_err(|_| ())?;
            // Do not silently replace non-Unicode paths with a broken command.
            Some(format!("\"{}\"", exe.to_str().ok_or(())?))
        } else { None };
        set_and_verify(&key, "waX VPN", command.as_deref()).map_err(|_| ())
    }
    #[cfg(not(windows))]
    { let _ = enabled; Err(()) }
}

#[cfg(all(windows, test))]
mod tests {
    use super::*;
    #[test]
    fn sandbox_registry_write_readback_and_idempotent_remove() {
        use winreg::{enums::HKEY_CURRENT_USER, RegKey};
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let suffix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let path = format!("Software\\waX-VPN-tests\\startup-{}-{suffix}", std::process::id());
        struct Cleanup(String);
        impl Drop for Cleanup { fn drop(&mut self) { let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(&self.0); } }
        let _cleanup = Cleanup(path.clone());
        let (key, _) = hkcu.create_subkey(&path).unwrap();
        // Never touches the real Run key or schedules a program to launch.
        set_and_verify(&key, "test", Some("\"C:\\test path\\waX VPN.exe\"")).unwrap();
        set_and_verify(&key, "test", Some("\"C:\\second\\waX VPN.exe\"")).unwrap();
        set_and_verify(&key, "test", None).unwrap();
        set_and_verify(&key, "test", None).unwrap();
    }
}
