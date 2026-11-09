//! waX VPN - WireGuard client for Windows. egui front end over wax-tunnel.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gargantua;
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
    Stage(&'static str),
    TunnelLost,
    CleanupResult(Result<(), String>),
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
    connect_stages: Vec<&'static str>,
    reconnect_attempts: u8,
    reconnecting: bool,
    picker_open: bool,
    confirm_delete: Option<(usize, f64)>,
    cleanup_issue: Option<String>,
    scene: gargantua::Gargantua,
    energy: f32,
    speed: f32,
    phase: f32,
    last_tick: Instant,
    #[cfg(windows)]
    tray: Option<tray::Tray>,
}

impl WaxApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_theme(egui::Theme::Dark);
        cc.egui_ctx.style_mut_of(egui::Theme::Dark, |style| {
            style.visuals.panel_fill = theme::BG;
            style.visuals.window_fill = theme::BG;
        });

        // Inter Display (OFL 1.1) - the product typeface. SemiBold rides in a
        // named family; egui's .strong() does not switch fonts on its own.
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "InterDisplay".into(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/InterDisplay-Regular.ttf"))),
        );
        fonts.font_data.insert(
            "InterDisplay-SemiBold".into(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/InterDisplay-SemiBold.ttf"))),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "InterDisplay".into());
        fonts.families.insert(
            egui::FontFamily::Name("semibold".into()),
            vec!["InterDisplay-SemiBold".into()],
        );
        cc.egui_ctx.set_fonts(fonts);

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
            connect_stages: Vec::new(),
            reconnect_attempts: 0,
            reconnecting: false,
            picker_open: false,
            confirm_delete: None,
            cleanup_issue: None,
            scene: gargantua::Gargantua::new(),
            energy: 0.5,
            speed: 0.7,
            phase: 0.0,
            last_tick: Instant::now(),
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
                    if !t.is_up() {
                        drop(guard);
                        let _ = tx.send(AppEvent::TunnelLost);
                        break;
                    }
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

    fn connect_manual(&mut self) {
        self.reconnect_attempts = 0;
        self.reconnecting = false;
        self.connect();
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
        self.connect_stages.clear();
        self.status_line = format!("Connecting to {}...", profile.name);
        let opts = ConnectOptions {
            kill_switch: self.settings.kill_switch,
            allow_lan: self.settings.allow_lan,
        };
        let tunnel = self.tunnel.clone();
        let tx = self.events_tx.clone();
        let tx_stage = tx.clone();
        std::thread::spawn(move || match Tunnel::connect_progress(&profile, &opts, move |stage| {
            let _ = tx_stage.send(AppEvent::Stage(stage));
        }) {
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
        self.reconnect_attempts = 0;
        self.status_line = "Disconnecting...".to_string();
        let tunnel = self.tunnel.clone();
        let tx = self.events_tx.clone();
        std::thread::spawn(move || {
            let taken = tunnel.lock().unwrap().take();
            let result = if let Some(t) = taken {
                t.shutdown().map_err(|e| e.to_string())
            } else {
                Tunnel::force_disconnect().map_err(|e| e.to_string())
            };
            let _ = tx.send(AppEvent::CleanupResult(result));
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
                AppEvent::TunnelLost => {
                    if !matches!(self.state, ConnState::Connected { .. }) {
                        continue;
                    }
                    if !self.settings.kill_switch {
                        // No kill switch: clean leftovers before reconnecting.
                        let _ = Tunnel::force_disconnect();
                    }
                    // With the kill switch on, leave its rules in place so the
                    // machine stays fail-closed between attempts; connecting
                    // re-adds them idempotently.
                    if self.settings.auto_reconnect {
                        self.reconnecting = true;
                        self.reconnect_attempts = 1;
                        self.status_line = "Connection lost - reconnecting (1/3)".into();
                        self.connect();
                    } else if self.settings.kill_switch {
                        self.state = ConnState::Failed(
                            "Connection lost; kill-switch rules remain enabled".into(),
                        );
                    } else {
                        self.state = ConnState::Failed("Connection lost".into());
                    }
                }
                AppEvent::Connected => {
                    self.state = ConnState::Connected { since: Instant::now() };
                    self.status_line = "Tunnel is up".to_string();
                    self.reconnect_attempts = 0;
                    self.reconnecting = false;
                    if let Some(idx) = self.selected {
                        if let Some(p) = self.profiles.get(idx) {
                            self.settings.last_profile = Some(p.name.clone());
                            settings::save(&self.settings);
                        }
                    }
                    self.spawn_stats_loop();
                }
                AppEvent::ConnectFailed(e) if self.reconnecting && self.reconnect_attempts < 3 => {
                    self.reconnect_attempts += 1;
                    self.status_line = format!(
                        "Connection lost - reconnecting ({}/3)",
                        self.reconnect_attempts
                    );
                    self.connect();
                    let _ = e;
                }
                AppEvent::ConnectFailed(e) => {
                    self.reconnecting = false;
                    let msg = if self.reconnect_attempts > 0 && self.settings.kill_switch {
                        format!("{e} - kill-switch rules remain enabled")
                    } else {
                        e
                    };
                    self.reconnect_attempts = 0;
                    self.state = ConnState::Failed(msg.clone());
                    self.status_line = msg;
                }
                AppEvent::CleanupResult(Ok(())) => {
                    self.cleanup_issue = None;
                }
                AppEvent::CleanupResult(Err(e)) => {
                    self.cleanup_issue = Some(e);
                }
                AppEvent::Disconnected => {
                    self.state = ConnState::Disconnected;
                    self.status_line = match &self.cleanup_issue {
                        None => "Disconnected".to_string(),
                        Some(e) => format!("Disconnected, but cleanup needs attention: {e}"),
                    };
                    self.stats = TunnelStats::default();
                }
                AppEvent::Stats(s) => self.stats = s,
                AppEvent::Stage(stage) => {
                    self.connect_stages.push(stage);
                    self.status_line = format!("{stage}...");
                }
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
                    ConnState::Disconnected | ConnState::Failed(_) => self.connect_manual(),
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

        // Keyboard shortcuts, only when no text field owns the keyboard.
        if ctx.memory(|m| m.focused().is_none()) {
            let (space, esc, q, digits) = ctx.input(|i| {
                (
                    i.key_pressed(egui::Key::Space),
                    i.key_pressed(egui::Key::Escape),
                    i.modifiers.ctrl && i.key_pressed(egui::Key::Q),
                    [
                        i.modifiers.ctrl && i.key_pressed(egui::Key::Num1),
                        i.modifiers.ctrl && i.key_pressed(egui::Key::Num2),
                        i.modifiers.ctrl && i.key_pressed(egui::Key::Num3),
                        i.modifiers.ctrl && i.key_pressed(egui::Key::Num4),
                        i.modifiers.ctrl && i.key_pressed(egui::Key::Num5),
                    ],
                )
            });
            for (idx, hit) in digits.iter().enumerate() {
                if *hit {
                    self.tab = [Tab::Connect, Tab::Profiles, Tab::Features, Tab::Warp, Tab::About][idx];
                }
            }
            if esc && self.picker_open {
                self.picker_open = false;
            }
            if space {
                match self.state {
                    ConnState::Disconnected | ConnState::Failed(_) => self.connect_manual(),
                    ConnState::Connected { .. } => self.disconnect(),
                    _ => {}
                }
            }
            #[cfg(windows)]
            if q {
                self.quit_app();
            }
            #[cfg(not(windows))]
            let _ = q;
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

        // ---------- custom title bar (frameless window) ----------
        egui::Panel::top("titlebar")
            .frame(egui::Frame::default().fill(theme::PANEL).inner_margin(egui::Margin::symmetric(10, 5)))
            .show(ui, |ui| {
                let bar = ui.max_rect();
                let drag = ui.interact(bar, ui.scope_id().with("titlebar_drag"), egui::Sense::click_and_drag());
                if drag.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                ui.horizontal(|ui| {
                    ui.label(RichText::new("waX VPN").size(12.5).color(theme::SUB).font(theme::semi(12.5)));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // close hides to the tray; Quit lives in the tray menu
                        if chrome_button(ui, "\u{00d7}", "Hide to tray - quit from the tray icon menu") {
                            self.hidden = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                        }
                        if chrome_button(ui, "\u{2013}", "Minimize") {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                        let (txt, col) = if self.is_admin {
                            ("Admin", theme::MUTE)
                        } else {
                            ("Not elevated", theme::WARN)
                        };
                        ui.label(RichText::new(txt).size(10.0).color(col));
                    });
                });
            });

        // ---------- icon rail ----------
        egui::Panel::left("rail")
            .resizable(false)
            .exact_size(76.0)
            .frame(egui::Frame::default().fill(theme::PANEL).inner_margin(egui::Margin::symmetric(6, 10)))
            .show(ui, |ui| {
                ui.add_space(6.0);
                let items = [
                    (Tab::Connect, "Connect"),
                    (Tab::Profiles, "Profiles"),
                    (Tab::Features, "Features"),
                    (Tab::Warp, "WARP"),
                    (Tab::About, "About"),
                ];
                for (tab, label) in items {
                    if rail_button(ui, tab, label, self.tab == tab) {
                        self.tab = tab;
                    }
                    ui.add_space(4.0);
                }
            });

        // ---------- central: deep-space scene with content on top ----------
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(theme::BG).inner_margin(egui::Margin::symmetric(18, 12)))
            .show(ui, |ui| {
                // Scene timing. Brightness and rotation speed are fixed moods
                // per connection state - never tied to measured throughput.
                let now = Instant::now();
                let dt = (now - self.last_tick).as_secs_f32().min(0.1);
                self.last_tick = now;
                let (energy_t, speed_t) = match &self.state {
                    ConnState::Disconnected => (0.5_f32, 0.7_f32),
                    ConnState::Connecting => (0.8, 1.35),
                    ConnState::Connected { .. } => (1.0, 1.0),
                    ConnState::Disconnecting => (0.55, 0.8),
                    ConnState::Failed(_) => (0.3, 0.45),
                };
                let k = (dt * 4.0).min(1.0);
                self.energy += (energy_t - self.energy) * k;
                self.speed += (speed_t - self.speed) * k;
                if !self.settings.reduced_motion {
                    self.phase += dt * self.speed;
                }
                let (phase, twinkle_t) = if self.settings.reduced_motion {
                    (1.5, 4.2)
                } else {
                    (self.phase, t)
                };

                // A whisper of starfield, brand-coherent with the site.
                {
                    let rect = ui.max_rect();
                    let painter = ui.painter().with_clip_rect(rect);
                    self.scene
                        .paint_stars(&painter, rect, rect.center(), 30.0, twinkle_t, 0.20);
                }

                let fade = theme::enter(ui, ui.scope_id().with(("tab", self.tab)), self.settings.reduced_motion);
                ui.scope(|ui| {
                    ui.set_opacity(fade);
                    match self.tab {
                        Tab::Connect => self.tab_connect(ui, t, phase, twinkle_t),
                        Tab::Profiles => self.tab_profiles(ui),
                        Tab::Features => self.tab_features(ui),
                        Tab::Warp => self.tab_warp(ui),
                        Tab::About => self.tab_about(ui),
                    }
                });
            });

        // Frameless windows have no native resize border: thin edge strips
        // and a corner grip restore it.
        let screen = ctx.content_rect();
        let grips = [
            (egui::Rect::from_min_max(egui::pos2(screen.right() - 4.0, screen.top()), egui::pos2(screen.right(), screen.bottom())), egui::viewport::ResizeDirection::East),
            (egui::Rect::from_min_max(egui::pos2(screen.left(), screen.bottom() - 4.0), egui::pos2(screen.right(), screen.bottom())), egui::viewport::ResizeDirection::South),
            (egui::Rect::from_min_max(egui::pos2(screen.right() - 14.0, screen.bottom() - 14.0), egui::pos2(screen.right(), screen.bottom())), egui::viewport::ResizeDirection::SouthEast),
        ];
        for (i, (gr, dir)) in grips.into_iter().enumerate() {
            let gresp = ui.interact(gr, ui.scope_id().with(("grip", i)), egui::Sense::click_and_drag());
            if gresp.hovered() || gresp.dragged() {
                ctx.set_cursor_icon(match dir {
                    egui::viewport::ResizeDirection::East => egui::CursorIcon::ResizeHorizontal,
                    _ => egui::CursorIcon::ResizeNwSe,
                });
            }
            if gresp.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(dir));
            }
        }
        let idle_still = matches!(self.state, ConnState::Disconnected) && self.settings.reduced_motion;
        ctx.request_repaint_after(if self.hidden {
            Duration::from_millis(1000)
        } else if idle_still {
            Duration::from_millis(250)
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

    fn tab_connect(&mut self, ui: &mut egui::Ui, t: f32, phase: f32, twinkle_t: f32) {
        let rect = ui.max_rect().shrink2(egui::vec2(24.0, 18.0));
        let col_w = 400.0_f32.min(rect.width());
        // Estimate the content block height so the column sits optically
        // centered instead of hugging the top with dead space below.
        let mut content_h = 66.0 + 16.0 + 42.0 + 8.0 + 22.0 + 18.0 + 26.0 + 52.0 + 8.0;
        if self.profiles.len() > 1
            && matches!(self.state, ConnState::Disconnected | ConnState::Failed(_))
        {
            content_h += 14.0; // switch hint
            if self.picker_open {
                content_h += 6.0 + self.profiles.len() as f32 * 30.0 + 24.0;
            }
        }
        if matches!(self.state, ConnState::Connected { .. }) {
            content_h += 22.0 + 1.0 + 14.0 + 40.0 + 12.0 + 1.0 + 10.0 + 18.0;
        }
        if matches!(self.state, ConnState::Connecting) {
            content_h += 28.0;
        }
        if matches!(self.state, ConnState::Failed(_)) {
            content_h += 24.0;
        }
        let col_h = content_h.min(rect.height());
        let col = egui::Rect::from_center_size(
            egui::pos2(rect.center().x, rect.center().y - 10.0),
            egui::vec2(col_w, col_h),
        );
        let mut cui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(col)
                .layout(Layout::top_down(Align::Center)),
        );
        let ui = &mut cui;

        // Brand mark: the black hole, small and quiet. Brightness and spin
        // follow the connection state as mood, never as measured speed.
        let er = 34.0;
        let (erect, _) = ui.allocate_exact_size(egui::vec2(er * 5.0, er * 2.2), egui::Sense::hover());
        self.scene
            .paint_hole(ui.painter(), erect.center(), er, phase, twinkle_t, self.energy);
        ui.add_space(16.0);

        let (word, wcol) = match &self.state {
            ConnState::Disconnected => ("Disconnected", theme::TEXT),
            ConnState::Connecting => ("Connecting", theme::ACCENT),
            ConnState::Connected { .. } => ("Connected", theme::GOOD),
            ConnState::Disconnecting => ("Disconnecting", theme::SUB),
            ConnState::Failed(_) => ("Connection failed", theme::BAD),
        };
        ui.label(RichText::new(word).font(theme::semi(32.0)).color(wcol));
        ui.add_space(8.0);

        // Profile: the name opens the picker while disconnected.
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
        let can_switch = matches!(self.state, ConnState::Disconnected | ConnState::Failed(_))
            && self.profiles.len() > 1;
        let name_resp = ui.add(
            egui::Label::new(RichText::new(&pname).font(theme::semi(15.0)).color(theme::SUB))
                .sense(if can_switch { egui::Sense::click() } else { egui::Sense::hover() }),
        );
        if name_resp.hovered() && can_switch {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if name_resp.clicked() && can_switch {
            self.picker_open = !self.picker_open;
        }
        ui.label(RichText::new(&pendpoint).size(11.5).color(theme::MUTE).monospace());
        if can_switch {
            ui.label(
                RichText::new(if self.picker_open { "choose a profile" } else { "click to switch" })
                    .size(10.0)
                    .color(theme::MUTE),
            );
        }
        if self.picker_open && can_switch {
            ui.add_space(6.0);
            theme::card_frame().show(ui, |ui| {
                ui.set_max_width(300.0);
                ui.set_min_width(280.0);
                for i in 0..self.profiles.len() {
                    let (name, ep) = {
                        let p = &self.profiles[i];
                        (
                            p.name.clone(),
                            p.config
                                .peers
                                .first()
                                .and_then(|x| x.endpoint.clone())
                                .unwrap_or_else(|| "-".into()),
                        )
                    };
                    let sel = self.selected == Some(i);
                    let (r, resp) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 30.0),
                        egui::Sense::click(),
                    );
                    let h = ui.ctx().animate_bool_responsive(resp.id.with("hov"), resp.hovered());
                    if h > 0.02 || sel {
                        ui.painter().rect(
                            r,
                            egui::CornerRadius::same(8),
                            theme::alpha(egui::Color32::WHITE, (8.0 * h) as u8 + if sel { 10 } else { 0 }),
                            egui::Stroke::NONE,
                            egui::StrokeKind::Inside,
                        );
                    }
                    ui.painter().text(
                        r.left_center() + egui::vec2(10.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        &name,
                        egui::FontId::proportional(13.0),
                        if sel { theme::ACCENT } else { theme::SUB },
                    );
                    ui.painter().text(
                        r.right_center() + egui::vec2(-10.0, 0.0),
                        egui::Align2::RIGHT_CENTER,
                        &ep,
                        egui::FontId::monospace(10.5),
                        theme::MUTE,
                    );
                    if resp.clicked() {
                        self.selected = Some(i);
                        self.confirm_delete = None;
                        self.settings.last_profile = Some(name);
                        settings::save(&self.settings);
                        self.picker_open = false;
                        self.status_line = "Ready".into();
                    }
                }
            });
        }

        ui.add_space(26.0);

        // The one confident action.
        let (action_label, can_act) = match &self.state {
            ConnState::Disconnected | ConnState::Failed(_) => ("Connect", self.selected.is_some()),
            ConnState::Connected { .. } => ("Disconnect", true),
            ConnState::Connecting => ("Connecting", false),
            ConnState::Disconnecting => ("Disconnecting", false),
        };
        if theme::primary_button(ui, action_label, can_act) {
            match self.state {
                ConnState::Disconnected | ConnState::Failed(_) => self.connect_manual(),
                ConnState::Connected { .. } => self.disconnect(),
                _ => {}
            }
        }
        ui.add_space(8.0);
        if self.selected.is_none() {
            ui.label(
                RichText::new("Add a profile from the Profiles tab to connect")
                    .size(11.0)
                    .color(theme::MUTE),
            );
        } else if !self.is_admin {
            if ui
                .link(RichText::new("Restart as administrator to connect").size(11.5).color(theme::WARN))
                .clicked()
            {
                self.restart_as_admin();
            }
        }

        if let ConnState::Failed(msg) = &self.state {
            let msg = msg.clone();
            ui.add_space(6.0);
            ui.label(RichText::new(msg).size(12.0).color(theme::BAD));
        }
        if matches!(self.state, ConnState::Connecting) {
            let stage = self.connect_stages.last().copied().unwrap_or("Starting");
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                let pulse = 0.5 + 0.5 * (t * 3.0).sin();
                ui.painter()
                    .circle_filled(r.center(), 3.0 + pulse, theme::alpha(theme::ACCENT, 150));
                ui.label(RichText::new(stage).size(12.0).color(theme::SUB));
            });
        }

        if let ConnState::Connected { since } = self.state {
            ui.add_space(22.0);
            theme::hairline(ui);
            ui.add_space(14.0);
            let session = fmt_duration(since.elapsed());
            let rx = fmt_bytes(self.stats.rx_bytes);
            let txb = fmt_bytes(self.stats.tx_bytes);
            ui.horizontal(|ui| {
                let w = ui.available_width() / 3.0;
                ui.allocate_ui_with_layout(
                    egui::vec2(w, 40.0),
                    Layout::top_down(Align::Center),
                    |ui| mini_stat(ui, "Session", &session),
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(w, 40.0),
                    Layout::top_down(Align::Center),
                    |ui| mini_stat(ui, "Received", &rx),
                );
                ui.allocate_ui_with_layout(
                    egui::vec2(w, 40.0),
                    Layout::top_down(Align::Center),
                    |ui| mini_stat(ui, "Sent", &txb),
                );
            });
            ui.add_space(12.0);
            theme::hairline(ui);
            ui.add_space(10.0);
            let hs = self
                .stats
                .last_handshake
                .and_then(|h| SystemTime::now().duration_since(h).ok())
                .map(fmt_duration)
                .unwrap_or_else(|| "-".into());
            ui.label(
                RichText::new(format!("last handshake {hs} ago"))
                    .size(11.0)
                    .color(theme::GOOD),
            );
        }

        // Quiet facts pinned to the bottom.
        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            ui.add_space(2.0);
            if let Some(issue) = self.cleanup_issue.clone() {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(issue).size(10.5).color(theme::WARN));
                    if ui.link(RichText::new("Retry").size(10.5).color(theme::WARN)).clicked() {
                        let tx = self.events_tx.clone();
                        std::thread::spawn(move || {
                            let _ = tx.send(AppEvent::CleanupResult(
                                Tunnel::force_disconnect().map_err(|e| e.to_string()),
                            ));
                            let _ = tx.send(AppEvent::Disconnected);
                        });
                    }
                });
            }
            ui.label(
                RichText::new(format!(
                    "Kill switch {}  ·  v{VERSION}",
                    if self.settings.kill_switch { "on" } else { "off" }
                ))
                .size(10.5)
                .color(theme::MUTE),
            );
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
                let (r, resp) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 48.0),
                    egui::Sense::hover(),
                );
                let hov = ui
                    .ctx()
                    .animate_bool_responsive(resp.id.with("hov"), resp.hovered());
                if hov > 0.02 || selected {
                    ui.painter().rect_filled(
                        r,
                        egui::CornerRadius::same(8),
                        theme::alpha(
                            egui::Color32::WHITE,
                            (5.0 * hov) as u8 + if selected { 6 } else { 0 },
                        ),
                    );
                }
                if selected {
                    ui.painter().rect_filled(
                        egui::Rect::from_center_size(
                            egui::pos2(r.left() + 2.0, r.center().y),
                            egui::vec2(2.5, 24.0),
                        ),
                        egui::CornerRadius::same(1),
                        theme::ACCENT,
                    );
                }
                let mut row_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(r.shrink2(egui::vec2(14.0, 0.0)))
                        .layout(Layout::left_to_right(Align::Center)),
                );
                row_ui.vertical(|ui| {
                    ui.label(
                        RichText::new(&p.name)
                            .size(13.5)
                            .color(if selected { theme::TEXT } else { theme::SUB })
                            .strong(),
                    );
                    ui.label(
                        RichText::new(format!("{ep}  ·  {addr}"))
                            .size(10.5)
                            .color(theme::MUTE)
                            .monospace(),
                    );
                });
                row_ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let now = ui.input(|inp| inp.time);
                    let armed = matches!(self.confirm_delete, Some((idx, t0)) if idx == i && now - t0 < 3.0);
                    if theme::danger_button(ui, if armed { "Confirm delete" } else { "Delete" }) {
                        if armed {
                            delete_idx = Some(i);
                            self.confirm_delete = None;
                        } else {
                            self.confirm_delete = Some((i, now));
                        }
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
                    let sel_resp = ui.add(
                        egui::Button::new(
                            RichText::new(if selected { "Selected" } else { "Select" })
                                .size(11.5)
                                .color(if selected { theme::ACCENT } else { theme::SUB }),
                        )
                        .fill(if selected { theme::alpha(theme::ACCENT, 18) } else { theme::alpha(egui::Color32::WHITE, 6) })
                        .stroke(egui::Stroke::new(1.0, theme::alpha(if selected { theme::ACCENT } else { theme::SUB }, 60)))
                        .corner_radius(egui::CornerRadius::same(8)),
                    );
                    if sel_resp.clicked() {
                        self.selected = Some(i);
                    }
                });
                theme::hairline(ui);
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
            self.confirm_delete = None;
            self.connect_manual();
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
            &mut self.settings.auto_reconnect,
            "Auto-reconnect",
            "Reconnect automatically if the tunnel drops (up to 3 attempts)",
        ) {
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

fn mini_stat(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.vertical(|ui| {
        ui.label(RichText::new(label).size(11.0).color(theme::MUTE));
        ui.label(RichText::new(value).size(14.0).color(theme::TEXT).monospace());
    });
}


fn chrome_button(ui: &mut egui::Ui, glyph: &str, tooltip: &str) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(30.0, 22.0), egui::Sense::click());
    let hov = ui.ctx().animate_bool_responsive(resp.id.with("h"), resp.hovered());
    if hov > 0.02 {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(6),
            theme::alpha(egui::Color32::WHITE, (10.0 * hov) as u8),
        );
    }
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        egui::FontId::proportional(13.0),
        theme::alpha(theme::SUB, (160.0 + 95.0 * hov) as u8),
    );
    resp.clone().on_hover_text(tooltip);
    resp.clicked()
}

