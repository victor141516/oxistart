use crate::app_model::AppManager;
use crate::utils;
use std::ffi::c_void;
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Dwm::*, Win32::Graphics::Gdi::*,
    Win32::Storage::FileSystem::*, Win32::UI::Controls::*, Win32::UI::Shell::*,
    Win32::UI::WindowsAndMessaging::*,
};

/// Initialize common controls for the application
pub unsafe fn init_common_controls() {
    let icce = INITCOMMONCONTROLSEX {
        dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
        dwICC: ICC_LISTVIEW_CLASSES | ICC_STANDARD_CLASSES,
    };
    InitCommonControlsEx(&icce);
}

/// Create a font for the UI
pub unsafe fn create_ui_font() -> HFONT {
    let mut lf = LOGFONTW::default();
    lf.lfHeight = -14;
    lf.lfWeight = 400;
    let font_name = utils::to_wide_string("Segoe UI");
    lf.lfFaceName[..font_name.len().min(32)].copy_from_slice(&font_name[..font_name.len().min(32)]);
    CreateFontIndirectW(&lf)
}

/// Setup window with rounded corners and dark mode
pub unsafe fn setup_window_style(hwnd: HWND, dark_mode: bool) {
    // Set rounded corners
    let rounded_pref = DWMWCP_ROUND;
    let _ = DwmSetWindowAttribute(
        hwnd,
        DWMWA_WINDOW_CORNER_PREFERENCE,
        &rounded_pref as *const _ as *const c_void,
        std::mem::size_of::<u32>() as u32,
    );

    // Apply dark mode if needed
    if dark_mode {
        let dark_mode_value = 1;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark_mode_value as *const _ as *const c_void,
            std::mem::size_of::<u32>() as u32,
        );
    }
}

/// Setup list view with dark mode support
pub unsafe fn setup_listview(hwnd: HWND, dark_mode: bool) -> Result<()> {
    if dark_mode {
        SetWindowTheme(hwnd, w!("DarkMode_Explorer"), None)?;
        let dark_grey = COLORREF(0x00202020);
        let white = COLORREF(0x00FFFFFF);
        SendMessageW(
            hwnd,
            LVM_SETBKCOLOR,
            WPARAM(0),
            LPARAM(dark_grey.0 as isize),
        );
        SendMessageW(
            hwnd,
            LVM_SETTEXTBKCOLOR,
            WPARAM(0),
            LPARAM(dark_grey.0 as isize),
        );
        SendMessageW(hwnd, LVM_SETTEXTCOLOR, WPARAM(0), LPARAM(white.0 as isize));
    } else {
        SetWindowTheme(hwnd, w!("Explorer"), None)?;
    }

    SendMessageW(
        hwnd,
        LVM_SETEXTENDEDLISTVIEWSTYLE,
        WPARAM(LVS_EX_FULLROWSELECT as usize | LVS_EX_DOUBLEBUFFER as usize),
        LPARAM(LVS_EX_FULLROWSELECT as isize | LVS_EX_DOUBLEBUFFER as isize),
    );

    Ok(())
}

/// Get system image list for icons
pub unsafe fn get_system_image_list() -> isize {
    let mut shfi = SHFILEINFOW::default();
    SHGetFileInfoW(
        w!(""),
        FILE_FLAGS_AND_ATTRIBUTES(FILE_ATTRIBUTE_NORMAL.0),
        Some(&mut shfi),
        std::mem::size_of::<SHFILEINFOW>() as u32,
        SHGFI_SYSICONINDEX | SHGFI_SMALLICON | SHGFI_USEFILEATTRIBUTES,
    ) as isize
}

