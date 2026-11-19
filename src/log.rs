//! Mathéo's journal: `matheo.log` beside his settings, what happened at start (the system, the
//! graphics card, the screens, an error) and his windows' changes, to know why he doesn't show on a
//! computer we can't see. Started again at each start (the previous kept as `matheo.old.log`).

use std::io::Write;
use std::sync::Mutex;

static FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

/// The journal's path.
pub(crate) fn path() -> Option<std::path::PathBuf> {
    crate::config_dir().map(|d| d.join("matheo.log"))
}

/// A new journal, the previous one kept; a panic written in it too.
pub(crate) fn start() {
    let Some(path) = path() else { return };
    let _ = std::fs::rename(&path, path.with_file_name("matheo.old.log"));
    if let Ok(file) = std::fs::File::create(&path)
        && let Ok(mut f) = FILE.lock()
    {
        *f = Some(file);
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
    if let Ok(mut f) = FILE.lock()
        && let Some(file) = f.as_mut()
    {
        let _ = writeln!(file, "{text}");
    }
}
