//! Mathéo over the screens: a transparent window over the screen where the pointer is, letting every
//! click through (but over him, his bubble, his watering can and his menu), out of the Dock and the
//! task bar, above the other windows (full-screen ones too, on macOS). Another window shows him on
//! another screen when he stands there. The pointer is read about 60 times a second; the window is
//! only drawn again when something moves.
//!
//! The same Mathéo as Ronnie's (src/app/desktop.rs there), on his own: no Gérard, no horde here.
//! macOS and Windows; Linux through X11 (XWayland under Wayland), with a compositor.

// Taken from Ronnie with all his looks: some (waving at Gérard, hiding from the horde) never show here.
#![allow(dead_code)]

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use egui::{Color32, FontId, Pos2, Rect, Stroke, Ui, Vec2};

use crate::settings::{Settings, Skin};

/// The pointer as last read: where (in points, the screens' space), the main button down, the
/// right one.
#[derive(Clone, Copy, Default, PartialEq)]
struct Mouse {
    at: Option<Pos2>,
    pressed: bool,
    right: bool,
}

/// Mathéo's window.
struct Overlay {
    mouse: Arc<Mutex<Mouse>>,
    /// Where the pointer matters (him, his bubble, his can, a margin around, where his eyes follow
    /// it): a move there draws again at once; elsewhere it waits for the next frame (see `run`).
    /// None: everywhere.
    hot: Arc<Mutex<Option<Rect>>>,
    settings: Settings,
    /// The screen covered and its part left by the Dock, the menu bar or the task bar (points).
    screen: Rect,
    work: Rect,
    pressed: bool,
    right: bool,
    last: Option<f64>,
    /// The screens another app fills (a film, a full-screen window): nothing shown there; and when
    /// last looked at.
    full: (Vec<Rect>, Option<Instant>),
    matheo: Plant,
    /// His menu, opened by a right click on him: where (screen points).
    menu: Option<Pos2>,
    /// A newer version said once; the update's end said once (installed: when, to start again).
    told_newer: bool,
    told_failed: bool,
    leaving: Option<Instant>,
    /// The windows over the other screens, by screen: kept a while once nothing goes along them
    /// (hidden), not made again each time.
    others: std::collections::HashMap<usize, Other>,
    /// The season's outfit he wears (Christmas, Halloween...), looked at each minute.
    season: Option<Season>,
    /// His own antics (a parachute, dust, wobbling as he lands): see `antics.rs`.
    antics: Antics,
    /// What he says of his own, and when (see `care.rs`).
    care: Care,
    /// The page his menu shows.
    page: Page,
    /// Your name being typed in his bubble (see `Naming`).
    naming: Option<Naming>,
    /// Asking how you are: his bubble and its thumbs up and down, as last drawn (screen points).
    mood: Option<[Rect; 3]>,
    /// What the journal last said of the windows (said again when it changes).
    logged: String,
    /// Windows: frames left to clear what stays behind the drawing (see `glass`), after a change
    /// that may have made it white.
    glass: u8,
    /// Where his eyes look (see `Look::gaze`), going smoothly towards the pointer near him.
    gaze: Vec2,
    /// Changing plant: the one before, and since when (see `swap_at`).
    swap: Option<(Skin, Instant)>,
    /// The watering can waiting beside him (kept until used): the one already counted as left
    /// unused (him thirstier), and whether it went away while he was moved (it hops in again
    /// beside him once he's down).
    can_counted: Option<Instant>,
    can_away: bool,
    /// His entrance at the start done (see `Entrance`); popping out of the bottom of the screen
    /// since when.
    entered: bool,
    pop: Option<Instant>,
    /// Debug builds: the tour of everything he does (see `TOUR`): its step (none: about to start)
    /// and since when.
    tour: Option<(Option<usize>, Instant)>,
}

/// How near the pointer comes for his eyes to follow it (points from his face).
const GAZE_FROM: f32 = 260.0;

/// A window over another screen than the pointer's.
struct Other {
    /// Frames drawn in it: it shows from the second on (before, a new window is a white square on
    /// Windows, not yet its size, its place nor see-through).
    frames: u32,
    /// When something last went along its screen.
    needed: Instant,
}

/// How long a window over another screen is kept, hidden, once nothing goes along it.
const OTHER_KEPT: Duration = Duration::from_secs(30);

/// Mathéo where he stands (his pot's foot, screen points), falling, or held.
struct Plant {
    at: Option<Pos2>,
    vy: f32,
    /// Held by the mouse: from the pointer to his foot.
    grab: Option<Vec2>,
    /// The window lets the mouse through (not over him).
    pass: bool,
    /// Where the press that grabbed him was: let go without moving, a click (his advice).
    press: Option<Pos2>,
    /// Thrown: his speed across (points a second), his tilt (radians) and how fast it turns.
    vx: f32,
    tilt: f32,
    spin: f32,
    /// Held: where he was the last frames (time, place), for his swing and his speed when let go.
    track: std::collections::VecDeque<(f64, Pos2)>,
    /// Held: when he was last swung back the other way, and which way he went (shaken: quiet).
    flips: Vec<f64>,
    way: f32,
    /// Quiet until then (Unix seconds): shaken, he keeps his advice to himself for 6 hours.
    quiet: Option<u64>,
    /// His head spinning until then (shaken, or banged against a wall).
    dizzy: Option<Instant>,
    /// When he last hit a wall, the ceiling or the floor hard (squashed a moment).
    bonk: Option<Instant>,
    /// What his bubble says and since when.
    tip: Option<(Said, Instant)>,
    /// The watering can beside him (with each reminder to drink): since when, and since when it pours.
    can: Option<(Instant, Option<Instant>)>,
    /// Watered: glad until then.
    glad: Option<Instant>,
    /// Reminders to drink left without watering him (he wilts), up to `THIRST_MAX`.
    thirst: u32,
    /// Late in the evening, as last looked at (each minute).
    tired: (bool, Option<Instant>),
    /// Friday afternoon, as last looked at (with `tired`).
    friday: bool,
    /// At the screen since then without a break (the pointer never still `BREAK` long), and when he
    /// last asked for one.
    active_since: Instant,
    break_said: Option<Instant>,
    /// Waving at Gérard (towards his side), and whether this walk of his was greeted.
    wave: f32,
    greeted: bool,
    /// Hiding from the horde (0 to 1), and whether he was.
    hide: f32,
    hiding: bool,
    /// How he grew: the day, waterings that day, days well watered (`GOOD_DAY` waterings).
    growth: (String, u32, u32),
    /// When the next one is due, and the number of the next.
    next_tip: Instant,
    told: usize,
    /// The pointer's last place and when it last moved: nobody there, no advice.
    moved: (Option<Pos2>, Instant),
    /// His bubble on the screens (points), as last drawn: a click there closes it.
    bubble: Option<Rect>,
}

impl Default for Plant {
    fn default() -> Self {
        // Not the same advice first at each start (water first all the same).
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() / 60) as usize;
        let quiet = matheo_quiet_file().and_then(|f| std::fs::read_to_string(f).ok()).and_then(|t| t.trim().parse().ok());
        Self { at: matheo_load(), vy: 0.0, grab: None, pass: true, press: None, vx: 0.0, tilt: 0.0, spin: 0.0, track: Default::default(), flips: Vec::new(), way: 0.0, quiet, dizzy: None, bonk: None, tip: None, can: None, glad: None, thirst: matheo_thirst_file().and_then(|f| std::fs::read_to_string(f).ok()).and_then(|t| t.trim().parse().ok()).unwrap_or(0), tired: (false, None), friday: false, active_since: Instant::now(), break_said: None, wave: 0.0, greeted: false, hide: 0.0, hiding: false, growth: matheo_growth_load(), next_tip: Instant::now() + TIP_FIRST, told: seed % 60 * 2, moved: (None, Instant::now()), bubble: None }
    }
}

impl Plant {
    /// His next advice in his bubble; with a reminder to drink, the watering can beside him.
    /// Late in the evening, off to bed instead (no can then); thirsty, he says so.
    fn advise(&mut self) {
        let k = self.told;
        self.told += 1;
        self.next_tip = Instant::now() + TIP_EVERY;
        if late() {
            self.tip = Some((Said::Bed(k / 2), Instant::now()));
            return;
        }
        let water = k.is_multiple_of(2);
        let long = self.active_since.elapsed() >= LONG_WORK && self.break_said.is_none_or(|t| t.elapsed() >= TIP_EVERY);
        let said = if long {
            self.break_said = Some(Instant::now());
            Said::Break(k / 2)
        } else if water && self.thirst >= 2 {
            Said::Thirsty
        } else if !water && friday() {
            // Friday afternoon: the weekend in place of moving.
            Said::Weekend(k / 2)
        } else {
            Said::Tip(k)
        };
        self.tip = Some((said, Instant::now()));
        if water && self.can.is_none_or(|(_, pour)| pour.is_none()) {
            self.can = Some((Instant::now(), None));
        }
    }

    /// Thirstier (a can left unused) or watered (0): kept beside the settings.
    fn set_thirst(&mut self, thirst: u32) {
        self.thirst = thirst.min(THIRST_MAX);
        if let Some(f) = matheo_thirst_file() {
            let _ = std::fs::write(f, self.thirst.to_string());
        }
    }

    /// Watered once more: a day with `GOOD_DAY` waterings counts as well watered (kept).
    fn watered(&mut self) {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        if self.growth.0 != today {
            self.growth = (today, 0, self.growth.2);
        }
        self.growth.1 += 1;
        if self.growth.1 == GOOD_DAY {
            self.growth.2 += 1;
        }
        if let Some(f) = matheo_growth_file() {
            let _ = std::fs::write(f, serde_json::to_string(&self.growth).unwrap_or_default());
        }
    }

    /// How wilted he looks (0 to 1).
    fn wilt(&self) -> f32 {
        self.thirst as f32 / THIRST_MAX as f32
    }
}

/// The pointer still that long: a break (two hours of work on end, he insists).
const BREAK: Duration = Duration::from_secs(5 * 60);
const LONG_WORK: Duration = Duration::from_secs(2 * 3600);
/// Waterings in a day for it to count as well watered; such days for a leaf more (up to 3), and
/// for his flower.
const GOOD_DAY: u32 = 3;
const DAYS_A_LEAF: u32 = 5;
const DAYS_A_FLOWER: u32 = 20;

fn matheo_growth_file() -> Option<std::path::PathBuf> {
    crate::config_dir().map(|d| d.join("matheo-growth.json"))
}

fn matheo_growth_load() -> (String, u32, u32) {
    matheo_growth_file().and_then(|f| std::fs::read_to_string(f).ok()).and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

/// Reminders to drink left without watering him, at most: then fully wilted.
const THIRST_MAX: u32 = 3;

fn matheo_thirst_file() -> Option<std::path::PathBuf> {
    crate::config_dir().map(|d| d.join("matheo-thirst.json"))
}

/// Late in the evening (from 11 pm to 5 am): he's tired, and sends you to bed.
fn late() -> bool {
    use chrono::Timelike;
    if let Some(forced) = forced_clock() {
        return forced == "late";
    }
    let h = chrono::Local::now().hour();
    !(5..23).contains(&h)
}

/// Friday afternoon (from 2 pm, until he's tired): sunglasses on, the weekend is near.
fn friday() -> bool {
    use chrono::{Datelike, Timelike};
    if let Some(forced) = forced_clock() {
        return forced == "friday";
    }
    let now = chrono::Local::now();
    now.weekday() == chrono::Weekday::Fri && (14..23).contains(&now.hour())
}

/// Debug builds: `MATHEO_GALLERY=1` shows each of his looks side by side in the middle of
/// the screen, alive, to see them all at once.
fn paint_matheo_gallery(painter: &egui::Painter, area: Rect, now: f64) {
    let base = Look::default();
    let pour = 0.3 + (now % 3.6) as f32;
    let looks: [(&str, Look, Option<f32>); 10] = [
        ("Normal", base, None),
        ("Tenu", Look { held: true, ..base }, None),
        ("Lancé", Look { tilt: (now as f32 * 6.0) % std::f32::consts::TAU, ..base }, None),
        ("Contre un mur", Look { squash: 0.3 * (1.0 - ((now % 1.2) as f32 / 0.25).min(1.0)), dizzy: true, ..base }, None),
        ("Secoué (6 h)", Look { dizzy: true, ..base }, None),
        ("L'arrosoir", base, Some(-1.0)),
        ("Arrosé", Look { glad: pour > POUR_FROM + 0.3, ..base }, Some(pour)),
        ("Soif (1/3)", Look { wilt: 1.0 / 3.0, ..base }, None),
        ("Dépéri (3/3)", Look { wilt: 1.0, ..base }, None),
        ("Le soir, 23 h", Look { tired: true, ..base }, None),
    ];
    let more = [
        ("Vendredi aprèm", Look { shades: true, ..base }, None),
        ("2 h sans pause", Look { hop: { let e = (now % 3.0) as f32; if e < 2.0 { (e * 9.0).sin().abs() * 8.0 * (1.0 - e / 2.0) } else { 0.0 } }, ..base }, None),
        ("Grandi, 1 mois", Look { extra: 3, flower: true, ..base }, None),
    ];
    let all: Vec<_> = looks.into_iter().chain(more).collect();
    let step = (area.width() / all.len() as f32).min(130.0);
    let left = area.center().x - step * (all.len() as f32 - 1.0) / 2.0;
    let y = area.center().y + 60.0;
    painter.rect_filled(Rect::from_min_max(Pos2::new(left - step / 2.0, y - 190.0), Pos2::new(left + step * (all.len() as f32 - 0.5), y + 40.0)), 14.0, Color32::from_rgba_unmultiplied(30, 32, 40, 200));
    for (i, (label, look, can)) in all.into_iter().enumerate() {
        let foot = Pos2::new(left + step * i as f32, y);
        paint_matheo_as(painter, foot, MATHEO_H, now, look);
        if let Some(t) = can {
            let pouring = (t >= 0.0).then(|| Instant::now() - Duration::from_secs_f32(t));
            let v = can_view(foot, 1.0, Instant::now() - Duration::from_secs(5), pouring);
            paint_can(painter, v, Vec2::ZERO);
        }
        painter.text(foot + Vec2::new(0.0, 22.0), egui::Align2::CENTER_CENTER, label, FontId::proportional(13.0), Color32::from_rgb(235, 235, 240));
    }
}

/// Debug builds: `MATHEO_CLOCK=late|friday` makes it so, to see him.
fn forced_clock() -> Option<String> {
    if cfg!(debug_assertions) { std::env::var("MATHEO_CLOCK").ok() } else { None }
}

impl Plant {
    /// His look's own part: held, tilted, dizzy, squashed, glad, tired, wilted.
    fn look(&self, tired: bool, shades: bool) -> Look {
        Look {
            held: self.grab.is_some(),
            tilt: self.tilt,
            dizzy: self.dizzy.is_some_and(|d| Instant::now() < d),
            squash: self.bonk.map_or(0.0, |b| (1.0 - b.elapsed().as_secs_f32() / 0.25).max(0.0) * 0.3),
            glad: self.glad.is_some_and(|g| Instant::now() < g) || self.can.and_then(|c| c.1).is_some_and(|p| p.elapsed().as_secs_f32() > POUR_FROM + 0.3),
            tired,
            shades: shades && !tired,
            wave: self.wave,
            hide: self.hide,
            hop: match self.tip {
                // Asking for a break: hopping a moment every ten seconds.
                Some((Said::Break(_), since)) => {
                    let e = since.elapsed().as_secs_f32() % 10.0;
                    if e < 2.0 { (e * 9.0).sin().abs() * 8.0 * (1.0 - e / 2.0) } else { 0.0 }
                }
                _ => 0.0,
            },
            extra: (self.growth.2 / DAYS_A_LEAF).min(3) as usize,
            // Today's flower (the date only looked at once watered enough).
            flower: self.growth.1 >= GOOD_DAY && self.growth.0 == chrono::Local::now().format("%Y-%m-%d").to_string(),
            gaze: Vec2::ZERO,
            skin: Skin::Classic,
            sink: 0.0,
            feast: None,
            wilt: self.wilt(),
        }
    }
}

/// Mathéo's first advice after the start, then one every half hour (water and moving in turns).
const TIP_FIRST: Duration = Duration::from_secs(20 * 60);
const TIP_EVERY: Duration = Duration::from_secs(30 * 60);
/// How long his bubble stays.
const TIP_SHOWN: Duration = Duration::from_secs(45);
/// The pointer still that long: nobody at the screen, the advice waits.
const TIP_IDLE: Duration = Duration::from_secs(3 * 60);
/// Away that long (a break, a meeting): back, a quarter of an hour before the next.
const TIP_AWAY: Duration = Duration::from_secs(10 * 60);

/// Shaken that long, he's quiet.
const QUIET_SECS: u64 = 6 * 3600;
/// Swung back the other way that many times within `SHAKE_WITHIN` seconds: shaken.
const SHAKE_FLIPS: usize = 6;
const SHAKE_WITHIN: f64 = 1.6;

/// What Mathéo's bubble says.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Said {
    /// His advice number `k` (see `matheo_tip`).
    Tip(usize),
    /// Being shaken.
    Shaking,
    /// Shaken enough: quiet for 6 hours.
    Quiet,
    /// Banged against the edge of the screen.
    Ouch,
    /// Watered.
    Thanks,
    /// Late in the evening, his advice number `k`: off to bed.
    Bed(usize),
    /// Not watered for a while: thirsty, both of you.
    Thirsty,
    /// Friday afternoon, his advice number `k`: the weekend is near.
    Weekend(usize),
    /// Two hours on end without a break: number `k`, he insists (hopping).
    Break(usize),
    /// Gérard passing by.
    Hello,
    /// The 10 o'clock horde running across: hiding.
    Hiding,
    /// The horde gone.
    Gone,
    /// A newer version of him published.
    Update,
    /// Hello, the very first time.
    Hi,
    /// Hello, the very first time on macOS, asking for Accessibility (to stand on the Dock).
    Dock,
    Installing,
    Updated,
    UpdateFailed,
    /// Looking for a newer version, asked from his menu.
    Checking,
    /// Asked: he's the latest version.
    Latest,
    /// Asked: floor couldn't be reached.
    Unreachable,
    /// What he says of his own (see `care.rs`).
    Own(Own),
}

impl Said {
    /// How long the bubble stays.
    fn lasts(self) -> Duration {
        match self {
            Said::Tip(_) => TIP_SHOWN,
            Said::Shaking => Duration::from_millis(700),
            Said::Quiet => Duration::from_secs(7),
            Said::Ouch => Duration::from_millis(1200),
            Said::Thanks => Duration::from_millis(3500),
            Said::Bed(_) | Said::Thirsty | Said::Weekend(_) => TIP_SHOWN,
            Said::Break(_) => TIP_SHOWN * 2,
            Said::Hello | Said::Gone => Duration::from_secs(4),
            Said::Hiding => Duration::from_millis(1500),
            Said::Update | Said::Hi | Said::Dock => Duration::from_secs(30),
            Said::Installing => Duration::from_secs(120),
            Said::Updated => Duration::from_secs(5),
            Said::UpdateFailed | Said::Latest | Said::Unreachable => Duration::from_secs(8),
            Said::Checking => Duration::from_secs(40),
            Said::Own(o) => o.lasts(),
        }
    }

    fn text(self, lang: crate::i18n::Lang) -> &'static str {
        let t = lang.strings();
        match self {
            Said::Tip(k) => matheo_tip(lang, k),
            Said::Shaking => t.matheo_shaking,
            Said::Quiet => t.matheo_quiet,
            Said::Ouch => t.matheo_ouch,
            Said::Thanks => t.matheo_thanks,
            Said::Bed(k) => t.matheo_bed.get(k % t.matheo_bed.len().max(1)).copied().unwrap_or_default(),
            Said::Thirsty => t.matheo_thirsty,
            Said::Weekend(k) => t.matheo_weekend.get(k % t.matheo_weekend.len().max(1)).copied().unwrap_or_default(),
            Said::Break(k) => t.matheo_break.get(k % t.matheo_break.len().max(1)).copied().unwrap_or_default(),
            Said::Hello => t.matheo_hello,
            Said::Hiding => t.matheo_hiding,
            Said::Gone => t.matheo_gone,
            Said::Update => t.matheo_update,
            Said::Hi => t.matheo_hi,
            Said::Dock => t.matheo_dock,
            Said::Installing => t.matheo_installing,
            Said::Updated => t.matheo_updated,
            Said::UpdateFailed => t.matheo_update_failed,
            Said::Checking => t.matheo_checking,
            Said::Latest => t.matheo_latest,
            Said::Unreachable => t.matheo_unreachable,
            Said::Own(o) => o.text(t, 0),
        }
    }
}

/// Mathéo's advice number `k`: water for the even ones, moving for the odd ones, each list in turn.
fn matheo_tip(lang: crate::i18n::Lang, k: usize) -> &'static str {
    let t = lang.strings();
    let list = if k.is_multiple_of(2) { t.matheo_water } else { t.matheo_moves };
    list.get(k / 2 % list.len().max(1)).copied().unwrap_or_default()
}

/// The watering can waits that long to be used, then goes.
const CAN_WAITS: Duration = Duration::from_secs(3 * 60);
/// Its watering, from the click (seconds): up over his leaves, pouring, back down, gone.
const POUR_UP: f32 = 0.5;
const POUR_FROM: f32 = 0.6;
const POUR_TO: f32 = 2.25;
const POUR_END: f32 = 2.9;

/// The side of Mathéo where the can stands (1 his right; his left near the screen's right edge).
fn can_side(foot: Pos2, works: &[Rect]) -> f32 {
    let right = works.iter().find(|w| (w.min.x..w.max.x).contains(&foot.x)).map_or(f32::INFINITY, |w| w.max.x);
    if foot.x + 90.0 > right { -1.0 } else { 1.0 }
}

/// Where the can stands beside him, to be clicked (screen points).
fn can_rect(foot: Pos2, side: f32) -> Rect {
    Rect::from_center_size(foot + Vec2::new(side * 48.0, -17.0), Vec2::new(50.0, 40.0))
}

/// The watering can as it shows: its bottom's middle, its turn, which way its spout points, how
/// opaque, and how long it has been pouring (seconds since the click).
#[derive(Clone, Copy)]
struct CanView {
    origin: Pos2,
    angle: f32,
    facing: f32,
    alpha: f32,
    pour: Option<f32>,
    /// Mathéo's foot, for the drops to land on his soil.
    foot: Pos2,
}

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn can_view(foot: Pos2, side: f32, since: Instant, pour: Option<Instant>) -> CanView {
    let rest = foot + Vec2::new(side * 48.0, 0.0);
    let above = foot + Vec2::new(side * 50.0, -99.0);
    let facing = -side;
    match pour.map(|p| p.elapsed().as_secs_f32()) {
        None => {
            // It hops in, then wiggles every few seconds to be noticed.
            let t = since.elapsed().as_secs_f32();
            let hop = (smooth(t / 0.35) * std::f32::consts::PI).sin() * 14.0;
            let k = t % 3.0;
            let wiggle = if t > 0.4 && k < 0.6 { (k * 28.0).sin() * 0.12 * (1.0 - k / 0.6) } else { 0.0 };
            CanView { origin: rest - Vec2::new(0.0, hop), angle: wiggle, facing, alpha: smooth(t / 0.2), pour: None, foot }
        }
        Some(t) => {
            let e = smooth(t / POUR_UP) * (1.0 - smooth((t - POUR_TO) / (POUR_END - POUR_TO - 0.15)));
            let arc = (e * std::f32::consts::PI).sin() * 16.0;
            let tilt = 1.15 * smooth((t - 0.2) / 0.4) * (1.0 - smooth((t - 2.2) / 0.4)) + if (POUR_FROM..POUR_TO).contains(&t) { (t * 9.0).sin() * 0.04 } else { 0.0 };
            CanView { origin: rest.lerp(above, e) + Vec2::new(side * arc, 0.0), angle: tilt * facing, facing, alpha: 1.0 - smooth((t - (POUR_END - 0.3)) / 0.3), pour: Some(t), foot }
        }
    }
}

