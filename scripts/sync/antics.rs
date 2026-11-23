/// Mathéo's own antics, put over the physics taken from Ronnie (Ronnie's Mathéo has none): now and
/// then a parachute when he falls from high, dust where he lands and slides, squashed and wobbling
/// as he lands, his eyes wide in the air, leaning back as he slides.
#[derive(Default)]
struct Antics {
    /// Last frame: in the air, and how fast he fell.
    air: bool,
    vy: f32,
    /// This fall's draw for a parachute made (once a fall).
    drawn: bool,
    chute: Option<Chute>,
    /// Dust puffs: where, since when, drifting which way.
    puffs: Vec<(Pos2, Instant, f32)>,
    /// The last puff while sliding.
    slid: Option<Instant>,
    /// Landed: when, how hard (0 to 1).
    landed: Option<(Instant, f32)>,
    seed: u64,
}

#[derive(Clone, Copy)]
enum Chute {
    /// Opening since then, his falling speed then.
    Open(Instant, f32),
    /// Landed (or caught): falling beside him since then, from where, to which side.
    Folding(Instant, Pos2, f32),
}

/// Falling from that high (points above the floor): a parachute one time in `CHUTE_ODDS`.
const CHUTE_HIGH: f32 = 260.0;
const CHUTE_ODDS: u64 = 3;
/// Under his parachute: how fast he comes down (points a second), how long it takes to open, to fold.
const CHUTE_FALL: f32 = 90.0;
const CHUTE_OPENS: f32 = 0.35;
const CHUTE_FOLDS: f32 = 0.9;
/// How long a puff of dust, a landing's wobble last.
const PUFF_LASTS: f32 = 0.55;
const WOBBLE_LASTS: f32 = 0.6;

impl Antics {
    fn roll(&mut self) -> u64 {
        if self.seed == 0 {
            self.seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(7, |d| d.as_nanos() as u64) | 1;
        }
        // xorshift
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        self.seed
    }

    /// After the physics' step: the parachute opened, held or folded, the dust, the landing; `floor`
    /// where he lands. True while something of it moves.
    fn step(&mut self, plant: &mut Plant, floor: f32, dt: f32) -> bool {
        let Some(at) = plant.at else { return false };
        let moment = Instant::now();
        let held = plant.grab.is_some();
        let air = !held && at.y < floor - 0.5;
        let side = |vx: f32| if vx < 0.0 { -1.0 } else { 1.0 };
        if held {
            // Caught: his parachute drops away, a new fall draws again.
            if let Some(Chute::Open(..)) = self.chute {
                self.chute = Some(Chute::Folding(moment, at, side(plant.vx)));
            }
            self.drawn = false;
        }
        if air && plant.vy > 0.0 && !self.drawn && floor - at.y > CHUTE_HIGH {
            self.drawn = true;
            // Debug builds: `MATHEO_CHUTE=1`, each time.
            let forced = cfg!(debug_assertions) && std::env::var_os("MATHEO_CHUTE").is_some();
            if forced || self.roll().is_multiple_of(CHUTE_ODDS) {
                self.chute = Some(Chute::Open(moment, plant.vy));
            }
        }
        if air && let Some(Chute::Open(since, v0)) = self.chute {
            // Slowed down as it opens, then gently down, swaying, barely drifting.
            let age = since.elapsed().as_secs_f32();
            let open = ease_out(age / CHUTE_OPENS);
            let cap = CHUTE_FALL + (v0.max(CHUTE_FALL) - CHUTE_FALL) * (1.0 - open);
            plant.vy = plant.vy.min(cap);
            plant.vx *= (-1.8 * dt).exp();
            plant.spin = 0.0;
            let tau = std::f32::consts::TAU;
            let mut tilt = plant.tilt.rem_euclid(tau);
            if tilt > std::f32::consts::PI {
                tilt -= tau;
            }
            let sway = 0.16 * (age * 2.4).sin();
            plant.tilt = tilt + (sway - tilt) * (1.0 - (-6.0 * dt).exp());
        }
        // Down on the floor: squashed (wobbling), dust, the parachute folding beside him.
        if self.air && !air && !held {
            let soft = matches!(self.chute, Some(Chute::Open(..)));
            if self.vy > 350.0 || soft {
                self.landed = Some((moment, if soft { 0.35 } else { (self.vy / 2200.0).clamp(0.3, 1.0) }));
            }
            if self.vy > 700.0 || soft {
                let n = if soft { 4 } else { 3 + (self.vy / 900.0).min(3.0) as usize };
                for k in 0..n {
                    let way = if k % 2 == 0 { -1.0 } else { 1.0 };
                    let off = 6.0 + (self.roll() % 12) as f32;
                    let drift = way * (0.6 + (self.roll() % 10) as f32 / 10.0);
                    self.puffs.push((at + Vec2::new(way * off, -2.0), moment, drift));
                }
            }
            if soft {
                let way = if self.roll().is_multiple_of(2) { -1.0 } else { 1.0 };
                self.chute = Some(Chute::Folding(moment, at, way));
            }
            self.drawn = false;
        }
        // Sliding along the floor: dust behind him, leaning back.
        if !air && !held && plant.vx.abs() > 120.0 {
            if self.slid.is_none_or(|t| t.elapsed() >= Duration::from_millis(60)) {
                self.slid = Some(moment);
                let back = -side(plant.vx);
                self.puffs.push((at + Vec2::new(back * 10.0, -2.0), moment, back * 0.5));
            }
            let tau = std::f32::consts::TAU;
            let up = (plant.tilt / tau).round() * tau;
            if (plant.tilt - up).abs() < 0.5 {
                let lean = -(plant.vx / 1800.0).clamp(-0.35, 0.35);
                plant.tilt += (up + lean - plant.tilt) * (1.0 - (-12.0 * dt).exp());
            }
        }
        self.puffs.retain(|p| p.1.elapsed().as_secs_f32() < PUFF_LASTS);
        if self.chute.is_some_and(|c| matches!(c, Chute::Folding(t, ..) if t.elapsed().as_secs_f32() >= CHUTE_FOLDS)) {
            self.chute = None;
        }
        if self.landed.is_some_and(|(t, _)| t.elapsed().as_secs_f32() >= WOBBLE_LASTS) {
            self.landed = None;
        }
        self.air = air;
        self.vy = plant.vy;
        let acting = matches!(plant.tip, Some((Said::Own(Own::Breathe | Own::Posture | Own::Back), _)));
        acting || self.chute.is_some() || !self.puffs.is_empty() || self.landed.is_some()
    }

