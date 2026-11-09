//! waX VPN - WireGuard client for Windows. egui front end over wax-tunnel.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod orb;
mod theme;
mod tray;
mod settings;

use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use eframe::egui::{self, Align, Layout, RichText};

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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Tab {
    Connect,
    Profiles,
    Features,
    Warp,
    About,
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
    tab: Tab,
    hidden: bool,
    #[cfg(windows)]
    tray: Option<tray::Tray>,
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
            tab: Tab::Connect,
            hidden: false,
            #[cfg(windows)]
            tray: tray::Tray::new(&cc.egui_ctx),
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
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x0800_0000;
                let _ = std::process::Command::new("powershell")
                    .args(["-NoProfile", "-Command", &script])
                    .creation_flags(CREATE_NO_WINDOW)
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
        self.handle_events(&ctx);

        // Close button hides to the tray instead of exiting; Quit lives in the tray menu.
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.hidden = true;
        }

        #[cfg(windows)]
        {
            let mut open = false;
            let mut toggle = false;
            let mut quit = false;
            while let Some(ev) = self.tray.as_ref().and_then(|t| t.poll()) {
                if let Some(t) = &self.tray {
                    if ev.id == t.open_id {
                        open = true;
                    } else if ev.id == t.toggle_id {
                        toggle = true;
                    } else if ev.id == t.quit_id {
                        quit = true;
                    }
                }
            }
            if open {
                self.hidden = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
            if toggle {
                match self.state {
                    ConnState::Disconnected | ConnState::Failed(_) => self.connect(),
                    ConnState::Connected { .. } => self.disconnect(),
                    _ => {}
                }
            }
            if let Some(t) = &self.tray {
                t.set_connected(matches!(self.state, ConnState::Connected { .. }));
            }
            if quit {
                self.quit_app();
            }
        }

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
                    self.tab = Tab::Profiles;
                }
            }
        }

        let t = self.start.elapsed().as_secs_f32();

        // ---------- top bar ----------
        egui::Panel::top("topbar")
            .frame(egui::Frame::default().fill(theme::PANEL).inner_margin(egui::Margin::symmetric(14, 10)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("waX").size(19.0).color(theme::TEXT).strong());
                    ui.label(RichText::new("VPN").size(19.0).color(theme::ACCENT).strong());
                    ui.add_space(6.0);
                    ui.label(RichText::new(format!("v{VERSION}")).size(10.5).color(theme::MUTE));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let (txt, col) = if self.is_admin {
                            ("Admin", theme::GOOD)
                        } else {
                            ("Not elevated", theme::WARN)
                        };
                        ui.label(
                            RichText::new(txt)
                                .size(10.0)
                                .color(col)
                                .strong(),
                        );
                    });
                });
            });

        // ---------- sidebar ----------
        egui::Panel::left("nav")
            .resizable(false)
            .exact_size(132.0)
            .frame(egui::Frame::default().fill(theme::PANEL).inner_margin(egui::Margin::symmetric(8, 12)))
            .show(ui, |ui| {
                let items = [
                    (Tab::Connect, "Connect"),
                    (Tab::Profiles, "Profiles"),
                    (Tab::Features, "Features"),
                    (Tab::Warp, "WARP"),
                    (Tab::About, "About"),
                ];
                for (tab, label) in items {
                    if theme::nav_button(ui, label, self.tab == tab) {
                        self.tab = tab;
                    }
                    ui.add_space(2.0);
                }
                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    ui.label(
                        RichText::new("WireGuard in Rust")
                            .size(9.5)
                            .color(theme::alpha(theme::MUTE, 140)),
                    );
                });
            });

        // ---------- central ----------
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(theme::BG).inner_margin(egui::Margin::symmetric(18, 12)))
            .show(ui, |ui| {
                let fade = theme::enter(ui, ui.scope_id().with(("tab", self.tab)), self.settings.reduced_motion);
                ui.scope(|ui| {
                    ui.set_opacity(fade);
                    match self.tab {
                        Tab::Connect => self.tab_connect(ui, t),
                        Tab::Profiles => self.tab_profiles(ui),
                        Tab::Features => self.tab_features(ui),
                        Tab::Warp => self.tab_warp(ui),
                        Tab::About => self.tab_about(ui),
                    }
                });
            });

        ctx.request_repaint_after(if self.hidden {
            Duration::from_millis(1000)
        } else {
            Duration::from_millis(33)
        });
    }
}