/// The watering can: green enamel, a little daisy on its side, its arched handle at the back, a long
/// spout ending in a brass rose; pouring, drops raining from the rose onto Mathéo's soil.
fn paint_can(painter: &egui::Painter, v: CanView, shift: Vec2) {
    let o = v.origin - shift;
    let (sin, cos) = v.angle.sin_cos();
    let turn = |x: f32, y: f32| {
        let p = Vec2::new(x * v.facing, y);
        Vec2::new(p.x * cos - p.y * sin, p.x * sin + p.y * cos)
    };
    let at = |x: f32, y: f32| o + turn(x, y);
    let c = |r: u8, g: u8, b: u8| Color32::from_rgb(r, g, b).gamma_multiply(v.alpha);
    let (body, dark, light, rim) = (c(52, 146, 98), c(33, 101, 68), c(140, 214, 170), c(28, 82, 55));
    let (brass, brass_dark) = (c(218, 176, 78), c(150, 110, 42));
    let poly = |pts: &[(f32, f32)], fill: Color32, stroke: Stroke| {
        painter.add(egui::Shape::convex_polygon(pts.iter().map(|&(x, y)| at(x, y)).collect(), fill, stroke));
    };
    let curve = |pts: Vec<Pos2>, stroke: Stroke| {
        painter.add(egui::Shape::line(pts, stroke));
    };
    // Its shadow on the floor, standing.
    if v.pour.is_none() {
        painter.add(egui::Shape::ellipse_filled(o, Vec2::new(16.0, 3.0), Color32::from_black_alpha((50.0 * v.alpha) as u8)));
    }
    // The handle, arched from the top's back down to the back of the body.
    let bezier = |p0: (f32, f32), p1: (f32, f32), p2: (f32, f32)| -> Vec<Pos2> {
        (0..=14)
            .map(|i| {
                let t = i as f32 / 14.0;
                let m = 1.0 - t;
                at(m * m * p0.0 + 2.0 * m * t * p1.0 + t * t * p2.0, m * m * p0.1 + 2.0 * m * t * p1.1 + t * t * p2.1)
            })
            .collect()
    };
    curve(bezier((-4.0, -27.0), (-27.0, -44.0), (-12.0, -9.0)), Stroke::new(3.6, rim));
    curve(bezier((-4.0, -27.5), (-26.0, -43.0), (-12.5, -9.5)), Stroke::new(1.2, body));
    // The spout, from the body's front low down, up to the rose.
    let (from, to) = ((8.0f32, -9.0f32), (32.0f32, -28.5f32));
    poly(&[(8.0, -5.0), (8.0, -13.0), (31.0, -30.0), (33.0, -27.0)], body, Stroke::new(1.0, rim));
    curve(vec![at(9.0, -11.0), at(31.0, -29.2)], Stroke::new(0.8, light));
    // The rose: a brass neck, then its round face full of holes, across the spout.
    let len = ((to.0 - from.0).powi(2) + (to.1 - from.1).powi(2)).sqrt();
    let d = ((to.0 - from.0) / len, (to.1 - from.1) / len);
    let n = (-d.1, d.0);
    let neck = (to.0 + d.0 * 2.0, to.1 + d.1 * 2.0);
    poly(&[(to.0 - n.0 * 1.6, to.1 - n.1 * 1.6), (to.0 + n.0 * 1.6, to.1 + n.1 * 1.6), (neck.0 + n.0 * 3.2, neck.1 + n.1 * 3.2), (neck.0 - n.0 * 3.2, neck.1 - n.1 * 3.2)], brass_dark, Stroke::NONE);
    let face = (neck.0 + d.0 * 1.2, neck.1 + d.1 * 1.2);
    let disc: Vec<Pos2> = (0..20).map(|i| {
        let s = i as f32 / 20.0 * std::f32::consts::TAU;
        at(face.0 + n.0 * 5.0 * s.cos() + d.0 * 1.7 * s.sin(), face.1 + n.1 * 5.0 * s.cos() + d.1 * 1.7 * s.sin())
    }).collect();
    painter.add(egui::Shape::convex_polygon(disc, brass, Stroke::new(0.8, brass_dark)));
    for k in [-2.6f32, -0.9, 0.9, 2.6] {
        painter.circle_filled(at(face.0 + n.0 * k + d.0 * 0.9, face.1 + n.1 * k + d.1 * 0.9), 0.55, brass_dark);
    }
    // The body, a little narrower at the top: shaded on its far side, a highlight on the near one.
    poly(&[(-13.0, 0.0), (13.0, 0.0), (11.0, -27.0), (-11.0, -27.0)], body, Stroke::NONE);
    poly(&[(7.5, 0.0), (13.0, 0.0), (11.0, -27.0), (6.5, -27.0)], dark, Stroke::NONE);
    poly(&[(-8.5, -2.5), (-6.3, -2.5), (-5.6, -24.5), (-7.6, -24.5)], light.gamma_multiply(0.7), Stroke::NONE);
    poly(&[(-13.4, 0.0), (13.4, 0.0), (13.1, -3.2), (-13.1, -3.2)], rim, Stroke::NONE);
    // A little daisy painted on its side.
    let daisy = (-1.0f32, -14.0f32);
    for k in 0..6 {
        let a = k as f32 * std::f32::consts::TAU / 6.0;
        painter.circle_filled(at(daisy.0 + a.cos() * 2.6, daisy.1 + a.sin() * 2.6), 1.7, c(250, 250, 244));
    }
    painter.circle_filled(at(daisy.0, daisy.1), 1.5, c(250, 200, 60));
    // Its open top: the rim, the dark inside.
    let ring = |rx: f32, ry: f32| -> Vec<Pos2> { (0..24).map(|i| { let s = i as f32 / 24.0 * std::f32::consts::TAU; at(rx * s.cos(), -27.0 + ry * s.sin()) }).collect() };
    painter.add(egui::Shape::convex_polygon(ring(11.6, 2.8), rim, Stroke::NONE));
    painter.add(egui::Shape::convex_polygon(ring(9.6, 1.9), c(18, 48, 33), Stroke::NONE));
    // Pouring: drops from the rose, falling onto his soil, a little splash where they land.
    let Some(t) = v.pour else {
        return;
    };
    let rose = at(face.0 + d.0 * 1.8, face.1 + d.1 * 1.8);
    let dir = turn(d.0, d.1);
    let side = turn(n.0, n.1);
    let soil = v.foot.y - shift.y - 31.0;
    let water = Color32::from_rgba_unmultiplied(110, 182, 255, (230.0 * v.alpha) as u8);
    let mut i = 0u32;
    loop {
        let born = POUR_FROM + i as f32 * 0.04;
        if born > t.min(POUR_TO) {
            break;
        }
        let age = t - born;
        let j = ((i.wrapping_mul(2654435761) >> 16) % 1000) as f32 / 500.0 - 1.0;
        let v0 = dir * (70.0 + j.abs() * 30.0) + side * j * 2.2 * 12.0;
        let pos = rose + side * j * 3.5 + v0 * age + Vec2::new(0.0, 550.0 * age * age);
        if pos.y < soil {
            let vel = (v0 + Vec2::new(0.0, 1100.0 * age)).normalized();
            painter.line_segment([pos, pos - vel * 4.0], Stroke::new(1.3, water));
            painter.circle_filled(pos, 1.7, water);
        } else {
            // Landed: a ring spreading out for a moment.
            let hit = ((pos.y - soil) / 1100.0f32.max(1.0)).sqrt();
            let k = (hit * 6.0).min(1.0);
            if k < 1.0 {
                painter.add(egui::Shape::ellipse_stroke(Pos2::new(pos.x, soil), Vec2::new(2.0 + k * 4.0, 0.8 + k * 1.2), Stroke::new(1.0, water.gamma_multiply(1.0 - k))));
            }
        }
        i += 1;
    }
}

/// Until when Mathéo keeps quiet, kept beside the settings (a restart doesn't wake him).
fn matheo_quiet_file() -> Option<std::path::PathBuf> {
    crate::config_dir().map(|d| d.join("matheo-quiet.json"))
}

fn unix_now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Mathéo's speech bubble above his leaves (his pot's foot at `foot`), kept within `bounds`: where it is.
fn paint_matheo_tip(painter: &egui::Painter, foot: Pos2, text: &str, bounds: Rect) -> Rect {
    let ink = Color32::from_rgb(46, 64, 36);
    let fill = Color32::from_rgb(250, 248, 234);
    let edge = Stroke::new(1.5, Color32::from_rgb(104, 156, 84));
    let galley = painter.layout(text.to_owned(), FontId::proportional(14.0), ink, 230.0);
    let size = galley.size() + Vec2::new(26.0, 18.0);
    let bottom = foot.y - MATHEO_H * 1.2;
    let x = foot.x.clamp(bounds.min.x + size.x / 2.0 + 6.0, (bounds.max.x - size.x / 2.0 - 6.0).max(bounds.min.x + size.x / 2.0 + 6.0));
    let top = (bottom - size.y).max(bounds.min.y + 6.0);
    let r = Rect::from_min_size(Pos2::new(x - size.x / 2.0, top), size);
    // Its tail towards his leaves: outlined, then the bubble over it, then filled again to join them.
    let tail_x = foot.x.clamp(r.min.x + 16.0, r.max.x - 16.0);
    let point = Pos2::new(foot.x, r.max.y + 12.0);
    painter.add(egui::Shape::convex_polygon(vec![Pos2::new(tail_x - 8.0, r.max.y - 1.0), Pos2::new(tail_x + 8.0, r.max.y - 1.0), point], fill, edge));
    painter.rect(r, 10.0, fill, edge, egui::StrokeKind::Inside);
    painter.add(egui::Shape::convex_polygon(vec![Pos2::new(tail_x - 6.5, r.max.y - 3.0), Pos2::new(tail_x + 6.5, r.max.y - 3.0), Pos2::new(point.x + (tail_x - point.x) * 0.15, point.y - 2.5)], fill, Stroke::NONE));
    painter.galley(r.min + Vec2::new(13.0, 9.0), galley, ink);
    r
}

/// Where a window of ours really is, its content's top left on the screens (points); before it is
/// known, where it was asked to be.
fn window_origin(ctx: &egui::Context, asked: Pos2) -> Pos2 {
    ctx.input(|i| i.viewport().inner_rect).map_or(asked, |r| r.min)
}

/// Mathéo's height on screen (points).
const MATHEO_H: f32 = 70.0;

/// Where Mathéo was left, kept beside the settings.
fn matheo_file() -> Option<std::path::PathBuf> {
    crate::config_dir().map(|d| d.join("matheo.json"))
}

fn matheo_load() -> Option<Pos2> {
    let text = std::fs::read_to_string(matheo_file()?).ok()?;
    let v: (f32, f32) = serde_json::from_str(&text).ok()?;
    Some(Pos2::new(v.0, v.1))
}

fn matheo_save(at: Pos2) {
    if let Some(path) = matheo_file() {
        let _ = std::fs::write(path, serde_json::to_string(&(at.x, at.y)).unwrap_or_default());
    }
}

/// Mathéo's place among the screens' foot (`grounds`): where he lands (the bottom of the screen under
/// him; on the Dock, `dock`, where it is), back on a screen if left between two.
fn matheo_floor(at: Pos2, grounds: &[Rect], dock: Option<Rect>) -> (f32, f32) {
    // The screens in his column: the one he's over, else the first floor below him, else the lowest;
    // none, the nearest screen. (Screens one above the other: the first one in his column took him
    // back to the screen above.)
    let column = || grounds.iter().filter(|w| (w.min.x..w.max.x).contains(&at.x));
    let Some(ground) = column()
        .find(|w| (w.min.y..=w.max.y).contains(&at.y))
        .or_else(|| column().filter(|w| w.max.y >= at.y).min_by(|a, b| a.max.y.total_cmp(&b.max.y)))
        .or_else(|| column().max_by(|a, b| a.max.y.total_cmp(&b.max.y)))
        .or_else(|| grounds.iter().min_by(|a, b| a.center().distance(at).total_cmp(&b.center().distance(at))))
    else {
        return (at.x, at.y);
    };
    let half = MATHEO_H * 0.4;
    let x = at.x.clamp(ground.min.x + half, ground.max.x - half);
    // Standing on the Dock: his pot's middle over it.
    let on_dock = dock.filter(|d| ground.contains(d.center()) && (d.min.x..d.max.x).contains(&x));
    (x, on_dock.map_or(ground.max.y, |d| d.min.y))
}

/// The screens' foot Mathéo stands on: the Dock known (its frame, from macOS' accessibility), the very
/// bottom of each screen (on the Dock where it is, beside it at the foot); else above the Dock or the
/// task bar, all along.
fn matheo_grounds(screens: &[(Rect, Rect)], works: &[Rect], dock: Option<Rect>) -> Vec<Rect> {
    match dock {
        Some(_) if !screens.is_empty() => screens.iter().map(|(s, w)| Rect::from_min_max(w.min, Pos2::new(w.max.x, s.max.y))).collect(),
        _ => works.to_vec(),
    }
}

/// Mathéo, Riff Hero's green plant, his pot's foot at `foot`, `size` tall: a terracotta pot with a
/// face, his big leaves swaying; held, his eyes wide and his leaves flapping.
#[allow(dead_code)]
fn paint_matheo(painter: &egui::Painter, foot: Pos2, size: f32, now: f64, held: bool) {
    paint_matheo_as(painter, foot, size, now, Look { held, ..Look::default() });
}

/// How Mathéo looks: held, tipped over (radians, turning around his middle), his head spinning,
/// squashed (0 to 1) against a wall.
#[derive(Clone, Copy, Default)]
struct Look {
    held: bool,
    tilt: f32,
    dizzy: bool,
    squash: f32,
    /// Watered: his eyes closed with pleasure, drops shining on his leaves.
    glad: bool,
    /// Late in the evening: heavy eyelids, yawns, leaves hanging a little, z's floating up.
    tired: bool,
    /// Friday afternoon: sunglasses.
    shades: bool,
    /// Waving a leaf at Gérard passing by: towards which side (1 right, -1 left, 0 not).
    wave: f32,
    /// Hiding from the 10 o'clock horde (0 to 1): leaves pulled in, trembling, eyes wide.
    hide: f32,
    /// Hopping to be noticed (points off the floor).
    hop: f32,
    /// Grown, well watered: leaves more (up to 3), and a flower.
    extra: usize,
    flower: bool,
    /// Not watered (0 to 1): his leaves hang and yellow, he sulks.
    wilt: f32,
    /// Where his eyes look, the pointer near him (his units, 1 at most): towards it.
    gaze: Vec2,
    /// Which plant he is (see `skins.rs`).
    skin: Skin,
    /// Changing plant: how far down in the soil (0 out, 1 gone; below 0, popping out bigger).
    sink: f32,
    /// The season's outfit he wears: Paloma and Hélio add a touch of their own (see `skins.rs`).
    feast: Option<Season>,
}

fn paint_matheo_as(painter: &egui::Painter, foot: Pos2, size: f32, now: f64, look: Look) {
    let held = look.held;
    let u = size / 70.0;
    let ground = foot;
    let tremble = if look.hide > 0.3 { (now * 47.0).sin() as f32 * 0.9 * look.hide } else { 0.0 };
    let foot = foot - Vec2::new(-tremble, look.hop);
    let pivot = foot - Vec2::new(0.0, 20.0 * u);
    let (sin, cos) = look.tilt.sin_cos();
    let at = |x: f32, y: f32| {
        let v = Vec2::new(x * (1.0 + look.squash * 0.6), (y + 20.0) * (1.0 - look.squash)) * u;
        pivot + Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
    };
    let quad = |(x0, y0): (f32, f32), (x1, y1): (f32, f32)| vec![at(x0, y0), at(x1, y0), at(x1, y1), at(x0, y1)];
    let ink = Color32::from_rgb(60, 36, 24);
    // The shadow (upright only), the pot (wider at the top), its rim, the soil.
    if look.tilt.abs() < 0.3 {
        painter.add(egui::Shape::ellipse_filled(ground, Vec2::new(20.0, 4.0) * u * (1.0 - look.hop / 40.0), Color32::from_black_alpha(60)));
    }
    painter.add(egui::Shape::convex_polygon(vec![at(-14.0, 0.0), at(14.0, 0.0), at(18.0, -26.0), at(-18.0, -26.0)], Color32::from_rgb(196, 104, 62), Stroke::NONE));
    painter.add(egui::Shape::convex_polygon(quad((-21.0, -32.0), (21.0, -24.0)), Color32::from_rgb(214, 122, 76), Stroke::NONE));
    painter.add(egui::Shape::convex_polygon(quad((-19.0, -33.0), (19.0, -31.0)), Color32::from_rgb(92, 62, 40), Stroke::NONE));
    // His face on the pot: his eyes in crosses when dizzy, wide when held.
    let blink = (now % 4.0) < 0.12 && !held;
    for s in [-1.0, 1.0] {
        let (x, y) = (s * 6.0, -15.0);
        if look.dizzy {
            let line = Stroke::new(1.6 * u, ink);
            painter.line_segment([at(x - 2.3, y - 2.3), at(x + 2.3, y + 2.3)], line);
            painter.line_segment([at(x - 2.3, y + 2.3), at(x + 2.3, y - 2.3)], line);
        } else if look.glad {
            let arch: Vec<Pos2> = (0..=6).map(|k| { let a = k as f32 / 6.0 * std::f32::consts::PI; at(x - 2.4 * a.cos(), y - 1.6 * a.sin()) }).collect();
            painter.add(egui::Shape::line(arch, Stroke::new(1.5 * u, ink)));
        } else if held || look.hide > 0.5 {
            painter.circle_filled(at(x, y), 3.2 * u, Color32::WHITE);
            painter.circle_filled(at(x, y), 1.8 * u, Color32::from_rgb(40, 30, 24));
        } else if look.tired {
            // Heavy eyelids: a line, half an eye under it.
            painter.line_segment([at(x - 2.6, y), at(x + 2.6, y)], Stroke::new(1.5 * u, ink));
            let half: Vec<Pos2> = (0..=6).map(|k| { let a = k as f32 / 6.0 * std::f32::consts::PI; at(x + 2.0 * a.cos(), y + 1.8 * a.sin()) }).collect();
            painter.add(egui::Shape::convex_polygon(half, ink, Stroke::NONE));
        } else if blink {
            painter.line_segment([at(x - 2.0, y), at(x + 2.0, y)], Stroke::new(1.5 * u, ink));
        } else {
            // Following the pointer near him.
            painter.circle_filled(at(x + look.gaze.x * 1.4, y + look.gaze.y * 1.1), 2.0 * u, ink);
        }
    }
    if look.dizzy {
        let wave: Vec<Pos2> = (0..=8).map(|k| at(-5.0 + k as f32 * 10.0 / 8.0, -8.0 + (k as f32 * 1.6 + now as f32 * 8.0).sin() * 1.2)).collect();
        painter.add(egui::Shape::line(wave, Stroke::new(1.5 * u, ink)));
    } else if held {
        painter.circle_stroke(at(0.0, -8.0), 2.5 * u, Stroke::new(1.5 * u, ink));
    } else if look.glad {
        let grin: Vec<Pos2> = (0..=8).map(|k| { let a = k as f32 / 8.0 * std::f32::consts::PI; at(-4.5 * a.cos(), -9.5 + 3.2 * a.sin()) }).collect();
        painter.add(egui::Shape::convex_polygon(grin, Color32::from_rgb(120, 50, 40), Stroke::new(1.2 * u, ink)));
    } else if look.tired {
        // A big yawn every few seconds, else a flat mouth.
        let k = (now % 7.0) as f32;
        let open = if k < 1.6 { (k / 1.6 * std::f32::consts::PI).sin() } else { 0.0 };
        if open > 0.1 {
            let mouth: Vec<Pos2> = (0..16).map(|i| { let a = i as f32 / 16.0 * std::f32::consts::TAU; at(2.4 * open * a.cos(), -8.0 + 3.4 * open * a.sin()) }).collect();
            painter.add(egui::Shape::convex_polygon(mouth, Color32::from_rgb(110, 46, 36), Stroke::new(1.2 * u, ink)));
        } else {
            painter.line_segment([at(-3.0, -8.0), at(3.0, -8.0)], Stroke::new(1.5 * u, ink));
        }
    } else if look.wilt > 0.4 {
        let frown: Vec<Pos2> = (0..=6).map(|k| at(-4.0 + k as f32 * 8.0 / 6.0, -7.0 - (1.0 - ((k as f32 - 3.0) / 3.0).powi(2)) * 2.0)).collect();
        painter.add(egui::Shape::line(frown, Stroke::new(1.5 * u, ink)));
    } else {
        let smile: Vec<Pos2> = (0..=6).map(|k| at(-4.0 + k as f32 * 8.0 / 6.0, -9.0 + ((k as f32 - 3.0) / 3.0).powi(2) * -2.0 + 2.0)).collect();
        painter.add(egui::Shape::line(smile, Stroke::new(1.5 * u, ink)));
    }
    painter.circle_filled(at(-10.0, -11.0), 2.2 * u, Color32::from_rgba_unmultiplied(255, 140, 140, 90));
    painter.circle_filled(at(10.0, -11.0), 2.2 * u, Color32::from_rgba_unmultiplied(255, 140, 140, 90));
    // Friday afternoon: black sunglasses, a glint on each lens, the arms going round the pot.
    if look.shades {
        let frame = Color32::from_rgb(12, 12, 16);
        painter.line_segment([at(-10.4, -16.4), at(-17.6, -18.0)], Stroke::new(1.4 * u, frame));
        painter.line_segment([at(10.4, -16.4), at(17.6, -18.0)], Stroke::new(1.4 * u, frame));
        let bridge: Vec<Pos2> = (0..=6).map(|k| { let a = k as f32 / 6.0 * std::f32::consts::PI; at(-1.6 * a.cos(), -16.6 - 0.9 * a.sin()) }).collect();
        painter.add(egui::Shape::line(bridge, Stroke::new(1.3 * u, frame)));
        for x in [-6.0f32, 6.0] {
            let lens: Vec<Pos2> = (0..24)
                .map(|i| {
                    let a = i as f32 / 24.0 * std::f32::consts::TAU;
                    at(x + 4.9 * a.cos(), -15.0 + (3.9 * a.sin()).max(-2.6))
                })
                .collect();
            painter.add(egui::Shape::convex_polygon(lens, Color32::from_rgb(28, 30, 38), Stroke::new(1.1 * u, frame)));
            painter.line_segment([at(x - 3.0, -15.6), at(x - 1.2, -17.0)], Stroke::new(1.0 * u, Color32::from_rgba_unmultiplied(255, 255, 255, 150)));
            painter.circle_filled(at(x + 2.6, -13.2), 0.6 * u, Color32::from_rgba_unmultiplied(255, 255, 255, 110));
        }
    }
    // Changing plant (see skins.rs): sinking into the soil, popping out.
    let grown = 1.0 - look.sink;
    let u = u * grown;
    let at = |x: f32, y: f32| at(x * grown, -32.0 + (y + 32.0) * grown);
    if grown > 0.02 {
        if look.skin != Skin::Classic {
            paint_skin(painter, look, now, u, &at);
        } else {
            // His leaves: stems from the soil, big leaves swaying (flapping, held).
            // Hanging when thirsty (and yellowing), a little when tired.
            let speed = if held { 9.0 } else if look.tired { 0.6 } else { 1.3 };
            let droop = look.wilt.max(if look.tired { 0.3 } else { 0.0 });
            let fade = |c: Color32, to: (u8, u8, u8)| {
                let w = look.wilt.clamp(0.0, 1.0) * 0.8;
                let m = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * w) as u8;
                Color32::from_rgb(m(c.r(), to.0), m(c.g(), to.1), m(c.b(), to.2))
            };
            // Grown: younger leaves between the first ones, drawn behind them.
            let leaves: Vec<(f32, f32)> = [(-0.75f32, 26.0f32), (0.35, 33.0), (0.85, 25.0)].into_iter().take(look.extra.min(3)).chain([(-1.0, 30.0), (-0.45, 38.0), (0.1, 42.0), (0.6, 36.0), (1.1, 28.0)]).collect();
            let first = leaves.len() - 5;
            // Waving at Gérard: the outer leaf on his side.
            let waving = if look.wave > 0.0 { Some(leaves.len() - 1) } else if look.wave < 0.0 { Some(first) } else { None };
            for (k, &(angle, len)) in leaves.iter().enumerate() {
                let sway = (now * speed + k as f64 * 0.9).sin() as f32 * if held { 0.25 } else { 0.08 };
                let len = len * (1.0 - 0.15 * look.wilt) * (1.0 - 0.7 * look.hide);
                let bend = if angle.abs() < 0.3 { droop * 0.6 } else { 0.0 };
                let mut a = -std::f32::consts::FRAC_PI_2 + angle * 0.8 * (1.0 + droop * 0.9) * (1.0 - 0.6 * look.hide) + bend + sway + look.tilt;
                if waving == Some(k) {
                    a = -std::f32::consts::FRAC_PI_2 + look.wave * (0.75 + (now * 12.0).sin() as f32 * 0.35) + look.tilt;
                }
                let base = at(angle * 6.0, -32.0);
                let tip = base + Vec2::angled(a) * len * u;
                let mid = base.lerp(tip, 0.55);
                painter.line_segment([base, mid], Stroke::new(2.0 * u, fade(Color32::from_rgb(60, 120, 50), (120, 100, 50))));
                let dir = Vec2::angled(a);
                let side = Vec2::new(-dir.y, dir.x);
                let leaf: Vec<Pos2> = (0..=12)
                    .map(|i| {
                        let f = i as f32 / 12.0;
                        let w = (f * std::f32::consts::PI).sin() * 9.0 * u;
                        mid + dir * f * (len * 0.6) * u + side * w
                    })
                    .chain((0..=12).rev().map(|i| {
                        let f = i as f32 / 12.0;
                        let w = (f * std::f32::consts::PI).sin() * 9.0 * u;
                        mid + dir * f * (len * 0.6) * u - side * w
                    }))
                    .collect();
                let young = k < first;
                let green = if young { Color32::from_rgb(118, 196, 96) } else { [Color32::from_rgb(72, 160, 70), Color32::from_rgb(90, 180, 80)][k % 2] };
                painter.add(egui::Shape::convex_polygon(leaf, fade(green, (186, 160, 72)), Stroke::new(1.0, fade(Color32::from_rgb(40, 100, 40), (120, 96, 44)))));
                painter.line_segment([mid, mid + dir * len * 0.55 * u], Stroke::new(1.0, fade(Color32::from_rgb(140, 210, 120), (214, 196, 120))));
                if look.glad {
                    // Drops shining on his leaves.
                    let drop = mid + dir * len * (0.25 + k as f32 * 0.06) * u + side * 3.0 * u;
                    painter.circle_filled(drop, 1.9 * u, Color32::from_rgba_unmultiplied(150, 205, 255, 220));
                    painter.circle_filled(drop - Vec2::new(0.6, 0.6) * u, 0.7 * u, Color32::WHITE);
                }
            }
            // Watered well today: a pink flower on a stem of its own, nodding (hanging when thirsty).
            if look.flower {
                let sway = (now * speed * 0.8).sin() as f32 * 0.06;
                let a = -std::f32::consts::FRAC_PI_2 + 0.12 + droop * 0.9 + sway + look.tilt;
                let base = at(1.0, -32.0);
                let top = base + Vec2::angled(a) * 50.0 * u * (1.0 - 0.6 * look.hide);
                let bend = base.lerp(top, 0.5) + Vec2::angled(a + 1.4) * 3.0 * u;
                let stem: Vec<Pos2> = (0..=10).map(|i| { let t = i as f32 / 10.0; let m = 1.0 - t; (base.to_vec2() * m * m + bend.to_vec2() * 2.0 * m * t + top.to_vec2() * t * t).to_pos2() }).collect();
                painter.add(egui::Shape::line(stem, Stroke::new(1.6 * u, fade(Color32::from_rgb(60, 130, 50), (120, 100, 50)))));
                let petal = fade(Color32::from_rgb(244, 128, 170), (196, 150, 130));
                for i in 0..6 {
                    let p = top + Vec2::angled(i as f32 * std::f32::consts::TAU / 6.0 + now as f32 * 0.2) * 4.2 * u;
                    painter.circle_filled(p, 3.4 * u, petal);
                    painter.circle_stroke(p, 3.4 * u, Stroke::new(0.6 * u, Color32::from_rgb(200, 80, 120)));
                }
                painter.circle_filled(top, 2.8 * u, Color32::from_rgb(255, 206, 64));
                painter.circle_filled(top - Vec2::new(0.8, 0.8) * u, 0.9 * u, Color32::from_rgb(255, 240, 170));
            }
        }
    }
    // Tired: z's floating up from his leaves.
    if look.tired && !look.dizzy && !held {
        for k in 0..3 {
            let f = ((now as f32 / 2.4) + k as f32 / 3.0).fract();
            let p = at(14.0 + f * 16.0 + (f * 9.0).sin() * 2.0, -60.0 - f * 34.0);
            let alpha = (f * std::f32::consts::PI).sin();
            painter.text(p, egui::Align2::CENTER_CENTER, "z", FontId::proportional((8.0 + f * 8.0) * u), Color32::from_rgba_unmultiplied(90, 110, 170, (230.0 * alpha) as u8));
        }
    }
    // Stars turning over his leaves.
    if look.dizzy {
        for k in 0..3 {
            let turn = now as f32 * 5.0 + k as f32 * std::f32::consts::TAU / 3.0;
            let c = at(0.0, -82.0) + Vec2::new(turn.cos() * 17.0, turn.sin() * 5.0) * u;
            let star: Vec<Pos2> = (0..10).map(|i| c + Vec2::angled(i as f32 * std::f32::consts::PI / 5.0 - std::f32::consts::FRAC_PI_2) * if i % 2 == 0 { 4.5 } else { 2.0 } * u).collect();
            painter.add(egui::Shape::Path(egui::epaint::PathShape::closed_line(star.clone(), Stroke::new(1.0 * u, Color32::from_rgb(200, 150, 20)))));
            painter.circle_filled(c, 2.0 * u, Color32::from_rgb(255, 214, 60));
        }
    }
}


