#![windows_subsystem = "windows"]

mod app_model;
mod calculator;
mod db;
mod hooks;
mod scanner;
mod ui;
mod utils;

use app_model::AppManager;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::*, Win32::System::Com::*,
    Win32::System::LibraryLoader::*, Win32::System::Threading::*, Win32::UI::Controls::*,
    Win32::UI::Input::KeyboardAndMouse::*, Win32::UI::Shell::*, Win32::UI::WindowsAndMessaging::*,
};

// Global State
static mut MY_WINDOW: HWND = HWND(0);
static mut H_EDIT: HWND = HWND(0);
static mut H_LIST: HWND = HWND(0);
static mut KEYBOARD_HOOK: HHOOK = HHOOK(0);
static mut MOUSE_HOOK: HHOOK = HHOOK(0);
static APP_MANAGER: Lazy<Mutex<AppManager>> = Lazy::new(|| Mutex::new(AppManager::new()));
const WM_APP_TRAY: u32 = WM_USER + 1;
const ID_TRAY_EXIT: usize = 1001;
const ID_EDIT: i32 = 1002;
const ID_LIST: i32 = 1003;
const ID_CALC_RESULT: i32 = 1004;
static mut IS_DARK_MODE: bool = false;
static mut H_FONT: HFONT = HFONT(0);
static mut H_CALC_LABEL: HWND = HWND(0);

fn main() -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();

        IS_DARK_MODE = utils::is_dark_mode();
        utils::set_dark_mode_preference(IS_DARK_MODE);

        ui::init_common_controls();

        let _ = db::init_db();
        {
            let mut manager = APP_MANAGER.lock().unwrap();
            scanner::scan_apps(&mut manager);
        }

        let instance = GetModuleHandleW(None)?;
        let class_name = w!("OxistartClass");

        let wc = WNDCLASSW {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance.into(),
            lpszClassName: class_name,
            lpfnWndProc: Some(wnd_proc),
            hbrBackground: utils::get_background_brush(IS_DARK_MODE),
            ..Default::default()
        };

        RegisterClassW(&wc);

        MY_WINDOW = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            class_name,
            w!("Oxistart"),
            WS_POPUP,
            0,
            0,
            400,
            600,
            None,
            None,
            instance,
            None,
        );

        ui::setup_window_style(MY_WINDOW, IS_DARK_MODE);
        H_FONT = ui::create_ui_font();
        let _ = ui::add_tray_icon(MY_WINDOW);

        KEYBOARD_HOOK = hooks::setup_keyboard_hook(instance.into(), Some(keyboard_hook))?;
        MOUSE_HOOK = hooks::setup_mouse_hook(instance.into(), Some(mouse_hook))?;

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        ui::remove_tray_icon(MY_WINDOW);
        let _ = hooks::remove_hook(KEYBOARD_HOOK);
        let _ = hooks::remove_hook(MOUSE_HOOK);
        CoUninitialize();
    }
    Ok(())
}

