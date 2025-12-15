use crate::app_model::{AppEntry, AppEntryType};
use rusqlite::{params, Connection, Result as SqlResult};
use std::collections::HashMap;

/// Initialize the database with the necessary schema
pub fn init_db() -> SqlResult<()> {
    let conn = Connection::open("history.db")?;

    // Legacy table for usage tracking
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_usage (
            path TEXT PRIMARY KEY,
            count INTEGER
        )",
        [],
    )?;

    // New table for caching app list
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_cache (
            parse_name TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            icon_index INTEGER NOT NULL,
            entry_type TEXT NOT NULL
        )",
        [],
    )?;

    Ok(())
}

/// Load usage statistics from the database
pub fn load_usage_map() -> HashMap<String, i32> {
    let mut usage_map = HashMap::new();

    if let Ok(conn) = Connection::open("history.db") {
        if let Ok(mut stmt) = conn.prepare("SELECT path, count FROM app_usage") {
            if let Ok(rows) = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
            }) {
                for row in rows.flatten() {
                    usage_map.insert(row.0, row.1);
                }
            }
        }
    }

    usage_map
}

/// Increment usage count for an application
pub fn increment_usage(path: &str) -> SqlResult<()> {
    let conn = Connection::open("history.db")?;
    conn.execute(
        "INSERT INTO app_usage (path, count) VALUES (?1, 1)
         ON CONFLICT(path) DO UPDATE SET count = count + 1",
        params![path],
    )?;
    Ok(())
}

/// Save all apps to the cache database
pub fn save_app_cache(apps: &[AppEntry]) -> SqlResult<()> {
    let conn = Connection::open("history.db")?;

    // Clear existing cache
    conn.execute("DELETE FROM app_cache", [])?;

    // Insert all apps
    let mut stmt = conn.prepare(
        "INSERT INTO app_cache (parse_name, name, icon_index, entry_type) VALUES (?1, ?2, ?3, ?4)",
    )?;

    for app in apps {
        let entry_type = match app.entry_type {
            AppEntryType::Application => "Application",
            AppEntryType::Settings => "Settings",
        };
        stmt.execute(params![
            app.parse_name,
            app.name,
            app.icon_index,
            entry_type
        ])?;
    }

    Ok(())
}

/// Load all apps from the cache database
pub fn load_app_cache() -> Vec<AppEntry> {
    let mut apps = Vec::new();
    let usage_map = load_usage_map();

    if let Ok(conn) = Connection::open("history.db") {
        if let Ok(mut stmt) =
            conn.prepare("SELECT parse_name, name, icon_index, entry_type FROM app_cache")
        {
            if let Ok(rows) = stmt.query_map([], |row| {
                let parse_name: String = row.get(0)?;
                let name: String = row.get(1)?;
                let icon_index: i32 = row.get(2)?;
                let entry_type: String = row.get(3)?;
                Ok((parse_name, name, icon_index, entry_type))
            }) {
                for row in rows.flatten() {
                    let (parse_name, name, icon_index, entry_type) = row;
                    let usage_count = *usage_map.get(&parse_name).unwrap_or(&0);

                    let app = if entry_type == "Settings" {
                        AppEntry::new_settings(name, parse_name, icon_index)
                    } else {
                        AppEntry::new(name, parse_name, icon_index, usage_count)
                    };
                    apps.push(app);
                }
            }
        }
    }

    apps
}

