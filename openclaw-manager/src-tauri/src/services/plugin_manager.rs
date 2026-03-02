//! 插件管理服务
//!
//! 提供插件的安装、卸载、启用、禁用等功能
//! 使用文件系统存储插件数据和配置

use crate::errors::app_error::{AppError, ConfigError};
use crate::models::plugin::{Plugin, PluginConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::RwLock;

/// 插件清单文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub plugin_type: String, // lua, js, wasm
    pub entry_point: String,
    pub config_schema: Option<serde_json::Value>,
    pub default_config: Option<serde_json::Value>,
    pub dependencies: Vec<String>,
    pub min_app_version: Option<String>,
}

impl From<PluginManifest> for Plugin {
    fn from(manifest: PluginManifest) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Plugin {
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
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

/// 已安装插件列表
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstalledPlugins {
    pub plugins: HashMap<String, Plugin>,
    pub version: u32,
}

/// 插件管理器
pub struct PluginManager {
    plugins_dir: PathBuf,
    installed_file: PathBuf,
    cache: RwLock<InstalledPlugins>,
}

impl PluginManager {
    const CURRENT_VERSION: u32 = 1;
    const INSTALLED_FILE: &'static str = "installed.json";

    /// 创建新的插件管理器实例
    pub async fn new() -> Result<Self, AppError> {
        let plugins_dir = Self::get_plugins_dir()?;
        let installed_file = plugins_dir.join(Self::INSTALLED_FILE);

        // 确保目录存在
        fs::create_dir_all(&plugins_dir)
            .await
            .map_err(|e| AppError::Io(e))?;

        let mut manager = Self {
            plugins_dir,
            installed_file,
            cache: RwLock::new(InstalledPlugins::default()),
        };

        // 加载已安装插件
        manager.load_installed().await?;

        Ok(manager)
    }

    /// 获取插件目录路径
    fn get_plugins_dir() -> Result<PathBuf, AppError> {
        let home = dirs::home_dir()
            .ok_or_else(|| ConfigError::FileNotFound("无法获取主目录".to_string()))?;
        Ok(home.join(".openclaw").join("plugins"))
    }

    /// 获取插件目录（公开方法）
    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }

    /// 从指定目录创建插件管理器（用于测试）
    #[cfg(test)]
    pub fn from_directory(plugins_dir: PathBuf) -> Self {
        let installed_file = plugins_dir.join(Self::INSTALLED_FILE);
        Self {
            plugins_dir,
            installed_file,
            cache: RwLock::new(InstalledPlugins::default()),
        }
    }

    /// 加载已安装插件列表
    async fn load_installed(&mut self) -> Result<(), AppError> {
        if self.installed_file.exists() {
            let content = fs::read_to_string(&self.installed_file)
                .await
                .map_err(|e| AppError::Io(e))?;

            let installed: InstalledPlugins =
                serde_json::from_str(&content).map_err(|e| AppError::Serialization(e.to_string()))?;

            *self.cache.write().await = installed;
        }
        Ok(())
    }

    /// 保存已安装插件列表
    async fn save_installed(&self) -> Result<(), AppError> {
        let cache = self.cache.read().await;
        let content = serde_json::to_string_pretty(&*cache)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        fs::write(&self.installed_file, content)
            .await
            .map_err(|e| AppError::Io(e))?;

        Ok(())
    }

    /// 获取所有已安装插件
    pub async fn get_all_plugins(&self) -> Vec<Plugin> {
        let cache = self.cache.read().await;
        cache.plugins.values().cloned().collect()
    }

    /// 根据ID获取插件
    pub async fn get_plugin(&self, id: &str) -> Option<Plugin> {
        let cache = self.cache.read().await;
        cache.plugins.get(id).cloned()
    }

    /// 安装插件
    pub async fn install_plugin(
        &self,
        manifest: PluginManifest,
        plugin_files: HashMap<String, Vec<u8>>,
    ) -> Result<Plugin, AppError> {
        let plugin_dir = self.plugins_dir.join(&manifest.id);

        // 检查插件是否已存在
        if plugin_dir.exists() {
            return Err(AppError::Config(ConfigError::ValidationFailed(format!(
                "插件 {} 已存在",
                manifest.id
            ))));
        }

        // 创建插件目录
        fs::create_dir_all(&plugin_dir)
            .await
            .map_err(|e| AppError::Io(e))?;

        // 写入插件文件
        for (file_name, content) in plugin_files {
            let file_path = plugin_dir.join(&file_name);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).await.map_err(|e| AppError::Io(e))?;
            }
            fs::write(&file_path, content)
                .await
                .map_err(|e| AppError::Io(e))?;
        }

