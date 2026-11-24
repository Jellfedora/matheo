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
    if let Some(season) = look.feast {
        paint_cactus_feast(painter, season, &p, &arms, h, u, t);
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
    if let Some(season) = look.feast {
        paint_sunflower_feast(painter, season, up, main[main.len() - 1], 1.0 - 0.3 * look.hide, u, t);
    }
}

/// Paloma's touch to the season's outfit (over the one on the pot): a striped scarf at Christmas, a
/// gold star on top for the New Year, a heart on top on Valentine's Day, an egg in her raised arm at
/// Easter, a little pumpkin on top at Halloween. `p` places a point of her (her units, upwards from
/// the soil, as she leans), `h` her height.
fn paint_cactus_feast(painter: &egui::Painter, season: Season, p: &dyn Fn(f32, f32) -> Pos2, arms: &[Vec<Pos2>], h: f32, u: f32, t: f32) {
    let top = p(0.0, h);
    match season {
        Season::Christmas => {
            // Wound round under her top, its end hanging down her side.
            let y = h - 11.5;
            let red = Color32::from_rgb(206, 32, 44);
            let white = Color32::from_rgb(250, 250, 248);
            painter.add(egui::Shape::convex_polygon(vec![p(-10.8, y - 2.4), p(10.8, y - 2.4), p(10.8, y + 2.4), p(-10.8, y + 2.4)], red, Stroke::new(0.7 * u, Color32::from_rgb(140, 16, 26))));
            for x in [-6.0f32, 0.0, 6.0] {
                painter.line_segment([p(x, y - 2.2), p(x, y + 2.2)], Stroke::new(1.4 * u, white));
            }
            let sway = (t * 1.5).sin() * 0.8;
            painter.add(egui::Shape::convex_polygon(vec![p(4.0, y - 1.5), p(8.5, y - 1.5), p(9.5 + sway, y - 12.0), p(5.0 + sway, y - 12.0)], red, Stroke::new(0.7 * u, Color32::from_rgb(140, 16, 26))));
            painter.line_segment([p(5.2 + sway, y - 9.0), p(9.3 + sway, y - 9.0)], Stroke::new(1.4 * u, white));
        }
        Season::NewYear => {
            // A gold star, turning slowly, twinkling.
            let c = top + Vec2::new(0.0, -6.0 * u);
            let gold = Color32::from_rgb(250, 204, 50);
            let r = (5.2 + (t * 3.0).sin() * 0.4) * u;
            let point = |i: usize, rad: f32| c + Vec2::angled(t * 0.6 - std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::TAU / 10.0) * rad;
            for i in (0..10).step_by(2) {
                painter.add(egui::Shape::convex_polygon(vec![point(i, r), point(i + 1, r * 0.45), c, point(i + 9, r * 0.45)], gold, Stroke::NONE));
            }
            painter.circle_filled(c, r * 0.45, gold);
        }
        Season::Valentine => {
            // A heart on her top, beating.
            let beat = 1.0 + ((t * 4.0).sin().max(0.0)).powi(4) * 0.18;
            heart(painter, top + Vec2::new(0.0, -4.0 * u), 4.8 * u * beat, Color32::from_rgb(226, 36, 78));
        }
        Season::Easter => {
            // An egg held up in her first arm.
            if let Some(&end) = arms.first().and_then(|a| a.last()) {
                easter_egg(painter, end + Vec2::new(0.0, -6.5 * u), 0.75 * u, Color32::from_rgb(250, 190, 210));
            }
        }
        Season::Halloween => {
            // A little pumpkin on her top, its face lit.
            let c = top + Vec2::new(0.0, -4.0 * u);
            let orange = Color32::from_rgb(240, 130, 30);
            let edge = Stroke::new(0.7 * u, Color32::from_rgb(170, 80, 10));
            for dx in [-2.6f32, 2.6, 0.0] {
                painter.add(egui::Shape::ellipse_filled(c + Vec2::new(dx, 0.0) * u, Vec2::new(3.6, 4.0) * u, orange));
                painter.add(egui::Shape::ellipse_stroke(c + Vec2::new(dx, 0.0) * u, Vec2::new(3.6, 4.0) * u, edge));
            }
            painter.line_segment([c + Vec2::new(0.0, -3.6) * u, c + Vec2::new(0.8, -6.0) * u], Stroke::new(1.4 * u, Color32::from_rgb(80, 110, 40)));
            let glow = Color32::from_rgb(255, 226, 90);
            for s in [-1.0f32, 1.0] {
                painter.add(egui::Shape::convex_polygon(vec![c + Vec2::new(s * 1.0, -0.4) * u, c + Vec2::new(s * 2.6, -0.4) * u, c + Vec2::new(s * 1.8, -1.8) * u], glow, Stroke::NONE));
            }
            painter.add(egui::Shape::convex_polygon(vec![c + Vec2::new(-2.2, 1.0) * u, c + Vec2::new(2.2, 1.0) * u, c + Vec2::new(0.0, 2.4) * u], glow, Stroke::NONE));
        }
    }
}

