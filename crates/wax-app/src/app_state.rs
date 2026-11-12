// waX VPN - WireGuard client for Windows. egui front end over wax-tunnel.



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
mod profile_view;
#[path = "profile_summary.rs"]
mod profile_summary;
#[cfg(any(windows, test))]
mod profile_file;
mod measurements;

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
    Network,
}


enum AppEvent {
    IpBefore(u64, String, Result<String, String>),
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
    measurements: measurements::Monitor,
    diagnostics_report: Option<String>,
    diagnostics_generated_at: Option<Instant>,
    diagnostics_busy: bool,
    autostart_notice: Option<String>,
    settings_notice: Option<&'static str>,
    settings: Settings,
    profiles: Vec<Profile>,
    profile_view: profile_view::ProfileView,
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
    import_check: Option<Result<profile::ImportSummary, ()>>,
    #[cfg(windows)]
    file_picker: Option<Receiver<profile_file::PickerResult>>,
    tab: Tab,
    hidden: bool,
    connect_stages: Vec<&'static str>,
    recovery: recovery::Recovery,
    connection_epoch: Arc<AtomicU64>,
    active_profile: Option<Profile>,
    picker_open: bool,
    picker_search: String,
    picker_return_focus: bool,
    keyboard_help_open: bool,
    profile_search: String,
    pinned_only: bool,
    profile_copy_notice: Option<String>,
    profile_summary_open: bool,
    profile_routes_open: bool,
    profile_summary_cache: profile_summary::DisplayCache,
    open_network_settings: bool,
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
        Self::from_context(&cc.egui_ctx, instance)
    }

    fn from_context(
        ctx: &egui::Context,
        #[cfg_attr(not(windows), allow(unused_variables))] instance: Option<single::Instance>,
    ) -> Self {
        ctx.set_theme(egui::Theme::Dark);
        ctx.style_mut_of(egui::Theme::Dark, |style| {
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
        ctx.set_fonts(fonts);

        let (tx, rx) = channel();
        startup::record("fonts ready; loading settings");
        let (settings, settings_notice) = match settings::load_checked() {
            Ok(settings) => (settings, None),
            Err(()) => {
                let mut settings = Settings::default();
                // Never revive automatic recovery from an unreadable choice.
                settings.auto_reconnect = false;
                (settings, Some("Saved settings could not be read and were kept unchanged. Auto-reconnect is off for this session. Check your settings file before relying on saved preferences."))
            }
        };
        startup::record("settings handled; loading profiles");
        let (profiles, profiles_notice, profiles_readable) = match profile::load_profiles_report() {
            Ok(report) => (report.profiles, profile_notice(report.skipped), true),
            Err(_) => (Vec::new(), Some("Profiles could not be read. Files were not changed. Reload in Profiles before connecting.".into()), false),
        };
        let selected = settings
            .last_profile
            .as_ref()
            .and_then(|last| profiles.iter().position(|p| &p.name == last))
            .or(if profiles.is_empty() { None } else { Some(0) });

        startup::record("profiles handled; loading WARP account");
        let warp_registered = matches!(wax_warp::load_account(), Ok(Some(_)));
        let mut app = Self {
            measurements: measurements::Monitor::default(),
            diagnostics_report: None,
            diagnostics_generated_at: None,
            diagnostics_busy: false,
            autostart_notice: None,
            settings_notice,
            settings,
            profiles,
            profile_view: profile_view::ProfileView::default(),
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
            import_check: None,
            #[cfg(windows)]
            file_picker: None,
            tab: Tab::Connect,
            hidden: false,
            connect_stages: Vec::new(),
            recovery: recovery::Recovery::default(),
            connection_epoch: Arc::new(AtomicU64::new(0)),
            active_profile: None,
            picker_open: false,
            picker_search: String::new(),
            picker_return_focus: false,
            keyboard_help_open: false,
            profile_search: String::new(),
            pinned_only: false,
            profile_copy_notice: None,
            profile_summary_open: false,
            profile_routes_open: false,
            profile_summary_cache: profile_summary::DisplayCache::default(),
            open_network_settings: false,
            split_edit: String::new(),
            split_edit_for: None,
            instance,
            confirm_delete: None,
            cleanup_issue: None,
            #[cfg(windows)]
            tray: if std::env::args_os().any(|arg| arg == "--no-tray") { None } else { tray::Tray::new(&ctx) },
        };
        startup::record("state and tray initialized");
        app.refresh_warp_summary();
        // Pick up a tunnel left running by an earlier instance.
        startup::record("checking existing tunnel");
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
                self.profile_summary_cache.invalidate();
                self.profile_view.invalidate();
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
        self.measurements.new_intent();
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
        let original_ip = (self.settings.network.public_ip && self.recovery.attempts() == 0)
            .then(|| self.settings.network.ip_endpoint.clone());
        let epoch = self.connection_epoch.clone();
        std::thread::spawn(move || {
            // Dispose of a lost owned adapter before creating its replacement.
            if let Some(old) = tunnel.lock().unwrap().take() {
                if old.shutdown().is_err() {
                    let _ = tx.send(AppEvent::ConnectFailed(generation, "Previous tunnel cleanup failed; disconnect before retrying".into()));
                    return;
                }
            }
            // Original-IP observation happens before adapter creation, never on recovery.
            // Failure is visible but does not prevent connecting. No UI thread network I/O.
            if epoch.load(Ordering::Acquire) != generation { return; }
            if let Some(url) = original_ip {
                let _ = tx.send(AppEvent::Stage(generation, "Checking public IP before tunnel"));
                let result = measurements::public_ip(&url);
                let _ = tx.send(AppEvent::IpBefore(generation, url, result));
            }
            if epoch.load(Ordering::Acquire) != generation { return; }
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
        startup::record("close-to-background requested");
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
        self.profile_view.invalidate();
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

    fn duplicate_selected_profile(&mut self) {
        // Check again at the file-writing boundary. Never change selection,
        // connection state or per-name app settings when making a local copy.
        if !self.profiles_readable || !matches!(self.state, ConnState::Disconnected | ConnState::Failed(_)) { return; }
        let Some(source) = self.selected.and_then(|i| self.profiles.get(i)) else { return; };
        let name = format!("{} - copy", source.name);
        match profile::import_conf(&name, &source.config.to_conf_string()) {
            Ok(copy) => {
                self.reload_profiles();
                self.profile_copy_notice = Some(format!("Created '{}'. Same keys; app settings were not copied. Selection unchanged.", copy.name));
            }
            Err(_) => {
                self.profile_copy_notice = Some("Could not create a local copy. Existing profiles were not replaced.".into());
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
        if let Some(rx) = &self.file_picker {
            match rx.try_recv() {
                Ok(result) => {
                    self.file_picker = None;
                    match result {
                        Ok(Some(file)) => { self.import_text(file.name, &file.text); }
                        Ok(None) => {}
                        Err(message) => { self.import_error = Some(message.into()); }
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.file_picker = None;
                    self.import_error = Some("The Windows file picker closed unexpectedly".into());
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }

        #[cfg(windows)]
        {
            let (want_quit, want_show) = match &self.instance {
                Some(inst) => (inst.take_quit(), inst.take_show()),
                None => (false, false),
            };
            if want_quit {
                startup::record("quit event received");
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
                AppEvent::IpBefore(generation, endpoint, result) => {
                    if generation == self.recovery.generation() && self.settings.network.public_ip
                        && endpoint == self.settings.network.ip_endpoint {
                        self.measurements.before = Some(result);
                    }
                }
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
                        self.settings.record_connection(&profile.name);
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

fn profile_match_label(count: usize) -> String {
    format!("{count} {}", if count == 1 { "match" } else { "matches" })
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
fn repaint_delay(hidden: bool, transitioning: bool) -> Duration {
    Duration::from_millis(if hidden { 1000 } else if transitioning { 33 } else { 250 })
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
    fn filtered_match_count_labels_zero_single_and_many() {
        assert_eq!(profile_match_label(0), "0 matches");
        assert_eq!(profile_match_label(1), "1 match");
        assert_eq!(profile_match_label(26), "26 matches");
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
        for transitioning in [false, true] {
            assert_eq!(repaint_delay(true, transitioning), Duration::from_millis(1000));
            assert_eq!(repaint_delay(false, transitioning), Duration::from_millis(if transitioning { 33 } else { 250 }));
        }
    }
                }