/// What his menu does.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Choice {
    /// His version, shown at the top (nothing to do).
    Version,
    Tips,
    Quiet,
    Autostart,
    Lang,
    Page,
    Install,
    /// Look for a newer version now.
    Check,
    /// His journal opened (to tell what went wrong).
    Journal,
    Dock,
    Quit,
    /// The page of his time reminders, and back.
    Hours,
    /// Your name, in its window.
    Name,
    /// Debug builds: the demo page, and one of its antics or words.
    Demo,
    Play(Demo),
    Back,
    HoursOn,
    /// One of his time reminders: a click on its − or + moves it.
    Hour(usize),
    HoursReset,
    /// The page of the plants he can be, and one of them.
    Skins,
    Skin(Skin),
}

/// Debug builds, for the smoke test (scripts/smoke/): the pointer read from the file
/// `MATHEO_TEST_POINTER` names (`x y`, then `left` and/or `right` while pressed), where the runner's
/// own pointer can't be moved.
fn test_pointer() -> Option<Mouse> {
    if !cfg!(debug_assertions) {
        return None;
    }
    let text = std::fs::read_to_string(std::env::var_os("MATHEO_TEST_POINTER")?).ok()?;
    let mut words = text.split_whitespace();
    let (x, y) = (words.next()?.parse().ok()?, words.next()?.parse().ok()?);
    let rest: Vec<&str> = words.collect();
    Some(Mouse { at: Some(Pos2::new(x, y)), pressed: rest.contains(&"left"), right: rest.contains(&"right") })
}

/// The title of the window following the pointer (the others: « Mathéo · n », n their screen).
const MAIN_TITLE: &str = "Mathéo";

/// Frames during which the windows are cleared after a change (Windows applies it a little later).
const GLASS: u8 = 4;

/// Windows: clears behind the drawing of each window of ours (see `crate::glass`).
fn clear_glass() {
    #[cfg(windows)]
    crate::glass::clear();
}

const MENU_W: f32 = 240.0;
/// The page of his plants, beside him: that far from his middle (points).
const SKINS_AWAY: f32 = 60.0;
/// A time reminder's row: its − and + buttons, from the menu's right edge (their left side).
const HOUR_MINUS: f32 = 108.0;
const HOUR_PLUS: f32 = 32.0;
const ROW_H: f32 = 30.0;

/// His menu's rows: what each says, whether it's a switch (and on), what it does.
/// On his time reminders' page, a reminder's row says its time after a tab (drawn on the right,
/// between − and +).
fn menu_rows(settings: &Settings, quiet: bool, page: Page) -> Vec<(String, Option<bool>, Choice)> {
    let t = settings.lang.strings();
    if page == Page::Demo {
        let mut rows = vec![(t.menu_back.to_owned(), None, Choice::Back)];
        rows.extend(DEMOS.iter().map(|(label, demo)| ((*label).to_owned(), None, Choice::Play(*demo))));
        return rows;
    }
    if page == Page::Skins {
        let mut rows = vec![(t.menu_back.to_owned(), None, Choice::Back)];
        rows.extend(Skin::ALL.iter().map(|&s| (s.label(settings.lang), Some(settings.skin == s), Choice::Skin(s))));
        return rows;
    }
    if page == Page::Hours {
        let h = settings.hours;
        let names = [t.menu_hour_morning, t.menu_hour_lunch, t.menu_hour_afternoon, t.menu_hour_home];
        let mut rows = vec![(t.menu_back.to_owned(), None, Choice::Back), (t.menu_hours_on.to_owned(), Some(h.on), Choice::HoursOn)];
        rows.extend(names.iter().enumerate().map(|(k, name)| (format!("{name}\t{:2}:{:02}", h.at[k] / 60, h.at[k] % 60), None, Choice::Hour(k))));
        rows.push((t.menu_hours_reset.to_owned(), None, Choice::HoursReset));
        return rows;
    }
    // In groups (a line between them, see `starts_group`): him and you, his advice, the computer,
    // the app, quitting.
    let mut rows = vec![
        (t.menu_version.replace("{v}", crate::update::VERSION), None, Choice::Version),
        (t.menu_skin.to_owned(), None, Choice::Skins),
        (t.menu_name.to_owned(), None, Choice::Name),
        (t.menu_tips_on.to_owned(), Some(settings.tips), Choice::Tips),
        (t.menu_hours.to_owned(), None, Choice::Hours),
        ((if quiet { t.menu_speak } else { t.menu_quiet }).to_owned(), None, Choice::Quiet),
        (t.menu_autostart.to_owned(), Some(settings.autostart), Choice::Autostart),
        (t.menu_lang.to_owned(), None, Choice::Lang),
    ];
    if !crate::access::allowed() {
        rows.push((t.menu_dock.to_owned(), None, Choice::Dock));
    }
    match crate::update::state() {
        crate::update::State::Available(v) => rows.push((t.menu_install.replace("{v}", &v), None, Choice::Install)),
        crate::update::State::Installing(_) | crate::update::State::Installed(_) => rows.push((t.menu_installing.to_owned(), None, Choice::Page)),
        _ => rows.push((t.menu_check.to_owned(), None, Choice::Check)),
    }
    rows.push((t.menu_page.to_owned(), None, Choice::Page));
    rows.push((t.menu_journal.to_owned(), None, Choice::Journal));
    if cfg!(debug_assertions) {
        rows.push(("Démo…".to_owned(), None, Choice::Demo));
    }
    rows.push((t.menu_quit.to_owned(), None, Choice::Quit));
    rows
}

#[cfg(test)]
mod menu_tests {
    use super::{Choice, starts_group};

    #[test]
    fn his_menu_goes_in_groups() {
        let rows: Vec<(String, Option<bool>, Choice)> = [Choice::Version, Choice::Skins, Choice::Name, Choice::Tips, Choice::Hours, Choice::Quiet, Choice::Autostart, Choice::Lang, Choice::Dock, Choice::Check, Choice::Page, Choice::Journal, Choice::Quit]
            .into_iter()
            .map(|c| (String::new(), None, c))
            .collect();
        let lines: Vec<Choice> = (0..rows.len()).filter(|&i| starts_group(&rows, i)).map(|i| rows[i].2).collect();
        assert_eq!(lines, [Choice::Tips, Choice::Autostart, Choice::Check, Choice::Quit]);
        // Without the Dock's row: the app right after the language.
        let rows: Vec<_> = rows.into_iter().filter(|r| r.2 != Choice::Dock).collect();
        assert!(starts_group(&rows, rows.iter().position(|r| r.2 == Choice::Check).unwrap()));
    }
}

/// His menu's pages.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Page {
    Main,
    /// His time reminders.
    Hours,
    /// The plants he can be.
    Skins,
    /// Debug builds: each of his antics and words at a click.
    Demo,
}

/// Debug builds: what the demo page plays.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Demo {
    /// Up high, coming down under his parachute.
    Chute,
    /// Up high, falling hard.
    Land,
    /// Sliding along the floor.
    Slide,
    /// His hello, the very first time.
    Hello,
    /// His advice (the watering can with water).
    Advice,
    /// Popping out of the bottom of the screen, as he may arrive at the start.
    Pop,
    Say(Own),
    /// Everything, one after another (see `TOUR`).
    Tour,
}

/// Debug builds: a step of the tour, besides his demos: how he looks (left as he is after it).
#[derive(Clone, Copy, PartialEq, Debug)]
enum Step {
    Play(Demo),
    Wilt(f32),
    Tired,
    Shades,
    Dizzy,
    Glad,
    Grown,
    Flower,
    Skin(Skin),
    /// Changing plant (from the next one to his own).
    Swap,
    /// The watering can pouring (counted as a watering).
    Water,
    /// A season's outfit, on one of his plants (the first time, its words said too).
    Season(Season, Skin, bool),
}

/// Debug builds: the tour, `MATHEO_TOUR=1` at the start or « Tout voir » on the demo page: his
/// demos, then his looks, his plants, his season outfits, each named at the top of his screen.
fn tour_steps(lang: crate::i18n::Lang) -> Vec<(String, Step)> {
    let mut steps: Vec<(String, Step)> = DEMOS.iter().filter(|(_, d)| *d != Demo::Tour).map(|(label, d)| ((*label).to_owned(), Step::Play(*d))).collect();
    steps.extend([
        ("L’arrosage", Step::Water),
        ("Content (arrosé)", Step::Glad),
        ("Soif (1/3)", Step::Wilt(1.0 / 3.0)),
        ("Soif (2/3)", Step::Wilt(2.0 / 3.0)),
        ("Dépéri (3/3)", Step::Wilt(1.0)),
        ("La tête qui tourne", Step::Dizzy),
        ("Le soir, fatigué", Step::Tired),
        ("Vendredi aprèm, lunettes de soleil", Step::Shades),
        ("Grandi (des feuilles en plus)", Step::Grown),
        ("La fleur du jour (3 arrosages)", Step::Flower),
        ("Changement d’apparence", Step::Swap),
    ].map(|(l, st)| (l.to_owned(), st)));
    steps.extend(Skin::ALL.iter().map(|&k| (format!("Apparence : {}", k.label(lang)), Step::Skin(k))));
    for season in Season::ALL {
        for (i, &skin) in Skin::ALL.iter().enumerate() {
            steps.push((format!("Saison : {} · {}", season.label(), skin.label(lang)), Step::Season(season, skin, i == 0)));
        }
    }
    steps
}

/// The tour started from a step: the one before it, as if it had begun that long ago (none lasts less).
const TOUR_BEFORE: Duration = Duration::from_secs(4);

/// How long a step of the tour shows.
fn tour_lasts(step: Step) -> Duration {
    match step {
        Step::Play(Demo::Say(Own::Breathe)) => Own::Breathe.lasts(),
        Step::Play(Demo::Say(Own::News) | Demo::Hello | Demo::Chute | Demo::Land) => Duration::from_secs(9),
        Step::Play(_) | Step::Water => Duration::from_secs(6),
        Step::Season(.., true) => Duration::from_secs(8),
        _ => Duration::from_secs(4),
    }
}

/// Debug builds: `MATHEO_TOUR=1`, the tour at the start; `MATHEO_TOUR=<word>`, from the first step
/// whose name has it (`MATHEO_TOUR=saison`, the season outfits).
fn tour_asked() -> Option<String> {
    cfg!(debug_assertions).then(|| std::env::var("MATHEO_TOUR").ok()).flatten()
}

/// The step the tour starts from (`tour_asked`).
fn tour_from(lang: crate::i18n::Lang) -> Option<usize> {
    let word = tour_asked()?.to_lowercase();
    tour_steps(lang).iter().position(|(label, _)| label.to_lowercase().contains(&word)).and_then(|k| k.checked_sub(1))
}

const DEMOS: [(&str, Demo); 26] = [
    ("Tout voir, l’un après l’autre", Demo::Tour),
    ("Parachute", Demo::Chute),
    ("Arrivée par le bas", Demo::Pop),
    ("Chute et atterrissage", Demo::Land),
    ("Glissade", Demo::Slide),
    ("Présentation (1er lancement)", Demo::Hello),
    ("Demande du prénom", Demo::Say(Own::AskName)),
    ("Prénom retenu", Demo::Say(Own::Named)),
    ("Nouveautés de la version", Demo::Say(Own::News)),
    ("Un conseil", Demo::Advice),
    ("Pause du matin", Demo::Say(Own::Clock(0))),
    ("Repas", Demo::Say(Own::Clock(1))),
    ("Pause de l’aprèm", Demo::Say(Own::Clock(2))),
    ("Fin de journée", Demo::Say(Own::Clock(3))),
    ("Lundi matin", Demo::Say(Own::Monday)),
    ("La lampe (soir d’hiver)", Demo::Say(Own::Lamp)),
    ("Travail le week-end", Demo::Say(Own::Weekend)),
    ("Les yeux", Demo::Say(Own::Eyes)),
    ("La posture", Demo::Say(Own::Posture)),
    ("Respirer ensemble", Demo::Say(Own::Breathe)),
    ("Il fait chaud", Demo::Say(Own::Hot)),
    ("Retour après une absence", Demo::Say(Own::Back)),
    ("Les verres d’eau du jour (le soir)", Demo::Say(Own::Glasses)),
    ("Ça va aujourd’hui ? (les pouces)", Demo::Say(Own::Mood)),
    ("Réponse au pouce levé", Demo::Say(Own::MoodUp)),
    ("Réponse au pouce baissé", Demo::Say(Own::MoodDown)),
];

/// Debug builds: the tour's step named at the top of his screen.
fn paint_tour_caption(painter: &egui::Painter, screen: Rect, caption: &str) {
    let galley = painter.layout_no_wrap(caption.to_owned(), FontId::proportional(20.0), Color32::WHITE);
    let r = Rect::from_center_size(Pos2::new(screen.center().x, screen.min.y + 70.0), galley.size() + Vec2::new(36.0, 18.0));
    painter.rect_filled(r, 12.0, Color32::from_rgba_unmultiplied(30, 32, 40, 220));
    painter.galley(r.center() - galley.size() / 2.0, galley, Color32::WHITE);
}

/// Where his menu shows (screen points): from the click, kept within the screen.
fn menu_rect(at: Pos2, rows: usize, screen: Rect) -> Rect {
    let size = Vec2::new(MENU_W, rows as f32 * ROW_H + 12.0);
    let mut min = at + Vec2::new(-size.x * 0.5, -size.y - 12.0);
    min.x = min.x.clamp(screen.min.x + 6.0, (screen.max.x - size.x - 6.0).max(screen.min.x + 6.0));
    min.y = min.y.max(screen.min.y + 6.0);
    Rect::from_min_size(min, size)
}

/// Whether row `i` of his main menu starts a group (a thin line over it): his advice, the computer,
/// the app (after the language, or the Dock), quitting.
fn starts_group(rows: &[(String, Option<bool>, Choice)], i: usize) -> bool {
    let Some((_, _, choice)) = rows.get(i) else { return false };
    let after = i.checked_sub(1).and_then(|k| rows.get(k)).map(|r| r.2);
    matches!(choice, Choice::Tips | Choice::Autostart | Choice::Quit) || (matches!(after, Some(Choice::Lang | Choice::Dock)) && *choice != Choice::Dock)
}

/// His menu: a light card, the row under the pointer lit, switches on the right, a thin line
/// between its groups.
fn paint_menu(painter: &egui::Painter, rect: Rect, rows: &[(String, Option<bool>, Choice)], hover: Option<usize>) {
    let ink = Color32::from_rgb(46, 64, 36);
    let green = Color32::from_rgb(76, 160, 70);
    painter.rect_filled(rect.translate(Vec2::new(0.0, 3.0)), 12.0, Color32::from_black_alpha(50));
    painter.rect(rect, 12.0, Color32::from_rgb(250, 248, 234), Stroke::new(1.5, Color32::from_rgb(104, 156, 84)), egui::StrokeKind::Inside);
    for (i, (label, switch, choice)) in rows.iter().enumerate() {
        let row = Rect::from_min_size(rect.min + Vec2::new(6.0, 6.0 + i as f32 * ROW_H), Vec2::new(rect.width() - 12.0, ROW_H));
        if starts_group(rows, i) {
            painter.line_segment([row.left_top() + Vec2::new(10.0, 0.0), row.right_top() - Vec2::new(10.0, 0.0)], Stroke::new(1.0, Color32::from_rgb(214, 222, 198)));
        }
        if hover == Some(i) && *choice != Choice::Version {
            painter.rect_filled(row, 8.0, Color32::from_rgb(222, 238, 206));
        }
        let color = match choice {
            Choice::Version => Color32::from_rgb(130, 140, 120),
            Choice::Quit => Color32::from_rgb(170, 60, 50),
            Choice::Install => Color32::from_rgb(40, 120, 40),
            _ => ink,
        };
        let (label, time) = label.split_once('\t').map_or((label.as_str(), None), |(l, t)| (l, Some(t)));
        painter.text(row.left_center() + Vec2::new(10.0, 0.0), egui::Align2::LEFT_CENTER, label, FontId::proportional(14.0), color);
        if let Some(time) = time {
            // − 10:00 +, the buttons lit under the pointer.
            let right = row.right_center();
            painter.text(right - Vec2::new(HOUR_PLUS - 6.0 + 30.0, 0.0), egui::Align2::CENTER_CENTER, time, FontId::monospace(13.0), ink);
            for (x, sign) in [(HOUR_MINUS, "−"), (HOUR_PLUS, "+")] {
                let c = right - Vec2::new(x - 13.0, 0.0);
                painter.circle_filled(c, 10.0, Color32::from_rgb(222, 238, 206));
                painter.text(c, egui::Align2::CENTER_CENTER, sign, FontId::proportional(15.0), green);
            }
        }
        if let Some(on) = switch {
            // A small switch: a track, its knob on the right when on.
            let track = Rect::from_center_size(row.right_center() - Vec2::new(24.0, 0.0), Vec2::new(30.0, 16.0));
            painter.rect_filled(track, 8.0, if *on { green } else { Color32::from_rgb(200, 200, 190) });
            let knob = if *on { track.right_center() - Vec2::new(8.0, 0.0) } else { track.left_center() + Vec2::new(8.0, 0.0) };
            painter.circle_filled(knob, 6.0, Color32::WHITE);
        }
    }
}

