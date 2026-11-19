//! Ronnie (or Mathéo himself) in front: Mathéo stays over it, even full screen (he only steps aside
//! for a film, a game). In Ronnie, his own Mathéo is Ronnie's child and never hides from it; here,
//! Ronnie is another app. On Windows, a click on Mathéo brings his window (as big as the screen) in
//! front: taken for a film, it hid him.

/// The app in front is Ronnie.
#[cfg(target_os = "macos")]
pub(crate) fn in_front() -> bool {
    objc2_app_kit::NSWorkspace::sharedWorkspace()
        .frontmostApplication()
        .and_then(|app| app.bundleIdentifier())
        .is_some_and(|id| id.to_string() == "com.jellfedora.ronnie")
}

/// The window in front is Ronnie's (ronnie.exe) or Mathéo's own.
#[cfg(windows)]
pub(crate) fn in_front() -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW};
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    // SAFETY: plain Win32 calls on values owned here; the process handle closed.
    unsafe {
        let front = GetForegroundWindow();
        if front.is_null() {
            return false;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(front, &mut pid);
        if pid == std::process::id() {
            return true;
        }
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return false;
        }
        let mut name = [0u16; 1024];
        let mut len = name.len() as u32;
        let ok = QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, name.as_mut_ptr(), &mut len) != 0;
        CloseHandle(process);
        ok && String::from_utf16_lossy(&name[..len as usize]).rsplit(['\\', '/']).next().is_some_and(|f| f.eq_ignore_ascii_case("ronnie.exe"))
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
pub(crate) fn in_front() -> bool {
    false
}
