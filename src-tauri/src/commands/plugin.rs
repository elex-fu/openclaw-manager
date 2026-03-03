use crate::errors::app_error::AppError;
use crate::models::plugin::{InstallPluginRequest, Plugin, PluginConfig};
use crate::models::ApiResponse;
use crate::services::plugin_manager::{PluginManager, PluginManifest};
use crate::services::plugin_market::{
    MarketClient, MarketPlugin, PluginCategory, SearchPluginsRequest, SearchPluginsResult, SortBy,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

/// 插件管理器状态
pub struct PluginManagerState {
    pub manager: PluginManager,
    pub market_client: MarketClient,
}

impl PluginManagerState {
    pub async fn new() -> Result<Self, AppError> {
        Ok(Self {
            manager: PluginManager::new().await?,
            market_client: MarketClient::new(),
        })
    }
}

/// 市场插件安装请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallFromMarketRequest {
    pub plugin_id: String,
}

/// 更新插件配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePluginConfigRequest {
    pub plugin_id: String,
    pub config: serde_json::Value,
}

/// 搜索插件请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMarketPluginsRequest {
    pub query: Option<String>,
    pub category: Option<String>,
    pub sort_by: Option<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

/// 获取所有已安装插件
#[tauri::command]
pub async fn get_plugins(
    state: State<'_, PluginManagerState>,
) -> Result<ApiResponse<Vec<Plugin>>, String> {
    let plugins = state.manager.get_all_plugins().await;
    Ok(ApiResponse::success(plugins))
}

