//! waX VPN - WireGuard client for Windows. egui front end over wax-tunnel.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod orb;
mod settings;

use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use eframe::egui::{self, Align, Color32, Layout, RichText, Vec2};

use settings::Settings;
use wax_core::{profile, Profile};
use wax_tunnel::{ConnectOptions, Tunnel, TunnelStats};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug)]
pub enum ConnState {
    Disconnected,
    Connecting,
    Connected { since: Instant },
    Disconnecting,
    Failed(String),
}

enum AppEvent {
    Connected,
    ConnectFailed(String),
    Disconnected,
    Stats(TunnelStats),
    WarpDone(Result<String, String>),
}

pub struct WaxApp {
    settings: Settings,
    profiles: Vec<Profile>,
    selected: Option<usize>,
    state: ConnState,
    stats: TunnelStats,
    tunnel: Arc<Mutex<Option<Tunnel>>>,
    events_tx: std::sync::mpsc::Sender<AppEvent>,
    events_rx: Receiver<AppEvent>,
    start: Instant,
    status_line: String,
    is_admin: bool,
    warp_registered: bool,
    warp_summary: String,
    warp_tos: bool,
    warp_busy: bool,
    import_paste: String,
    import_error: Option<String>,
}

impl WaxApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_theme(egui::Theme::Dark);
        cc.egui_ctx.style_mut_of(egui::Theme::Dark, |style| {
            style.visuals.panel_fill = orb::BG;
            style.visuals.window_fill = orb::BG;
        });

        let (tx, rx) = channel();
        let settings = settings::load();
        let profiles = profile::load_profiles().unwrap_or_default();
        let selected = settings
            .last_profile
            .as_ref()
            .and_then(|last| profiles.iter().position(|p| &p.name == last))
            .or(if profiles.is_empty() { None } else { Some(0) });

        let warp_registered = matches!(wax_warp::load_account(), Ok(Some(_)));
        let mut app = Self {
            settings,
            profiles,
            selected,
            state: ConnState::Disconnected,
            stats: TunnelStats::default(),
            tunnel: Arc::new(Mutex::new(None)),
            events_tx: tx,
            events_rx: rx,
            start: Instant::now(),
            status_line: "Ready".to_string(),
            is_admin: wax_tunnel::is_elevated(),
            warp_registered,
            warp_summary: String::new(),
            warp_tos: false,
            warp_busy: false,
            import_paste: String::new(),
            import_error: None,
        };
        app.refresh_warp_summary();
        // Pick up a tunnel left running by an earlier instance.
        if let Ok(Some(stats)) = Tunnel::existing_status() {
            app.state = ConnState::Connected { since: Instant::now() };
            app.stats = stats;
            app.spawn_stats_loop();
        }
        app
    }

    fn refresh_warp_summary(&mut self) {
        match wax_warp::load_account() {
            Ok(Some(account)) => {
                self.warp_registered = true;
                self.warp_summary = format!("Registered - license {}", account.license);
            }
            _ => {
                self.warp_registered = false;
                self.warp_summary = "Not registered".to_string();
            }
        }
    }

    fn reload_profiles(&mut self) {
        let keep = self.selected.and_then(|i| self.profiles.get(i)).map(|p| p.name.clone());
        self.profiles = profile::load_profiles().unwrap_or_default();
        self.selected = keep
            .and_then(|name| self.profiles.iter().position(|p| p.name == name))
            .or(if self.profiles.is_empty() { None } else { Some(0) });
    }

    fn spawn_stats_loop(&self) {
        let tunnel = self.tunnel.clone();
        let tx = self.events_tx.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(1));
            let guard = tunnel.lock().unwrap();
            match guard.as_ref() {
                Some(t) => {
                    let stats = t.stats();
                    drop(guard);
                    if tx.send(AppEvent::Stats(stats)).is_err() {
                        break;
                    }
                }
                None => break,
            }
        });
    }

    fn connect(&mut self) {
        let Some(idx) = self.selected else {
            self.state = ConnState::Failed("Pick or import a profile first".into());
            return;
        };
        if !self.is_admin {
            self.state = ConnState::Failed(
                "Administrator rights are required to create the tunnel adapter".into(),
            );
            return;
        }
        let Some(profile) = self.profiles.get(idx).cloned() else { return };
        self.state = ConnState::Connecting;
        self.status_line = format!("Connecting to {}...", profile.name);
        let opts = ConnectOptions {
            kill_switch: self.settings.kill_switch,
            allow_lan: self.settings.allow_lan,
        };
        let tunnel = self.tunnel.clone();
        let tx = self.events_tx.clone();
        std::thread::spawn(move || match Tunnel::connect(&profile, &opts) {
            Ok(t) => {
                *tunnel.lock().unwrap() = Some(t);
                let _ = tx.send(AppEvent::Connected);
            }
            Err(e) => {
                let _ = tx.send(AppEvent::ConnectFailed(e.to_string()));
            }
        });
    }

    fn disconnect(&mut self) {
        self.state = ConnState::Disconnecting;
        self.status_line = "Disconnecting...".to_string();
        let tunnel = self.tunnel.clone();
        let tx = self.events_tx.clone();
        std::thread::spawn(move || {
            let taken = tunnel.lock().unwrap().take();
            if let Some(t) = taken {
                let _ = t.shutdown();
            } else {
                let _ = Tunnel::force_disconnect();
            }
            let _ = tx.send(AppEvent::Disconnected);
        });
    }

    fn restart_as_admin(&mut self) {
        #[cfg(windows)]
        {
            if let Ok(exe) = std::env::current_exe() {
                let script = format!("Start-Process -FilePath '{}' -Verb RunAs", exe.display());
                let _ = std::process::Command::new("powershell")
                    .args(["-NoProfile", "-Command", &script])
                    .spawn();
                std::process::exit(0);
            }
        }
    }

    fn toggle_autostart(&mut self) {
        #[cfg(windows)]
        {
            use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
            use winreg::RegKey;
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            if let Ok((run, _)) = hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run") {
                if self.settings.autostart {
                    if let Ok(exe) = std::env::current_exe() {
                        let _ = run.set_value("waX VPN", &format!("\"{}\"", exe.display()));
                    }
                } else {
                    let _: Result<(), std::io::Error> = run.delete_value("waX VPN");
                }
                let _ = run;
            }
        }
    }

    fn import_text(&mut self, name: String, text: &str) {
        match profile::import_conf(&name, text) {
            Ok(p) => {
                self.status_line = format!("Imported profile '{}'", p.name);
                self.import_error = None;
                self.reload_profiles();
            }
            Err(e) => self.import_error = Some(e.to_string()),
        }
    }

    fn handle_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.events_rx.try_recv() {
            match event {
                AppEvent::Connected => {
                    self.state = ConnState::Connected { since: Instant::now() };
                    self.status_line = "Tunnel is up".to_string();
                    if let Some(idx) = self.selected {
                        if let Some(p) = self.profiles.get(idx) {
                            self.settings.last_profile = Some(p.name.clone());
                            settings::save(&self.settings);
                        }
                    }
                    self.spawn_stats_loop();
                }
                AppEvent::ConnectFailed(e) => {
                    self.state = ConnState::Failed(e.clone());
                    self.status_line = e;
                }
                AppEvent::Disconnected => {
                    self.state = ConnState::Disconnected;
                    self.status_line = "Disconnected".to_string();
                    self.stats = TunnelStats::default();
                }
                AppEvent::Stats(s) => self.stats = s,
                AppEvent::WarpDone(Ok(msg)) => {
                    self.warp_busy = false;
                    self.status_line = msg;
                    self.refresh_warp_summary();
                    self.reload_profiles();
                }
                AppEvent::WarpDone(Err(e)) => {
                    self.warp_busy = false;
                    self.status_line = format!("WARP: {e}");
                }
            }
        }
        ctx.request_repaint_after(Duration::from_millis(66));
    }

    fn warp_register(&mut self) {
        self.warp_busy = true;
        self.status_line = "Registering WARP device...".to_string();
        let tx = self.events_tx.clone();
        std::thread::spawn(move || {
            let result = (|| -> Result<String, String> {
                if wax_warp::load_account().map_err(|e| e.to_string())?.is_some() {
                    return Err("already registered".to_string());
                }
                let private = wax_core::keys::generate_private_key();
                let public = wax_core::keys::public_key_from_private(&private);
                let client = wax_warp::WarpClient::new().map_err(|e| e.to_string())?;
                let reg = client
                    .register(&wax_core::keys::key_to_base64(&public), "PC", true)
                    .map_err(|e| e.to_string())?;
                let account = wax_warp::WarpAccount {
                    device_id: reg.id.clone(),
                    token: reg.token.clone(),
                    license: reg.account.license.clone().unwrap_or_default(),
                    private_key: wax_core::keys::key_to_base64(&private),
                    model: "PC".to_string(),
                    created_at: String::new(),
                    registration: reg,
                };
                wax_warp::save_account(&account).map_err(|e| e.to_string())?;
                let profile =
                    wax_warp::store::warp_profile(&account, "Cloudflare WARP").map_err(|e| e.to_string())?;
                profile::save_profile(&profile).map_err(|e| e.to_string())?;
                Ok("WARP registered; profile 'Cloudflare WARP' saved".to_string())
            })();
            let _ = tx.send(AppEvent::WarpDone(result));
        });
    }

    fn warp_unregister(&mut self) {
        self.warp_busy = true;
        let tx = self.events_tx.clone();
        std::thread::spawn(move || {
            let result = (|| -> Result<String, String> {
                let account = wax_warp::load_account()
                    .map_err(|e| e.to_string())?
                    .ok_or("not registered")?;
                let client = wax_warp::WarpClient::new().map_err(|e| e.to_string())?;
                let _ = client.delete_device(&account.device_id, &account.token);
                wax_warp::delete_account().map_err(|e| e.to_string())?;
                let _ = profile::delete_profile("Cloudflare WARP");
                Ok("WARP registration removed".to_string())
            })();
            let _ = tx.send(AppEvent::WarpDone(result));
        });
    }
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

