/// A season's outfit, Mathéo's own (Ronnie's Mathéo has none): put on over his drawing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Season {
    /// December 1 to 25: a Santa hat, snow falling.
    Christmas,
    /// December 31 to January 2: a party hat, confetti.
    NewYear,
    /// February 14: a red bow, hearts floating up.
    Valentine,
    /// Holy week to Easter Monday: bunny ears, an egg beside him.
    Easter,
    /// October 25 to 31: a witch's hat, a bat flying round.
    Halloween,
}

impl Season {
    const ALL: [Season; 5] = [Season::Christmas, Season::NewYear, Season::Valentine, Season::Easter, Season::Halloween];

    /// The outfit worn on `day`, if any.
    fn on(day: chrono::NaiveDate) -> Option<Season> {
        use chrono::Datelike;
        let (m, d) = (day.month(), day.day());
        let easter = easter(day.year());
        match (m, d) {
            (12, 1..=25) => Some(Season::Christmas),
            (12, 31) | (1, 1..=2) => Some(Season::NewYear),
            (2, 14) => Some(Season::Valentine),
            (10, 25..=31) => Some(Season::Halloween),
            _ if easter.is_some_and(|e| (-6..=1).contains(&(day - e).num_days())) => Some(Season::Easter),
            _ => None,
        }
    }

    /// Today's outfit (debug builds: `MATHEO_SEASON=christmas|newyear|valentine|easter|halloween`).
    fn now() -> Option<Season> {
        if cfg!(debug_assertions)
            && let Ok(forced) = std::env::var("MATHEO_SEASON")
        {
            return Season::ALL.into_iter().find(|s| format!("{s:?}").eq_ignore_ascii_case(&forced));
        }
        Season::on(chrono::Local::now().date_naive())
    }

    /// Its name, in the gallery.
    fn label(self) -> &'static str {
        match self {
            Season::Christmas => "Noël",
            Season::NewYear => "Nouvel an",
            Season::Valentine => "Saint-Valentin",
            Season::Easter => "Pâques",
            Season::Halloween => "Halloween",
        }
    }
}

/// Easter Sunday in `year` (Gregorian calendar, the anonymous algorithm).
fn easter(year: i32) -> Option<chrono::NaiveDate> {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = (h + l - 7 * m + 114) % 31 + 1;
    chrono::NaiveDate::from_ymd_opt(year, month as u32, day as u32)
}

/// A heart (point down) centred at `c`, `r` wide on each side.
fn heart(painter: &egui::Painter, c: Pos2, r: f32, color: Color32) {
    painter.circle_filled(c + Vec2::new(-r * 0.5, -r * 0.2), r * 0.55, color);
    painter.circle_filled(c + Vec2::new(r * 0.5, -r * 0.2), r * 0.55, color);
    painter.add(egui::Shape::convex_polygon(vec![c + Vec2::new(-r * 1.02, 0.0), c + Vec2::new(r * 1.02, 0.0), c + Vec2::new(0.0, r * 1.15)], color, Stroke::NONE));
}

