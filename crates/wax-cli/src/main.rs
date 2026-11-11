//! wax.exe - the waX VPN command line interface.


use clap::{Parser, Subcommand};

use wax_core::{keys, profile};
use wax_tunnel::{ConnectOptions, Tunnel};

#[derive(Parser)]
#[command(name = "wax", version = env!("WAX_BUILD_VERSION"), about = "waX VPN - WireGuard client for Windows", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print allowlisted local diagnostics JSON, without profiles or secrets
    Diagnostics,
    /// Generate a new WireGuard keypair
    Keygen,
    /// List saved profiles
    Profiles,
    /// Import a .conf file as a profile
    Import {
        /// Path to the .conf file
        file: String,
        /// Name for the profile (defaults to file name)
        #[arg(long)]
        name: Option<String>,
    },
    /// Show a profile (private key redacted)
    Show { name: String },
    /// Delete a profile
    Remove { name: String },
    /// Connect a profile (requires administrator)
    Connect {
        name: String,
        /// Configure and bring the tunnel up, then exit (used by automation)
        #[arg(long)]
        no_wait: bool,
    },
    /// Disconnect the running tunnel
    Disconnect,
    /// Tunnel status and counters
    Status,
    /// Cloudflare WARP commands (opt-in)
    #[command(subcommand)]
    Warp(WarpCommands),
}

#[derive(Subcommand)]
enum WarpCommands {
    /// Register a new WARP account. Implies acceptance of Cloudflare's Terms
    /// of Service (https://www.cloudflare.com/application/terms/) only when
    /// --accept-tos is passed.
    Register {
        #[arg(long)]
        accept_tos: bool,
        /// Device model label shown in the 1.1.1.1 app
        #[arg(long, default_value = "PC")]
        model: String,
    },
    /// Show stored WARP registration details
    Account,
    /// Generate the "Cloudflare WARP" profile without replacing an existing file
    Generate,
    /// Delete the WARP registration (server-side and local)
    Unregister,
}

