use once_cell::sync::Lazy;
use rusqlite::{Connection, Result as SqliteResult};
use std::sync::Mutex;

pub mod migrations;

static DB_CONNECTION: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

pub fn init_database(db_path: &str) -> anyhow::Result<()> {
    let conn = Connection::open(db_path)?;

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Run migrations
    migrations::run_migrations(&conn)?;

    let mut db = DB_CONNECTION.lock().unwrap();
    *db = Some(conn);

    log::info!("Database initialized at: {}", db_path);
    Ok(())
}

pub fn get_connection() -> anyhow::Result<std::sync::MutexGuard<'static, Option<Connection>>> {
    let guard = DB_CONNECTION.lock().map_err(|e| {
        anyhow::anyhow!("Failed to lock database connection: {}", e)
    })?;
    Ok(guard)
}

pub fn with_connection<F, T>(f: F) -> anyhow::Result<T>
where
    F: FnOnce(&Connection) -> SqliteResult<T>,
{
    let guard = get_connection()?;
    let conn = guard.as_ref().ok_or_else(|| {
        anyhow::anyhow!("Database not initialized")
    })?;
    f(conn).map_err(|e| anyhow::anyhow!("Database error: {}", e))
}
