use crate::app_model::{AppEntry, AppManager};
use crate::db;
use crate::settings;
use crate::utils;
use std::fs::File;
use std::io::{BufRead, BufReader};
use windows::{core::*, Win32::System::Com::*};

/// Scan all applications from Start Menu shortcuts
pub unsafe fn scan_apps(app_manager: &mut AppManager) {
    app_manager.clear();

    let usage_map = db::load_usage_map();

    // Get username for Start Menu paths
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "Default".to_string());

    // Scan Start Menu folders
    let start_menu_paths = [
        format!(
            "C:\\Users\\{}\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs",
            username
        ),
        "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs".to_string(),
    ];

    for start_menu_path in start_menu_paths {
        if let Ok(entries) = std::fs::read_dir(&start_menu_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(extension) = entry.path().extension() {
                            if extension == "lnk" {
                                if let Some(app) = process_shortcut(&entry.path(), &usage_map) {
                                    app_manager.add_app(app);
                                }
                            } else if extension == "url" {
                                if let Some(app) = process_url_shortcut(&entry.path(), &usage_map) {
                                    app_manager.add_app(app);
                                }
                            }
                        }
                    } else if file_type.is_dir() {
                        // Recursively scan subdirectories
                        scan_directory_recursively(&entry.path(), app_manager, &usage_map);
                    }
                }
            }
        }
    }

    // Add Windows Settings items
    let settings_items = settings::get_settings_items();
    for settings_item in settings_items {
        let display_name = settings::get_localized_name(settings_item.canonical_name)
            .unwrap_or_else(|| settings_item.display_name_en.to_string());

        let settings_entry =
            AppEntry::new_settings(display_name, settings_item.ms_settings_uri.to_string(), -1);
        app_manager.add_app(settings_entry);
    }

    app_manager.sort_by_usage();
    app_manager.filter("");
}

/// Recursively scan a directory for shortcuts
fn scan_directory_recursively(
    dir_path: &std::path::Path,
    app_manager: &mut AppManager,
    usage_map: &std::collections::HashMap<String, i32>,
) {
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Some(extension) = entry.path().extension() {
                        if extension == "lnk" {
                            if let Some(app) = process_shortcut(&entry.path(), usage_map) {
                                app_manager.add_app(app);
                            }
                        } else if extension == "url" {
                            if let Some(app) = process_url_shortcut(&entry.path(), usage_map) {
                                app_manager.add_app(app);
                            }
                        }
                    }
                } else if file_type.is_dir() {
                    // Continue recursing
                    scan_directory_recursively(&entry.path(), app_manager, usage_map);
                }
            }
        }
    }
}

/// Process a shortcut file to extract application information
fn process_shortcut(
    shortcut_path: &std::path::Path,
    usage_map: &std::collections::HashMap<String, i32>,
) -> Option<AppEntry> {
    // Read the shortcut target
    if let Some(target_path) = get_shortcut_target(shortcut_path) {
        // Get the display name from the filename (without .lnk extension)
        let name = shortcut_path.file_stem()?.to_string_lossy().to_string();

        // Skip problematic apps
        if should_filter_app(&name, &target_path) {
            write_debug_log(&format!(
                "Filtering out problematic app: {} -> {}",
                name, target_path
            ));
            return None;
        }

        // Get icon index
        let icon_index = unsafe {
            utils::get_file_info_path(&target_path)
                .map(|shfi| shfi.iIcon)
                .unwrap_or(0)
        };

        // Get usage count
        let usage_count = *usage_map.get(&target_path).unwrap_or(&0);

        write_debug_log(&format!("Added app: {} -> {}", name, target_path));

        Some(AppEntry::new(name, target_path, icon_index, usage_count))
    } else {
        None
    }
}

/// Process a URL shortcut file (.url) to extract application information
/// These are internet shortcuts used by Steam games and other applications
fn process_url_shortcut(
    shortcut_path: &std::path::Path,
    usage_map: &std::collections::HashMap<String, i32>,
) -> Option<AppEntry> {
    // Parse the .url file
    let (url, icon_file, icon_index) = parse_url_file(shortcut_path)?;

    // Get the display name from the filename (without .url extension)
    let name = shortcut_path.file_stem()?.to_string_lossy().to_string();

    // Skip problematic entries
    if should_filter_url_shortcut(&name, &url) {
        write_debug_log(&format!("Filtering out URL shortcut: {} -> {}", name, url));
        return None;
    }

    // Get icon index - try from the icon file if specified, otherwise use default
    let final_icon_index = if let Some(ref icon_path) = icon_file {
        unsafe {
            utils::get_file_info_path(icon_path)
                .map(|shfi| shfi.iIcon)
                .unwrap_or(icon_index.unwrap_or(0))
        }
    } else {
        // Try to get icon from a known application for the protocol
        get_protocol_icon_index(&url).unwrap_or(0)
    };

    // Get usage count
    let usage_count = *usage_map.get(&url).unwrap_or(&0);

    write_debug_log(&format!("Added URL shortcut: {} -> {}", name, url));

    Some(AppEntry::new(name, url, final_icon_index, usage_count))
}

