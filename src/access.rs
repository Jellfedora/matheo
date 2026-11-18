//! macOS: Mathéo stands on the Dock if allowed in Accessibility (the Dock's frame is read through it);
//! else just above it. The other systems need nothing.

/// Allowed (always, off macOS).
pub(crate) fn allowed() -> bool {
    #[cfg(target_os = "macos")]
    {
        #[link(name = "ApplicationServices", kind = "framework")]
        unsafe extern "C" {
            fn AXIsProcessTrusted() -> bool;
        }
        // SAFETY: a plain query.
        unsafe { AXIsProcessTrusted() }
    }
    #[cfg(not(target_os = "macos"))]
    true
}

/// Asks for it: macOS' own dialog (which adds Mathéo to the Accessibility list), and its settings
/// opened at that list.
pub(crate) fn ask() {
    #[cfg(target_os = "macos")]
    {
        use std::ffi::c_void;
        #[link(name = "ApplicationServices", kind = "framework")]
        unsafe extern "C" {
            fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
            static kAXTrustedCheckOptionPrompt: *const c_void;
        }
        #[link(name = "CoreFoundation", kind = "framework")]
        unsafe extern "C" {
            static kCFBooleanTrue: *const c_void;
            static kCFTypeDictionaryKeyCallBacks: c_void;
            static kCFTypeDictionaryValueCallBacks: c_void;
            fn CFDictionaryCreate(allocator: *const c_void, keys: *const *const c_void, values: *const *const c_void, count: isize, key_callbacks: *const c_void, value_callbacks: *const c_void) -> *const c_void;
            fn CFRelease(cf: *const c_void);
        }
        // SAFETY: a dictionary of one CF key and value, released once asked.
        unsafe {
            let keys = [kAXTrustedCheckOptionPrompt];
            let values = [kCFBooleanTrue];
            let options = CFDictionaryCreate(std::ptr::null(), keys.as_ptr(), values.as_ptr(), 1, &raw const kCFTypeDictionaryKeyCallBacks, &raw const kCFTypeDictionaryValueCallBacks);
            if !options.is_null() {
                AXIsProcessTrustedWithOptions(options);
                CFRelease(options);
            }
        }
        let _ = std::process::Command::new("open").arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility").spawn();
    }
}
