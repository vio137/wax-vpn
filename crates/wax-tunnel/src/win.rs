//! Windows tunnel backend on the official WireGuardNT kernel driver.

use std::net::ToSocketAddrs;
use std::path::PathBuf;
use std::process::Command;

use wireguard_nt::{Adapter, SetInterface, SetPeer, Wireguard};

use wax_core::Profile;

use crate::{ConnectOptions, TunnelError, TunnelStats, ADAPTER_NAME, ADAPTER_POOL};

const KILLSWITCH_RULE_PREFIX: &str = "waX VPN KillSwitch";

/// Locate wireguard.dll shipped side-by-side with the app.
fn dll_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            out.push(dir.join("wireguard.dll"));
            out.push(dir.join("drivers").join("wireguard.dll"));
        }
    }
    out.push(PathBuf::from(r"C:\Program Files\waX VPN\wireguard.dll"));
    out.push(PathBuf::from(r"vendor\wireguard-nt\amd64\wireguard.dll"));
    out
}

fn load_driver() -> Result<Wireguard, TunnelError> {
    for path in dll_candidates() {
        if path.exists() {
            return unsafe { wireguard_nt::load_from_path(&path) }
                .map_err(|e| TunnelError::Driver(format!("failed to load {}: {e}", path.display())));
        }
    }
    Err(TunnelError::Driver(
        "wireguard.dll not found next to the app; reinstall waX VPN".to_string(),
    ))
}

fn run(cmd: &str, args: &[&str]) -> Result<(), TunnelError> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let output = Command::new(cmd)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| TunnelError::Command(format!("{cmd}: {e}")))?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Err(TunnelError::Command(format!(
            "{cmd} {} failed: {} {}",
            args.join(" "),
            stderr,
            stdout
        )))
    }
}

fn powershell(script: &str) -> Result<(), TunnelError> {
    run("powershell", &["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script])
}

fn set_dns(adapter: &str, dns: &[std::net::IpAddr]) -> Result<(), TunnelError> {
    let v4: Vec<String> = dns.iter().filter(|d| d.is_ipv4()).map(|d| d.to_string()).collect();
    let v6: Vec<String> = dns.iter().filter(|d| d.is_ipv6()).map(|d| d.to_string()).collect();
    if let Some(first) = v4.first() {
        run("netsh", &["interface", "ipv4", "set", "dnsservers", &format!("name={adapter}"), "static", first, "primary"])?;
        for (i, d) in v4.iter().enumerate().skip(1) {
            run("netsh", &["interface", "ipv4", "add", "dnsservers", adapter, d, &format!("index={}", i + 1)])?;
        }
    }
    if let Some(first) = v6.first() {
        run("netsh", &["interface", "ipv6", "set", "dnsservers", &format!("name={adapter}"), "static", first, "primary"])?;
        for (i, d) in v6.iter().enumerate().skip(1) {
            run("netsh", &["interface", "ipv6", "add", "dnsservers", adapter, d, &format!("index={}", i + 1)])?;
        }
    }
    Ok(())
}

fn run_out(cmd: &str, args: &[&str]) -> Result<String, TunnelError> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let output = Command::new(cmd)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| TunnelError::Command(format!("{cmd}: {e}")))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(TunnelError::Command(format!("{cmd} {} failed: {stderr}", args.join(" "))))
    }
}

