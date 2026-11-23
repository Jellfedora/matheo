// Which plant he is (Ronnie's Mathéo has only his leaves): drawn by `paint_matheo_as` instead of his
// leaves and his flower, on the same pot with the same face. Paloma, a cactus; Hélio, a sunflower.

/// `c` towards `to` as he wilts (`wilt`, 0 to 1), as his leaves yellow.
fn wilted(c: Color32, to: (u8, u8, u8), wilt: f32) -> Color32 {
    let w = wilt.clamp(0.0, 1.0) * 0.8;
    let m = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * w) as u8;
    Color32::from_rgb(m(c.r(), to.0), m(c.g(), to.1), m(c.b(), to.2))
}

/// A quadratic Bézier curve from `a` to `c`, pulled towards `b`, in `n` steps.
fn bezier(a: Vec2, b: Vec2, c: Vec2, n: usize) -> Vec<Vec2> {
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let m = 1.0 - t;
            a * m * m + b * 2.0 * m * t + c * t * t
        })
        .collect()
}

/// Drops shining on him, watered.
fn drop_at(painter: &egui::Painter, p: Pos2, u: f32) {
    painter.circle_filled(p, 1.9 * u, Color32::from_rgba_unmultiplied(150, 205, 255, 220));
    painter.circle_filled(p - Vec2::new(0.6, 0.6) * u, 0.7 * u, Color32::WHITE);
}

/// What grows from his pot when he isn't Mathéo; `at` places a point of his as `paint_matheo_as`
/// does (his units, the soil at y = -32).
fn paint_skin(painter: &egui::Painter, look: Look, now: f64, u: f32, at: &dyn Fn(f32, f32) -> Pos2) {
    let speed = if look.held { 9.0 } else if look.tired { 0.6 } else { 1.3 };
    let droop = look.wilt.max(if look.tired { 0.3 } else { 0.0 });
    // From the soil, upwards.
    let up = |v: Vec2| at(v.x, -32.0 - v.y);
    match look.skin {
        Skin::Classic => {}
        Skin::Cactus => paint_cactus(painter, look, now, u, &up, speed, droop),
        Skin::Sunflower => paint_sunflower(painter, look, now, u, &up, speed, droop),
    }
}

