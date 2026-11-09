//! Gargantua-inspired procedural black hole, painted with plain egui shapes.
//!
//! A faithful port of the wax-vpn-site hero scene (same math, same palette):
//! a lensed starfield, an amber accretion disk with doppler brightening and
//! differential rotation, an event horizon, a photon ring, and the vertical
//! lensed image of the far side of the disk. No textures, no shaders.
//!
//! Motion is mood, not telemetry: rotation speed and brightness are fixed
//! per connection state and never derived from measured throughput, so the
//! animation never implies a speed or security level it has not measured.

use eframe::egui::{self, Color32, Painter, Pos2, Rect, Stroke, Vec2};

const TILT: f32 = 0.19;

struct Star {
    x: f32,
    y: f32,
    z: f32,
    r: f32,
    tw: f32,
}

struct DiskP {
    a0: f32,
    r: f32,
    w: f32,
    o: f32,
}

struct Spark {
    a0: f32,
    r: f32,
    sp: f32,
    f: f32,
    ph: f32,
    o: f32,
}

pub struct Gargantua {
    stars: Vec<Star>,
    disk: Vec<DiskP>,
    sparks: Vec<Spark>,
}

// Deterministic tiny PRNG (xorshift) so the scene is stable frame to frame
// without pulling in a rand dependency.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 40) as f32 / (1u64 << 24) as f32
    }
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.next() * (hi - lo)
    }
}

impl Default for Gargantua {
    fn default() -> Self {
        Self::new()
    }
}

impl Gargantua {
    pub fn new() -> Self {
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        let mut stars = Vec::with_capacity(240);
        for _ in 0..240 {
            stars.push(Star {
                x: rng.next(),
                y: rng.next(),
                z: rng.range(0.3, 1.0),
                r: rng.range(0.35, 1.55),
                tw: rng.range(0.0, 6.28),
            });
        }
        let mut disk = Vec::with_capacity(1100);
        for _ in 0..1100 {
            disk.push(DiskP {
                a0: rng.range(0.0, 6.283),
                r: rng.next().powf(1.6),
                w: rng.range(0.5, 1.8),
                o: rng.range(0.35, 1.0),
            });
        }
        let mut sparks = Vec::with_capacity(72);
        for _ in 0..72 {
            sparks.push(Spark {
                a0: rng.range(0.0, 6.283),
                r: rng.next(),
                sp: rng.range(0.8, 2.4),
                f: rng.range(0.5, 2.7),
                ph: rng.range(0.0, 6.283),
                o: rng.range(0.4, 1.0),
            });
        }
        Gargantua { stars, disk, sparks }
    }

    /// Starfield with gravitational lensing around (center, R), clipped to
    /// the painter's clip rect. `dim` scales overall brightness.
    pub fn paint_stars(&self, painter: &Painter, rect: Rect, center: Pos2, r_hole: f32, t: f32, dim: f32) {
        let lr = r_hole * 3.4;
        for st in &self.stars {
            let mut sx = rect.left() + st.x * rect.width();
            let mut sy = rect.top() + st.y * rect.height();
            let a = (0.2 + 0.55 * st.z * (0.6 + 0.4 * (t * 1.2 + st.tw).sin())) * dim;
            if a <= 0.02 {
                continue;
            }
            let dx = sx - center.x;
            let dy = sy - center.y;
            let d2 = dx * dx + dy * dy;
            if d2 < lr * lr && d2 > 1.0 {
                let d = d2.sqrt();
                let bend = (r_hole * r_hole * 0.9 / d).min(r_hole);
                let nx = sx + dx / d * bend;
                let ny = sy + dy / d * bend;
                if d < r_hole * 1.6 {
                    // Starlight smeared along the photon sphere.
                    let tx = -dy / d;
                    let ty = dx / d;
                    let sl = (1.6 - d / (r_hole * 1.6)) * 7.0 * st.z + 2.0;
                    painter.line_segment(
                        [
                            Pos2::new(nx - tx * sl, ny - ty * sl),
                            Pos2::new(nx + tx * sl, ny + ty * sl),
                        ],
                        Stroke::new(st.r, alpha(Color32::from_rgb(216, 228, 255), a * 1.35)),
                    );
                    continue;
                }
                sx = nx;
                sy = ny;
            }
            painter.circle_filled(Pos2::new(sx, sy), st.r, alpha(Color32::from_rgb(206, 222, 255), a));
        }
    }

