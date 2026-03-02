//! 模型管理相关命令

use crate::models::config::{ConnectionTestResult, ModelConfigFull, ModelParameters};
use crate::services::config_manager::ConfigManager;
use crate::services::secure_storage::SecureStorage;
use std::sync::Arc;
use tauri::State;

/// 测试模型连接
#[tauri::command]
pub async fn test_model_connection(
    model_id: String,
    config_manager: State<'_, Arc<ConfigManager>>,
) -> Result<crate::models::openclaw::ApiResponse<ConnectionTestResult>, String> {
    let start_time = std::time::Instant::now();

    // 获取模型配置
    let models = config_manager.get_models_full().await.map_err(|e| e.to_string())?;
    let model = models
        .into_iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| format!("模型 {} 不存在", model_id))?;

    // 获取 API Key
    let api_key = SecureStorage::get_api_key(&model.provider)
        .map_err(|e| format!("无法获取 API Key: {}", e))?
        .ok_or_else(|| "API Key 未设置".to_string())?;

    // 确定 API 基础 URL
    let api_base = model
        .api_base
        .unwrap_or_else(|| get_default_api_base(&model.provider));

    // 执行连接测试
    let result = match test_connection_internal(&api_base, &api_key, &model.model).await {
        Ok(model_info) => {
            let latency = start_time.elapsed().as_millis() as u64;
            ConnectionTestResult {
                success: true,
                latency,
                message: None,
                model_info: Some(model_info),
            }
        }
        Err(e) => {
            let latency = start_time.elapsed().as_millis() as u64;
            ConnectionTestResult {
                success: false,
                latency,
                message: Some(e),
                model_info: None,
            }
        }
    };

    let message = result.message.clone();
    Ok(crate::models::openclaw::ApiResponse {
        success: result.success,
        data: Some(result),
        error: message,
    })
}

/// 获取默认 API 基础 URL
fn get_default_api_base(provider: &str) -> String {
    match provider.to_lowercase().as_str() {
        "openai" => "https://api.openai.com/v1".to_string(),
        "anthropic" => "https://api.anthropic.com/v1".to_string(),
        "google" => "https://generativelanguage.googleapis.com/v1".to_string(),
        "azure" => "https://api.openai.azure.com".to_string(),
        _ => "https://api.openai.com/v1".to_string(),
    }
}

/// 内部连接测试函数
async fn test_connection_internal(
    api_base: &str,
    api_key: &str,
    model: &str,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP 客户端创建失败: {}", e))?;

    // 根据提供商使用不同的测试端点
    if api_base.contains("anthropic") {
        test_anthropic_connection(&client, api_base, api_key).await
    } else if api_base.contains("google") {
        test_google_connection(&client, api_base, api_key, model).await
    } else {
        test_openai_compatible_connection(&client, api_base, api_key).await
    }
}

/// 测试 OpenAI 兼容 API 连接
async fn test_openai_compatible_connection(
    client: &reqwest::Client,
    api_base: &str,
    api_key: &str,
) -> Result<String, String> {
    let url = format!("{}/models", api_base);

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    if response.status().is_success() {
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))?;

        let model_count = json
            .get("data")
            .and_then(|d| d.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        Ok(format!("可用模型数量: {}", model_count))
    } else {
        let status = response.status();
        let text = response
            .text()
            .await
            .unwrap_or_else(|_| "未知错误".to_string());
        Err(format!("API 错误 ({}): {}", status, text))
    }
}

/// 测试 Anthropic API 连接
async fn test_anthropic_connection(
    client: &reqwest::Client,
    api_base: &str,
    api_key: &str,
) -> Result<String, String> {
    let url = format!("{}/models", api_base);

    let response = client
        .get(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    if response.status().is_success() {
        Ok("Anthropic API 连接成功".to_string())
    } else {
        let status = response.status();
        let text = response
            .text()
            .await
            .unwrap_or_else(|_| "未知错误".to_string());
        Err(format!("API 错误 ({}): {}", status, text))
    }
}

/// 测试 Google API 连接
async fn test_google_connection(
    client: &reqwest::Client,
    api_base: &str,
    api_key: &str,
    model: &str,
) -> Result<String, String> {
    let url = format!("{}/models/{}?key={}", api_base, model, api_key);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("连接失败: {}", e))?;

    if response.status().is_success() {
        Ok("Google API 连接成功".to_string())
    } else {
        let status = response.status();
        let text = response
            .text()
            .await
            .unwrap_or_else(|_| "未知错误".to_string());
        Err(format!("API 错误 ({}): {}", status, text))
    }
}

/// 更新模型优先级（批量）
#[tauri::command]
pub async fn update_model_priority(
    model_orders: Vec<(String, i32)>,
    config_manager: State<'_, Arc<ConfigManager>>,
) -> Result<crate::models::openclaw::ApiResponse<bool>, String> {
    config_manager
        .update_model_priorities(model_orders)
        .await
        .map_err(|e| e.to_string())?;

    Ok(crate::models::openclaw::ApiResponse {
        success: true,
        data: Some(true),
        error: None,
    })
}

/// 获取所有模型（完整配置）
#[tauri::command]
pub async fn get_all_models_full(
    config_manager: State<'_, Arc<ConfigManager>>,
) -> Result<crate::models::openclaw::ApiResponse<Vec<ModelConfigFull>>, String> {
    let models = config_manager
        .get_models_full()
        .await
        .map_err(|e| e.to_string())?;

    Ok(crate::models::openclaw::ApiResponse {
        success: true,
        data: Some(models),
        error: None,
    })
}

/// 保存模型配置（完整版）
#[tauri::command]
pub async fn save_model_full(
    model: ModelConfigFull,
    config_manager: State<'_, Arc<ConfigManager>>,
) -> Result<crate::models::openclaw::ApiResponse<ModelConfigFull>, String> {
    // 验证模型参数
    if let Err(e) = validate_model_parameters(&model.parameters) {
        return Ok(crate::models::openclaw::ApiResponse {
            success: false,
            data: None,
            error: Some(e),
        });
    }

    config_manager
        .save_model_full(model.clone())
        .await
        .map_err(|e| e.to_string())?;

    Ok(crate::models::openclaw::ApiResponse {
        success: true,
        data: Some(model),
        error: None,
    })
}

/// 验证模型参数
fn validate_model_parameters(params: &ModelParameters) -> Result<(), String> {
    if !(0.0..=2.0).contains(&params.temperature) {
        return Err("temperature 必须在 0-2 之间".to_string());
    }
    if !(1..=8192).contains(&params.max_tokens) {
        return Err("max_tokens 必须在 1-8192 之间".to_string());
    }
    if !(0.0..=1.0).contains(&params.top_p) {
        return Err("top_p 必须在 0-1 之间".to_string());
    }
    if !(-2.0..=2.0).contains(&params.presence_penalty) {
        return Err("presence_penalty 必须在 -2 到 2 之间".to_string());
    }
    if !(-2.0..=2.0).contains(&params.frequency_penalty) {
        return Err("frequency_penalty 必须在 -2 到 2 之间".to_string());
    }
    Ok(())
}
