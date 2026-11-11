//! waX VPN - WireGuard client for Windows. egui front end over wax-tunnel.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use wax_diagnostics::health as connection_health;
mod recovery;
mod autostart;
mod elevation;
// Preparatory policy only. Production notification ownership is not wired yet.
#[cfg(test)]
mod network_change;
#[cfg(all(windows, test))]
mod network_notify;
#[cfg(all(windows, test))]
mod underlay;
#[cfg(windows)]
mod single;
#[cfg(not(windows))]
mod single {
    pub struct Instance;
}
mod theme;
mod tray;
mod settings;

use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

use eframe::egui::{self, Align, Layout, RichText};

use settings::Settings;
use wax_core::{profile, Profile};
use wax_tunnel::{ConnectOptions, Tunnel, TunnelStats};

const VERSION: &str = env!("WAX_BUILD_VERSION");

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
    DiagnosticsReady(String),
    Stage(u64, &'static str),
    TunnelLost(u64),
    LostCleanup(u64, Result<(), String>),
    CleanupResult(Result<(), String>),
    Connected(u64, Tunnel),
    ConnectFailed(u64, String),
    Disconnected,
    Stats(u64, TunnelStats),
    ExternalStatus(Result<Option<TunnelStats>, ()>),
    WarpDone(Result<String, String>),
}

pub struct WaxApp {
    diagnostics_report: Option<String>,
    diagnostics_generated_at: Option<Instant>,
    diagnostics_busy: bool,
    autostart_notice: Option<String>,
    settings_notice: Option<&'static str>,
    settings: Settings,
    profiles: Vec<Profile>,
    profiles_notice: Option<String>,
    profiles_readable: bool,
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
    warp_notice: Option<(String, bool)>,
    import_paste: String,
    import_name: String,
    import_error: Option<String>,
    tab: Tab,
    hidden: bool,
    connect_stages: Vec<&'static str>,
    recovery: recovery::Recovery,
    connection_epoch: Arc<AtomicU64>,
    active_profile: Option<Profile>,
    picker_open: bool,
    split_edit: String,
    split_edit_for: Option<usize>,
    #[allow(dead_code)]
    instance: Option<single::Instance>,
    confirm_delete: Option<(usize, f64)>,
    cleanup_issue: Option<String>,
    #[cfg(windows)]
    tray: Option<tray::Tray>,
}

impl WaxApp {
    fn new(
        cc: &eframe::CreationContext<'_>,
        #[cfg_attr(not(windows), allow(unused_variables))] instance: Option<single::Instance>,
    ) -> Self {
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
        let (settings, settings_notice) = match settings::load_checked() {
            Ok(settings) => (settings, None),
            Err(()) => {
                let mut settings = Settings::default();
                // Never revive automatic recovery from an unreadable choice.
                settings.auto_reconnect = false;
                (settings, Some("Saved settings could not be read and were kept unchanged. Auto-reconnect is off for this session. Check your settings file before relying on saved preferences."))
            }
        };
        let (profiles, profiles_notice, profiles_readable) = match profile::load_profiles_report() {
            Ok(report) => (report.profiles, profile_notice(report.skipped), true),
            Err(_) => (Vec::new(), Some("Profiles could not be read. Files were not changed. Reload in Profiles before connecting.".into()), false),
        };
        let selected = settings
            .last_profile
            .as_ref()
            .and_then(|last| profiles.iter().position(|p| &p.name == last))
            .or(if profiles.is_empty() { None } else { Some(0) });

        let warp_registered = matches!(wax_warp::load_account(), Ok(Some(_)));
        let mut app = Self {
            diagnostics_report: None,
            diagnostics_generated_at: None,
            diagnostics_busy: false,
            autostart_notice: None,
            settings_notice,
            settings,
            profiles,
            profiles_notice,
            profiles_readable,
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
            warp_notice: None,
            import_paste: String::new(),
            import_name: String::new(),
            import_error: None,
            tab: Tab::Connect,
            hidden: false,
            connect_stages: Vec::new(),
            recovery: recovery::Recovery::default(),
            connection_epoch: Arc::new(AtomicU64::new(0)),
            active_profile: None,
            picker_open: false,
            split_edit: String::new(),
            split_edit_for: None,
            instance,
            confirm_delete: None,
            cleanup_issue: None,
            #[cfg(windows)]
            tray: tray::Tray::new(&cc.egui_ctx),
        };
        app.refresh_warp_summary();
        // Pick up a tunnel left running by an earlier instance.
        if let Ok(Some(stats)) = Tunnel::existing_status() {
            app.state = ConnState::Connected { since: Instant::now() };
            app.stats = stats;
            app.spawn_external_stats_loop();
        }
        app
    }

    fn refresh_warp_summary(&mut self) {
        match wax_warp::load_account() {
            Ok(Some(_account)) => {
                self.warp_registered = true;
                self.warp_summary = "Registered on this device".into();
            }
            _ => {
                self.warp_registered = false;
                self.warp_summary = "Not registered".to_string();
            }
        }
    }

    fn reload_profiles(&mut self) {
        let keep = self.selected.and_then(|i| self.profiles.get(i)).map(|p| p.name.clone());
        match profile::load_profiles_report() {
            Ok(report) => {
                self.profiles = report.profiles;
                self.profiles_notice = profile_notice(report.skipped);
                self.profiles_readable = true;
            }
            Err(_) => {
                // Keep the visible snapshot, but do not connect stale data.
                self.profiles_notice = Some("Profiles could not be reloaded. The previous list is shown and files were not changed. Reload in Profiles before connecting.".into());
                self.profiles_readable = false;
                return;
            }
        }
        self.selected = keep
            .and_then(|name| self.profiles.iter().position(|p| p.name == name))
            .or(if self.profiles.is_empty() { None } else { Some(0) });
        // These caches use list indices. A sorted reload can assign an old
        // index to a different profile, so always repopulate from its name.
        self.split_edit_for = None;
        self.confirm_delete = None;
        self.picker_open = false;
    }

