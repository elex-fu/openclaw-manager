use crate::db;
use crate::models::{
    file::{FileItem, FileScanRequest, FileScanResult, ParseFileInfo, UpdateFileRequest},
    ApiResponse,
};
use crate::utils::file_parser;
use rusqlite::params;
use std::collections::HashSet;
use uuid::Uuid;
use walkdir::WalkDir;

#[tauri::command]
pub fn scan_files(req: FileScanRequest) -> ApiResponse<FileScanResult> {
    let mut files = Vec::new();
    let mut total_size: i64 = 0;

    let file_types: HashSet<String> = req
        .file_types
        .map(|types| types.into_iter().collect())
        .unwrap_or_default();

    let walker = if req.recursive {
        WalkDir::new(&req.path)
    } else {
        WalkDir::new(&req.path).max_depth(1)
    };

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() {
                let file_path = entry.path().to_string_lossy().to_string();
                let file_name = entry
                    .file_name()
                    .to_string_lossy()
                    .to_string();

                let ext = entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase())
                    .unwrap_or_default();

                // Filter by file types if specified
                if !file_types.is_empty() && !file_types.contains(&ext) {
                    continue;
                }

                let size = metadata.len() as i64;
                total_size += size;

                let file_item = FileItem {
                    id: Uuid::new_v4().to_string(),
                    file_name: file_name.clone(),
                    file_path: file_path.clone(),
                    file_type: ext,
                    file_size: size,
                    description: None,
                    tags: None,
                    is_collected: false,
                    is_classified: false,
                    classification: None,
                    custom_attributes: None,
                    created_at: chrono::Utc::now().to_rfc3339(),
                    updated_at: chrono::Utc::now().to_rfc3339(),
                };

                // Insert into database
                if let Err(e) = db::with_connection(|conn| {
                    conn.execute(
                        "INSERT OR IGNORE INTO files
                         (id, file_name, file_path, file_type, file_size,
                          description, tags, is_collected, is_classified,
                          classification, custom_attributes, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                        params![
                            &file_item.id,
                            &file_item.file_name,
                            &file_item.file_path,
                            &file_item.file_type,
                            file_item.file_size,
                            &file_item.description,
                            &file_item.tags,
                            file_item.is_collected,
                            file_item.is_classified,
                            &file_item.classification,
                            &file_item.custom_attributes,
                            &file_item.created_at,
                            &file_item.updated_at,
                        ],
                    )?;
                    Ok(())
                }) {
                    log::warn!("Failed to insert file {}: {}", file_path, e);
                }

                files.push(file_item);
            }
        }
    }

    ApiResponse::success(FileScanResult {
        total_count: files.len(),
        total_size,
        files,
    })
}

#[tauri::command]
pub fn get_files(
    file_type: Option<String>,
    is_collected: Option<bool>,
    is_classified: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> ApiResponse<Vec<FileItem>> {
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);

    let mut query = String::from(
        "SELECT id, file_name, file_path, file_type, file_size,
         description, tags, is_collected, is_classified,
         classification, custom_attributes, created_at, updated_at
         FROM files WHERE 1=1"
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(t) = file_type {
        query.push_str(" AND file_type = ?");
        params.push(Box::new(t));
    }
    if let Some(c) = is_collected {
        query.push_str(" AND is_collected = ?");
        params.push(Box::new(c));
    }
    if let Some(c) = is_classified {
        query.push_str(" AND is_classified = ?");
        params.push(Box::new(c));
    }

    query.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
    params.push(Box::new(limit));
    params.push(Box::new(offset));

    match db::with_connection(|conn| {
        let param_refs: Vec<&dyn rusqlite::ToSql> = params
            .iter()
            .map(|p| p.as_ref())
            .collect();

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map(
            param_refs.as_slice(),
            |row| {
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
            },
        )?;

        rows.collect::<Result<Vec<_>, _>>()
    }) {
        Ok(files) => ApiResponse::success(files),
        Err(e) => {
            log::error!("Failed to get files: {}", e);
            ApiResponse::error(format!("Failed to get files: {}", e))
        }
    }
}

#[tauri::command]
pub fn get_file_by_id(id: String) -> ApiResponse<Option<FileItem>> {
    match db::with_connection(|conn| {
        conn.query_row(
            "SELECT id, file_name, file_path, file_type, file_size,
             description, tags, is_collected, is_classified,
             classification, custom_attributes, created_at, updated_at
             FROM files WHERE id = ?1",
            params![id],
            |row| {
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
            },
        )
    }) {
        Ok(file) => ApiResponse::success(Some(file)),
        Err(e) if e.to_string().contains("QueryReturnedNoRows") => ApiResponse::success(None),
        Err(e) => {
            log::error!("Failed to get file: {}", e);
            ApiResponse::error(format!("Failed to get file: {}", e))
        }
    }
}

#[tauri::command]
pub fn update_file(req: UpdateFileRequest) -> ApiResponse<FileItem> {
    let now = chrono::Utc::now().to_rfc3339();

    match db::with_connection(|conn| {
        if let Some(desc) = &req.description {
            conn.execute(
                "UPDATE files SET description = ?1, updated_at = ?2 WHERE id = ?3",
                params![desc, &now, &req.id],
            )?;
        }

        if let Some(tags) = &req.tags {
            let tags_json = serde_json::to_string(tags).map_err(|e| {
                rusqlite::Error::InvalidParameterName(format!("JSON error: {}", e))
            })?;
            conn.execute(
                "UPDATE files SET tags = ?1, updated_at = ?2 WHERE id = ?3",
                params![tags_json, &now, &req.id],
            )?;
        }

        if let Some(collected) = req.is_collected {
            conn.execute(
                "UPDATE files SET is_collected = ?1, updated_at = ?2 WHERE id = ?3",
                params![collected, &now, &req.id],
            )?;
        }

        if let Some(attrs) = &req.custom_attributes {
            let attrs_json = attrs.to_string();
            conn.execute(
                "UPDATE files SET custom_attributes = ?1, updated_at = ?2 WHERE id = ?3",
                params![attrs_json, &now, &req.id],
            )?;
        }

        conn.query_row(
            "SELECT id, file_name, file_path, file_type, file_size,
             description, tags, is_collected, is_classified,
             classification, custom_attributes, created_at, updated_at
             FROM files WHERE id = ?1",
            params![&req.id],
            |row| {
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
            },
        )
    }) {
        Ok(file) => ApiResponse::success(file),
        Err(e) => {
            log::error!("Failed to update file: {}", e);
            ApiResponse::error(format!("Failed to update file: {}", e))
        }
    }
}

#[tauri::command]
pub fn delete_file(id: String) -> ApiResponse<bool> {
    match db::with_connection(|conn| {
        let rows = conn.execute(
            "DELETE FROM files WHERE id = ?1",
            params![id],
        )?;
        Ok(rows > 0)
    }) {
        Ok(deleted) => ApiResponse::success(deleted),
        Err(e) => {
            log::error!("Failed to delete file: {}", e);
            ApiResponse::error(format!("Failed to delete file: {}", e))
        }
    }
}

#[tauri::command]
pub fn parse_file_info(file_name: String) -> ApiResponse<ParseFileInfo> {
    match file_parser::parse_file_name(&file_name) {
        Ok(parsed) => ApiResponse::success(ParseFileInfo {
            file_name: file_name.clone(),
            parsed_data: parsed,
        }),
        Err(e) => {
            log::error!("Failed to parse file info: {}", e);
            ApiResponse::error(format!("Failed to parse file info: {}", e))
        }
    }
}