/// Hélio's touch to the season's outfit (over the one on the pot): a little Santa hat on his head at
/// Christmas, a gold crown for the New Year, his heart a red heart on Valentine's Day, a chick on his
/// head at Easter, a spider hanging from his head at Halloween. `c` his head's centre (his units,
/// upwards), `k` its size.
fn paint_sunflower_feast(painter: &egui::Painter, season: Season, up: &dyn Fn(Vec2) -> Pos2, c: Vec2, k: f32, u: f32, t: f32) {
    let at = |x: f32, y: f32| up(c + Vec2::new(x, y) * k);
    let poly = |points: &[(f32, f32)], fill: Color32, edge: Stroke| {
        painter.add(egui::Shape::convex_polygon(points.iter().map(|&(x, y)| at(x, y)).collect(), fill, edge));
    };
    match season {
        Season::Christmas => {
            let red = Color32::from_rgb(206, 32, 44);
            let edge = Stroke::new(0.8 * u, Color32::from_rgb(140, 16, 26));
            poly(&[(-8.0, 13.0), (8.0, 13.0), (5.0, 22.0), (-1.0, 25.0)], red, edge);
            // (Its three points never in a line: a flat triangle drew huge shapes over the screen.)
            poly(&[(-1.0, 25.0), (5.0, 22.0), (12.0, 17.0)], red, edge);
            let fur = Color32::from_rgb(250, 250, 248);
            let soft = Stroke::new(0.7 * u, Color32::from_rgb(200, 200, 205));
            poly(&[(-9.5, 11.0), (9.5, 11.0), (9.5, 15.0), (-9.5, 15.0)], fur, soft);
            painter.circle_filled(at(12.0, 17.0), 2.4 * k * u, fur);
            painter.circle_stroke(at(12.0, 17.0), 2.4 * k * u, soft);
        }
        Season::NewYear => {
            let gold = Color32::from_rgb(244, 196, 48);
            let edge = Stroke::new(0.8 * u, Color32::from_rgb(190, 140, 20));
            poly(&[(-8.0, 13.0), (8.0, 13.0), (8.0, 17.0), (-8.0, 17.0)], gold, edge);
            for x in [-6.0f32, 0.0, 6.0] {
                poly(&[(x - 2.6, 16.5), (x + 2.6, 16.5), (x, 23.0)], gold, edge);
            }
            for (x, color) in [(-6.0f32, Color32::from_rgb(60, 170, 230)), (0.0, Color32::from_rgb(230, 50, 80)), (6.0, Color32::from_rgb(120, 200, 90))] {
                painter.circle_filled(at(x, 15.0), 1.2 * k * u, color);
            }
        }
        Season::Valentine => {
            let beat = 1.0 + ((t * 4.0).sin().max(0.0)).powi(4) * 0.18;
            heart(painter, at(0.0, 0.0) + Vec2::new(0.0, -0.8 * u), 6.0 * k * u * beat, Color32::from_rgb(226, 36, 78));
        }
        Season::Easter => {
            // A chick sitting on his head, looking about now and then.
            let yellow = Color32::from_rgb(252, 222, 70);
            let edge = Stroke::new(0.7 * u, Color32::from_rgb(210, 170, 30));
            let look = if (t % 4.0) < 2.0 { 1.0 } else { -1.0 };
            let body = at(0.0, 18.5);
            painter.circle_filled(body, 4.6 * k * u, yellow);
            painter.circle_stroke(body, 4.6 * k * u, edge);
            let head = at(look * 2.4, 23.5);
            painter.circle_filled(head, 3.0 * k * u, yellow);
            painter.circle_stroke(head, 3.0 * k * u, edge);
            painter.circle_filled(head + Vec2::new(look * 1.0, -0.6) * k * u, 0.6 * k * u, Color32::from_rgb(40, 30, 20));
            painter.add(egui::Shape::convex_polygon(vec![head + Vec2::new(look * 2.4, -0.6) * k * u, head + Vec2::new(look * 4.6, 0.2) * k * u, head + Vec2::new(look * 2.4, 0.9) * k * u], Color32::from_rgb(244, 140, 40), Stroke::NONE));
        }
        Season::Halloween => {
            // A spider on its thread, from the tip of his petals on the left, going up and down.
            let from = at(-15.0, 2.0);
            let to = from + Vec2::new(0.0, (16.0 + (t * 1.3).sin() * 5.0) * u);
            painter.line_segment([from, to], Stroke::new(0.6 * u, Color32::from_rgb(225, 225, 235)));
            let black = Color32::from_rgb(30, 22, 36);
            for s in [-1.0f32, 1.0] {
                for (i, a) in [-0.6f32, 0.0, 0.6, 1.1].into_iter().enumerate() {
                    let wiggle = (t * 6.0 + i as f32).sin() * 0.25;
                    let knee = to + Vec2::new(s * 3.8, -2.0 + a * 2.6) * u;
                    painter.line_segment([to, knee], Stroke::new(0.8 * u, black));
                    painter.line_segment([knee, knee + Vec2::new(s * 1.6, 2.4 + wiggle) * u], Stroke::new(0.8 * u, black));
                }
            }
            painter.circle_filled(to + Vec2::new(0.0, 1.6 * u), 3.0 * u, black);
            painter.circle_filled(to - Vec2::new(0.0, 1.2 * u), 2.0 * u, black);
            painter.circle_filled(to + Vec2::new(-0.8, -1.5) * u, 0.5 * u, Color32::WHITE);
            painter.circle_filled(to + Vec2::new(0.8, -1.5) * u, 0.5 * u, Color32::WHITE);
        }
    }
}

