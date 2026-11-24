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