    /// The hole itself: ambient glow, accretion disk (back then front half),
    /// event horizon, limb, photon ring, and the vertical lensed image.
    /// `phase` is integrated scene time (already speed-scaled by the caller),
    /// `t` is wall-clock seconds for flicker/pulse, `energy` is 0..1 mood.
    pub fn paint_hole(&self, painter: &Painter, center: Pos2, r_hole: f32, phase: f32, t: f32, energy: f32) {
        let dim = energy;
        // Ambient amber glow.
        radial_glow(painter, center, r_hole * 4.6, Color32::from_rgba_unmultiplied(255, 176, 100, (26.0 * dim) as u8));

        // Back half of the disk, dimmer (it is "behind" the hole).
        self.paint_disk_half(painter, center, r_hole, phase, t, dim * 0.55, true);

        // Event horizon.
        painter.circle_filled(center, r_hole * 1.005, Color32::from_rgb(1, 2, 4));

        // Limb: soft warm arc on the left inner edge of the shadow.
        ellipse_stroke(
            painter,
            center,
            r_hole * 0.94,
            r_hole * 0.94,
            std::f32::consts::PI * 0.45,
            std::f32::consts::PI * 1.55,
            Stroke::new(r_hole * 0.1, Color32::from_rgba_unmultiplied(255, 184, 105, (30.0 * dim) as u8)),
        );

        // Photon ring with a slow pulse and a faked bloom.
        let pulse = 0.8 + 0.18 * (t * 0.9).sin();
        let ring_a = 0.9 * pulse * dim;
        painter.circle_stroke(center, r_hole * 1.045, Stroke::new(7.0, alpha(Color32::from_rgb(255, 206, 138), ring_a * 0.10)));
        painter.circle_stroke(center, r_hole * 1.045, Stroke::new(3.2, alpha(Color32::from_rgb(255, 220, 160), ring_a * 0.28)));
        painter.circle_stroke(center, r_hole * 1.045, Stroke::new(1.3, alpha(Color32::from_rgb(255, 242, 224), ring_a)));

        // Vertical lensed image of the disk (the iconic top/bottom arcs).
        let rx = r_hole * 0.78;
        let ry = r_hole * 1.42;
        ellipse_stroke(painter, center, rx, ry, 0.0, std::f32::consts::TAU, Stroke::new(1.7, Color32::from_rgba_unmultiplied(255, 200, 140, (76.0 * dim) as u8)));
        ellipse_stroke(painter, center, rx, ry, std::f32::consts::PI * 1.18, std::f32::consts::PI * 1.82, Stroke::new(2.2, alpha(Color32::from_rgb(255, 244, 222), 0.8 * dim)));
        ellipse_stroke(painter, center, rx, ry, std::f32::consts::PI * 0.18, std::f32::consts::PI * 0.82, Stroke::new(2.2, alpha(Color32::from_rgb(255, 244, 222), 0.8 * dim)));
        ellipse_stroke(painter, center, r_hole * 0.83, r_hole * 1.5, 0.0, std::f32::consts::TAU, Stroke::new(r_hole * 0.05, Color32::from_rgba_unmultiplied(255, 196, 128, (25.0 * dim) as u8)));

        // Front half of the disk.
        self.paint_disk_half(painter, center, r_hole, phase, t, dim, false);

        // Inner rim of the disk: an elliptical hairline, brighter on the left.
        let rim = r_hole * 1.26;
        let steps = 48;
        for i in 0..steps {
            let a0 = i as f32 / steps as f32 * std::f32::consts::TAU;
            let a1 = (i + 1) as f32 / steps as f32 * std::f32::consts::TAU;
            let mid = (a0 + a1) * 0.5;
            // left bright -> right dim, matching the site's linear gradient
            let f = (mid.cos() * -0.5 + 0.5).clamp(0.0, 1.0); // 1 on the left
            let c = mix(
                Color32::from_rgb(255, 150, 80),
                Color32::from_rgb(255, 246, 228),
                0.28 + 0.62 * f,
            );
            let p0 = Pos2::new(center.x + a0.cos() * rim, center.y + a0.sin() * rim * TILT);
            let p1 = Pos2::new(center.x + a1.cos() * rim, center.y + a1.sin() * rim * TILT);
            painter.line_segment([p0, p1], Stroke::new(1.5, alpha(c, (0.28 + 0.62 * f) * dim)));
        }
    }