/// Update the list view with filtered applications
pub unsafe fn update_listview(list_hwnd: HWND, app_manager: &AppManager) {
    SendMessageW(list_hwnd, LVM_DELETEALLITEMS, WPARAM(0), LPARAM(0));
    SendMessageW(
        list_hwnd,
        LVM_SETITEMCOUNT,
        WPARAM(app_manager.filtered_indices().len()),
        LPARAM(0),
    );

    for (list_idx, &app_idx) in app_manager.filtered_indices().iter().enumerate() {
        if let Some(app) = app_manager.apps().get(app_idx) {
            let mut name_wide = utils::to_wide_string(&app.name);
            let mut item = LVITEMW {
                mask: LVIF_TEXT | LVIF_IMAGE | LVIF_PARAM,
                iItem: list_idx as i32,
                iSubItem: 0,
                pszText: PWSTR(name_wide.as_mut_ptr()),
                iImage: app.icon_index,
                lParam: LPARAM(app_idx as isize),
                ..Default::default()
            };
            SendMessageW(
                list_hwnd,
                LVM_INSERTITEMW,
                WPARAM(0),
                LPARAM(&mut item as *mut _ as isize),
            );
        }
    }

    // Select first item if available
    if !app_manager.filtered_indices().is_empty() {
        let mut item = LVITEMW {
            mask: LVIF_STATE,
            state: LIST_VIEW_ITEM_STATE_FLAGS(LVIS_SELECTED.0 | LVIS_FOCUSED.0),
            stateMask: LIST_VIEW_ITEM_STATE_FLAGS(LVIS_SELECTED.0 | LVIS_FOCUSED.0),
            ..Default::default()
        };
        SendMessageW(
            list_hwnd,
            LVM_SETITEMSTATE,
            WPARAM(0),
            LPARAM(&mut item as *mut _ as isize),
        );
    }
}

/// Add a system tray icon
pub unsafe fn add_tray_icon(hwnd: HWND) -> Result<()> {
    let mut tooltip = [0u16; 128];
    let msg_wide = utils::to_wide_string("StartWin");
    for (i, &c) in msg_wide.iter().enumerate() {
        if i < 127 {
            tooltip[i] = c;
        }
    }

    let nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: WM_USER + 1,
        hIcon: LoadIconW(None, IDI_APPLICATION).unwrap_or(HICON(0)),
        szTip: tooltip,
        ..Default::default()
    };

    Shell_NotifyIconW(NIM_ADD, &nid);
    Ok(())
}

/// Remove the system tray icon
pub unsafe fn remove_tray_icon(hwnd: HWND) {
    let nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1,
        ..Default::default()
    };
    Shell_NotifyIconW(NIM_DELETE, &nid);
}

/// Get the target rectangle for positioning the menu window
pub unsafe fn get_target_rect() -> Option<RECT> {
    let taskbar_window = FindWindowW(w!("Shell_TrayWnd"), None);
    if taskbar_window.0 == 0 {
        return None;
    }

    let mut taskbar_rect = RECT::default();
    GetWindowRect(taskbar_window, &mut taskbar_rect).ok()?;

    let menu_width = 400;
    let menu_height = 600;
    let mut x = 0;
    let mut y = 0;

    let width = taskbar_rect.right - taskbar_rect.left;
    let height = taskbar_rect.bottom - taskbar_rect.top;

    if width > height {
        // Horizontal taskbar
        if taskbar_rect.top >= 0 && taskbar_rect.bottom >= GetSystemMetrics(SM_CYSCREEN) - 50 {
            y = taskbar_rect.top - menu_height;
        } else {
            y = taskbar_rect.bottom;
        }
    } else {
        // Vertical taskbar
        if taskbar_rect.left < 50 {
            x = taskbar_rect.right;
        } else {
            x = taskbar_rect.left - menu_width;
        }
    }

    Some(RECT {
        left: x,
        top: y,
        right: x + menu_width,
        bottom: y + menu_height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ui_font() {
        unsafe {
            let font = create_ui_font();
            assert_ne!(font.0, 0);
        }
    }

    #[test]
    fn test_get_system_image_list() {
        unsafe {
            let img_list = get_system_image_list();
            assert_ne!(img_list, 0);
        }
    }
}
