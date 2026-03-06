//! 安全存储相关命令

use crate::errors::ApiResponse;
use crate::services::secure_storage::SecureStorage;
use serde::Deserialize;

/// 保存 API Key 请求
#[derive(Debug, Deserialize)]
pub struct SaveApiKeyRequest {
    pub provider: String,
    pub api_key: String,
}

/// 保存 API Key
#[tauri::command]
pub async fn save_api_key(request: SaveApiKeyRequest) -> Result<ApiResponse<()>, ()> {
    let storage = SecureStorage::global().map_err(|_| ())?;
    let result = storage.save_api_key(&request.provider, &request.api_key);
    Ok(ApiResponse::from_result(result))
}

/// 获取 API Key
#[tauri::command]
pub async fn get_api_key(provider: String) -> Result<ApiResponse<Option<String>>, ()> {
    let storage = SecureStorage::global().map_err(|_| ())?;
    let result = storage.get_api_key(&provider);
    Ok(ApiResponse::from_result(result))
}

/// 删除 API Key
#[tauri::command]
pub async fn delete_api_key(provider: String) -> Result<ApiResponse<()>, ()> {
    let storage = SecureStorage::global().map_err(|_| ())?;
    let result = storage.delete_api_key(&provider);
    Ok(ApiResponse::from_result(result))
}

/// 检查是否存在 API Key
#[tauri::command]
pub async fn has_api_key(provider: String) -> Result<ApiResponse<bool>, ()> {
    let storage = SecureStorage::global().map_err(|_| ())?;
    let result = storage.has_api_key(&provider);
    Ok(ApiResponse::from_result(result))
}
