//! Tiny persisted settings (JSON in the config dir).

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub kill_switch: bool,
    pub allow_lan: bool,
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
            kill_switch: false,
            allow_lan: true,
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
pub fn debug_log_path() -> std::path::PathBuf {
    wax_core::profile::config_dir().join("wax-debug.log")
}

pub fn load() -> Settings {
    let p = path();
    if !p.exists() {
        return Settings::default();
    }
    std::fs::read_to_string(p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save(settings: &Settings) {
    let p = path();
    if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(p, json);
    }
}
