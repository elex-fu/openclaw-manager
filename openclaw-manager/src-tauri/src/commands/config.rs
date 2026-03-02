//! 配置管理相关命令

#![allow(dead_code)]

use crate::models::config::{Config, CreateConfigRequest};

// 保持向后兼容的原有命令

/// 获取配置列表（旧版兼容）
#[tauri::command]
pub async fn get_configs() -> Result<Vec<Config>, String> {
    // 返回空列表，用于向后兼容
    Ok(Vec::new())
}

/// 获取单个配置（旧版兼容）
#[tauri::command]
pub async fn get_config(_id: String) -> Result<Option<Config>, String> {
    // 返回 None，用于向后兼容
    Ok(None)
}

/// 创建配置（旧版兼容）
#[tauri::command]
pub async fn set_config(request: CreateConfigRequest) -> Result<Config, String> {
    // 创建空配置返回，用于向后兼容
    let now = chrono::Utc::now().to_rfc3339();
    Ok(Config {
        id: uuid::Uuid::new_v4().to_string(),
        key: request.key,
        value: request.value,
        description: request.description,
        created_at: now.clone(),
        updated_at: now,
    })
}

/// 删除配置（旧版兼容）
#[tauri::command]
pub async fn delete_config(_id: String) -> Result<bool, String> {
    // 模拟删除成功
    Ok(true)
}