fn rail_button(ui: &mut egui::Ui, tab: Tab, label: &str, active: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(64.0, 54.0), egui::Sense::click());
    let hov = ui.ctx().animate_bool_responsive(resp.id.with("h"), resp.hovered());
    let on = ui.ctx().animate_bool(resp.id.with("on"), active);
    let bg = 6.0 * hov + 12.0 * on;
    if bg > 0.5 {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(10),
            theme::alpha(egui::Color32::WHITE, bg as u8),
        );
    }
    let col = if active {
        theme::ACCENT
    } else if resp.hovered() {
        theme::TEXT
    } else {
        theme::SUB
    };
    if on > 0.03 {
        let bar = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 2.0, rect.center().y),
            egui::vec2(2.5, 22.0),
        );
        ui.painter().rect_filled(bar, egui::CornerRadius::same(1), theme::alpha(theme::ACCENT, (255.0 * on) as u8));
    }
    rail_icon(ui.painter(), tab, rect.center_top() + egui::vec2(0.0, 18.0), 9.0, col);
    ui.painter().text(
        rect.center_bottom() + egui::vec2(0.0, -9.0),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(10.0),
        col,
    );
    resp.clone().widget_info(move || {
        egui::WidgetInfo::labeled(egui::accesskit::Role::Button, true, format!("{label} tab"))
    });
    resp.clone().on_hover_text(label);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

