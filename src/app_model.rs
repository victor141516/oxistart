use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// Type of application entry
#[derive(Debug, Clone, PartialEq)]
pub enum AppEntryType {
    Application,
    Settings,
}

/// Represents an application entry with its metadata
#[derive(Debug, Clone, PartialEq)]
pub struct AppEntry {
    pub name: String,
    pub parse_name: String,
    pub arguments: Option<String>,
    pub icon_index: i32,
    pub usage_count: i32,
    pub entry_type: AppEntryType,
}

impl AppEntry {
    /// Create a new AppEntry
    pub fn new(name: String, parse_name: String, icon_index: i32, usage_count: i32) -> Self {
        Self {
            name,
            parse_name,
            arguments: None,
            icon_index,
            usage_count,
            entry_type: AppEntryType::Application,
        }
    }

    /// Create a new AppEntry with arguments
    pub fn new_with_args(
        name: String,
        parse_name: String,
        arguments: Option<String>,
        icon_index: i32,
        usage_count: i32,
    ) -> Self {
        Self {
            name,
            parse_name,
            arguments,
            icon_index,
            usage_count,
            entry_type: AppEntryType::Application,
        }
    }

    /// Create a new Settings entry
    pub fn new_settings(name: String, parse_name: String, icon_index: i32) -> Self {
        Self {
            name,
            parse_name,
            arguments: None,
            icon_index,
            usage_count: 0, // Settings items don't track usage
            entry_type: AppEntryType::Settings,
        }
    }
}

/// Manages a collection of applications
pub struct AppManager {
    apps: Vec<AppEntry>,
    filtered_indices: Vec<usize>,
}

impl AppManager {
    /// Create a new empty AppManager
    pub fn new() -> Self {
        Self {
            apps: Vec::new(),
            filtered_indices: Vec::new(),
        }
    }

    /// Add an application to the manager (with deduplication)
    pub fn add_app(&mut self, app: AppEntry) {
        // Check for duplicates by parse_name (exact match)
        if self
            .apps
            .iter()
            .any(|existing| existing.parse_name == app.parse_name)
        {
            return;
        }

        // Also check for duplicates by name (case-insensitive)
        let app_name_lower = app.name.to_lowercase();
        if self
            .apps
            .iter()
            .any(|existing| existing.name.to_lowercase() == app_name_lower)
        {
            return;
        }

        self.apps.push(app);
    }

    /// Add an application without deduplication (for loading from cache)
    pub fn add_app_unchecked(&mut self, app: AppEntry) {
        self.apps.push(app);
    }

    /// Set the apps list directly (replaces all apps)
    #[allow(dead_code)]
    pub fn set_apps(&mut self, apps: Vec<AppEntry>) {
        self.apps = apps;
        self.filtered_indices.clear();
    }

    /// Clear all applications
    pub fn clear(&mut self) {
        self.apps.clear();
        self.filtered_indices.clear();
    }

    /// Get all applications
    pub fn apps(&self) -> &[AppEntry] {
        &self.apps
    }

    /// Get filtered indices
    pub fn filtered_indices(&self) -> &[usize] {
        &self.filtered_indices
    }

