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
    /// Windows: frames left to clear what stays behind the drawing (see `glass`), after a change
    /// that may have made it white.
    glass: u8,
}

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
    Dock,
    Quit,
}

/// Frames during which the windows are cleared after a change (Windows applies it a little later).
const GLASS: u8 = 4;

/// Windows: clears behind the drawing of each window of ours (see `crate::glass`).
fn clear_glass() {
    #[cfg(windows)]
    crate::glass::clear();
}

const MENU_W: f32 = 240.0;
const ROW_H: f32 = 30.0;

/// His menu's rows: what each says, whether it's a switch (and on), what it does.
fn menu_rows(settings: &Settings, quiet: bool) -> Vec<(String, Option<bool>, Choice)> {
    let t = settings.lang.strings();
    let mut rows = vec![
        (t.menu_version.replace("{v}", crate::update::VERSION), None, Choice::Version),
        (t.menu_tips_on.to_owned(), Some(settings.tips), Choice::Tips),
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
    rows.push((t.menu_quit.to_owned(), None, Choice::Quit));
    rows
}

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
        painter.text(row.left_center() + Vec2::new(10.0, 0.0), egui::Align2::LEFT_CENTER, label, FontId::proportional(14.0), color);
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
        eprintln!("Mathéo : pas de compositeur, rien à afficher");
        return;
    }
    let mouse = Arc::new(Mutex::new(Mouse::default()));
    let ctx_slot: Arc<Mutex<Option<egui::Context>>> = Arc::new(Mutex::new(None));
    // The pointer, read about 60 times a second; a move draws again.
    {
        let (mouse, ctx_slot) = (mouse.clone(), ctx_slot.clone());
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(16));
                let now = os::mouse();
                let changed = mouse.lock().map(|mut m| std::mem::replace(&mut *m, now) != now).unwrap_or(false);
                if changed && let Some(ctx) = ctx_slot.lock().ok().and_then(|c| c.clone()) {
                    ctx.request_repaint();
                }
            }
        });
    }

    let start = os::mouse().at.and_then(os::screen_at).map(|(screen, _)| screen).unwrap_or(Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0)));
    let viewport = egui::ViewportBuilder::default()
        .with_title("Mathéo")
        .with_decorations(false)
        .with_transparent(true)
        .with_mouse_passthrough(true)
        .with_window_level(egui::WindowLevel::AlwaysOnTop)
        .with_taskbar(false)
        .with_active(false)
        .with_has_shadow(false)
        .with_resizable(false)
        .with_position(start.min)
        .with_inner_size(start.size());
    #[cfg_attr(not(any(windows, target_os = "macos")), allow(unused_mut))]
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
    let first = !matheo_file().is_some_and(|f| f.exists());
    let result = eframe::run_native(
        "matheo",
        options,
        Box::new(move |cc| {
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
            Ok(Box::new(Overlay { mouse, settings, screen: start, work, pressed: false, right: false, last: None, full: (Vec::new(), None), matheo, menu: None, told_newer: false, told_failed: false, leaving: None, others: Default::default(), season: Season::now(), glass: GLASS }))
        }),
    );
    if let Err(e) = result {
        eprintln!("Mathéo : {e}");
    }
}

impl Overlay {
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
            // Linux: the window manager told about our windows once it lists them (Windows: their
            // frame taken off again, should one have slipped through).
            #[cfg(any(windows, all(unix, not(target_os = "macos"))))]
            os::tune_all();
            clear_glass();
        }
        let covered = |full: &[Rect], screen: Rect| full.iter().any(|f| (f.min - screen.min).length() < 1.0 && (f.max - screen.max).length() < 1.0);
        if covered(&self.full.0, self.screen) {
            ctx.request_repaint_after(Duration::from_secs(1));
            return;
        }
        // The pointer went to another screen: the window follows it (not while his menu shows).
        if let Some(at) = mouse.at
            && self.menu.is_none()
            && !self.screen.contains(at)
            && let Some((screen, work)) = os::screen_at(at)
        {
            self.screen = screen;
            self.work = work;
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(screen.min));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(screen.size()));
            self.glass = GLASS;
        }
        // Where this window really is: macOS may have slid it under the menu bar (and cut it to fit), so
        // everything is drawn from its true top left, not the screen's.
        let origin = window_origin(&ctx, self.screen.min);
        let pressed = mouse.pressed && !self.pressed;
        self.pressed = mouse.pressed;
        let right_click = mouse.right && !self.right;
        self.right = mouse.right;
        let screens = os::screens();
        let works: Vec<Rect> = if screens.is_empty() { vec![self.work] } else { screens.iter().map(|s| s.1).collect() };
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
        if pressed
            && matches!(self.matheo.tip, Some((Said::Update, _)))
            && self.matheo.bubble.zip(mouse.at).is_some_and(|(r, p)| r.contains(p))
        {
            crate::update::install();
            self.matheo.tip = Some((Said::Installing, Instant::now()));
        }
        // His menu: a click on a row does it, a click anywhere else closes it.
        let quiet = self.matheo.quiet.is_some_and(|until| unix_now() < until);
        let rows = menu_rows(&self.settings, quiet);
        let menu = self.menu.map(|at| menu_rect(at, rows.len(), self.screen));
        let on_menu = menu.zip(mouse.at).is_some_and(|(r, p)| r.contains(p));
        let hover = menu.zip(mouse.at).filter(|(r, p)| r.contains(*p)).map(|(r, p)| (((p.y - r.min.y - 6.0) / ROW_H).floor().max(0.0) as usize).min(rows.len() - 1));
        let mut click = pressed;
        if pressed && let Some(menu) = menu {
            click = false;
            self.menu = None;
            if menu.contains(mouse.at.unwrap_or(Pos2::ZERO))
                && let Some(i) = hover
            {
                self.choose(rows[i].2);
            }
        }
        let mut open_menu = None;
