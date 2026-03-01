use crate::models::{
    plugin::{InstallPluginRequest, Plugin},
    ApiResponse,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// 内存存储插件（临时方案）
lazy_static::lazy_static! {
    static ref PLUGINS: Arc<RwLock<HashMap<String, Plugin>>> = Arc::new(RwLock::new(HashMap::new()));
}

#[tauri::command]
pub async fn get_plugins() -> ApiResponse<Vec<Plugin>> {
    let plugins: tokio::sync::RwLockReadGuard<'_, HashMap<String, Plugin>> = PLUGINS.read().await;
    let list: Vec<Plugin> = plugins.values().cloned().collect();
    ApiResponse::success(list)
}

#[tauri::command]
pub async fn install_plugin(req: InstallPluginRequest) -> ApiResponse<Plugin> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let plugin = Plugin {
        id: id.clone(),
        name: req.market_item_id.clone(),
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

    let mut plugins: tokio::sync::RwLockWriteGuard<'_, HashMap<String, Plugin>> = PLUGINS.write().await;
    plugins.insert(id, plugin.clone());

    ApiResponse::success(plugin)
}

#[tauri::command]
pub async fn uninstall_plugin(id: String) -> ApiResponse<bool> {
    let mut plugins: tokio::sync::RwLockWriteGuard<'_, HashMap<String, Plugin>> = PLUGINS.write().await;
    let removed = plugins.remove(&id).is_some();
    ApiResponse::success(removed)
}

#[tauri::command]
pub async fn enable_plugin(id: String) -> ApiResponse<Plugin> {
    let mut plugins: tokio::sync::RwLockWriteGuard<'_, HashMap<String, Plugin>> = PLUGINS.write().await;

    if let Some(plugin) = plugins.get_mut(&id) {
        plugin.is_enabled = true;
        plugin.updated_at = chrono::Utc::now().to_rfc3339();
        ApiResponse::success(plugin.clone())
    } else {
        ApiResponse::error(format!("Plugin not found: {}", id))
    }
}

#[tauri::command]
pub async fn disable_plugin(id: String) -> ApiResponse<Plugin> {
    let mut plugins: tokio::sync::RwLockWriteGuard<'_, HashMap<String, Plugin>> = PLUGINS.write().await;

    if let Some(plugin) = plugins.get_mut(&id) {
        plugin.is_enabled = false;
        plugin.updated_at = chrono::Utc::now().to_rfc3339();
        ApiResponse::success(plugin.clone())
    } else {
        ApiResponse::error(format!("Plugin not found: {}", id))
    }
}