pub(crate) fn run(mut settings: Settings) {
    // Without a compositor, the transparent window would hide the screen in black: nothing then.
    #[cfg(all(unix, not(target_os = "macos")))]
    if !os::composited() {
        crate::log::line("pas de compositeur (ou pas de serveur X : DISPLAY ?), rien à afficher");
        return;
    }
    let mouse = Arc::new(Mutex::new(Mouse::default()));
    let ctx_slot: Arc<Mutex<Option<egui::Context>>> = Arc::new(Mutex::new(None));
    let hot: Arc<Mutex<Option<Rect>>> = Arc::new(Mutex::new(None));
    // The pointer, read about 60 times a second; a click, or a move near him, draws again (a move
    // elsewhere waits for the next frame: drawing each screen's window 60 times a second as the
    // pointer goes about cost a third of a core).
    {
        let (mouse, ctx_slot, hot) = (mouse.clone(), ctx_slot.clone(), hot.clone());
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(16));
                let now = test_pointer().unwrap_or_else(os::mouse);
                let was = mouse.lock().map(|mut m| std::mem::replace(&mut *m, now)).unwrap_or(now);
                let near = |p: Option<Pos2>| hot.lock().ok().is_none_or(|h| h.is_none_or(|r| p.is_some_and(|p| r.contains(p))));
                let changed = was.pressed != now.pressed || was.right != now.right || (was.at != now.at && (near(was.at) || near(now.at)));
                if changed && let Some(ctx) = ctx_slot.lock().ok().and_then(|c| c.clone()) {
                    ctx.request_repaint();
                }
            }
        });
    }

    // The main screen (the one at the screens' origin), else the first.
    let start = os::screens().iter().map(|s| s.0).find(|s| s.contains(Pos2::new(1.0, 1.0))).or_else(|| os::screens().first().map(|s| s.0)).unwrap_or(Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0)));
    // egui's own window: a point, still, on the main screen, drawing nothing. Each screen has a window
    // of its own, made once at its size and never moved (see `ui`).
    let viewport = egui::ViewportBuilder::default()
        .with_title(MAIN_TITLE)
        .with_decorations(false)
        .with_transparent(true)
        .with_mouse_passthrough(true)
        .with_window_level(egui::WindowLevel::AlwaysOnTop)
        .with_taskbar(false)
        .with_active(false)
        .with_has_shadow(false)
        .with_resizable(false)
        .with_position(start.min)
        .with_inner_size([1.0, 1.0]);
    let mut options = eframe::NativeOptions { viewport, persist_window: false, ..Default::default() };
    // Out of the Dock and of ⌘ Tab.
    #[cfg(target_os = "macos")]
    {
        options.event_loop_builder = Some(Box::new(|builder| {
            use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
            builder.with_activation_policy(ActivationPolicy::Accessory);
        }));
    }
    // Windows: a transparent window needs DirectX 12 through DirectComposition.
    #[cfg(windows)]
    {
        let mut setup = eframe::egui_wgpu::WgpuSetupCreateNew::without_display_handle();
        setup.instance_descriptor.backends = eframe::wgpu::Backends::DX12;
        setup.instance_descriptor.backend_options.dx12.presentation_system = eframe::wgpu::Dx12SwapchainKind::DxgiFromVisual;
        options.wgpu_options.wgpu_setup = setup.into();
    }
    // Linux: OpenGL. Through Vulkan, the window shows but nothing drawn in it does (seen with Mesa
    // on X11, the compositor getting no alpha). WGPU_BACKEND still picks another one.
    #[cfg(all(unix, not(target_os = "macos")))]
    if std::env::var_os("WGPU_BACKEND").is_none() {
        let mut setup = eframe::egui_wgpu::WgpuSetupCreateNew::without_display_handle();
        setup.instance_descriptor.backends = eframe::wgpu::Backends::GL;
        options.wgpu_options.wgpu_setup = setup.into();
    }
    let first = !matheo_file().is_some_and(|f| f.exists());
    // A new version (not his very first start): he tells what's new in it. Debug builds:
    // `MATHEO_NEWS=1` to see it.
    let news = (!first && settings.seen.as_deref() != Some(crate::update::VERSION)) || (cfg!(debug_assertions) && std::env::var_os("MATHEO_NEWS").is_some());
    if settings.seen.as_deref() != Some(crate::update::VERSION) {
        settings.seen = Some(crate::update::VERSION.to_owned());
        settings.save();
    }
    crate::log::line(&format!("écrans {:?}, souris {:?}, fenêtre sur {start:?}", os::screens(), os::mouse().at));
    let result = eframe::run_native(
        "matheo",
        options,
        Box::new(move |cc| {
            if let Some(render) = &cc.wgpu_render_state {
                let info = render.adapter.get_info();
                crate::log::line(&format!("carte graphique : {} ({:?}, {:?}, pilote {} {})", info.name, info.backend, info.device_type, info.driver, info.driver_info));
            }
            os::tune(cc);
            if let Ok(mut slot) = ctx_slot.lock() {
                *slot = Some(cc.egui_ctx.clone());
            }
            let work = os::screen_at(start.center()).map_or(start, |(_, work)| work);
            let mut matheo = Plant::default();
            // The very first time: hello, and how to reach his menu; on macOS, Accessibility asked
            // for (to stand on the Dock).
            if first {
                let dock = cfg!(target_os = "macos") && !crate::access::allowed();
                matheo.tip = Some((if dock { Said::Dock } else { Said::Hi }, Instant::now()));
                // Not from a debug build (macOS would list it, not the installed app).
                if dock && !cfg!(debug_assertions) {
                    crate::access::ask();
                }
            }
            // The tour: from its start (a few seconds after his), or from the step asked.
            let tour = tour_asked().map(|_| match tour_from(settings.lang) {
                // After his entrance.
                Some(k) => (Some(k), Instant::now() + Duration::from_secs(3) - TOUR_BEFORE),
                None => (None, Instant::now()),
            });
            Ok(Box::new(Overlay { mouse, hot: hot.clone(), settings, screen: start, work, pressed: false, right: false, last: None, full: (Vec::new(), None), matheo, menu: None, told_newer: false, told_failed: false, leaving: None, others: Default::default(), season: Season::now(), antics: Antics::default(), care: Care::new(first, news), page: Page::Main, naming: None, mood: None, logged: String::new(), glass: GLASS, gaze: Vec2::ZERO, swap: None, can_counted: None, can_away: false, entered: false, pop: None, tour }))
        }),
    );
    match result {
        Err(e) => crate::log::line(&format!("arrêté sur une erreur : {e}")),
        Ok(()) => crate::log::line("fenêtre fermée"),
    }
}

impl Overlay {
    /// Debug builds: plays one of his antics or words (see `DEMOS`).
    fn play(&mut self, demo: Demo) {
        let m = &mut self.matheo;
        let Some(at) = m.at else { return };
        let now = Instant::now();
        let top = os::screen_at(at).map_or(self.screen.min.y, |(_, work)| work.min.y) + MATHEO_H + 80.0;
        match demo {
            Demo::Chute | Demo::Land => {
                m.at = Some(Pos2::new(at.x, (at.y - 600.0).max(top)));
                (m.vx, m.vy, m.spin) = (0.0, 0.0, 0.0);
                self.antics.drawn = true;
                if demo == Demo::Chute {
                    self.antics.chute = Some(Chute::Open(now, 0.0));
                }
            }
            Demo::Slide => {
                let screen = os::screen_at(at).map_or(self.screen, |(screen, _)| screen);
                m.vx = if at.x > screen.center().x { -1500.0 } else { 1500.0 };
            }
            Demo::Hello => m.tip = Some((if cfg!(target_os = "macos") && !crate::access::allowed() { Said::Dock } else { Said::Hi }, now)),
            Demo::Advice => m.advise(),
            Demo::Pop => self.pop = Some(now),
            Demo::Say(own) => m.tip = Some((Said::Own(own), now)),
            Demo::Tour => self.tour = Some((None, now - Duration::from_secs(10))),
        }
    }

    /// Debug builds: the tour's next step when due (a few seconds after the start first); each
    /// starts with nothing over him. Its step and its name, while it goes on.
    fn tour_tick(&mut self) -> Option<(usize, usize, String, Step)> {
        let (step, since) = self.tour?;
        let steps = tour_steps(self.settings.lang);
        let due = match step {
            None => since.elapsed() >= Duration::from_secs(4),
            Some(k) => steps.get(k).is_none_or(|(_, st)| since.elapsed() >= tour_lasts(*st)),
        };
        let k = match (step, due) {
            (Some(k), false) => k,
            _ => {
                let k = step.map_or(0, |k| k + 1);
                let Some(&(_, st)) = steps.get(k) else {
                    self.tour = None;
                    self.matheo.tip = None;
                    crate::log::line("visite : finie");
                    return None;
                };
                let now = Instant::now();
                self.tour = Some((Some(k), now));
                crate::log::line(&format!("visite : étape {}/{} · {}", k + 1, steps.len(), steps[k].0));
                let m = &mut self.matheo;
                m.tip = None;
                m.can = None;
                match st {
                    Step::Play(demo) => self.play(demo),
                    Step::Water => self.matheo.can = Some((now, Some(now))),
                    Step::Dizzy => self.matheo.dizzy = Some(now + tour_lasts(st)),
                    Step::Glad => self.matheo.glad = Some(now + tour_lasts(st)),
                    Step::Season(k, _, true) => self.matheo.tip = Some((Said::Own(Own::Feast(k, false)), now)),
                    Step::Swap => self.swap = Some((Skin::ALL[(Skin::ALL.iter().position(|&k| k == self.settings.skin).unwrap_or(0) + 1) % Skin::ALL.len()], now)),
                    _ => {}
                }
                k
            }
        };
        let (label, st) = steps.get(k)?.clone();
        Some((k + 1, steps.len(), label, st))
    }

    /// Asks your name: his bubble with the field to type it in.
    fn ask_name(&mut self) {
        self.matheo.tip = Some((Said::Own(Own::AskName), Instant::now()));
        self.naming = Some(Naming { typed: self.settings.name.clone().unwrap_or_default(), keys: None, rects: None });
    }

    /// Your name typed (kept, he's glad), or skipped (kept empty: not asked again, his menu asks).
    fn named(&mut self, name: Option<String>) {
        let name = name.map(|n| n.trim().to_owned()).filter(|n| !n.is_empty());
        self.matheo.tip = name.is_some().then(|| (Said::Own(Own::Named), Instant::now()));
        self.settings.name = Some(name.unwrap_or_default());
        self.settings.save();
        if self.naming.take().is_some_and(|n| n.keys.is_some()) {
            release_keys();
        }
    }

    /// Keys typed in his name bubble: letters added, Backspace takes the last off, Enter keeps it,
    /// Escape skips.
    fn type_name(&mut self, events: &[egui::Event]) {
        for event in events {
            let Some(n) = &mut self.naming else { return };
            match event {
                egui::Event::Text(text) => {
                    for c in text.chars().filter(|c| !c.is_control()) {
                        if n.typed.chars().count() < 24 {
                            n.typed.push(c);
                        }
                    }
                }
                egui::Event::Key { key: egui::Key::Backspace, pressed: true, .. } => {
                    n.typed.pop();
                }
                egui::Event::Key { key: egui::Key::Enter, pressed: true, .. } => {
                    let typed = n.typed.clone();
                    self.named(Some(typed));
                }
                egui::Event::Key { key: egui::Key::Escape, pressed: true, .. } => self.named(None),
                _ => {}
            }
        }
    }

    /// A choice in his menu.
    fn choose(&mut self, choice: Choice) {
        match choice {
            Choice::Tips => {
                self.settings.tips = !self.settings.tips;
                if !self.settings.tips {
                    self.matheo.tip = None;
                    self.matheo.can = None;
                }
            }
            Choice::Quiet => {
                let quiet = self.matheo.quiet.is_some_and(|until| unix_now() < until);
                if quiet {
                    self.matheo.quiet = None;
                    if let Some(f) = matheo_quiet_file() {
                        let _ = std::fs::remove_file(f);
                    }
                } else {
                    let until = unix_now() + QUIET_SECS;
                    self.matheo.quiet = Some(until);
                    if let Some(f) = matheo_quiet_file() {
                        let _ = std::fs::write(f, until.to_string());
                    }
                    self.matheo.tip = Some((Said::Quiet, Instant::now()));
                }
            }
            Choice::Autostart => self.settings.autostart = crate::autostart::set(!self.settings.autostart),
            Choice::Lang => self.settings.lang = self.settings.lang.other(),
            Choice::Version => {}
            Choice::Page => crate::open(crate::PAGE),
            Choice::Journal => {
                if let Some(path) = crate::log::path() {
                    crate::open(&path.to_string_lossy());
                }
            }
            Choice::Install => {
                crate::update::install();
                self.matheo.tip = Some((Said::Installing, Instant::now()));
            }
            Choice::Check => {
                crate::update::check();
                // A newer version found: said again, even if already said.
                self.told_newer = false;
                self.matheo.tip = Some((Said::Checking, Instant::now()));
            }
            Choice::Dock => crate::access::ask(),
            Choice::Hours => self.page = Page::Hours,
            Choice::Demo => self.page = Page::Demo,
            Choice::Back => self.page = Page::Main,
            Choice::Name => self.ask_name(),
            Choice::Play(demo) => self.play(demo),
            Choice::HoursOn => self.settings.hours.on = !self.settings.hours.on,
            // Moved by the click's place in its row (see where the menu is clicked).
            Choice::Hour(_) => {}
            Choice::HoursReset => self.settings.hours = crate::settings::Hours::default(),
            Choice::Skins => {
                self.page = Page::Skins;
                // Beside him, not over him: his change seen.
                if let Some(foot) = self.matheo.at {
                    let screen = os::screen_at(foot).map_or(self.screen, |(screen, _)| screen);
                    let x = if foot.x - SKINS_AWAY - MENU_W > screen.min.x { foot.x - SKINS_AWAY - MENU_W / 2.0 } else { foot.x + SKINS_AWAY + MENU_W / 2.0 };
                    self.menu = Some(Pos2::new(x, foot.y + 8.0));
                }
            }
            Choice::Skin(skin) => {
                if skin != self.settings.skin {
                    self.swap = Some((self.settings.skin, Instant::now()));
                    // Nothing over him while he changes: his bubble closed.
                    self.matheo.tip = None;
                }
                self.settings.skin = skin;
            }
            Choice::Quit => {
                if let Some(at) = self.matheo.at {
                    matheo_save(at);
                }
                std::process::exit(0);
            }
        }
        self.settings.save();
    }
}