impl WaxApp {
    /// Explicit quit from the tray menu: disconnect cleanly, then exit.
    #[cfg(windows)]
    fn quit_app(&mut self) {
        let taken = self.tunnel.lock().unwrap().take();
        if let Some(t) = taken {
            let _ = t.shutdown();
        } else {
            let _ = Tunnel::force_disconnect();
        }
        std::process::exit(0);
    }

    fn tab_connect(&mut self, ui: &mut egui::Ui, t: f32) {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space(2.0);
            let response = orb::paint_orb(ui, &self.state, t, self.settings.reduced_motion);
            if response.clicked() {
                match self.state {
                    ConnState::Disconnected | ConnState::Failed(_) => self.connect(),
                    ConnState::Connected { .. } => self.disconnect(),
                    _ => {}
                }
            }
        });

        // selected profile summary
        let (pname, pendpoint) = self
            .selected
            .and_then(|i| self.profiles.get(i))
            .map(|p| {
                let ep = p
                    .config
                    .peers
                    .first()
                    .and_then(|peer| peer.endpoint.clone())
                    .unwrap_or_else(|| "no endpoint".to_string());
                (p.name.clone(), ep)
            })
            .unwrap_or_else(|| ("No profile selected".to_string(), "import or register WARP".to_string()));
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.label(RichText::new(&pname).size(15.0).color(theme::TEXT).strong());
            ui.label(RichText::new(&pendpoint).size(11.0).color(theme::MUTE));
        });

        ui.add_space(4.0);

        // failure card
        let failed_msg = if let ConnState::Failed(msg) = &self.state { Some(msg.clone()) } else { None };
        if let Some(msg) = failed_msg {
            theme::card_frame().show(ui, |ui| {
                ui.label(RichText::new("Connection failed").size(12.5).color(theme::BAD).strong());
                ui.label(RichText::new(&msg).size(11.5).color(theme::SUB));
                if !self.is_admin && theme::primary_button(ui, "Restart as administrator", true) {
                    self.restart_as_admin();
                }
            });
            ui.add_space(6.0);
        }

        // stats row
        if let ConnState::Connected { since } = self.state {
            let hs = self
                .stats
                .last_handshake
                .and_then(|h| SystemTime::now().duration_since(h).ok())
                .map(fmt_duration)
                .unwrap_or_else(|| "-".into());
            let session = fmt_duration(since.elapsed());
            let rx = fmt_bytes(self.stats.rx_bytes);
            let tx = fmt_bytes(self.stats.tx_bytes);
            ui.horizontal(|ui| {
                theme::stat_tile(ui, "SESSION", &session, theme::TEXT);
                theme::stat_tile(ui, "RECEIVED", &rx, theme::ACCENT);
                theme::stat_tile(ui, "SENT", &tx, theme::ACCENT);
            });
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("last handshake {hs} ago"))
                        .size(11.0)
                        .color(theme::GOOD),
                );
            });
            ui.add_space(4.0);
        }

        // explicit action button (orb is also clickable)
        let (label, enabled) = match &self.state {
            ConnState::Disconnected | ConnState::Failed(_) => ("Connect", self.selected.is_some()),
            ConnState::Connected { .. } => ("Disconnect", true),
            ConnState::Connecting => ("Connecting...", false),
            ConnState::Disconnecting => ("Disconnecting...", false),
        };
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            if theme::primary_button(ui, label, enabled) {
                match self.state {
                    ConnState::Disconnected | ConnState::Failed(_) => self.connect(),
                    ConnState::Connected { .. } => self.disconnect(),
                    _ => {}
                }
            }
            if self.selected.is_none() {
                ui.label(
                    RichText::new("Add a profile from the Profiles tab to connect")
                        .size(11.0)
                        .color(theme::MUTE),
                );
            } else if !self.is_admin {
                if ui.link(RichText::new("Restart as administrator to connect").size(11.5).color(theme::WARN)).clicked() {
                    self.restart_as_admin();
                }
            }
            ui.add_space(2.0);
            ui.label(
                RichText::new(&self.status_line).size(11.0).color(theme::SUB),
            );
        });

        ui.add_space(10.0);
        // Connection details card (fills the lower pane, honest facts only)
        let details = self
            .selected
            .and_then(|i| self.profiles.get(i))
            .map(|p| {
                let addr = p
                    .config
                    .interface
                    .addresses
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                Some((p.config.peers.first().and_then(|x| x.endpoint.clone()).unwrap_or_else(|| "-".into()), addr))
            })
            .flatten();
        theme::card_frame().show(ui, |ui| {
            ui.label(RichText::new("Connection details").size(12.0).color(theme::MUTE).strong());
            ui.add_space(2.0);
            if let Some((ep, addr)) = details {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Endpoint").size(11.5).color(theme::MUTE));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(ep).size(11.5).color(theme::SUB));
                    });
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Tunnel address").size(11.5).color(theme::MUTE));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(addr).size(11.5).color(theme::SUB));
                    });
                });
            } else {
                ui.label(RichText::new("No profile selected").size(11.5).color(theme::MUTE));
            }
            ui.horizontal(|ui| {
                ui.label(RichText::new("Kill switch").size(11.5).color(theme::MUTE));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(if self.settings.kill_switch { "On" } else { "Off" })
                            .size(11.5)
                            .color(if self.settings.kill_switch { theme::GOOD } else { theme::MUTE }),
                    );
                });
            });
        });
    }

    fn tab_profiles(&mut self, ui: &mut egui::Ui) {
        theme::section_heading(ui, "Profiles", "WireGuard configs on this device. Drop a .conf file anywhere to import.");
        let mut connect_idx: Option<usize> = None;
        let mut delete_idx: Option<usize> = None;
        let connected = matches!(self.state, ConnState::Connected { .. });
        egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
            for (i, p) in self.profiles.iter().enumerate() {
                let selected = self.selected == Some(i);
                let ep = p
                    .config
                    .peers
                    .first()
                    .and_then(|peer| peer.endpoint.clone())
                    .unwrap_or_else(|| "-".to_string());
                let addr = p
                    .config
                    .interface
                    .addresses
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                let frame = if selected {
                    theme::card_frame().stroke(egui::Stroke::new(1.0, theme::ACCENT))
                } else {
                    theme::card_frame()
                };
                frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&p.name).size(13.5).color(theme::TEXT).strong());
                            ui.label(RichText::new(format!("{ep}  -  {addr}")).size(10.5).color(theme::MUTE));
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if theme::danger_button(ui, "Delete") {
                                delete_idx = Some(i);
                            }
                            if !connected
                                && ui
                                    .add(egui::Button::new(RichText::new("Connect").size(12.0).color(theme::ACCENT))
                                        .fill(theme::alpha(theme::ACCENT, 18))
                                        .stroke(egui::Stroke::new(1.0, theme::alpha(theme::ACCENT, 90)))
                                        .corner_radius(egui::CornerRadius::same(8)))
                                    .clicked()
                            {
                                connect_idx = Some(i);
                            }
                            if ui.selectable_label(selected, RichText::new(if selected { "Selected" } else { "Select" }).size(11.5)).clicked() {
                                self.selected = Some(i);
                            }
                        });
                    });
                });
                ui.add_space(4.0);
            }
            if self.profiles.is_empty() {
                theme::card_frame().show(ui, |ui| {
                    ui.label(RichText::new("No profiles yet").size(13.0).color(theme::SUB));
                    ui.label(
                        RichText::new("Paste a config below, drop a .conf file, or register Cloudflare WARP from the WARP tab.")
                            .size(11.0)
                            .color(theme::MUTE),
                    );
                });
            }
        });
        if let Some(i) = delete_idx {
            if let Some(p) = self.profiles.get(i) {
                let _ = profile::delete_profile(&p.name);
                self.reload_profiles();
            }
        }
        if let Some(i) = connect_idx {
            self.selected = Some(i);
            self.connect();
        }

        ui.add_space(8.0);
        theme::section_heading(ui, "Import", "Paste a WireGuard config");
        theme::card_frame().show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("import-scroll")
                .max_height(110.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.import_paste)
                            .hint_text("[Interface]\nPrivateKey = ...")
                            .desired_width(f32::INFINITY),
                    );
                });
            ui.add_space(4.0);
            if theme::primary_button(ui, "Import config", !self.import_paste.trim().is_empty()) {
                let text = self.import_paste.trim().to_string();
                self.import_text(format!("Imported {}", chrono_free_stamp()), &text);
                self.import_paste.clear();
            }
            if let Some(err) = &self.import_error {
                ui.label(RichText::new(err).size(11.0).color(theme::BAD));
            }
        });
    }

    fn tab_features(&mut self, ui: &mut egui::Ui) {
        theme::section_heading(ui, "Features", "Protection and behaviour");
        if theme::toggle_row(
            ui,
            &mut self.settings.kill_switch,
            "Kill switch",
            "Block all traffic outside the tunnel while connected",
        ) {
            settings::save(&self.settings);
        }
        ui.add_space(4.0);
        if theme::toggle_row(
            ui,
            &mut self.settings.allow_lan,
            "Allow local network",
            "Keep printers and local devices reachable when the kill switch is on",
        ) {
            settings::save(&self.settings);
        }
        ui.add_space(4.0);
        if theme::toggle_row(
            ui,
            &mut self.settings.autostart,
            "Start with Windows",
            "Launch waX VPN when you sign in",
        ) {
            self.toggle_autostart();
            settings::save(&self.settings);
        }
        ui.add_space(4.0);
        if theme::toggle_row(
            ui,
            &mut self.settings.reduced_motion,
            "Reduced motion",
            "Calm the interface animation",
        ) {
            settings::save(&self.settings);
        }
        ui.add_space(8.0);
        theme::card_frame().show(ui, |ui| {
            ui.label(
                RichText::new(if self.is_admin {
                    "Running with administrator rights"
                } else {
                    "Not elevated - connecting requires a restart as administrator"
                })
                .size(11.5)
                .color(theme::MUTE),
            );
        });
    }

    fn tab_warp(&mut self, ui: &mut egui::Ui) {
        theme::section_heading(ui, "Cloudflare WARP", "Optional. waX VPN is not affiliated with Cloudflare.");
        theme::card_frame().show(ui, |ui| {
            ui.label(
                RichText::new("Registering creates a Cloudflare WARP account from this device and adds a 'Cloudflare WARP' profile.")
                    .size(11.5)
                    .color(theme::SUB),
            );
            ui.add_space(4.0);
            ui.label(RichText::new(&self.warp_summary).size(12.5).color(theme::TEXT));
            ui.add_space(6.0);
            if self.warp_registered {
                if theme::danger_button(ui, "Unregister WARP") && !self.warp_busy {
                    self.warp_unregister();
                }
            } else {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.warp_tos, RichText::new("I accept").size(12.0).color(theme::SUB));
                    ui.hyperlink_to(
                        RichText::new("Cloudflare's Terms of Service").size(12.0).color(theme::ACCENT),
                        "https://www.cloudflare.com/application/terms/",
                    );
                });
                ui.add_space(2.0);
                if theme::primary_button(ui, "Register WARP and create profile", self.warp_tos && !self.warp_busy) {
                    self.warp_register();
                }
            }
        });
    }

    fn tab_about(&mut self, ui: &mut egui::Ui) {
        theme::section_heading(ui, "About", "");
        theme::card_frame().show(ui, |ui| {
            ui.label(RichText::new(format!("waX VPN v{VERSION}")).size(14.0).color(theme::TEXT).strong());
            ui.label(
                RichText::new("A WireGuard client for Windows written in Rust, using the official wireguard-nt driver.")
                    .size(11.5)
                    .color(theme::SUB),
            );
            ui.add_space(4.0);
            ui.hyperlink_to(
                RichText::new("github.com/vio137/wax-vpn").size(12.0).color(theme::ACCENT),
                "https://github.com/vio137/wax-vpn",
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new("Not affiliated with Cloudflare or the WireGuard project.")
                    .size(10.5)
                    .color(theme::MUTE),
            );
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
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon-256.png"))
        .expect("embedded icon must decode");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("waX VPN")
            .with_app_id("wax-vpn")
            .with_icon(icon)
            .with_inner_size([620.0, 600.0])
            .with_min_inner_size([560.0, 520.0])
            .with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "waX VPN",
        options,
        Box::new(|cc| Ok(Box::new(WaxApp::new(cc)))),
    )
}