    /// His look with the antics: eyes wide in the air (but calm under his parachute), squashed and
    /// wobbling as he lands.
    fn dress(&self, mut look: Look, plant: &Plant) -> Look {
        let speed = Vec2::new(plant.vx, plant.vy).length();
        if self.air && !matches!(self.chute, Some(Chute::Open(..))) && speed > 400.0 && !look.dizzy {
            look.held = true;
        }
        // What he says of his own, acted: breathing with you (taller breathing in), sitting up
        // straight, hopping, glad, to see you back.
        let upright = look.tilt.abs() < 0.3 && self.landed.is_none();
        match plant.tip {
            Some((Said::Own(Own::Breathe), since)) if upright => {
                // Full of air: taller, wider at the top, his eyes closed with pleasure.
                if let Some((_, _, _, full)) = breath(since.elapsed().as_secs_f32()) {
                    let w = -0.2 * full;
                    look.squash = w;
                    look.hop -= 20.0 * w;
                    look.glad = full > 0.5;
                }
            }
            Some((Said::Own(Own::Posture), since)) if upright && since.elapsed().as_secs_f32() < 2.0 => {
                let w = -0.12 * (since.elapsed().as_secs_f32() / 2.0 * std::f32::consts::PI).sin();
                look.squash = w;
                look.hop -= 20.0 * w;
            }
            Some((Said::Own(Own::Back), since)) => {
                look.glad = true;
                let t = since.elapsed().as_secs_f32();
                if upright && t < 1.5 {
                    look.hop += (t * 9.0).sin().abs() * 9.0 * (1.0 - t / 1.5);
                }
            }
            _ => {}
        }
        if let Some((since, hard)) = self.landed {
            let w = wobble(since.elapsed().as_secs_f32(), hard);
            if look.tilt.abs() < 0.3 && w.abs() > look.squash.abs() {
                look.squash = w;
                // Squashed onto his foot, not around his middle.
                look.hop -= 20.0 * w;
            }
        }
        look
    }
}

/// 0 to 1, a little past 1 then back (an opening that overshoots).
fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0) - 1.0;
    let c = 1.70158;
    1.0 + (c + 1.0) * t * t * t + c * t * t
}

/// A landing's squash `age` seconds after, `hard` (0 to 1): flat at once, then stretched, squashed
/// again, less and less.
fn wobble(age: f32, hard: f32) -> f32 {
    if age >= WOBBLE_LASTS {
        return 0.0;
    }
    0.32 * hard * (-age * 7.0).exp() * (age * 22.0).cos()
}

