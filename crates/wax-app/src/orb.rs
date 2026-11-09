//! The animated connect orb: the signature visual of waX VPN.
//! Pure egui painting - no textures, GPU-friendly, honours reduced motion.

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Sense, Shape, Stroke};

use crate::ConnState;

pub const ACCENT: Color32 = Color32::from_rgb(56, 225, 255);
pub const GOOD: Color32 = Color32::from_rgb(52, 211, 153);
pub const WARN: Color32 = Color32::from_rgb(251, 191, 36);
pub const BAD: Color32 = Color32::from_rgb(248, 113, 113);
pub const DIM: Color32 = Color32::from_rgb(71, 85, 105);
pub const BG: Color32 = Color32::from_rgb(10, 14, 26);

fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

fn arc_points(center: Pos2, radius: f32, start: f32, sweep: f32, segments: usize) -> Vec<Pos2> {
    (0..=segments)
        .map(|i| {
            let t = start + sweep * (i as f32 / segments as f32);
            Pos2::new(center.x + radius * t.cos(), center.y + radius * t.sin())
        })
        .collect()
}

fn status_word(state: &ConnState) -> (&'static str, Color32) {
    match state {
        ConnState::Disconnected => ("CONNECT", Color32::from_rgb(226, 232, 240)),
        ConnState::Connecting => ("CONNECTING", WARN),
        ConnState::Connected { .. } => ("SECURE", GOOD),
        ConnState::Disconnecting => ("CLOSING", WARN),
        ConnState::Failed(_) => ("RETRY", BAD),
    }
}

/// Returns the response so the caller can treat a click as connect/disconnect.
pub fn paint_orb(
    ui: &mut egui::Ui,
    state: &ConnState,
    t: f32,
    reduced_motion: bool,
) -> egui::Response {
    let size = egui::vec2(ui.available_width().min(320.0), 260.0);
    let (response, painter) = ui.allocate_painter(size, Sense::click());
    let center = response.rect.center();
    let base_r = 92.0;
    let (word, word_color) = status_word(state);

    let anim_t = if reduced_motion { 0.0 } else { t };

    // Halo layers
    let (halo_color, activity) = match state {
        ConnState::Disconnected => (DIM, 0.12),
        ConnState::Connecting => (WARN, 0.85),
        ConnState::Connected { .. } => (ACCENT, 1.0),
        ConnState::Disconnecting => (WARN, 0.5),
        ConnState::Failed(_) => (BAD, 0.4),
    };
    for i in 0..4u8 {
        let wobble = if reduced_motion { 0.0 } else { (anim_t * 1.3 + i as f32).sin() * 3.0 };
        let r = base_r + 10.0 + i as f32 * 9.0 + wobble;
        let alpha = (activity * (40.0 - i as f32 * 8.0)).max(6.0) as u8;
        painter.circle_stroke(center, r, Stroke::new(1.5, with_alpha(halo_color, alpha)));
    }

    // Base ring
    painter.circle_stroke(center, base_r, Stroke::new(2.5, with_alpha(halo_color, 160)));

    // Rotating arc sweeps
    if !reduced_motion {
        let (sweep, speed) = match state {
            ConnState::Connecting => (std::f32::consts::TAU * 0.28, 3.4),
            ConnState::Connected { .. } => (std::f32::consts::TAU * 0.62, 0.55),
            ConnState::Disconnecting => (std::f32::consts::TAU * 0.20, 2.2),
            _ => (std::f32::consts::TAU * 0.16, 0.5),
        };
        let start = anim_t * speed;
        painter.add(Shape::line(
            arc_points(center, base_r, start, sweep, 72),
            Stroke::new(4.0, with_alpha(halo_color, 230)),
        ));
        painter.add(Shape::line(
            arc_points(center, base_r, -start * 0.7 + 2.1, sweep * 0.5, 48),
            Stroke::new(2.0, with_alpha(halo_color, 110)),
        ));
    }

    // Orbiting particles while connected
    if matches!(state, ConnState::Connected { .. }) && !reduced_motion {
        for i in 0..42usize {
            let fi = i as f32;
            let angle = fi * 2.399_963 + anim_t * (0.25 + (i % 5) as f32 * 0.05);
            let radius = base_r + 14.0 + (fi * 7.0 % 26.0) + (anim_t * 0.9 + fi).sin() * 4.0;
            let pos = Pos2::new(center.x + radius * angle.cos(), center.y + radius * angle.sin());
            let alpha = (110.0 + 120.0 * (anim_t * 1.7 + fi * 1.31).sin()).clamp(0.0, 235.0) as u8;
            let dot = 1.2 + (i % 3) as f32 * 0.8;
            painter.circle_filled(pos, dot, with_alpha(if i % 4 == 0 { GOOD } else { ACCENT }, alpha));
        }
    }

    // Inner disc + label
    let breathe = if reduced_motion { 0.0 } else { (anim_t * 2.0).sin() * 2.0 };
    let disc_r = base_r - 14.0 + breathe;
    painter.circle_filled(center, disc_r, Color32::from_rgb(15, 22, 40));
    painter.circle_stroke(center, disc_r, Stroke::new(1.0, with_alpha(halo_color, 60)));

    painter.text(
        Pos2::new(center.x, center.y - 6.0),
        Align2::CENTER_CENTER,
        word,
        FontId::proportional(24.0),
        word_color,
    );
    let sub = match state {
        ConnState::Disconnected => "tap to secure your traffic",
        ConnState::Connecting => "negotiating tunnel",
        ConnState::Connected { .. } => "tap to disconnect",
        ConnState::Disconnecting => "restoring network",
        ConnState::Failed(_) => "something needs attention",
    };
    painter.text(
        Pos2::new(center.x, center.y + 22.0),
        Align2::CENTER_CENTER,
        sub,
        FontId::proportional(12.0),
        Color32::from_rgb(148, 163, 184),
    );

    if response.hovered() && !reduced_motion {
        painter.circle_stroke(center, base_r + 4.0, Stroke::new(1.0, with_alpha(halo_color, 90)));
    }
    response
}