/// An Easter egg at `c`, `s` its size (1: about 10 points by 13), pale `color` with a pink zigzag
/// and yellow dots.
fn easter_egg(painter: &egui::Painter, c: Pos2, s: f32, color: Color32) {
    let egg: Vec<Pos2> = (0..24).map(|i| { let a = i as f32 / 24.0 * std::f32::consts::TAU; c + Vec2::new(5.0 * a.cos(), 6.5 * a.sin() * if a.sin() < 0.0 { 1.12 } else { 1.0 }) * s }).collect();
    painter.add(egui::Shape::convex_polygon(egg, color, Stroke::new(0.8 * s, Color32::from_rgb(200, 130, 160))));
    let zig: Vec<Pos2> = (0..=6).map(|i| c + Vec2::new(-4.6 + i as f32 * 1.53, if i % 2 == 0 { -1.0 } else { 1.2 }) * s).collect();
    painter.add(egui::Shape::line(zig, Stroke::new(1.2 * s, Color32::from_rgb(150, 110, 220))));
    for d in [Vec2::new(-1.8, 3.4), Vec2::new(1.8, 3.4), Vec2::new(0.0, -4.0)] {
        painter.circle_filled(c + d * s, 0.9 * s, Color32::from_rgb(250, 220, 80));
    }
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

    /// Every plant, in every outfit, every way he may be (tipped over, squashed, hopping, hiding,
    /// wilted, held, sinking): nothing drawn far from him (a stray point made a shape cover the screen).
    #[test]
    fn what_he_wears_stays_on_him() {
        use super::{Look, Season, paint_chute, paint_matheo_as, paint_season};
        let ctx = egui::Context::default();
        let foot = egui::Pos2::new(500.0, 500.0);
        let mut looks = vec![Look::default()];
        for tilt in [-3.0f32, -1.2, -0.4, 0.4, 1.2, 3.0] {
            looks.push(Look { tilt, ..Look::default() });
        }
        for (squash, hop, hide, wilt, sink) in [(0.3f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32), (-0.2, 20.0, 0.0, 0.0, 0.0), (0.0, 0.0, 1.0, 0.0, 0.0), (0.0, 0.0, 0.0, 1.0, 0.0), (0.0, 0.0, 0.0, 0.0, 0.6), (0.0, 0.0, 0.0, 0.0, -0.1)] {
            looks.push(Look { squash, hop, hide, wilt, sink, ..Look::default() });
        }
        looks.push(Look { held: true, wave: 1.0, extra: 3, flower: true, tired: true, glad: true, gaze: egui::Vec2::new(1.0, -1.0), ..Look::default() });
        for skin in Skin::ALL {
            for season in Season::ALL {
                for (i, look) in looks.iter().enumerate() {
                    let look = Look { skin, feast: Some(season), ..*look };
                    for (now, open) in [(0.0, 0.0), (0.7, 0.05), (1.9, 0.3), (3.3, 1.0), (4.1, 1.08)] {
                        let mut out = ctx.run_ui(Default::default(), |ui| {
                            let painter = ui.ctx().layer_painter(egui::LayerId::background());
                            paint_chute(&painter, foot, 1.0, look, open);
                            paint_matheo_as(&painter, foot, 70.0, now, look);
                            paint_season(&painter, foot, 70.0, now, look, season);
                        });
                        out.textures_delta.clear();
                        // What is really drawn: the triangles, feathered edges and all.
                        for shape in out.shapes {
                            for prim in ctx.tessellate(vec![shape.clone()], 2.0) {
                                if let egui::epaint::Primitive::Mesh(mesh) = prim.primitive {
                                    for v in &mesh.vertices {
                                        assert!(v.pos.is_finite() && (v.pos - foot).length() < 260.0, "{skin:?} {season:?} look {i} at {now}, open {open}: {:?} in {:?}", v.pos, shape.shape);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn paloma_speaks_as_a_she() {
        assert_eq!(Skin::Cactus.says("Enchanté, Julie ! Moi c’est toujours Mathéo, hein."), "Enchantée, Julie ! Moi c’est toujours Mathéo, hein.");
        assert_eq!(Skin::Cactus.says("Je suis tout sec, Julie…"), "Je suis toute sèche, Julie…");
        assert_eq!(Skin::Sunflower.says("Je suis tout sec, Julie…"), "Je suis tout sec, Julie…");
    }
}