/// Paloma: a round-topped cactus, ribbed and spiny, two arms up (hanging when thirsty, flapping when
/// held, one waving); grown, taller, a third arm, a little one at his foot; a pink flower on top on a day well watered.
fn paint_cactus(painter: &egui::Painter, look: Look, now: f64, u: f32, up: &dyn Fn(Vec2) -> Pos2, speed: f64, droop: f32) {
    let t = now as f32;
    let w = 10.0f32;
    let h = (40.0 + 4.0 * look.extra.min(3) as f32) * (1.0 - 0.45 * look.hide) * (1.0 - 0.1 * look.wilt);
    // Leaning around his foot: swaying a little, more when held, over to one side when thirsty.
    let lean = (now * speed).sin() as f32 * if look.held { 0.16 } else { 0.03 } + droop * 0.22;
    let (sn, cs) = lean.sin_cos();
    let p = |x: f32, y: f32| up(Vec2::new(x * cs + y * sn, -x * sn + y * cs));
    let green = wilted(Color32::from_rgb(86, 158, 92), (176, 164, 84), look.wilt);
    let dark = wilted(Color32::from_rgb(38, 96, 56), (118, 100, 46), look.wilt);
    let light = wilted(Color32::from_rgb(132, 196, 124), (206, 192, 118), look.wilt);
    let spine = Color32::from_rgb(250, 244, 214);
    let tick = |x: f32, y: f32| {
        let c = p(x, y);
        painter.line_segment([c, p(x - 1.5, y + 1.3)], Stroke::new(0.7 * u, spine));
        painter.line_segment([c, p(x + 1.5, y + 1.3)], Stroke::new(0.7 * u, spine));
    };
    // Grown for long: a little one at his foot, upright.
    if look.extra >= 3 {
        let c = up(Vec2::new(14.0, 4.0));
        painter.circle_filled(c, 5.0 * u, dark);
        painter.circle_filled(c, 4.2 * u, green);
        painter.line_segment([up(Vec2::new(14.0, 1.0)), up(Vec2::new(14.0, 7.5))], Stroke::new(0.7 * u, light));
    }
    // His arms: out from his sides, then up.
    let flap = if look.held { (t * 12.0).sin() * 4.0 } else { 0.0 };
    let mut arms = vec![(-1.0f32, 0.42f32, 13.0f32), (1.0, 0.6, 10.0)];
    if look.extra >= 2 {
        arms.push((1.0, 0.3, 7.0));
    }
    let arms: Vec<Vec<Pos2>> = arms
        .iter()
        .enumerate()
        .map(|(k, &(s, high, len))| {
            let y = h * high;
            let mut rise = len * (1.0 - 1.7 * droop) + flap * s;
            let mut out = w + 7.0;
            if k < 2 && look.wave != 0.0 && s == look.wave.signum() {
                rise = len + 6.0;
                out += (t * 12.0).sin() * 3.0;
            }
            bezier(Vec2::new(s * (w - 3.0), y), Vec2::new(s * out, y - 1.0), Vec2::new(s * out, y + rise), 10).into_iter().map(|v| p(v.x, v.y)).collect()
        })
        .collect();
    // Edges first, then the fills over them: where an arm meets him, no edge.
    for arm in &arms {
        painter.add(egui::Shape::line(arm.clone(), Stroke::new(8.6 * u, dark)));
        painter.circle_filled(arm[arm.len() - 1], 4.3 * u, dark);
    }
    painter.line_segment([p(0.0, 1.0), p(0.0, h - w)], Stroke::new((2.0 * w + 1.6) * u, dark));
    painter.circle_filled(p(0.0, h - w), (w + 0.8) * u, dark);
    for arm in &arms {
        painter.add(egui::Shape::line(arm.clone(), Stroke::new(7.0 * u, green)));
        painter.circle_filled(arm[arm.len() - 1], 3.5 * u, green);
    }
    painter.line_segment([p(0.0, 1.0), p(0.0, h - w)], Stroke::new(2.0 * w * u, green));
    painter.circle_filled(p(0.0, h - w), w * u, green);
    // His ribs, his spines between them (and at his arms' tips).
    for x in [-5.0f32, 0.0, 5.0] {
        let top = h - w + (w * w - x * x).sqrt() - 2.5;
        painter.line_segment([p(x, 2.0), p(x, top)], Stroke::new(0.9 * u, light));
    }
    for x in [-7.5f32, -2.5, 2.5, 7.5] {
        let top = h - w + (w * w - x * x).sqrt() - 2.0;
        let mut y = if x.abs() > 5.0 { 8.0 } else { 4.5 };
        while y < top {
            tick(x, y);
            y += 7.0;
        }
    }
    for arm in &arms {
        let end = arm[arm.len() - 1];
        for d in [Vec2::new(-1.6, -1.6), Vec2::new(1.6, -1.6), Vec2::new(0.0, -2.2)] {
            painter.line_segment([end + d * 1.2 * u, end + d * 2.0 * u], Stroke::new(0.7 * u, spine));
        }
    }
    if look.glad {
        drop_at(painter, p(-4.0, h * 0.5), u);
        drop_at(painter, p(5.0, h * 0.78), u);
    }
    // Watered well today: a pink flower on top.
    if look.flower {
        let c = Vec2::new(0.0, h + 0.5);
        let petal = wilted(Color32::from_rgb(250, 110, 150), (196, 150, 130), look.wilt);
        for i in 0..5 {
            let a = i as f32 / 5.0 * std::f32::consts::TAU + t * 0.15;
            let q = c + Vec2::new(a.cos() * 3.4, a.sin() * 2.4 + 1.2);
            painter.circle_filled(p(q.x, q.y), 2.9 * u, petal);
            painter.circle_stroke(p(q.x, q.y), 2.9 * u, Stroke::new(0.6 * u, Color32::from_rgb(200, 60, 100)));
        }
        painter.circle_filled(p(c.x, c.y + 1.2), 1.9 * u, Color32::from_rgb(255, 214, 70));
    }
}