fn reset_dns(adapter: &str) -> Result<(), TunnelError> {
    // A confirmed-absent adapter is already clean: Windows reclaimed it and
    // there is no DNS entry left to reset (netsh errors on a nonexistent
    // interface name). Only a confirmed absence skips the reset - a failed
    // or garbled probe is reported, never treated as clean.
    let probe = run_out(
        "powershell",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &format!("if (Get-NetAdapter -Name '{adapter}' -ErrorAction SilentlyContinue) {{ 'PRESENT' }} else {{ 'ABSENT' }}"),
        ],
    );
    match probe {
        Ok(s) if s.contains("ABSENT") => return Ok(()),
        Ok(s) if s.contains("PRESENT") => {}
        Ok(other) => {
            return Err(TunnelError::Message(format!(
                "could not determine adapter presence (unexpected probe output: {other})"
            )))
        }
        Err(e) => {
            return Err(TunnelError::Message(format!(
                "could not determine adapter presence: {e}"
            )))
        }
    }
    let mut problems = Vec::new();
    if let Err(e) = run("netsh", &["interface", "ipv4", "set", "dnsservers", &format!("name={adapter}"), "dhcp"]) {
        problems.push(format!("ipv4 dns reset: {e}"));
    }
    if let Err(e) = run("netsh", &["interface", "ipv6", "set", "dnsservers", &format!("name={adapter}"), "dhcp"]) {
        problems.push(format!("ipv6 dns reset: {e}"));
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(TunnelError::Message(problems.join("; ")))
    }
}

pub fn disable_kill_switch() -> Result<(), TunnelError> {
    powershell(&format!(
        "Remove-NetFirewallRule -DisplayName '{KILLSWITCH_RULE_PREFIX}*' -ErrorAction SilentlyContinue"
    ))
}

/// A live tunnel. Dropping without calling `shutdown` still closes the
/// driver handle, but `shutdown` also restores DNS and the firewall.
pub struct Tunnel {
    adapter: Adapter,
    #[allow(dead_code)]
    wireguard: Wireguard,
    opts: ConnectOptions,
}

fn parse_endpoint(endpoint: &str) -> Result<(String, u16), TunnelError> {
    // Accept host:port, [v6]:port.
    if let Some(rest) = endpoint.strip_prefix('[') {
        if let Some((host, port)) = rest.split_once("]:") {
            let port: u16 = port.parse().map_err(|_| TunnelError::Resolve(endpoint.into(), "bad port".into()))?;
            return Ok((host.to_string(), port));
        }
    }
    match endpoint.rsplit_once(':') {
        Some((host, port)) => {
            let port: u16 = port.parse().map_err(|_| TunnelError::Resolve(endpoint.into(), "bad port".into()))?;
            Ok((host.to_string(), port))
        }
        None => Err(TunnelError::Resolve(endpoint.into(), "missing port".into())),
    }
}

impl Tunnel {
    pub fn connect(profile: &Profile, opts: &ConnectOptions) -> Result<Tunnel, TunnelError> {
        Self::connect_progress(profile, opts, |_| {})
    }