fn redact_conf(text: &str) -> String {
    text.lines()
        .map(|line| {
            let l = line.trim_start();
            if l.starts_with("PrivateKey") {
                "PrivateKey = <redacted>".to_string()
            } else if l.starts_with("PresharedKey") {
                "PresharedKey = <redacted>".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn fmt_bytes(n: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut v = n as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    format!("{v:.1} {}", UNITS[i])
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Keygen => {
            let private = keys::generate_private_key();
            let public = keys::public_key_from_private(&private);
            println!("PrivateKey = {}", keys::key_to_base64(&private));
            println!("PublicKey  = {}", keys::key_to_base64(&public));
        }
        Commands::Profiles => {
            let profiles = profile::load_profiles().map_err(|e| e.to_string())?;
            if profiles.is_empty() {
                println!("No profiles yet. Import one with `wax import <file.conf>` or set up WARP with `wax warp register --accept-tos`.");
            }
            for p in profiles {
                let peers = p.config.peers.len();
                let endpoint = p
                    .config
                    .peers
                    .first()
                    .and_then(|peer| peer.endpoint.clone())
                    .unwrap_or_else(|| "-".to_string());
                println!("{:<28} {} peer(s)  endpoint {}", p.name, peers, endpoint);
            }
        }
        Commands::Import { file, name } => {
            let text = std::fs::read_to_string(&file).map_err(|e| format!("read {file}: {e}"))?;
            let name = name.unwrap_or_else(|| {
                std::path::Path::new(&file)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("imported")
                    .to_string()
            });
            let p = profile::import_conf(&name, &text).map_err(|e| e.to_string())?;
            println!("Imported profile '{}'.", p.name);
        }
        Commands::Show { name } => {
            let p = profile::load_profile(&name).map_err(|e| e.to_string())?;
            println!("{}", redact_conf(&p.config.to_conf_string()));
        }
        Commands::Remove { name } => {
            profile::delete_profile(&name).map_err(|e| e.to_string())?;
            println!("Removed profile '{name}'.");
        }
        Commands::Connect { name, no_wait } => {
            if !wax_tunnel::is_elevated() {
                return Err("connecting needs administrator rights - re-run from an elevated terminal or use the waX VPN app, which offers a Restart as administrator button".into());
            }
            if no_wait {
                return Err("`--no-wait` is not safe yet: a tunnel stops when its owning process exits, so a fire-and-forget connect would die instantly. Omit --no-wait and keep this terminal open, or use the waX VPN app, which holds the tunnel while it runs. A proper background mode is planned.".into());
            }
            let p = profile::load_profile(&name).map_err(|e| e.to_string())?;
            let opts = ConnectOptions::default();
            let stop = ConsoleStop::new()?;
            let tunnel = Tunnel::connect(&p, &opts).map_err(|e| e.to_string())?;
            println!("Connected to '{}'. Tunnel adapter '{}' is up.", p.name, wax_tunnel::ADAPTER_NAME);
            println!("Press Ctrl+C or Enter to disconnect cleanly.");
            // Keep the owning process alive, then use the same cleanup path
            // for a console interrupt and the existing Enter/EOF exit.
            stop.wait();
            tunnel.shutdown().map_err(|e| e.to_string())?;
            println!("Disconnected.");
        }
        Commands::Diagnostics => {
            println!("{}", wax_diagnostics::report(env!("WAX_BUILD_VERSION"), wax_tunnel::is_elevated(), Tunnel::existing_status()));
        }
        Commands::Disconnect => {
            Tunnel::force_disconnect().map_err(|e| e.to_string())?;
            println!("Disconnected.");
        }
        Commands::Status => match Tunnel::existing_status().map_err(|e| e.to_string())? {
            Some(stats) => {
                let now = std::time::SystemTime::now();
                let label = match wax_diagnostics::health::health(stats.last_handshake, now) {
                    wax_diagnostics::health::Health::AwaitingHandshake => "Tunnel ready",
                    wax_diagnostics::health::Health::RecentHandshake => "Connected",
                    wax_diagnostics::health::Health::OlderHandshake => "Tunnel up",
                };
                println!("{label}. rx {} / tx {}", fmt_bytes(stats.rx_bytes), fmt_bytes(stats.tx_bytes));
                println!("Internet reachability is not tested.");
                match stats.last_handshake {
                    Some(t) => {
                        match now.duration_since(t) {
                            Ok(age) => println!("Last handshake {}s ago. Idle peers may not handshake.", age.as_secs()),
                            Err(_) => println!("Handshake timestamp is in the future; peer state is uncertain."),
                        }
                    }
                    None => println!("No handshake yet."),
                }
            }
            None => println!("Not connected."),
        },
        Commands::Warp(cmd) => run_warp(cmd)?,
    }
    Ok(())
}

static RUNNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn SetConsoleCtrlHandler(handler: Option<unsafe extern "system" fn(u32) -> i32>, add: i32) -> i32;
}

#[cfg(windows)]
unsafe extern "system" fn console_interrupt(event: u32) -> i32 {
    // CTRL_C_EVENT / CTRL_BREAK_EVENT only. Close/logoff/shutdown have short
    // OS deadlines and are not advertised as graceful cleanup paths.
    if event > 1 { return 0; }
    RUNNING.store(false, std::sync::atomic::Ordering::Release);
    1
}

struct ConsoleStop;
impl ConsoleStop {
    fn new() -> Result<Self, String> {
        RUNNING.store(true, std::sync::atomic::Ordering::Release);
        #[cfg(windows)]
        if unsafe { SetConsoleCtrlHandler(Some(console_interrupt), 1) } == 0 {
            return Err("Could not install console interrupt handling; no tunnel was started".into());
        }
        Ok(Self)
    }
    fn wait(&self) {
        std::thread::spawn(|| {
            let mut line = String::new();
            let _ = std::io::stdin().read_line(&mut line);
            RUNNING.store(false, std::sync::atomic::Ordering::Release);
        });
        while RUNNING.load(std::sync::atomic::Ordering::Acquire) {
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }
}
impl Drop for ConsoleStop {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe { SetConsoleCtrlHandler(Some(console_interrupt), 0); }
    }
}

#[cfg(all(windows, test))]
mod console_tests {
    use super::*;
    #[test]
    fn interrupt_only_requests_existing_cleanup_for_ctrl_c_and_break() {
        for event in [2, 5, 6, 99, 0, 1] {
            RUNNING.store(true, std::sync::atomic::Ordering::Release);
            assert_eq!(unsafe { console_interrupt(event) }, i32::from(event <= 1));
            assert_eq!(RUNNING.load(std::sync::atomic::Ordering::Acquire), event > 1);
        }
    }
}

fn ensure_warp_profile_available(path: &std::path::Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Err("the reserved 'Cloudflare WARP' profile path is occupied; keep or move it before generating a new profile".into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("could not check the reserved WARP profile: {error}")),
    }
}

fn run_warp(cmd: WarpCommands) -> Result<(), String> {
    match cmd {
        WarpCommands::Register { accept_tos, model } => {
            if !accept_tos {
                return Err("registering creates a Cloudflare WARP account and implies accepting Cloudflare's Terms of Service (https://www.cloudflare.com/application/terms/). Re-run with --accept-tos to confirm.".into());
            }
            ensure_warp_profile_available(&profile::profile_path("Cloudflare WARP"))?;
            if wax_warp::load_account().map_err(|e| e.to_string())?.is_some() {
                return Err("a WARP registration already exists; run `wax warp unregister` first".into());
            }
            let private = keys::generate_private_key();
            let public = keys::public_key_from_private(&private);
            let client = wax_warp::WarpClient::new().map_err(|e| e.to_string())?;
            println!("Registering a new WARP device...");
            let reg = client
                .register(&keys::key_to_base64(&public), &model, true)
                .map_err(|e| e.to_string())?;
            let account = wax_warp::WarpAccount {
                device_id: reg.id.clone(),
                token: reg.token.clone(),
                license: reg.account.license.clone().unwrap_or_default(),
                private_key: keys::key_to_base64(&private),
                model,
                created_at: String::new(),
                registration: reg,
            };
            wax_warp::save_account(&account).map_err(|e| e.to_string())?;
            println!("Registered. License: {}...", &account.license.chars().take(6).collect::<String>());
            let profile = wax_warp::store::warp_profile(&account, "Cloudflare WARP").map_err(|e| e.to_string())?;
            profile::create_profile(&profile).map_err(|e| format!("could not create the WARP profile without replacement: {e}. Local account kept; keep or move the existing profile and retry `wax warp generate`"))?;
            println!("Saved profile 'Cloudflare WARP'. Connect with `wax connect \"Cloudflare WARP\"`.");
        }
        WarpCommands::Account => {
            let account = wax_warp::load_account().map_err(|e| e.to_string())?.ok_or("not registered")?;
            println!("Device id: {}", account.device_id);
            println!("License:   {}", account.license);
            println!("Model:     {}", account.model);
            let client = wax_warp::WarpClient::new().map_err(|e| e.to_string())?;
            match client.account(&account.device_id, &account.token) {
                Ok(a) => {
                    println!("Type:      {}", a.account_type);
                    if let Some(rc) = a.referral_count {
                        println!("Referrals: {rc}");
                    }
                }
                Err(e) => println!("(live account lookup failed: {e})"),
            }
        }
        WarpCommands::Generate => {
            ensure_warp_profile_available(&profile::profile_path("Cloudflare WARP"))?;
            let account = wax_warp::load_account().map_err(|e| e.to_string())?.ok_or("not registered")?;
            let profile = wax_warp::store::warp_profile(&account, "Cloudflare WARP").map_err(|e| e.to_string())?;
            profile::create_profile(&profile).map_err(|e| format!("could not create the WARP profile without replacement: {e}. Local account kept; keep or move the existing profile and retry `wax warp generate`"))?;
            println!("Saved profile 'Cloudflare WARP'.");
        }
        WarpCommands::Unregister => {
            let account = wax_warp::load_account().map_err(|e| e.to_string())?.ok_or("not registered")?;
            let client = wax_warp::WarpClient::new().map_err(|e| e.to_string())?;
            client.delete_device(&account.device_id, &account.token)
                .map_err(|e| format!("Could not remove remote registration: {e}. Local account kept for retry"))?;
            wax_warp::delete_account().map_err(|e| e.to_string())?;
            let _ = profile::delete_profile("Cloudflare WARP");
            println!("WARP registration removed.");
        }
    }
    Ok(())
}

fn main() {
    env_logger_init();
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn env_logger_init() {}

#[cfg(test)]
mod warp_profile_guard_tests {
    use super::*;
    #[test]
    fn occupied_reserved_path_is_rejected_without_changing_it() {
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("wax-warp-guard-{}-{stamp}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("Cloudflare WARP.conf");
        assert!(ensure_warp_profile_available(&path).is_ok());
        std::fs::write(&path, b"keep original").unwrap();
        assert!(ensure_warp_profile_available(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"keep original");
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert!(ensure_warp_profile_available(&path).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