    /// Sort applications by usage count (descending) and name (ascending)
    pub fn sort_by_usage(&mut self) {
        self.apps.sort_by(|a, b| {
            b.usage_count
                .cmp(&a.usage_count)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
    }

    /// Filter applications by search term using fuzzy matching
    pub fn filter(&mut self, search: &str) {
        self.filtered_indices.clear();

        if search.is_empty() {
            // If search is empty, show all apps
            for i in 0..self.apps.len() {
                self.filtered_indices.push(i);
            }
            return;
        }

        let matcher = SkimMatcherV2::default();
        let mut matches: Vec<(usize, i64)> = Vec::new();

        for (i, app) in self.apps.iter().enumerate() {
            // Try fuzzy matching on the app name
            if let Some(score) = matcher.fuzzy_match(&app.name, search) {
                matches.push((i, score));
            }
        }

        // Sort by score (highest first)
        matches.sort_by(|a, b| b.1.cmp(&a.1));

        // Store the indices of matched apps
        self.filtered_indices = matches.iter().map(|(idx, _)| *idx).collect();
    }

    /// Get an application by its index in the filtered list
    #[allow(dead_code)]
    pub fn get_filtered_app(&self, filtered_index: usize) -> Option<&AppEntry> {
        self.filtered_indices
            .get(filtered_index)
            .and_then(|&app_idx| self.apps.get(app_idx))
    }

    /// Increment usage count for an application
    pub fn increment_usage(&mut self, app_index: usize) {
        if let Some(app) = self.apps.get_mut(app_index) {
            app.usage_count += 1;
        }
    }
}

impl Default for AppManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_entry_creation() {
        let app = AppEntry::new(
            "Test App".to_string(),
            "shell:AppsFolder\\TestApp".to_string(),
            0,
            5,
        );

        assert_eq!(app.name, "Test App");
        assert_eq!(app.parse_name, "shell:AppsFolder\\TestApp");
        assert_eq!(app.icon_index, 0);
        assert_eq!(app.usage_count, 5);
        assert_eq!(app.entry_type, AppEntryType::Application);
    }

    #[test]
    fn test_settings_entry_creation() {
        let settings = AppEntry::new_settings(
            "Display settings".to_string(),
            "ms-settings:display".to_string(),
            100,
        );

        assert_eq!(settings.name, "Display settings");
        assert_eq!(settings.parse_name, "ms-settings:display");
        assert_eq!(settings.icon_index, 100);
        assert_eq!(settings.usage_count, 0);
        assert_eq!(settings.entry_type, AppEntryType::Settings);
    }

    #[test]
    fn test_app_manager_add_and_get() {
        let mut manager = AppManager::new();

        manager.add_app(AppEntry::new("App1".to_string(), "path1".to_string(), 0, 3));

        manager.add_app(AppEntry::new("App2".to_string(), "path2".to_string(), 1, 5));

        assert_eq!(manager.apps().len(), 2);
        assert_eq!(manager.apps()[0].name, "App1");
        assert_eq!(manager.apps()[1].name, "App2");
    }

    #[test]
    fn test_sort_by_usage() {
        let mut manager = AppManager::new();

        manager.add_app(AppEntry::new("App1".to_string(), "path1".to_string(), 0, 3));
        manager.add_app(AppEntry::new("App2".to_string(), "path2".to_string(), 1, 5));
        manager.add_app(AppEntry::new("App3".to_string(), "path3".to_string(), 2, 1));

        manager.sort_by_usage();

        assert_eq!(manager.apps()[0].name, "App2");
        assert_eq!(manager.apps()[1].name, "App1");
        assert_eq!(manager.apps()[2].name, "App3");
    }

    #[test]
    fn test_filter() {
        let mut manager = AppManager::new();

        manager.add_app(AppEntry::new(
            "Calculator".to_string(),
            "path1".to_string(),
            0,
            3,
        ));
        manager.add_app(AppEntry::new(
            "Calendar".to_string(),
            "path2".to_string(),
            1,
            5,
        ));
        manager.add_app(AppEntry::new(
            "Notepad".to_string(),
            "path3".to_string(),
            2,
            1,
        ));

        manager.filter("cal");

        assert_eq!(manager.filtered_indices().len(), 2);
        assert_eq!(manager.get_filtered_app(0).unwrap().name, "Calculator");
        assert_eq!(manager.get_filtered_app(1).unwrap().name, "Calendar");
    }

    #[test]
    fn test_filter_empty() {
        let mut manager = AppManager::new();

        manager.add_app(AppEntry::new("App1".to_string(), "path1".to_string(), 0, 3));
        manager.add_app(AppEntry::new("App2".to_string(), "path2".to_string(), 1, 5));

        manager.filter("");

        assert_eq!(manager.filtered_indices().len(), 2);
    }

    #[test]
    fn test_increment_usage() {
        let mut manager = AppManager::new();

        manager.add_app(AppEntry::new("App1".to_string(), "path1".to_string(), 0, 3));

        manager.increment_usage(0);

        assert_eq!(manager.apps()[0].usage_count, 4);
    }

    #[test]
    fn test_clear() {
        let mut manager = AppManager::new();

        manager.add_app(AppEntry::new("App1".to_string(), "path1".to_string(), 0, 3));
        manager.filter("");

        manager.clear();

        assert_eq!(manager.apps().len(), 0);
        assert_eq!(manager.filtered_indices().len(), 0);
    }

    #[test]
    fn test_deduplication_by_parse_name() {
        let mut manager = AppManager::new();

        manager.add_app(AppEntry::new(
            "App1".to_string(),
            "path/to/app".to_string(),
            0,
            3,
        ));
        manager.add_app(AppEntry::new(
            "App2".to_string(),
            "path/to/app".to_string(), // Same parse_name
            1,
            5,
        ));

        assert_eq!(manager.apps().len(), 1);
        assert_eq!(manager.apps()[0].name, "App1");
    }

    #[test]
    fn test_deduplication_by_name_case_insensitive() {
        let mut manager = AppManager::new();

        manager.add_app(AppEntry::new(
            "Calculator".to_string(),
            "path1".to_string(),
            0,
            3,
        ));
        manager.add_app(AppEntry::new(
            "CALCULATOR".to_string(), // Same name, different case
            "path2".to_string(),
            1,
            5,
        ));
        manager.add_app(AppEntry::new(
            "calculator".to_string(), // Same name, different case
            "path3".to_string(),
            2,
            1,
        ));

        assert_eq!(manager.apps().len(), 1);
        assert_eq!(manager.apps()[0].name, "Calculator");
    }

    #[test]
    fn test_add_app_unchecked_no_deduplication() {
        let mut manager = AppManager::new();

        manager.add_app_unchecked(AppEntry::new("App1".to_string(), "path1".to_string(), 0, 3));
        manager.add_app_unchecked(AppEntry::new(
            "App1".to_string(),  // Same name
            "path1".to_string(), // Same parse_name
            0,
            3,
        ));

        // add_app_unchecked should not deduplicate
        assert_eq!(manager.apps().len(), 2);
    }

    #[test]
    fn test_default_trait() {
        let manager = AppManager::default();
        assert_eq!(manager.apps().len(), 0);
        assert_eq!(manager.filtered_indices().len(), 0);
    }
}
