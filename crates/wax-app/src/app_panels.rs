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
                    .and_then(|peer| peer.endpoint.as_deref())
                    .unwrap_or("no endpoint");
                (p.name.as_str(), ep)
            })
            .unwrap_or(("No profile selected", "Import a WireGuard .conf profile"));
        let can_switch = matches!(self.state, ConnState::Disconnected | ConnState::Failed(_))
            && self.profiles.len() > 1;
        let name_resp = ui.add(
            egui::Label::new(RichText::new(pname).font(theme::semi(16.0)).color(theme::SUB)).truncate()
                .sense(if can_switch { egui::Sense::click() } else { egui::Sense::hover() }),
        );
        if name_resp.hovered() && can_switch {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if name_resp.clicked() && can_switch {
            self.picker_open = !self.picker_open;
        }
        ui.add(egui::Label::new(RichText::new(pendpoint).size(13.0).color(theme::SUB).monospace()).truncate()).on_hover_text(pendpoint);
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
                            p.name.as_str(),
                            p.config
                                .peers
                                .first()
                                .and_then(|x| x.endpoint.as_deref())
                                .unwrap_or("-"),
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
                        self.settings.last_profile = Some(name.to_owned());
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
            if let Some(issue) = &self.cleanup_issue {
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
                    .and_then(|peer| peer.endpoint.as_deref())
                    .unwrap_or("-");
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
                let mut parsed_exclusions = wax_core::split::parse_exclusions(&self.split_edit);
                let invalid = parsed_exclusions.is_err();
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
                    // Input can change after the border was drawn. Keep the
                    // validation label current on editing frames as well.
                    parsed_exclusions = wax_core::split::parse_exclusions(&self.split_edit);
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
                // Reuse this frame's parse for the border and validation label.
                match parsed_exclusions {
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
        let open = std::mem::take(&mut self.open_network_settings);
        if measurements::settings_ui(ui, &mut self.settings.network, open) { self.save_settings(); }
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
                RichText::new("waxvpn.pages.dev").size(12.0).color(theme::ACCENT),
                "https://waxvpn.pages.dev/",
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
    resp.clone().on_hover_text(match tab {
        Tab::Connect => "Connect (Ctrl+1)", Tab::Profiles => "Profiles (Ctrl+2)",
        Tab::Features => "Features (Ctrl+3)", Tab::Warp => "WARP (Ctrl+4)",
        Tab::About => "About (Ctrl+5)", Tab::Network => "Network (Ctrl+6)",
    });
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
        Tab::Network => {
            p.line_segment([c + egui::vec2(-s, s), c + egui::vec2(-s, -s)], st);
            p.line_segment([c + egui::vec2(-s, s), c + egui::vec2(s, s)], st);
            p.add(egui::Shape::line(vec![c + egui::vec2(-s, 0.5*s), c + egui::vec2(-0.3*s, -0.3*s), c + egui::vec2(0.2*s, 0.2*s), c + egui::vec2(s, -s)], st));
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

// Local-only startup markers deliberately exclude settings, profile names,
// account data and panic payloads. Native driver faults can bypass Rust hooks.
