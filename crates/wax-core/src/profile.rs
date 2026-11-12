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

/// Creates exactly this profile name and never replaces an existing path.
/// This is separate from explicit profile updates and additive imports.
pub fn create_profile(profile: &Profile) -> Result<PathBuf, CoreError> {
    create_profile_in(&profiles_dir(), profile)
}

fn create_profile_in(dir: &Path, profile: &Profile) -> Result<PathBuf, CoreError> {
    fs::create_dir_all(dir)?;
    let path = dir.join(format!("{}.conf", sanitize_filename(&profile.name)));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    { use std::os::unix::fs::OpenOptionsExt; options.mode(0o600); }
    let mut file = options.open(&path)?;
    let result = file.write_all(profile.config.to_conf_string().as_bytes()).and_then(|_| file.sync_all());
    drop(file);
    if let Err(error) = result {
        let _ = fs::remove_file(&path);
        return Err(error.into());
    }
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
        if !path.extension().and_then(|e| e.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("conf"))
        {
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
    report.profiles.sort_by_cached_key(|p| p.name.to_lowercase());
    Ok(report)
}

pub fn delete_profile(name: &str) -> Result<(), CoreError> {
    let path = profile_path(name);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// Non-secret, local import inspection. Does not read or write files, resolve
/// endpoints, register a service or create a tunnel.
#[derive(Debug, PartialEq, Eq)]
pub struct ImportSummary {
    pub addresses: usize,
    pub dns_servers: usize,
    pub peers: usize,
    pub route_entries: usize,
    pub default_ipv4: bool,
    pub default_ipv6: bool,
}

pub fn inspect_conf(text: &str) -> Result<ImportSummary, CoreError> {
    let config = WireGuardConfig::parse(text)?;
    config.validate_import_endpoints()?;
    Ok(ImportSummary {
        addresses: config.interface.addresses.len(),
        dns_servers: config.interface.dns.len(),
        peers: config.peers.len(),
        route_entries: config.peers.iter().map(|p| p.allowed_ips.len()).sum(),
        default_ipv4: config.peers.iter().flat_map(|p| &p.allowed_ips).any(|ip| ip.addr().is_ipv4() && ip.prefix_len() == 0),
        default_ipv6: config.peers.iter().flat_map(|p| &p.allowed_ips).any(|ip| ip.addr().is_ipv6() && ip.prefix_len() == 0),
    })
}

/// Imports are additive. Exclusive creation avoids a check-then-overwrite race
/// between GUI/CLI instances and returns the actual canonical filename stem.
pub fn import_conf(name: &str, text: &str) -> Result<Profile, CoreError> {
    import_into(&profiles_dir(), name, text)
}

fn import_into(dir: &Path, name: &str, text: &str) -> Result<Profile, CoreError> {
    let config = WireGuardConfig::parse(text)?;
    config.validate_import_endpoints()?;
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
    #[test]
    fn inspection_matches_import_validation_without_secret_output() {
        let private = crate::keys::key_to_base64(&[1; 32]);
        let public = crate::keys::key_to_base64(&[2; 32]);
        let text = format!("[Interface]\nPrivateKey = {private}\nAddress = 10.0.0.2/32, fd00::2/128\nDNS = 1.1.1.1\n[Peer]\nPublicKey = {public}\nAllowedIPs = 0.0.0.0/0, ::/0\nEndpoint = example.invalid:51820\n");
        let summary = inspect_conf(&text).unwrap();
        assert_eq!(summary, ImportSummary { addresses: 2, dns_servers: 1, peers: 1, route_entries: 2, default_ipv4: true, default_ipv6: true });
        let output = format!("{summary:?}");
        assert!(!output.contains(&private));
        assert!(!output.contains(&public));
        assert!(!output.contains("example.invalid"));
        assert!(inspect_conf(&text.replace("example.invalid:51820", "broken:port")).is_err());
        assert!(inspect_conf(&text.replace("Address =", "PostUp = script\nAddress =")).is_err());
        assert!(inspect_conf("not a config").is_err());
        let partial = inspect_conf(&text.replace("0.0.0.0/0, ::/0", "10.0.0.1/32")).unwrap();
        assert!(!partial.default_ipv4 && !partial.default_ipv6);
        assert_eq!(partial.route_entries, 1);
    }

    use super::*;
    #[test]
    fn exact_creation_never_replaces_an_existing_profile_or_directory() {
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("wax-create-{}-{stamp}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let profile = Profile { name: "Cloudflare WARP".into(), config: WireGuardConfig {
            interface: crate::InterfaceConfig { private_key:[1;32], addresses:vec![], dns:vec![], mtu:None, listen_port:None },
            peers: vec![],
        }};
        let path = create_profile_in(&dir, &profile).unwrap();
        let original = fs::read(&path).unwrap();
        let mut changed = profile.clone();
        changed.config.interface.private_key = [2;32];
        assert!(create_profile_in(&dir, &changed).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(create_profile_in(&dir, &profile).is_err());
        assert!(path.is_dir());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn local_copy_keeps_loaded_config_and_original_with_collision_safe_names() {
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("wax-copy-{}-{stamp}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let config = WireGuardConfig {
            interface: crate::InterfaceConfig { private_key:[1;32], addresses:vec!["10.55.0.2/32".parse().unwrap()], dns:vec!["1.1.1.1".parse().unwrap()], mtu:Some(1380), listen_port:Some(51820) },
            peers:vec![crate::PeerConfig { public_key:[2;32], preshared_key:Some([3;32]),
                allowed_ips:vec!["0.0.0.0/0".parse().unwrap(),"::/0".parse().unwrap()], endpoint:Some("example.invalid:51820".into()), persistent_keepalive:Some(25) }],
        };
        let original = import_into(&dir, "Office", &config.to_conf_string()).unwrap();
        let bytes = fs::read(dir.join("Office.conf")).unwrap();
        for expected in ["Office - copy", "Office - copy - 2"] {
            let copy = import_into(&dir, &format!("{} - copy", original.name), &original.config.to_conf_string()).unwrap();
            assert_eq!(copy.name, expected);
            assert_eq!(copy.config, original.config);
            assert_eq!(WireGuardConfig::parse(&fs::read_to_string(dir.join(format!("{expected}.conf"))).unwrap()).unwrap(), original.config);
            assert_eq!(fs::read(dir.join("Office.conf")).unwrap(), bytes);
            #[cfg(unix)] {
                use std::os::unix::fs::PermissionsExt;
                assert_eq!(fs::metadata(dir.join(format!("{expected}.conf"))).unwrap().permissions().mode() & 0o777, 0o600);
            }
        }
        fs::remove_dir_all(&dir).unwrap();
    }

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
        let before_count = fs::read_dir(&dir).unwrap().count();
        let duplicate = config.replace("[Peer]", "[Peer]\nEndpoint = localhost:12345\nEndpoint = do-not-echo-secret");
        let error = import_into(&dir, "duplicate", &duplicate).unwrap_err().to_string();
        assert!(error.contains("duplicate Endpoint in [Peer]"));
        assert!(!error.contains("do-not-echo-secret"));
        assert!(!dir.join("duplicate.conf").exists());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), before_count);
        assert_eq!(fs::read(dir.join("office_name.conf")).unwrap(), original);
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
            peers:vec![crate::PeerConfig { public_key:[2;32], preshared_key:None,
                allowed_ips:vec!["0.0.0.0/0".parse().unwrap()], endpoint:None, persistent_keepalive:None }],
        }.to_conf_string();
        assert!(WireGuardConfig::parse(&config).is_ok());
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
    fn profile_loader_accepts_mixed_case_conf_extensions_without_changing_files() {
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("wax-case-load-{}-{stamp}", std::process::id()));
        struct Cleanup(PathBuf);
        impl Drop for Cleanup { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
        let _cleanup = Cleanup(dir.clone());
        fs::create_dir_all(&dir).unwrap();
        let config = WireGuardConfig {
            interface: crate::InterfaceConfig { private_key:[1;32], addresses:vec![], dns:vec![], mtu:None, listen_port:None },
            peers:vec![crate::PeerConfig { public_key:[2;32], preshared_key:None,
                allowed_ips:vec!["0.0.0.0/0".parse().unwrap()], endpoint:None, persistent_keepalive:None }],
        }.to_conf_string();
        assert!(WireGuardConfig::parse(&config).is_ok());
        for name in ["Office.CONF", "Home.CoNf", "Mobile.conf"] {
            fs::write(dir.join(name), &config).unwrap();
        }
        fs::write(dir.join("broken.cOnF"), b"not a configuration").unwrap();
        fs::write(dir.join("ignored.CONF.bak"), b"ignored").unwrap();
        let report = load_profiles_from(&dir).unwrap();
        assert_eq!(report.profiles.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), vec!["Home", "Mobile", "Office"]);
        assert_eq!(report.skipped, 1);
        for name in ["Office.CONF", "Home.CoNf", "Mobile.conf"] {
            assert_eq!(fs::read(dir.join(name)).unwrap(), config.as_bytes());
        }
        assert_eq!(fs::read(dir.join("broken.cOnF")).unwrap(), b"not a configuration");
        assert_eq!(fs::read(dir.join("ignored.CONF.bak")).unwrap(), b"ignored");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 5);
    }
    #[test]
    fn legacy_empty_allowed_ips_loads_and_malformed_import_preserves_files() {
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("wax-legacy-lists-{}-{stamp}", std::process::id()));
        struct Cleanup(PathBuf);
        impl Drop for Cleanup { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
        let _cleanup = Cleanup(dir.clone());
        fs::create_dir_all(&dir).unwrap();
        let private = crate::keys::key_to_base64(&[1;32]);
        let public = crate::keys::key_to_base64(&[2;32]);
        let old = format!("[Interface]\nPrivateKey = {private}\nAddress = 10.0.0.2/32\nDNS = 1.1.1.1\n[Peer]\nPublicKey = {public}\nAllowedIPs = \n");
        let path = dir.join("legacy.conf");
        fs::write(&path, &old).unwrap();
        let report = load_profiles_from(&dir).unwrap();
        assert_eq!(report.skipped, 0);
        assert_eq!(report.profiles.len(), 1);
        assert!(report.profiles[0].config.peers[0].allowed_ips.is_empty());
        assert_eq!(fs::read(&path).unwrap(), old.as_bytes());
        for (key, value) in [("Address", "10.0.0.2,,"), ("DNS", "1.1.1.1,"), ("AllowedIPs", ",10.0.0.0/8")] {
            let invalid = old.lines().map(|line| {
                if line.starts_with(&format!("{key} =")) { format!("{key} = {value}") }
                else { line.to_string() }
            }).collect::<Vec<_>>().join("\n");
            assert!(import_into(&dir, "invalid", &invalid).is_err());
            assert!(!dir.join("invalid.conf").exists());
            assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
            assert_eq!(fs::read(&path).unwrap(), old.as_bytes());
        }
    }

    #[test]
    fn import_rejects_bad_endpoints_but_saved_odd_endpoints_still_load() {
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("wax-endpoint-import-{}-{stamp}", std::process::id()));
        struct Cleanup(PathBuf);
        impl Drop for Cleanup { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
        let _cleanup = Cleanup(dir.clone());
        let private = crate::keys::key_to_base64(&[1;32]);
        let public = crate::keys::key_to_base64(&[2;32]);
        let config = |endpoint: &str| format!("[Interface]\nPrivateKey = {private}\n[Peer]\nPublicKey = {public}\nEndpoint = {endpoint}\n");
        let invalid = config("never-echo.invalid");
        assert!(import_into(&dir, "invalid", &invalid).is_err());
        assert!(!dir.exists());
        fs::create_dir_all(&dir).unwrap();
        let old_path = dir.join("legacy.conf");
        fs::write(&old_path, &invalid).unwrap();
        let report = load_profiles_from(&dir).unwrap();
        assert_eq!(report.skipped, 0);
        assert_eq!(report.profiles.len(), 1);
        assert_eq!(report.profiles[0].config.peers[0].endpoint.as_deref(), Some("never-echo.invalid"));
        for endpoint in ["never-echo.invalid", ":1234", "host:", "host:65536", "[not-v6]:1234", "::1:1234"] {
            let error = import_into(&dir, "legacy", &config(endpoint)).unwrap_err().to_string();
            assert!(error.contains("peer 1: invalid Endpoint"));
            assert!(!error.contains("never-echo.invalid"));
            assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
            assert_eq!(fs::read(&old_path).unwrap(), invalid.as_bytes());
        }
        for endpoint in ["vpn.Example:65535", "192.0.2.1:0", "[2001:db8::1]:1234", "[fe80::1%12]:51820", "[fe80::1%Ethernet 3]:51820"] {
            let imported = import_into(&dir, "valid", &config(endpoint)).unwrap();
            assert_eq!(imported.config.peers[0].endpoint.as_deref(), Some(endpoint));
            let saved = fs::read_to_string(dir.join(format!("{}.conf", imported.name))).unwrap();
            assert_eq!(WireGuardConfig::parse(&saved).unwrap(), imported.config);
            assert_eq!(fs::read(&old_path).unwrap(), invalid.as_bytes());
        }
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
