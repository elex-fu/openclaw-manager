//! 配置管理相关命令

use crate::errors::ApiResponse;
use crate::services::config_manager::{ConfigManager, ConfigState, ModelConfig, AppConfig};
use serde::Serialize;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// 应用状态
pub struct AppState {
    pub config_manager: Arc<Mutex<ConfigManager>>,
}

/// 获取完整配置状态
#[tauri::command]
pub async fn get_full_config(
    state: State<'_, AppState>,
) -> Result<ApiResponse<ConfigState>, ()> {
    let manager = state.config_manager.lock().await;
    let config = manager.get_state().await;
    Ok(ApiResponse::success(config))
}

/// 获取应用配置
#[tauri::command]
pub async fn get_app_config(
    state: State<'_, AppState>,
) -> Result<ApiResponse<AppConfig>, ()> {
    let manager = state.config_manager.lock().await;
    let config = manager.get_state().await;
    Ok(ApiResponse::success(config.app))
}

/// 更新应用配置
#[tauri::command]
pub async fn update_app_config(
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<ApiResponse<()>, ()> {
    let manager = state.config_manager.lock().await;
    let result = manager.update_partial(|state| {
        state.app = config;
    }).await;
    Ok(ApiResponse::from_result(result))
}

/// 获取模型配置列表
#[tauri::command]
pub async fn get_model_configs(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<ModelConfig>>, ()> {
    let manager = state.config_manager.lock().await;
    let config = manager.get_state().await;
    Ok(ApiResponse::success(config.models))
}

/// 获取单个模型配置
#[tauri::command]
pub async fn get_model_config(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<ApiResponse<Option<ModelConfig>>, ()> {
    let manager = state.config_manager.lock().await;
    let config = manager.get_state().await;
    let model = config.models.into_iter().find(|m| m.id == model_id);
    Ok(ApiResponse::success(model))
}

/// 添加模型配置
#[tauri::command]
pub async fn add_model_config(
    state: State<'_, AppState>,
    model: ModelConfig,
) -> Result<ApiResponse<()>, ()> {
    let manager = state.config_manager.lock().await;
    let result = manager.add_model(model).await;
    Ok(ApiResponse::from_result(result))
}

/// 更新模型配置
#[tauri::command]
pub async fn update_model_config(
    state: State<'_, AppState>,
    model: ModelConfig,
) -> Result<ApiResponse<()>, ()> {
    let manager = state.config_manager.lock().await;
    let result = manager.update_model(&model.id, |m| {
        m.name = model.name;
        m.provider = model.provider;
        m.api_base = model.api_base;
        m.model = model.model;
        m.temperature = model.temperature;
        m.max_tokens = model.max_tokens;
        m.enabled = model.enabled;
        m.default = model.default;
    }).await;
    Ok(ApiResponse::from_result(result))
}

/// 删除模型配置
#[tauri::command]
pub async fn delete_model_config(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<ApiResponse<()>, ()> {
    let manager = state.config_manager.lock().await;
    let result = manager.remove_model(&model_id).await;
    Ok(ApiResponse::from_result(result))
}

/// 设置默认模型
#[tauri::command]
pub async fn set_default_model(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<ApiResponse<()>, ()> {
    let manager = state.config_manager.lock().await;
    
    let result = manager.update_partial(|state| {
        for model in &mut state.models {
            model.default = model.id == model_id;
        }
    }).await;
    
    Ok(ApiResponse::from_result(result))
}

/// 更新配置（带乐观锁）
#[tauri::command]
pub async fn update_config_with_version(
    state: State<'_, AppState>,
    config: ConfigState,
    expected_version: u32,
) -> Result<ApiResponse<()>, ()> {
    let manager = state.config_manager.lock().await;
    let result = manager.update_state(config, expected_version).await;
    Ok(ApiResponse::from_result(result))
}

/// 导出配置
#[tauri::command]
pub async fn export_config(
    state: State<'_, AppState>,
    path: String,
) -> Result<ApiResponse<()>, ()> {
    let manager = state.config_manager.lock().await;
    let result = manager.export_to(path).await;
    Ok(ApiResponse::from_result(result))
}

/// 导入配置
#[tauri::command]
pub async fn import_config(
    state: State<'_, AppState>,
    path: String,
) -> Result<ApiResponse<()>, ()> {
    let manager = state.config_manager.lock().await;
    let result = manager.import_from(path).await;
    Ok(ApiResponse::from_result(result))
}

/// 重置配置
#[tauri::command]
pub async fn reset_config(
    state: State<'_, AppState>,
) -> Result<ApiResponse<()>, ()> {
    let manager = state.config_manager.lock().await;
    let result = manager.reset_to_default().await;
    Ok(ApiResponse::from_result(result))
}

/// 验证配置
#[tauri::command]
pub async fn validate_config(
    state: State<'_, AppState>,
) -> Result<ApiResponse<ValidationResponse>, ()> {
    let manager = state.config_manager.lock().await;
    let config = manager.get_state().await;
    let validation = ConfigManager::validate(&config);
    
    Ok(ApiResponse::success(ValidationResponse {
        valid: validation.valid,
        errors: validation.errors,
    }))
}

/// 获取配置版本
#[tauri::command]
pub async fn get_config_version(
    state: State<'_, AppState>,
) -> Result<ApiResponse<u32>, ()> {
    let manager = state.config_manager.lock().await;
    let version = manager.get_version().await;
    Ok(ApiResponse::success(version))
}

/// 验证响应
#[derive(Debug, Serialize)]
pub struct ValidationResponse {
    pub valid: bool,
    pub errors: Vec<String>,
}

// 保持向后兼容的原有命令
use crate::models::config::{Config, CreateConfigRequest};

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