        // 写入清单文件
        let manifest_path = plugin_dir.join("manifest.json");
        let manifest_content = serde_json::to_string_pretty(&manifest)
            .map_err(|e| AppError::Serialization(e.to_string()))?;
        fs::write(&manifest_path, manifest_content)
            .await
            .map_err(|e| AppError::Io(e))?;

        // 创建默认配置
        if let Some(default_config) = &manifest.default_config {
            let config_path = plugin_dir.join("config.json");
            let config_content = serde_json::to_string_pretty(default_config)
                .map_err(|e| AppError::Serialization(e.to_string()))?;
            fs::write(&config_path, config_content)
                .await
                .map_err(|e| AppError::Io(e))?;
        }

        // 添加到已安装列表
        let mut plugin: Plugin = manifest.into();
        plugin.created_at = chrono::Utc::now().to_rfc3339();
        plugin.updated_at = plugin.created_at.clone();

        {
            let mut cache = self.cache.write().await;
            cache.plugins.insert(plugin.id.clone(), plugin.clone());
            cache.version = Self::CURRENT_VERSION;
        }

        self.save_installed().await?;

        Ok(plugin)
    }

    /// 从本地目录安装插件（用于开发或离线安装）
    pub async fn install_from_directory(&self, source_dir: &Path) -> Result<Plugin, AppError> {
        // 读取清单文件
        let manifest_path = source_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Err(AppError::Config(ConfigError::FileNotFound(
                "manifest.json not found".to_string(),
            )));
        }

        let manifest_content = fs::read_to_string(&manifest_path)
            .await
            .map_err(|e| AppError::Io(e))?;
        let manifest: PluginManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        // 收集所有插件文件
        let mut plugin_files: HashMap<String, Vec<u8>> = HashMap::new();

        let mut entries = fs::read_dir(source_dir)
            .await
            .map_err(|e| AppError::Io(e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| AppError::Io(e))?
        {
            let path = entry.path();
            if path.is_file() {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| AppError::Serialization("Invalid file name".to_string()))?
                    .to_string();
                let content = fs::read(&path).await.map_err(|e| AppError::Io(e))?;
                plugin_files.insert(file_name, content);
            }
        }

        self.install_plugin(manifest, plugin_files).await
    }

    /// 卸载插件
    pub async fn uninstall_plugin(&self, id: &str) -> Result<bool, AppError> {
        let plugin_dir = self.plugins_dir.join(id);

        if !plugin_dir.exists() {
            return Ok(false);
        }

        // 删除插件目录
        fs::remove_dir_all(&plugin_dir)
            .await
            .map_err(|e| AppError::Io(e))?;

        // 从已安装列表中移除
        {
            let mut cache = self.cache.write().await;
            cache.plugins.remove(id);
        }

        self.save_installed().await?;

        Ok(true)
    }

    /// 启用插件
    pub async fn enable_plugin(&self, id: &str) -> Result<Plugin, AppError> {
        let mut cache = self.cache.write().await;

        if let Some(plugin) = cache.plugins.get_mut(id) {
            plugin.is_enabled = true;
            plugin.updated_at = chrono::Utc::now().to_rfc3339();

            let updated_plugin = plugin.clone();
            drop(cache);
            self.save_installed().await?;

            Ok(updated_plugin)
        } else {
            Err(AppError::Config(ConfigError::FileNotFound(format!(
                "Plugin not found: {}",
                id
            ))))
        }
    }

    /// 禁用插件
    pub async fn disable_plugin(&self, id: &str) -> Result<Plugin, AppError> {
        let mut cache = self.cache.write().await;

        if let Some(plugin) = cache.plugins.get_mut(id) {
            plugin.is_enabled = false;
            plugin.updated_at = chrono::Utc::now().to_rfc3339();

            let updated_plugin = plugin.clone();
            drop(cache);
            self.save_installed().await?;

            Ok(updated_plugin)
        } else {
            Err(AppError::Config(ConfigError::FileNotFound(format!(
                "Plugin not found: {}",
                id
            ))))
        }
    }

    /// 获取插件配置
    pub async fn get_plugin_config(&self, id: &str) -> Result<Option<PluginConfig>, AppError> {
        let config_path = self.plugins_dir.join(id).join("config.json");

        if !config_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&config_path)
            .await
            .map_err(|e| AppError::Io(e))?;
        let config: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        Ok(Some(PluginConfig {
            plugin_id: id.to_string(),
            config,
        }))
    }

    /// 更新插件配置
    pub async fn update_plugin_config(
        &self,
        id: &str,
        config: serde_json::Value,
    ) -> Result<PluginConfig, AppError> {
        // 检查插件是否存在
        if !self.cache.read().await.plugins.contains_key(id) {
            return Err(AppError::Config(ConfigError::FileNotFound(format!(
                "Plugin not found: {}",
                id
            ))));
        }

        let config_path = self.plugins_dir.join(id).join("config.json");
        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        fs::write(&config_path, content)
            .await
            .map_err(|e| AppError::Io(e))?;

        Ok(PluginConfig {
            plugin_id: id.to_string(),
            config,
        })
    }

    /// 获取插件清单
    pub async fn get_plugin_manifest(&self, id: &str) -> Result<Option<PluginManifest>, AppError> {
        let manifest_path = self.plugins_dir.join(id).join("manifest.json");

        if !manifest_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&manifest_path)
            .await
            .map_err(|e| AppError::Io(e))?;
        let manifest: PluginManifest = serde_json::from_str(&content)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        Ok(Some(manifest))
    }

    /// 检查插件是否已安装
    pub async fn is_installed(&self, id: &str) -> bool {
        self.cache.read().await.plugins.contains_key(id)
    }

    /// 获取已启用的插件
    pub async fn get_enabled_plugins(&self) -> Vec<Plugin> {
        self.get_all_plugins()
            .await
            .into_iter()
            .filter(|p| p.is_enabled)
            .collect()
    }

    /// 获取插件入口文件路径
    pub fn get_plugin_entry_path(&self, id: &str, entry_point: &str) -> PathBuf {
        self.plugins_dir.join(id).join(entry_point)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        // 使用临时目录作为默认配置
        let temp_dir = std::env::temp_dir().join("openclaw_plugins");
        let installed_file = temp_dir.join("installed.json");

        Self {
            plugins_dir: temp_dir,
            installed_file,
            cache: RwLock::new(InstalledPlugins::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_manager() -> (PluginManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().join("plugins");
        fs::create_dir_all(&plugins_dir).await.unwrap();

        let manager = PluginManager::from_directory(plugins_dir);

        (manager, temp_dir)
    }

    fn create_test_manifest(id: &str) -> PluginManifest {
        PluginManifest {
            id: id.to_string(),
            name: format!("Test Plugin {}", id),
            version: "1.0.0".to_string(),
            description: Some("A test plugin".to_string()),
            author: Some("Test Author".to_string()),
            plugin_type: "lua".to_string(),
            entry_point: "main.lua".to_string(),
            config_schema: None,
            default_config: None,
            dependencies: vec![],
            min_app_version: None,
        }
    }

    #[tokio::test]
    async fn test_install_plugin() {
        let (manager, _temp) = create_test_manager().await;

        let manifest = create_test_manifest("test-plugin-1");
        let mut files = HashMap::new();
        files.insert("main.lua".to_string(), b"print('hello')".to_vec());

        let plugin = manager.install_plugin(manifest, files).await.unwrap();

        assert_eq!(plugin.id, "test-plugin-1");
        assert_eq!(plugin.name, "Test Plugin test-plugin-1");
        assert!(!plugin.is_enabled);

        // 验证文件是否写入
        let entry_path = manager.get_plugin_entry_path(&plugin.id, &plugin.entry_point);
        assert!(entry_path.exists());
    }

    #[tokio::test]
    async fn test_enable_disable_plugin() {
        let (manager, _temp) = create_test_manager().await;

        // 先安装插件
        let manifest = create_test_manifest("test-plugin-2");
        let mut files = HashMap::new();
        files.insert("main.lua".to_string(), b"print('hello')".to_vec());
        manager.install_plugin(manifest, files).await.unwrap();

        // 启用
        let enabled = manager.enable_plugin("test-plugin-2").await.unwrap();
        assert!(enabled.is_enabled);

        // 禁用
        let disabled = manager.disable_plugin("test-plugin-2").await.unwrap();
        assert!(!disabled.is_enabled);
    }

    #[tokio::test]
    async fn test_uninstall_plugin() {
        let (manager, _temp) = create_test_manager().await;

        // 安装插件
        let manifest = create_test_manifest("test-plugin-3");
        let mut files = HashMap::new();
        files.insert("main.lua".to_string(), b"print('hello')".to_vec());
        manager.install_plugin(manifest, files).await.unwrap();

        // 卸载
        let result = manager.uninstall_plugin("test-plugin-3").await.unwrap();
        assert!(result);

        // 验证已删除
        let plugin_dir = manager.plugins_dir.join("test-plugin-3");
        assert!(!plugin_dir.exists());

        // 再次卸载应该返回 false
        let result = manager.uninstall_plugin("test-plugin-3").await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_get_all_plugins() {
        let (manager, _temp) = create_test_manager().await;

        // 安装多个插件
        for i in 1..=3 {
            let manifest = create_test_manifest(&format!("test-plugin-{}", i));
            let mut files = HashMap::new();
            files.insert("main.lua".to_string(), b"print('hello')".to_vec());
            manager.install_plugin(manifest, files).await.unwrap();
        }

        let plugins = manager.get_all_plugins().await;
        assert_eq!(plugins.len(), 3);
    }

    #[tokio::test]
    async fn test_plugin_config() {
        let (manager, _temp) = create_test_manager().await;

        // 安装带默认配置的插件
        let mut manifest = create_test_manifest("test-plugin-config");
        manifest.default_config = Some(serde_json::json!({
            "key1": "value1",
            "key2": 42
        }));

        let mut files = HashMap::new();
        files.insert("main.lua".to_string(), b"print('hello')".to_vec());
        manager.install_plugin(manifest, files).await.unwrap();

        // 读取配置
        let config = manager.get_plugin_config("test-plugin-config").await.unwrap();
        assert!(config.is_some());
        let config = config.unwrap();
        assert_eq!(config.config["key1"], "value1");
        assert_eq!(config.config["key2"], 42);

        // 更新配置
        let new_config = serde_json::json!({
            "key1": "updated",
            "key2": 100
        });
        let updated = manager
            .update_plugin_config("test-plugin-config", new_config.clone())
            .await
            .unwrap();
        assert_eq!(updated.config["key1"], "updated");
        assert_eq!(updated.config["key2"], 100);
    }
}