fn rail_icon(p: &egui::Painter, tab: Tab, c: egui::Pos2, s: f32, col: egui::Color32) {
    let st = egui::Stroke::new(1.4, col);
    match tab {
        Tab::Connect => {
            // mini black hole: dark core, ring, tilted disk
            let pts: Vec<egui::Pos2> = (0..=40)
                .map(|i| {
                    let a = i as f32 / 40.0 * std::f32::consts::TAU;
                    c + egui::vec2(a.cos() * s, a.sin() * s * 0.32)
                })
                .collect();
            p.add(egui::Shape::line(pts, st));
            p.circle_filled(c, s * 0.5, egui::Color32::from_rgb(2, 3, 6));
            p.circle_stroke(c, s * 0.5, st);
        }
        Tab::Profiles => {
            for i in -1..=1 {
                let r = egui::Rect::from_center_size(
                    c + egui::vec2(i as f32 * 2.4, i as f32 * -3.4),
                    egui::vec2(s * 1.5, s * 0.85),
                );
                p.rect_stroke(r, egui::CornerRadius::same(2), st, egui::StrokeKind::Inside);
            }
        }
        Tab::Features => {
            for i in -1..=1 {
                let y = c.y + i as f32 * s * 0.55;
                p.line_segment(
                    [egui::pos2(c.x - s * 0.85, y), egui::pos2(c.x + s * 0.85, y)],
                    st,
                );
                let kx = c.x + (i as f32) * s * 0.38;
                p.circle_filled(egui::pos2(kx, y), s * 0.2, col);
            }
        }
        Tab::Warp => {
            let pts: Vec<egui::Pos2> = (0..40)
                .map(|i| {
                    let a = i as f32 / 40.0 * std::f32::consts::TAU * 1.5;
                    let r = s * i as f32 / 40.0;
                    c + egui::vec2(a.cos() * r, a.sin() * r)
                })
                .collect();
            p.add(egui::Shape::line(pts, st));
        }
        Tab::About => {
            p.circle_stroke(c, s * 0.85, st);
            p.text(
                c,
                egui::Align2::CENTER_CENTER,
                "i",
                egui::FontId::proportional(s * 1.1),
                col,
            );
        }
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
            .with_inner_size([880.0, 600.0])
            .with_min_inner_size([760.0, 540.0])
            .with_resizable(true)
            .with_decorations(false),
        ..Default::default()
    };
    eframe::run_native(
        "waX VPN",
        options,
        Box::new(|cc| Ok(Box::new(WaxApp::new(cc)))),
    )
}