    /// Like [`Self::connect`], but reports each stage so callers can show
    /// honest in-progress UI instead of a dead spinner.
    pub fn connect_progress(
        profile: &Profile,
        opts: &ConnectOptions,
        mut progress: impl FnMut(&'static str),
    ) -> Result<Tunnel, TunnelError> {
        if opts.kill_switch {
            return Err(TunnelError::Message("Kill switch is unavailable in v0.1.0: the previous block-all firewall implementation is disabled. No tunnel or firewall changes were made.".into()));
        }
        progress("Loading WireGuard driver");
        let wireguard = load_driver()?;
        progress("Preparing the waX adapter");
        let adapter = match Adapter::open(&wireguard, ADAPTER_NAME) {
            Ok(a) => a,
            Err(_) => Adapter::create(&wireguard, ADAPTER_POOL, ADAPTER_NAME, None)
                .map_err(|e| TunnelError::Driver(format!("create adapter: {e}")))?,
        };

        progress("Resolving the endpoint");
        let cfg = &profile.config;
        let mut peers = Vec::new();
        for peer in &cfg.peers {
            let endpoint_str = peer
                .endpoint
                .clone()
                .ok_or_else(|| TunnelError::Message("peer has no Endpoint".into()))?;
            let (host, port) = parse_endpoint(&endpoint_str)?;
            let addr = (host.as_str(), port)
                .to_socket_addrs()
                .map_err(|e| TunnelError::Resolve(endpoint_str.clone(), e.to_string()))?
                .next()
                .ok_or_else(|| TunnelError::Resolve(endpoint_str.clone(), "no address".into()))?;
            peers.push(SetPeer {
                public_key: Some(peer.public_key),
                preshared_key: peer.preshared_key,
                keep_alive: peer.persistent_keepalive,
                endpoint: addr,
                allowed_ips: peer.allowed_ips.clone(),
            });
        }

        let set_if = SetInterface {
            listen_port: cfg.interface.listen_port,
            public_key: None,
            private_key: Some(cfg.interface.private_key),
            peers,
        };
        progress("Configuring the tunnel");
        adapter
            .set_config(&set_if)
            .map_err(|e| TunnelError::Driver(format!("set config: {e}")))?;
        progress("Setting routes");
        adapter
            .set_default_route(&cfg.interface.addresses, &set_if)
            .map_err(|e| TunnelError::Driver(format!("set routes: {e}")))?;

        if !cfg.interface.dns.is_empty() {
            progress("Applying tunnel DNS");
            set_dns(ADAPTER_NAME, &cfg.interface.dns)?;
        }

        progress("Starting the tunnel");
        adapter.up().map_err(|e| TunnelError::Driver(format!("bring adapter up: {e}")))?;
        log::info!("tunnel up for profile {}", profile.name);
        Ok(Tunnel { adapter, wireguard, opts: opts.clone() })
    }

    pub fn stats(&self) -> TunnelStats {
        let cfg = self.adapter.get_config();
        let mut stats = TunnelStats::default();
        for peer in cfg.peers {
            stats.rx_bytes += peer.rx_bytes;
            stats.tx_bytes += peer.tx_bytes;
            if let Some(hs) = peer.last_handshake {
                if stats.last_handshake.map(|cur| hs > cur).unwrap_or(true) {
                    stats.last_handshake = Some(hs);
                }
            }
        }
        stats
    }

    pub fn is_up(&self) -> bool {
        self.adapter.is_up().unwrap_or(false)
    }

    /// Bring the tunnel down, restoring DNS and removing kill-switch rules.
    pub fn shutdown(self) -> Result<(), TunnelError> {
        // Run every cleanup step even when one fails, then report all
        // failures together - never a silent partial cleanup.
        let mut problems = Vec::new();
        if let Err(e) = self.adapter.down() {
            problems.push(format!("adapter down: {e}"));
        }
        if let Err(e) = reset_dns(ADAPTER_NAME) {
            problems.push(e.to_string());
        }
        if self.opts.kill_switch {
            if let Err(e) = disable_kill_switch() {
                problems.push(format!("kill-switch rules remain: {e}"));
            }
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(TunnelError::Message(problems.join("; ")))
        }
    }

    /// Down any leftover waX VPN adapter from a previous run (CLI disconnect).
    pub fn force_disconnect() -> Result<(), TunnelError> {
        let wireguard = load_driver()?;
        let mut problems = Vec::new();
        if let Ok(adapter) = Adapter::open(&wireguard, ADAPTER_NAME) {
            if let Err(e) = adapter.down() {
                problems.push(format!("adapter down: {e}"));
            }
        }
        if let Err(e) = reset_dns(ADAPTER_NAME) {
            problems.push(e.to_string());
        }
        if let Err(e) = disable_kill_switch() {
            problems.push(format!("kill-switch rules remain: {e}"));
        }
        if !problems.is_empty() {
            return Err(TunnelError::Message(problems.join("; ")));
        }
        Ok(())
    }

    /// Whether a waX VPN adapter exists and is up (for CLI status).
    pub fn existing_status() -> Result<Option<TunnelStats>, TunnelError> {
        let wireguard = load_driver()?;
        match Adapter::open(&wireguard, ADAPTER_NAME) {
            Ok(adapter) => {
                if adapter.is_up().unwrap_or(false) {
                    let cfg = adapter.get_config();
                    let mut stats = TunnelStats::default();
                    for peer in cfg.peers {
                        stats.rx_bytes += peer.rx_bytes;
                        stats.tx_bytes += peer.tx_bytes;
                        if let Some(hs) = peer.last_handshake {
                            if stats.last_handshake.map(|cur| hs > cur).unwrap_or(true) {
                                stats.last_handshake = Some(hs);
                            }
                        }
                    }
                    Ok(Some(stats))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }
}