/// Hélio: a sunflower on a tall stem, his leaves along it, his face turning to the pointer near
/// him (as Mathéo's eyes follow it), his head hanging when thirsty; grown, more leaves and taller,
/// and a little sunflower beside him on a day well watered.
fn paint_sunflower(painter: &egui::Painter, look: Look, now: f64, u: f32, up: &dyn Fn(Vec2) -> Pos2, speed: f64, droop: f32) {
    let t = now as f32;
    let stem_color = wilted(Color32::from_rgb(74, 140, 52), (130, 110, 50), look.wilt);
    let stem = |base: Vec2, h: f32, bow: f32| -> Vec<Vec2> {
        let top = base + Vec2::new(bow.sin() * h, bow.cos() * h);
        bezier(base, base + Vec2::new(0.0, h * 0.55), top, 12)
    };
    let face = Vec2::new(look.gaze.x * 2.2, -look.gaze.y * 2.2 - droop * 2.5);
    // Watered well today: a little sunflower beside him, behind.
    if look.flower {
        let sway = (now * speed * 0.8 + 1.7).sin() as f32 * 0.05;
        let small = stem(Vec2::new(-2.0, 0.0), 30.0 * (1.0 - 0.6 * look.hide), -0.42 + sway - droop * 0.3);
        painter.add(egui::Shape::line(small.iter().map(|&v| up(v)).collect(), Stroke::new(1.8 * u, stem_color)));
        sunflower_head(painter, look, up, small[small.len() - 1], 0.55, face, u);
    }
    let h = (48.0 + 3.0 * look.extra.min(3) as f32) * (1.0 - 0.6 * look.hide) * (1.0 - 0.1 * look.wilt);
    let sway = (now * speed * 0.8).sin() as f32 * if look.held { 0.2 } else { 0.04 };
    let main = stem(Vec2::ZERO, h, sway + droop * 0.5);
    painter.add(egui::Shape::line(main.iter().map(|&v| up(v)).collect(), Stroke::new(2.6 * u, stem_color)));
    // His leaves along the stem, one on each side, more as he grows.
    let leaves = [(0.32f32, -1.0f32), (0.5, 1.0), (0.62, -1.0), (0.22, 1.0), (0.72, 1.0)];
    let green = [Color32::from_rgb(84, 170, 70), Color32::from_rgb(70, 156, 62)];
    let mut waved = false;
    for (k, &(along, side)) in leaves.iter().take(2 + look.extra.min(3)).enumerate() {
        let base = main[(along * 12.0).round() as usize];
        let wiggle = (now * speed + k as f64 * 0.9).sin() as f32 * if look.held { 0.3 } else { 0.08 };
        let mut a = side * (1.0 + droop * 0.7) + wiggle;
        if !waved && look.wave != 0.0 && side == look.wave.signum() {
            waved = true;
            a = side * (0.5 + (t * 12.0).sin() * 0.35);
        }
        let len = 17.0 * (1.0 - 0.15 * look.wilt) * (1.0 - 0.5 * look.hide);
        let dir = Vec2::new(a.sin(), a.cos());
        let across = Vec2::new(dir.y, -dir.x);
        let start = base + dir * 2.0;
        let leaf: Vec<Pos2> = (0..=10)
            .map(|i| i as f32 / 10.0)
            .map(|f| start + dir * f * len + across * (f * std::f32::consts::PI).sin().max(0.0).powf(0.8) * 5.5)
            .chain((0..=10).rev().map(|i| i as f32 / 10.0).map(|f| start + dir * f * len - across * (f * std::f32::consts::PI).sin().max(0.0).powf(0.8) * 5.5))
            .map(up)
            .collect();
        painter.line_segment([up(base), up(start)], Stroke::new(1.4 * u, stem_color));
        painter.add(egui::Shape::convex_polygon(leaf, wilted(green[k % 2], (186, 160, 72), look.wilt), Stroke::new(1.0, wilted(Color32::from_rgb(40, 100, 40), (120, 96, 44), look.wilt))));
        painter.line_segment([up(start), up(start + dir * len * 0.85)], Stroke::new(0.9, wilted(Color32::from_rgb(150, 214, 124), (214, 196, 120), look.wilt)));
        if look.glad {
            drop_at(painter, up(start + dir * len * 0.45 + across * 2.0), u);
        }
    }
    sunflower_head(painter, look, up, main[main.len() - 1], 1.0, face, u);
}

