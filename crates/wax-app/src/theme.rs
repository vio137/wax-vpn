//! Design tokens and shared widgets for the waX VPN interface.
//!
//! Direction (unslop pass, CC BY 4.0 rubric by Damir Sokolovsky): quiet dark
//! chrome in one tinted neutral ramp, a single cyan accent, glassy raised
//! surfaces, and the connect orb as the one rich moment. Microinteractions
//! are short, eased and only ever color/surface/opacity, never layout.

use eframe::egui::{self, Color32, Pos2, Sense, Stroke, Vec2};

pub const BG: Color32 = Color32::from_rgb(4, 6, 14);
pub const PANEL: Color32 = Color32::from_rgb(8, 12, 22);
#[allow(dead_code)]
pub const CARD: Color32 = Color32::from_rgb(17, 25, 46);
#[allow(dead_code)]
pub const CARD_HOVER: Color32 = Color32::from_rgb(22, 32, 58);
pub const BORDER: Color32 = Color32::from_rgb(34, 48, 79);
pub const TEXT: Color32 = Color32::from_rgb(226, 232, 240);
pub const SUB: Color32 = Color32::from_rgb(148, 163, 184);
pub const MUTE: Color32 = Color32::from_rgb(100, 116, 139);
pub const ACCENT: Color32 = Color32::from_rgb(56, 225, 255);
pub const ACCENT_HI: Color32 = Color32::from_rgb(96, 236, 255);
pub const GOOD: Color32 = Color32::from_rgb(52, 211, 153);
pub const WARN: Color32 = Color32::from_rgb(251, 191, 36);
pub const BAD: Color32 = Color32::from_rgb(248, 113, 113);

pub fn alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

fn lerp(a: Color32, b: Color32, t: f32) -> Color32 {
    let f = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t.clamp(0.0, 1.0)) as u8;
    Color32::from_rgb(f(a.r(), b.r()), f(a.g(), b.g()), f(a.b(), b.b()))
}

/// Glassy raised surface: translucent light fill, hairline light border,
/// soft neutral shadow. On the dark shell this reads as frosted glass
/// without a real backdrop blur (egui cannot do one).
pub fn card_frame() -> egui::Frame {
    egui::Frame::default()
        .fill(alpha(Color32::WHITE, 7))
        .stroke(Stroke::new(1.0, alpha(Color32::WHITE, 16)))
        .corner_radius(egui::CornerRadius::same(14))
        .inner_margin(egui::Margin::same(14))
        .shadow(egui::epaint::Shadow {
            offset: [0, 6],
            blur: 16,
            spread: 0,
            color: alpha(Color32::BLACK, 70),
        })
}

/// One-time fade-in progress (0 -> 1, cubic-out, ~220ms) for a freshly
/// shown region. Honours reduced motion by returning 1 immediately.
pub fn enter(ui: &egui::Ui, id: egui::Id, reduced_motion: bool) -> f32 {
    if reduced_motion {
        return 1.0;
    }
    let now = ui.input(|i| i.time);
    let start = ui.ctx().data_mut(|d| *d.get_temp_mut_or(id, now));
    let raw = ((now - start) / 0.22).clamp(0.0, 1.0) as f32;
    let t = 1.0 - (1.0 - raw).powi(3);
    if raw < 1.0 {
        ui.ctx().request_repaint();
    }
    t
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
        let hov = ui
            .ctx()
            .animate_bool_responsive(response.id.with("hov"), response.hovered());
        let track_off = Color32::from_rgb(30, 41, 66);
        let track_fill = lerp(track_off, ACCENT, how_on);
        let track = egui::epaint::RectShape::new(
            rect,
            egui::CornerRadius::same(10),
            track_fill,
            Stroke::new(1.0, lerp(BORDER, ACCENT, how_on.max(hov * 0.5))),
            egui::StrokeKind::Inside,
        );
        ui.painter().add(egui::Shape::Rect(track));
        let r = 7.0 + hov * 1.0;
        let x = rect.left() + 12.0 + (rect.width() - 24.0) * how_on;
        let c = Pos2::new(x, rect.center().y);
        if hov > 0.05 {
            ui.painter()
                .circle_stroke(c, r + 3.0, Stroke::new(1.0, alpha(TEXT, (40.0 * hov) as u8)));
        }
        ui.painter().circle(c, r, TEXT, Stroke::NONE);
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



pub fn section_heading(ui: &mut egui::Ui, title: &str, sub: &str) {
    ui.label(egui::RichText::new(title).size(17.0).color(TEXT).strong());
    if !sub.is_empty() {
        ui.label(egui::RichText::new(sub).size(11.5).color(MUTE));
    }
    ui.add_space(6.0);
}

/// Primary action button: accent fill, hover lift, press settle, all eased.
/// Returns true when clicked while enabled.
pub fn primary_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> bool {
    let desired = Vec2::new(180.0, 36.0);
    let (rect, response) = ui.allocate_exact_size(desired, if enabled { Sense::click() } else { Sense::hover() });
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, label));
    let clicked = enabled && response.clicked();
    if ui.is_rect_visible(rect) {
        let live = enabled && response.hovered();
        let hov = ui
            .ctx()
            .animate_bool_responsive(response.id.with("hov"), live);
        let press = ui.ctx().animate_bool_responsive(
            response.id.with("press"),
            enabled && response.is_pointer_button_down_on(),
        );
        let mut fill = lerp(ACCENT, ACCENT_HI, hov * 0.9);
        fill = lerp(fill, lerp(ACCENT, Color32::BLACK, 0.25), press * 0.6);
        if !enabled {
            fill = alpha(ACCENT, 60);
        }
        let lift = hov * (1.0 - press) * 1.5;
        let r = rect.translate(egui::vec2(0.0, -lift));
        if hov > 0.05 {
            ui.painter().add(egui::epaint::RectShape::new(
                r.translate(egui::vec2(0.0, 3.0)),
                egui::CornerRadius::same(10),
                Color32::TRANSPARENT,
                Stroke::new(6.0, alpha(Color32::BLACK, (60.0 * hov) as u8)),
                egui::StrokeKind::Outside,
            ));
        }
        ui.painter().rect(
            r,
            egui::CornerRadius::same(10),
            fill,
            Stroke::NONE,
            egui::StrokeKind::Inside,
        );
        ui.painter().text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(14.0),
            if enabled {
                Color32::from_rgb(4, 10, 18)
            } else {
                alpha(Color32::from_rgb(4, 10, 18), 160)
            },
        );
    }
    clicked
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

/// SemiBold Inter for headings and the in-void state word.
pub fn semi(size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Name("semibold".into()))
}
