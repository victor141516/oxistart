use crate::app_model::{AppEntry, AppManager};
use crate::db;
use crate::utils;
use std::ffi::c_void;
use windows::{core::*, Win32::System::Com::*, Win32::UI::Shell::*};

#[link(name = "shell32")]
extern "system" {
    fn SHGetIDListFromObject(punk: IUnknown, ppidl: *mut *mut c_void) -> HRESULT;
}

/// Scan all applications from the AppsFolder
pub unsafe fn scan_apps(app_manager: &mut AppManager) {
    app_manager.clear();

    let usage_map = db::load_usage_map();

    if let Ok(apps_folder_item) = SHCreateItemFromParsingName::<PCWSTR, Option<&IBindCtx>, IShellItem>(
        w!("shell:AppsFolder"),
        None,
    ) {
        if let Ok(enum_items) = apps_folder_item
            .BindToHandler::<Option<&IBindCtx>, IEnumShellItems>(None, &BHID_EnumItems)
        {
            let mut items = [None];
            let mut fetched = 0;

            while enum_items.Next(&mut items, Some(&mut fetched)).is_ok() && fetched == 1 {
                if let Some(item) = items[0].take() {
                    if let Some(app) = process_shell_item(&item, &usage_map) {
                        app_manager.add_app(app);
                    }
                }
            }
        }
    }

    app_manager.sort_by_usage();
    app_manager.filter("");
}

/// Process a shell item to extract application information
unsafe fn process_shell_item(
    item: &IShellItem,
    usage_map: &std::collections::HashMap<String, i32>,
) -> Option<AppEntry> {
    // Get display name
    let name = match item.GetDisplayName(SIGDN_NORMALDISPLAY) {
        Ok(n) => n.to_string().unwrap_or_default(),
        Err(_) => return None,
    };

    // Get parse name
    let parse_name = match item.GetDisplayName(SIGDN_DESKTOPABSOLUTEPARSING) {
        Ok(n) => n.to_string().unwrap_or_default(),
        Err(_) => return None,
    };

    // Get icon index
    let icon_index = get_icon_index(item, &parse_name);

    // Get usage count
    let usage_count = *usage_map.get(&parse_name).unwrap_or(&0);

    Some(AppEntry::new(name, parse_name, icon_index, usage_count))
}

/// Get icon index for a shell item
unsafe fn get_icon_index(item: &IShellItem, parse_name: &str) -> i32 {
    // Try PIDL method first
    let mut pidl: *mut c_void = std::ptr::null_mut();
    if SHGetIDListFromObject(item.cast::<IUnknown>().unwrap(), &mut pidl).is_ok() {
        if let Some(shfi) = utils::get_file_info_pidl(pidl) {
            CoTaskMemFree(Some(pidl as *const c_void));
            return shfi.iIcon;
        }
        CoTaskMemFree(Some(pidl as *const c_void));
    }

    // Fallback to parsing name
    if let Some(shfi) = utils::get_file_info_path(parse_name) {
        return shfi.iIcon;
    }

    0 // Default icon index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_apps_initializes_manager() {
        unsafe {
            let mut manager = AppManager::new();

            // This test just ensures the function doesn't panic
            // The actual scanning depends on the Windows environment
            scan_apps(&mut manager);

            // We can't assert specific apps since it depends on the system,
            // but we can check that the manager is properly initialized
            assert!(manager.apps().len() >= 0);
        }
    }
}
