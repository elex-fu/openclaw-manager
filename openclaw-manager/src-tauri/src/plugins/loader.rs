use crate::models::plugin::Plugin;
use std::path::Path;

pub struct PluginLoader;

impl PluginLoader {
    /// Load a plugin from a directory
    pub fn load_from_dir<P: AsRef<Path>>(path: P) -> anyhow::Result<Plugin> {
        let manifest_path = path.as_ref().join("manifest.json");
        let manifest_content = std::fs::read_to_string(&manifest_path)?;
        let manifest: PluginManifest = serde_json::from_str(&manifest_content)?;

        Ok(Plugin {
            id: manifest.id,
            name: manifest.name,
            version: manifest.version,
            description: manifest.description,
            author: manifest.author,
            plugin_type: manifest.plugin_type,
            entry_point: manifest.entry_point,
            is_enabled: false,
            config_schema: manifest.config_schema.map(|s| s.to_string()),
            default_config: manifest.default_config.map(|c| c.to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Install a plugin from a URL
    pub async fn install_from_url(url: &str, target_dir: &Path) -> anyhow::Result<Plugin> {
        // TODO: Download and extract plugin
        // For now, this is a placeholder
        anyhow::bail!("Not implemented yet")
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    #[serde(rename = "type")]
    pub plugin_type: String,
    pub entry_point: String,
    pub config_schema: Option<serde_json::Value>,
    pub default_config: Option<serde_json::Value>,
}