unsafe fn update_filter(search: &str) {
    // Check if it's a mathematical expression
    if calculator::is_math_expression(search) {
        if let Some(result) = calculator::evaluate(search) {
            // Show calculation result
            let result_text = format!("= {}", result);
            let result_wide = utils::to_wide_string(&result_text);
            SetWindowTextW(H_CALC_LABEL, PCWSTR(result_wide.as_ptr()));
            ShowWindow(H_CALC_LABEL, SW_SHOW);
        } else {
            ShowWindow(H_CALC_LABEL, SW_HIDE);
        }
    } else {
        ShowWindow(H_CALC_LABEL, SW_HIDE);
    }

    let mut manager = APP_MANAGER.lock().unwrap();
    manager.filter(search);
    ui::update_listview(H_LIST, &manager);
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            H_EDIT = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                w!("EDIT"),
                None,
                WINDOW_STYLE((WS_CHILD | WS_VISIBLE | WS_BORDER).0 | ES_AUTOHSCROLL as u32),
                10,
                10,
                380,
                25,
                hwnd,
                HMENU(ID_EDIT as isize),
                None,
                None,
            );
            SendMessageW(H_EDIT, WM_SETFONT, WPARAM(H_FONT.0 as usize), LPARAM(1));

            // Create calculator result label (below the input, left-aligned)
            H_CALC_LABEL = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                None,
                WINDOW_STYLE(WS_CHILD.0 | 0x00000000), // WS_CHILD | SS_LEFT
                10,
                40,
                370,
                20,
                hwnd,
                HMENU(ID_CALC_RESULT as isize),
                None,
                None,
            );
            SendMessageW(
                H_CALC_LABEL,
                WM_SETFONT,
                WPARAM(H_FONT.0 as usize),
                LPARAM(1),
            );
            ShowWindow(H_CALC_LABEL, SW_HIDE);

            H_LIST = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                w!("SysListView32"),
                None,
                WINDOW_STYLE(
                    (WS_CHILD | WS_VISIBLE).0
                        | LVS_REPORT as u32
                        | LVS_NOCOLUMNHEADER as u32
                        | LVS_SINGLESEL as u32
                        | LVS_SHOWSELALWAYS as u32
                        | LVS_SHAREIMAGELISTS as u32,
                ),
                10,
                70,
                380,
                520,
                hwnd,
                HMENU(ID_LIST as isize),
                None,
                None,
            );

            let _ = ui::setup_listview(H_LIST, IS_DARK_MODE);
            SendMessageW(H_LIST, WM_SETFONT, WPARAM(H_FONT.0 as usize), LPARAM(1));

            let sys_img_list = ui::get_system_image_list();
            if sys_img_list != 0 {
                SendMessageW(
                    H_LIST,
                    LVM_SETIMAGELIST,
                    WPARAM(LVSIL_SMALL as usize),
                    LPARAM(sys_img_list),
                );
            }

            let mut col = LVCOLUMNW {
                mask: LVCF_WIDTH,
                cx: 350,
                ..Default::default()
            };
            SendMessageW(
                H_LIST,
                LVM_INSERTCOLUMNW,
                WPARAM(0),
                LPARAM(&mut col as *mut _ as isize),
            );
            update_filter("");
            LRESULT(0)
        }
        WM_SIZE => {
            let width = (lparam.0 & 0xFFFF) as i32;
            let height = ((lparam.0 >> 16) & 0xFFFF) as i32;
            let _ = MoveWindow(H_EDIT, 10, 10, width - 20, 25, true);
            let _ = MoveWindow(H_CALC_LABEL, 10, 40, width - 20, 20, true);
            let _ = MoveWindow(H_LIST, 10, 70, width - 20, height - 80, true);
            let mut col = LVCOLUMNW {
                mask: LVCF_WIDTH,
                cx: width - 40,
                ..Default::default()
            };
            SendMessageW(
                H_LIST,
                LVM_SETCOLUMNW,
                WPARAM(0),
                LPARAM(&mut col as *mut _ as isize),
            );
            LRESULT(0)
        }
        WM_COMMAND => {
            let id = wparam.0 & 0xFFFF;
            let code = (wparam.0 >> 16) & 0xFFFF;
            if id == ID_EDIT as usize && code == EN_CHANGE as usize {
                let len = GetWindowTextLengthW(H_EDIT);
                let mut buffer = vec![0u16; (len + 1) as usize];
                GetWindowTextW(H_EDIT, &mut buffer);
                let text = String::from_utf16_lossy(&buffer[..len as usize]);
                update_filter(&text);
            }
            if id == ID_TRAY_EXIT {
                PostQuitMessage(0);
            }
            LRESULT(0)
        }
        WM_NOTIFY => {
            let nmhdr = &*(lparam.0 as *const NMHDR);
            if nmhdr.idFrom == ID_LIST as usize {
                if nmhdr.code == NM_DBLCLK {
                    launch_selected_app();
                }
                if nmhdr.code == LVN_KEYDOWN {
                    let nmkd = &*(lparam.0 as *const NMLVKEYDOWN);
                    if nmkd.wVKey == VK_RETURN.0 {
                        launch_selected_app();
                    }
                }
            }
            LRESULT(0)
        }
        WM_CTLCOLOREDIT => {
            if IS_DARK_MODE {
                let hdc = HDC(wparam.0 as isize);
                SetTextColor(hdc, COLORREF(0x00FFFFFF));
                SetBkColor(hdc, COLORREF(0x00303030));
                return LRESULT(CreateSolidBrush(COLORREF(0x00303030)).0 as isize);
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_CTLCOLORSTATIC => {
            // Handle static control (calculator label) colors
            if IS_DARK_MODE {
                let hdc = HDC(wparam.0 as isize);
                SetTextColor(hdc, COLORREF(0x00FFFFFF)); // White text for dark mode
                SetBkColor(hdc, COLORREF(0x00202020)); // Dark background
                return LRESULT(CreateSolidBrush(COLORREF(0x00202020)).0 as isize);
            } else {
                let hdc = HDC(wparam.0 as isize);
                SetTextColor(hdc, COLORREF(0x00000000)); // Black text for light mode
                SetBkColor(hdc, COLORREF(0x00F0F0F0)); // Light background
                return LRESULT(CreateSolidBrush(COLORREF(0x00F0F0F0)).0 as isize);
            }
        }
        WM_APP_TRAY => {
            if lparam.0 as u32 == WM_RBUTTONUP {
                let mut pt = POINT::default();
                let _ = GetCursorPos(&mut pt);
                SetForegroundWindow(hwnd);
                let hmenu = CreatePopupMenu().unwrap_or(HMENU(0));
                let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_EXIT, w!("Exit Oxistart"));
                TrackPopupMenu(
                    hmenu,
                    TPM_BOTTOMALIGN | TPM_LEFTALIGN,
                    pt.x,
                    pt.y,
                    0,
                    hwnd,
                    None,
                );
                let _ = DestroyMenu(hmenu);
            }
            LRESULT(0)
        }
        WM_ACTIVATE => {
            if wparam.0 == WA_INACTIVE as usize {
                ShowWindow(hwnd, SW_HIDE);
                let _ = SetWindowTextW(H_EDIT, w!(""));
                update_filter("");
            } else {
                SetFocus(H_EDIT);
                SendMessageW(H_EDIT, EM_SETSEL, WPARAM(0), LPARAM(-1));
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn launch_selected_app_with_modifier(as_admin: bool, open_location: bool) {
    let sel = SendMessageW(
        H_LIST,
        LVM_GETNEXTITEM,
        WPARAM(usize::MAX),
        LPARAM(LVNI_SELECTED as isize),
    );
    if sel.0 == -1 {
        return;
    }

    let mut item = LVITEMW {
        mask: LVIF_PARAM,
        iItem: sel.0 as i32,
        iSubItem: 0,
        ..Default::default()
    };
    SendMessageW(
        H_LIST,
        LVM_GETITEMW,
        WPARAM(0),
        LPARAM(&mut item as *mut _ as isize),
    );
    let app_idx = item.lParam.0 as usize;

    // Obtener la información necesaria Y liberar el lock ANTES de ejecutar la app
    let (parse_name, parse_name_wide) = {
        let manager = APP_MANAGER.lock().unwrap();
        if let Some(app) = manager.apps().get(app_idx) {
            let parse_name = app.parse_name.clone();
            let parse_name_wide = utils::to_wide_string(&parse_name);
            (parse_name, parse_name_wide)
        } else {
            return;
        }
    }; // El lock se libera aquí

    // Handle different actions
    if open_location {
        // Open file location
        let verb = w!("open");
        let params = format!("/select,\"{}\"", parse_name);
        let params_wide = utils::to_wide_string(&params);
        ShellExecuteW(
            None,
            verb,
            w!("explorer.exe"),
            PCWSTR(params_wide.as_ptr()),
            None,
            SW_SHOW,
        );
    } else if as_admin {
        // Run as administrator
        let verb = w!("runas");
        ShellExecuteW(
            None,
            verb,
            PCWSTR(parse_name_wide.as_ptr()),
            None,
            None,
            SW_SHOW,
        );
    } else {
        // Normal execution
        ShellExecuteW(
            None,
            w!("open"),
            PCWSTR(parse_name_wide.as_ptr()),
            None,
            None,
            SW_SHOW,
        );
    }

    // Ocultar ventana
    ShowWindow(MY_WINDOW, SW_HIDE);
    let _ = SetWindowTextW(H_EDIT, w!(""));

    // Ahora actualizar el estado en un bloque separado
    {
        let mut manager = APP_MANAGER.lock().unwrap();
        let _ = db::increment_usage(&parse_name);
        manager.increment_usage(app_idx);
        manager.sort_by_usage();
        manager.filter("");
    } // El lock se libera aquí

    // Actualizar UI sin lock
    update_filter("");
}

unsafe fn launch_selected_app() {
    launch_selected_app_with_modifier(false, false);
}

unsafe fn launch_selected_app_as_admin() {
    launch_selected_app_with_modifier(true, false);
}

unsafe fn launch_selected_app_location() {
    launch_selected_app_with_modifier(false, true);
}

unsafe extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kbd = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        if wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize {
            if kbd.vkCode == VK_LWIN.0 as u32 || kbd.vkCode == VK_RWIN.0 as u32 {
                toggle_menu();
                return LRESULT(1);
            }
            if GetForegroundWindow() == MY_WINDOW {
                // Keep focus on the edit control always
                if GetFocus() != H_EDIT {
                    SetFocus(H_EDIT);
                }

                if kbd.vkCode == VK_DOWN.0 as u32 {
                    // Navigate down in the list without changing focus
                    let sel = SendMessageW(
                        H_LIST,
                        LVM_GETNEXTITEM,
                        WPARAM(usize::MAX),
                        LPARAM(LVNI_SELECTED as isize),
                    );
                    let count = SendMessageW(H_LIST, LVM_GETITEMCOUNT, WPARAM(0), LPARAM(0));

                    if sel.0 == -1 && count.0 > 0 {
                        // No selection, select first item
                        let mut item = LVITEMW {
                            mask: LVIF_STATE,
                            state: LIST_VIEW_ITEM_STATE_FLAGS(LVIS_SELECTED.0 | LVIS_FOCUSED.0),
                            stateMask: LIST_VIEW_ITEM_STATE_FLAGS(LVIS_SELECTED.0 | LVIS_FOCUSED.0),
                            iItem: 0,
                            ..Default::default()
                        };
                        SendMessageW(
                            H_LIST,
                            LVM_SETITEMSTATE,
                            WPARAM(0),
                            LPARAM(&mut item as *mut _ as isize),
                        );
                    } else if sel.0 < count.0 - 1 {
                        // Select next item
                        let mut item = LVITEMW {
                            mask: LVIF_STATE,
                            state: LIST_VIEW_ITEM_STATE_FLAGS(0),
                            stateMask: LIST_VIEW_ITEM_STATE_FLAGS(LVIS_SELECTED.0 | LVIS_FOCUSED.0),
                            iItem: sel.0 as i32,
                            ..Default::default()
                        };
                        SendMessageW(
                            H_LIST,
                            LVM_SETITEMSTATE,
                            WPARAM(sel.0 as usize),
                            LPARAM(&mut item as *mut _ as isize),
                        );

                        item.state = LIST_VIEW_ITEM_STATE_FLAGS(LVIS_SELECTED.0 | LVIS_FOCUSED.0);
                        item.iItem = sel.0 as i32 + 1;
                        SendMessageW(
                            H_LIST,
                            LVM_SETITEMSTATE,
                            WPARAM((sel.0 + 1) as usize),
                            LPARAM(&mut item as *mut _ as isize),
                        );
                        SendMessageW(
                            H_LIST,
                            LVM_ENSUREVISIBLE,
                            WPARAM((sel.0 + 1) as usize),
                            LPARAM(0),
                        );
                    }
                    return LRESULT(1);
                }
                if kbd.vkCode == VK_UP.0 as u32 {
                    // Navigate up in the list without changing focus
                    let sel = SendMessageW(
                        H_LIST,
                        LVM_GETNEXTITEM,
                        WPARAM(usize::MAX),
                        LPARAM(LVNI_SELECTED as isize),
                    );

                    if sel.0 > 0 {
                        // Select previous item
                        let mut item = LVITEMW {
                            mask: LVIF_STATE,
                            state: LIST_VIEW_ITEM_STATE_FLAGS(0),
                            stateMask: LIST_VIEW_ITEM_STATE_FLAGS(LVIS_SELECTED.0 | LVIS_FOCUSED.0),
                            iItem: sel.0 as i32,
                            ..Default::default()
                        };
                        SendMessageW(
                            H_LIST,
                            LVM_SETITEMSTATE,
                            WPARAM(sel.0 as usize),
                            LPARAM(&mut item as *mut _ as isize),
                        );

                        item.state = LIST_VIEW_ITEM_STATE_FLAGS(LVIS_SELECTED.0 | LVIS_FOCUSED.0);
                        item.iItem = sel.0 as i32 - 1;
                        SendMessageW(
                            H_LIST,
                            LVM_SETITEMSTATE,
                            WPARAM((sel.0 - 1) as usize),
                            LPARAM(&mut item as *mut _ as isize),
                        );
                        SendMessageW(
                            H_LIST,
                            LVM_ENSUREVISIBLE,
                            WPARAM((sel.0 - 1) as usize),
                            LPARAM(0),
                        );
                    }
                    return LRESULT(1);
                }
                if kbd.vkCode == VK_RETURN.0 as u32 {
                    let is_alt_pressed = GetKeyState(VK_MENU.0 as i32) < 0;
                    let is_shift_pressed = GetKeyState(VK_SHIFT.0 as i32) < 0;

                    if is_alt_pressed {
                        // Alt+Enter: Run as administrator
                        launch_selected_app_as_admin();
                    } else if is_shift_pressed {
                        // Shift+Enter: Open file location
                        launch_selected_app_location();
                    } else {
                        // Normal Enter: Launch app
                        launch_selected_app();
                    }
                    return LRESULT(1);
                }
                if kbd.vkCode == VK_ESCAPE.0 as u32 {
                    if IsWindowVisible(MY_WINDOW).as_bool() {
                        toggle_menu();
                        return LRESULT(1);
                    }
                }
            }
        }
        if wparam.0 == WM_KEYUP as usize || wparam.0 == WM_SYSKEYUP as usize {
            if kbd.vkCode == VK_LWIN.0 as u32 || kbd.vkCode == VK_RWIN.0 as u32 {
                return LRESULT(1);
            }
        }
    }
    CallNextHookEx(KEYBOARD_HOOK, code, wparam, lparam)
}

unsafe extern "system" fn mouse_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let msg = wparam.0 as u32;

        // Interceptar tanto el click (down) como el release (up) del botón de inicio
        if msg == WM_LBUTTONDOWN || msg == WM_LBUTTONUP {
            let mouse = &*(lparam.0 as *const MSLLHOOKSTRUCT);
            let pt = POINT {
                x: mouse.pt.x,
                y: mouse.pt.y,
            };

            if hooks::is_start_button_click(pt) {
                // Solo abrir el menú en el down, pero bloquear ambos eventos
                if msg == WM_LBUTTONDOWN {
                    toggle_menu();
                }
                return LRESULT(1);
            }
        }
    }
    CallNextHookEx(MOUSE_HOOK, code, wparam, lparam)
}

unsafe fn toggle_menu() {
    if IsWindowVisible(MY_WINDOW).as_bool() {
        ShowWindow(MY_WINDOW, SW_HIDE);
        let _ = SetWindowTextW(H_EDIT, w!(""));
        update_filter("");
    } else {
        if let Some(rect) = ui::get_target_rect() {
            let _ = SetWindowPos(
                MY_WINDOW,
                HWND_TOPMOST,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_SHOWWINDOW,
            );
            ShowWindow(MY_WINDOW, SW_SHOW);

            let foreground_thread = GetWindowThreadProcessId(GetForegroundWindow(), None);
            let my_thread = GetCurrentThreadId();
            if foreground_thread != my_thread {
                AttachThreadInput(my_thread, foreground_thread, true);
                SetForegroundWindow(MY_WINDOW);
                SetFocus(H_EDIT);
                AttachThreadInput(my_thread, foreground_thread, false);
            } else {
                SetForegroundWindow(MY_WINDOW);
                SetFocus(H_EDIT);
            }
            SendMessageW(H_EDIT, EM_SETSEL, WPARAM(0), LPARAM(-1));
        }
    }
}
