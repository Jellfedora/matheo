//! Mathéo: a potted green plant living at the foot of the screen, over every window, who looks after
//! you. Every half hour a bubble reminds you to drink a glass of water or to move a little; with the
//! water comes a watering can, to water him (forgotten, he wilts; well watered, he grows and flowers).
//! Late in the evening he sends you to bed, on Friday afternoon he wears sunglasses. Grab him, throw
//! him, shake him to keep him quiet for 6 hours. A right click on him: his menu (advice, language,
//! starting with the computer, quitting).

#![windows_subsystem = "windows"]

mod access;
mod autostart;
#[cfg(windows)]
mod glass;
mod i18n;
mod overlay;
mod ronnie;
mod settings;
mod update;

use std::path::PathBuf;

/// Mathéo's own folder: his settings, where he stands, his thirst, his growth (a debug build's
/// apart, so trying one never touches the installed Mathéo).
pub(crate) fn config_dir() -> Option<PathBuf> {
    let name = if cfg!(debug_assertions) { "Matheo-dev" } else { "Matheo" };
    let dir = directories::ProjectDirs::from("", "", name)?.config_dir().to_path_buf();
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

/// His page on floor: what he is, his downloads, his last version.
pub(crate) const PAGE: &str = "https://185.245.143.130/floor/matheo";

/// A link opened in the browser.
pub(crate) fn open(url: &str) {
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // Without a console window flashing.
        let _ = std::process::Command::new("cmd").args(["/C", "start", "", url]).creation_flags(0x0800_0000).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}

fn main() {
    // One Mathéo at a time: a second start just ends (the port held for the process's life). A debug
    // build has its own, to be tried beside the installed Mathéo.
    let port = if cfg!(debug_assertions) { 47_392 } else { 47_391 };
    let Ok(_alone) = std::net::TcpListener::bind(("127.0.0.1", port)) else {
        return;
    };
    let mut settings = settings::Settings::load();
    // The first time: he starts with the computer (switched off from his menu; not a debug build).
    if settings.first {
        settings.first = false;
        settings.autostart = !cfg!(debug_assertions) && autostart::set(true);
        settings.save();
    }
    update::watch();
    overlay::run(settings);
}