/// A sunflower's head at `c` (his units, upwards), `scale` its size: two rings of petals (hanging
/// as he wilts), its brown heart and its seeds turned towards `face`.
fn sunflower_head(painter: &egui::Painter, look: Look, up: &dyn Fn(Vec2) -> Pos2, c: Vec2, scale: f32, face: Vec2, u: f32) {
    let k = scale * (1.0 - 0.3 * look.hide);
    let rings = [
        (0.5f32, 9.0f32, Color32::from_rgb(240, 160, 30), Color32::from_rgb(190, 110, 20)),
        (0.0, 8.0, Color32::from_rgb(252, 208, 48), Color32::from_rgb(210, 150, 20)),
    ];
    for (shift, len, fill, edge) in rings {
        for i in 0..14 {
            let a = (i as f32 + shift) / 14.0 * std::f32::consts::TAU;
            let dir = (Vec2::new(a.cos(), a.sin()) - Vec2::new(0.0, 0.5 * look.wilt)).normalized();
            let across = Vec2::new(-dir.y, dir.x);
            let petal: Vec<Pos2> = (0..=6)
                .map(|j| j as f32 / 6.0)
                .map(|f| c + dir * (6.5 + len * f) * k + across * (f * std::f32::consts::PI).sin() * 2.6 * k)
                .chain((0..=6).rev().map(|j| j as f32 / 6.0).map(|f| c + dir * (6.5 + len * f) * k - across * (f * std::f32::consts::PI).sin() * 2.6 * k))
                .map(up)
                .collect();
            painter.add(egui::Shape::convex_polygon(petal, wilted(fill, (196, 170, 110), look.wilt), Stroke::new(0.7 * u, wilted(edge, (150, 120, 70), look.wilt))));
        }
    }
    painter.circle_filled(up(c), 8.2 * k * u, Color32::from_rgb(108, 66, 30));
    painter.circle_stroke(up(c), 8.2 * k * u, Stroke::new(0.9 * u, Color32::from_rgb(70, 40, 18)));
    let seed = Color32::from_rgb(160, 108, 52);
    let heart = c + face * k;
    painter.circle_filled(up(heart), 1.0 * k * u, seed);
    for (r, n) in [(2.8f32, 6), (5.2, 11)] {
        for i in 0..n {
            let a = i as f32 / n as f32 * std::f32::consts::TAU + r;
            painter.circle_filled(up(heart * 0.6 + c * 0.4 + Vec2::angled(a) * r * k), 0.85 * k * u, seed);
        }
    }
}