fn fmt_duration(d: Duration) -> String {
    let s = d.as_secs();
    if s < 60 {
        format!("{s}s")
    } else if s < 3600 {
        format!("{}m {:02}s", s / 60, s % 60)
    } else {
        format!("{}h {:02}m", s / 3600, (s % 3600) / 60)
    }
}

impl eframe::App for WaxApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let ctx = &ctx;
        self.handle_events(ctx);

        // Drag & drop .conf import
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        for file in dropped {
            let path = file.path().to_path_buf();
            if path.extension().and_then(|e| e.to_str()) == Some("conf") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("imported")
                    .to_string();
                if let Ok(text) = std::fs::read_to_string(&path) {
                    self.import_text(name, &text);
                }
            }
        }

        let t = self.start.elapsed().as_secs_f32();

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(orb::BG))
            .show(ui, |ui| {
                ui.add_space(10.0);
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.label(
                        RichText::new("waX VPN")
                            .size(26.0)
                            .color(Color32::from_rgb(226, 232, 240))
                            .strong(),
                    );
                    ui.label(
                        RichText::new(format!("v{VERSION} - WireGuard, in Rust"))
                            .size(11.0)
                            .color(Color32::from_rgb(100, 116, 139)),
                    );
                });

                ui.add_space(6.0);
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    let response = orb::paint_orb(ui, &self.state, t, self.settings.reduced_motion);
                    if response.clicked() {
                        match self.state {
                            ConnState::Disconnected | ConnState::Failed(_) => self.connect(),
                            ConnState::Connected { .. } => self.disconnect(),
                            _ => {}
                        }
                    }
                });

                let failed_msg = if let ConnState::Failed(msg) = &self.state { Some(msg.clone()) } else { None };
                if let Some(msg) = failed_msg {
                    ui.add_space(4.0);
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        ui.label(RichText::new(&msg).size(12.0).color(orb::BAD));
                        if !self.is_admin {
                            if ui.button("Restart as administrator").clicked() {
                                self.restart_as_admin();
                            }
                        }
                    });
                }

                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.label(
                        RichText::new(&self.status_line)
                            .size(12.0)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                });

                if let ConnState::Connected { since } = self.state {
                    ui.add_space(2.0);
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        let hs = self
                            .stats
                            .last_handshake
                            .and_then(|h| SystemTime::now().duration_since(h).ok())
                            .map(fmt_duration)
                            .unwrap_or_else(|| "-".into());
                        ui.label(
                            RichText::new(format!(
                                "{} up  -  rx {}  -  tx {}  -  handshake {} ago",
                                fmt_duration(since.elapsed()),
                                fmt_bytes(self.stats.rx_bytes),
                                fmt_bytes(self.stats.tx_bytes),
                                hs
                            ))
                            .size(12.0)
                            .color(orb::GOOD),
                        );
                    });
                }

                ui.add_space(8.0);
                // Profile picker
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Profile").color(Color32::from_rgb(148, 163, 184)));
                    let current = self
                        .selected
                        .and_then(|i| self.profiles.get(i))
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| "none".to_string());
                    egui::ComboBox::from_id_salt("profile-picker")
                        .selected_text(current)
                        .show_ui(ui, |ui| {
                            for (i, p) in self.profiles.iter().enumerate() {
                                ui.selectable_value(&mut self.selected, Some(i), &p.name);
                            }
                        });
                    if ui.small_button("Delete").clicked() {
                        if let Some(i) = self.selected {
                            if let Some(p) = self.profiles.get(i) {
                                let _ = profile::delete_profile(&p.name);
                                self.reload_profiles();
                            }
                        }
                    }
                });

                ui.add_space(4.0);
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.collapsing("Settings", |ui| {
                        if ui
                            .checkbox(&mut self.settings.kill_switch, "Kill switch (block traffic outside the tunnel)")
                            .changed()
                        {
                            settings::save(&self.settings);
                        }
                        if ui
                            .checkbox(&mut self.settings.allow_lan, "Allow local network while kill switch is on")
                            .changed()
                        {
                            settings::save(&self.settings);
                        }
                        if ui.checkbox(&mut self.settings.autostart, "Start waX VPN with Windows").changed() {
                            self.toggle_autostart();
                            settings::save(&self.settings);
                        }
                        if ui
                            .checkbox(&mut self.settings.reduced_motion, "Reduced motion (calm interface)")
                            .changed()
                        {
                            settings::save(&self.settings);
                        }
                        ui.label(
                            RichText::new(if self.is_admin {
                                "Running with administrator rights"
                            } else {
                                "Not elevated - connecting will need a restart as administrator"
                            })
                            .size(11.0)
                            .color(Color32::from_rgb(100, 116, 139)),
                        );
                    });

                    ui.collapsing("Cloudflare WARP (opt-in)", |ui| {
                        ui.label(
                            RichText::new("waX VPN is not affiliated with Cloudflare. Registering creates a Cloudflare WARP account.")
                                .size(11.0)
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                        ui.label(RichText::new(&self.warp_summary).size(12.0));
                        if self.warp_registered {
                            if ui
                                .add_enabled(!self.warp_busy, egui::Button::new("Unregister WARP"))
                                .clicked()
                            {
                                self.warp_unregister();
                            }
                        } else {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut self.warp_tos, "I accept");
                                ui.hyperlink_to(
                                    "Cloudflare's Terms of Service",
                                    "https://www.cloudflare.com/application/terms/",
                                );
                            });
                            if ui
                                .add_enabled(
                                    self.warp_tos && !self.warp_busy,
                                    egui::Button::new("Register WARP and create profile"),
                                )
                                .clicked()
                            {
                                self.warp_register();
                            }
                        }
                    });

                    ui.collapsing("Import a profile", |ui| {
                        ui.label(
                            RichText::new("Drop a .conf file anywhere in this window, or paste config text:")
                                .size(11.0)
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                        egui::ScrollArea::vertical()
                            .id_salt("import-scroll")
                            .max_height(120.0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.import_paste)
                                        .hint_text("[Interface]\nPrivateKey = ...")
                                        .desired_width(f32::INFINITY),
                                );
                            });
                        if ui.button("Import pasted config").clicked() {
                            let text = self.import_paste.trim().to_string();
                            if !text.is_empty() {
                                self.import_text(format!("Imported {}", chrono_free_stamp()), &text);
                                self.import_paste.clear();
                            }
                        }
                        if let Some(err) = &self.import_error {
                            ui.label(RichText::new(err).size(11.0).color(orb::BAD));
                        }
                    });

                    ui.add_space(8.0);
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        ui.label(
                            RichText::new("Not affiliated with Cloudflare or the WireGuard project")
                                .size(10.0)
                                .color(Color32::from_rgb(71, 85, 105)),
                        );
                    });
                    let _ = Vec2::ZERO; // keep egui::Vec2 import used
                });
            });
    }
}

fn chrono_free_stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", secs % 100_000)
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("waX VPN")
            .with_app_id("wax-vpn")
            .with_inner_size([430.0, 760.0])
            .with_min_inner_size([400.0, 700.0])
            .with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "waX VPN",
        options,
        Box::new(|cc| Ok(Box::new(WaxApp::new(cc)))),
    )
}