/// `season`'s outfit on Mathéo standing at `foot`, `size` high, as `paint_matheo_as` draws him (the
/// same frame: what he wears follows him tipped over, squashed, hopping); what floats around him
/// (snow, confetti, hearts, the bat) stays upright.
fn paint_season(painter: &egui::Painter, foot: Pos2, size: f32, now: f64, look: Look, season: Season) {
    let u = size / 70.0;
    let ground = foot;
    // As in `paint_matheo_as`.
    let tremble = if look.hide > 0.3 { (now * 47.0).sin() as f32 * 0.9 * look.hide } else { 0.0 };
    let foot = foot - Vec2::new(-tremble, look.hop);
    let pivot = foot - Vec2::new(0.0, 20.0 * u);
    let (sin, cos) = look.tilt.sin_cos();
    let at = |x: f32, y: f32| {
        let v = Vec2::new(x * (1.0 + look.squash * 0.6), (y + 20.0) * (1.0 - look.squash)) * u;
        pivot + Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
    };
    let poly = |points: &[(f32, f32)], fill: Color32, edge: Stroke| {
        painter.add(egui::Shape::convex_polygon(points.iter().map(|&(x, y)| at(x, y)).collect(), fill, edge));
    };
    let t = now as f32;
    // Around him, upright: x, y in his units from where he stands.
    let around = |x: f32, y: f32| ground + Vec2::new(x, y) * u;
    match season {
        Season::Christmas => {
            // Snow falling slowly around him, in front of the screen.
            for k in 0..8 {
                let k = k as f32;
                let fall = (t * 9.0 + k * 23.0) % 90.0;
                let p = around(-38.0 + k * 11.0 + (t * 0.9 + k).sin() * 4.0, -88.0 + fall);
                let alpha = (255.0 * (1.0 - fall / 90.0).min(1.0)) as u8;
                painter.circle_filled(p, 1.7 * u, Color32::from_rgba_unmultiplied(255, 255, 255, alpha));
                painter.circle_stroke(p, 1.7 * u, Stroke::new(0.5 * u, Color32::from_rgba_unmultiplied(150, 170, 200, alpha)));
            }
            // A Santa hat on his head (the pot's rim), its tip falling to the right.
            let red = Color32::from_rgb(206, 32, 44);
            let edge = Stroke::new(1.0 * u, Color32::from_rgb(140, 16, 26));
            poly(&[(-17.0, -33.0), (17.0, -33.0), (8.0, -52.0), (-3.0, -56.0)], red, edge);
            poly(&[(-3.0, -56.0), (8.0, -52.0), (19.0, -46.0)], red, edge);
            let fur = Color32::from_rgb(250, 250, 248);
            let soft = Stroke::new(0.8 * u, Color32::from_rgb(200, 200, 205));
            poly(&[(-19.5, -38.0), (19.5, -38.0), (20.5, -30.5), (-20.5, -30.5)], fur, soft);
            painter.circle_filled(at(19.5, -45.0), 3.8 * u, fur);
            painter.circle_stroke(at(19.5, -45.0), 3.8 * u, soft);
        }
        Season::NewYear => {
            // Confetti falling, turning.
            let colors = [Color32::from_rgb(240, 70, 90), Color32::from_rgb(250, 200, 40), Color32::from_rgb(60, 170, 230), Color32::from_rgb(120, 200, 90), Color32::from_rgb(180, 100, 220)];
            for k in 0..10 {
                let kf = k as f32;
                let fall = (t * 14.0 + kf * 17.0) % 95.0;
                let c = around(-42.0 + kf * 9.0 + (t * 1.7 + kf).sin() * 5.0, -92.0 + fall);
                let d = Vec2::angled(t * 4.0 + kf) * 2.2 * u;
                let n = Vec2::new(-d.y, d.x) * 0.5;
                painter.add(egui::Shape::convex_polygon(vec![c - d - n, c + d - n, c + d + n, c - d + n], colors[k % colors.len()], Stroke::NONE));
            }
            // A party hat, gold with purple stripes, a pompom on top.
            let gold = Color32::from_rgb(244, 196, 48);
            poly(&[(-10.0, -33.0), (10.0, -33.0), (0.0, -60.0)], gold, Stroke::new(1.0 * u, Color32::from_rgb(190, 140, 20)));
            let stripe = Stroke::new(1.6 * u, Color32::from_rgb(150, 70, 200));
            for (y, w) in [(-39.0, 7.6), (-46.0, 5.0), (-53.0, 2.4)] {
                painter.line_segment([at(-w, y), at(w, y + 1.5)], stripe);
            }
            for (i, c) in colors.iter().enumerate() {
                painter.circle_filled(at(0.0, -61.0) + Vec2::angled(i as f32 * 1.256 + t * 2.0) * 1.8 * u, 1.9 * u, *c);
            }
        }
        Season::Valentine => {
            // Hearts floating up, swaying, fading.
            for k in 0..4 {
                let kf = k as f32;
                let rise = (t * 12.0 + kf * 21.0) % 80.0;
                let c = around(-26.0 + kf * 17.0 + (t * 1.3 + kf * 2.0).sin() * 5.0, -40.0 - rise);
                let alpha = (230.0 * (1.0 - rise / 80.0)) as u8;
                heart(painter, c, 4.0 * u, Color32::from_rgba_unmultiplied(230, 40, 80, alpha));
            }
            // A red bow under his face.
            let red = Color32::from_rgb(214, 30, 70);
            let edge = Stroke::new(0.8 * u, Color32::from_rgb(150, 10, 40));
            poly(&[(0.0, -4.0), (-8.0, -7.5), (-8.0, -0.5)], red, edge);
            poly(&[(0.0, -4.0), (8.0, -7.5), (8.0, -0.5)], red, edge);
            painter.circle_filled(at(0.0, -4.0), 2.0 * u, red);
        }
        Season::Easter => {
            // An Easter egg beside his pot, on the ground (only while he stands there).
            if look.tilt.abs() < 0.3 && look.hop < 1.0 && !look.held {
                let c = ground + Vec2::new(27.0, -6.5) * u;
                let egg: Vec<Pos2> = (0..24).map(|i| { let a = i as f32 / 24.0 * std::f32::consts::TAU; c + Vec2::new(5.0 * a.cos(), 6.5 * a.sin() * if a.sin() < 0.0 { 1.12 } else { 1.0 }) * u }).collect();
                painter.add(egui::Shape::convex_polygon(egg, Color32::from_rgb(150, 210, 245), Stroke::new(0.8 * u, Color32::from_rgb(90, 150, 200))));
                let zig: Vec<Pos2> = (0..=6).map(|i| c + Vec2::new(-4.6 + i as f32 * 1.53, if i % 2 == 0 { -1.0 } else { 1.2 }) * u).collect();
                painter.add(egui::Shape::line(zig, Stroke::new(1.2 * u, Color32::from_rgb(250, 120, 170))));
                painter.circle_filled(c + Vec2::new(-1.8, 3.4) * u, 0.9 * u, Color32::from_rgb(250, 220, 80));
                painter.circle_filled(c + Vec2::new(1.8, 3.4) * u, 0.9 * u, Color32::from_rgb(250, 220, 80));
                painter.circle_filled(c + Vec2::new(0.0, -4.0) * u, 0.9 * u, Color32::from_rgb(250, 220, 80));
            }
            // Bunny ears on a headband, twitching now and then.
            let white = Color32::from_rgb(250, 248, 246);
            let pink = Color32::from_rgb(248, 170, 190);
            let edge = Stroke::new(0.8 * u, Color32::from_rgb(190, 180, 180));
            for s in [-1.0f32, 1.0] {
                let twitch = if (now + f64::from(s)) % 5.0 < 0.3 { 0.18 } else { 0.0 };
                let a = s * (0.32 + twitch);
                let (base_x, base_y) = (s * 8.0, -34.0);
                let ear = |w: f32, len: f32, lift: f32| -> Vec<(f32, f32)> {
                    (0..20)
                        .map(|i| {
                            let b = i as f32 / 20.0 * std::f32::consts::TAU;
                            let (lx, ly) = (w * b.cos(), -lift - len * 0.5 - len * 0.5 * b.sin());
                            (base_x + lx * a.cos() - ly * a.sin(), base_y + lx * a.sin() + ly * a.cos())
                        })
                        .collect()
                };
                poly(&ear(4.6, 24.0, 0.0), white, edge);
                poly(&ear(2.2, 17.0, 3.0), pink, Stroke::NONE);
            }
            painter.add(egui::Shape::line((0..=10).map(|i| { let x = -18.0 + i as f32 * 3.6; at(x, -33.5 - (1.0 - (x / 18.0).powi(2)) * 2.0) }).collect(), Stroke::new(2.2 * u, pink)));
        }
        Season::Halloween => {
            // A bat flying round him, flapping.
            let c = around((t * 1.1).cos() * 36.0, -66.0 + (t * 2.2).sin() * 8.0);
            let flap = (t * 16.0).sin() * 3.0 * u;
            let bat = Color32::from_rgb(30, 22, 36);
            for s in [-1.0f32, 1.0] {
                painter.add(egui::Shape::convex_polygon(vec![c, c + Vec2::new(s * 9.0 * u, -2.5 * u - flap), c + Vec2::new(s * 6.0 * u, 2.0 * u)], bat, Stroke::NONE));
            }
            painter.circle_filled(c, 2.4 * u, bat);
            painter.circle_filled(c + Vec2::new(-0.9, -0.4) * u, 0.5 * u, Color32::from_rgb(250, 200, 60));
            painter.circle_filled(c + Vec2::new(0.9, -0.4) * u, 0.5 * u, Color32::from_rgb(250, 200, 60));
            // A witch's hat: a wide brim on his rim, the crown bent at the tip, an orange band.
            let black = Color32::from_rgb(44, 30, 62);
            let edge = Stroke::new(0.9 * u, Color32::from_rgb(20, 12, 30));
            let brim: Vec<(f32, f32)> = (0..24).map(|i| { let a = i as f32 / 24.0 * std::f32::consts::TAU; (27.0 * a.cos(), -33.0 + 4.5 * a.sin()) }).collect();
            poly(&brim, black, edge);
            poly(&[(-12.0, -34.0), (12.0, -34.0), (5.0, -54.0), (-3.0, -57.0)], black, edge);
            poly(&[(-3.0, -57.0), (5.0, -54.0), (14.0, -63.0)], black, edge);
            poly(&[(-11.4, -40.0), (11.0, -40.0), (12.0, -35.5), (-12.0, -35.5)], Color32::from_rgb(240, 130, 30), Stroke::NONE);
            poly(&[(-2.6, -40.4), (2.6, -40.4), (2.6, -35.2), (-2.6, -35.2)], Color32::from_rgb(250, 210, 70), Stroke::new(0.8 * u, Color32::from_rgb(170, 130, 30)));
        }
    }
}

