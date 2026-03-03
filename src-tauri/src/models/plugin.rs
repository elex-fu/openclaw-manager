// 允许未使用的代码，这些类型用于将来的插件系统
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub plugin_type: String,
    pub entry_point: String,
    pub is_enabled: bool,
    pub config_schema: Option<String>,
    pub default_config: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMarketItem {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub downloads: i32,
    pub rating: f32,
    pub icon_url: Option<String>,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallPluginRequest {
    pub market_item_id: String,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugin_id: String,
    pub config: serde_json::Value,
}

/// 市场插件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub author_avatar: Option<String>,
    pub downloads: i32,
    pub rating: f32,
    pub rating_count: i32,
    pub icon_url: Option<String>,
    pub download_url: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub size_bytes: i64,
    pub min_app_version: Option<String>,
    pub changelog: Option<String>,
}

/// 插件分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCategory {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub plugin_count: i32,
}

/// 插件搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPluginsResult {
    pub plugins: Vec<MarketPlugin>,
    pub total: i32,
    pub page: i32,
    pub per_page: i32,
    pub has_more: bool,
}