/// What his antics put behind him (his parachute) and in front (the dust), Mathéo standing at
/// `foot` as `paint_matheo_as` draws him with `look`.
fn paint_antics_behind(painter: &egui::Painter, antics: &Antics, foot: Pos2, size: f32, look: Look, origin: Vec2) {
    let u = size / 70.0;
    match antics.chute {
        Some(Chute::Open(since, _)) => paint_chute(painter, foot, u, look, ease_out(since.elapsed().as_secs_f32() / CHUTE_OPENS)),
        Some(Chute::Folding(since, from, way)) => {
            let f = (since.elapsed().as_secs_f32() / CHUTE_FOLDS).min(1.0);
            // Falling to his side, tipping over, flattening, fading.
            let center = from - origin + Vec2::new(way * 46.0 * f, -98.0 + 92.0 * f * f) * u;
            let (sin, cos) = (way * 1.3 * f).sin_cos();
            let (w, h) = (1.0 + 0.25 * f, 1.0 - 0.8 * f);
            let map = |x: f32, y: f32| {
                let v = Vec2::new(x * w, y * h) * u;
                center + Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
            };
            paint_canopy(painter, &map, 1.0, ((1.0 - f.powf(1.5)) * 255.0) as u8, u);
        }
        None => {}
    }
}

fn paint_antics_front(painter: &egui::Painter, antics: &Antics, size: f32, origin: Vec2) {
    let u = size / 70.0;
    for &(at, since, way) in &antics.puffs {
        let f = since.elapsed().as_secs_f32() / PUFF_LASTS;
        let c = at - origin + Vec2::new(way * 16.0 * f, -9.0 * f) * u;
        let alpha = (120.0 * (1.0 - f)) as u8;
        painter.circle_filled(c, (3.0 + 6.0 * f) * u, Color32::from_rgba_unmultiplied(205, 195, 180, alpha));
    }
}

/// His parachute open (`open`, 0 to 1 and a little past), its strings tied to the pot's rim, in his
/// frame (tipped over with him).
fn paint_chute(painter: &egui::Painter, foot: Pos2, u: f32, look: Look, open: f32) {
    let foot = foot - Vec2::new(0.0, look.hop);
    let pivot = foot - Vec2::new(0.0, 20.0 * u);
    let (sin, cos) = look.tilt.sin_cos();
    let at = |x: f32, y: f32| {
        let v = Vec2::new(x, y + 20.0) * u;
        pivot + Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
    };
    let o = open.max(0.05);
    let base = -36.0 - 62.0 * o.min(1.0);
    let line = Stroke::new(0.9 * u, Color32::from_rgb(90, 80, 70));
    for (rim, edge) in [(-16.0, -1.0), (-6.0, -0.34), (6.0, 0.34), (16.0, 1.0)] {
        painter.line_segment([at(rim, -32.0), at(edge * 40.0 * o, base)], line);
    }
    let map = |x: f32, y: f32| at(x * o, base + y * o);
    paint_canopy(painter, &map, 1.0, 255, u);
}

/// A parachute's canopy, 80 wide and 28 high in `map`'s units (0, 0 the middle of its edge): red and
/// white gores, its edge scalloped.
fn paint_canopy(painter: &egui::Painter, map: &dyn Fn(f32, f32) -> Pos2, _open: f32, alpha: u8, u: f32) {
    let (r, h) = (40.0, 28.0);
    let pi = std::f32::consts::PI;
    let arc = |a: f32| (r * a.cos(), -h * a.sin());
    let colors = [(230, 72, 60), (250, 244, 232)];
    let gores = 6;
    for k in 0..gores {
        let (a0, a1) = (k as f32 / gores as f32 * pi, (k + 1) as f32 / gores as f32 * pi);
        let (c0, c1) = ((r * a0.cos()), (r * a1.cos()));
        let mut points: Vec<Pos2> = (0..=4).map(|i| { let (x, y) = arc(a0 + (a1 - a0) * i as f32 / 4.0); map(x, y) }).collect();
        points.push(map(c1, 0.0));
        points.push(map((c0 + c1) / 2.0, 4.0));
        points.push(map(c0, 0.0));
        let (cr, cg, cb) = colors[k % 2];
        painter.add(egui::Shape::convex_polygon(points, Color32::from_rgba_unmultiplied(cr, cg, cb, alpha), Stroke::NONE));
    }
    let dome: Vec<Pos2> = (0..=24).map(|i| { let (x, y) = arc(i as f32 / 24.0 * pi); map(x, y) }).collect();
    painter.add(egui::Shape::line(dome, Stroke::new(1.0 * u, Color32::from_rgba_unmultiplied(140, 40, 34, alpha))));
}

