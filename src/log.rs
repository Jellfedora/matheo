//! Mathéo's journal: `matheo.log` beside his settings, what happened at start (the system, the
//! graphics card, the screens, an error) and his windows' changes, to know why he doesn't show on a
//! computer we can't see. Started again at each start and past 1 MB (the previous kept as
//! `matheo.old.log`): 2 MB at most.

use std::io::Write;
use std::sync::Mutex;

/// The journal open, and what was written in it.
static FILE: Mutex<Option<(std::fs::File, u64)>> = Mutex::new(None);

/// Past this, started again.
const MAX: u64 = 1024 * 1024;

/// The journal's path.
pub(crate) fn path() -> Option<std::path::PathBuf> {
    crate::config_dir().map(|d| d.join("matheo.log"))
}

/// A new journal, the previous one kept as `matheo.old.log`.
fn renew() -> Option<std::fs::File> {
    let path = path()?;
    let _ = std::fs::rename(&path, path.with_file_name("matheo.old.log"));
    std::fs::File::create(&path).ok()
}

/// A new journal, the previous one kept; a panic written in it too.
pub(crate) fn start() {
    if let Some(file) = renew()
        && let Ok(mut f) = FILE.lock()
    {
        *f = Some((file, 0));
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        line(&format!("panic: {info}"));
        previous(info);
    }));
    line(&format!("Mathéo {} ({} {}), {}", crate::update::VERSION, std::env::consts::OS, std::env::consts::ARCH, system()));
}

/// What the system says of itself (Linux: the session, the display).
fn system() -> String {
    let vars = ["XDG_SESSION_TYPE", "XDG_CURRENT_DESKTOP", "DISPLAY", "WAYLAND_DISPLAY"];
    vars.iter().filter_map(|k| std::env::var(k).ok().map(|v| format!("{k}={v}"))).collect::<Vec<_>>().join(" ")
}

/// A line, with the time. Also on the standard error (seen when started from a terminal).
pub(crate) fn line(text: &str) {
    let text = format!("{} {text}", chrono::Local::now().format("%H:%M:%S%.3f"));
    eprintln!("{text}");
    let Ok(mut f) = FILE.lock() else { return };
    if f.as_ref().is_some_and(|(_, written)| *written > MAX) {
        *f = renew().map(|file| (file, 0));
    }
    if let Some((file, written)) = f.as_mut()
        && writeln!(file, "{text}").is_ok()
    {
        *written += text.len() as u64 + 1;
    }
}