impl eframe::App for Overlay {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let now = ctx.input(|i| i.time);
        let dt = self.last.map_or(0.016, |l| (now - l) as f32).clamp(0.001, 0.05);
        self.last = Some(now);
        let mouse = self.mouse.lock().map(|m| *m).unwrap_or_default();
        // Another app full screen (a film...): nothing over it; on the other screens either. Looked
        // at each second.
        if self.full.1.is_none_or(|t| t.elapsed() >= Duration::from_secs(1)) {
            let all = os::screens();
            let screens: Vec<Rect> = if all.is_empty() { vec![self.screen] } else { all.into_iter().map(|s| s.0).collect() };
            // Not for Ronnie (full screen too): he stays over it.
            let ronnie = crate::ronnie::in_front();
            self.full = (screens.into_iter().filter(|s| !ronnie && os::full_screen(*s)).collect(), Some(Instant::now()));
            // The journal: the screens, those hidden from (full screen), this window, where he stands,
            // his windows; when it changes.
            let real = ctx.input(|i| i.viewport().inner_rect);
            let mut others: Vec<&usize> = self.others.keys().collect();
            others.sort();
            let state = format!(
                "écrans {:?} ; plein écran {:?} ; fenêtre {:?} (réelle {real:?}) ; Mathéo {:?} ; ses fenêtres {others:?} ; souris {:?}{}{} ; attrapé {} ; menu {:?}",
                os::screens(),
                self.full.0,
                self.screen,
                self.matheo.at.map(|p| (p.x.round(), p.y.round())),
                mouse.at.map(|p| (p.x.round(), p.y.round())),
                if mouse.pressed { " gauche" } else { "" },
                if mouse.right { " droit" } else { "" },
                self.matheo.grab.is_some(),
                self.menu.map(|p| (p.x.round(), p.y.round())),
            );
            // Said when something but the pointer changed (else a line each second it moves).
            let key = state.split(" ; souris ").next().unwrap_or_default().to_owned() + &format!("{} {}", self.matheo.grab.is_some(), self.menu.is_some());
            if key != self.logged {
                crate::log::line(&state);
                self.logged = key;
            }
            // Linux: the window manager told about our windows once it lists them (Windows: their
            // frame taken off again, should one have slipped through).
            #[cfg(any(windows, all(unix, not(target_os = "macos"))))]
            os::tune_all();
            clear_glass();
        }
        let covered = |full: &[Rect], screen: Rect| full.iter().any(|f| (f.min - screen.min).length() < 1.0 && (f.max - screen.max).length() < 1.0);
        let pressed = mouse.pressed && !self.pressed;
        self.pressed = mouse.pressed;
        let right_click = mouse.right && !self.right;
        self.right = mouse.right;
        let screens = {
            let all = os::screens();
            if all.is_empty() { vec![(self.screen, self.work)] } else { all }
        };
        let works: Vec<Rect> = screens.iter().map(|s| s.1).collect();
        // The screen of a point (his menu's), else the main one.
        let main = self.screen;
        let screen_of = |p: Pos2| screens.iter().map(|s| s.0).find(|s| s.contains(p)).unwrap_or(main);
        let dock = os::dock();
        let grounds = matheo_grounds(&screens, &works, dock);
        // A newer version: said once, a click on his bubble installs it. Installed, he says so and
        // starts again; failed, he says so.
        match crate::update::state() {
            crate::update::State::Available(_) if !self.told_newer && (self.matheo.tip.is_none() || matches!(self.matheo.tip, Some((Said::Checking, _)))) => {
                self.told_newer = true;
                self.told_failed = false;
                self.matheo.tip = Some((Said::Update, Instant::now()));
            }
            crate::update::State::Installed(_) => match self.leaving {
                None => {
                    self.leaving = Some(Instant::now());
                    self.matheo.tip = Some((Said::Updated, Instant::now()));
                }
                Some(t) if t.elapsed() >= Duration::from_millis(2500) => {
                    if let Some(at) = self.matheo.at {
                        matheo_save(at);
                    }
                    crate::update::relaunch();
                    std::process::exit(0);
                }
                Some(_) => {}
            },
            // Asked from his menu: the answer replaces "I'm looking".
            crate::update::State::Latest | crate::update::State::Unreachable if matches!(self.matheo.tip, Some((Said::Checking, _))) => {
                let latest = crate::update::state() == crate::update::State::Latest;
                self.matheo.tip = Some((if latest { Said::Latest } else { Said::Unreachable }, Instant::now()));
            }
            crate::update::State::Failed(_) if !self.told_failed => {
                self.told_failed = true;
                self.told_newer = false;
                self.matheo.tip = Some((Said::UpdateFailed, Instant::now()));
            }
            _ => {}
        }
        // Asking your name: his bubble stays while it does; a click on « That's me! » keeps it, on
        // « Skip » leaves it, elsewhere on the bubble takes the keys to type it.
        let asking = matches!(self.matheo.tip, Some((Said::Own(Own::AskName), _)));
        if asking && self.naming.is_none() {
            self.naming = Some(Naming { typed: String::new(), keys: None, rects: None });
        }
        if !asking && self.naming.take().is_some_and(|n| n.keys.is_some()) {
            release_keys();
        }
        let mut on_name = false;
        if let (Some(naming), Some(p)) = (&mut self.naming, mouse.at) {
            self.matheo.tip = Some((Said::Own(Own::AskName), Instant::now()));
            if let Some([bubble, _, ok, skip]) = naming.rects
                && bubble.contains(p)
            {
                on_name = true;
                if pressed && ok.contains(p) {
                    let typed = naming.typed.clone();
                    self.named(Some(typed));
                } else if pressed && skip.contains(p) {
                    self.named(None);
                } else if pressed {
                    naming.keys = Some(0);
                }
            }
        }
        // Asking how you are: a thumb answers (elsewhere on the bubble closes it, unanswered).
        if !matches!(self.matheo.tip, Some((Said::Own(Own::Mood), _))) {
            self.mood = None;
        }
        if let (true, Some([_, up, down]), Some(p)) = (pressed, self.mood, mouse.at)
            && (up.contains(p) || down.contains(p))
        {
            on_name = true;
            self.mood = None;
            self.matheo.tip = Some((Said::Own(if up.contains(p) { Own::MoodUp } else { Own::MoodDown }), Instant::now()));
        }
        if pressed
            && matches!(self.matheo.tip, Some((Said::Update, _)))
            && self.matheo.bubble.zip(mouse.at).is_some_and(|(r, p)| r.contains(p))
        {
            crate::update::install();
            self.matheo.tip = Some((Said::Installing, Instant::now()));
        }
        // His menu: a click on a row does it, a click anywhere else closes it.
        let quiet = self.matheo.quiet.is_some_and(|until| unix_now() < until);
        let rows = menu_rows(&self.settings, quiet, self.page);
        let menu = self.menu.map(|at| menu_rect(at, rows.len(), screen_of(at)));
        let on_menu = menu.zip(mouse.at).is_some_and(|(r, p)| r.contains(p));
        let hover = menu.zip(mouse.at).filter(|(r, p)| r.contains(*p)).map(|(r, p)| (((p.y - r.min.y - 6.0) / ROW_H).floor().max(0.0) as usize).min(rows.len() - 1));
        // A click on the name bubble is its own (not his: the bubble would close).
        let mut click = pressed && !on_name;
        if pressed && let Some(menu) = menu {
            click = false;
            let at = self.menu.take();
            if menu.contains(mouse.at.unwrap_or(Pos2::ZERO))
                && let Some(i) = hover
            {
                let choice = rows[i].2;
                // A time reminder: its − or + moves it.
                if let (Choice::Hour(k), Some(p)) = (choice, mouse.at) {
                    let x = menu.max.x - 6.0 - p.x;
                    let step = crate::settings::Hours::STEP;
                    if (HOUR_PLUS - 26.0..=HOUR_PLUS).contains(&x) {
                        self.settings.hours.shift(k, step);
                    } else if (HOUR_MINUS - 26.0..=HOUR_MINUS).contains(&x) {
                        self.settings.hours.shift(k, -step);
                    }
                }
                self.choose(choice);
                // His time reminders' page stays open.
                if matches!(choice, Choice::Hours | Choice::Demo | Choice::Back | Choice::HoursOn | Choice::Hour(_) | Choice::HoursReset | Choice::Skins | Choice::Skin(_)) {
                    self.menu = self.menu.or(at);
                }
            }
        }
        let mut open_menu = None;
        let catching;
        // Mathéo: grabbed with the mouse (the window takes the click over him only), dragged, dropped:
        // he falls back to the foot of the screen under him.
        let mut plant_busy = false;
        {
            let plant = &mut self.matheo;
            let at = *plant.at.get_or_insert_with(|| Pos2::new(self.work.max.x - 160.0, self.work.max.y));
            let body = Rect::from_center_size(at - Vec2::new(0.0, MATHEO_H * 0.5), Vec2::new(MATHEO_H * 0.7, MATHEO_H));
            let on_bubble = plant.tip.is_some() && mouse.at.zip(plant.bubble).is_some_and(|(p, r)| r.contains(p));
            let side = can_side(at, &works);
            let on_can = plant.can.is_some_and(|(_, pour)| pour.is_none()) && plant.grab.is_none() && mouse.at.is_some_and(|p| can_rect(at, side).contains(p));
            let over = mouse.at.is_some_and(|p| body.contains(p)) || on_bubble || on_can;
            if right_click && (mouse.at.is_some_and(|p| body.contains(p)) || on_bubble) {
                open_menu = mouse.at;
            }
            if plant.quiet.is_some_and(|until| unix_now() >= until) {
                plant.quiet = None;
            }
            if click && on_can {
                // Watered: the can lifts over his leaves and pours.
                if let Some(can) = &mut plant.can {
                    can.1 = Some(Instant::now());
                }
                if matches!(plant.tip, Some((Said::Tip(_), _))) {
                    plant.tip = None;
                }
            } else if click && on_bubble {
                plant.tip = None;
            } else if click && over {
                // Caught, even in the air.
                plant.grab = mouse.at.map(|p| at - p);
                plant.press = mouse.at;
                plant.track.clear();
                plant.flips.clear();
                (plant.vx, plant.vy, plant.spin) = (0.0, 0.0, 0.0);
            }
            match (plant.grab, mouse.at) {
                (Some(grab), Some(p)) if mouse.pressed => {
                    let to = p + grab;
                    // Swung back the other way, fast: a shake.
                    if let Some(&(then, from)) = plant.track.back()
                        && now > then
                    {
                        let speed = (to.x - from.x) / (now - then) as f32;
                        if speed.abs() > 700.0 {
                            if plant.way != 0.0 && speed.signum() != plant.way {
                                plant.flips.push(now);
                            }
                            plant.way = speed.signum();
                        }
                    }
                    plant.track.push_back((now, to));
                    while plant.track.len() > 8 {
                        plant.track.pop_front();
                    }
                    plant.flips.retain(|t| now - t < SHAKE_WITHIN);
                    // He leans with the swing.
                    let swing = plant.track.front().filter(|(then, _)| now > *then).map_or(0.0, |(then, from)| (to.x - from.x) / (now - then) as f32);
                    plant.tilt += ((swing / 2500.0).clamp(-0.7, 0.7) - plant.tilt) * (1.0 - (-14.0 * dt).exp());
                    if plant.flips.len() >= SHAKE_FLIPS {
                        plant.flips.clear();
                        plant.press = None;
                        let until = unix_now() + QUIET_SECS;
                        plant.quiet = Some(until);
                        if let Some(f) = matheo_quiet_file() {
                            let _ = std::fs::write(f, until.to_string());
                        }
                        plant.dizzy = Some(Instant::now() + Duration::from_secs(5));
                        plant.tip = Some((Said::Quiet, Instant::now()));
                    } else if plant.flips.len() >= 2 && !matches!(plant.tip, Some((Said::Quiet, _))) {
                        plant.tip = Some((Said::Shaking, Instant::now()));
                    }
                    plant.at = Some(to);
                    plant_busy = true;
                }
                (Some(_), _) => {
                    plant.grab = None;
                    if plant.press.take().zip(mouse.at).is_some_and(|(a, b)| a.distance(b) < 4.0) {
                        // Let go where he was taken: a click, his advice at once (or none, if one
                        // shows); he speaks again if he was quiet.
                        if self.settings.tips {
                            if matches!(plant.tip, Some((Said::Tip(_), _))) {
                                plant.tip = None;
                            } else {
                                plant.quiet = None;
                                if let Some(f) = matheo_quiet_file() {
                                    let _ = std::fs::remove_file(f);
                                }
                                plant.advise();
                            }
                        }
                    } else if let (Some(&(t0, p0)), Some(&(t1, p1))) = (plant.track.front(), plant.track.back())
                        && t1 > t0
                    {
                        // Let go moving: thrown, turning on himself.
                        let v = (p1 - p0) / (t1 - t0) as f32;
                        let v = if v.length() > 4500.0 { v.normalized() * 4500.0 } else { v };
                        (plant.vx, plant.vy, plant.spin) = (v.x, v.y, v.x * 0.004);
                    }
                    plant.track.clear();
                    plant_busy = true;
                }
                _ => {
                    // Falling, flying, bouncing off the screens' edges, standing up again on the floor.
                    let half = MATHEO_H * 0.4;
                    let left = works.iter().map(|w| w.min.x).fold(f32::INFINITY, f32::min) + half;
                    let right = works.iter().map(|w| w.max.x).fold(f32::NEG_INFINITY, f32::max) - half;
                    let top = works.iter().map(|w| w.min.y).fold(f32::INFINITY, f32::min) + MATHEO_H;
                    let (rest_x, floor) = matheo_floor(at, &grounds, dock);
                    let upright = plant.tilt.abs() < 0.005;
                    if at.y < floor - 0.5 || at.y > floor + 0.5 || plant.vy != 0.0 || plant.vx != 0.0 || !upright || rest_x != at.x {
                        let mut p = at;
                        let mut bonk = false;
                        plant.vy += 2600.0 * dt;
                        p += Vec2::new(plant.vx, plant.vy) * dt;
                        if p.x < left || p.x > right {
                            p.x = p.x.clamp(left, right.max(left));
                            bonk = plant.vx.abs() > 300.0;
                            plant.vx = -plant.vx * 0.55;
                            plant.spin = -plant.spin * 0.7;
                        }
                        if p.y < top && plant.vy < 0.0 {
                            p.y = top;
                            bonk = plant.vy < -300.0;
                            plant.vy = -plant.vy * 0.4;
                        }
                        let (_, floor) = matheo_floor(p, &grounds, dock);
                        let grounded = p.y >= floor;
                        if grounded {
                            p.y = floor;
                            if plant.vy > 1400.0 {
                                plant.bonk = Some(Instant::now());
                            }
                            plant.vy = if plant.vy > 500.0 { -plant.vy * 0.35 } else { 0.0 };
                            plant.vx *= (-6.0 * dt).exp();
                            if plant.vx.abs() < 20.0 {
                                plant.vx = 0.0;
                            }
                            plant.spin *= (-10.0 * dt).exp();
                            // Back on his foot, the nearest way up.
                            let upright = (plant.tilt / std::f32::consts::TAU).round() * std::f32::consts::TAU;
                            plant.tilt += (upright - plant.tilt) * (1.0 - (-10.0 * dt).exp());
                            if (upright - plant.tilt).abs() < 0.005 && plant.vx == 0.0 {
                                plant.tilt = 0.0;
                            }
                        } else {
                            plant.tilt += plant.spin * dt;
                        }
                        if bonk {
                            plant.bonk = Some(Instant::now());
                            plant.dizzy = Some(plant.dizzy.map_or(Instant::now(), |d| d.max(Instant::now())) + Duration::from_millis(1500));
                            if !matches!(plant.tip, Some((Said::Quiet, _))) {
                                plant.tip = Some((Said::Ouch, Instant::now()));
                            }
                        }
                        if grounded && plant.vx == 0.0 && plant.vy == 0.0 {
                            p.x = matheo_floor(p, &grounds, dock).0;
                            if plant.tilt == 0.0 {
                                matheo_save(p);
                            }
                        }
                        plant.at = Some(p);
                        plant_busy = true;
                    }
                }
            }
            // His advice, every half hour while someone is at the screen (back from a break, a while
            // first); shown a little while.
            let moment = Instant::now();
            if mouse.at != plant.moved.0 {
                if plant.moved.1.elapsed() >= TIP_AWAY {
                    plant.next_tip = plant.next_tip.max(moment + TIP_EVERY / 2);
                }
                if plant.moved.1.elapsed() >= BREAK {
                    plant.active_since = moment;
                }
                plant.moved = (mouse.at, moment);
            }
            if plant.tip.is_some_and(|(said, since)| since.elapsed() >= said.lasts() || (!self.settings.tips && matches!(said, Said::Tip(_)))) {
                plant.tip = None;
            } else if self.settings.tips && plant.quiet.is_none() && plant.tip.is_none() && moment >= plant.next_tip && plant.moved.1.elapsed() < TIP_IDLE && plant.grab.is_none() {
                plant.advise();
            }
            match plant.can {
                Some((since, None)) if since.elapsed() >= CAN_WAITS && self.can_counted != Some(since) => {
                    self.can_counted = Some(since);
                    plant.set_thirst(plant.thirst + 1);
                }
                Some((since, None)) => {
                    let t = since.elapsed().as_secs_f32();
                    if t < 0.6 || t % 3.0 < 0.7 {
                        plant_busy = true;
                    }
                }
                Some((_, Some(pour))) if pour.elapsed().as_secs_f32() >= POUR_END => {
                    plant.can = None;
                    plant.set_thirst(0);
                    plant.watered();
                    plant.glad = Some(Instant::now() + Duration::from_secs(4));
                    plant.tip = Some((Said::Thanks, Instant::now()));
                }
                Some(_) => plant_busy = true,
                None => {}
            }
            // Over him (or his bubble, his can, his menu), or holding him: the window under the
            // pointer catches the clicks (below); else they go through.
            catching = over || plant.grab.is_some() || on_menu;
            if catching != !plant.pass {
                plant.pass = !catching;
                self.glass = GLASS;
            }
        }
        if let Some(at) = open_menu {
            self.menu = Some(at);
            self.page = Page::Main;
        }
        if let Some(at) = self.matheo.at {
            let floor = matheo_floor(at, &grounds, dock).1;
            if self.antics.step(&mut self.matheo, floor, dt) {
                plant_busy = true;
            }
        }
        if self.matheo.tired.1.is_none_or(|t| t.elapsed() >= Duration::from_secs(60)) {
            self.matheo.tired = (late(), Some(Instant::now()));
            self.matheo.friday = friday();
            self.season = Season::now();
        }
        self.care.step(&mut self.matheo, self.settings.tips, self.settings.hours, self.settings.name.is_some(), mouse.at, self.season);
        // His eyes following the pointer near him (back ahead when it goes away), smoothly.
        let to = self.matheo.at.zip(mouse.at).map_or(Vec2::ZERO, |(at, p)| {
            let d = p - (at - Vec2::new(0.0, MATHEO_H * 0.25));
            if d.length() > GAZE_FROM || d.length() < 1.0 { Vec2::ZERO } else { d.normalized() * (d.length() / 60.0).min(1.0) }
        });
        self.gaze += (to - self.gaze) * (1.0 - (-14.0 * dt).exp());
        if (to - self.gaze).length() > 0.02 {
            plant_busy = true;
        } else {
            self.gaze = to;
        }
        // His entrance, at the start: under his parachute or popping out of the bottom of the screen.
        if !self.entered && self.matheo.at.is_some() {
            self.entered = true;
            match Entrance::now() {
                Entrance::Chute => self.play(Demo::Chute),
                Entrance::Pop => self.pop = Some(Instant::now()),
            }
        }
        // Popping out: under the screen's bottom first (how far: `rise`, points), then hopping.
        let mut rise = 0.0;
        let mut pop_look = None;
        if let (Some(since), Some(at)) = (self.pop, self.matheo.at) {
            let depth = os::screen_at(at).map_or(self.screen.max.y, |(screen, _)| screen.max.y) - at.y + MATHEO_H * 1.6;
            match pop_at(since.elapsed().as_secs_f32(), depth) {
                Some((below, hop, squash)) => {
                    rise = below;
                    pop_look = Some((hop, squash));
                    plant_busy = true;
                }
                None => self.pop = None,
            }
        }
        // His can, waiting: away while he's held or flying, hopping in again beside him once down.
        if let Some((since, None)) = self.matheo.can {
            if self.matheo.grab.is_some() || self.antics.air {
                self.can_away = true;
            } else if self.can_away {
                self.can_away = false;
                let now = Instant::now();
                if self.can_counted == Some(since) {
                    self.can_counted = Some(now);
                }
                self.matheo.can = Some((now, None));
                plant_busy = true;
            }
        } else {
            self.can_away = false;
        }
        let tour = self.tour_tick();
        let m = &self.matheo;
        let mut look = Look { gaze: self.gaze, skin: self.settings.skin, ..self.antics.dress(m.look(m.tired.0, m.friday), m) };
        match tour.as_ref().map(|t| t.3) {
            Some(Step::Wilt(w)) => look.wilt = w,
            Some(Step::Tired) => look.tired = true,
            Some(Step::Shades) => look.shades = true,
            Some(Step::Grown) => look.extra = 3,
            Some(Step::Flower) => (look.extra, look.flower) = (3, true),
            Some(Step::Skin(k) | Step::Season(_, k, _)) => look.skin = k,
            _ => {}
        }
        // Changing plant: the one before sinking into the soil, the new one popping out.
        if let Some((before, since)) = self.swap {
            let t = since.elapsed().as_secs_f32();
            match swap_at(t) {
                Some((old, sink)) => {
                    look.sink = sink;
                    if old {
                        look.skin = before;
                    }
                    plant_busy = true;
                }
                None => self.swap = None,
            }
        }
        if let Some((hop, squash)) = pop_look {
            look.hop += hop;
            look.squash = squash;
        }
        let swapping = self.swap.map(|(_, since)| since.elapsed().as_secs_f32());
        let can = m.at.zip(m.can).filter(|_| !self.can_away).map(|(at, (since, pour))| can_view(at, can_side(at, &works), since, pour));
        let plant = m.at.map(|at| (at, look));
        let season = match tour.as_ref().map(|t| t.3) {
            Some(Step::Season(k, ..)) => Some(k),
            _ => self.season,
        };
        let tour_caption = tour.map(|(k, of, label, _)| format!("Visite {k}/{of} · {label}"));
        // His plant's own touch to the outfit.
        let plant = plant.map(|(at, look)| (at, Look { feast: season, ..look }));
        let antics = &self.antics;
        let mut keys = Vec::new();
        let naming = self.naming.as_ref().map(|n| Naming { typed: n.typed.clone(), keys: n.keys, rects: None });
        let t = self.settings.lang.strings();
        // His words: one of a few at each bubble, with your name now and then.
        let v = self.care.variant(m.tip);
        let asking_mood = matches!(m.tip, Some((Said::Own(Own::Mood), _)));
        let tip = m.tip.map(|(said, since)| {
            let text = match said {
                Said::Own(Own::Breathe) => breathe_text(since.elapsed().as_secs_f32(), t, v),
                Said::Own(Own::Glasses) => glasses_text(glasses_today(&m.growth).max(1), t, v),
                Said::Own(own) => own.text(t, v).to_owned(),
                Said::Ouch => pick(t.more_ouch, v).to_owned(),
                Said::Shaking => pick(t.more_shaking, v).to_owned(),
                Said::Thanks => thanks_text(glasses_today(&m.growth), t, v),
                Said::Thirsty => pick(t.more_thirsty, v).to_owned(),
                _ => said.text(self.settings.lang).to_owned(),
            };
            // In the feminine in Paloma's skin.
            self.settings.skin.says(&personal(said, &text, self.settings.name.as_deref(), t))
        });

        let mut busy = plant_busy;
        let mut next: f32 = 1.0;
        let home = screens.iter().position(|(s, _)| (s.min - self.screen.min).length() < 1.0).unwrap_or(0);
        let on = |p: Pos2| screens.iter().position(|(s, _)| s.contains(p));
        // Mathéo on the screen he's on, his menu on its own, the pointer on its own.
        let plant_on = plant.map(|(at, _)| on(at - Vec2::new(0.0, 1.0)).unwrap_or(home));
        let menu = self.menu.map(|at| menu_rect(at, rows.len(), screen_of(at)));
        let menu_on = menu.and_then(|m| on(m.center()));
        let pointer_on = mouse.at.and_then(on);
        let gallery = cfg!(debug_assertions) && std::env::var_os("MATHEO_GALLERY").is_some();
        if menu.is_some() || gallery || tour_caption.is_some() {
            busy = true;
        }
        // A window over each screen, made once at its size and never moved, drawing what is on its
        // screen: Mathéo (his bubble, his can), his menu. (A window following the pointer from screen
        // to screen, at different scales, flashed, jumped, went to and fro on Windows.) Made hidden,
        // shown once drawn (else, on Windows, a white square first), then always there; none over a
        // screen another app fills (a film).
        for (i, (screen, _)) in screens.iter().enumerate() {
            if covered(&self.full.0, *screen) {
                self.others.remove(&i);
                continue;
            }
            let other = self.others.entry(i).or_insert(Other { frames: 0, needed: Instant::now() });
            other.needed = Instant::now();
            let visible = other.frames >= 2;
            if other.frames < 2 + u32::from(GLASS) {
                // Just made or shown: cleared too, drawn again at once to be shown.
                self.glass = self.glass.max(1);
                next = next.min(0.02);
            }
            other.frames = other.frames.saturating_add(1);
            let screen = *screen;
            let his = plant_on == Some(i);
            let menu_here = menu.filter(|_| menu_on == Some(i));
            // The clicks caught by this window only when the pointer is on its screen over him.
            let pass = !(catching && pointer_on == Some(i));
            // Its own title: Windows finds it by it (see `crate::place`).
            let title = format!("Mathéo · {i}");
            let builder = egui::ViewportBuilder::default()
                .with_visible(visible)
                .with_title(title.clone())
                .with_decorations(false)
                .with_transparent(true)
                .with_mouse_passthrough(pass)
                .with_window_level(egui::WindowLevel::AlwaysOnTop)
                .with_taskbar(false)
                .with_active(false)
                .with_has_shadow(false)
                .with_resizable(false)
                .with_position(screen.min)
                .with_inner_size(screen.size());
            // Typing your name: the keys taken by this window, a few frames (see `take_keys`).
            let keys_here = his && naming.as_ref().and_then(|n| n.keys).is_some_and(|k| k < 3);
            if keys_here {
                take_keys(&title);
            }
            let (bubble, name_rects, typed_keys, mood_rects) = ctx.show_viewport_immediate(egui::ViewportId::from_hash_of(("matheo-screen", i)), builder, |ctx, _| {
                if keys_here {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                // Windows places a new window with the scale of the screen it was made on: put over
                // its screen in that screen's pixels (see `crate::place`).
                #[cfg(windows)]
                crate::place::put(&title, screen);
                // Drawn from where the window really is (macOS slides a new one under the menu bar).
                let origin = window_origin(ctx, screen.min);
                let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("matheo")));
                let mut bubble = None;
                let mut name_rects = None;
                let mut mood_rects = None;
                let mut typed_keys = Vec::new();
                if let (Some((at, look)), true) = (plant, his) {
                    // Popping out of the bottom of the screen: drawn lower (where he is stays).
                    let at = at + Vec2::new(0.0, rise);
                    paint_antics_behind(&painter, antics, at - origin.to_vec2(), MATHEO_H, look, origin.to_vec2());
                    paint_matheo_as(&painter, at - origin.to_vec2(), MATHEO_H, now, look);
                    if let Some(t) = swapping {
                        paint_swap_dirt(&painter, at - origin.to_vec2(), MATHEO_H, look, t);
                    }
                    if let Some(season) = season {
                        paint_season(&painter, at - origin.to_vec2(), MATHEO_H, now, look, season);
                    }
                    paint_antics_front(&painter, antics, MATHEO_H, origin.to_vec2());
                    if let Some(can) = can {
                        paint_can(&painter, can, origin.to_vec2());
                    }
                    if let (Some(n), Some(text)) = (naming.as_ref(), tip.as_deref()) {
                        let rects = paint_name_bubble(&painter, at - origin.to_vec2(), text, &n.typed, n.keys.is_some(), now, t, screen.translate(-origin.to_vec2())).map(|r| r.translate(origin.to_vec2()));
                        bubble = Some(rects[0]);
                        name_rects = Some(rects);
                        if n.keys.is_some() {
                            typed_keys = ctx.input(|i| i.events.clone());
                        }
                    } else if let (true, Some(text)) = (asking_mood, tip.as_deref()) {
                        let rects = paint_mood_bubble(&painter, at - origin.to_vec2(), text, mouse.at.map(|p| p - origin.to_vec2()), screen.translate(-origin.to_vec2())).map(|r| r.translate(origin.to_vec2()));
                        bubble = Some(rects[0]);
                        mood_rects = Some(rects);
                    } else {
                        bubble = tip.as_deref().map(|text| paint_matheo_tip(&painter, at - origin.to_vec2(), text, screen.translate(-origin.to_vec2())).translate(origin.to_vec2()));
                    }
                }
                // His menu above his bubble.
                if let Some(menu) = menu_here {
                    paint_menu(&painter, menu.translate(-origin.to_vec2()), &rows, hover);
                }
                if let (Some(caption), true) = (tour_caption.as_deref(), his) {
                    paint_tour_caption(&painter, screen.translate(-origin.to_vec2()), caption);
                }
                if gallery && i == home {
                    paint_matheo_gallery(&painter, screen.translate(-origin.to_vec2()), now);
                    paint_season_gallery(&painter, screen.translate(-origin.to_vec2()), now);
                    paint_skin_gallery(&painter, screen.translate(-origin.to_vec2()), now);
                    paint_antics_gallery(&painter, screen.translate(-origin.to_vec2()), now);
                }
                (bubble, name_rects, typed_keys, mood_rects)
            });
            if his && mood_rects.is_some() {
                self.mood = mood_rects;
            }
            if bubble.is_some() {
                self.matheo.bubble = bubble;
            }
            if let Some(n) = &mut self.naming {
                if his && name_rects.is_some() {
                    n.rects = name_rects;
                }
                if let Some(k) = &mut n.keys
                    && his
                {
                    *k = k.saturating_add(1);
                }
            }
            keys.extend(typed_keys);
            if his {
                // His leaves sway gently, slowly: 5 frames a second are enough (what floats around
                // him in season, smoothly); his blink, on time.
                next = next.min(if season.is_some() { 0.04 } else { 0.2 });
                let blink = now % 4.0;
                next = next.min(if blink < 0.12 { 0.12 - blink } else { 4.0 - blink } as f32 + 0.005);
            }
        }
        if self.naming.as_ref().is_some_and(|n| n.keys.is_some()) {
            // The cursor blinking.
            next = next.min(0.25);
        }
        if !keys.is_empty() {
            self.type_name(&keys);
        }
        // Screens gone: their windows with them.
        self.others.retain(|i, _| *i < screens.len());
        os::tune_all();
        if self.glass > 0 {
            self.glass -= 1;
            clear_glass();
            busy = true;
        }
        // Where a move of the pointer draws again at once (see `run`).
        let near = self.matheo.at.map(|at| {
            let mut r = Rect::from_center_size(at - Vec2::new(0.0, MATHEO_H * 0.5), Vec2::new(MATHEO_H * 0.7, MATHEO_H));
            if let Some(b) = self.matheo.bubble.filter(|_| self.matheo.tip.is_some()) {
                r = r.union(b);
            }
            if self.matheo.can.is_some() {
                r = r.union(can_rect(at, can_side(at, &works)));
            }
            r.expand(32.0).union(Rect::from_center_size(at - Vec2::new(0.0, MATHEO_H * 0.25), Vec2::splat(GAZE_FROM * 2.0)))
        });
        if let Ok(mut hot) = self.hot.lock() {
            // A menu open, a name typed: everywhere.
            *hot = if self.menu.is_some() || self.naming.is_some() { None } else { near };
        }
        if busy {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(Duration::from_secs_f32(next.clamp(0.02, 1.0)));
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }
}

/// Your name being typed in his bubble: what so far, the keys taken (frames since, to take them a
/// few times), his bubble's parts as last drawn (the bubble, the field, « That's me! », « Skip »).
struct Naming {
    typed: String,
    keys: Option<u32>,
    rects: Option<[Rect; 4]>,
}

