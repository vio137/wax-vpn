//! Tiny persisted settings (JSON in the config dir).

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub network: crate::measurements::Config,
    pub autostart: bool,
    pub auto_reconnect: bool,
    pub reduced_motion: bool,
    pub last_profile: Option<String>,
    /// Per-profile split-tunnel exclusions (profile name -> raw CIDR list).
    pub split_exclusions: std::collections::BTreeMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            network: crate::measurements::Config::default(),
            autostart: false,
            auto_reconnect: true,
            reduced_motion: false,
            last_profile: None,
            split_exclusions: std::collections::BTreeMap::new(),
        }
    }
}

fn path() -> std::path::PathBuf {
    wax_core::profile::config_dir().join("settings.json")
}

/// Path of the small diagnostics log for the cross-instance show path.
#[cfg(windows)]
pub fn debug_log_path() -> std::path::PathBuf {
    wax_core::profile::config_dir().join("wax-debug.log")
}

/// Missing settings are a first launch. Other failures must stay visible.
pub fn load_checked() -> Result<Settings, ()> { load_from(&path()) }

// Compatibility until the GUI handles explicit load/save results.
pub fn load() -> Settings { load_checked().unwrap_or_else(|_| {
    let mut value = Settings::default(); value.auto_reconnect = false; value
}) }
pub fn save(settings: &Settings) { let _ = save_checked(settings); }

fn load_from(p: &std::path::Path) -> Result<Settings, ()> {
    match std::fs::read_to_string(p) {
        Ok(text) => {
            // Windows editors may prefix otherwise valid UTF-8 JSON with a BOM.
            let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
            serde_json::from_str(text).map_err(|_| ())
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(_) => Err(()),
    }
}

pub fn save_checked(settings: &Settings) -> Result<(), ()> {
    save_to(&path(), settings).map_err(|_| ())
}

fn save_to(p: &std::path::Path, settings: &Settings) -> std::io::Result<()> {
    use std::io::Write;
    // Do not silently replace an unreadable or malformed original. The user can
    // preserve it and fix it outside the app; session changes remain in memory.
    load_from(p).map_err(|_| std::io::Error::other("settings_existing_unreadable"))?;
    let parent = p.parent().ok_or_else(|| std::io::Error::other("settings_parent"))?;
    std::fs::create_dir_all(parent)?;
    let bytes = serde_json::to_vec_pretty(settings).map_err(std::io::Error::other)?;
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let (temp, mut file) = loop {
        let seq = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let temp = parent.join(format!(".settings-{}-{seq}.tmp", std::process::id()));
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&temp) {
            Ok(file) => break (temp, file),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    };
    let result = file.write_all(&bytes).and_then(|_| file.sync_all());
    drop(file);
    // Same-directory replacement, no delete-old fallback. This reduces the
    // direct-truncation window, but is not a power-loss durability guarantee.
    let result = result.and_then(|_| std::fs::rename(&temp, p));
    if result.is_err() { let _ = std::fs::remove_file(&temp); }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn directory() -> std::path::PathBuf {
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("wax-settings-{}-{stamp}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
    #[test]
    fn settings_roundtrip_preserves_disabled_recovery_and_exclusions() {
        let dir = directory(); let p = dir.join("settings.json");
        assert!(load_from(&p).unwrap().auto_reconnect);
        let mut value = Settings::default(); value.auto_reconnect = false;
        value.split_exclusions.insert("office".into(), "192.168.0.0/16".into());
        save_to(&p, &value).unwrap();
        value.reduced_motion = true; save_to(&p, &value).unwrap();
        let read = load_from(&p).unwrap();
        assert!(!read.auto_reconnect); assert!(read.reduced_motion);
        assert_eq!(read.split_exclusions.get("office").unwrap(), "192.168.0.0/16");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn utf8_bom_settings_load_and_save_without_resetting_preferences() {
        let dir = directory(); let p = dir.join("settings.json");
        let mut value = Settings::default(); value.auto_reconnect = false;
        value.reduced_motion = true;
        value.last_profile = Some("office".into());
        value.split_exclusions.insert("office".into(), "192.168.0.0/16".into());
        let text = format!("\u{feff}{}", serde_json::to_string_pretty(&value).unwrap());
        std::fs::write(&p, text.as_bytes()).unwrap();
        let mut read = load_from(&p).unwrap();
        assert!(!read.auto_reconnect); assert!(read.reduced_motion);
        assert_eq!(read.last_profile.as_deref(), Some("office"));
        assert_eq!(read.split_exclusions.get("office").unwrap(), "192.168.0.0/16");
        assert_eq!(std::fs::read(&p).unwrap(), text.as_bytes());
        read.autostart = true;
        save_to(&p, &read).unwrap();
        let saved = load_from(&p).unwrap();
        assert!(saved.autostart); assert!(!saved.auto_reconnect); assert!(saved.reduced_motion);
        assert_eq!(saved.last_profile, read.last_profile);
        assert_eq!(saved.split_exclusions, read.split_exclusions);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn invalid_bom_settings_stay_errors_and_are_not_replaced() {
        let dir = directory(); let p = dir.join("settings.json");
        for text in ["\u{feff}", "\u{feff}{broken", "\u{feff}\u{feff}{}"] {
            std::fs::write(&p, text.as_bytes()).unwrap();
            assert!(load_from(&p).is_err());
            assert!(save_to(&p, &Settings::default()).is_err());
            assert_eq!(std::fs::read(&p).unwrap(), text.as_bytes());
            assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn malformed_original_is_not_replaced_and_directory_errors_are_reported() {
        let dir = directory(); let p = dir.join("settings.json");
        std::fs::write(&p, b"{broken").unwrap();
        assert!(load_from(&p).is_err());
        assert!(save_to(&p, &Settings::default()).is_err());
        assert_eq!(std::fs::read(&p).unwrap(), b"{broken");
        assert!(save_to(&p.join("child.json"), &Settings::default()).is_err());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[cfg(windows)]
    #[test]
    fn locked_windows_destination_preserves_previous_bytes() {
        use std::os::windows::fs::OpenOptionsExt;
        let dir = directory(); let p = dir.join("settings.json");
        save_to(&p, &Settings::default()).unwrap();
        let before = std::fs::read(&p).unwrap();
        let locked = std::fs::OpenOptions::new().read(true).share_mode(1).open(&p).unwrap();
        let mut value = Settings::default(); value.auto_reconnect = false;
        assert!(save_to(&p, &value).is_err());
        assert_eq!(std::fs::read(&p).unwrap(), before);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        drop(locked); std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn removed_preview_settings_are_discarded() {
        let old = r#"{"kill_switch":true,"allow_lan":false,"reduced_motion":true}"#;
        let settings: Settings = serde_json::from_str(old).unwrap();
        assert!(settings.reduced_motion);
        let migrated = serde_json::to_string(&settings).unwrap();
        assert!(!migrated.contains("kill_switch"));
        assert!(!migrated.contains("allow_lan"));
    }
}
