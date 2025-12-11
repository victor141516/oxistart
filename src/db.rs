use rusqlite::{params, Connection, Result as SqlResult};
use std::collections::HashMap;

/// Initialize the database with the necessary schema
pub fn init_db() -> SqlResult<()> {
    let conn = Connection::open("history.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_usage (
            path TEXT PRIMARY KEY,
            count INTEGER
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
}
