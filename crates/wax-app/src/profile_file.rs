//! User-selected local profile files. Never log paths or configuration text.
use std::{fs::File, io::Read, path::Path};
const MAX_CONFIG_BYTES: u64 = 1024 * 1024;
pub struct ImportedFile {
    pub name: String,
    pub text: String,
}
pub fn read(path: &Path) -> Result<ImportedFile, &'static str> {
    if !path
        .extension()
        .is_some_and(|s| s.eq_ignore_ascii_case("conf"))
    {
        return Err("Choose a WireGuard .conf file");
    }
    let file = File::open(path).map_err(|_| "The selected file could not be opened")?;
    let mut text = String::new();
    file.take(MAX_CONFIG_BYTES + 1)
        .read_to_string(&mut text)
        .map_err(|_| "The file must contain readable UTF-8 WireGuard text")?;
    if text.len() as u64 > MAX_CONFIG_BYTES {
        return Err("The selected file is larger than 1 MiB");
    }
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text).to_owned();
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Imported")
        .to_owned();
    Ok(ImportedFile { name, text })
}
#[cfg(windows)]
pub fn choose() -> Result<Option<ImportedFile>, &'static str> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::UI::{
        Controls::Dialogs::*,
        WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
    };
    let filter: Vec<u16> = "WireGuard profiles (*.conf)\0*.conf\0\0"
        .encode_utf16()
        .collect();
    let title: Vec<u16> = "Import a WireGuard profile\0".encode_utf16().collect();
    let mut filename = vec![0u16; 32768];
    let mut dialog: OPENFILENAMEW = unsafe { std::mem::zeroed() };
    dialog.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    let owner = unsafe { GetForegroundWindow() };
    let mut owner_pid = 0;
    unsafe {
        GetWindowThreadProcessId(owner, &mut owner_pid);
    }
    if owner_pid == std::process::id() {
        dialog.hwndOwner = owner;
    }
    dialog.lpstrFilter = filter.as_ptr();
    dialog.nFilterIndex = 1;
    dialog.lpstrTitle = title.as_ptr();
    dialog.lpstrFile = filename.as_mut_ptr();
    dialog.nMaxFile = filename.len() as u32;
    dialog.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR | OFN_EXPLORER;
    if unsafe { GetOpenFileNameW(&mut dialog) } == 0 {
        return if unsafe { CommDlgExtendedError() } == 0 {
            Ok(None)
        } else {
            Err("The Windows file picker could not open")
        };
    }
    let length = filename
        .iter()
        .position(|c| *c == 0)
        .ok_or("The selected path is too long")?;
    let path = std::path::PathBuf::from(std::ffi::OsString::from_wide(&filename[..length]));
    read(&path).map(Some)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_file_read_is_bounded_and_keeps_original_unchanged() {
        let dir = std::env::temp_dir().join(format!(
            "wax-file-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("Office.CONF");
        let body = "\u{feff}[Interface]\n# local fixture\n";
        std::fs::write(&path, body).unwrap();
        let result = read(&path).unwrap();
        assert_eq!(result.name, "Office");
        assert_eq!(result.text, "[Interface]\n# local fixture\n");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), body);
        std::fs::write(&path, vec![b'x'; MAX_CONFIG_BYTES as usize + 1]).unwrap();
        assert!(read(&path).is_err());
        std::fs::write(&path, [0xff, 0xfe]).unwrap();
        assert!(read(&path).is_err());
        assert!(read(&dir.join("missing.conf")).is_err());
        assert!(read(&dir.join("not-a-profile.txt")).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