/// Debug builds, with `MATHEO_GALLERY=1`: each plant side by side (well, watered, thirsty, grown),
/// on the right of the outfits.
fn paint_skin_gallery(painter: &egui::Painter, area: Rect, now: f64) {
    let looks = [
        ("", Look::default()),
        ("arrosage", Look { glad: true, ..Look::default() }),
        ("soif", Look { wilt: 1.0, ..Look::default() }),
        ("fleur", Look { extra: 3, flower: true, ..Look::default() }),
    ];
    let step = 110.0;
    let cols = looks.len() as f32;
    // On the right, beside the outfits and the antics.
    let left = area.max.x - step * (cols - 0.5) - 20.0;
    for (row, skin) in [Skin::Cactus, Skin::Sunflower].into_iter().enumerate() {
        let y = area.center().y - 380.0 + row as f32 * 175.0;
        painter.rect_filled(Rect::from_min_max(Pos2::new(left - step / 2.0, y - 130.0), Pos2::new(left + step * (cols - 0.5), y + 40.0)), 14.0, Color32::from_rgba_unmultiplied(30, 32, 40, 200));
        for (i, (label, look)) in looks.iter().enumerate() {
            let foot = Pos2::new(left + step * i as f32, y);
            paint_matheo_as(painter, foot, MATHEO_H, now, Look { skin, ..*look });
            let text = if label.is_empty() { skin.name().to_owned() } else { format!("{} {label}", skin.name()) };
            painter.text(foot + Vec2::new(0.0, 22.0), egui::Align2::CENTER_CENTER, text, FontId::proportional(13.0), Color32::from_rgb(235, 235, 240));
        }
    }
    // Changing plant, again and again: Mathéo, Paloma, Hélio in turn.
    let every = 2.0f64;
    let k = (now / every) as usize;
    let t = (now % every) as f32;
    let (before, after) = (Skin::ALL[k % 3], Skin::ALL[(k + 1) % 3]);
    let foot = Pos2::new(left - step * 1.1, area.center().y - 380.0 + 175.0 / 2.0);
    painter.rect_filled(Rect::from_min_max(foot + Vec2::new(-step / 2.0, -130.0), foot + Vec2::new(step / 2.0, 40.0)), 14.0, Color32::from_rgba_unmultiplied(30, 32, 40, 200));
    let look = match swap_at(t) {
        Some((true, sink)) => Look { skin: before, sink, ..Look::default() },
        Some((false, sink)) => Look { skin: after, sink, ..Look::default() },
        None => Look { skin: after, ..Look::default() },
    };
    paint_matheo_as(painter, foot, MATHEO_H, now, look);
    paint_swap_dirt(painter, foot, MATHEO_H, look, t);
    painter.text(foot + Vec2::new(0.0, 22.0), egui::Align2::CENTER_CENTER, "Changement", FontId::proportional(13.0), Color32::from_rgb(235, 235, 240));
}

/// Changing plant: the one before sinks into the soil (`SINKS` seconds), the new one pops out,
/// a little bigger, then settles (`POPS`), soil flying.
const SINKS: f32 = 0.45;
const POPS: f32 = 0.6;

/// Changing plant, `t` seconds in: still the one before, and how far down in the soil (see
/// `Look::sink`); None once over.
fn swap_at(t: f32) -> Option<(bool, f32)> {
    if t < SINKS {
        let f = t / SINKS;
        Some((true, f * f))
    } else if t < SINKS + POPS {
        // Out with a bounce (ease out back): past its size, then back.
        let f = (t - SINKS) / POPS - 1.0;
        let c = 2.4;
        Some((false, 1.0 - (1.0 + (c + 1.0) * f * f * f + c * f * f)))
    } else {
        None
    }
}