/// Typing his name: his app in front and the window `title` (over his screen) taking the keys. On
/// macOS his app has no Dock icon nor menu bar, and his windows never take the keys by themselves.
fn take_keys(title: &str) {
    #[cfg(target_os = "macos")]
    {
        let Some(mtm) = objc2::MainThreadMarker::new() else { return };
        let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
        #[allow(deprecated)]
        app.activateIgnoringOtherApps(true);
        for window in app.windows().iter() {
            if window.title().to_string() == title {
                window.makeKeyAndOrderFront(None);
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = title;
}

/// Done typing: the keys back to the app that had them.
fn release_keys() {
    #[cfg(target_os = "macos")]
    if let Some(mtm) = objc2::MainThreadMarker::new() {
        objc2_app_kit::NSApplication::sharedApplication(mtm).deactivate();
    }
}

/// What Mathéo says of his own (Ronnie's Mathéo has none of it): his hours (the break at 10 and 16,
/// lunch at 12, home at 18), Monday morning, the lamp on winter evenings, working at the weekend,
/// back after a long while away; and between his advice, in turn, his care: the eyes, sitting up,
/// breathing (a big glass of water too in summer).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Own {
    /// His time reminders (`Hours`), by number: the morning break, lunch, the afternoon break, home.
    Clock(usize),
    Monday,
    Lamp,
    Weekend,
    Eyes,
    Posture,
    /// Breathing with him: he swells and shrinks slowly while his bubble shows.
    Breathe,
    Hot,
    /// Back after a long while away: glad, hopping.
    Back,
    /// What he may call you (a click on the bubble asks), and glad to know it.
    AskName,
    Named,
    /// After an update, your name not known: hello again (then he asks it).
    Again,
    /// The first start of a new version: what's new in it (in place of hello again).
    News,
    /// The evening: the glasses of water of the day (see `glasses_text`).
    Glasses,
    /// Some days: how are you? A thumb up or down in his bubble (see `mood_due`), and his answer.
    Mood,
    MoodUp,
    MoodDown,
    /// His season's outfit on: a word of the feast (on the day itself, for Christmas), now and then
    /// (see `feast_due`).
    Feast(Season, bool),
}

impl Own {
    /// What he may say, in turn (one picked at each bubble, see `Care::variant`).
    fn texts(self, t: &'static crate::i18n::Strings) -> &'static [&'static str] {
        match self {
            Own::Clock(1) => t.matheo_lunch,
            Own::Clock(3) => t.matheo_home,
            Own::Clock(_) => t.matheo_pause,
            Own::Monday => t.matheo_monday,
            Own::Lamp => t.matheo_lamp,
            Own::Weekend => t.matheo_weekend_work,
            Own::Eyes => t.matheo_eyes,
            Own::Posture => t.matheo_posture,
            Own::Breathe => t.matheo_breathe,
            Own::Hot => t.matheo_hot,
            Own::Back => t.matheo_back,
            Own::AskName => std::slice::from_ref(&t.matheo_ask_name),
            Own::Named => t.matheo_named,
            Own::Again => t.matheo_again,
            Own::News => t.matheo_news,
            Own::Glasses => t.matheo_glasses_good,
            Own::Mood => t.matheo_mood,
            Own::MoodUp => t.matheo_mood_up,
            Own::MoodDown => t.matheo_mood_down,
            Own::Feast(Season::Christmas, false) => t.matheo_christmas_soon,
            Own::Feast(Season::Christmas, true) => t.matheo_christmas,
            Own::Feast(Season::NewYear, _) => t.matheo_new_year,
            Own::Feast(Season::Valentine, _) => t.matheo_valentine,
            Own::Feast(Season::Easter, _) => t.matheo_easter,
            Own::Feast(Season::Halloween, _) => t.matheo_halloween,
        }
    }

    /// Its words number `v` (any number: taken in turn).
    fn text(self, t: &'static crate::i18n::Strings, v: usize) -> &'static str {
        pick(self.texts(t), v)
    }

    fn lasts(self) -> Duration {
        match self {
            Own::Back | Own::Named => Duration::from_secs(6),
            Own::Again => Duration::from_secs(8),
            Own::News => Duration::from_secs(15),
            Own::AskName => Duration::from_secs(60),
            Own::Breathe => BREATHE_LASTS,
            Own::Mood => Duration::from_secs(90),
            Own::MoodUp | Own::MoodDown => Duration::from_secs(12),
            _ => TIP_SHOWN,
        }
    }
}

/// His care: the first a while after the start, then one every `CARE_EVERY`, never sooner than
/// `CARE_GAP` after another bubble.
const CARE_FIRST: Duration = Duration::from_secs(25 * 60);
const CARE_EVERY: Duration = Duration::from_secs(40 * 60);
const CARE_GAP: Duration = Duration::from_secs(8 * 60);
/// How you are, the glasses of water of the day: asked, said that long after any other bubble.
const QUIET_GAP: Duration = Duration::from_secs(2 * 60);
/// Away that long (the pointer still, the computer asleep): glad to see you back.
const AWAY_LONG: Duration = Duration::from_secs(3600);
/// Your name asked that long after his start (after his hello).
const ASK_NAME_AFTER: Duration = Duration::from_secs(2);
/// Breathing with him: what he says first (seconds), breathing in, out, how many times, the end.
const BREATHE_INTRO: f32 = 5.0;
const BREATHE_IN: f32 = 4.0;
const BREATHE_OUT: f32 = 6.0;
const BREATHS: u32 = 4;
const BREATHE_END: f32 = 5.0;
const BREATHE_LASTS: Duration = Duration::from_secs((BREATHE_INTRO + (BREATHE_IN + BREATHE_OUT) * BREATHS as f32 + BREATHE_END) as u64);

/// Breathing with him, `t` seconds after it started: which breath (from 1), breathing in, the
/// seconds left of it, how full he is (0 to 1); none before (his words) and after.
fn breath(t: f32) -> Option<(u32, bool, u32, f32)> {
    let t = t - BREATHE_INTRO;
    let cycle = BREATHE_IN + BREATHE_OUT;
    if !(0.0..cycle * BREATHS as f32).contains(&t) {
        return None;
    }
    let k = (t / cycle) as u32 + 1;
    let c = t % cycle;
    let smooth = |x: f32| x * x * (3.0 - 2.0 * x);
    Some(if c < BREATHE_IN {
        (k, true, (BREATHE_IN - c).ceil() as u32, smooth(c / BREATHE_IN))
    } else {
        (k, false, (cycle - c).ceil() as u32, 1.0 - smooth((c - BREATHE_IN) / BREATHE_OUT))
    })
}

/// His bubble while breathing with him, `t` seconds after it started: his words, then each breath
/// (in or out, the seconds left, which one), then the end.
fn breathe_text(t: f32, s: &'static crate::i18n::Strings, v: usize) -> String {
    match breath(t) {
        Some((k, breathing_in, left, _)) => {
            let what = if breathing_in { s.breathe_in } else { s.breathe_out };
            format!("{}\n{}", what.replace("{n}", &left.to_string()), s.breathe_count.replace("{k}", &k.to_string()).replace("{of}", &BREATHS.to_string()))
        }
        None if t < BREATHE_INTRO => pick(s.matheo_breathe, v).to_owned(),
        None => pick(s.breathe_done, v).to_owned(),
    }
}

/// What he said of his own, and when to say it.
struct Care {
    /// Today's words already said (each once a day).
    said: (Option<chrono::NaiveDate>, Vec<Own>),
    /// His next care, and which in turn.
    next: Instant,
    turn: usize,
    /// When a bubble last showed.
    bubble: Instant,
    /// The pointer's last place and when it moved (the wall clock: the computer's sleep counts).
    seen: (Option<Pos2>, std::time::SystemTime),
    /// His start, and whether he asked your name since.
    start: Instant,
    asked: bool,
    /// His very first start (his hello first), else hello again before asking.
    first: bool,
    /// The bubble showing (since when) and which of its words.
    variant: (Option<(Said, Instant)>, usize),
    seed: u64,
    /// A new version: what's new in it, said once a little after the start.
    news: bool,
}

impl Care {
    fn new(first: bool, news: bool) -> Self {
        let care = if care_forced() { Duration::from_secs(5) } else { CARE_FIRST };
        Self { said: (None, Vec::new()), next: Instant::now() + care, turn: 0, bubble: Instant::now() - CARE_GAP, seen: (None, std::time::SystemTime::now()), start: Instant::now(), asked: false, first, variant: (None, 0), seed: 0, news }
    }

    /// Which words the bubble `tip` (what, since when) says: picked at random once a bubble. The
    /// same words said again within a second (shaking him, bumping him: his bubble set again each
    /// frame) are the same bubble, its words kept (else they changed each frame, flickering).
    fn variant(&mut self, tip: Option<(Said, Instant)>) -> usize {
        let same = match (tip, self.variant.0) {
            (Some((said, since)), Some((was, then))) => said == was && since.saturating_duration_since(then) < Duration::from_secs(1),
            (None, None) => true,
            _ => false,
        };
        if !same {
            if self.seed == 0 {
                self.seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(11, |d| d.as_nanos() as u64) | 1;
            }
            self.seed ^= self.seed << 13;
            self.seed ^= self.seed >> 7;
            self.seed ^= self.seed << 17;
            self.variant.1 = (self.seed >> 16) as usize;
        }
        self.variant.0 = tip;
        self.variant.1
    }

    /// Each frame, after his own advice: says what is due, if his advice is on, he isn't quiet, nor
    /// held, someone is at the screen and no other bubble shows (his advice can give way to his
    /// hours).
    fn step(&mut self, plant: &mut Plant, tips: bool, hours: crate::settings::Hours, named: bool, mouse: Option<Pos2>, season: Option<Season>) {
        let moment = Instant::now();
        let wall = std::time::SystemTime::now();
        let mut back = false;
        if mouse != self.seen.0 {
            back = self.seen.0.is_some() && wall.duration_since(self.seen.1).is_ok_and(|d| d >= AWAY_LONG);
            self.seen = (mouse, wall);
        }
        if plant.tip.is_some() {
            self.bubble = moment;
        }
        // A new version: what's new in it, first (in place of hello again).
        if self.news && plant.tip.is_none() && plant.grab.is_none() && self.start.elapsed() >= ASK_NAME_AFTER {
            self.news = false;
            self.first = true;
            plant.tip = Some((Said::Own(Own::News), moment));
            return;
        }
        // Your name, not known yet (never asked, nor skipped): asked once a start, right after his
        // hello the very first time; after an update, hello again first.
        if !named && !self.asked && plant.tip.is_none() && plant.grab.is_none() && self.start.elapsed() >= ASK_NAME_AFTER {
            if !self.first {
                self.first = true;
                plant.tip = Some((Said::Own(Own::Again), moment));
                return;
            }
            self.asked = true;
            plant.tip = Some((Said::Own(Own::AskName), moment));
            return;
        }
        let free = tips && plant.quiet.is_none_or(|until| unix_now() >= until) && plant.grab.is_none() && plant.moved.1.elapsed() < TIP_AWAY;
        if !free {
            return;
        }
        let only_advice = plant.tip.is_none_or(|(said, _)| matches!(said, Said::Tip(_)));
        if back && only_advice {
            plant.tip = Some((Said::Own(Own::Back), moment));
            return;
        }
        let now = clock_now();
        if self.said.0 != Some(now.date()) {
            self.said = (Some(now.date()), Vec::new());
        }
        if only_advice && let Some(own) = day_due(now, hours, &self.said.1) {
            self.said.1.push(own);
            plant.tip = Some((Said::Own(own), moment));
            return;
        }
        let calm = plant.tip.is_none() && (self.bubble.elapsed() >= QUIET_GAP || care_forced());
        // His season's outfit on: a word of the feast, once on the days it's due.
        if calm && let Some(feast) = feast_due(now, season, &self.said.1) {
            self.said.1.push(feast);
            plant.tip = Some((Said::Own(feast), moment));
            return;
        }
        // Some days, in the day: how are you? Once.
        if calm && mood_due(now, &self.said.1) {
            self.said.1.push(Own::Mood);
            plant.tip = Some((Said::Own(Own::Mood), moment));
            return;
        }
        // The evening (from the end of the day on): the glasses of water of the day, once.
        if calm && glasses_due(now, hours, glasses_today(&plant.growth), &self.said.1) {
            self.said.1.push(Own::Glasses);
            plant.tip = Some((Said::Own(Own::Glasses), moment));
            return;
        }
        if plant.tip.is_none() && moment >= self.next && (self.bubble.elapsed() >= CARE_GAP || care_forced()) {
            use chrono::Datelike;
            let list: &[Own] = if (6..=8).contains(&now.month()) { &[Own::Eyes, Own::Hot, Own::Posture, Own::Breathe] } else { &[Own::Eyes, Own::Posture, Own::Breathe] };
            let own = list[self.turn % list.len()];
            self.turn += 1;
            self.next = moment + if care_forced() { Duration::from_secs(20) } else { CARE_EVERY };
            plant.tip = Some((Said::Own(own), moment));
        }
    }
}

/// Glasses of water today: his waterings today (each with a reminder to drink a glass, then water
/// him).
fn glasses_today(growth: &(String, u32, u32)) -> u32 {
    if growth.0 == chrono::Local::now().format("%Y-%m-%d").to_string() { growth.1 } else { 0 }
}

/// The glasses of water of the day due at `now`: some drunk, the end of the day (`hours`' last, on or
/// off) past, not said yet today.
fn glasses_due(now: chrono::NaiveDateTime, hours: crate::settings::Hours, glasses: u32, said: &[Own]) -> bool {
    use chrono::Timelike;
    glasses > 0 && now.hour() * 60 + now.minute() >= hours.at[3] && !said.contains(&Own::Glasses)
}

/// Asking how you are due at `now`: two days in five (the same all day long, picked from the date),
/// from a time of the day's own between 10:30 and 15:30 until 5 pm; once.
fn mood_due(now: chrono::NaiveDateTime, said: &[Own]) -> bool {
    use chrono::{Datelike, Timelike};
    let h = (now.date().num_days_from_ce() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let h = h ^ (h >> 29);
    let from = 10 * 60 + 30 + ((h >> 8) % (5 * 60)) as u32;
    h % 5 < 2 && (from..17 * 60).contains(&(now.hour() * 60 + now.minute())) && !said.contains(&Own::Mood)
}

/// A word of the feast due at `now`, `season`'s outfit on: between 9 am and 8 pm, once; a day in two
/// for the long ones (Christmas, Halloween) but on their days (December 24 and 25, October 31), every
/// day for the others.
fn feast_due(now: chrono::NaiveDateTime, season: Option<Season>, said: &[Own]) -> Option<Own> {
    use chrono::{Datelike, Timelike};
    let season = season?;
    let (m, d) = (now.month(), now.day());
    let day = matches!((season, m, d), (Season::Christmas, 12, 24..=25) | (Season::Halloween, 10, 31));
    let long = matches!(season, Season::Christmas | Season::Halloween);
    let today = day || !long || now.date().num_days_from_ce() % 2 == 0;
    let said_already = said.iter().any(|o| matches!(o, Own::Feast(..)));
    (today && (9..20).contains(&now.hour()) && !said_already).then_some(Own::Feast(season, season == Season::Christmas && day))
}

/// « 3 glasses of water ».
fn glasses_said(n: u32, t: &'static crate::i18n::Strings) -> String {
    (if n == 1 { t.glass_one } else { t.glass_many }).replace("{n}", &n.to_string())
}

/// The evening's words on the `n` glasses of water of the day (number `v`).
fn glasses_text(n: u32, t: &'static crate::i18n::Strings, v: usize) -> String {
    let list = match n {
        0..=2 => t.matheo_glasses_few,
        3..=5 => t.matheo_glasses_good,
        _ => t.matheo_glasses_lots,
    };
    pick(list, v).replace("{glasses}", &glasses_said(n, t))
}

/// Thanks for watering him (words number `v`), and the glasses of water of the day, `n` with this one.
fn thanks_text(n: u32, t: &'static crate::i18n::Strings, v: usize) -> String {
    let count = if n <= 1 { t.thanks_glass_first.to_owned() } else { t.thanks_glass_more.replace("{glasses}", &glasses_said(n, t)) };
    format!("{}\n{count}", pick(t.more_thanks, v))
}

/// Words number `v` among `list` (any number: taken in turn).
fn pick(list: &'static [&'static str], v: usize) -> &'static str {
    list.get(v % list.len().max(1)).copied().unwrap_or_default()
}

/// His bubble's words with your name (`{name}` in them; without a name, left out); now and then his
/// advice starts with it.
fn personal(said: Said, text: &str, name: Option<&str>, t: &crate::i18n::Strings) -> String {
    let name = name.map(str::trim).filter(|n| !n.is_empty());
    let text = match (said, name) {
        (Said::Tip(k), Some(_)) if k % 3 == 0 && !text.contains("{name}") => format!("{}{text}", t.matheo_psst),
        _ => text.to_owned(),
    };
    match name {
        Some(name) => text.replace("{name}", name),
        None => text.replace(", {name}", "").replace(" {name}", "").replace("{name}", ""),
    }
}

/// What to say once today at `now`, if anything (`said`: what was, today): Monday morning, his time
/// reminders on weekdays (`hours`, in their first half hour), the lamp at 5 pm from November to
/// January, working at the weekend.
fn day_due(now: chrono::NaiveDateTime, hours: crate::settings::Hours, said: &[Own]) -> Option<Own> {
    use chrono::{Datelike, Timelike};
    let (h, day) = (now.hour(), now.weekday().number_from_monday());
    let minute = h * 60 + now.minute();
    let weekday = day <= 5;
    let clock = (0..hours.at.len()).find(|&k| hours.on && weekday && (hours.at[k]..hours.at[k] + 30).contains(&minute) && !said.contains(&Own::Clock(k)));
    [
        (day == 1 && (7..12).contains(&h), Own::Monday),
        (clock.is_some(), Own::Clock(clock.unwrap_or_default())),
        (matches!(now.month(), 11 | 12 | 1) && h == 17, Own::Lamp),
        (!weekday && (9..20).contains(&h), Own::Weekend),
    ]
    .into_iter()
    .find(|&(due, own)| due && !said.contains(&own))
    .map(|(_, own)| own)
}

/// Now; debug builds: `MATHEO_CLOCK=10|12|16|17|18` that hour on a Monday (in November),
/// `MATHEO_CLOCK=saturday` a Saturday morning.
fn clock_now() -> chrono::NaiveDateTime {
    let day = |d| chrono::NaiveDate::from_ymd_opt(2026, 11, d).unwrap_or_default();
    let forced = match forced_clock().as_deref() {
        Some("saturday") => day(21).and_hms_opt(10, 5, 0),
        Some(h) => h.parse::<u32>().ok().and_then(|h| day(16).and_hms_opt(h, 5, 0)),
        None => None,
    };
    forced.unwrap_or_else(|| chrono::Local::now().naive_local())
}

/// Debug builds: `MATHEO_CARE=1`, his care 5 seconds after the start, then every 20 seconds.
fn care_forced() -> bool {
    cfg!(debug_assertions) && std::env::var_os("MATHEO_CARE").is_some()
}

#[cfg(test)]
mod care_tests {
    use chrono::NaiveDate;

    use super::{Own, day_due};
    use crate::settings::Hours;

    fn day_due_(now: chrono::NaiveDateTime, said: &[Own]) -> Option<Own> {
        day_due(now, Hours::default(), said)
    }

    fn at(d: u32, h: u32, m: u32) -> chrono::NaiveDateTime {
        // November 2026: the 16th a Monday, the 21st a Saturday.
        NaiveDate::from_ymd_opt(2026, 11, d).unwrap().and_hms_opt(h, m, 0).unwrap()
    }

    #[test]
    fn shaking_him_keeps_his_words() {
        use super::{Care, Said};
        use std::time::{Duration, Instant};
        let mut care = Care::new(false, false);
        let start = Instant::now();
        let v = care.variant(Some((Said::Shaking, start)));
        // His bubble set again each frame while shaken: the same words.
        for k in 1..60 {
            assert_eq!(care.variant(Some((Said::Shaking, start + Duration::from_millis(16 * k)))), v);
        }
        // Another bubble: words picked again (the same ones only by chance).
        let picks: Vec<usize> = (0..8).map(|k| care.variant(Some((Said::Ouch, start + Duration::from_secs(5 * (k + 1)))))).collect();
        assert!(picks.windows(2).any(|w| w[0] != w[1]));
    }

    #[test]
    fn his_hours_are_said_once_on_weekdays() {
        for (k, h) in [10, 12, 16, 18].into_iter().enumerate() {
            assert_eq!(day_due_(at(17, h, 0), &[]), Some(Own::Clock(k)));
            assert_eq!(day_due_(at(17, h, 29), &[Own::Clock(k)]), None, "said once");
            assert_eq!(day_due_(at(17, h, 30), &[]), None, "half an hour late: too late");
        }
        assert_eq!(day_due_(at(17, 11, 0), &[]), None);
        assert_eq!(day_due_(at(21, 10, 5), &[Own::Weekend]), None, "not at the weekend");
    }

    #[test]
    fn monday_morning_first_then_the_break() {
        assert_eq!(day_due_(at(16, 9, 0), &[]), Some(Own::Monday));
        assert_eq!(day_due_(at(16, 10, 5), &[]), Some(Own::Monday));
        assert_eq!(day_due_(at(16, 10, 5), &[Own::Monday]), Some(Own::Clock(0)));
        assert_eq!(day_due_(at(16, 13, 0), &[]), None);
        assert_eq!(day_due_(at(17, 9, 0), &[]), None, "Tuesday");
    }

    #[test]
    fn the_lamp_on_winter_evenings_and_the_weekend() {
        assert_eq!(day_due_(at(17, 17, 40), &[]), Some(Own::Lamp));
        let june = NaiveDate::from_ymd_opt(2026, 6, 16).unwrap().and_hms_opt(17, 10, 0).unwrap();
        assert_eq!(day_due_(june, &[]), None);
        assert_eq!(day_due_(at(21, 10, 0), &[]), Some(Own::Weekend));
        assert_eq!(day_due_(at(22, 21, 0), &[]), None, "Sunday evening: leave it");
    }

    #[test]
    fn the_glasses_of_water_in_the_evening_once() {
        use super::{glasses_due, glasses_text, personal, thanks_text, Said};
        let hours = Hours::default();
        assert!(!glasses_due(at(17, 17, 59), hours, 4, &[]), "before the end of the day");
        assert!(glasses_due(at(17, 18, 0), hours, 4, &[]));
        assert!(glasses_due(at(21, 22, 30), Hours { on: false, ..hours }, 1, &[]), "his hours off, the weekend: still");
        assert!(!glasses_due(at(17, 19, 0), hours, 0, &[]), "none drunk: nothing said");
        assert!(!glasses_due(at(17, 19, 0), hours, 4, &[Own::Glasses]), "said once");
        let t = crate::i18n::Lang::Fr.strings();
        assert_eq!(personal(Said::Own(Own::Glasses), &glasses_text(1, t, 0), None, t), "Petit bilan du jour : 1 verre d’eau. Demain, on en boit un de plus ?");
        assert_eq!(personal(Said::Own(Own::Glasses), &glasses_text(4, t, 0), Some("Julie"), t), "Bilan du jour : 4 verres d’eau, bien joué Julie ! Toi et mes racines, on est bien hydratés.");
        assert_eq!(personal(Said::Own(Own::Glasses), &glasses_text(4, t, 0), None, t), "Bilan du jour : 4 verres d’eau, bien joué ! Toi et mes racines, on est bien hydratés.");
        assert_eq!(glasses_text(7, t, 1), "Record ! 7 verres d’eau aujourd’hui. Je suis fier de toi.");
        assert!(thanks_text(1, t, 0).ends_with("\nPremier verre d’eau de la journée !"));
        assert!(thanks_text(3, t, 0).ends_with("\nEt de 3 verres d’eau aujourd’hui !"));
        let en = crate::i18n::Lang::En.strings();
        assert!(thanks_text(2, en, 0).ends_with("\nThat’s 2 glasses of water today!"));
    }

    #[test]
    fn a_word_of_the_feast_now_and_then() {
        use super::{Season, feast_due};
        let day = |m, d, h| NaiveDate::from_ymd_opt(2026, m, d).unwrap().and_hms_opt(h, 0, 0).unwrap();
        assert_eq!(feast_due(day(12, 24, 10), Some(Season::Christmas), &[]), Some(Own::Feast(Season::Christmas, true)));
        assert_eq!(feast_due(day(12, 25, 19), Some(Season::Christmas), &[]), Some(Own::Feast(Season::Christmas, true)));
        assert_eq!(feast_due(day(12, 24, 8), Some(Season::Christmas), &[]), None, "too early");
        assert_eq!(feast_due(day(12, 24, 10), Some(Season::Christmas), &[Own::Feast(Season::Christmas, true)]), None, "once");
        assert_eq!(feast_due(day(12, 24, 10), None, &[]), None, "no outfit");
        // Before Christmas, a day in two, its words before the day.
        let said: Vec<Option<Own>> = (1..=23).map(|d| feast_due(day(12, d, 10), Some(Season::Christmas), &[])).collect();
        assert!(said.windows(2).all(|w| w[0].is_some() != w[1].is_some()), "{said:?}");
        assert!(said.iter().flatten().all(|o| *o == Own::Feast(Season::Christmas, false)));
        assert!(feast_due(day(10, 31, 10), Some(Season::Halloween), &[]).is_some());
        assert!((1..=2).all(|d| feast_due(day(1, d, 10), Some(Season::NewYear), &[]).is_some()), "every day");
        assert!(feast_due(day(2, 14, 12), Some(Season::Valentine), &[]).is_some());
    }

    #[test]
    fn how_are_you_some_days_once() {
        use super::mood_due;
        let mut days = 0;
        for d in 0..200 {
            let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap() + chrono::Days::new(d);
            let asked: Vec<u32> = (0..24 * 60).filter(|m| mood_due(date.and_hms_opt(m / 60, m % 60, 0).unwrap(), &[])).collect();
            if let Some(&first) = asked.first() {
                days += 1;
                assert!((10 * 60 + 30..15 * 60 + 30).contains(&first), "{date}: from {first}");
                assert_eq!(asked.last(), Some(&(17 * 60 - 1)), "{date}: until 5 pm");
                assert!(!mood_due(date.and_hms_opt(16, 59, 0).unwrap(), &[Own::Mood]), "once");
            }
        }
        assert!((60..100).contains(&days), "{days} days in 200");
    }

    #[test]
    fn his_hours_as_set_in_his_menu() {
        let mut hours = Hours::default();
        hours.shift(1, 30);
        assert_eq!(day_due(at(17, 12, 10), hours, &[]), None, "lunch moved to 12:30");
        assert_eq!(day_due(at(17, 12, 35), hours, &[]), Some(Own::Clock(1)));
        hours.on = false;
        assert_eq!(day_due(at(17, 12, 35), hours, &[]), None, "off");
        hours.shift(3, -24 * 60);
        assert_eq!(hours.at[3], 0);
        hours.shift(3, 48 * 60);
        assert_eq!(hours.at[3], 24 * 60 - 15);
    }

    #[test]
    fn his_words_with_your_name_or_without() {
        use super::{Said, Season, glasses_text, personal, thanks_text};
        let t = crate::i18n::Lang::Fr.strings();
        let back = Said::Own(Own::Back);
        assert_eq!(personal(back, Own::Back.text(t, 0), Some("Julie"), t), "Te revoilà, Julie ! Tu m’as manqué.");
        assert_eq!(personal(back, Own::Back.text(t, 0), None, t), "Te revoilà ! Tu m’as manqué.");
        assert_eq!(personal(Said::Own(Own::Weekend), Own::Weekend.text(t, 0), None, t).split('.').next(), Some("Psst… c’est le week-end"));
        assert_eq!(personal(Said::Tip(3), "Bois.", Some(" Julie "), t), "Psst, Julie… Bois.");
        assert_eq!(personal(Said::Tip(4), "Bois.", Some("Julie"), t), "Bois.");
        for lang in crate::i18n::Lang::ALL {
            let t = lang.strings();
            let all = [Own::Clock(0), Own::Clock(1), Own::Clock(3), Own::Monday, Own::Lamp, Own::Weekend, Own::Eyes, Own::Posture, Own::Breathe, Own::Hot, Own::Back, Own::AskName, Own::Again, Own::News, Own::Mood, Own::MoodUp, Own::MoodDown, Own::Feast(Season::Christmas, false), Own::Feast(Season::Christmas, true), Own::Feast(Season::NewYear, false), Own::Feast(Season::Valentine, false), Own::Feast(Season::Easter, false), Own::Feast(Season::Halloween, false)];
            // (`Named` only with a name.)
            for own in all {
                assert!(!own.texts(t).is_empty(), "{own:?}");
                for v in 0..own.texts(t).len() {
                    let text = personal(Said::Own(own), own.text(t, v), None, t);
                    assert!(!text.contains('{') && !text.contains(" ,") && !text.starts_with(' '), "{own:?} {v}: {text}");
                }
            }
            for n in 0..10 {
                for v in 0..4 {
                    for text in [glasses_text(n, t, v), thanks_text(n, t, v)] {
                        let text = personal(Said::Own(Own::Glasses), &text, None, t);
                        assert!(!text.contains('{') && !text.contains("  ") && !text.contains(" ,"), "{text}");
                    }
                }
            }
            for list in [t.more_ouch, t.more_shaking, t.more_thanks, t.more_thirsty, t.breathe_done] {
                for text in list {
                    assert!(!personal(Said::Thanks, text, None, t).contains('{'), "{text}");
                }
            }
        }
    }
}

/// Asking how you are: his bubble (as `paint_matheo_tip` draws it, `question` on top) with a thumb up
/// and a thumb down under it, the one under the pointer (`hover`) lit. The bubble, the two thumbs.
fn paint_mood_bubble(painter: &egui::Painter, foot: Pos2, question: &str, hover: Option<Pos2>, bounds: Rect) -> [Rect; 3] {
    let r = paint_matheo_tip(painter, foot, &format!("{question}\n\n\n"), bounds);
    let size = Vec2::new(58.0, 30.0);
    let y = r.max.y - 10.0 - size.y;
    let up = Rect::from_min_size(Pos2::new(r.center().x - size.x - 7.0, y), size);
    let down = Rect::from_min_size(Pos2::new(r.center().x + 7.0, y), size);
    for (b, thumb, fill, lit) in [(up, "👍", Color32::from_rgb(76, 160, 70), Color32::from_rgb(92, 182, 84)), (down, "👎", Color32::from_rgb(214, 138, 74), Color32::from_rgb(232, 156, 90))] {
        let on = hover.is_some_and(|p| b.contains(p));
        painter.rect_filled(if on { b.expand(1.5) } else { b }, 15.0, if on { lit } else { fill });
        painter.text(b.center() + Vec2::new(0.0, 1.0), egui::Align2::CENTER_CENTER, thumb, FontId::proportional(18.0), Color32::WHITE);
    }
    [r, up, down]
}

/// Asking your name: his bubble (as `paint_matheo_tip` draws it, `question` on top) with a field to
/// type it in, « That's me! » and « Skip » under it; `typing`, the keys his, the cursor blinking.
/// The bubble, the field, the two buttons.
#[allow(clippy::too_many_arguments)]
fn paint_name_bubble(painter: &egui::Painter, foot: Pos2, question: &str, typed: &str, typing: bool, now: f64, t: &crate::i18n::Strings, bounds: Rect) -> [Rect; 4] {
    let r = paint_matheo_tip(painter, foot, &format!("{question}\n\n\n\n"), bounds);
    let ink = Color32::from_rgb(46, 64, 36);
    let green = Color32::from_rgb(76, 160, 70);
    let grey = Color32::from_rgb(150, 156, 140);
    let field = Rect::from_min_size(Pos2::new(r.min.x + 13.0, r.max.y - 9.0 - 54.0), Vec2::new(r.width() - 26.0, 24.0));
    painter.rect(field, 6.0, Color32::WHITE, Stroke::new(if typing { 1.6 } else { 1.0 }, if typing { green } else { grey }), egui::StrokeKind::Inside);
    let at = field.left_center() + Vec2::new(8.0, 0.0);
    let end = if typed.is_empty() {
        if !typing {
            painter.text(at, egui::Align2::LEFT_CENTER, t.name_hint, FontId::proportional(14.0), grey);
        }
        at.x
    } else {
        painter.text(at, egui::Align2::LEFT_CENTER, typed, FontId::proportional(14.0), ink).max.x
    };
    if typing && (now * 2.0) as i64 % 2 == 0 {
        painter.line_segment([Pos2::new(end + 1.0, field.min.y + 5.0), Pos2::new(end + 1.0, field.max.y - 5.0)], Stroke::new(1.3, ink));
    }
    let ok_text = painter.layout_no_wrap(t.name_ok.to_owned(), FontId::proportional(13.0), Color32::WHITE);
    let ok = Rect::from_min_size(Pos2::new(field.min.x, field.max.y + 6.0), Vec2::new(ok_text.size().x + 22.0, 22.0));
    painter.rect_filled(ok, 11.0, green);
    painter.galley(ok.center() - ok_text.size() / 2.0, ok_text, Color32::WHITE);
    let skip_text = painter.layout_no_wrap(t.name_later.to_owned(), FontId::proportional(13.0), grey);
    let skip = Rect::from_min_size(Pos2::new(field.max.x - skip_text.size().x - 16.0, ok.min.y), Vec2::new(skip_text.size().x + 16.0, 22.0));
    painter.galley(skip.center() - skip_text.size() / 2.0, skip_text, grey);
    [r, field, ok, skip]
}

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

#[cfg(target_os = "macos")]
mod os {
    use std::ffi::c_void;

    use egui::{Pos2, Rect, Vec2};

    use super::Mouse;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGPoint {
        x: f64,
        y: f64,
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGEventCreate(source: *const c_void) -> *mut c_void;
        fn CGEventGetLocation(event: *mut c_void) -> CGPoint;
        fn CGEventSourceButtonState(state: i32, button: u32) -> bool;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFRelease(cf: *const c_void);
        static kCFBooleanTrue: *const c_void;
    }

    // The window server's own calls (private, used by the pointer utilities): with this property, a
    // process hides the pointer even while another one is in front.
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn _CGSDefaultConnection() -> i32;
        fn CGSSetConnectionProperty(cid: i32, target: i32, key: *const c_void, value: *const c_void) -> i32;
        fn CGDisplayHideCursor(display: u32) -> i32;
        fn CGDisplayShowCursor(display: u32) -> i32;
    }

    thread_local! {
        /// How many times the pointer was hidden, and when last.
        static HIDDEN: std::cell::Cell<(u32, Option<std::time::Instant>)> = const { std::cell::Cell::new((0, None)) };
    }

    /// The system's pointer hidden (`hide`) or shown again. Hidden once, it stays in sight on recent
    /// macOS: it is hidden again every half second while it should be, and shown again as many times
    /// as it was hidden (the window server counts them).
    pub(super) fn hide_pointer(hide: bool) {
        static ALLOWED: std::sync::Once = std::sync::Once::new();
        let (count, last) = HIDDEN.get();
        // SAFETY: the window server's calls on our own connection; the key is a CFString kept alive.
        unsafe {
            if hide {
                ALLOWED.call_once(|| {
                    let key = objc2_foundation::NSString::from_str("SetsCursorInBackground");
                    let cid = _CGSDefaultConnection();
                    CGSSetConnectionProperty(cid, cid, objc2::rc::Retained::as_ptr(&key).cast(), kCFBooleanTrue);
                });
                if last.is_none_or(|t| t.elapsed() >= std::time::Duration::from_millis(500)) {
                    CGDisplayHideCursor(0);
                    HIDDEN.set((count + 1, Some(std::time::Instant::now())));
                }
            } else if count > 0 {
                for _ in 0..count {
                    CGDisplayShowCursor(0);
                }
                HIDDEN.set((0, None));
            }
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct CGRect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGWindowListCopyWindowInfo(option: u32, relative_to: u32) -> *const c_void;
        fn CGRectMakeWithDictionaryRepresentation(dict: *const c_void, rect: *mut CGRect) -> bool;
        static kCGWindowLayer: *const c_void;
        static kCGWindowBounds: *const c_void;
        static kCGWindowOwnerPID: *const c_void;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFArrayGetCount(array: *const c_void) -> isize;
        fn CFArrayGetValueAtIndex(array: *const c_void, index: isize) -> *const c_void;
        fn CFDictionaryGetValue(dict: *const c_void, key: *const c_void) -> *const c_void;
        fn CFNumberGetValue(number: *const c_void, kind: isize, out: *mut c_void) -> bool;
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGGetDisplaysWithPoint(point: CGPoint, max: u32, displays: *mut u32, count: *mut u32) -> i32;
        fn CGDisplayCreateUUIDFromDisplayID(display: u32) -> *const c_void;
        fn CGSCopyManagedDisplaySpaces(cid: i32) -> *const c_void;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFUUIDCreateString(alloc: *const c_void, uuid: *const c_void) -> *const c_void;
        fn CFEqual(a: *const c_void, b: *const c_void) -> bool;
    }

    /// `screen` (points, top left origin) shows a film, a game, a window in full screen: its desktop
    /// is a full-screen one (an app's own, the menu bar hidden; on a screen with a notch the window
    /// stops under it, so its size alone wouldn't tell), or the app in front covers it whole. Not when
    /// that app is Ronnie.
    pub(super) fn full_screen(screen: Rect) -> bool {
        let Some(front) = objc2_app_kit::NSWorkspace::sharedWorkspace().frontmostApplication() else { return false };
        let pid = front.processIdentifier();
        if pid as u32 == std::os::unix::process::parent_id() || pid as u32 == std::process::id() {
            return false;
        }
        covered(screen, pid, full_space(screen))
    }

    /// The desktop shown on `screen` is an app's full-screen one (the window server's own list, read
    /// as the window utilities do: type 4).
    fn full_space(screen: Rect) -> bool {
        let key = |s: &str| objc2_foundation::NSString::from_str(s);
        let (display_key, current_key, type_key, main) = (key("Display Identifier"), key("Current Space"), key("type"), key("Main"));
        let ptr = |s: &objc2::rc::Retained<objc2_foundation::NSString>| objc2::rc::Retained::as_ptr(s).cast::<c_void>();
        // SAFETY: CoreGraphics' and the window server's lists, read then released; every value checked.
        unsafe {
            let mut display = 0u32;
            let mut count = 0u32;
            let center = CGPoint { x: f64::from(screen.center().x), y: f64::from(screen.center().y) };
            if CGGetDisplaysWithPoint(center, 1, &mut display, &mut count) != 0 || count == 0 {
                return false;
            }
            let uuid = CGDisplayCreateUUIDFromDisplayID(display);
            if uuid.is_null() {
                return false;
            }
            let name = CFUUIDCreateString(std::ptr::null(), uuid);
            CFRelease(uuid);
            if name.is_null() {
                return false;
            }
            let list = CGSCopyManagedDisplaySpaces(_CGSDefaultConnection());
            let mut full = false;
            if !list.is_null() {
                for k in 0..CFArrayGetCount(list) {
                    let d = CFArrayGetValueAtIndex(list, k);
                    let id = CFDictionaryGetValue(d, ptr(&display_key));
                    // « Main »: the screens share their desktops.
                    if id.is_null() || !(CFEqual(id, name) || CFEqual(id, ptr(&main))) {
                        continue;
                    }
                    let current = CFDictionaryGetValue(d, ptr(&current_key));
                    if !current.is_null() {
                        let kind = CFDictionaryGetValue(current, ptr(&type_key));
                        let mut v: i64 = 0;
                        full = !kind.is_null() && CFNumberGetValue(kind, 4, (&raw mut v).cast()) && v == 4;
                    }
                    break;
                }
                CFRelease(list);
            }
            CFRelease(name);
            full
        }
    }

    /// The app `pid` has a window covering the whole of `screen`; on a full-screen desktop
    /// (`in_space`), its top may stop under the notch.
    fn covered(screen: Rect, pid: i32, in_space: bool) -> bool {
        // SAFETY: the window server's list, read then released; every value checked before use.
        unsafe {
            // On screen only (1), without the desktop's (16).
            let list = CGWindowListCopyWindowInfo(1 | 16, 0);
            if list.is_null() {
                return false;
            }
            let number = |d: *const c_void, key: *const c_void| {
                let n = CFDictionaryGetValue(d, key);
                let mut v: i64 = 0;
                // kCFNumberSInt64Type.
                (!n.is_null() && CFNumberGetValue(n, 4, (&raw mut v).cast())).then_some(v)
            };
            let mut full = false;
            for k in 0..CFArrayGetCount(list) {
                let d = CFArrayGetValueAtIndex(list, k);
                if number(d, kCGWindowOwnerPID) != Some(i64::from(pid)) || number(d, kCGWindowLayer) != Some(0) {
                    continue;
                }
                let mut b = CGRect::default();
                let bounds = CFDictionaryGetValue(d, kCGWindowBounds);
                if !bounds.is_null() && CGRectMakeWithDictionaryRepresentation(bounds, &mut b) {
                    let r = Rect::from_min_size(Pos2::new(b.x as f32, b.y as f32), Vec2::new(b.w as f32, b.h as f32));
                    let sides = (r.min.x - screen.min.x).abs() < 2.0 && (r.max - screen.max).length() < 2.0;
                    let top = (r.min.y - screen.min.y).abs() < 2.0 || (in_space && r.min.y >= screen.min.y && r.min.y - screen.min.y < 60.0);
                    if sides && top {
                        full = true;
                        break;
                    }
                }
            }
            CFRelease(list);
            full
        }
    }

    /// The pointer shown by the system (whichever app set it) is the plain arrow: same hot spot, same
    /// size (the arrow-like ones too: the contextual menu's, the copy's...).
    pub(super) fn plain_pointer() -> bool {
        use objc2_app_kit::NSCursor;
        // Deprecated for screen captures, but the only way to see another app's pointer.
        #[allow(deprecated)]
        let Some(current) = NSCursor::currentSystemCursor() else { return true };
        let arrow = NSCursor::arrowCursor();
        let (a, b) = (current.hotSpot(), arrow.hotSpot());
        let (sa, sb) = (current.image().size(), arrow.image().size());
        (a.x - b.x).abs() < 0.5 && (a.y - b.y).abs() < 0.5 && (sa.width - sb.width).abs() < 0.5 && (sa.height - sb.height).abs() < 0.5
    }

    /// The pointer, in points from the main screen's top left (as the windows are placed).
    pub(super) fn mouse() -> Mouse {
        // SAFETY: an event made from nothing, read and released; CoreGraphics' calls are thread-safe.
        unsafe {
            let event = CGEventCreate(std::ptr::null());
            if event.is_null() {
                return Mouse::default();
            }
            let p = CGEventGetLocation(event);
            CFRelease(event);
            // The session's state (0), the left button (0).
            Mouse { at: Some(Pos2::new(p.x as f32, p.y as f32)), pressed: CGEventSourceButtonState(0, 0), right: CGEventSourceButtonState(0, 1) }
        }
    }

    /// The screen under `at`, and its part left by the Dock and the menu bar (points, top left origin).
    pub(super) fn screen_at(at: Pos2) -> Option<(Rect, Rect)> {
        let mtm = objc2::MainThreadMarker::new()?;
        let screens = objc2_app_kit::NSScreen::screens(mtm);
        // AppKit counts from the bottom of the main screen, upwards.
        let height = screens.firstObject()?.frame().size.height;
        let flip = |r: objc2_foundation::NSRect| Rect::from_min_size(Pos2::new(r.origin.x as f32, (height - r.origin.y - r.size.height) as f32), Vec2::new(r.size.width as f32, r.size.height as f32));
        screens.iter().map(|s| (flip(s.frame()), flip(s.visibleFrame()))).find(|(frame, _)| frame.contains(at))
    }

    thread_local! {
        /// The Dock's frame as last read, and when.
        static DOCK: std::cell::Cell<(Option<Rect>, Option<std::time::Instant>)> = const { std::cell::Cell::new((None, None)) };
    }

    /// The Dock's frame (points, top left origin), along the foot of a screen: read through the
    /// accessibility (allowed to the app), every two seconds. Not allowed, hidden or on a side: none.
    pub(super) fn dock() -> Option<Rect> {
        DOCK.with(|cell| {
            let (known, at) = cell.get();
            if at.is_some_and(|t| t.elapsed() < std::time::Duration::from_secs(2)) {
                return known;
            }
            let dock = dock_now();
            cell.set((dock, Some(std::time::Instant::now())));
            dock
        })
    }

    fn dock_now() -> Option<Rect> {
        #[repr(C)]
        #[derive(Default)]
        struct CGSize {
            width: f64,
            height: f64,
        }
        #[link(name = "ApplicationServices", kind = "framework")]
        unsafe extern "C" {
            fn AXIsProcessTrusted() -> bool;
            fn AXUIElementCreateApplication(pid: i32) -> *const c_void;
            fn AXUIElementCopyAttributeValue(element: *const c_void, attribute: *const c_void, value: *mut *const c_void) -> i32;
            fn AXValueGetValue(value: *const c_void, kind: u32, out: *mut c_void) -> bool;
        }
        #[link(name = "CoreFoundation", kind = "framework")]
        unsafe extern "C" {
            fn CFArrayGetCount(array: *const c_void) -> isize;
            fn CFArrayGetValueAtIndex(array: *const c_void, index: isize) -> *const c_void;
        }
        let bundle = objc2_foundation::NSString::from_str("com.apple.dock");
        let pid = objc2_app_kit::NSRunningApplication::runningApplicationsWithBundleIdentifier(&bundle).firstObject()?.processIdentifier();
        // SAFETY: CoreFoundation objects made by the copying calls, each released once; AXValues read
        // into structs of their own type (1: CGPoint, 2: CGSize).
        unsafe {
            if !AXIsProcessTrusted() {
                return None;
            }
            let app = AXUIElementCreateApplication(pid);
            if app.is_null() {
                return None;
            }
            let copy = |element: *const c_void, name: &str| -> *const c_void {
                let key = objc2_foundation::NSString::from_str(name);
                let mut value = std::ptr::null();
                if AXUIElementCopyAttributeValue(element, objc2::rc::Retained::as_ptr(&key).cast(), &mut value) == 0 { value } else { std::ptr::null() }
            };
            let children = copy(app, "AXChildren");
            let mut found = None;
            if !children.is_null() {
                for i in 0..CFArrayGetCount(children) {
                    let child = CFArrayGetValueAtIndex(children, i);
                    let (pos, size) = (copy(child, "AXPosition"), copy(child, "AXSize"));
                    let (mut p, mut s) = (CGPoint { x: 0.0, y: 0.0 }, CGSize::default());
                    if !pos.is_null() && !size.is_null() && AXValueGetValue(pos, 1, (&raw mut p).cast()) && AXValueGetValue(size, 2, (&raw mut s).cast()) {
                        found = Some(Rect::from_min_size(Pos2::new(p.x as f32, p.y as f32), Vec2::new(s.width as f32, s.height as f32)));
                    }
                    for v in [pos, size] {
                        if !v.is_null() {
                            CFRelease(v);
                        }
                    }
                    if found.is_some() {
                        break;
                    }
                }
                CFRelease(children);
            }
            CFRelease(app);
            // Along the foot, in sight (not hidden below the screen, not on a side).
            found.filter(|r| r.width() > r.height() && r.height() > 4.0 && screens().iter().any(|(s, _)| s.contains(r.center()) && r.min.y < s.max.y - 4.0))
        }
    }

    /// Every screen and its part left by the Dock and the menu bar (points, top left origin), left to
    /// right.
    pub(super) fn screens() -> Vec<(Rect, Rect)> {
        let Some(mtm) = objc2::MainThreadMarker::new() else { return Vec::new() };
        let screens = objc2_app_kit::NSScreen::screens(mtm);
        let Some(main) = screens.firstObject() else { return Vec::new() };
        let height = main.frame().size.height;
        let flip = |r: objc2_foundation::NSRect| Rect::from_min_size(Pos2::new(r.origin.x as f32, (height - r.origin.y - r.size.height) as f32), Vec2::new(r.size.width as f32, r.size.height as f32));
        let mut list: Vec<(Rect, Rect)> = screens.iter().map(|s| (flip(s.frame()), flip(s.visibleFrame()))).collect();
        list.sort_by(|a, b| a.0.min.x.total_cmp(&b.0.min.x).then(a.0.min.y.total_cmp(&b.0.min.y)));
        list
    }

    /// Every window of ours (the other screens' too) above everything, on every desktop, letting the
    /// mouse through.
    pub(super) fn tune_all() {
        use objc2_app_kit::NSWindowCollectionBehavior as B;
        let Some(mtm) = objc2::MainThreadMarker::new() else { return };
        for window in objc2_app_kit::NSApplication::sharedApplication(mtm).windows().iter() {
            if window.level() != 1000 {
                window.setLevel(1000);
                window.setCollectionBehavior(B::CanJoinAllSpaces | B::Stationary | B::FullScreenAuxiliary | B::IgnoresCycle);
                window.setHasShadow(false);
                window.setIgnoresMouseEvents(true);
            }
        }
    }

    /// Above every window, the full-screen ones too, on every desktop, out of the window cycle; never
    /// put to sleep by App Nap.
    pub(super) fn tune(cc: &eframe::CreationContext<'_>) {
        use objc2_app_kit::NSWindowCollectionBehavior as B;
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        if let Ok(handle) = cc.window_handle()
            && let RawWindowHandle::AppKit(appkit) = handle.as_raw()
        {
            // SAFETY: the handle points to the live NSView of our window, on the main thread.
            let view: &objc2_app_kit::NSView = unsafe { appkit.ns_view.cast().as_ref() };
            if let Some(window) = view.window() {
                // NSScreenSaverWindowLevel.
                window.setLevel(1000);
                window.setCollectionBehavior(B::CanJoinAllSpaces | B::Stationary | B::FullScreenAuxiliary | B::IgnoresCycle);
                window.setHasShadow(false);
                window.setIgnoresMouseEvents(true);
            }
        }
        let info = objc2_foundation::NSProcessInfo::processInfo();
        let reason = objc2_foundation::NSString::from_str("Mathéo over the screen");
        let activity = info.beginActivityWithOptions_reason(objc2_foundation::NSActivityOptions::UserInitiatedAllowingIdleSystemSleep, &reason);
        // Kept for the process's life.
        std::mem::forget(activity);
    }
}

#[cfg(windows)]
mod os {
    use egui::{Pos2, Rect};
    use windows_sys::Win32::Foundation::{LPARAM, POINT, RECT};
    use windows_sys::Win32::Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITOR_DEFAULTTONULL, MONITORINFO, MonitorFromPoint};
    use windows_sys::core::BOOL;
    use windows_sys::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON};
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

    use super::Mouse;

    /// The monitor at `p` (pixels): its rect and work area in pixels, its scale.
    fn monitor(p: POINT) -> Option<(Rect, Rect, f32)> {
        // SAFETY: a plain Win32 call.
        monitor_info(unsafe { MonitorFromPoint(p, MONITOR_DEFAULTTONULL) })
    }

    /// Monitor `m`: its rect and work area in pixels, its scale.
    fn monitor_info(m: HMONITOR) -> Option<(Rect, Rect, f32)> {
        if m.is_null() {
            return None;
        }
        // SAFETY: plain Win32 calls on values owned here.
        unsafe {
            let mut info: MONITORINFO = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            if GetMonitorInfoW(m, &mut info) == 0 {
                return None;
            }
            let (mut dx, mut dy) = (96u32, 96u32);
            let scale = if GetDpiForMonitor(m, MDT_EFFECTIVE_DPI, &mut dx, &mut dy) == 0 { dx as f32 / 96.0 } else { 1.0 };
            let rect = |r: windows_sys::Win32::Foundation::RECT| Rect::from_min_max(Pos2::new(r.left as f32, r.top as f32), Pos2::new(r.right as f32, r.bottom as f32));
            Some((rect(info.rcMonitor), rect(info.rcWork), scale))
        }
    }

    /// The pointer, in points (pixels over its monitor's scale).
    pub(super) fn mouse() -> Mouse {
        let mut p = POINT { x: 0, y: 0 };
        // SAFETY: plain Win32 calls.
        let (ok, pressed, right) = unsafe { (GetCursorPos(&mut p) != 0, (GetAsyncKeyState(VK_LBUTTON as i32) as u16 & 0x8000) != 0, (GetAsyncKeyState(VK_RBUTTON as i32) as u16 & 0x8000) != 0) };
        if !ok {
            return Mouse::default();
        }
        let scale = monitor(p).map_or(1.0, |(_, _, s)| s);
        Mouse { at: Some(Pos2::new(p.x as f32 / scale, p.y as f32 / scale)), pressed, right }
    }

    /// Windows keeps its pointer (replacing it system-wide would outlive a crash).
    pub(super) fn hide_pointer(_hide: bool) {}

    /// The window in front (not the desktop's) covers its whole monitor: a film, a game.
    pub(super) fn full_screen(_screen: Rect) -> bool {
        use windows_sys::Win32::Foundation::RECT;
        use windows_sys::Win32::Graphics::Gdi::{MONITOR_DEFAULTTONULL, MonitorFromWindow};
        use windows_sys::Win32::UI::WindowsAndMessaging::{GetDesktopWindow, GetForegroundWindow, GetShellWindow, GetWindowRect};
        // SAFETY: plain Win32 calls on values owned here.
        unsafe {
            let front = GetForegroundWindow();
            if front.is_null() || front == GetDesktopWindow() || front == GetShellWindow() {
                return false;
            }
            let mut r: RECT = std::mem::zeroed();
            if GetWindowRect(front, &mut r) == 0 {
                return false;
            }
            let m = MonitorFromWindow(front, MONITOR_DEFAULTTONULL);
            if m.is_null() {
                return false;
            }
            let mut info: MONITORINFO = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            GetMonitorInfoW(m, &mut info) != 0 && r.left <= info.rcMonitor.left && r.top <= info.rcMonitor.top && r.right >= info.rcMonitor.right && r.bottom >= info.rcMonitor.bottom
        }
    }

    /// The system's pointer is the plain arrow.
    pub(super) fn plain_pointer() -> bool {
        use windows_sys::Win32::UI::WindowsAndMessaging::{CURSORINFO, GetCursorInfo, IDC_ARROW, LoadCursorW};
        // SAFETY: plain Win32 calls on values owned here.
        unsafe {
            let mut info: CURSORINFO = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<CURSORINFO>() as u32;
            GetCursorInfo(&mut info) == 0 || info.hCursor == LoadCursorW(std::ptr::null_mut(), IDC_ARROW)
        }
    }

    /// The screen under `at` (points), and its work area.
    pub(super) fn screen_at(at: Pos2) -> Option<(Rect, Rect)> {
        // Back to pixels: the scales of the screens are tried until one holds the point.
        for scale in [1.0f32, 1.25, 1.5, 1.75, 2.0, 2.25, 2.5, 3.0] {
            let p = POINT { x: (at.x * scale) as i32, y: (at.y * scale) as i32 };
            if let Some((rect, work, s)) = monitor(p).filter(|(_, _, s)| (s - scale).abs() < 0.01) {
                let pts = |r: Rect| Rect::from_min_max((r.min.to_vec2() / s).to_pos2(), (r.max.to_vec2() / s).to_pos2());
                return Some((pts(rect), pts(work)));
            }
        }
        None
    }

    pub(super) fn tune(_cc: &eframe::CreationContext<'_>) {
        tune_all();
    }

    /// The Dock's frame: macOS only (the task bar spans the whole width).
    pub(super) fn dock() -> Option<Rect> {
        None
    }

    /// Every screen and its work area (points: each monitor's pixels over its own scale, as
    /// `screen_at` gives them), left to right.
    pub(super) fn screens() -> Vec<(Rect, Rect)> {
        unsafe extern "system" fn each(m: HMONITOR, _dc: HDC, _r: *mut RECT, list: LPARAM) -> BOOL {
            // SAFETY: `list` is the Vec handed to EnumDisplayMonitors below, alive during the call.
            let list = unsafe { &mut *(list as *mut Vec<HMONITOR>) };
            list.push(m);
            1
        }
        let mut found: Vec<HMONITOR> = Vec::new();
        // SAFETY: the callback only pushes into `found`, which outlives the call.
        unsafe { EnumDisplayMonitors(std::ptr::null_mut(), std::ptr::null(), Some(each), &raw mut found as LPARAM) };
        let pts = |r: Rect, s: f32| Rect::from_min_max((r.min.to_vec2() / s).to_pos2(), (r.max.to_vec2() / s).to_pos2());
        let mut list: Vec<(Rect, Rect)> = found.into_iter().filter_map(monitor_info).map(|(rect, work, s)| (pts(rect, s), pts(work, s))).collect();
        list.sort_by(|a, b| a.0.min.x.total_cmp(&b.0.min.x).then(a.0.min.y.total_cmp(&b.0.min.y)));
        list
    }

    /// Windows 11 draws a frame (a thin line, rounded corners) around every window, ours too though
    /// they have no decorations: over a transparent window as big as the screen, only that frame
    /// showed. Taken off each of this process's windows, once (a window is made hidden, so before
    /// it shows).
    pub(super) fn tune_all() {
        use std::collections::HashSet;
        use std::sync::Mutex;

        use windows_sys::Win32::Foundation::HWND;
        use windows_sys::Win32::Graphics::Dwm::{DWMWA_BORDER_COLOR, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND, DwmSetWindowAttribute};
        use windows_sys::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId};

        /// No colour for the frame (`DWMWA_COLOR_NONE`).
        const COLOR_NONE: u32 = 0xFFFF_FFFE;
        static DONE: Mutex<Option<HashSet<usize>>> = Mutex::new(None);

        unsafe extern "system" fn each(w: HWND, list: LPARAM) -> BOOL {
            // SAFETY: `list` is the Vec handed to EnumWindows below, alive during the call.
            let list = unsafe { &mut *(list as *mut Vec<HWND>) };
            let mut pid = 0u32;
            // SAFETY: a plain Win32 call on a window handed by EnumWindows.
            unsafe { GetWindowThreadProcessId(w, &mut pid) };
            if pid == std::process::id() {
                list.push(w);
            }
            1
        }
        let mut ours: Vec<HWND> = Vec::new();
        // SAFETY: the callback only pushes into `ours`, which outlives the call.
        unsafe { EnumWindows(Some(each), &raw mut ours as LPARAM) };
        let Ok(mut done) = DONE.lock() else { return };
        let done = done.get_or_insert_with(HashSet::new);
        // Windows gone: their handles may come back for new ones.
        done.retain(|h| ours.iter().any(|w| *w as usize == *h));
        for w in ours {
            if !done.insert(w as usize) {
                continue;
            }
            let set = |attr: i32, value: u32| {
                // SAFETY: a plain Win32 call, `value` living through it (refused before Windows 11: nothing then).
                unsafe { DwmSetWindowAttribute(w, attr as u32, (&raw const value).cast(), std::mem::size_of::<u32>() as u32) };
            };
            // Not DWMNCRP_DISABLED: winit's windows keep WS_CAPTION | WS_BORDER (their frame only
            // trimmed through WM_NCCALCSIZE), and without DWM drawing the frame, DefWindowProc painted
            // a classic one over the whole window at each WM_NCACTIVATE (him taking or losing the
            // focus): a screen-sized frame around nothing.
            set(DWMWA_BORDER_COLOR, COLOR_NONE);
            set(DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND as u32);
        }
    }
}

