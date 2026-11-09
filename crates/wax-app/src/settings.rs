//! Tiny persisted settings (JSON in the config dir).

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub kill_switch: bool,
    pub allow_lan: bool,
    pub autostart: bool,
    pub reduced_motion: bool,
    pub last_profile: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            kill_switch: false,
            allow_lan: true,
            autostart: false,
            reduced_motion: false,
            last_profile: None,
        }
    }
}

fn path() -> std::path::PathBuf {
    wax_core::profile::config_dir().join("settings.json")
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