/// Debug builds, with `MATHEO_GALLERY=1`: his antics, alive, above his outfits.
fn paint_antics_gallery(painter: &egui::Painter, area: Rect, now: f64) {
    let step = 130.0;
    let left = area.center().x - step;
    let y = area.center().y - 330.0;
    painter.rect_filled(Rect::from_min_max(Pos2::new(left - step / 2.0, y - 150.0), Pos2::new(left + step * 2.5, y + 40.0)), 14.0, Color32::from_rgba_unmultiplied(30, 32, 40, 200));
    let base = Look::default();
    let t = (now % 2.0) as f32;
    let label = |foot: Pos2, text: &str| {
        painter.text(foot + Vec2::new(0.0, 22.0), egui::Align2::CENTER_CENTER, text, FontId::proportional(13.0), Color32::from_rgb(235, 235, 240));
    };
    // Under his parachute, swaying, opening again every four seconds.
    let foot = Pos2::new(left, y);
    let look = Look { tilt: 0.16 * (now as f32 * 2.4).sin(), ..base };
    paint_chute(painter, foot, 1.0, look, ease_out((now % 4.0) as f32 / CHUTE_OPENS));
    paint_matheo_as(painter, foot, MATHEO_H, now, look);
    label(foot, "Parachute");
    // Landing hard.
    let foot = Pos2::new(left + step, y);
    let w = wobble(t, 1.0);
    paint_matheo_as(painter, foot, MATHEO_H, now, Look { squash: w, hop: -20.0 * w, ..base });
    label(foot, "Atterrissage");
    // In the air.
    let foot = Pos2::new(left + step * 2.0, y);
    paint_matheo_as(painter, foot, MATHEO_H, now, Look { held: true, tilt: (now as f32 * 5.0) % std::f32::consts::TAU, ..base });
    label(foot, "En vol");
}

/// How he arrives at each start, at random: under his parachute from up high, or popping out of
/// the bottom of the screen with a bomp.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Entrance {
    Chute,
    Pop,
}

impl Entrance {
    /// This start's (debug builds: `MATHEO_ENTRANCE=chute|pop`).
    fn now() -> Entrance {
        if cfg!(debug_assertions)
            && let Ok(forced) = std::env::var("MATHEO_ENTRANCE")
        {
            return if forced.eq_ignore_ascii_case("pop") { Entrance::Pop } else { Entrance::Chute };
        }
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.subsec_nanos());
        if (nanos / 1000).is_multiple_of(2) { Entrance::Chute } else { Entrance::Pop }
    }
}

/// Popping out of the bottom of the screen: rising from `depth` points under his place, past it
/// (`POP_JUMP` points over), falling back, then squashed and wobbling (the bomp).
const POP_RISE: f32 = 0.42;
const POP_FALL: f32 = 0.26;
const POP_SETTLE: f32 = 0.5;
const POP_JUMP: f32 = 28.0;

/// Popping out, `t` seconds in: how far under his place he's drawn, how high off it (his `hop`),
/// how squashed; None once over.
fn pop_at(t: f32, depth: f32) -> Option<(f32, f32, f32)> {
    if t < 0.0 {
        return Some((depth, 0.0, 0.0));
    }
    if t < POP_RISE {
        let f = 1.0 - (1.0 - t / POP_RISE).powi(3);
        let y = -depth + (depth + POP_JUMP) * f;
        return Some(if y < 0.0 { (-y, 0.0, -0.12 * (1.0 - f)) } else { (0.0, y, 0.0) });
    }
    let t = t - POP_RISE;
    if t < POP_FALL {
        let g = t / POP_FALL;
        return Some((0.0, POP_JUMP * (1.0 - g * g), 0.0));
    }
    let t = t - POP_FALL;
    if t < POP_SETTLE {
        let g = t / POP_SETTLE;
        return Some((0.0, 0.0, 0.26 * (1.0 - g).powf(1.5) * (g * 2.5 * std::f32::consts::PI).cos()));
    }
    None
}

#[cfg(test)]
mod entrance_tests {
    use super::{POP_FALL, POP_JUMP, POP_RISE, POP_SETTLE, pop_at};

    #[test]
    fn he_pops_out_of_the_bottom_then_bomps() {
        let depth = 150.0;
        // Hidden under the screen at first.
        assert!(pop_at(0.0, depth).is_some_and(|(below, hop, _)| below >= depth - 0.1 && hop == 0.0));
        // Over his place at the top of his jump.
        assert!(pop_at(POP_RISE, depth).is_some_and(|(below, hop, _)| below == 0.0 && (hop - POP_JUMP).abs() < 0.1));
        // Down, squashed.
        assert!(pop_at(POP_RISE + POP_FALL + 0.001, depth).is_some_and(|(below, hop, squash)| below == 0.0 && hop.abs() < 0.5 && squash > 0.2));
        assert_eq!(pop_at(POP_RISE + POP_FALL + POP_SETTLE, depth), None);
        // Never below ground once out, never NaN.
        for k in 0..200 {
            let (below, hop, squash) = pop_at(k as f32 / 150.0, depth).unwrap_or_default();
            assert!(below >= 0.0 && hop >= 0.0 && squash.is_finite());
        }
    }
}