/// Debug builds, with `MATHEO_GALLERY=1`: each outfit side by side, above his other looks.
fn paint_season_gallery(painter: &egui::Painter, area: Rect, now: f64) {
    let step = 130.0;
    let left = area.center().x - step * (Season::ALL.len() as f32 - 1.0) / 2.0;
    let y = area.center().y - 180.0;
    painter.rect_filled(Rect::from_min_max(Pos2::new(left - step / 2.0, y - 120.0), Pos2::new(left + step * (Season::ALL.len() as f32 - 0.5), y + 40.0)), 14.0, Color32::from_rgba_unmultiplied(30, 32, 40, 200));
    for (i, season) in Season::ALL.into_iter().enumerate() {
        let foot = Pos2::new(left + step * i as f32, y);
        paint_matheo_as(painter, foot, MATHEO_H, now, Look::default());
        paint_season(painter, foot, MATHEO_H, now, Look::default(), season);
        painter.text(foot + Vec2::new(0.0, 22.0), egui::Align2::CENTER_CENTER, season.label(), FontId::proportional(13.0), Color32::from_rgb(235, 235, 240));
    }
}

#[cfg(test)]
mod season_tests {
    use chrono::NaiveDate;

    use super::{Season, easter};

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn easter_falls_on_its_sunday() {
        for (y, m, d) in [(2024, 3, 31), (2025, 4, 20), (2026, 4, 5), (2027, 3, 28), (2038, 4, 25)] {
            assert_eq!(easter(y), Some(day(y, m, d)), "{y}");
        }
    }

