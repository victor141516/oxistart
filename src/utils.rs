use std::ffi::c_void;
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::*, Win32::Storage::FileSystem::*,
    Win32::System::LibraryLoader::*, Win32::System::Registry::*, Win32::UI::Shell::*,
};

/// Check if Windows is in dark mode
pub unsafe fn is_dark_mode() -> bool {
    let mut key = HKEY::default();
    if RegOpenKeyExW(
        HKEY_CURRENT_USER,
        w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
        0,
        KEY_READ,
        &mut key,
    )
    .is_ok()
    {
        let mut value: u32 = 0;
        let mut size = std::mem::size_of::<u32>() as u32;
        if RegQueryValueExW(
            key,
            w!("AppsUseLightTheme"),
            None,
            None,
            Some(&mut value as *mut _ as *mut u8),
            Some(&mut size),
        )
        .is_ok()
        {
            let _ = RegCloseKey(key);
            return value == 0;
        }
        let _ = RegCloseKey(key);
    }
    false
}

/// Set dark mode preference for the application
pub unsafe fn set_dark_mode_preference(dark_mode: bool) {
    if let Ok(uxtheme) = LoadLibraryW(w!("uxtheme.dll")) {
        let set_preferred: Option<unsafe extern "system" fn(i32) -> i32> =
            std::mem::transmute(GetProcAddress(uxtheme, PCSTR(135 as *const u8)));
        if let Some(func) = set_preferred {
            func(if dark_mode { 2 } else { 0 });
        }
    }
}

/// Create a solid brush with the specified color
pub unsafe fn create_brush(color: u32) -> HBRUSH {
    CreateSolidBrush(COLORREF(color))
}

/// Get the background brush for the current theme
pub unsafe fn get_background_brush(dark_mode: bool) -> HBRUSH {
    if dark_mode {
        create_brush(0x00202020)
    } else {
        HBRUSH((COLOR_WINDOW.0 + 1) as isize)
    }
}

/// Convert a string to a null-terminated wide string vector
pub fn to_wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Get file info using PIDL
#[allow(dead_code)]
pub unsafe fn get_file_info_pidl(pidl: *mut c_void) -> Option<SHFILEINFOW> {
    let mut shfi = SHFILEINFOW::default();
    let result = SHGetFileInfoW(
        PCWSTR(pidl as *const u16),
        FILE_FLAGS_AND_ATTRIBUTES(0),
        Some(&mut shfi),
        std::mem::size_of::<SHFILEINFOW>() as u32,
        SHGFI_PIDL | SHGFI_SYSICONINDEX | SHGFI_SMALLICON,
    );

    if result != 0 {
        Some(shfi)
    } else {
        None
    }
}

/// Get file info using a path string
pub unsafe fn get_file_info_path(path: &str) -> Option<SHFILEINFOW> {
    let mut shfi = SHFILEINFOW::default();
    let path_wide = to_wide_string(path);

    let result = SHGetFileInfoW(
        PCWSTR(path_wide.as_ptr()),
        FILE_FLAGS_AND_ATTRIBUTES(FILE_ATTRIBUTE_NORMAL.0),
        Some(&mut shfi),
        std::mem::size_of::<SHFILEINFOW>() as u32,
        SHGFI_SYSICONINDEX | SHGFI_SMALLICON,
    );

    if result != 0 {
        Some(shfi)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_wide_string() {
        let s = "Hello";
        let wide = to_wide_string(s);

        // Should contain the characters plus null terminator
        assert_eq!(wide.len(), 6);
        assert_eq!(wide[5], 0);

        // Convert back to string to verify
        let back = String::from_utf16_lossy(&wide[..5]);
        assert_eq!(back, "Hello");
    }

    #[test]
    fn test_to_wide_string_empty() {
        let s = "";
        let wide = to_wide_string(s);

        assert_eq!(wide.len(), 1);
        assert_eq!(wide[0], 0);
    }

    #[test]
    fn test_to_wide_string_unicode() {
        let s = "Hola 世界";
        let wide = to_wide_string(s);

        // Should have characters plus null terminator
        assert!(wide.len() > 1);
        assert_eq!(wide[wide.len() - 1], 0);

        // Convert back to verify
        let back = String::from_utf16_lossy(&wide[..wide.len() - 1]);
        assert_eq!(back, "Hola 世界");
    }

    #[test]
    fn test_create_brush() {
        unsafe {
            let brush = create_brush(0x00FF0000);
            assert_ne!(brush.0, 0);
            let _ = windows::Win32::Graphics::Gdi::DeleteObject(brush);
        }
    }

    #[test]
    fn test_get_background_brush() {
        unsafe {
            let dark_brush = get_background_brush(true);
            assert_ne!(dark_brush.0, 0);

            let light_brush = get_background_brush(false);
            assert_ne!(light_brush.0, 0);

            // Dark and light brushes should be different
            assert_ne!(dark_brush.0, light_brush.0);
        }
    }
}
