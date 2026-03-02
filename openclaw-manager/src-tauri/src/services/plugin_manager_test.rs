//! 插件管理器测试

#[cfg(test)]
mod tests {
    use super::super::plugin_manager::{PluginManager, PluginManifest};
    use std::collections::HashMap;
    use tempfile::TempDir;
    use tokio::fs;

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
        let plugin_dir = manager.plugins_dir().join("test-plugin-3");
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

    #[tokio::test]
    async fn test_is_installed() {
        let (manager, _temp) = create_test_manager().await;

        // 安装前
        assert!(!manager.is_installed("test-plugin").await);

        // 安装
        let manifest = create_test_manifest("test-plugin");
        let mut files = HashMap::new();
        files.insert("main.lua".to_string(), b"print('hello')".to_vec());
        manager.install_plugin(manifest, files).await.unwrap();

        // 安装后
        assert!(manager.is_installed("test-plugin").await);
    }

    #[tokio::test]
    async fn test_get_enabled_plugins() {
        let (manager, _temp) = create_test_manager().await;

        // 安装两个插件
        for id in ["plugin-1", "plugin-2"] {
            let manifest = create_test_manifest(id);
            let mut files = HashMap::new();
            files.insert("main.lua".to_string(), b"print('hello')".to_vec());
            manager.install_plugin(manifest, files).await.unwrap();
        }

        // 启用一个
        manager.enable_plugin("plugin-1").await.unwrap();

        let enabled = manager.get_enabled_plugins().await;
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].id, "plugin-1");
    }

    #[tokio::test]
    async fn test_install_duplicate_plugin() {
        let (manager, _temp) = create_test_manager().await;

        let manifest = create_test_manifest("duplicate-plugin");
        let mut files = HashMap::new();
        files.insert("main.lua".to_string(), b"print('hello')".to_vec());

        // 第一次安装
        manager.install_plugin(manifest.clone(), files.clone()).await.unwrap();

        // 第二次安装应该失败
        let result = manager.install_plugin(manifest, files).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_plugin_manifest() {
        let (manager, _temp) = create_test_manager().await;

        let manifest = create_test_manifest("manifest-test");
        let mut files = HashMap::new();
        files.insert("main.lua".to_string(), b"print('hello')".to_vec());
        manager.install_plugin(manifest, files).await.unwrap();

        let retrieved = manager.get_plugin_manifest("manifest-test").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, "manifest-test");
        assert_eq!(retrieved.name, "Test Plugin manifest-test");
    }
}
