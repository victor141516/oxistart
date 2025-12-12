/// Represents a Windows Settings item
#[derive(Debug, Clone)]
pub struct SettingsItem {
    pub canonical_name: &'static str,
    pub display_name_en: &'static str,
    pub ms_settings_uri: &'static str,
}

/// Get the list of common Windows Settings items
pub fn get_settings_items() -> Vec<SettingsItem> {
    vec![
        SettingsItem {
            canonical_name: "Display",
            display_name_en: "Display settings",
            ms_settings_uri: "ms-settings:display",
        },
        SettingsItem {
            canonical_name: "Sound",
            display_name_en: "Sound settings",
            ms_settings_uri: "ms-settings:sound",
        },
        SettingsItem {
            canonical_name: "Network",
            display_name_en: "Network settings",
            ms_settings_uri: "ms-settings:network",
        },
        SettingsItem {
            canonical_name: "Bluetooth",
            display_name_en: "Bluetooth settings",
            ms_settings_uri: "ms-settings:bluetooth",
        },
        SettingsItem {
            canonical_name: "Printers",
            display_name_en: "Printers & scanners",
            ms_settings_uri: "ms-settings:printers",
        },
        SettingsItem {
            canonical_name: "Apps",
            display_name_en: "Apps & features",
            ms_settings_uri: "ms-settings:appsfeatures",
        },
        SettingsItem {
            canonical_name: "Power",
            display_name_en: "Power & sleep",
            ms_settings_uri: "ms-settings:powersleep",
        },
        SettingsItem {
            canonical_name: "Storage",
            display_name_en: "Storage settings",
            ms_settings_uri: "ms-settings:storagesense",
        },
        SettingsItem {
            canonical_name: "Personalization",
            display_name_en: "Personalization",
            ms_settings_uri: "ms-settings:personalization",
        },
        SettingsItem {
            canonical_name: "Time",
            display_name_en: "Date & time",
            ms_settings_uri: "ms-settings:dateandtime",
        },
        SettingsItem {
            canonical_name: "Language",
            display_name_en: "Language settings",
            ms_settings_uri: "ms-settings:regionlanguage",
        },
        SettingsItem {
            canonical_name: "Updates",
            display_name_en: "Windows Update",
            ms_settings_uri: "ms-settings:windowsupdate",
        },
        SettingsItem {
            canonical_name: "Privacy",
            display_name_en: "Privacy settings",
            ms_settings_uri: "ms-settings:privacy",
        },
        SettingsItem {
            canonical_name: "Accounts",
            display_name_en: "Your account",
            ms_settings_uri: "ms-settings:yourinfo",
        },
        SettingsItem {
            canonical_name: "WiFi",
            display_name_en: "Wi-Fi settings",
            ms_settings_uri: "ms-settings:network-wifi",
        },
    ]
}

/// Get localized display name for a settings item
pub unsafe fn get_localized_name(canonical_name: &str) -> Option<String> {
    // Try to get localized name from Settings app resources
    // For now, we'll use a fallback to the English name
    // In a production app, you would use MUI (Multilingual User Interface) APIs

    // Fallback: use display name from our list
    for item in get_settings_items() {
        if item.canonical_name == canonical_name {
            return Some(item.display_name_en.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_settings_items() {
        let items = get_settings_items();
        assert!(!items.is_empty());
        assert!(items.iter().any(|i| i.canonical_name == "Display"));
    }

    #[test]
    fn test_get_localized_name() {
        unsafe {
            let name = get_localized_name("Display");
            assert!(name.is_some());
        }
    }
}
