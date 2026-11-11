impl WaxApp {
    fn draw_ui(&mut self, ui: &mut egui::Ui) {
        static FRAME_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let startup_frame = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
        if startup_frame == 0 { startup::record("first UI frame entered"); }
        if startup_frame == 10 { startup::record("ten UI frames entered"); }
        let ctx = ui.ctx().clone();
        theme::sync_motion(&ctx, self.settings.reduced_motion);
        self.handle_events(&ctx);
        let phase = match self.state { ConnState::Connected { .. } => 2, ConnState::Connecting | ConnState::Disconnecting => 1, _ => 0 };
        self.measurements.tick(&self.settings.network, self.recovery.generation(), phase,
            self.tab == Tab::Network && !self.hidden, self.stats.rx_bytes, self.stats.tx_bytes);

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
                startup::record("tray quit action received");
                self.quit_app();
            }
        }

        // Find is safe while editing: it changes the view, never a tunnel.
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::F)) {
            self.tab = Tab::Profiles;
            ctx.memory_mut(|m| m.request_focus(egui::Id::new("profile-search")));
        }

        // Connection shortcuts only when no text field owns the keyboard.
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
                        i.modifiers.ctrl && i.key_pressed(egui::Key::Num6),
                    ],
                )
            });
            for (idx, hit) in digits.iter().enumerate() {
                if *hit {
                    self.tab = [Tab::Connect, Tab::Profiles, Tab::Features, Tab::Warp, Tab::About, Tab::Network][idx];
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
                startup::record("Ctrl+Q received");
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
                    (Tab::Network, "Network"),
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
                let title = match self.tab { Tab::Connect => "Connect", Tab::Profiles => "Profiles", Tab::Features => "Features", Tab::Warp => "WARP", Tab::About => "About", Tab::Network => "Network" };
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
                        egui::ScrollArea::vertical().id_salt("connect-scroll").auto_shrink([false, false]).show(ui, |ui| self.tab_connect(ui));
                    } else {
                        egui::ScrollArea::vertical()
                            .id_salt(("tab-content", self.tab))
                            .auto_shrink([false, false])
                            .show(ui, |ui| match self.tab {
                                Tab::Profiles => self.tab_profiles(ui),
                                Tab::Features => self.tab_features(ui),
                                Tab::Warp => self.tab_warp(ui),
                                Tab::About => self.tab_about(ui),
                                Tab::Network => {
                                    if self.measurements.ui(ui, &self.settings.network,
                                        matches!(self.state, ConnState::Connected { .. } | ConnState::Disconnected | ConnState::Failed(_)),
                                        matches!(self.state, ConnState::Connected { .. })) {
                                        self.open_network_settings = true;
                                        self.tab = Tab::Features;
                                    }
                                },
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
        if startup_frame == 0 { startup::record("first UI frame complete"); }
        ctx.request_repaint_after(repaint_delay(
            self.hidden,
            matches!(self.state, ConnState::Connecting | ConnState::Disconnecting),
        ));
    }
}

impl eframe::App for WaxApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) { self.draw_ui(ui); }
}
impl egui_software_backend::App for WaxApp {
    fn ui(&mut self, ui: &mut egui::Ui, _backend: &mut egui_software_backend::SoftwareBackend) { self.draw_ui(ui); }
}
