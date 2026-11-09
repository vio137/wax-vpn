//! Design tokens and shared widgets for the waX VPN interface.

use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};

pub const BG: Color32 = Color32::from_rgb(8, 12, 24);
pub const PANEL: Color32 = Color32::from_rgb(12, 18, 34);
pub const CARD: Color32 = Color32::from_rgb(17, 25, 46);
pub const CARD_HOVER: Color32 = Color32::from_rgb(22, 32, 58);
pub const BORDER: Color32 = Color32::from_rgb(34, 48, 79);
pub const TEXT: Color32 = Color32::from_rgb(226, 232, 240);
pub const SUB: Color32 = Color32::from_rgb(148, 163, 184);
pub const MUTE: Color32 = Color32::from_rgb(100, 116, 139);
pub const ACCENT: Color32 = Color32::from_rgb(56, 225, 255);
pub const GOOD: Color32 = Color32::from_rgb(52, 211, 153);
pub const WARN: Color32 = Color32::from_rgb(251, 191, 36);
pub const BAD: Color32 = Color32::from_rgb(248, 113, 113);

pub fn alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

pub fn card_frame() -> egui::Frame {
    egui::Frame::default()
        .fill(CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::same(12))
}

/// A toggle switch. Returns true when it was clicked.
pub fn toggle(ui: &mut egui::Ui, on: &mut bool) -> bool {
    let size = Vec2::new(38.0, 20.0);
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let track = egui::epaint::RectShape::new(
            rect,
            egui::CornerRadius::same(10),
            CARD.blend(if *on { ACCENT } else { egui::Color32::from_rgb(30, 41, 66) }),
            Stroke::new(1.0, if *on { ACCENT } else { BORDER }),
            egui::StrokeKind::Inside,
        );
        ui.painter().add(egui::Shape::Rect(track));
        let r = 7.0;
        let x = rect.left() + 12.0 + (rect.width() - 24.0) * how_on;
        ui.painter()
            .circle(Pos2::new(x, rect.center().y), r, TEXT, Stroke::NONE);
    }
    response.clicked()
}

/// One labelled toggle row used on the Features tab.
pub fn toggle_row(ui: &mut egui::Ui, on: &mut bool, title: &str, desc: &str) -> bool {
    let mut changed = false;
    card_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(title).size(13.5).color(TEXT).strong());
                ui.label(egui::RichText::new(desc).size(11.0).color(MUTE));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                changed = toggle(ui, on);
            });
        });
    });
    changed
}

/// Sidebar navigation button.
pub fn nav_button(ui: &mut egui::Ui, label: &str, selected: bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 34.0),
        Sense::click(),
    );
    if ui.is_rect_visible(rect) {
        let hovered = response.hovered();
        let fill = if selected {
            alpha(ACCENT, 26)
        } else if hovered {
            CARD_HOVER
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().rect(
            rect,
            egui::CornerRadius::same(8),
            fill,
            Stroke::NONE,
            egui::StrokeKind::Inside,
        );
        if selected {
            let bar = Rect::from_min_size(rect.min, Vec2::new(3.0, rect.height()));
            ui.painter().rect_filled(bar, egui::CornerRadius::same(2), ACCENT);
        }
        ui.painter().text(
            rect.left_center() + egui::vec2(14.0, 0.0),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(13.5),
            if selected { ACCENT } else { SUB },
        );
    }
    response.clicked()
}

/// Small stat tile used under the orb.
pub fn stat_tile(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    card_frame().show(ui, |ui| {
        ui.set_min_width(104.0);
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(label).size(10.5).color(MUTE));
            ui.label(egui::RichText::new(value).size(14.5).color(color).strong());
        });
    });
}

pub fn section_heading(ui: &mut egui::Ui, title: &str, sub: &str) {
    ui.label(egui::RichText::new(title).size(17.0).color(TEXT).strong());
    if !sub.is_empty() {
        ui.label(egui::RichText::new(sub).size(11.5).color(MUTE));
    }
    ui.add_space(6.0);
}

/// Primary action button with the brand accent.
pub fn primary_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> bool {
    ui.add_enabled(
        enabled,
        egui::Button::new(egui::RichText::new(label).size(13.5).color(Color32::from_rgb(4, 10, 18)).strong())
            .fill(ACCENT)
            .corner_radius(egui::CornerRadius::same(8))
            .min_size(Vec2::new(180.0, 34.0)),
    )
    .clicked()
}

pub fn danger_button(ui: &mut egui::Ui, label: &str) -> bool {
    ui.add(
        egui::Button::new(egui::RichText::new(label).size(12.5).color(BAD))
            .fill(alpha(BAD, 18))
            .stroke(Stroke::new(1.0, alpha(BAD, 90)))
            .corner_radius(egui::CornerRadius::same(8))
            .min_size(Vec2::new(0.0, 26.0)),
    )
    .clicked()
}