//@BLOCK@
            let pass = !(over || plant.grab.is_some() || on_menu);
            if pass != plant.pass {
                plant.pass = pass;
                ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(pass));
                self.glass = GLASS;
            }
        }
        if let Some(at) = open_menu {
            self.menu = Some(at);
        }
        if self.matheo.tired.1.is_none_or(|t| t.elapsed() >= Duration::from_secs(60)) {
            self.matheo.tired = (late(), Some(Instant::now()));
            self.matheo.friday = friday();
            self.season = Season::now();
        }
        let m = &self.matheo;
        let look = m.look(m.tired.0, m.friday);
        let can = m.at.zip(m.can).map(|(at, (since, pour))| can_view(at, can_side(at, &works), since, pour));
        let plant = m.at.map(|at| (at, look));
        let season = self.season;
        let tip = m.tip.map(|(said, _)| said.text(self.settings.lang));

        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("matheo")));
        let mut busy = plant_busy;
        let mut next: f32 = 1.0;
        let here = screens.iter().position(|(s, _)| (s.min - self.screen.min).length() < 1.0).unwrap_or(0);
        // Mathéo on the screen he's on.
        let plant_on = plant.map(|(at, _)| screens.iter().position(|(s, _)| s.contains(at - Vec2::new(0.0, 1.0))).unwrap_or(here));
        if let Some(menu) = self.menu.map(|at| menu_rect(at, rows.len(), self.screen)) {
            paint_menu(&painter, menu.translate(-origin.to_vec2()), &rows, hover);
            busy = true;
        }
        // Mathéo is drawn by a window of his own over his screen, never by this one: this one follows
        // the pointer from screen to screen (for his menu), he stays still meanwhile (drawn by the
        // window going away, then by the one arriving, he jumped a little at each change). His window
        // is made hidden, shown once drawn (else, on Windows, a white square first), hidden again
        // when he's no longer on its screen, closed a while after.
        for (i, (screen, _)) in screens.iter().enumerate() {
            let needed = plant_on == Some(i);
            if covered(&self.full.0, *screen) {
                self.others.remove(&i);
                continue;
            }
            if needed {
                self.others.entry(i).or_insert(Other { frames: 0, needed: Instant::now() }).needed = Instant::now();
            }
            let Some(other) = self.others.get_mut(&i).filter(|o| o.needed.elapsed() < OTHER_KEPT) else {
                self.others.remove(&i);
                continue;
            };
            let visible = needed && other.frames >= 2;
            if other.frames < 2 + u32::from(GLASS) {
                // Just made or shown: cleared too.
                self.glass = self.glass.max(1);
            }
            other.frames = other.frames.saturating_add(1);
            if needed && !visible {
                // Drawn again at once, to be shown.
                next = next.min(0.02);
            }
            let screen = *screen;
            let builder = egui::ViewportBuilder::default()
                .with_visible(visible)
                .with_title("Mathéo")
                .with_decorations(false)
                .with_transparent(true)
                .with_mouse_passthrough(true)
                .with_window_level(egui::WindowLevel::AlwaysOnTop)
                .with_taskbar(false)
                .with_active(false)
                .with_has_shadow(false)
                .with_resizable(false)
                .with_position(screen.min)
                .with_inner_size(screen.size());
            let bubble = ctx.show_viewport_immediate(egui::ViewportId::from_hash_of(("matheo-screen", i)), builder, |ctx, _| {
                // Windows places a new window with the scale of the screen it was made on: put back
                // over its screen, with that screen's own scale (as the main window is when it moves).
                if cfg!(windows)
                    && let Some(r) = ctx.input(|i| i.viewport().outer_rect)
                    && ((r.min - screen.min).length() > 1.0 || (r.size() - screen.size()).length() > 1.0)
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(screen.min));
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(screen.size()));
                }
                if !needed {
                    return None;
                }
                // Drawn from where the window really is (macOS slides a new one under the menu bar).
                let origin = window_origin(ctx, screen.min);
                let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("matheo")));
                let (Some((at, look)), true) = (plant, plant_on == Some(i)) else {
                    return None;
                };
                paint_matheo_as(&painter, at - origin.to_vec2(), MATHEO_H, now, look);
                if let Some(season) = season {
                    paint_season(&painter, at - origin.to_vec2(), MATHEO_H, now, look, season);
                }
                if let Some(can) = can {
                    paint_can(&painter, can, origin.to_vec2());
                }
                tip.map(|text| paint_matheo_tip(&painter, at - origin.to_vec2(), text, screen.translate(-origin.to_vec2())).translate(origin.to_vec2()))
            });
            if bubble.is_some() {
                self.matheo.bubble = bubble;
            }
            os::tune_all();
            if needed {
                // His leaves sway gently (what floats around him in season, smoothly).
                next = next.min(if season.is_some() { 0.04 } else { 0.1 });
            }
        }
        if cfg!(debug_assertions) && std::env::var_os("MATHEO_GALLERY").is_some() {
            paint_matheo_gallery(&painter, self.screen.translate(-origin.to_vec2()), now);
            paint_season_gallery(&painter, self.screen.translate(-origin.to_vec2()), now);
            busy = true;
        }
        if self.glass > 0 {
            self.glass -= 1;
            clear_glass();
            busy = true;
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

//@SEASON@

//@OS@

//@TESTS@
