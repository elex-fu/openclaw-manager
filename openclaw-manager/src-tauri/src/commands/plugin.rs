use crate::db;
use crate::models::{
    plugin::{InstallPluginRequest, Plugin, PluginMarketItem},
    ApiResponse,
};
use rusqlite::params;
use uuid::Uuid;

#[tauri::command]
pub fn get_plugins() -> ApiResponse<Vec<Plugin>> {
    match db::with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, name, version, description, author, plugin_type,
             entry_point, is_enabled, config_schema, default_config, created_at, updated_at
             FROM plugins ORDER BY created_at DESC"
        )?;

        let plugins = stmt.query_map([], |row| {
            Ok(Plugin {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                description: row.get(3)?,
                author: row.get(4)?,
                plugin_type: row.get(5)?,
                entry_point: row.get(6)?,
                is_enabled: row.get(7)?,
                config_schema: row.get(8)?,
                default_config: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?;

        plugins.collect::<Result<Vec<_>, _>>()
    }) {
        Ok(plugins) => ApiResponse::success(plugins),
        Err(e) => {
            log::error!("Failed to get plugins: {}", e);
            ApiResponse::error(format!("Failed to get plugins: {}", e))
        }
    }
}

#[tauri::command]
pub fn install_plugin(req: InstallPluginRequest) -> ApiResponse<Plugin> {
    // TODO: Download and install plugin from market
    // For now, create a placeholder plugin
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let plugin = Plugin {
        id: id.clone(),
        name: "示例插件".to_string(),
        version: "1.0.0".to_string(),
        description: Some("示例插件描述".to_string()),
        author: Some("OpenClaw".to_string()),
        plugin_type: "lua".to_string(),
        entry_point: format!("plugins/{}/main.lua", id),
        is_enabled: false,
        config_schema: None,
        default_config: None,
        created_at: now.clone(),
        updated_at: now,
    };

    match db::with_connection(|conn| {
        conn.execute(
            "INSERT INTO plugins (id, name, version, description, author, plugin_type,
             entry_point, is_enabled, config_schema, default_config, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                &plugin.id, &plugin.name, &plugin.version, &plugin.description,
                &plugin.author, &plugin.plugin_type, &plugin.entry_point,
                plugin.is_enabled, &plugin.config_schema, &plugin.default_config,
                &plugin.created_at, &plugin.updated_at
            ],
        )?;
        Ok(plugin.clone())
    }) {
        Ok(p) => ApiResponse::success(p),
        Err(e) => {
            log::error!("Failed to install plugin: {}", e);
            ApiResponse::error(format!("Failed to install plugin: {}", e))
        }
    }
}

#[tauri::command]
pub fn uninstall_plugin(id: String) -> ApiResponse<bool> {
    match db::with_connection(|conn| {
        let rows = conn.execute(
            "DELETE FROM plugins WHERE id = ?1",
            params![id],
        )?;
        Ok(rows > 0)
    }) {
        Ok(deleted) => ApiResponse::success(deleted),
        Err(e) => {
            log::error!("Failed to uninstall plugin: {}", e);
            ApiResponse::error(format!("Failed to uninstall plugin: {}", e))
        }
    }
}

#[tauri::command]
pub fn enable_plugin(id: String) -> ApiResponse<Plugin> {
    let now = chrono::Utc::now().to_rfc3339();

    match db::with_connection(|conn| {
        conn.execute(
            "UPDATE plugins SET is_enabled = 1, updated_at = ?1 WHERE id = ?2",
            params![&now, &id],
        )?;

        conn.query_row(
            "SELECT id, name, version, description, author, plugin_type,
             entry_point, is_enabled, config_schema, default_config, created_at, updated_at
             FROM plugins WHERE id = ?1",
            params![&id],
            |row| {
                Ok(Plugin {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    version: row.get(2)?,
                    description: row.get(3)?,
                    author: row.get(4)?,
                    plugin_type: row.get(5)?,
                    entry_point: row.get(6)?,
                    is_enabled: row.get(7)?,
                    config_schema: row.get(8)?,
                    default_config: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            },
        )
    }) {
        Ok(plugin) => ApiResponse::success(plugin),
        Err(e) => {
            log::error!("Failed to enable plugin: {}", e);
            ApiResponse::error(format!("Failed to enable plugin: {}", e))
        }
    }
}

#[tauri::command]
pub fn disable_plugin(id: String) -> ApiResponse<Plugin> {
    let now = chrono::Utc::now().to_rfc3339();

    match db::with_connection(|conn| {
        conn.execute(
            "UPDATE plugins SET is_enabled = 0, updated_at = ?1 WHERE id = ?2",
            params![&now, &id],
        )?;

        conn.query_row(
            "SELECT id, name, version, description, author, plugin_type,
             entry_point, is_enabled, config_schema, default_config, created_at, updated_at
             FROM plugins WHERE id = ?1",
            params![&id],
            |row| {
                Ok(Plugin {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    version: row.get(2)?,
                    description: row.get(3)?,
                    author: row.get(4)?,
                    plugin_type: row.get(5)?,
                    entry_point: row.get(6)?,
                    is_enabled: row.get(7)?,
                    config_schema: row.get(8)?,
                    default_config: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            },
        )
    }) {
        Ok(plugin) => ApiResponse::success(plugin),
        Err(e) => {
            log::error!("Failed to disable plugin: {}", e);
            ApiResponse::error(format!("Failed to disable plugin: {}", e))
        }
    }
}
