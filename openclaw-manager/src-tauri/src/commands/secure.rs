//! 安全存储相关命令

use crate::errors::ApiResponse;
use crate::services::secure_storage::SecureStorage;
use serde::{Deserialize, Serialize};

/// 保存 API Key 请求
#[derive(Debug, Deserialize)]
pub struct SaveApiKeyRequest {
    pub provider: String,
    pub api_key: String,
}

/// API Key 响应
#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub has_key: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// 保存 API Key
///
/// # Arguments
/// * `request` - 包含 provider 和 api_key 的请求
///
/// # Example
/// ```javascript
/// await invoke('save_api_key', { 
///   request: { provider: 'openai', api_key: 'sk-...' } 
/// });
/// ```
#[tauri::command]
pub async fn save_api_key(request: SaveApiKeyRequest) -> Result<ApiResponse<()>, ()> {
    let result = SecureStorage::save_api_key(&request.provider, &request.api_key);
    Ok(ApiResponse::from_result(result))
}

/// 获取 API Key
///
/// # Arguments
/// * `provider` - 提供商名称
///
/// # Returns
/// API Key（如果存在）
#[tauri::command]
pub async fn get_api_key(provider: String) -> Result<ApiResponse<Option<String>>, ()> {
    let result = SecureStorage::get_api_key(&provider);
    Ok(ApiResponse::from_result(result))
}

/// 删除 API Key
///
/// # Arguments
/// * `provider` - 提供商名称
#[tauri::command]
pub async fn delete_api_key(provider: String) -> Result<ApiResponse<()>, ()> {
    let result = SecureStorage::delete_api_key(&provider);
    Ok(ApiResponse::from_result(result))
}

/// 检查是否存在 API Key
///
/// # Arguments
/// * `provider` - 提供商名称
#[tauri::command]
pub async fn has_api_key(provider: String) -> Result<ApiResponse<bool>, ()> {
    let result = SecureStorage::has_api_key(&provider);
    Ok(ApiResponse::from_result(result))
}

/// 获取所有已配置 API Key 的提供商列表
#[tauri::command]
pub async fn get_configured_providers() -> Result<ApiResponse<Vec<String>>, ()> {
    let known = SecureStorage::get_known_providers();
    
    // 检查每个提供商是否有配置
    let mut configured = Vec::new();
    for provider in known {
        match SecureStorage::has_api_key(provider) {
            Ok(true) => configured.push(provider.to_string()),
            _ => {}
        }
    }
    
    Ok(ApiResponse::success(configured))
}

/// 获取支持的提供商列表
#[tauri::command]
pub async fn get_supported_providers() -> Result<ApiResponse<Vec<ProviderInfo>>, ()> {
    let providers = vec![
        ProviderInfo {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            description: "GPT-4, GPT-3.5".to_string(),
            website: "https://platform.openai.com".to_string(),
        },
        ProviderInfo {
            id: "anthropic".to_string(),
            name: "Anthropic".to_string(),
            description: "Claude 3".to_string(),
            website: "https://console.anthropic.com".to_string(),
        },
        ProviderInfo {
            id: "google".to_string(),
            name: "Google".to_string(),
            description: "Gemini".to_string(),
            website: "https://ai.google.dev".to_string(),
        },
        ProviderInfo {
            id: "azure".to_string(),
            name: "Azure OpenAI".to_string(),
            description: "Azure OpenAI Service".to_string(),
            website: "https://azure.microsoft.com".to_string(),
        },
        ProviderInfo {
            id: "cohere".to_string(),
            name: "Cohere".to_string(),
            description: "Command, Embed".to_string(),
            website: "https://cohere.com".to_string(),
        },
        ProviderInfo {
            id: "mistral".to_string(),
            name: "Mistral".to_string(),
            description: "Mistral Large".to_string(),
            website: "https://mistral.ai".to_string(),
        },
        ProviderInfo {
            id: "deepseek".to_string(),
            name: "DeepSeek".to_string(),
            description: "DeepSeek Chat".to_string(),
            website: "https://platform.deepseek.com".to_string(),
        },
    ];
    
    Ok(ApiResponse::success(providers))
}

/// 测试安全存储
#[tauri::command]
pub async fn test_secure_storage() -> Result<ApiResponse<()>, ()> {
    let result = SecureStorage::test_storage();
    Ok(ApiResponse::from_result(result))
}

/// 提供商信息
#[derive(Debug, Serialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub website: String,
}