/// Soil flying out of his pot as the new plant pops out (`t` seconds into the change), him
/// standing at `foot`, `size` high.
fn paint_swap_dirt(painter: &egui::Painter, foot: Pos2, size: f32, look: Look, t: f32) {
    let u = size / 70.0;
    let soil = foot - Vec2::new(0.0, look.hop) + Vec2::new(0.0, -32.0) * u;
    let from = SINKS - 0.08;
    let fly = t - from;
    if !(0.0..0.6).contains(&fly) {
        return;
    }
    let alpha = (255.0 * (1.0 - fly / 0.6)) as u8;
    let colors = [Color32::from_rgba_unmultiplied(92, 62, 40, alpha), Color32::from_rgba_unmultiplied(124, 86, 56, alpha)];
    for k in 0..9 {
        let kf = k as f32;
        let x = (kf - 4.0) / 4.0;
        let v = Vec2::new(x * 70.0 + (kf * 2.3).sin() * 12.0, -95.0 - (kf * 1.7).cos().abs() * 50.0);
        let p = soil + Vec2::new(x * 10.0, 0.0) * u + (v * fly + Vec2::new(0.0, 380.0) * fly * fly) * u;
        painter.circle_filled(p, (1.4 + (k % 3) as f32 * 0.5) * u, colors[k % 2]);
    }
}

/// What he says about himself, in French, said by a she (Paloma): each masculine word of his
/// sentences and its feminine (the sentences come from Ronnie's: see the test below).
const SHE: [(&str, &str); 6] = [
    ("arrosé ce matin", "arrosée ce matin"),
    ("tout seul", "toute seule"),
    ("fier de toi", "fière de toi"),
    ("je suis content", "je suis contente"),
    ("Enchanté,", "Enchantée,"),
    ("tout sec", "toute sèche"),
];

impl Skin {
    /// A she (Paloma), not a he.
    fn she(self) -> bool {
        self == Skin::Cactus
    }

    /// `text`, said by him in this skin: in the feminine for a she (still Mathéo: the skins' names
    /// are only theirs).
    fn says(self, text: &str) -> String {
        let mut text = text.to_owned();
        if self.she() {
            for (he, she) in SHE {
                text = text.replace(he, she);
            }
        }
        text
    }

}

#[cfg(test)]
mod skin_tests {
    use super::{SHE, Skin};

    /// Every French sentence of his.
    fn french() -> Vec<&'static str> {
        let t = crate::i18n::Lang::Fr.strings();
        let mut all = vec![t.matheo_update, t.matheo_named[0], t.menu_tips_on];
        for list in [t.matheo_water, t.matheo_home, t.matheo_eyes, t.matheo_back, t.matheo_named, t.more_thirsty] {
            all.extend_from_slice(list);
        }
        all
    }

    #[test]
    fn each_feminine_word_is_still_said() {
        let all = french();
        for (he, _) in SHE {
            assert!(all.iter().any(|s| s.contains(he)), "no more « {he} » in his sentences: change SHE");
        }
    }

    #[test]
    fn a_new_plant_sinks_then_pops() {
        use super::{POPS, SINKS, swap_at};
        assert_eq!(swap_at(0.0), Some((true, 0.0)));
        assert!(swap_at(SINKS - 0.01).is_some_and(|(old, sink)| old && sink > 0.9));
        assert!(swap_at(SINKS + 0.01).is_some_and(|(old, sink)| !old && sink > 0.9));
        // Bigger than himself a moment, popping out.
        assert!((0..60).any(|k| swap_at(SINKS + POPS * k as f32 / 60.0).is_some_and(|(_, sink)| sink < -0.05)));
        assert!(swap_at(SINKS + POPS - 0.001).is_some_and(|(_, sink)| sink.abs() < 0.02));
        assert_eq!(swap_at(SINKS + POPS), None);
    }

    #[test]
    fn paloma_speaks_as_a_she() {
        assert_eq!(Skin::Cactus.says("Enchanté, Julie ! Moi c’est toujours Mathéo, hein."), "Enchantée, Julie ! Moi c’est toujours Mathéo, hein.");
        assert_eq!(Skin::Cactus.says("Je suis tout sec, Julie…"), "Je suis toute sèche, Julie…");
        assert_eq!(Skin::Sunflower.says("Je suis tout sec, Julie…"), "Je suis tout sec, Julie…");
    }
}
