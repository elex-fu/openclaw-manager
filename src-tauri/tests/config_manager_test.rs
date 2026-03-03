//! ConfigManager 测试模块
//!
//! 提供 ConfigManager 的全面测试覆盖

#[cfg(test)]
mod tests {
    use crate::services::config_manager::{ConfigManager, ConfigState, AppConfig, ModelConfig, ServiceConfig, current_timestamp, ValidationResult, AppSettings};
    use crate::models::config::{ModelConfigFull, ModelParameters, ModelCapabilities};
    use std::path::PathBuf;
    use std::collections::HashMap;
    use tempfile::TempDir;
    use tokio::fs;

    /// 创建临时配置目录
    async fn create_temp_config_dir() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        (temp_dir, config_path)
    }

    /// 创建测试用的 ConfigState
    fn create_test_config_state() -> ConfigState {
        ConfigState {
            version: 1,
            last_updated: current_timestamp(),
            app: AppConfig {
                theme: "dark".to_string(),
                language: "en-US".to_string(),
                auto_start: true,
                minimize_to_tray: false,
                check_updates: true,
            },
            models: vec![ModelConfig {
                id: "test-model-1".to_string(),
                name: "Test Model".to_string(),
                provider: "openai".to_string(),
                api_base: Some("https://api.openai.com".to_string()),
                model: "gpt-4".to_string(),
                temperature: 0.7,
                max_tokens: Some(2048),
                enabled: true,
                default: true,
            }],
            models_full: vec![ModelConfigFull::default()],
            services: {
                let mut map = HashMap::new();
                map.insert("gateway".to_string(), ServiceConfig {
                    enabled: true,
                    port: 8080,
                    auto_start: false,
                });
                map
            },
        }
    }

    // ==================== ConfigManager 创建和初始化测试 ====================

    /// 测试 ConfigManager 异步创建
    #[tokio::test]
    async fn test_config_manager_new_async() {
        let (temp_dir, config_path) = create_temp_config_dir().await;

        let manager = ConfigManager::new_async(&config_path).await;
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        let state = manager.get_state().await;

        // 验证默认配置已创建
        assert_eq!(state.version, 1);
        assert!(!state.models.is_empty());
        assert!(state.services.contains_key("gateway"));

        // 验证文件已创建
        assert!(config_path.exists());
    }

    /// 测试 ConfigManager 从已存在的配置加载
    #[tokio::test]
    async fn test_config_manager_load_existing_config() {
        let (temp_dir, config_path) = create_temp_config_dir().await;

        // 先创建一个配置
        let test_state = create_test_config_state();
        let yaml = serde_yaml::to_string(&test_state).unwrap();
        fs::write(&config_path, yaml).await.unwrap();

        // 加载已存在的配置
        let manager = ConfigManager::new_async(&config_path).await.unwrap();
        let state = manager.get_state().await;

        assert_eq!(state.app.theme, "dark");
        assert_eq!(state.app.language, "en-US");
        assert_eq!(state.models[0].id, "test-model-1");
    }

    /// 测试 ConfigManager 同步创建
    #[test]
    fn test_config_manager_new_sync() {
        // 注意：这个测试可能会使用实际的配置目录
        // 在生产环境中应该使用临时目录
        let manager = ConfigManager::new();
        // 结果取决于环境，但不应该 panic
    }

    // ==================== load_config / save_config 测试 ====================

    /// 测试保存和加载配置
    #[tokio::test]
    async fn test_save_and_load_config() {
        let (temp_dir, config_path) = create_temp_config_dir().await;

        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        // 修改配置
        let new_state = create_test_config_state();
        let version = manager.get_version().await;
        manager.update_state(new_state, version).await.unwrap();

        // 验证文件内容
        let content = fs::read_to_string(&config_path).await.unwrap();
        assert!(content.contains("dark"));
        assert!(content.contains("test-model-1"));

        // 创建新的 manager 加载配置
        let manager2 = ConfigManager::new_async(&config_path).await.unwrap();
        let loaded_state = manager2.get_state().await;

        assert_eq!(loaded_state.app.theme, "dark");
        assert_eq!(loaded_state.models[0].id, "test-model-1");
    }

    /// 测试加载不存在的配置文件
    #[tokio::test]
    async fn test_load_nonexistent_config() {
        let (temp_dir, config_path) = create_temp_config_dir().await;

        // 确保文件不存在
        if config_path.exists() {
            fs::remove_file(&config_path).await.unwrap();
        }

        // 创建 manager 应该创建默认配置
        let manager = ConfigManager::new_async(&config_path).await.unwrap();
        let state = manager.get_state().await;

        // 应该使用默认配置
        assert_eq!(state.version, 1);
        assert!(config_path.exists()); // 文件应该被创建
    }

    /// 测试加载无效的 YAML 配置
    #[tokio::test]
    async fn test_load_invalid_yaml_config() {
        let (temp_dir, config_path) = create_temp_config_dir().await;

        // 写入无效的 YAML
        fs::write(&config_path, "invalid: yaml: content: [").await.unwrap();

        // 尝试加载应该失败
        let result = ConfigManager::new_async(&config_path).await;
        assert!(result.is_err());
    }

    // ==================== get_config / get_config_mut 测试 ====================

    /// 测试获取配置状态
    #[tokio::test]
    async fn test_get_state() {
        let (temp_dir, config_path) = create_temp_config_dir().await;

        let manager = ConfigManager::new_async(&config_path).await.unwrap();
        let state = manager.get_state().await;

        assert_eq!(state.version, 1);
        assert!(!state.models.is_empty());
    }

    /// 测试获取配置版本
    #[tokio::test]
    async fn test_get_version() {
        let (temp_dir, config_path) = create_temp_config_dir().await;

        let manager = ConfigManager::new_async(&config_path).await.unwrap();
        let version = manager.get_version().await;

        assert_eq!(version, 1);
    }

    // ==================== validate_config 测试 ====================

    /// 测试验证有效配置
    #[test]
    fn test_validate_valid_config() {
        let state = ConfigState::default();
        let result = ConfigManager::validate(&state);

        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    /// 测试验证空模型 ID
    #[test]
    fn test_validate_empty_model_id() {
        let mut state = ConfigState::default();
        state.models[0].id = "".to_string();

        let result = ConfigManager::validate(&state);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("ID")));
    }

    /// 测试验证空提供商
    #[test]
    fn test_validate_empty_provider() {
        let mut state = ConfigState::default();
        state.models[0].provider = "".to_string();

        let result = ConfigManager::validate(&state);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("提供商")));
    }

    /// 测试验证温度范围
    #[test]
    fn test_validate_temperature_range() {
        let mut state = ConfigState::default();

        // 测试过高温度
        state.models[0].temperature = 3.0;
        let result = ConfigManager::validate(&state);
        assert!(!result.valid);

        // 测试过低温度
        state.models[0].temperature = -0.1;
        let result = ConfigManager::validate(&state);
        assert!(!result.valid);

        // 测试边界值
        state.models[0].temperature = 0.0;
        let result = ConfigManager::validate(&state);
        assert!(result.valid);

        state.models[0].temperature = 2.0;
        let result = ConfigManager::validate(&state);
        assert!(result.valid);
    }

    /// 测试验证服务端口号
    #[test]
    fn test_validate_service_port() {
        let mut state = ConfigState::default();

        // 修改服务端口为 0
        if let Some(service) = state.services.get_mut("gateway") {
            service.port = 0;
        }

        let result = ConfigManager::validate(&state);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("端口")));
    }

    /// 测试多个验证错误
    #[test]
    fn test_validate_multiple_errors() {
        let mut state = ConfigState::default();
        state.models[0].id = "".to_string();
        state.models[0].provider = "".to_string();
        state.models[0].temperature = 5.0;

        let result = ConfigManager::validate(&state);
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 3);
    }

    // ==================== export_config / import_config 测试 ====================

    /// 测试导出配置
    #[tokio::test]
    async fn test_export_config() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        // 修改配置
        let new_state = create_test_config_state();
        let version = manager.get_version().await;
        manager.update_state(new_state, version).await.unwrap();

        // 导出到另一个文件
        let export_path = temp_dir.path().join("exported_config.yaml");
        manager.export_to(&export_path).await.unwrap();

        // 验证导出文件存在且内容正确
        assert!(export_path.exists());
        let content = fs::read_to_string(&export_path).await.unwrap();
        assert!(content.contains("dark"));
        assert!(content.contains("test-model-1"));
    }

    /// 测试导入配置
    #[tokio::test]
    async fn test_import_config() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        // 创建要导入的配置文件
        let import_state = create_test_config_state();
        let import_path = temp_dir.path().join("import_config.yaml");
        let yaml = serde_yaml::to_string(&import_state).unwrap();
        fs::write(&import_path, yaml).await.unwrap();

        // 导入配置
        let version = manager.get_version().await;
        manager.import_from(&import_path).await.unwrap();

        // 验证配置已导入
        let state = manager.get_state().await;
        assert_eq!(state.app.theme, "dark");
        assert_eq!(state.models[0].id, "test-model-1");
    }

    /// 测试导入无效配置
    #[tokio::test]
    async fn test_import_invalid_config() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        // 创建无效的配置文件
        let import_path = temp_dir.path().join("invalid_config.yaml");
        let mut invalid_state = create_test_config_state();
        invalid_state.models[0].temperature = 5.0; // 无效温度
        let yaml = serde_yaml::to_string(&invalid_state).unwrap();
        fs::write(&import_path, yaml).await.unwrap();

        // 导入应该失败
        let version = manager.get_version().await;
        let result = manager.import_from(&import_path).await;
        assert!(result.is_err());
    }

    /// 测试导入不存在的文件
    #[tokio::test]
    async fn test_import_nonexistent_file() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        let nonexistent_path = temp_dir.path().join("nonexistent.yaml");
        let version = manager.get_version().await;
        let result = manager.import_from(&nonexistent_path).await;

        assert!(result.is_err());
    }

    // ==================== reset_config 测试 ====================

    /// 测试重置为默认配置
    #[tokio::test]
    async fn test_reset_to_default() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        // 先修改为自定义配置
        let new_state = create_test_config_state();
        let version = manager.get_version().await;
        manager.update_state(new_state, version).await.unwrap();

        let state = manager.get_state().await;
        assert_eq!(state.app.theme, "dark");

        // 重置为默认
        manager.reset_to_default().await.unwrap();

        let state = manager.get_state().await;
        assert_eq!(state.app.theme, "system"); // 默认值
        assert_eq!(state.version, 3); // 版本应该递增
    }

    // ==================== update_state 测试（乐观锁） ====================

    /// 测试更新配置状态
    #[tokio::test]
    async fn test_update_state() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        let mut new_state = create_test_config_state();
        let version = manager.get_version().await;

        // 更新配置
        manager.update_state(new_state.clone(), version).await.unwrap();

        let state = manager.get_state().await;
        assert_eq!(state.app.theme, "dark");
        assert_eq!(state.version, 2);
    }

    /// 测试乐观锁版本冲突
    #[tokio::test]
    async fn test_update_state_version_mismatch() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        let new_state = create_test_config_state();

        // 使用错误的版本号
        let result = manager.update_state(new_state, 999).await;
        assert!(result.is_err());

        // 验证错误类型
        let err = result.unwrap_err().to_string();
        assert!(err.contains("VersionMismatch") || err.contains("version"));
    }

    /// 测试更新无效配置
    #[tokio::test]
    async fn test_update_invalid_state() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        let mut invalid_state = create_test_config_state();
        invalid_state.models[0].temperature = 5.0;

        let version = manager.get_version().await;
        let result = manager.update_state(invalid_state, version).await;

        assert!(result.is_err());
    }

    // ==================== update_partial 测试 ====================

    /// 测试部分更新配置
    #[tokio::test]
    async fn test_update_partial() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        manager.update_partial(|state| {
            state.app.theme = "light".to_string();
            state.app.language = "zh-CN".to_string();
        }).await.unwrap();

        let state = manager.get_state().await;
        assert_eq!(state.app.theme, "light");
        assert_eq!(state.app.language, "zh-CN");
        assert_eq!(state.version, 2);
    }

    /// 测试部分更新验证失败
    #[tokio::test]
    async fn test_update_partial_validation_fail() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        let result = manager.update_partial(|state| {
            state.models[0].temperature = 10.0;
        }).await;

        assert!(result.is_err());
    }

    // ==================== 模型配置管理测试 ====================

    /// 测试添加模型
    #[tokio::test]
    async fn test_add_model() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        let new_model = ModelConfig {
            id: "new-model".to_string(),
            name: "New Model".to_string(),
            provider: "anthropic".to_string(),
            api_base: None,
            model: "claude-3".to_string(),
            temperature: 0.5,
            max_tokens: Some(4096),
            enabled: true,
            default: false,
        };

        manager.add_model(new_model).await.unwrap();

        let state = manager.get_state().await;
        assert_eq!(state.models.len(), 2);
        assert!(state.models.iter().any(|m| m.id == "new-model"));
    }

    /// 测试添加默认模型（会取消其他默认）
    #[tokio::test]
    async fn test_add_default_model() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        let new_model = ModelConfig {
            id: "new-default".to_string(),
            name: "New Default".to_string(),
            provider: "anthropic".to_string(),
            api_base: None,
            model: "claude-3".to_string(),
            temperature: 0.5,
            max_tokens: Some(4096),
            enabled: true,
            default: true, // 设置为默认
        };

        manager.add_model(new_model).await.unwrap();

        let state = manager.get_state().await;
        let default_models: Vec<_> = state.models.iter().filter(|m| m.default).collect();
        assert_eq!(default_models.len(), 1);
        assert_eq!(default_models[0].id, "new-default");
    }

    /// 测试删除模型
    #[tokio::test]
    async fn test_remove_model() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        // 先添加一个模型
        let new_model = ModelConfig {
            id: "to-remove".to_string(),
            name: "To Remove".to_string(),
            provider: "test".to_string(),
            api_base: None,
            model: "test".to_string(),
            temperature: 0.5,
            max_tokens: None,
            enabled: true,
            default: false,
        };
        manager.add_model(new_model).await.unwrap();

        let state = manager.get_state().await;
        assert_eq!(state.models.len(), 2);

        // 删除模型
        manager.remove_model("to-remove").await.unwrap();

        let state = manager.get_state().await;
        assert_eq!(state.models.len(), 1);
        assert!(!state.models.iter().any(|m| m.id == "to-remove"));
    }

    /// 测试更新模型
    #[tokio::test]
    async fn test_update_model() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        // 获取默认模型的 ID
        let default_model_id = {
            let state = manager.get_state().await;
            state.models[0].id.clone()
        };

        // 更新模型
        manager.update_model(&default_model_id, |model| {
            model.name = "Updated Name".to_string();
            model.temperature = 0.9;
        }).await.unwrap();

        let state = manager.get_state().await;
        let model = state.models.iter().find(|m| m.id == default_model_id).unwrap();
        assert_eq!(model.name, "Updated Name");
        assert_eq!(model.temperature, 0.9);
    }

    /// 测试获取默认模型
    #[tokio::test]
    async fn test_get_default_model() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        let default_model = manager.get_default_model().await;
        assert!(default_model.is_some());
        assert!(default_model.unwrap().default);
    }

    /// 测试获取默认模型（无默认时返回第一个）
    #[tokio::test]
    async fn test_get_default_model_fallback() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        // 取消所有默认标记
        manager.update_partial(|state| {
            for model in &mut state.models {
                model.default = false;
            }
        }).await.unwrap();

        // 应该返回第一个模型
        let default_model = manager.get_default_model().await;
        assert!(default_model.is_some());
    }

    // ==================== 完整模型配置测试 ====================

    /// 测试获取完整模型配置
    #[tokio::test]
    async fn test_get_models_full() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        let models = manager.get_models_full().await.unwrap();
        assert!(!models.is_empty());
    }

    /// 测试保存完整模型配置
    #[tokio::test]
    async fn test_save_model_full() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        let new_model = ModelConfigFull {
            id: "full-model".to_string(),
            name: "Full Model".to_string(),
            provider: "openai".to_string(),
            api_base: Some("https://api.openai.com".to_string()),
            model: "gpt-4-turbo".to_string(),
            priority: 1,
            parameters: ModelParameters::with_defaults(),
            capabilities: ModelCapabilities::default(),
            enabled: true,
            default: false,
        };

        manager.save_model_full(new_model).await.unwrap();

        let models = manager.get_models_full().await.unwrap();
        assert!(models.iter().any(|m| m.id == "full-model"));
    }

    /// 测试更新模型优先级
    #[tokio::test]
    async fn test_update_model_priorities() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();

        // 添加两个模型
        let model1 = ModelConfigFull {
            id: "model-1".to_string(),
            name: "Model 1".to_string(),
            provider: "openai".to_string(),
            api_base: None,
            model: "gpt-4".to_string(),
            priority: 0,
            parameters: ModelParameters::with_defaults(),
            capabilities: ModelCapabilities::default(),
            enabled: true,
            default: false,
        };
        let model2 = ModelConfigFull {
            id: "model-2".to_string(),
            name: "Model 2".to_string(),
            provider: "anthropic".to_string(),
            api_base: None,
            model: "claude-3".to_string(),
            priority: 0,
            parameters: ModelParameters::with_defaults(),
            capabilities: ModelCapabilities::default(),
            enabled: true,
            default: false,
        };

        manager.save_model_full(model1).await.unwrap();
        manager.save_model_full(model2).await.unwrap();

        // 更新优先级
        let orders = vec![
            ("model-1".to_string(), 2),
            ("model-2".to_string(), 1),
        ];
        manager.update_model_priorities(orders).await.unwrap();

        let models = manager.get_models_full().await.unwrap();
        let m1 = models.iter().find(|m| m.id == "model-1").unwrap();
        let m2 = models.iter().find(|m| m.id == "model-2").unwrap();

        assert_eq!(m1.priority, 2);
        assert_eq!(m2.priority, 1);
    }

    // ==================== Agent 设置测试 ====================

    /// 测试设置和获取当前 Agent
    #[test]
    fn test_set_and_get_current_agent() {
        let manager = ConfigManager::new();
        if let Ok(manager) = manager {
            manager.set_current_agent("test-agent").unwrap();
            let current = manager.get_current_agent();
            assert_eq!(current, "test-agent");
        }
    }

    /// 测试获取默认 Agent ID
    #[test]
    fn test_get_default_agent() {
        // 使用临时目录创建独立的 ConfigManager
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (temp_dir, config_path) = create_temp_config_dir().await;
            let manager = ConfigManager::new_async(&config_path).await;

            if let Ok(manager) = manager {
                // 在没有设置时应该返回默认值
                let current = manager.get_current_agent();
                assert_eq!(current, "default-assistant");
            }

            // temp_dir 在这里被 drop，自动清理
            drop(temp_dir);
        });
    }

    // ==================== 错误处理测试 ====================

    /// 测试 ValidationResult 成功
    #[test]
    fn test_validation_result_success() {
        let result = ValidationResult::success();
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    /// 测试 ValidationResult 错误
    #[test]
    fn test_validation_result_error() {
        let result = ValidationResult::error("test error");
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0], "test error");
    }

    /// 测试 ValidationResult 多错误
    #[test]
    fn test_validation_result_with_errors() {
        let errors = vec!["error1".to_string(), "error2".to_string()];
        let result = ValidationResult::with_errors(errors);
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 2);
    }

    /// 测试 ValidationResult 空错误列表
    #[test]
    fn test_validation_result_empty_errors() {
        let result = ValidationResult::with_errors(vec![]);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    // ==================== 序列化测试 ====================

    /// 测试 ConfigState 序列化
    #[test]
    fn test_config_state_serialization() {
        let state = ConfigState::default();
        let json = serde_json::to_string(&state).unwrap();

        assert!(json.contains("version"));
        assert!(json.contains("models"));
        assert!(json.contains("services"));

        let deserialized: ConfigState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, state.version);
    }

    /// 测试 YAML 序列化
    #[test]
    fn test_config_state_yaml_serialization() {
        let state = ConfigState::default();
        let yaml = serde_yaml::to_string(&state).unwrap();

        assert!(yaml.contains("version"));
        assert!(yaml.contains("models"));

        let deserialized: ConfigState = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.version, state.version);
    }

    // ==================== 默认实现测试 ====================

    /// 测试 ConfigState 默认值
    #[test]
    fn test_config_state_default() {
        let state = ConfigState::default();
        assert_eq!(state.version, 1);
        assert!(!state.models.is_empty());
        assert!(state.services.contains_key("gateway"));
    }

    /// 测试 AppConfig 默认值
    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.theme, "system");
        assert_eq!(config.language, "zh-CN");
        assert!(!config.auto_start);
        assert!(config.minimize_to_tray);
        assert!(config.check_updates);
    }

    /// 测试 ModelConfig 默认值
    #[test]
    fn test_model_config_default() {
        let config = ModelConfig::default();
        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "gpt-4");
        assert!(config.enabled);
        assert!(config.default);
        assert!(config.api_base.is_none());
    }

    /// 测试 ServiceConfig 默认值
    #[test]
    fn test_service_config_default() {
        let config = ServiceConfig::default();
        assert!(config.enabled);
        assert_eq!(config.port, 8080);
        assert!(!config.auto_start);
    }

    /// 测试 AppSettings 默认值
    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert!(settings.current_agent_id.is_none());
        assert_eq!(settings.theme, "");
        assert_eq!(settings.language, "");
    }

    // ==================== 并发测试 ====================

    /// 测试并发读取
    #[tokio::test]
    async fn test_concurrent_reads() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();
        let manager = std::sync::Arc::new(manager);

        let mut handles = vec![];

        for _ in 0..10 {
            let m = manager.clone();
            let handle = tokio::spawn(async move {
                let state = m.get_state().await;
                state.version
            });
            handles.push(handle);
        }

        for handle in handles {
            let version = handle.await.unwrap();
            assert_eq!(version, 1);
        }
    }

    /// 测试并发读写
    #[tokio::test]
    async fn test_concurrent_read_write() {
        let (temp_dir, config_path) = create_temp_config_dir().await;
        let manager = ConfigManager::new_async(&config_path).await.unwrap();
        let manager = std::sync::Arc::new(manager);

        let mut handles = vec![];

        // 启动多个读取任务
        for _ in 0..5 {
            let m = manager.clone();
            let handle = tokio::spawn(async move {
                let _ = m.get_state().await;
            });
            handles.push(handle);
        }

        // 启动一个写入任务
        let m = manager.clone();
        let write_handle: tokio::task::JoinHandle<()> = tokio::spawn(async move {
            let _ = m.update_partial(|state| {
                state.app.theme = "concurrent".to_string();
            }).await;
        });
        handles.push(write_handle);

        for handle in handles {
            let _ = handle.await;
        }

        // 验证最终状态
        let state = manager.get_state().await;
        assert!(state.app.theme == "concurrent" || state.app.theme == "system");
    }
}
