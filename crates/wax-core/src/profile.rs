//! Named profile storage: profiles live as .conf files under the waX VPN
//! config directory (%APPDATA%/wax-vpn/profiles on Windows).

use std::fs;
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
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .collect::<String>()
        .trim()
        .to_string()
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

pub fn load_profiles() -> Result<Vec<Profile>, CoreError> {
    let dir = profiles_dir();
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("conf") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("profile")
            .to_string();
        match fs::read_to_string(&path) {
            Ok(text) => match WireGuardConfig::parse(&text) {
                Ok(config) => out.push(Profile { name, config }),
                Err(e) => log::warn!("skipping unparsable profile {}: {}", path.display(), e),
            },
            Err(e) => log::warn!("skipping unreadable profile {}: {}", path.display(), e),
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

pub fn delete_profile(name: &str) -> Result<(), CoreError> {
    let path = profile_path(name);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn import_conf(name: &str, text: &str) -> Result<Profile, CoreError> {
    let config = WireGuardConfig::parse(text)?;
    let profile = Profile { name: name.to_string(), config };
    save_profile(&profile)?;
    Ok(profile)
}