/// Linux: over X11 (under Wayland, through XWayland: a Wayland window can neither place itself nor stay
/// above the others), in pixels (the process is started at scale 1). Under Wayland the pointer is only
/// known while it is over an X11 window: the walkers and the horde show, the trail and the wand hardly.
#[cfg(all(unix, not(target_os = "macos")))]
mod os {
    use std::sync::OnceLock;

    use egui::{Pos2, Rect, Vec2};
    use x11rb::connection::Connection;
    use x11rb::protocol::randr::ConnectionExt as _;
    use x11rb::protocol::xproto::{AtomEnum, ClientMessageEvent, ConnectionExt as _, EventMask, KeyButMask, Window};
    use x11rb::rust_connection::RustConnection;

    use super::Mouse;

    /// The display's connection and its root window, opened once.
    fn x() -> Option<&'static (RustConnection, Window)> {
        static X: OnceLock<Option<(RustConnection, Window)>> = OnceLock::new();
        X.get_or_init(|| {
            let (conn, screen) = x11rb::connect(None).ok()?;
            let root = conn.setup().roots.get(screen)?.root;
            Some((conn, root))
        })
        .as_ref()
    }

    fn atom(conn: &RustConnection, name: &str) -> Option<u32> {
        Some(conn.intern_atom(false, name.as_bytes()).ok()?.reply().ok()?.atom)
    }

    /// A property of 32-bit values.
    fn values(conn: &RustConnection, window: Window, name: &str) -> Vec<u32> {
        let Some(prop) = atom(conn, name) else { return Vec::new() };
        conn.get_property(false, window, prop, AtomEnum::ANY, 0, 4096).ok().and_then(|c| c.reply().ok()).and_then(|r| r.value32().map(Iterator::collect)).unwrap_or_default()
    }

    /// A compositor draws the windows: without one, the transparent window would be black.
    pub(super) fn composited() -> bool {
        let Some((conn, _)) = x() else { return false };
        let screen = std::env::var("DISPLAY").ok().and_then(|d| d.rsplit('.').next().filter(|s| !s.contains(':')).and_then(|s| s.parse::<u32>().ok())).unwrap_or(0);
        atom(conn, &format!("_NET_WM_CM_S{screen}")).and_then(|a| conn.get_selection_owner(a).ok()?.reply().ok()).is_some_and(|r| r.owner != x11rb::NONE)
    }

    pub(super) fn mouse() -> Mouse {
        let Some((conn, root)) = x() else { return Mouse::default() };
        match conn.query_pointer(*root).ok().and_then(|c| c.reply().ok()) {
            Some(p) => Mouse { at: Some(Pos2::new(f32::from(p.root_x), f32::from(p.root_y))), pressed: p.mask.contains(KeyButMask::BUTTON1), right: p.mask.contains(KeyButMask::BUTTON3) },
            None => Mouse::default(),
        }
    }

    /// The system keeps its pointer (the wand drawn at its tip).
    pub(super) fn hide_pointer(_hide: bool) {}

    pub(super) fn plain_pointer() -> bool {
        true
    }

    /// The active window (not ours, nor Ronnie's) is in full screen.
    pub(super) fn full_screen(_screen: Rect) -> bool {
        let Some((conn, root)) = x() else { return false };
        let Some(&front) = values(conn, *root, "_NET_ACTIVE_WINDOW").first() else { return false };
        if front == 0 {
            return false;
        }
        let pid = values(conn, front, "_NET_WM_PID").first().copied();
        if pid.is_some_and(|p| p == std::process::id() || p == std::os::unix::process::parent_id()) {
            return false;
        }
        atom(conn, "_NET_WM_STATE_FULLSCREEN").is_some_and(|full| values(conn, front, "_NET_WM_STATE").contains(&full))
    }

    pub(super) fn screen_at(at: Pos2) -> Option<(Rect, Rect)> {
        screens().into_iter().find(|(s, _)| s.contains(at))
    }

    /// The Dock's frame: macOS only (the task bar spans the whole width).
    pub(super) fn dock() -> Option<Rect> {
        None
    }

    /// Every monitor and its part left by the panels (the desktop's work area cut to it), left to right.
    pub(super) fn screens() -> Vec<(Rect, Rect)> {
        let Some((conn, root)) = x() else { return Vec::new() };
        let Some(reply) = conn.randr_get_monitors(*root, true).ok().and_then(|c| c.reply().ok()) else { return Vec::new() };
        let area = values(conn, *root, "_NET_WORKAREA");
        let area = (area.len() >= 4).then(|| Rect::from_min_size(Pos2::new(area[0] as f32, area[1] as f32), Vec2::new(area[2] as f32, area[3] as f32)));
        let mut list: Vec<(Rect, Rect)> = reply
            .monitors
            .iter()
            .map(|m| {
                let s = Rect::from_min_size(Pos2::new(f32::from(m.x), f32::from(m.y)), Vec2::new(f32::from(m.width), f32::from(m.height)));
                let work = area.map(|a| a.intersect(s)).filter(|w| w.width() > 100.0 && w.height() > 100.0).unwrap_or(s);
                (s, work)
            })
            .collect();
        list.sort_by(|a, b| a.0.min.x.total_cmp(&b.0.min.x).then(a.0.min.y.total_cmp(&b.0.min.y)));
        list
    }

    pub(super) fn tune(_cc: &eframe::CreationContext<'_>) {}

    /// Every window of ours out of the task bar and the pager, above, on every desktop. Written on
    /// the window while it is not shown yet (the window manager reads it as it takes the window in;
    /// some docks never look again: a window per screen listed as an open app), then asked of the
    /// window manager each second for those shown without it (a window hidden then shown loses it).
    pub(super) fn tune_all() {
        use std::time::{Duration, Instant};

        use x11rb::protocol::xproto::{MapState, PropMode};
        use x11rb::wrapper::ConnectionExt as _;

        /// The root's children looked at, the listed windows not ours, those asked again, when last.
        type Seen = (Vec<Window>, Vec<Window>, Vec<Window>, Option<Instant>);
        static SEEN: std::sync::Mutex<Seen> = std::sync::Mutex::new((Vec::new(), Vec::new(), Vec::new(), None));
        let Some((conn, root)) = x() else { return };
        let me = std::process::id();
        let names = ["_NET_WM_STATE", "_NET_WM_DESKTOP", "_NET_WM_STATE_SKIP_TASKBAR", "_NET_WM_STATE_SKIP_PAGER", "_NET_WM_STATE_STICKY", "_NET_WM_STATE_ABOVE"];
        let Some([state, desktop, skip_bar, skip_pager, sticky, above]) = names.map(|n| atom(conn, n)).into_iter().collect::<Option<Vec<_>>>().and_then(|v| <[u32; 6]>::try_from(v).ok()) else { return };
        let Ok(mut seen) = SEEN.lock() else { return };
        let (children, strangers, asked, last) = &mut *seen;
        // Ours not shown yet (right under the root: a shown one is in its frame).
        if let Some(tree) = conn.query_tree(*root).ok().and_then(|c| c.reply().ok()) {
            children.retain(|w| tree.children.contains(w));
            for &w in &tree.children {
                if children.contains(&w) {
                    continue;
                }
                children.push(w);
                if values(conn, w, "_NET_WM_PID").first() != Some(&me) {
                    continue;
                }
                if conn.get_window_attributes(w).ok().and_then(|c| c.reply().ok()).is_some_and(|a| a.map_state == MapState::UNMAPPED) {
                    let _ = conn.change_property32(PropMode::REPLACE, w, state, AtomEnum::ATOM, &[skip_bar, skip_pager, above, sticky]);
                    let _ = conn.change_property32(PropMode::REPLACE, w, desktop, AtomEnum::CARDINAL, &[0xFFFF_FFFF]);
                }
            }
        }
        // Ours shown without it: asked of the window manager.
        if last.is_none_or(|t| t.elapsed() >= Duration::from_secs(1)) {
            *last = Some(Instant::now());
            let listed = values(conn, *root, "_NET_CLIENT_LIST");
            strangers.retain(|w| listed.contains(w));
            let mask = EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY;
            for w in listed {
                if strangers.contains(&w) {
                    continue;
                }
                if values(conn, w, "_NET_WM_PID").first() != Some(&me) {
                    strangers.push(w);
                    continue;
                }
                if values(conn, w, "_NET_WM_STATE").contains(&skip_bar) {
                    continue;
                }
                if !asked.contains(&w) {
                    asked.push(w);
                    crate::log::line(&format!("fenêtre {w:#x} dans la barre des tâches : sortie demandée"));
                }
                // 1: add; then the source, a normal application.
                let _ = conn.send_event(false, *root, mask, ClientMessageEvent::new(32, w, state, [1, skip_bar, skip_pager, 1, 0]));
                let _ = conn.send_event(false, *root, mask, ClientMessageEvent::new(32, w, state, [1, sticky, 0, 1, 0]));
                let _ = conn.send_event(false, *root, mask, ClientMessageEvent::new(32, w, desktop, [0xFFFF_FFFF, 1, 0, 0, 0]));
            }
        }
        let _ = conn.flush();
    }
}

