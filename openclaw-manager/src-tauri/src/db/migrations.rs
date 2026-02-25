use rusqlite::{Connection, Result};

const MIGRATIONS: &[&str] = &[
    // Migration 1: Initial schema
    r#"
    CREATE TABLE IF NOT EXISTS configs (
        id TEXT PRIMARY KEY,
        key TEXT UNIQUE NOT NULL,
        value TEXT NOT NULL,
        description TEXT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS files (
        id TEXT PRIMARY KEY,
        file_name TEXT NOT NULL,
        file_path TEXT UNIQUE NOT NULL,
        file_type TEXT NOT NULL,
        file_size INTEGER NOT NULL DEFAULT 0,
        description TEXT,
        tags TEXT,
        is_collected BOOLEAN DEFAULT 0,
        is_classified BOOLEAN DEFAULT 0,
        classification TEXT,
        custom_attributes TEXT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    CREATE INDEX IF NOT EXISTS idx_files_type ON files(file_type);
    CREATE INDEX IF NOT EXISTS idx_files_collected ON files(is_collected);
    CREATE INDEX IF NOT EXISTS idx_files_classified ON files(is_classified);

    CREATE TABLE IF NOT EXISTS groups (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT,
        icon TEXT,
        color TEXT,
        sort_order INTEGER DEFAULT 0,
        is_default BOOLEAN DEFAULT 0,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS file_groups (
        file_id TEXT NOT NULL,
        group_id TEXT NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (file_id, group_id),
        FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
        FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS plugins (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        version TEXT NOT NULL,
        description TEXT,
        author TEXT,
        plugin_type TEXT NOT NULL,
        entry_point TEXT NOT NULL,
        is_enabled BOOLEAN DEFAULT 0,
        config_schema TEXT,
        default_config TEXT,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS plugin_configs (
        plugin_id TEXT PRIMARY KEY,
        config TEXT NOT NULL,
        FOREIGN KEY (plugin_id) REFERENCES plugins(id) ON DELETE CASCADE
    );

    -- Insert default group
    INSERT OR IGNORE INTO groups (id, name, description, is_default, sort_order)
    VALUES ('default', '未分组', '默认分组，未分类的文件', 1, 0);
    "#,
];

pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Create migrations table if not exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS __migrations (
            version INTEGER PRIMARY KEY,
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM __migrations",
            [],
            |row| row.get(0),
        )?;

    for (i, migration) in MIGRATIONS.iter().enumerate() {
        let version = (i + 1) as i32;
        if version > current_version {
            conn.execute_batch(migration)?;
            conn.execute(
                "INSERT INTO __migrations (version) VALUES (?1)",
                [version],
            )?;
            log::info!("Applied migration version {}", version);
        }
    }

    Ok(())
}
