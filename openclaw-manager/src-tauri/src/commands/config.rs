use crate::db;
use crate::models::{
    config::{Config, CreateConfigRequest, UpdateConfigRequest},
    ApiResponse,
};
use rusqlite::params;
use uuid::Uuid;

#[tauri::command]
pub fn get_config(key: String) -> ApiResponse<Option<Config>> {
    match db::with_connection(|conn| {
        conn.query_row(
            "SELECT id, key, value, description, created_at, updated_at
             FROM configs WHERE key = ?1",
            params![key],
            |row| {
                Ok(Config {
                    id: row.get(0)?,
                    key: row.get(1)?,
                    value: row.get(2)?,
                    description: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
    }) {
        Ok(config) => ApiResponse::success(Some(config)),
        Err(e) if e.to_string().contains("QueryReturnedNoRows") => ApiResponse::success(None),
        Err(e) => {
            log::error!("Failed to get config: {}", e);
            ApiResponse::error(format!("Failed to get config: {}", e))
        }
    }
}

#[tauri::command]
pub fn set_config(req: CreateConfigRequest) -> ApiResponse<Config> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    match db::with_connection(|conn| {
        conn.execute(
            "INSERT INTO configs (id, key, value, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(key) DO UPDATE SET
             value = excluded.value,
             updated_at = excluded.updated_at",
            params![&id, &req.key, &req.value, &req.description, &now, &now],
        )?;

        conn.query_row(
            "SELECT id, key, value, description, created_at, updated_at
             FROM configs WHERE key = ?1",
            params![&req.key],
            |row| {
                Ok(Config {
                    id: row.get(0)?,
                    key: row.get(1)?,
                    value: row.get(2)?,
                    description: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
    }) {
        Ok(config) => ApiResponse::success(config),
        Err(e) => {
            log::error!("Failed to set config: {}", e);
            ApiResponse::error(format!("Failed to set config: {}", e))
        }
    }
}

#[tauri::command]
pub fn delete_config(key: String) -> ApiResponse<bool> {
    match db::with_connection(|conn| {
        let rows = conn.execute(
            "DELETE FROM configs WHERE key = ?1",
            params![key],
        )?;
        Ok(rows > 0)
    }) {
        Ok(deleted) => ApiResponse::success(deleted),
        Err(e) => {
            log::error!("Failed to delete config: {}", e);
            ApiResponse::error(format!("Failed to delete config: {}", e))
        }
    }
}