    #[test]
    fn each_season_has_its_days() {
        assert_eq!(Season::on(day(2026, 11, 30)), None);
        assert_eq!(Season::on(day(2026, 12, 1)), Some(Season::Christmas));
        assert_eq!(Season::on(day(2026, 12, 25)), Some(Season::Christmas));
        assert_eq!(Season::on(day(2026, 12, 26)), None);
        assert_eq!(Season::on(day(2026, 12, 31)), Some(Season::NewYear));
        assert_eq!(Season::on(day(2027, 1, 2)), Some(Season::NewYear));
        assert_eq!(Season::on(day(2027, 1, 3)), None);
        assert_eq!(Season::on(day(2027, 2, 14)), Some(Season::Valentine));
        assert_eq!(Season::on(day(2026, 10, 24)), None);
        assert_eq!(Season::on(day(2026, 10, 25)), Some(Season::Halloween));
        assert_eq!(Season::on(day(2026, 10, 31)), Some(Season::Halloween));
        // Easter 2026 is April 5: from Monday March 30 to Easter Monday April 6.
        assert_eq!(Season::on(day(2026, 3, 29)), None);
        assert_eq!(Season::on(day(2026, 3, 30)), Some(Season::Easter));
        assert_eq!(Season::on(day(2026, 4, 6)), Some(Season::Easter));
        assert_eq!(Season::on(day(2026, 4, 7)), None);
    }
}
