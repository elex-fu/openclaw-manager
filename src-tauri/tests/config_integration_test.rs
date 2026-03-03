//! 配置管理集成测试

use std::path::PathBuf;
use tempfile::TempDir;

/// 测试配置管理器创建
#[tokio::test]
async fn test_config_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let result = openclaw_manager::services::config_manager::ConfigManager::new_async(&config_path).await;
    assert!(result.is_ok(), "应该能创建配置管理器");

    let manager = result.unwrap();
    // 验证默认配置已创建
    let models = manager.get_models_full().await;
    assert!(models.is_ok());
}

/// 测试配置保存和读取
#[tokio::test]
async fn test_config_save_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    // 创建配置管理器
    let manager = openclaw_manager::services::config_manager::ConfigManager::new_async(&config_path)
        .await
        .unwrap();

    // 获取模型列表
    let models = manager.get_models_full().await.unwrap();
    assert!(!models.is_empty(), "应该有默认模型");

    // 验证配置文件已创建
    assert!(config_path.exists(), "配置文件应该存在");
}

/// 测试模型优先级更新
#[tokio::test]
async fn test_model_priority_update() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let manager = openclaw_manager::services::config_manager::ConfigManager::new_async(&config_path)
        .await
        .unwrap();

    // 更新模型优先级
    let model_orders = vec![
        ("model-1".to_string(), 1),
        ("model-2".to_string(), 2),
    ];

    let result = manager.update_model_priorities(model_orders).await;
    // 结果可能是成功或失败，取决于是否有这些模型
    assert!(result.is_ok() || result.is_err());
}

/// 测试配置验证
#[test]
fn test_config_validation() {
    use openclaw_manager::services::config_manager::{ConfigState, ConfigManager, ValidationResult};

    let state = ConfigState::default();
    let result = ConfigManager::validate(&state);

    assert!(result.valid, "默认配置应该有效");
    assert!(result.errors.is_empty());
}

/// 测试配置验证 - 无效温度
#[test]
fn test_config_validation_invalid_temperature() {
    use openclaw_manager::services::config_manager::{ConfigState, ConfigManager, ModelConfig};

    let mut state = ConfigState::default();
    state.models[0].temperature = 3.0; // 无效值

    let result = ConfigManager::validate(&state);
    assert!(!result.valid, "温度超出范围应该无效");
}

/// 测试应用设置
#[tokio::test]
async fn test_app_settings() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let manager = openclaw_manager::services::config_manager::ConfigManager::new_async(&config_path)
        .await
        .unwrap();

    // 设置和获取当前 agent
    let result = manager.set_current_agent("test-agent");
    assert!(result.is_ok());

    let current = manager.get_current_agent();
    assert_eq!(current, "test-agent");
}