    // Read status of a tunnel owned by a CLI/earlier process. Never pretend
    // we own its handle and never reconnect a guessed profile when it ends.
    fn spawn_external_stats_loop(&self) {
        let tunnel = self.tunnel.clone();
        let tx = self.events_tx.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(1));
            if tunnel.lock().unwrap().is_some() { break; }
            let state = Tunnel::existing_status().map_err(|_| ());
            let finished = matches!(state, Ok(None));
            if tx.send(AppEvent::ExternalStatus(state)).is_err() || finished { break; }
        });
    }

    fn spawn_stats_loop(&self) {
        let generation = self.recovery.generation();
        let epoch = self.connection_epoch.clone();
        let tunnel = self.tunnel.clone();
        let tx = self.events_tx.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(1));
            if epoch.load(Ordering::Acquire) != generation { break; }
            let guard = tunnel.lock().unwrap();
            match guard.as_ref() {
                Some(t) => {
                    if !t.is_up() {
                        drop(guard);
                        let _ = tx.send(AppEvent::TunnelLost(generation));
                        break;
                    }
                    let stats = t.stats();
                    drop(guard);
                    if tx.send(AppEvent::Stats(generation, stats)).is_err() {
                        break;
                    }
                }
                None => break,
            }
        });
    }

    fn connect_manual(&mut self) {
        let generation = self.recovery.new_intent();
        self.connection_epoch.store(generation, Ordering::Release);
        self.active_profile = None;
        self.connect();
    }

    fn connect(&mut self) {
        if !self.profiles_readable {
            self.state = ConnState::Failed("Reload profiles before connecting".into());
            return;
        }
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
        let Some(mut profile) = self.profiles.get(idx).cloned() else { return };
        if let Some(raw) = self.settings.split_exclusions.get(&profile.name) {
            if !raw.trim().is_empty() {
                match wax_core::split::parse_exclusions(raw) {
                    Ok(excls) => {
                        for peer in &mut profile.config.peers {
                            peer.allowed_ips =
                                wax_core::split::apply_exclusions(&peer.allowed_ips, &excls);
                        }
                        if profile.config.peers.iter().all(|pr| pr.allowed_ips.is_empty()) {
                            self.state = ConnState::Failed(
                                "Split tunneling exclusions cover all traffic - nothing left to tunnel".into(),
                            );
                            return;
                        }
                    }
                    Err(e) => {
                        self.state = ConnState::Failed(format!("Split tunneling: {e}"));
                        return;
                    }
                }
            }
        }
        self.active_profile = Some(profile.clone());
        self.connect_prepared(profile);
    }

    fn connect_prepared(&mut self, profile: Profile) {
        let generation = self.recovery.generation();
        self.state = ConnState::Connecting;
        self.connect_stages.clear();
        self.status_line = format!("Connecting to {}...", profile.name);
        let opts = ConnectOptions::default();
        let tunnel = self.tunnel.clone();
        let tx = self.events_tx.clone();
        let tx_stage = tx.clone();
        std::thread::spawn(move || {
            // Dispose of a lost owned adapter before creating its replacement.
            if let Some(old) = tunnel.lock().unwrap().take() {
                if old.shutdown().is_err() {
                    let _ = tx.send(AppEvent::ConnectFailed(generation, "Previous tunnel cleanup failed; disconnect before retrying".into()));
                    return;
                }
            }
            match Tunnel::connect_progress(&profile, &opts, move |stage| {
                let _ = tx_stage.send(AppEvent::Stage(generation, stage));
            }) {
                Ok(t) => { let _ = tx.send(AppEvent::Connected(generation, t)); }
                Err(e) => { let _ = tx.send(AppEvent::ConnectFailed(generation, e.to_string())); }
            }
        });
    }

    fn schedule_recovery(&mut self) -> bool {
        if self.recovery.schedule(self.recovery.generation(), self.start.elapsed(), self.active_profile.is_some(), self.settings.auto_reconnect) {
            self.state = ConnState::Connecting;
            self.status_line = format!("Connection lost - waiting to retry ({}/3)", self.recovery.attempts());
            true
        } else { false }
    }

    fn disconnect(&mut self) {
        self.state = ConnState::Disconnecting;
        let generation = self.recovery.new_intent();
        self.connection_epoch.store(generation, Ordering::Release);
        self.active_profile = None;
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

    fn tray_available(&self) -> bool {
        #[cfg(windows)]
        { self.tray.is_some() }
        #[cfg(not(windows))]
        { false }
    }

    fn close_to_background(&mut self, ctx: &egui::Context) {
        let available = self.tray_available();
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        ctx.send_viewport_cmd(background_command(available));
        self.hidden = available;
    }

    fn restart_as_admin(&mut self) {
        #[cfg(windows)]
        {
            match elevation::restart() {
                Ok(()) => std::process::exit(0),
                Err(()) => {
                    let message = "Administrator restart was cancelled or could not start. This window is still open.";
                    self.state = ConnState::Failed(message.into());
                    self.status_line = message.into();
                }
            }
        }
    }

    fn save_settings(&mut self) {
        self.settings_notice = if settings::save_checked(&self.settings).is_err() {
            Some("Settings could not be saved. Current choices may not be restored next time you open waX.")
        } else { None };
    }

    fn toggle_autostart(&mut self) {
        match autostart::apply(self.settings.autostart) {
            Ok(()) => {
                self.autostart_notice = None;
                self.save_settings();
            }
            Err(()) => {
                self.settings.autostart = !self.settings.autostart;
                self.autostart_notice = Some("Could not verify the startup change. Check Windows Startup settings before relying on it.".into());
            }
        }
    }

    fn import_text(&mut self, name: String, text: &str) -> bool {
        // Core import exclusively creates a new canonical name for every caller.
        match profile::import_conf(&name, text) {
            Ok(p) => {
                self.status_line = format!("Imported profile '{}'", p.name);
                self.import_error = None;
                self.reload_profiles();
                if matches!(self.state, ConnState::Disconnected | ConnState::Failed(_)) {
                    self.selected = self.profiles.iter().position(|item| item.name == p.name);
                    self.split_edit_for = None;
                }
                true
            }
            Err(e) => {
                self.import_error = Some(e.to_string());
                false
            }
        }
    }

    fn handle_events(&mut self, ctx: &egui::Context) {
        #[cfg(windows)]
        {
            let (want_quit, want_show) = match &self.instance {
                Some(inst) => (inst.take_quit(), inst.take_show()),
                None => (false, false),
            };
            if want_quit {
                self.quit_app();
            }
            if want_show {
                crate::single::diag("primary: show signal consumed");
                self.hidden = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                ctx.request_repaint();
            }
        }
        // Invalidate the episode before consuming a queued worker result.
        // A disabled automatic attempt must not revive via Connected events.
        if matches!(self.state, ConnState::Connecting) && self.recovery.should_cancel(self.settings.auto_reconnect) {
            self.disconnect();
        }
        while let Ok(event) = self.events_rx.try_recv() {
            match event {
                AppEvent::TunnelLost(generation) => {
                    if generation != self.recovery.generation() || !matches!(self.state, ConnState::Connected { .. }) { continue; }
                    self.state = ConnState::Connecting;
                    self.status_line = "Connection lost - cleaning up".into();
                    let tunnel = self.tunnel.clone();
                    let tx = self.events_tx.clone();
                    std::thread::spawn(move || {
                        let taken = tunnel.lock().unwrap().take();
                        let result = match taken {
                            Some(t) => t.shutdown(),
                            None => Tunnel::force_disconnect(),
                        }.map_err(|_| "Tunnel cleanup needs attention. Disconnect before retrying".to_string());
                        let _ = tx.send(AppEvent::LostCleanup(generation, result));
                    });
                }
                AppEvent::LostCleanup(generation, result) => {
                    if generation != self.recovery.generation() { continue; }
                    if let Err(message) = result {
                        self.cleanup_issue = Some(message.clone());
                        self.state = ConnState::Failed(message);
                    } else if !self.schedule_recovery() {
                        self.state = ConnState::Failed("Connection lost. Reconnect manually when ready".into());
                    }
                }
                AppEvent::Connected(generation, t) => {
                    if generation != self.recovery.generation() || !matches!(self.state, ConnState::Connecting) {
                        std::thread::spawn(move || { let _ = t.shutdown(); });
                        continue;
                    }
                    *self.tunnel.lock().unwrap() = Some(t);
                    self.state = ConnState::Connected { since: Instant::now() };
                    self.recovery.connected(generation, self.start.elapsed());
                    self.status_line = "Tunnel is up".to_string();
                    if let Some(profile) = &self.active_profile {
                        self.settings.last_profile = Some(profile.name.clone());
                        self.save_settings();
                    }
                    self.spawn_stats_loop();
                }
                AppEvent::ConnectFailed(generation, e) => {
                    if generation != self.recovery.generation() { continue; }
                    // Only an automatic recovery episode retries. An initial
                    // user connection failure remains explicit and actionable.
                    if self.recovery.attempts() == 0 || !self.schedule_recovery() {
                        self.state = ConnState::Failed(e.clone());
                        self.status_line = e;
                    }
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
                AppEvent::ExternalStatus(state) => {
                    if matches!(self.state, ConnState::Connected { .. }) && self.tunnel.lock().unwrap().is_none() {
                        match state {
                            Ok(Some(stats)) => {
                                self.stats = stats;
                                self.status_line = "Existing tunnel status refreshed".into();
                            }
                            Ok(None) => {
                                self.state = ConnState::Disconnected;
                                self.stats = TunnelStats::default();
                                self.status_line = "Externally owned tunnel stopped".into();
                            }
                            Err(()) => {
                                self.stats = TunnelStats::default();
                                self.status_line = "Tunnel status unavailable. Administrator access or the driver may need attention.".into();
                            }
                        }
                    }
                }
                AppEvent::DiagnosticsReady(report) => {
                    self.diagnostics_report = Some(report);
                    self.diagnostics_generated_at = Some(Instant::now());
                    self.diagnostics_busy = false;
                }
                AppEvent::Stats(generation, s) => {
                    if generation == self.recovery.generation() { self.stats = s; }
                }
                AppEvent::Stage(generation, stage) => {
                    if generation != self.recovery.generation() { continue; }
                    self.connect_stages.push(stage);
                    self.status_line = format!("{stage}...");
                }
                AppEvent::WarpDone(Ok(msg)) => {
                    self.warp_busy = false;
                    self.warp_notice = Some((msg.clone(), false));
                    self.status_line = msg;
                    self.refresh_warp_summary();
                    self.reload_profiles();
                }
                AppEvent::WarpDone(Err(e)) => {
                    self.warp_busy = false;
                    self.warp_notice = Some((e.clone(), true));
                    self.status_line = format!("WARP: {e}");
                    self.refresh_warp_summary();
                    self.reload_profiles();
                }
            }
        }
        if self.recovery.take_due(self.start.elapsed()).is_some() {
            if let Some(profile) = self.active_profile.clone() {
                self.connect_prepared(profile);
            }
        }
    }

    fn warp_register(&mut self) {
        // Keep consent and duplicate-operation checks at the effect boundary,
        // not only in the button's enabled state.
        if !self.warp_tos || self.warp_busy {
            return;
        }
        if profile::profile_path("Cloudflare WARP").exists() {
            self.warp_notice = Some((
                "A profile named 'Cloudflare WARP' already exists. Back it up and remove it before registering; it will not be overwritten".into(),
                true,
            ));
            return;
        }
        self.warp_busy = true;
        self.status_line = "Registering WARP device...".to_string();
        self.warp_notice = Some(("Registering WARP device...".into(), false));
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

    fn warp_restore_profile(&mut self) {
        if self.warp_busy || !matches!(self.state, ConnState::Disconnected | ConnState::Failed(_)) {
            return;
        }
        self.warp_busy = true;
        self.warp_notice = Some(("Restoring local WARP profile...".into(), false));
        let tx = self.events_tx.clone();
        std::thread::spawn(move || {
            let result = (|| -> Result<String, String> {
                if profile::profile_path("Cloudflare WARP").exists() {
                    return Err("A 'Cloudflare WARP' profile already exists; it was not overwritten".into());
                }
                let account = wax_warp::load_account()
                    .map_err(|e| e.to_string())?
                    .ok_or("No saved WARP account to restore from")?;
                let restored = wax_warp::store::warp_profile(&account, "Cloudflare WARP")
                    .map_err(|e| e.to_string())?;
                profile::save_profile(&restored).map_err(|e| e.to_string())?;
                Ok("WARP profile restored from the saved account. No new registration created".into())
            })();
            let _ = tx.send(AppEvent::WarpDone(result));
        });
    }

    fn warp_unregister(&mut self) {
        if self.warp_busy || !matches!(self.state, ConnState::Disconnected | ConnState::Failed(_)) {
            return;
        }
        self.warp_busy = true;
        self.warp_notice = Some(("Removing WARP registration...".into(), false));
        let tx = self.events_tx.clone();
        std::thread::spawn(move || {
            let result = (|| -> Result<String, String> {
                let account = wax_warp::load_account()
                    .map_err(|e| e.to_string())?
                    .ok_or("not registered")?;
                let client = wax_warp::WarpClient::new().map_err(|e| e.to_string())?;
                client.delete_device(&account.device_id, &account.token)
                    .map_err(|e| format!("Could not remove remote registration: {e}. Local account kept for retry"))?;
                wax_warp::delete_account().map_err(|e| format!(
                    "Remote registration removed, but local account cleanup failed: {e}"
                ))?;
                profile::delete_profile("Cloudflare WARP").map_err(|e| format!(
                    "Registration removed, but local WARP profile cleanup failed: {e}"
                ))?;
                Ok("WARP registration removed".to_string())
            })();
            let _ = tx.send(AppEvent::WarpDone(result));
        });
    }
}

fn background_command(tray_available: bool) -> egui::ViewportCommand {
    if tray_available { egui::ViewportCommand::Visible(false) }
    else { egui::ViewportCommand::Minimized(true) }
}

fn tray_notice(tray_available: bool) -> Option<&'static str> {
    (!tray_available).then_some("Tray icon unavailable. Closing minimizes this window instead of hiding it. Restore it from the taskbar; use Ctrl+Q to quit.")
}

fn profile_notice(skipped: usize) -> Option<String> {
    (skipped > 0).then(|| format!("{skipped} profile file(s) could not be read or parsed and were skipped. Files were not changed."))
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

fn split_profile_label(name: &str) -> String {
    format!("Excluded IP ranges for profile: {name}")
}

// Count only the loaded in-memory list, never all files on disk. A failed
// reload keeps that list visible, so label it as previous rather than current.
fn loaded_profiles_label(count: usize, readable: bool) -> String {
    let noun=if count==1 { "profile" } else { "profiles" };
    if readable { format!("{count} {noun} loaded") }
    else { format!("Previous list: {count} {noun}") }
}

/// Periodic polling only, not a frame-rate or CPU-use guarantee. Hidden mode
/// still polls tray and cross-instance signals within a one-second interval.
fn repaint_delay(hidden: bool, reduced_motion: bool, disconnected: bool) -> Duration {
    Duration::from_millis(if hidden { 1000 } else if reduced_motion && disconnected { 250 } else { 33 })
}

#[cfg(test)]
mod repaint_tests {
    use super::*;
    #[test]
    fn background_policy_never_hides_a_window_without_a_tray() {
        assert!(matches!(background_command(true), egui::ViewportCommand::Visible(false)));
        assert!(matches!(background_command(false), egui::ViewportCommand::Minimized(true)));
        assert!(tray_notice(true).is_none());
        assert!(tray_notice(false).unwrap().contains("Ctrl+Q"));
    }
    #[test]
    fn profile_notice_is_count_only_and_absent_for_complete_loads() {
        assert_eq!(profile_notice(0), None);
        assert_eq!(profile_notice(2).as_deref(), Some("2 profile file(s) could not be read or parsed and were skipped. Files were not changed."));
    }
    #[test]
    fn split_profile_context_preserves_the_profile_name() {
        assert_eq!(split_profile_label("Home"), "Excluded IP ranges for profile: Home");
        assert_eq!(split_profile_label("Office - 東京"), "Excluded IP ranges for profile: Office - 東京");
        let name = "long profile name ".repeat(20);
        assert_eq!(split_profile_label(&name), format!("Excluded IP ranges for profile: {name}"));
    }

    #[test]
    fn loaded_profile_count_distinguishes_current_and_retained_lists() {
        assert_eq!(loaded_profiles_label(0,true),"0 profiles loaded");
        assert_eq!(loaded_profiles_label(1,true),"1 profile loaded");
        assert_eq!(loaded_profiles_label(12,true),"12 profiles loaded");
        assert_eq!(loaded_profiles_label(0,false),"Previous list: 0 profiles");
        assert_eq!(loaded_profiles_label(1,false),"Previous list: 1 profile");
        assert_eq!(loaded_profiles_label(12,false),"Previous list: 12 profiles");
    }
    #[test]
    fn periodic_repaint_policy_keeps_active_ui_responsive_and_hidden_polling_bounded() {
        for reduced in [false, true] {
            for disconnected in [false, true] {
                assert_eq!(repaint_delay(true, reduced, disconnected), Duration::from_millis(1000));
                let expected = if reduced && disconnected { 250 } else { 33 };
                assert_eq!(repaint_delay(false, reduced, disconnected), Duration::from_millis(expected));
            }
        }
    }
}

impl eframe::App for WaxApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.style_mut_of(egui::Theme::Dark, |style| style.animation_time = if self.settings.reduced_motion { 0.0 } else { 0.175 });
        self.handle_events(&ctx);

        // Never hide an unreachable window when the tray failed to initialize.
        if ctx.input(|i| i.viewport().close_requested()) {
            self.close_to_background(&ctx);
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
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                ctx.request_repaint();
            }
            if toggle {
                match self.state {
                    ConnState::Disconnected | ConnState::Failed(_) => self.connect_manual(),
                    ConnState::Connected { .. } => self.disconnect(),
                    ConnState::Connecting if self.recovery.is_pending() => self.disconnect(),
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
                    ConnState::Connecting if self.recovery.is_pending() => self.disconnect(),
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
            if path.extension().and_then(|e| e.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("conf"))
            {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("imported")
                    .to_string();
                match std::fs::read_to_string(&path) {
                    Ok(text) => { self.import_text(name, &text); }
                    Err(e) => {
                        self.import_error = Some(format!("Could not read '{name}': {e}"));
                    }
                }
                self.tab = Tab::Profiles;
            }
        }


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
                        let close_hint = if self.tray_available() {
                            "Hide to tray - quit from the tray icon menu"
                        } else { "Minimize - tray unavailable" };
                        if chrome_button(ui, "\u{00d7}", close_hint) {
                            self.close_to_background(&ctx);
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
            .exact_size(if ctx.content_rect().width() < 840.0 { 152.0 } else { 176.0 })
            .frame(egui::Frame::default().fill(theme::PANEL).inner_margin(egui::Margin::symmetric(6, 10)))
            .show(ui, |ui| {
                ui.add_space(14.0);
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
                let title = match self.tab { Tab::Connect => "Connect", Tab::Profiles => "Profiles", Tab::Features => "Features", Tab::Warp => "WARP", Tab::About => "About" };
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title).font(theme::semi(14.0)).color(theme::TEXT));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| { ui.label(RichText::new(format!("v{VERSION}")).size(12.0).color(theme::SUB)); });
                });
                ui.add_space(12.0);
                theme::hairline(ui);
                ui.add_space(12.0);
                if let Some(notice) = tray_notice(self.tray_available()) {
                    ui.add(egui::Label::new(RichText::new(notice).size(12.0).color(theme::WARN)).wrap());
                    ui.add_space(8.0);
                }
                if let Some(notice) = &self.profiles_notice {
                    ui.add(egui::Label::new(RichText::new(notice).size(12.0).color(theme::WARN)).wrap());
                    ui.add_space(8.0);
                }
                if let Some(notice) = self.settings_notice {
                    ui.add(egui::Label::new(RichText::new(notice).size(12.0).color(theme::WARN)).wrap());
                    ui.add_space(8.0);
                }
                let fade = theme::enter(ui, ui.scope_id().with(("tab", self.tab)), self.settings.reduced_motion);
                ui.scope(|ui| {
                    ui.set_opacity(fade);
                    if self.tab == Tab::Connect {
                        self.tab_connect(ui);
                    } else {
                        egui::ScrollArea::vertical()
                            .id_salt(("tab-content", self.tab))
                            .auto_shrink([false, false])
                            .show(ui, |ui| match self.tab {
                                Tab::Profiles => self.tab_profiles(ui),
                                Tab::Features => self.tab_features(ui),
                                Tab::Warp => self.tab_warp(ui),
                                Tab::About => self.tab_about(ui),
                                Tab::Connect => unreachable!(),
                            });
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
        // One periodic request: an earlier shorter delay would override this
        // policy. egui input/animation requests may still wake the UI sooner.
        ctx.request_repaint_after(repaint_delay(
            self.hidden,
            self.settings.reduced_motion,
            matches!(self.state, ConnState::Disconnected),
        ));
    }
}

impl WaxApp {
    /// Explicit quit from the tray menu: disconnect cleanly, then exit.
    #[cfg(windows)]
    fn quit_app(&mut self) {
        let generation = self.recovery.new_intent();
        self.connection_epoch.store(generation, Ordering::Release);
        self.active_profile = None;
        #[cfg(windows)]
        crate::single::diag("quit: signal consumed, shutting down tunnel");
        let taken = self.tunnel.lock().unwrap().take();
        let result = if let Some(t) = taken {
            crate::single::diag("quit: owned tunnel cleanup starting");
            t.shutdown()
        } else {
            crate::single::diag("quit: leftover cleanup starting");
            let cleanup_start = Instant::now();
            Tunnel::force_disconnect_progress(|stage| {
                crate::single::diag(&format!("{stage} elapsed_ms={}", cleanup_start.elapsed().as_millis()));
            })
        };
        crate::single::diag(if result.is_ok() { "quit: cleanup succeeded" } else { "quit: cleanup returned error" });
        #[cfg(windows)]
        crate::single::diag("quit: cleanup finished, exiting");
        std::process::exit(0);
    }

    fn tab_connect(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let col_w = 480.0_f32.min(rect.width());
        let col = egui::Rect::from_min_size(
            egui::pos2(rect.center().x - col_w / 2.0, rect.top() + 12.0),
            egui::vec2(col_w, rect.height() - 12.0),
        );
        let mut content = ui.new_child(egui::UiBuilder::new().max_rect(col).layout(Layout::top_down(Align::Min)));
        let ui = &mut content;
        let (word, color, detail) = match &self.state {
            ConnState::Disconnected => ("Disconnected", theme::SUB, "Connect through your selected WireGuard profile."),
            ConnState::Connecting => ("Connecting", theme::WARN, "Preparing your WireGuard connection."),
            ConnState::Disconnecting => ("Disconnecting", theme::SUB, "Removing the tunnel and restoring network settings."),
            ConnState::Failed(_) => ("Connection failed", theme::BAD, "Review the connection message below before retrying."),
            ConnState::Connected { .. } => match connection_health::health(self.stats.last_handshake, SystemTime::now()) {
                connection_health::Health::AwaitingHandshake => ("Tunnel ready", theme::WARN, "Waiting for evidence from the peer."),
                connection_health::Health::RecentHandshake => ("Connected", theme::GOOD, "Your WireGuard tunnel is up."),
                connection_health::Health::OlderHandshake => ("Tunnel up", theme::WARN, "The peer handshake is older. Idle peers may not handshake."),
            },
        };
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(egui::vec2(8.0, 16.0), egui::Sense::hover());
            ui.painter().circle_filled(r.center(), 3.5, color);
            let context = match self.state { ConnState::Connected { .. } => "Peer evidence", ConnState::Connecting => "Establishing tunnel", ConnState::Failed(_) => "Action needed", _ => "WireGuard connection" };
            ui.label(RichText::new(context).size(13.0).color(theme::SUB));
        });
        ui.add_space(8.0);
        ui.label(RichText::new(word).font(theme::semi(32.0)).color(theme::TEXT));
        ui.add_space(4.0);
        let (detail_rect, _) = ui.allocate_exact_size(egui::vec2(col_w, 32.0), egui::Sense::hover());
        let mut detail_ui = ui.new_child(egui::UiBuilder::new().max_rect(detail_rect).layout(Layout::top_down(Align::Min)));
        detail_ui.add(egui::Label::new(RichText::new(detail).size(13.0).color(theme::SUB)).wrap());
        ui.add_space(8.0);
        theme::hairline(ui);
        ui.add_space(12.0);
        ui.label(RichText::new("Selected profile").size(12.0).color(theme::SUB));
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
            .unwrap_or_else(|| ("No profile selected".to_string(), "Import a WireGuard .conf profile".to_string()));
        let can_switch = matches!(self.state, ConnState::Disconnected | ConnState::Failed(_))
            && self.profiles.len() > 1;
        let name_resp = ui.add(
            egui::Label::new(RichText::new(&pname).font(theme::semi(16.0)).color(theme::SUB)).truncate()
                .sense(if can_switch { egui::Sense::click() } else { egui::Sense::hover() }),
        );
        if name_resp.hovered() && can_switch {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if name_resp.clicked() && can_switch {
            self.picker_open = !self.picker_open;
        }
        ui.add(egui::Label::new(RichText::new(&pendpoint).size(13.0).color(theme::SUB).monospace()).truncate()).on_hover_text(&pendpoint);
        let (hint_rect, _) = ui.allocate_exact_size(egui::vec2(col_w, 16.0), egui::Sense::hover());
        if can_switch {
            let mut hint_ui = ui.new_child(egui::UiBuilder::new().max_rect(hint_rect).layout(Layout::top_down(Align::Min)));
            hint_ui.label(RichText::new(if self.picker_open { "choose a profile" } else { "click to switch" }).size(10.0).color(theme::MUTE));
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
                        self.save_settings();
                        self.picker_open = false;
                        self.status_line = "Ready".into();
                    }
                }
            });
        }


        ui.add_space(8.0);
        theme::hairline(ui);
        ui.add_space(20.0);
        let (action_label, can_act) = match &self.state {
            ConnState::Disconnected | ConnState::Failed(_) => ("Connect", self.selected.is_some()),
            ConnState::Connected { .. } => ("Disconnect", true),
            ConnState::Connecting if self.recovery.is_pending() => ("Cancel retry", true),
            ConnState::Connecting => ("Connecting", false),
            ConnState::Disconnecting => ("Disconnecting", false),
        };
        let action_a11y = match &self.state {
            ConnState::Disconnected | ConnState::Failed(_) => self
                .selected
                .and_then(|i| self.profiles.get(i))
                .map(|p| format!("Connect to {}", p.name)),
            ConnState::Connected { .. } => self
                .selected
                .and_then(|i| self.profiles.get(i))
                .map(|p| format!("Disconnect from {}", p.name)),
            _ => None,
        };
        if theme::primary_button_named(
            ui,
            action_label,
            can_act,
            action_a11y.as_deref().unwrap_or(action_label),
        ) {
            match self.state {
                ConnState::Disconnected | ConnState::Failed(_) => self.connect_manual(),
                ConnState::Connected { .. } => self.disconnect(),
                ConnState::Connecting if self.recovery.is_pending() => self.disconnect(),
                _ => {}
            }
        }

        ui.add_space(8.0);
        let (stage_rect, _) = ui.allocate_exact_size(egui::vec2(col_w, 28.0), egui::Sense::hover());
        let mut stage_ui = ui.new_child(egui::UiBuilder::new().max_rect(stage_rect).layout(Layout::top_down(Align::Center)));
        if self.selected.is_none() {
            stage_ui.label(RichText::new("Add a profile from Profiles to connect").size(12.0).color(theme::SUB));
        } else if !self.is_admin {
            if stage_ui.link(RichText::new("Restart as administrator to connect").size(12.0).color(theme::WARN)).clicked() { self.restart_as_admin(); }
        } else if let ConnState::Failed(msg) = &self.state {
            stage_ui.add(egui::Label::new(RichText::new(msg).size(12.0).color(theme::BAD)).wrap());
        } else if matches!(self.state, ConnState::Connecting) {
            let stage = if self.recovery.is_pending() { self.status_line.as_str() } else { self.connect_stages.last().copied().unwrap_or("Starting") };
            stage_ui.label(RichText::new(stage).size(12.0).color(theme::SUB));
        }
        theme::hairline(ui);
        ui.add_space(12.0);
        let connected = matches!(self.state, ConnState::Connected { .. });
        let session = match self.state { ConnState::Connected { since } => fmt_duration(since.elapsed()), _ => "-".into() };
        let rx = if connected { fmt_bytes(self.stats.rx_bytes) } else { "-".into() };
        let txb = if connected { fmt_bytes(self.stats.tx_bytes) } else { "-".into() };
        ui.columns(3, |columns| {
            mini_stat(&mut columns[0], "Session", &session);
            mini_stat(&mut columns[1], "Received", &rx);
            mini_stat(&mut columns[2], "Sent", &txb);
        });
        ui.add_space(12.0);
        theme::hairline(ui);
        ui.add_space(12.0);
        let evidence = if connected {
            let age = self.stats.last_handshake.and_then(|h| SystemTime::now().duration_since(h).ok());
            match connection_health::health(self.stats.last_handshake, SystemTime::now()) {
                connection_health::Health::AwaitingHandshake => "Waiting for peer handshake. Internet reachability is not tested.".into(),
                connection_health::Health::RecentHandshake => format!("Peer handshake {} ago. Internet reachability is not tested.", age.map(fmt_duration).unwrap_or_default()),
                connection_health::Health::OlderHandshake => format!("Last handshake {} ago. Idle peers may not handshake.", age.map(fmt_duration).unwrap_or_default()),
            }
        } else { "Traffic can use the normal connection when the tunnel is off.".into() };
        ui.add(egui::Label::new(RichText::new(evidence).size(12.0).color(theme::SUB)).wrap());
            if let Some(issue) = self.cleanup_issue.clone() {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(issue).size(10.5).color(theme::WARN));
                    let can_retry = matches!(self.state, ConnState::Disconnected | ConnState::Failed(_));
                    let retry = ui.add_enabled_ui(can_retry, |ui| {
                        ui.link(RichText::new("Retry").size(10.5).color(theme::WARN))
                    }).inner.on_disabled_hover_text("Disconnect before retrying cleanup");
                    if retry.clicked() && can_retry {
                        // Lock controls until the cleanup result arrives. This
                        // prevents a concurrent Connect from being torn down.
                        self.state = ConnState::Disconnecting;
                        self.status_line = "Retrying cleanup...".into();
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

    }

    fn tab_profiles(&mut self, ui: &mut egui::Ui) {
        theme::section_heading(ui, "Profiles", "Drop a .conf file to add a profile. Existing profiles are kept; duplicate names get a number.");
        let mut connect_idx: Option<usize> = None;
        let mut delete_idx: Option<usize> = None;
        let can_manage = matches!(self.state, ConnState::Disconnected | ConnState::Failed(_));
        ui.horizontal_wrapped(|ui| {
            if ui.add_enabled(can_manage, egui::Button::new("Reload profiles")).clicked() {
                self.reload_profiles();
            }
            ui.label(RichText::new(loaded_profiles_label(self.profiles.len(), self.profiles_readable))
                .size(11.5).color(theme::MUTE));
        });
        ui.add_space(8.0);
        let can_manage = can_manage && self.profiles_readable;
        if !matches!(self.state, ConnState::Disconnected | ConnState::Failed(_)) {
            ui.label(RichText::new("Disconnect before switching or deleting profiles").size(11.0).color(theme::MUTE));
            ui.add_space(8.0);
        }
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
                // Reserve action space before laying out potentially long text.
                let text_width = (row_ui.available_width() - 280.0).max(80.0);
                row_ui.allocate_ui_with_layout(
                    egui::vec2(text_width, 48.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.add(egui::Label::new(
                            RichText::new(&p.name)
                                .size(13.5)
                                .color(if selected { theme::TEXT } else { theme::SUB })
                                .strong(),
                        ).truncate()).on_hover_text(&p.name);
                        let detail = format!("{ep}  ·  {addr}");
                        ui.add(egui::Label::new(
                            RichText::new(&detail)
                                .size(10.5)
                                .color(theme::MUTE)
                                .monospace(),
                        ).truncate()).on_hover_text(detail);
                    },
                );
                row_ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let now = ui.input(|inp| inp.time);
                    let armed = matches!(self.confirm_delete, Some((idx, t0)) if idx == i && now - t0 < 3.0);
                    if ui.add_enabled_ui(can_manage, |ui| {
                        theme::danger_button(ui, if armed { "Confirm delete" } else { "Delete" })
                    }).inner {
                        if armed {
                            delete_idx = Some(i);
                            self.confirm_delete = None;
                        } else {
                            self.confirm_delete = Some((i, now));
                        }
                    }
                    if can_manage
                        && ui
                            .add(egui::Button::new(RichText::new("Connect").size(12.0).color(theme::ACCENT))
                                .fill(theme::alpha(theme::ACCENT, 18))
                                .stroke(egui::Stroke::new(1.0, theme::alpha(theme::ACCENT, 90)))
                                .corner_radius(egui::CornerRadius::same(8)))
                            .clicked()
                    {
                        connect_idx = Some(i);
                    }
                    let sel_resp = ui.add_enabled(
                        can_manage,
                        egui::Button::new(
                            RichText::new(if selected { "Selected" } else { "Select" })
                                .size(11.5)
                                .color(if selected { theme::ACCENT } else { theme::SUB }),
                        )
                        .fill(if selected { theme::alpha(theme::ACCENT, 18) } else { theme::alpha(egui::Color32::WHITE, 6) })
                        .stroke(egui::Stroke::new(1.0, theme::alpha(if selected { theme::ACCENT } else { theme::SUB }, 60)))
                        .corner_radius(egui::CornerRadius::same(8)),
                    );
                    if sel_resp.clicked() && can_manage {
                        self.selected = Some(i);
                        self.split_edit_for = None;
                        self.confirm_delete = None;
                    }
                });
                theme::hairline(ui);
            }
            if self.profiles.is_empty() && self.profiles_readable {
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
                let name = p.name.clone();
                match profile::delete_profile(&name) {
                    Ok(()) => {
                        self.settings.split_exclusions.remove(&name);
                        if self.settings.last_profile.as_deref() == Some(name.as_str()) {
                            self.settings.last_profile = None;
                        }
                        self.save_settings();
                        self.reload_profiles();
                        self.split_edit_for = None;
                        self.import_error = None;
                        self.status_line = format!("Deleted profile '{name}'");
                    }
                    Err(e) => {
                        self.import_error = Some(format!("Could not delete '{name}': {e}"));
                    }
                }
            }
        }
        if let Some(i) = connect_idx {
            self.selected = Some(i);
            self.confirm_delete = None;
            self.connect_manual();
        }

        if let Some(sel) = self.selected {
            if self.profiles.get(sel).is_some() {
                if self.split_edit_for != Some(sel) {
                    self.split_edit_for = Some(sel);
                    self.split_edit = self
                        .profiles
                        .get(sel)
                        .and_then(|pr| self.settings.split_exclusions.get(&pr.name).cloned())
                        .unwrap_or_default();
                }
                ui.add_space(10.0);
                theme::section_heading(ui, "Split tunneling", "Applies to the named profile on next connect.");
                ui.add(egui::Label::new(RichText::new("Enter CIDR ranges separated by commas or spaces.")
                    .size(11.5).color(theme::SUB)).wrap());
                ui.add_space(4.0);
                if let Some(profile) = self.profiles.get(sel) {
                    ui.add(egui::Label::new(RichText::new(split_profile_label(&profile.name))
                        .size(12.0).color(theme::TEXT)).wrap());
                    ui.add_space(4.0);
                }
                let input_id = ui.make_persistent_id("split-exclusions-input");
                let focused = ui.memory(|m| m.has_focus(input_id));
                let invalid = wax_core::split::parse_exclusions(&self.split_edit).is_err();
                let border = if invalid {
                    theme::alpha(theme::BAD, 140)
                } else if focused {
                    theme::alpha(theme::ACCENT, 140)
                } else {
                    theme::alpha(theme::SUB, 45)
                };
                let resp = egui::Frame::default()
                    .fill(theme::PANEL)
                    .stroke(egui::Stroke::new(1.0, border))
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.split_edit)
                                .id(input_id)
                                .font(egui::FontId::monospace(12.0))
                                .text_color(theme::TEXT)
                                .frame(egui::Frame::NONE)
                                .hint_text("192.168.0.0/16, fd00::/8")
                                .desired_width(f32::INFINITY),
                        )
                    })
                    .inner;
                resp.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Excluded IP ranges")
                });
                if resp.changed() {
                    if let Some(pr) = self.profiles.get(sel) {
                        let name = pr.name.clone();
                        if self.split_edit.trim().is_empty() {
                            self.settings.split_exclusions.remove(&name);
                        } else {
                            self.settings
                                .split_exclusions
                                .insert(name, self.split_edit.trim().to_string());
                        }
                        self.save_settings();
                    }
                }
                match wax_core::split::parse_exclusions(&self.split_edit) {
                    Ok(excls) if !excls.is_empty() => {
                        ui.label(
                            RichText::new(format!("{} excluded range(s)", excls.len()))
                                .size(10.5)
                                .color(theme::MUTE),
                        );
                    }
                    Ok(_) => {
                        ui.add_space(4.0);
                        ui.label(RichText::new("No exclusions for this profile").size(11.0).color(theme::MUTE));
                    }
                    Err(e) => {
                        ui.label(RichText::new(format!("{e}")).size(11.0).color(theme::BAD));
                    }
                }
            }
        }

        ui.add_space(8.0);
        theme::section_heading(ui, "Import", "Paste a WireGuard config");
        theme::card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Profile name").size(11.5).color(theme::SUB));
                let name_id = ui.make_persistent_id("wireguard-import-name");
                let name_focused = ui.memory(|m| m.has_focus(name_id));
                let resp = egui::Frame::default()
                    .fill(theme::PANEL)
                    .stroke(egui::Stroke::new(1.0, theme::alpha(
                        if name_focused { theme::ACCENT } else { theme::SUB },
                        if name_focused { 140 } else { 45 },
                    )))
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.import_name)
                                .id(name_id)
                                .frame(egui::Frame::NONE)
                                .text_color(theme::TEXT)
                                .hint_text("Optional, e.g. Home")
                                .desired_width(220.0),
                        )
                    }).inner;
                resp.widget_info(|| egui::WidgetInfo::labeled(
                    egui::Role::TextInput, true, "Imported profile name"
                ));
            });
            ui.add_space(8.0);
            let input_id = ui.make_persistent_id("wireguard-import-input");
            let focused = ui.memory(|m| m.has_focus(input_id));
            let border = if focused {
                theme::alpha(theme::ACCENT, 140)
            } else {
                theme::alpha(theme::SUB, 45)
            };
            egui::Frame::default()
                .fill(theme::PANEL)
                .stroke(egui::Stroke::new(1.0, border))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("import-scroll")
                        .max_height(110.0)
                        .show(ui, |ui| {
                            let resp = ui.add(
                                egui::TextEdit::multiline(&mut self.import_paste)
                                    .id(input_id)
                                    .font(egui::FontId::monospace(12.0))
                                    .text_color(theme::TEXT)
                                    .frame(egui::Frame::NONE)
                                    .desired_rows(4)
                                    .hint_text("[Interface]\nPrivateKey = ...")
                                    .desired_width(f32::INFINITY),
                            );
                            resp.widget_info(|| egui::WidgetInfo::labeled(
                                egui::Role::TextInput, true, "WireGuard configuration to import"
                            ));
                        });
                });
            ui.add_space(4.0);
            if theme::primary_button(ui, "Import config", !self.import_paste.trim().is_empty()) {
                let text = self.import_paste.trim().to_string();
                let name = if self.import_name.trim().is_empty() {
                    format!("Imported {}", chrono_free_stamp())
                } else {
                    self.import_name.trim().to_string()
                };
                if self.import_text(name, &text) {
                    self.import_paste.clear();
                    self.import_name.clear();
                }
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
            &mut self.settings.autostart,
            "Start with Windows",
            "Launch waX VPN when you sign in",
        ) {
            self.toggle_autostart();
        }
        if let Some(notice) = &self.autostart_notice {
            ui.add(egui::Label::new(RichText::new(notice).size(12.0).color(theme::WARN)).wrap());
            ui.add_space(4.0);
        }
        ui.add_space(4.0);
        if theme::toggle_row(
            ui,
            &mut self.settings.auto_reconnect,
            "Auto-reconnect",
            "Retry owned adapter loss after 2, 4 and 8 seconds; budget resets after 60 seconds up. Physical-network roaming is not supported yet.",
        ) {
            self.save_settings();
        }
        ui.add_space(4.0);
        if theme::toggle_row(
            ui,
            &mut self.settings.reduced_motion,
            "Reduced motion",
            "Calm the interface animation",
        ) {
            self.save_settings();
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
            if let Some((notice, failed)) = &self.warp_notice {
                ui.add_space(4.0);
                ui.label(RichText::new(notice).size(11.5).color(
                    if *failed { theme::BAD } else { theme::SUB }
                ));
            }
            ui.add_space(6.0);
            if self.warp_registered {
                let idle = matches!(self.state, ConnState::Disconnected | ConnState::Failed(_));
                if !self.profiles.iter().any(|p| p.name == "Cloudflare WARP") {
                    ui.label(RichText::new("The local WARP profile is missing").size(11.5).color(theme::SUB));
                    if theme::primary_button(ui, "Restore WARP profile", !self.warp_busy && idle) {
                        self.warp_restore_profile();
                    }
                    ui.label(RichText::new("Uses the saved account on this device, without a new registration").size(11.0).color(theme::MUTE));
                    ui.add_space(8.0);
                }
                let unregister = ui.add_enabled_ui(!self.warp_busy && idle, |ui| {
                    theme::danger_button(ui, "Unregister WARP")
                }).inner;
                if unregister {
                    self.warp_unregister();
                }
                if !idle {
                    ui.label(RichText::new("Disconnect before removing WARP registration").size(11.0).color(theme::MUTE));
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
        ui.add_space(10.0);
        theme::card_frame().show(ui, |ui| {
            ui.label(RichText::new("What WARP does").size(11.0).color(theme::MUTE));
            ui.add_space(6.0);
            let rows = [
                "Routes your traffic through Cloudflare's network over WireGuard.",
                "Free. Registering creates a WARP account tied to this device.",
                "Afterwards, 'Cloudflare WARP' shows up in Profiles and connects like any profile.",
            ];
            for r in rows {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("·").size(11.5).color(theme::ACCENT));
                    ui.label(RichText::new(r).size(11.5).color(theme::SUB));
                });
                ui.add_space(2.0);
            }
        });
    }

    fn tab_about(&mut self, ui: &mut egui::Ui) {
        theme::section_heading(ui, "About", "");
        theme::card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::hover());
                rail_icon(ui.painter(), Tab::Connect, rect.center(), 14.0, theme::ACCENT);
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.add_space(2.0);
                    ui.label(RichText::new("waX VPN").size(17.0).color(theme::TEXT).strong());
                    ui.label(
                        RichText::new(format!("v{VERSION}"))
                            .size(11.0)
                            .color(theme::MUTE),
                    );
                });
            });
            ui.add_space(10.0);
            ui.label(
                RichText::new("A WireGuard client for Windows written in Rust, using the official wireguard-nt driver.")
                    .size(11.5)
                    .color(theme::SUB),
            );
            ui.add_space(6.0);
            ui.hyperlink_to(
                RichText::new("github.com/vio137/wax-vpn").size(12.0).color(theme::ACCENT),
                "https://github.com/vio137/wax-vpn",
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new("Not affiliated with Cloudflare or the WireGuard project.")
                    .size(10.5)
                    .color(theme::MUTE),
            );
        });
        ui.add_space(10.0);
        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new("Local diagnostics").size(13.0).color(theme::TEXT).strong());
            ui.add_space(4.0);
            ui.add(egui::Label::new(RichText::new("Build and adapter status only. No profiles, keys, endpoints or logs. Nothing is uploaded.").size(11.5).color(theme::SUB)).wrap());
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if ui.add_enabled(!self.diagnostics_busy, egui::Button::new(if self.diagnostics_busy { "Reading local status..." } else { "Generate local report" })).clicked() {
                    self.diagnostics_busy = true;
                    let tx = self.events_tx.clone();
                    std::thread::spawn(move || {
                        let report = wax_diagnostics::report(VERSION, wax_tunnel::is_elevated(), Tunnel::existing_status());
                        let _ = tx.send(AppEvent::DiagnosticsReady(report));
                    });
                }
                if let Some(report) = &self.diagnostics_report {
                    if ui.button("Copy report").clicked() { ui.ctx().copy_text(report.clone()); }
                    if ui.button("Clear").clicked() {
                        self.diagnostics_report = None;
                        self.diagnostics_generated_at = None;
                    }
                }
            });
            if let Some(report) = &self.diagnostics_report {
                ui.add_space(8.0);
                if let Some(generated) = self.diagnostics_generated_at {
                    ui.add(egui::Label::new(RichText::new(format!(
                        "Report age: {}. Generate a new report to refresh.", fmt_duration(generated.elapsed())
                    )).size(11.5).color(theme::SUB)).wrap());
                }
                ui.add(egui::Label::new(RichText::new("Snapshot, not a live monitor. Review before sharing. Internet reachability is not tested.").size(10.5).color(theme::MUTE)).wrap());
                egui::ScrollArea::vertical().id_salt("diagnostics-report").max_height(200.0).show(ui, |ui| {
                    let mut text = report.as_str();
                    ui.add(egui::TextEdit::multiline(&mut text).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY));
                });
            }
        });
        ui.add_space(10.0);
        theme::card_frame().show(ui, |ui| {
            ui.label(RichText::new("Built with").size(11.0).color(theme::MUTE));
            ui.add_space(6.0);
            let rows = [
                ("Core", "Rust and the official wireguard-nt driver"),
                ("Interface", "egui, pure Rust"),
                ("Type", "Inter Display"),
                ("License", "MIT"),
            ];
            for (k, v) in rows {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{k:<10}"))
                            .size(11.5)
                            .color(theme::MUTE)
                            .monospace(),
                    );
                    ui.label(RichText::new(v).size(11.5).color(theme::TEXT));
                });
                ui.add_space(2.0);
            }
            ui.add_space(4.0);
            ui.label(
                RichText::new("WireGuard is a registered trademark of Jason A. Donenfeld.")
                    .size(10.5)
                    .color(theme::MUTE),
            );
        });
    }
}

