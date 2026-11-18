//! Mathéo started with the computer: a launch agent on macOS, the Run key on Windows, an autostart
//! entry on Linux. `set` says whether it is now on.

/// The program to start: the .app on macOS, the AppImage if he runs from one, else this executable.
fn program() -> Option<std::path::PathBuf> {
    if let Some(image) = std::env::var_os("APPIMAGE") {
        return Some(image.into());
    }
    let exe = std::env::current_exe().ok()?;
    #[cfg(target_os = "macos")]
    if let Some(app) = exe.ancestors().find(|p| p.extension().is_some_and(|e| e == "app")) {
        return Some(app.to_path_buf());
    }
    Some(exe)
}

#[cfg(target_os = "macos")]
pub(crate) fn set(on: bool) -> bool {
    let Some(home) = directories::BaseDirs::new().map(|b| b.home_dir().to_path_buf()) else { return false };
    let plist = home.join("Library/LaunchAgents/fr.jellfedora.matheo.plist");
    if !on {
        let _ = std::fs::remove_file(&plist);
        return false;
    }
    let Some(program) = program() else { return false };
    let escape = |s: &str| s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    let args = if program.extension().is_some_and(|e| e == "app") {
        format!("<string>/usr/bin/open</string><string>-a</string><string>{}</string>", escape(&program.to_string_lossy()))
    } else {
        format!("<string>{}</string>", escape(&program.to_string_lossy()))
    };
    let text = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n  <key>Label</key><string>fr.jellfedora.matheo</string>\n  <key>ProgramArguments</key><array>{args}</array>\n  <key>RunAtLoad</key><true/>\n</dict>\n</plist>\n"
    );
    let _ = std::fs::create_dir_all(plist.parent().unwrap_or(&home));
    std::fs::write(&plist, text).is_ok()
}

#[cfg(windows)]
pub(crate) fn set(on: bool) -> bool {
    use std::os::windows::process::CommandExt;
    const KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    let mut reg = std::process::Command::new("reg");
    if on {
        let Some(program) = program() else { return false };
        reg.args(["add", KEY, "/v", "Matheo", "/t", "REG_SZ", "/d", &format!("\"{}\"", program.display()), "/f"]);
    } else {
        reg.args(["delete", KEY, "/v", "Matheo", "/f"]);
    }
    // Without a console window flashing.
    let done = reg.creation_flags(0x0800_0000).status().is_ok_and(|s| s.success());
    on && done
}

#[cfg(all(unix, not(target_os = "macos")))]
pub(crate) fn set(on: bool) -> bool {
    let Some(dir) = directories::BaseDirs::new().map(|b| b.config_dir().join("autostart")) else { return false };
    let entry = dir.join("matheo.desktop");
    if !on {
        let _ = std::fs::remove_file(&entry);
        return false;
    }
    let Some(program) = program() else { return false };
    let text = format!("[Desktop Entry]\nType=Application\nName=Mathéo\nComment=La plante verte qui veille sur toi\nExec=\"{}\"\nX-GNOME-Autostart-enabled=true\n", program.display());
    let _ = std::fs::create_dir_all(&dir);
    std::fs::write(entry, text).is_ok()
}
