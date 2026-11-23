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

use crate::settings::Settings;

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
    /// What the journal last said of the windows (said again when it changes).
    logged: String,
    /// Windows: frames left to clear what stays behind the drawing (see `glass`), after a change
    /// that may have made it white.
    glass: u8,
    /// Where his eyes look (see `Look::gaze`), going smoothly towards the pointer near him.
    gaze: Vec2,
}

/// How near the pointer comes for his eyes to follow it (points from his face).
const GAZE_FROM: f32 = 260.0;

//@PLANT@

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
    if page == Page::Hours {
        let h = settings.hours;
        let names = [t.menu_hour_morning, t.menu_hour_lunch, t.menu_hour_afternoon, t.menu_hour_home];
        let mut rows = vec![(t.menu_back.to_owned(), None, Choice::Back), (t.menu_hours_on.to_owned(), Some(h.on), Choice::HoursOn)];
        rows.extend(names.iter().enumerate().map(|(k, name)| (format!("{name}\t{:2}:{:02}", h.at[k] / 60, h.at[k] % 60), None, Choice::Hour(k))));
        rows.push((t.menu_hours_reset.to_owned(), None, Choice::HoursReset));
        return rows;
    }
    let mut rows = vec![
        (t.menu_version.replace("{v}", crate::update::VERSION), None, Choice::Version),
        (t.menu_tips_on.to_owned(), Some(settings.tips), Choice::Tips),
        (t.menu_hours.to_owned(), None, Choice::Hours),
        (t.menu_name.to_owned(), None, Choice::Name),
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

/// His menu's pages.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Page {
    Main,
    /// His time reminders.
    Hours,
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
    Say(Own),
}

const DEMOS: [(&str, Demo); 19] = [
    ("Parachute", Demo::Chute),
    ("Chute et atterrissage", Demo::Land),
    ("Glissade", Demo::Slide),
    ("Présentation (1er lancement)", Demo::Hello),
    ("Demande du prénom", Demo::Say(Own::AskName)),
    ("Prénom retenu", Demo::Say(Own::Named)),
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
];

/// Where his menu shows (screen points): from the click, kept within the screen.
fn menu_rect(at: Pos2, rows: usize, screen: Rect) -> Rect {
    let size = Vec2::new(MENU_W, rows as f32 * ROW_H + 12.0);
    let mut min = at + Vec2::new(-size.x * 0.5, -size.y - 12.0);
    min.x = min.x.clamp(screen.min.x + 6.0, (screen.max.x - size.x - 6.0).max(screen.min.x + 6.0));
    min.y = min.y.max(screen.min.y + 6.0);
    Rect::from_min_size(min, size)
}

/// His menu: a light card, the row under the pointer lit, switches on the right.
fn paint_menu(painter: &egui::Painter, rect: Rect, rows: &[(String, Option<bool>, Choice)], hover: Option<usize>) {
    let ink = Color32::from_rgb(46, 64, 36);
    let green = Color32::from_rgb(76, 160, 70);
    painter.rect_filled(rect.translate(Vec2::new(0.0, 3.0)), 12.0, Color32::from_black_alpha(50));
    painter.rect(rect, 12.0, Color32::from_rgb(250, 248, 234), Stroke::new(1.5, Color32::from_rgb(104, 156, 84)), egui::StrokeKind::Inside);
    for (i, (label, switch, choice)) in rows.iter().enumerate() {
        let row = Rect::from_min_size(rect.min + Vec2::new(6.0, 6.0 + i as f32 * ROW_H), Vec2::new(rect.width() - 12.0, ROW_H));
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

pub(crate) fn run(settings: Settings) {
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
            Ok(Box::new(Overlay { mouse, hot: hot.clone(), settings, screen: start, work, pressed: false, right: false, last: None, full: (Vec::new(), None), matheo, menu: None, told_newer: false, told_failed: false, leaving: None, others: Default::default(), season: Season::now(), antics: Antics::default(), care: Care::new(first), page: Page::Main, naming: None, logged: String::new(), glass: GLASS, gaze: Vec2::ZERO }))
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
            Demo::Say(own) => m.tip = Some((Said::Own(own), now)),
        }
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
                if matches!(choice, Choice::Hours | Choice::Demo | Choice::Back | Choice::HoursOn | Choice::Hour(_) | Choice::HoursReset) {
                    self.menu = at;
                }
            }
        }
        let mut open_menu = None;
        let catching;
//@BLOCK@
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
        self.care.step(&mut self.matheo, self.settings.tips, self.settings.hours, self.settings.name.is_some(), mouse.at);
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
        let m = &self.matheo;
        let look = Look { gaze: self.gaze, ..self.antics.dress(m.look(m.tired.0, m.friday), m) };
        let can = m.at.zip(m.can).map(|(at, (since, pour))| can_view(at, can_side(at, &works), since, pour));
        let plant = m.at.map(|at| (at, look));
        let season = self.season;
        let antics = &self.antics;
        let mut keys = Vec::new();
        let naming = self.naming.as_ref().map(|n| Naming { typed: n.typed.clone(), keys: n.keys, rects: None });
        let t = self.settings.lang.strings();
        // His words: one of a few at each bubble, with your name now and then.
        let v = self.care.variant(m.tip.map(|(_, since)| since));
        let tip = m.tip.map(|(said, since)| {
            let text = match said {
                Said::Own(Own::Breathe) => breathe_text(since.elapsed().as_secs_f32(), t, v),
                Said::Own(own) => own.text(t, v).to_owned(),
                Said::Ouch => pick(t.more_ouch, v).to_owned(),
                Said::Shaking => pick(t.more_shaking, v).to_owned(),
                Said::Thanks => pick(t.more_thanks, v).to_owned(),
                Said::Thirsty => pick(t.more_thirsty, v).to_owned(),
                _ => said.text(self.settings.lang).to_owned(),
            };
            personal(said, &text, self.settings.name.as_deref(), t)
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
        if menu.is_some() || gallery {
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
            let (bubble, name_rects, typed_keys) = ctx.show_viewport_immediate(egui::ViewportId::from_hash_of(("matheo-screen", i)), builder, |ctx, _| {
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
                let mut typed_keys = Vec::new();
                if let (Some((at, look)), true) = (plant, his) {
                    paint_antics_behind(&painter, antics, at - origin.to_vec2(), MATHEO_H, look, origin.to_vec2());
                    paint_matheo_as(&painter, at - origin.to_vec2(), MATHEO_H, now, look);
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
                    } else {
                        bubble = tip.as_deref().map(|text| paint_matheo_tip(&painter, at - origin.to_vec2(), text, screen.translate(-origin.to_vec2())).translate(origin.to_vec2()));
                    }
                }
                // His menu above his bubble.
                if let Some(menu) = menu_here {
                    paint_menu(&painter, menu.translate(-origin.to_vec2()), &rows, hover);
                }
                if gallery && i == home {
                    paint_matheo_gallery(&painter, screen.translate(-origin.to_vec2()), now);
                    paint_season_gallery(&painter, screen.translate(-origin.to_vec2()), now);
                    paint_antics_gallery(&painter, screen.translate(-origin.to_vec2()), now);
                }
                (bubble, name_rects, typed_keys)
            });
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

//@CARE@

//@SEASON@

//@ANTICS@

//@OS@

//@TESTS@