/// 从市场安装插件
#[tauri::command]
pub async fn install_plugin(
    state: State<'_, PluginManagerState>,
    req: InstallPluginRequest,
) -> Result<ApiResponse<Plugin>, String> {
    // 获取插件详情
    let market_plugin = match state.market_client.get_plugin_details(&req.market_item_id).await {
        Ok(p) => p,
        Err(e) => return Ok(ApiResponse::error(e.to_string())),
    };

    // 下载插件
    let plugin_data = match state.market_client.download_plugin(&req.market_item_id).await {
        Ok(data) => data,
        Err(e) => return Ok(ApiResponse::error(e.to_string())),
    };

    // 创建插件清单（从市场信息转换）
    let manifest = PluginManifest {
        id: market_plugin.id.clone(),
        name: market_plugin.name.clone(),
        version: market_plugin.version.clone(),
        description: market_plugin.description.clone(),
        author: market_plugin.author.clone(),
        plugin_type: "lua".to_string(), // 默认类型，实际应从市场获取
        entry_point: "main.lua".to_string(), // 默认入口
        config_schema: None,
        default_config: None,
        dependencies: vec![],
        min_app_version: market_plugin.min_app_version.clone(),
    };

    // 创建插件文件映射（模拟，实际应解压下载的包）
    let mut files = HashMap::new();
    files.insert("main.lua".to_string(), plugin_data);

    // 安装插件
    match state.manager.install_plugin(manifest, files).await {
        Ok(plugin) => Ok(ApiResponse::success(plugin)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

/// 卸载插件
#[tauri::command]
pub async fn uninstall_plugin(
    state: State<'_, PluginManagerState>,
    id: String,
) -> Result<ApiResponse<bool>, String> {
    match state.manager.uninstall_plugin(&id).await {
        Ok(result) => Ok(ApiResponse::success(result)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

/// 启用插件
#[tauri::command]
pub async fn enable_plugin(
    state: State<'_, PluginManagerState>,
    id: String,
) -> Result<ApiResponse<Plugin>, String> {
    match state.manager.enable_plugin(&id).await {
        Ok(plugin) => Ok(ApiResponse::success(plugin)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

/// 禁用插件
#[tauri::command]
pub async fn disable_plugin(
    state: State<'_, PluginManagerState>,
    id: String,
) -> Result<ApiResponse<Plugin>, String> {
    match state.manager.disable_plugin(&id).await {
        Ok(plugin) => Ok(ApiResponse::success(plugin)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

/// 获取插件配置
#[tauri::command]
pub async fn get_plugin_config(
    state: State<'_, PluginManagerState>,
    plugin_id: String,
) -> Result<ApiResponse<Option<PluginConfig>>, String> {
    match state.manager.get_plugin_config(&plugin_id).await {
        Ok(config) => Ok(ApiResponse::success(config)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

/// 更新插件配置
#[tauri::command]
pub async fn update_plugin_config(
    state: State<'_, PluginManagerState>,
    req: UpdatePluginConfigRequest,
) -> Result<ApiResponse<PluginConfig>, String> {
    match state.manager.update_plugin_config(&req.plugin_id, req.config).await {
        Ok(config) => Ok(ApiResponse::success(config)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

/// 搜索市场插件
#[tauri::command]
pub async fn search_market_plugins(
    state: State<'_, PluginManagerState>,
    req: SearchMarketPluginsRequest,
) -> Result<ApiResponse<SearchPluginsResult>, String> {
    let sort_by = req.sort_by.and_then(|s| match s.as_str() {
        "downloads" => Some(SortBy::Downloads),
        "rating" => Some(SortBy::Rating),
        "created_at" => Some(SortBy::CreatedAt),
        "updated_at" => Some(SortBy::UpdatedAt),
        _ => Some(SortBy::Relevance),
    });

    let search_req = SearchPluginsRequest {
        query: req.query,
        category: req.category,
        tags: None,
        sort_by,
        page: req.page,
        per_page: req.per_page,
    };

    match state.market_client.search_plugins(search_req).await {
        Ok(result) => Ok(ApiResponse::success(result)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

/// 获取市场插件详情
#[tauri::command]
pub async fn get_market_plugin_details(
    state: State<'_, PluginManagerState>,
    plugin_id: String,
) -> Result<ApiResponse<MarketPlugin>, String> {
    match state.market_client.get_plugin_details(&plugin_id).await {
        Ok(plugin) => Ok(ApiResponse::success(plugin)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

/// 获取插件分类
#[tauri::command]
pub async fn get_plugin_categories(
    state: State<'_, PluginManagerState>,
) -> Result<ApiResponse<Vec<PluginCategory>>, String> {
    match state.market_client.get_categories().await {
        Ok(categories) => Ok(ApiResponse::success(categories)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

/// 获取热门插件
#[tauri::command]
pub async fn get_popular_plugins(
    state: State<'_, PluginManagerState>,
    limit: Option<i32>,
) -> Result<ApiResponse<Vec<MarketPlugin>>, String> {
    match state.market_client.get_popular_plugins(limit.unwrap_or(10)).await {
        Ok(plugins) => Ok(ApiResponse::success(plugins)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

/// 获取最新插件
#[tauri::command]
pub async fn get_latest_plugins(
    state: State<'_, PluginManagerState>,
    limit: Option<i32>,
) -> Result<ApiResponse<Vec<MarketPlugin>>, String> {
    match state.market_client.get_latest_plugins(limit.unwrap_or(10)).await {
        Ok(plugins) => Ok(ApiResponse::success(plugins)),
        Err(e) => Ok(ApiResponse::error(e.to_string())),
    }
}

/// 检查插件是否已安装
#[tauri::command]
pub async fn check_plugin_installed(
    state: State<'_, PluginManagerState>,
    plugin_id: String,
) -> Result<ApiResponse<bool>, String> {
    Ok(ApiResponse::success(state.manager.is_installed(&plugin_id).await))
}

/// 获取已启用的插件
#[tauri::command]
pub async fn get_enabled_plugins(
    state: State<'_, PluginManagerState>,
) -> Result<ApiResponse<Vec<Plugin>>, String> {
    let plugins = state.manager.get_enabled_plugins().await;
    Ok(ApiResponse::success(plugins))
}