fn mini_stat(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.set_min_width(ui.available_width());
        ui.label(RichText::new(label).size(11.0).color(theme::MUTE));
        ui.label(RichText::new(value).font(theme::semi(16.0)).color(theme::TEXT));
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
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 40.0), egui::Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), label));
    let hovered = ui.ctx().animate_bool_responsive(resp.id.with("hover"), resp.hovered());
    let opacity = if active { 28 } else { (12.0 * hovered) as u8 };
    ui.painter().rect_filled(rect, egui::CornerRadius::same(8), theme::alpha(theme::TEXT, opacity));
    let color = if active { theme::TEXT } else { theme::SUB };
    rail_icon(ui.painter(), tab, rect.left_center() + egui::vec2(20.0, 0.0), 7.0, color);
    ui.painter().text(rect.left_center() + egui::vec2(40.0, 0.0), egui::Align2::LEFT_CENTER, label,
        if active { theme::semi(14.0) } else { egui::FontId::proportional(14.0) }, color);
    if resp.has_focus() { ui.painter().rect_stroke(rect, egui::CornerRadius::same(8), egui::Stroke::new(1.0, theme::ACCENT), egui::StrokeKind::Inside); }
    if resp.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
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
    if std::env::args_os().any(|a| a == "--quit") {
        #[cfg(windows)]
        if !single::request_quit() {
            eprintln!("waX VPN is not running");
        }
        return Ok(());
    }
    #[cfg(windows)]
    if elevation::finish_restart().is_err() {
        eprintln!("Administrator restart could not complete its bounded parent handoff");
        return Ok(());
    }
    #[cfg(windows)]
    let instance = match single::acquire() {
        single::Acquire::Primary(inst) => Some(inst),
        single::Acquire::AlreadyRunning => return Ok(()),
    };
    #[cfg(not(windows))]
    let instance: Option<single::Instance> = None;
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
        Box::new(|cc| Ok(Box::new(WaxApp::new(cc, instance)))),
    )
                    }