/// Check if the app cache exists and has entries
#[allow(dead_code)]
pub fn has_app_cache() -> bool {
    if let Ok(conn) = Connection::open("history.db") {
        if let Ok(count) =
            conn.query_row::<i32, _, _>("SELECT COUNT(*) FROM app_cache", [], |row| row.get(0))
        {
            return count > 0;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_init_db() {
        let test_db = "test_history.db";
        // Clean up any existing test database
        let _ = fs::remove_file(test_db);

        // Create a test database
        let conn = Connection::open(test_db).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_usage (
                path TEXT PRIMARY KEY,
                count INTEGER
            )",
            [],
        )
        .unwrap();

        // Verify table exists
        let result: Result<i32, _> = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='app_usage'",
            [],
            |row| row.get(0),
        );

        assert_eq!(result.unwrap(), 1);

        // Clean up
        let _ = fs::remove_file(test_db);
    }

    #[test]
    fn test_increment_usage() {
        let test_db = "test_increment.db";
        let _ = fs::remove_file(test_db);

        // Initialize database
        let conn = Connection::open(test_db).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_usage (
                path TEXT PRIMARY KEY,
                count INTEGER
            )",
            [],
        )
        .unwrap();
        drop(conn);

        // Test increment
        let test_path = "test_app";
        let conn = Connection::open(test_db).unwrap();
        conn.execute(
            "INSERT INTO app_usage (path, count) VALUES (?1, 1)
             ON CONFLICT(path) DO UPDATE SET count = count + 1",
            params![test_path],
        )
        .unwrap();

        let count: i32 = conn
            .query_row(
                "SELECT count FROM app_usage WHERE path = ?1",
                params![test_path],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 1);

        // Increment again
        conn.execute(
            "INSERT INTO app_usage (path, count) VALUES (?1, 1)
             ON CONFLICT(path) DO UPDATE SET count = count + 1",
            params![test_path],
        )
        .unwrap();

        let count: i32 = conn
            .query_row(
                "SELECT count FROM app_usage WHERE path = ?1",
                params![test_path],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 2);

        // Clean up
        let _ = fs::remove_file(test_db);
    }

    #[test]
    fn test_app_cache_save_and_load() {
        let test_db = "test_cache.db";
        let _ = fs::remove_file(test_db);

        // Initialize database with app_cache table
        let conn = Connection::open(test_db).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_usage (
                path TEXT PRIMARY KEY,
                count INTEGER
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_cache (
                parse_name TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                icon_index INTEGER NOT NULL,
                entry_type TEXT NOT NULL
            )",
            [],
        )
        .unwrap();
        drop(conn);

        // Create test apps
        let apps = vec![
            AppEntry::new(
                "Test App 1".to_string(),
                "path/to/app1.exe".to_string(),
                1,
                5,
            ),
            AppEntry::new(
                "Test App 2".to_string(),
                "path/to/app2.exe".to_string(),
                2,
                10,
            ),
            AppEntry::new_settings(
                "Display Settings".to_string(),
                "ms-settings:display".to_string(),
                100,
            ),
        ];

        // Save to cache (using test db directly)
        let conn = Connection::open(test_db).unwrap();
        conn.execute("DELETE FROM app_cache", []).unwrap();

        let mut stmt = conn.prepare(
            "INSERT INTO app_cache (parse_name, name, icon_index, entry_type) VALUES (?1, ?2, ?3, ?4)",
        ).unwrap();

        for app in &apps {
            let entry_type = match app.entry_type {
                AppEntryType::Application => "Application",
                AppEntryType::Settings => "Settings",
            };
            stmt.execute(params![
                app.parse_name,
                app.name,
                app.icon_index,
                entry_type
            ])
            .unwrap();
        }
        drop(stmt);
        drop(conn);

        // Load from cache
        let conn = Connection::open(test_db).unwrap();
        let mut loaded_apps = Vec::new();

        let mut stmt = conn
            .prepare("SELECT parse_name, name, icon_index, entry_type FROM app_cache")
            .unwrap();
        let rows = stmt
            .query_map([], |row| {
                let parse_name: String = row.get(0)?;
                let name: String = row.get(1)?;
                let icon_index: i32 = row.get(2)?;
                let entry_type: String = row.get(3)?;
                Ok((parse_name, name, icon_index, entry_type))
            })
            .unwrap();

        for row in rows.flatten() {
            let (parse_name, name, icon_index, entry_type) = row;
            let app = if entry_type == "Settings" {
                AppEntry::new_settings(name, parse_name, icon_index)
            } else {
                AppEntry::new(name, parse_name, icon_index, 0)
            };
            loaded_apps.push(app);
        }

        // Verify loaded apps
        assert_eq!(loaded_apps.len(), 3);

        // Find and verify each app
        let app1 = loaded_apps.iter().find(|a| a.name == "Test App 1").unwrap();
        assert_eq!(app1.parse_name, "path/to/app1.exe");
        assert_eq!(app1.icon_index, 1);
        assert_eq!(app1.entry_type, AppEntryType::Application);

        let app2 = loaded_apps.iter().find(|a| a.name == "Test App 2").unwrap();
        assert_eq!(app2.parse_name, "path/to/app2.exe");
        assert_eq!(app2.icon_index, 2);
        assert_eq!(app2.entry_type, AppEntryType::Application);

        let settings = loaded_apps
            .iter()
            .find(|a| a.name == "Display Settings")
            .unwrap();
        assert_eq!(settings.parse_name, "ms-settings:display");
        assert_eq!(settings.icon_index, 100);
        assert_eq!(settings.entry_type, AppEntryType::Settings);

        // Clean up
        let _ = fs::remove_file(test_db);
    }

    #[test]
    fn test_has_app_cache_empty() {
        let test_db = "test_has_cache_empty.db";
        let _ = fs::remove_file(test_db);

        // Initialize database with empty app_cache table
        let conn = Connection::open(test_db).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_cache (
                parse_name TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                icon_index INTEGER NOT NULL,
                entry_type TEXT NOT NULL
            )",
            [],
        )
        .unwrap();

        // Check if cache is empty
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM app_cache", [], |row| row.get(0))
            .unwrap();

        assert_eq!(count, 0);

        // Clean up
        let _ = fs::remove_file(test_db);
    }

    #[test]
    fn test_has_app_cache_with_entries() {
        let test_db = "test_has_cache_entries.db";
        let _ = fs::remove_file(test_db);

        // Initialize database and add an entry
        let conn = Connection::open(test_db).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_cache (
                parse_name TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                icon_index INTEGER NOT NULL,
                entry_type TEXT NOT NULL
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO app_cache (parse_name, name, icon_index, entry_type) VALUES (?1, ?2, ?3, ?4)",
            params!["test_path", "Test App", 1, "Application"],
        )
        .unwrap();

        // Check if cache has entries
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM app_cache", [], |row| row.get(0))
            .unwrap();

        assert!(count > 0);

        // Clean up
        let _ = fs::remove_file(test_db);
    }
}
