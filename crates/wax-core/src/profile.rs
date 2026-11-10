//! Named profile storage: profiles live as .conf files under the waX VPN
//! config directory (%APPDATA%/wax-vpn/profiles on Windows).

use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crate::{conf::WireGuardConfig, CoreError};

#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    pub name: String,
    pub config: WireGuardConfig,
}

pub fn config_dir() -> PathBuf {
    if let Some(d) = dirs::config_dir() {
        d.join("wax-vpn")
    } else {
        PathBuf::from(".")
    }
}

pub fn profiles_dir() -> PathBuf {
    config_dir().join("profiles")
}

fn sanitize_filename(name: &str) -> String {
    let mut safe = name.chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .collect::<String>()
        .trim()
        .trim_end_matches(['.', ' '])
        .to_string();
    if safe.is_empty() { safe = "profile".into(); }
    // Windows reserves device names even with an extension (CON.conf,
    // NUL.office.conf). Prefix rather than failing at the filesystem API.
    let stem = safe.split('.').next().unwrap_or("").trim_end().to_ascii_uppercase();
    let numbered_device = stem.strip_prefix("COM").or_else(|| stem.strip_prefix("LPT"))
        .map(|n| matches!(n, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"))
        .unwrap_or(false);
    if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL") || numbered_device {
        safe.insert(0, '_');
    }
    safe
}

pub fn profile_path(name: &str) -> PathBuf {
    profiles_dir().join(format!("{}.conf", sanitize_filename(name)))
}

pub fn save_profile(profile: &Profile) -> Result<PathBuf, CoreError> {
    let dir = profiles_dir();
    fs::create_dir_all(&dir)?;
    let path = profile_path(&profile.name);
    fs::write(&path, profile.config.to_conf_string())?;
    Ok(path)
}

pub fn load_profile(name: &str) -> Result<Profile, CoreError> {
    let path = profile_path(name);
    let text = fs::read_to_string(&path)?;
    let config = WireGuardConfig::parse(&text)?;
    Ok(Profile { name: name.to_string(), config })
}

/// Loading may succeed partially. Counts never include file names or contents.
#[derive(Debug, Default)]
pub struct ProfileLoadReport {
    pub profiles: Vec<Profile>,
    pub skipped: usize,
}

pub fn load_profiles() -> Result<Vec<Profile>, CoreError> {
    Ok(load_profiles_report()?.profiles)
}

pub fn load_profiles_report() -> Result<ProfileLoadReport, CoreError> {
    load_profiles_from(&profiles_dir())
}

fn load_profiles_from(dir: &Path) -> Result<ProfileLoadReport, CoreError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(ProfileLoadReport::default()),
        Err(e) => return Err(e.into()),
    };
    let mut report = ProfileLoadReport::default();
    for entry in entries {
        // A directory enumeration failure is not an empty or complete list.
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("conf") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
            report.skipped += 1;
            continue;
        };
        match fs::read_to_string(&path).ok().and_then(|text| WireGuardConfig::parse(&text).ok()) {
            Some(config) => report.profiles.push(Profile { name: name.to_string(), config }),
            None => report.skipped += 1,
        }
    }
    report.profiles.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(report)
}