/// Parse a .url file and extract URL, IconFile, and IconIndex
fn parse_url_file(path: &std::path::Path) -> Option<(String, Option<String>, Option<i32>)> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);

    let mut url: Option<String> = None;
    let mut icon_file: Option<String> = None;
    let mut icon_index: Option<i32> = None;

    for line in reader.lines().map_while(|r| r.ok()) {
        let line = line.trim();

        if let Some(value) = line.strip_prefix("URL=") {
            url = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("IconFile=") {
            icon_file = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("IconIndex=") {
            icon_index = value.parse().ok();
        }
    }

    url.map(|u| (u, icon_file, icon_index))
}

/// Check if a URL shortcut should be filtered out
fn should_filter_url_shortcut(name: &str, url: &str) -> bool {
    // Filter out empty URLs
    if url.is_empty() {
        return true;
    }

    // Filter out uninstall shortcuts
    let name_lower = name.to_lowercase();
    if name_lower.contains("uninstall") || name_lower.contains("setup") {
        return true;
    }

    // Filter out http/https URLs (we only want protocol handlers like steam://)
    if url.starts_with("http://") || url.starts_with("https://") {
        return true;
    }

    false
}

/// Get icon index for known protocol handlers
fn get_protocol_icon_index(url: &str) -> Option<i32> {
    // For Steam URLs, try to get the Steam icon
    if url.starts_with("steam://") {
        // Try to find Steam executable for its icon
        let steam_paths = [
            "C:\\Program Files (x86)\\Steam\\steam.exe",
            "C:\\Program Files\\Steam\\steam.exe",
        ];

        for steam_path in steam_paths {
            if std::path::Path::new(steam_path).exists() {
                return unsafe { utils::get_file_info_path(steam_path).map(|shfi| shfi.iIcon) };
            }
        }
    }

    None
}

/// Get the target path from a .lnk shortcut file
fn get_shortcut_target(shortcut_path: &std::path::Path) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::System::Com::IPersistFile;
    use windows::Win32::UI::Shell::*;

    // ShellLink CLSID: {00021401-0000-0000-C000-000000000046}
    const CLSID_SHELL_LINK: GUID = GUID::from_u128(0x00021401_0000_0000_C000_000000000046);

    unsafe {
        // Create a shell link object
        let shell_link: IShellLinkW =
            CoCreateInstance(&CLSID_SHELL_LINK, None, CLSCTX_INPROC_SERVER).ok()?;

        // Get the IPersistFile interface to load the shortcut
        let persist_file: IPersistFile = shell_link.cast().ok()?;

        // Load the shortcut file
        let path_wide: Vec<u16> = shortcut_path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        persist_file
            .Load(PCWSTR(path_wide.as_ptr()), STGM(0))
            .ok()?;

        // Get the target path
        let mut target_path = [0u16; 260];
        shell_link
            .GetPath(
                &mut target_path,
                std::ptr::null_mut(),
                SLGP_RAWPATH.0 as u32,
            )
            .ok()?;

        // Convert to string
        let target_str = String::from_utf16_lossy(&target_path);
        let target_str = target_str.trim_end_matches('\0');

        if target_str.is_empty() {
            None
        } else {
            Some(target_str.to_string())
        }
    }
}

/// Check if an app should be filtered out
fn should_filter_app(name: &str, target_path: &str) -> bool {
    // Filter out Microsoft Store apps that can't be launched directly
    if target_path.contains("Microsoft.AutoGenerated.")
        || target_path.contains("WindowsApps\\")
        || name.to_lowercase().contains("uninstall")
        || name.to_lowercase().contains("setup")
    {
        return true;
    }

    false
}

/// Write a message to the debug log file
fn write_debug_log(message: &str) {
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("oxistart_debug.log")
    {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = std::io::Write::write_all(
            &mut file,
            format!("[{}] {}\n", timestamp, message).as_bytes(),
        );
    }
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
            // The scan may produce apps depending on the system
            let _ = manager.apps().len(); // Just ensure it's accessible
        }
    }
}
