use crate::db;
use crate::models::{
    group::{
        AddFileToGroupRequest, CreateGroupRequest, Group, GroupWithFiles, UpdateGroupRequest,
    },
    file::FileItem,
    ApiResponse,
};
use rusqlite::params;
use uuid::Uuid;

#[tauri::command]
pub fn get_groups(with_files: Option<bool>) -> ApiResponse<Vec<GroupWithFiles>> {
    let include_files = with_files.unwrap_or(false);

    match db::with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, name, description, icon, color, sort_order, is_default,
             created_at, updated_at
             FROM groups ORDER BY sort_order, created_at"
        )?;

        let groups = stmt.query_map([], |row| {
            let group_id: String = row.get(0)?;

            Ok(Group {
                id: group_id,
                name: row.get(1)?,
                description: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                sort_order: row.get(5)?,
                is_default: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        let mut result: Vec<GroupWithFiles> = Vec::new();
        for group in groups {
            let group = group?;

            let files = if include_files {
                let mut stmt = conn.prepare(
                    "SELECT f.id, f.file_name, f.file_path, f.file_type, f.file_size,
                     f.description, f.tags, f.is_collected, f.is_classified,
                     f.classification, f.custom_attributes, f.created_at, f.updated_at
                     FROM files f
                     JOIN file_groups fg ON f.id = fg.file_id
                     WHERE fg.group_id = ?1
                     ORDER BY f.created_at DESC"
                )?;

                let files: Result<Vec<FileItem>, rusqlite::Error> = stmt.query_map(params![&group.id], |row| {
                    Ok(FileItem {
                        id: row.get(0)?,
                        file_name: row.get(1)?,
                        file_path: row.get(2)?,
                        file_type: row.get(3)?,
                        file_size: row.get(4)?,
                        description: row.get(5)?,
                        tags: row.get(6)?,
                        is_collected: row.get(7)?,
                        is_classified: row.get(8)?,
                        classification: row.get(9)?,
                        custom_attributes: row.get(10)?,
                        created_at: row.get(11)?,
                        updated_at: row.get(12)?,
                    })
                })?.collect();
                files?
            } else {
                Vec::new()
            };

            let file_count: i32 = conn.query_row(
                "SELECT COUNT(*) FROM file_groups WHERE group_id = ?1",
                params![&group.id],
                |row| row.get(0),
            )?;

            result.push(GroupWithFiles {
                group,
                files,
                file_count,
            });
        }

        Ok(result)
    }) {
        Ok(groups) => ApiResponse::success(groups),
        Err(e) => {
            log::error!("Failed to get groups: {}", e);
            ApiResponse::error(format!("Failed to get groups: {}", e))
        }
    }
}

#[tauri::command]
pub fn create_group(req: CreateGroupRequest) -> ApiResponse<Group> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    match db::with_connection(|conn| {
        // Get max sort_order
        let max_order: i32 = conn.query_row(
            "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM groups",
            [],
            |row| row.get(0),
        )?;

        conn.execute(
            "INSERT INTO groups (id, name, description, icon, color, sort_order, is_default, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![&id, &req.name, &req.description, &req.icon, &req.color, max_order, false, &now, &now],
        )?;

        Ok(Group {
            id,
            name: req.name,
            description: req.description,
            icon: req.icon,
            color: req.color,
            sort_order: max_order,
            is_default: false,
            created_at: now.clone(),
            updated_at: now,
        })
    }) {
        Ok(group) => ApiResponse::success(group),
        Err(e) => {
            log::error!("Failed to create group: {}", e);
            ApiResponse::error(format!("Failed to create group: {}", e))
        }
    }
}

#[tauri::command]
pub fn update_group(req: UpdateGroupRequest) -> ApiResponse<Group> {
    let now = chrono::Utc::now().to_rfc3339();

    match db::with_connection(|conn| {
        if let Some(name) = &req.name {
            conn.execute(
                "UPDATE groups SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![name, &now, &req.id],
            )?;
        }
        if let Some(desc) = &req.description {
            conn.execute(
                "UPDATE groups SET description = ?1, updated_at = ?2 WHERE id = ?3",
                params![desc, &now, &req.id],
            )?;
        }
        if let Some(icon) = &req.icon {
            conn.execute(
                "UPDATE groups SET icon = ?1, updated_at = ?2 WHERE id = ?3",
                params![icon, &now, &req.id],
            )?;
        }
        if let Some(color) = &req.color {
            conn.execute(
                "UPDATE groups SET color = ?1, updated_at = ?2 WHERE id = ?3",
                params![color, &now, &req.id],
            )?;
        }
        if let Some(order) = req.sort_order {
            conn.execute(
                "UPDATE groups SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
                params![order, &now, &req.id],
            )?;
        }

        conn.query_row(
            "SELECT id, name, description, icon, color, sort_order, is_default, created_at, updated_at
             FROM groups WHERE id = ?1",
            params![&req.id],
            |row| {
                Ok(Group {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    icon: row.get(3)?,
                    color: row.get(4)?,
                    sort_order: row.get(5)?,
                    is_default: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
    }) {
        Ok(group) => ApiResponse::success(group),
        Err(e) => {
            log::error!("Failed to update group: {}", e);
            ApiResponse::error(format!("Failed to update group: {}", e))
        }
    }
}

#[tauri::command]
pub fn delete_group(id: String) -> ApiResponse<bool> {
    match db::with_connection(|conn| {
        // Check if it's a default group
        let is_default: bool = conn.query_row(
            "SELECT is_default FROM groups WHERE id = ?1",
            params![&id],
            |row| row.get(0),
        )?;

        if is_default {
            return Err(rusqlite::Error::ExecuteReturnedResults);
        }

        // Move files to default group
        conn.execute(
            "INSERT OR IGNORE INTO file_groups (file_id, group_id)
             SELECT file_id, 'default' FROM file_groups WHERE group_id = ?1",
            params![&id],
        )?;

        let rows = conn.execute(
            "DELETE FROM groups WHERE id = ?1",
            params![id],
        )?;
        Ok(rows > 0)
    }) {
        Ok(deleted) => ApiResponse::success(deleted),
        Err(e) => {
            log::error!("Failed to delete group: {}", e);
            ApiResponse::error(format!("Failed to delete group: {}", e))
        }
    }
}

#[tauri::command]
pub fn add_file_to_group(req: AddFileToGroupRequest) -> ApiResponse<bool> {
    match db::with_connection(|conn| {
        conn.execute(
            "INSERT OR IGNORE INTO file_groups (file_id, group_id) VALUES (?1, ?2)",
            params![&req.file_id, &req.group_id],
        )?;
        Ok(true)
    }) {
        Ok(result) => ApiResponse::success(result),
        Err(e) => {
            log::error!("Failed to add file to group: {}", e);
            ApiResponse::error(format!("Failed to add file to group: {}", e))
        }
    }
}

#[tauri::command]
pub fn remove_file_from_group(file_id: String, group_id: String) -> ApiResponse<bool> {
    match db::with_connection(|conn| {
        let rows = conn.execute(
            "DELETE FROM file_groups WHERE file_id = ?1 AND group_id = ?2",
            params![&file_id, &group_id],
        )?;
        Ok(rows > 0)
    }) {
        Ok(result) => ApiResponse::success(result),
        Err(e) => {
            log::error!("Failed to remove file from group: {}", e);
            ApiResponse::error(format!("Failed to remove file from group: {}", e))
        }
    }
}
