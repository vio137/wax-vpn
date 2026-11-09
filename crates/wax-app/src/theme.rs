//! Design tokens and shared widgets for the waX VPN interface.
//!
//! Direction (unslop pass, CC BY 4.0 rubric by Damir Sokolovsky): calm and
//! editorial. Deep-ink chrome, one warm amber accent used sparingly, hairline
//! dividers instead of card boxes, Inter type with a real hierarchy, generous
//! intentional spacing. Motion is short, eased, and only ever
//! color/surface/opacity, never layout.

use eframe::egui::{self, Color32, Pos2, Sense, Stroke, Vec2};

pub const BG: Color32 = Color32::from_rgb(11, 13, 18);
pub const PANEL: Color32 = Color32::from_rgb(14, 17, 24);
#[allow(dead_code)]
pub const CARD: Color32 = Color32::from_rgb(18, 22, 31);
#[allow(dead_code)]
pub const CARD_HOVER: Color32 = Color32::from_rgb(23, 28, 39);
pub const HAIRLINE: Color32 = Color32::from_rgba_premultiplied(14, 14, 14, 14);
pub const TEXT: Color32 = Color32::from_rgb(230, 233, 240);
pub const SUB: Color32 = Color32::from_rgb(154, 163, 178);
pub const MUTE: Color32 = Color32::from_rgb(110, 118, 132);
pub const ACCENT: Color32 = Color32::from_rgb(242, 176, 102);
pub const GOOD: Color32 = Color32::from_rgb(88, 214, 141);
pub const WARN: Color32 = Color32::from_rgb(251, 191, 36);
pub const BAD: Color32 = Color32::from_rgb(248, 113, 113);

pub fn alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

/// SemiBold Inter for headings and emphasis.
pub fn semi(size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Name("semibold".into()))
}

/// 1px full-width hairline divider.
pub fn hairline(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().line_segment(
        [rect.left_center(), rect.right_center()],
        Stroke::new(1.0, HAIRLINE),
    );
}

/// Soft radial glow via a triangle-fan mesh (egui has no gradient primitive).

/// Card frame retained for floating surfaces (pickers, dialogs) only.
pub fn card_frame() -> egui::Frame {
    egui::Frame::default()
        .fill(alpha(Color32::WHITE, 5))
        .stroke(Stroke::new(1.0, HAIRLINE))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(14, 10))
}

/// Fade-in when a tab first shows; 1.0 instantly under reduced motion.
pub fn enter(ui: &egui::Ui, id: egui::Id, reduced_motion: bool) -> f32 {
    if reduced_motion {
        1.0
    } else {
        ui.ctx().animate_bool(id, true)
    }
}

pub fn toggle(ui: &mut egui::Ui, on: &mut bool, label: &str) -> bool {
    let (rect, mut resp) = ui.allocate_exact_size(Vec2::new(38.0, 22.0), Sense::click());
    let t = ui.ctx().animate_bool(resp.id, *on);
    let track = Color32::from_rgb(46, 52, 68).lerp_to_gamma(ACCENT, t);
    ui.painter().rect_filled(rect, egui::CornerRadius::same(11), Color32::from(track));
    let cx = rect.left() + 11.0 + (rect.width() - 22.0) * t;
    ui.painter()
        .circle_filled(Pos2::new(cx, rect.center().y), 8.0, Color32::from_rgb(244, 246, 250));
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let changed = resp.clicked();
    if changed {
        *on = !*on;
        resp.mark_changed();
    }
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::CheckBox, ui.is_enabled(), *on, label));
    if resp.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(3.0),
            egui::CornerRadius::same(14),
            Stroke::new(1.0, ACCENT),
            egui::StrokeKind::Outside,
        );
    }
    changed
}

/// Settings row: title + description left, control right, hairline below.
pub fn toggle_row(ui: &mut egui::Ui, on: &mut bool, title: &str, desc: &str) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(title).size(13.0).color(TEXT));
            ui.label(egui::RichText::new(desc).size(11.0).color(MUTE));
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            changed = toggle(ui, on, title);
        });
    });
    ui.add_space(6.0);
    hairline(ui);
    ui.add_space(6.0);
    changed
}

pub fn section_heading(ui: &mut egui::Ui, title: &str, sub: &str) {
    ui.label(egui::RichText::new(title).font(semi(20.0)).color(TEXT));
    ui.add_space(2.0);
    ui.label(egui::RichText::new(sub).size(12.0).color(SUB));
    ui.add_space(12.0);
}

/// The one confident action style. Real egui button: accessible name from text.
pub fn primary_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> bool {
    primary_button_named(ui, label, enabled, label)
}

pub fn primary_button_named(ui: &mut egui::Ui, label: &str, enabled: bool, a11y_name: &str) -> bool {
    let btn = egui::Button::new(
        egui::RichText::new(label)
            .font(semi(14.0))
            .color(if enabled { BG } else { MUTE }),
    )
    .fill(if enabled { ACCENT } else { alpha(ACCENT, 36) })
    .corner_radius(egui::CornerRadius::same(12))
    .min_size(Vec2::new(ui.available_width(), 46.0));
    let resp = ui.add_enabled(enabled, btn);
    let a11y = a11y_name.to_string();
    resp.widget_info(move || egui::WidgetInfo::labeled(egui::Role::Button, enabled, a11y.clone()));
    resp.clicked()
}

pub fn danger_button(ui: &mut egui::Ui, label: &str) -> bool {
    let btn = egui::Button::new(egui::RichText::new(label).size(11.5).color(BAD))
        .fill(alpha(BAD, 14))
        .stroke(Stroke::new(1.0, alpha(BAD, 60)))
        .corner_radius(egui::CornerRadius::same(8))
        .min_size(Vec2::new(0.0, 26.0));
    ui.add(btn).clicked()
}
