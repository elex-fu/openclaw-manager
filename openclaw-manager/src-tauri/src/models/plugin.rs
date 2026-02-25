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
