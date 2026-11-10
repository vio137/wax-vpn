//! wax.exe - the waX VPN command line interface.

mod diagnostics;

use clap::{Parser, Subcommand};

use wax_core::{keys, profile};
use wax_tunnel::{ConnectOptions, Tunnel};

#[derive(Parser)]
#[command(name = "wax", version, about = "waX VPN - WireGuard client for Windows", long_about = None)]
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
    /// Generate/update the "Cloudflare WARP" profile from the registration
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
            let tunnel = Tunnel::connect(&p, &opts).map_err(|e| e.to_string())?;
            println!("Connected to '{}'. Tunnel adapter '{}' is up.", p.name, wax_tunnel::ADAPTER_NAME);
            println!("Press Ctrl+C to disconnect.");
            // Keep the process (and driver handle) alive; Ctrl+C triggers shutdown.
            let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
            let r = running.clone();
            ctrlc_like(move || r.store(false, std::sync::atomic::Ordering::SeqCst));
            while running.load(std::sync::atomic::Ordering::SeqCst) {
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
            tunnel.shutdown().map_err(|e| e.to_string())?;
            println!("Disconnected.");
        }
        Commands::Diagnostics => {
            println!("{}", diagnostics::report(wax_tunnel::is_elevated(), Tunnel::existing_status()));
        }
        Commands::Disconnect => {
            Tunnel::force_disconnect().map_err(|e| e.to_string())?;
            println!("Disconnected.");
        }
        Commands::Status => match Tunnel::existing_status().map_err(|e| e.to_string())? {
            Some(stats) => {
                println!("Connected. rx {} / tx {}", fmt_bytes(stats.rx_bytes), fmt_bytes(stats.tx_bytes));
                match stats.last_handshake {
                    Some(t) => {
                        let age = std::time::SystemTime::now().duration_since(t).map(|d| d.as_secs()).unwrap_or(0);
                        println!("Last handshake {}s ago.", age);
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

fn ctrlc_like(set: impl FnOnce() + Send + 'static) {
    // Minimal Ctrl+C handling without an extra dependency: spawn a thread that
    // waits for the process to receive CTRL_C via SetConsoleCtrlHandler would
    // need the windows crate; instead we poll stdin-close as a fallback. The
    // GUI is the primary interface; this keeps wax connect usable.
    std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = std::io::stdin().read_line(&mut buf);
        set();
    });
}

fn run_warp(cmd: WarpCommands) -> Result<(), String> {
    match cmd {
        WarpCommands::Register { accept_tos, model } => {
            if !accept_tos {
                return Err("registering creates a Cloudflare WARP account and implies accepting Cloudflare's Terms of Service (https://www.cloudflare.com/application/terms/). Re-run with --accept-tos to confirm.".into());
            }
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
            profile::save_profile(&profile).map_err(|e| e.to_string())?;
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
            let account = wax_warp::load_account().map_err(|e| e.to_string())?.ok_or("not registered")?;
            let profile = wax_warp::store::warp_profile(&account, "Cloudflare WARP").map_err(|e| e.to_string())?;
            profile::save_profile(&profile).map_err(|e| e.to_string())?;
            println!("Saved profile 'Cloudflare WARP'.");
        }
        WarpCommands::Unregister => {
            let account = wax_warp::load_account().map_err(|e| e.to_string())?.ok_or("not registered")?;
            let client = wax_warp::WarpClient::new().map_err(|e| e.to_string())?;
            if let Err(e) = client.delete_device(&account.device_id, &account.token) {
                eprintln!("warning: server-side delete failed ({e}); removing local registration anyway");
            }
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