pub fn delete_profile(name: &str) -> Result<(), CoreError> {
    let path = profile_path(name);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// Imports are additive. Exclusive creation avoids a check-then-overwrite race
/// between GUI/CLI instances and returns the actual canonical filename stem.
pub fn import_conf(name: &str, text: &str) -> Result<Profile, CoreError> {
    import_into(&profiles_dir(), name, text)
}

fn import_into(dir: &Path, name: &str, text: &str) -> Result<Profile, CoreError> {
    let config = WireGuardConfig::parse(text)?;
    let serialized = config.to_conf_string();
    fs::create_dir_all(dir)?;
    let base = sanitize_filename(name);
    let mut number = 1u32;
    loop {
        let candidate = if number == 1 { base.clone() } else { format!("{base} - {number}") };
        let path = dir.join(format!("{candidate}.conf"));
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        { use std::os::unix::fs::OpenOptionsExt; options.mode(0o600); }
        match options.open(&path) {
            Ok(mut file) => {
                let result = file.write_all(serialized.as_bytes()).and_then(|_| file.sync_all());
                drop(file);
                if let Err(error) = result {
                    let _ = fs::remove_file(&path);
                    return Err(error.into());
                }
                return Ok(Profile { name: candidate, config });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                number = number.checked_add(1).ok_or_else(|| std::io::Error::other("profile_name_exhausted"))?;
            }
            Err(error) => return Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn additive_import_preserves_existing_and_canonicalizes_names() {
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("wax-import-{}-{stamp}", std::process::id()));
        struct Cleanup(PathBuf);
        impl Drop for Cleanup { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
        let _cleanup = Cleanup(dir.clone());
        let config = WireGuardConfig {
            interface: crate::InterfaceConfig { private_key:[1;32], addresses:vec![], dns:vec![], mtu:Some(1380), listen_port:None },
            peers:vec![crate::PeerConfig { public_key:[2;32], preshared_key:None,
                allowed_ips:vec!["0.0.0.0/0".parse().unwrap()], endpoint:None, persistent_keepalive:None }],
        }.to_conf_string();
        let first = import_into(&dir, "office/name.", &config).unwrap();
        assert_eq!(first.name, "office_name");
        let original = fs::read(dir.join("office_name.conf")).unwrap();
        let second = import_into(&dir, "office?name", &config).unwrap();
        assert_eq!(second.name, "office_name - 2");
        assert_eq!(fs::read(dir.join("office_name.conf")).unwrap(), original);
        assert_eq!(import_into(&dir, "CON", &config).unwrap().name, "_CON");
        assert!(import_into(&dir, "bad", "not a configuration").is_err());
        assert!(!dir.join("bad.conf").exists());
        // Concurrent imports cannot overwrite each other or return one name.
        let handles = (0..4).map(|_| {
            let dir=dir.clone(); let config=config.clone();
            std::thread::spawn(move || import_into(&dir, "office_name", &config).unwrap().name)
        }).collect::<Vec<_>>();
        let names=handles.into_iter().map(|h| h.join().unwrap()).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(names.len(),4);
        assert!(!names.contains("office_name"));
        assert!(!names.contains("office_name - 2"));
        assert_eq!(fs::read(dir.join("office_name.conf")).unwrap(), original);
    }
    #[test]
    fn profile_load_report_preserves_bad_files_and_distinguishes_directory_failure() {
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("wax-load-{}-{stamp}", std::process::id()));
        struct Cleanup(PathBuf);
        impl Drop for Cleanup { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
        let _cleanup = Cleanup(dir.clone());
        assert!(load_profiles_from(&dir).unwrap().profiles.is_empty());
        fs::create_dir_all(&dir).unwrap();
        let config = WireGuardConfig {
            interface: crate::InterfaceConfig { private_key:[1;32], addresses:vec![], dns:vec![], mtu:None, listen_port:None },
            peers:vec![],
        }.to_conf_string();
        fs::write(dir.join("Zulu.conf"), &config).unwrap();
        fs::write(dir.join("alpha.conf"), &config).unwrap();
        fs::write(dir.join("bad.conf"), b"not a configuration").unwrap();
        fs::write(dir.join("ignored.txt"), b"ignored").unwrap();
        fs::create_dir(dir.join("unreadable.conf")).unwrap();
        let report = load_profiles_from(&dir).unwrap();
        assert_eq!(report.profiles.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), vec!["alpha", "Zulu"]);
        assert_eq!(report.skipped, 2);
        assert_eq!(fs::read(dir.join("bad.conf")).unwrap(), b"not a configuration");
        assert!(dir.join("unreadable.conf").is_dir());
        assert!(load_profiles_from(&dir.join("ignored.txt")).is_err());
        assert_eq!(fs::read(dir.join("Zulu.conf")).unwrap(), config.as_bytes());
    }
    #[test]
    fn profile_filenames_are_windows_safe() {
        for input in ["CON", "nul.office", "COM1", "LPT9", "com¹", "AUX."] {
            assert!(sanitize_filename(input).starts_with('_'), "{input}");
        }
        assert_eq!(sanitize_filename(" ... "), "profile");
        assert_eq!(sanitize_filename(" office. "), "office");
        assert_eq!(sanitize_filename("office . ."), "office");
        assert_eq!(sanitize_filename("Office VPN"), "Office VPN");
        assert_eq!(sanitize_filename("COM10"), "COM10");
        assert_eq!(sanitize_filename("a/b"), "a_b");
    }
}