#[cfg(not(any(target_os = "macos", windows, unix)))]
mod os {
    use egui::{Pos2, Rect};

    use super::Mouse;

    pub(super) fn mouse() -> Mouse {
        Mouse::default()
    }

    pub(super) fn hide_pointer(_hide: bool) {}

    pub(super) fn plain_pointer() -> bool {
        true
    }

    pub(super) fn full_screen(_screen: Rect) -> bool {
        false
    }

    pub(super) fn screen_at(_at: Pos2) -> Option<(Rect, Rect)> {
        None
    }

    pub(super) fn tune(_cc: &eframe::CreationContext<'_>) {}

    /// The Dock's frame: macOS only (the task bar spans the whole width).
    pub(super) fn dock() -> Option<Rect> {
        None
    }

    pub(super) fn screens() -> Vec<(Rect, Rect)> {
        Vec::new()
    }

    pub(super) fn tune_all() {}
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matheo_lands_at_the_foot_of_his_screen() {
        let works = [Rect::from_min_max(Pos2::new(0.0, 25.0), Pos2::new(1440.0, 860.0)), Rect::from_min_max(Pos2::new(1440.0, 0.0), Pos2::new(3360.0, 1040.0))];
        // On the second screen: its foot. Between two (past the last): back on the nearest one.
        assert_eq!(matheo_floor(Pos2::new(2000.0, 300.0), &works, None), (2000.0, 1040.0));
        assert_eq!(matheo_floor(Pos2::new(700.0, 100.0), &works, None), (700.0, 860.0));
        let (x, y) = matheo_floor(Pos2::new(3500.0, 100.0), &works, None);
        assert!(x < 3360.0 && y == 1040.0);
    }

    #[test]
    fn matheo_lands_on_the_screen_under_him_when_screens_are_stacked() {
        // A laptop under a bigger screen.
        let top = Rect::from_min_max(Pos2::new(-200.0, -1080.0), Pos2::new(1720.0, 0.0));
        let bottom = Rect::from_min_max(Pos2::new(0.0, 25.0), Pos2::new(1512.0, 982.0));
        let grounds = [top, bottom];
        assert_eq!(matheo_floor(Pos2::new(700.0, 500.0), &grounds, None), (700.0, 982.0));
        assert_eq!(matheo_floor(Pos2::new(700.0, -500.0), &grounds, None), (700.0, 0.0));
        // Between the two (the lower one's menu bar): onto the lower one.
        assert_eq!(matheo_floor(Pos2::new(700.0, 10.0), &grounds, None), (700.0, 982.0));
    }

    #[test]
    fn mathéo_says_water_and_moving_in_turns() {
        for lang in crate::i18n::Lang::ALL {
            let t = lang.strings();
            assert!(t.matheo_water.contains(&matheo_tip(lang, 0)));
            assert!(t.matheo_moves.contains(&matheo_tip(lang, 1)));
            for said in [Said::Tip(7), Said::Shaking, Said::Quiet, Said::Ouch, Said::Thanks, Said::Bed(12), Said::Thirsty, Said::Weekend(3), Said::Break(4), Said::Hello, Said::Hiding, Said::Gone] {
                assert!(!said.text(lang).is_empty(), "{said:?}");
            }
        }
        // Every one of each list comes, in turn.
        let lang = crate::i18n::Lang::Fr;
        let waters: std::collections::HashSet<_> = (0..40).step_by(2).map(|k| matheo_tip(lang, k)).collect();
        assert_eq!(waters.len(), lang.strings().matheo_water.len());
    }

    #[test]
    fn the_can_stands_on_the_free_side_and_comes_back_down() {
        let screen = [Rect::from_min_max(Pos2::ZERO, Pos2::new(1000.0, 800.0))];
        assert_eq!(can_side(Pos2::new(500.0, 800.0), &screen), 1.0);
        assert_eq!(can_side(Pos2::new(950.0, 800.0), &screen), -1.0);
        let foot = Pos2::new(500.0, 800.0);
        let rest = can_view(foot, 1.0, Instant::now() - Duration::from_secs(5), None);
        assert!((rest.origin - (foot + Vec2::new(48.0, 0.0))).length() < 0.5);
        // Up over his leaves, tipped towards him, while pouring.
        let pouring = can_view(foot, 1.0, Instant::now(), Some(Instant::now() - Duration::from_secs(1)));
        assert!(pouring.origin.y < foot.y - 80.0 && pouring.angle < -1.0, "{:?} {}", pouring.origin, pouring.angle);
        assert!(can_rect(foot, 1.0).contains(rest.origin - Vec2::new(0.0, 10.0)));
    }

    #[test]
    fn mathéo_wilts_with_thirst() {
        let mut plant = Plant { thirst: 0, ..Plant::default() };
        assert_eq!(plant.wilt(), 0.0);
        plant.thirst = THIRST_MAX + 4;
        plant.thirst = plant.thirst.min(THIRST_MAX);
        assert_eq!(plant.wilt(), 1.0);
    }
}