    fn paint_disk_half(&self, painter: &Painter, center: Pos2, r_hole: f32, phase: f32, t: f32, dim: f32, back: bool) {
        for p in &self.disk {
            // Differential rotation: inner material orbits faster.
            let speed = 0.33 * (1.26 / (1.26 + p.r * 1.9)).powf(1.5);
            let a = p.a0 + phase * speed;
            let sa = a.sin();
            if back != (sa < 0.0) {
                continue;
            }
            let r = r_hole * (1.26 + p.r * 3.14);
            let x = center.x + a.cos() * r;
            let y = center.y + sa * r * TILT;
            let k = p.r;
            let dop = -a.cos(); // approaching side is brighter
            let bright = p.o * (1.0 + 0.62 * dop) * (1.0 - 0.25 * k) * dim;
            if bright <= 0.03 {
                continue;
            }
            let g = 236.0 - 120.0 * k - 44.0 * (-dop).max(0.0);
            let b = 208.0 - 156.0 * k - 100.0 * (-dop).max(0.0);
            let da = 0.009 * (1.0 - 0.3 * k);
            let x2 = center.x + (a - da).cos() * r;
            let y2 = center.y + (a - da).sin() * r * TILT;
            painter.line_segment(
                [Pos2::new(x2, y2), Pos2::new(x, y)],
                Stroke::new(
                    p.w * (1.5 - 0.6 * k),
                    Color32::from_rgba_unmultiplied(255, g as u8, b as u8, (bright.min(1.0) * 0.8 * 255.0) as u8),
                ),
            );
        }
        for s in &self.sparks {
            let speed = 1.8 * s.sp * (1.26 / (1.26 + s.r * 0.5)).powf(1.5);
            let a = s.a0 + phase * speed;
            let sa = a.sin();
            if back != (sa < 0.0) {
                continue;
            }
            let rr = r_hole * (1.27 + s.r * 0.55);
            let sx = center.x + a.cos() * rr;
            let sy = center.y + sa * rr * TILT;
            let fl = 0.5 + 0.5 * (t * s.f * 3.0 + s.ph).sin();
            let br = s.o * fl * (1.0 + 0.5 * (-a.cos())) * dim;
            if br <= 0.04 {
                continue;
            }
            let dl = 0.02;
            let qx = center.x + (a - dl).cos() * rr;
            let qy = center.y + (a - dl).sin() * rr * TILT;
            painter.line_segment(
                [Pos2::new(qx, qy), Pos2::new(sx, sy)],
                Stroke::new(1.1, Color32::from_rgba_unmultiplied(255, (240.0 - 60.0 * s.r) as u8, (214.0 - 100.0 * s.r) as u8, (br.min(1.0) * 0.9 * 255.0) as u8)),
            );
        }
    }
}

fn alpha(c: Color32, a: f32) -> Color32 {
    let a = (a.clamp(0.0, 1.0) * 255.0) as u8;
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

/// Soft radial glow via a triangle-fan mesh (egui has no gradient primitive).
fn radial_glow(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
    let n = 40;
    let mut mesh = egui::Mesh::default();
    let edge = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 0);
    mesh.colored_vertex(center, color);
    for i in 0..=n {
        let a = i as f32 / n as f32 * std::f32::consts::TAU;
        mesh.colored_vertex(center + Vec2::new(a.cos() * radius, a.sin() * radius), edge);
    }
    for i in 0..n {
        mesh.add_triangle(0, 1 + i as u32, 2 + i as u32);
    }
    painter.add(egui::Shape::mesh(mesh));
}

fn ellipse_stroke(painter: &Painter, center: Pos2, rx: f32, ry: f32, a0: f32, a1: f32, stroke: Stroke) {
    let steps = 64;
    let pts: Vec<Pos2> = (0..=steps)
        .map(|i| {
            let a = a0 + (a1 - a0) * i as f32 / steps as f32;
            Pos2::new(center.x + a.cos() * rx, center.y + a.sin() * ry)
        })
        .collect();
    painter.add(egui::Shape::line(pts, stroke));
}
