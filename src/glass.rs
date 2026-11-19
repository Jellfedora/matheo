//! Windows: what stays behind Mathéo's drawing made clear again.
//!
//! A window of ours is drawn by DirectX through DirectComposition, over the window's own surface
//! (the one GDI paints). That surface should let the screen through (winit asks for it once, when
//! the window is made), but Windows makes it again, often white and opaque, when the window
//! changes: letting clicks through or not (over Mathéo, his bubble, his menu), moving to another
//! screen, another scale. Then a white window as big as the screen shows wherever the pointer goes.
//! So, on each of this process's windows, now and after each such change: the whole window counted
//! as glass (`DwmExtendFrameIntoClientArea`, margins -1), its surface's alpha honoured (blur behind,
//! empty region), and the surface painted transparent black.

use windows_sys::Win32::Foundation::{HWND, LPARAM, RECT};
use windows_sys::Win32::Graphics::Dwm::{DWM_BB_BLURREGION, DWM_BB_ENABLE, DWM_BLURBEHIND, DwmEnableBlurBehindWindow, DwmExtendFrameIntoClientArea};
use windows_sys::Win32::Graphics::Gdi::{BLACK_BRUSH, CreateRectRgn, DeleteObject, FillRect, GetDC, GetStockObject, HBRUSH, ReleaseDC};
use windows_sys::Win32::UI::Controls::MARGINS;
use windows_sys::Win32::UI::WindowsAndMessaging::{EnumWindows, GetClientRect, GetWindowThreadProcessId};
use windows_sys::core::BOOL;

/// This process's windows.
fn ours() -> Vec<HWND> {
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
    let mut list: Vec<HWND> = Vec::new();
    // SAFETY: the callback only pushes into `list`, which outlives the call.
    unsafe { EnumWindows(Some(each), &raw mut list as LPARAM) };
    list
}

/// Every window of ours: clear behind what DirectX draws.
pub(crate) fn clear() {
    for w in ours() {
        // SAFETY: plain Win32 calls on a window of this process and on values owned here.
        unsafe {
            let glass = MARGINS { cxLeftWidth: -1, cxRightWidth: -1, cyTopHeight: -1, cyBottomHeight: -1 };
            DwmExtendFrameIntoClientArea(w, &glass);
            let region = CreateRectRgn(0, 0, -1, -1);
            let blur = DWM_BLURBEHIND { dwFlags: DWM_BB_ENABLE | DWM_BB_BLURREGION, fEnable: 1, hRgnBlur: region, fTransitionOnMaximized: 0 };
            DwmEnableBlurBehindWindow(w, &blur);
            DeleteObject(region);
            let mut r: RECT = std::mem::zeroed();
            if GetClientRect(w, &mut r) != 0 {
                let dc = GetDC(w);
                if !dc.is_null() {
                    FillRect(dc, &r, GetStockObject(BLACK_BRUSH) as HBRUSH);
                    ReleaseDC(w, dc);
                }
            }
        }
    }
}
