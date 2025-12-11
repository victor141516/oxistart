use windows::{core::*, Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*};

/// Setup keyboard hook
pub unsafe fn setup_keyboard_hook(instance: HINSTANCE, callback: HOOKPROC) -> Result<HHOOK> {
    SetWindowsHookExW(WH_KEYBOARD_LL, callback, instance, 0)
}

/// Setup mouse hook
pub unsafe fn setup_mouse_hook(instance: HINSTANCE, callback: HOOKPROC) -> Result<HHOOK> {
    SetWindowsHookExW(WH_MOUSE_LL, callback, instance, 0)
}

/// Remove a hook
pub unsafe fn remove_hook(hook: HHOOK) -> Result<()> {
    if !hook.is_invalid() {
        UnhookWindowsHookEx(hook)?;
    }
    Ok(())
}

/// Check if a click is on the Start button
pub unsafe fn is_start_button_click(pt: POINT) -> bool {
    let taskbar = FindWindowW(w!("Shell_TrayWnd"), None);
    let start_btn = FindWindowExW(taskbar, HWND(0), w!("Start"), None);

    if start_btn.0 != 0 {
        let mut rect = RECT::default();
        if GetWindowRect(start_btn, &mut rect).is_ok() {
            return pt.x >= rect.left
                && pt.x <= rect.right
                && pt.y >= rect.top
                && pt.y <= rect.bottom;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_start_button_click_outside() {
        unsafe {
            // Test a point that's definitely not on the start button
            let pt = POINT { x: -1000, y: -1000 };
            let result = is_start_button_click(pt);
            assert_eq!(result, false);
        }
    }
}
