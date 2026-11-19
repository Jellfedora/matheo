//! Windows: each window of ours put over its screen in pixels, by Windows itself.
//!
//! egui places a window in points, turned into pixels with the scale of the screen the window is on
//! at that moment. Screens at different scales (125 % here, 100 % there): sent to the other screen,
//! the window landed elsewhere, was sent again, changed screen and scale on the way, and so on:
//! windows showing and hiding. Here the screen's own pixels are asked for (`SetWindowPos`), checked
//! at each frame, and set again only when they differ (once more after Windows, changing the
//! window's scale on the way, resized it).

use std::sync::Mutex;

use egui::{Pos2, Rect};
use windows_sys::Win32::Foundation::{HWND, LPARAM, RECT};
use windows_sys::Win32::Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO};
use windows_sys::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, SWP_NOACTIVATE, SWP_NOOWNERZORDER, SWP_NOZORDER, SetWindowPos,
};
use windows_sys::core::BOOL;

/// Every screen: its rect in points (its pixels over its own scale, as Mathéo counts them) and in
/// pixels.
fn screens() -> Vec<(Rect, RECT)> {
    unsafe extern "system" fn each(m: HMONITOR, _dc: HDC, _r: *mut RECT, list: LPARAM) -> BOOL {
        // SAFETY: `list` is the Vec handed to EnumDisplayMonitors below, alive during the call.
        let list = unsafe { &mut *(list as *mut Vec<HMONITOR>) };
        list.push(m);
        1
    }
    let mut found: Vec<HMONITOR> = Vec::new();
    // SAFETY: the callback only pushes into `found`, which outlives the call.
    unsafe { EnumDisplayMonitors(std::ptr::null_mut(), std::ptr::null(), Some(each), &raw mut found as LPARAM) };
    found
        .into_iter()
        .filter_map(|m| {
            // SAFETY: plain Win32 calls on a monitor just listed and on values owned here.
            unsafe {
                let mut info: MONITORINFO = std::mem::zeroed();
                info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                if GetMonitorInfoW(m, &mut info) == 0 {
                    return None;
                }
                let (mut dx, mut dy) = (96u32, 96u32);
                let scale = if GetDpiForMonitor(m, MDT_EFFECTIVE_DPI, &mut dx, &mut dy) == 0 { dx as f32 / 96.0 } else { 1.0 };
                let r = info.rcMonitor;
                let points = Rect::from_min_max(Pos2::new(r.left as f32 / scale, r.top as f32 / scale), Pos2::new(r.right as f32 / scale, r.bottom as f32 / scale));
                Some((points, r))
            }
        })
        .collect()
}

/// This process's window titled `title`.
fn window(title: &str) -> Option<HWND> {
    struct Search {
        title: Vec<u16>,
        found: Option<HWND>,
    }
    unsafe extern "system" fn each(w: HWND, search: LPARAM) -> BOOL {
        // SAFETY: `search` is the Search handed to EnumWindows below, alive during the call.
        let search = unsafe { &mut *(search as *mut Search) };
        let mut pid = 0u32;
        // SAFETY: plain Win32 calls on a window handed by EnumWindows, into a buffer owned here.
        unsafe {
            GetWindowThreadProcessId(w, &mut pid);
            if pid != std::process::id() {
                return 1;
            }
            let mut text = [0u16; 128];
            let n = GetWindowTextW(w, text.as_mut_ptr(), text.len() as i32);
            if n > 0 && text[..n as usize] == search.title[..] {
                search.found = Some(w);
                return 0;
            }
        }
        1
    }
    let mut search = Search { title: title.encode_utf16().collect(), found: None };
    // SAFETY: the callback only touches `search`, which outlives the call.
    unsafe { EnumWindows(Some(each), &raw mut search as LPARAM) };
    search.found
}

/// Times a window was put back, said in the journal (not more: a loop would fill it).
static TOLD: Mutex<u32> = Mutex::new(0);

/// The window titled `title` over `screen` (points, as `os::screens` gives it): its pixels set when
/// they differ from the screen's.
pub(crate) fn put(title: &str, screen: Rect) {
    let Some((_, px)) = screens().into_iter().find(|(points, _)| (points.min - screen.min).length() < 1.0 && (points.max - screen.max).length() < 1.0) else {
        return;
    };
    let Some(w) = window(title) else { return };
    // SAFETY: plain Win32 calls on a window of this process and on values owned here.
    unsafe {
        let mut now: RECT = std::mem::zeroed();
        if GetWindowRect(w, &mut now) == 0 {
            return;
        }
        if (now.left, now.top, now.right, now.bottom) == (px.left, px.top, px.right, px.bottom) {
            return;
        }
        if let Ok(mut told) = TOLD.lock()
            && *told < 100
        {
            *told += 1;
            crate::log::line(&format!(
                "« {title} » remise sur son écran : ({}, {}) – ({}, {}) → ({}, {}) – ({}, {}) pixels",
                now.left, now.top, now.right, now.bottom, px.left, px.top, px.right, px.bottom
            ));
        }
        SetWindowPos(w, std::ptr::null_mut(), px.left, px.top, px.right - px.left, px.bottom - px.top, SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER);
    }
}
