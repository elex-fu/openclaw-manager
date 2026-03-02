//! 配置迁移功能单元测试
//!
//! 测试配置版本比较、迁移逻辑、备份恢复等功能

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// 测试版本比较函数
    #[test]
    fn test_compare_versions() {
        // v1 < v2
        assert!(UpdateManager::compare_versions("1.0.0", "2.0.0").unwrap());
        assert!(UpdateManager::compare_versions("1.0.0", "1.1.0").unwrap());
        assert!(UpdateManager::compare_versions("1.0.0", "1.0.1").unwrap());

        // v1 == v2
        assert!(!UpdateManager::compare_versions("1.0.0", "1.0.0").unwrap());

        // v1 > v2
        assert!(!UpdateManager::compare_versions("2.0.0", "1.0.0").unwrap());
        assert!(!UpdateManager::compare_versions("1.1.0", "1.0.0").unwrap());
        assert!(!UpdateManager::compare_versions("1.0.1", "1.0.0").unwrap());

        // With v prefix
        assert!(UpdateManager::compare_versions("v1.0.0", "v2.0.0").unwrap());

        // Different version lengths
        assert!(UpdateManager::compare_versions("1.0", "1.0.0").unwrap());
        assert!(UpdateManager::compare_versions("1.0.0", "1.0.0.1").unwrap());
    }

    /// 测试版本比较错误处理
    #[test]
    fn test_compare_versions_invalid() {
        // Invalid version format
        assert!(UpdateManager::compare_versions("invalid", "1.0.0").is_err());
        assert!(UpdateManager::compare_versions("1.0.0", "invalid").is_err());
        assert!(UpdateManager::compare_versions("", "1.0.0").is_err());
    }

    /// 测试 v0.1.0 迁移
    #[test]
    fn test_migrate_to_v0_1_0() {
        let config = serde_yaml::Mapping::new();
        let mut config_value = serde_yaml::Value::Mapping(config);

        let result = UpdateManager::migrate_to_v0_1_0(&mut config_value);
        assert!(result.is_ok());

        // 验证 models 字段被添加
        if let serde_yaml::Value::Mapping(map) = config_value {
            assert!(map.contains_key(&serde_yaml::Value::String("models".to_string())));
            assert!(map.contains_key(&serde_yaml::Value::String("agents".to_string())));
        }
    }

    /// 测试 v0.2.0 迁移（移除 api_key）
    #[test]
    fn test_migrate_to_v0_2_0() {
        let mut config = serde_yaml::Mapping::new();
        config.insert(
            serde_yaml::Value::String("api_key".to_string()),
            serde_yaml::Value::String("secret_key".to_string()),
        );
        let mut config_value = serde_yaml::Value::Mapping(config);

        let result = UpdateManager::migrate_to_v0_2_0(&mut config_value);
        assert!(result.is_ok());

        // 验证 api_key 被移除
        if let serde_yaml::Value::Mapping(map) = config_value {
            assert!(!map.contains_key(&serde_yaml::Value::String("api_key".to_string())));
        }
    }

    /// 测试 v0.3.0 迁移（添加 plugins）
    #[test]
    fn test_migrate_to_v0_3_0() {
        let config = serde_yaml::Mapping::new();
        let mut config_value = serde_yaml::Value::Mapping(config);

        let result = UpdateManager::migrate_to_v0_3_0(&mut config_value);
        assert!(result.is_ok());

        // 验证 plugins 字段被添加
        if let serde_yaml::Value::Mapping(map) = config_value {
            assert!(map.contains_key(&serde_yaml::Value::String("plugins".to_string())));
        }
    }

    /// 测试配置版本检测
    #[test]
    fn test_config_version_detection() {
        let mut config = serde_yaml::Mapping::new();
        config.insert(
            serde_yaml::Value::String("version".to_string()),
            serde_yaml::Value::String("1.0.0".to_string()),
        );

        let version = config
            .get(&serde_yaml::Value::String("version".to_string()))
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0");

        assert_eq!(version, "1.0.0");

        // 测试无版本的情况
        let empty_config = serde_yaml::Mapping::new();
        let version = empty_config
            .get(&serde_yaml::Value::String("version".to_string()))
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0");
        assert_eq!(version, "0.0.0");
    }

    /// 测试配置迁移判断逻辑
    #[test]
    fn test_migration_decision_logic() {
        // 当前版本 0.0.0，目标版本 0.1.0 - 需要迁移
        assert!(
            UpdateManager::compare_versions("0.0.0", "0.1.0").unwrap(),
            "Should need migration from 0.0.0 to 0.1.0"
        );

        // 当前版本 0.1.0，目标版本 0.1.0 - 不需要迁移
        assert!(
            !UpdateManager::compare_versions("0.1.0", "0.1.0").unwrap(),
            "Should not need migration when versions are equal"
        );

        // 当前版本 0.2.0，目标版本 0.1.0 - 不需要迁移（回滚情况）
        assert!(
            !UpdateManager::compare_versions("0.2.0", "0.1.0").unwrap(),
            "Should not migrate to older version"
        );

        // 验证多版本迁移逻辑
        let current = "0.0.0";
        let target = "0.3.0";

        // 应该执行 0.1.0, 0.2.0, 0.3.0 的迁移
        assert!(UpdateManager::compare_versions(current, "0.1.0").unwrap());
        assert!(UpdateManager::compare_versions(current, "0.2.0").unwrap());
        assert!(UpdateManager::compare_versions(current, "0.3.0").unwrap());

        // 目标版本大于迁移版本时才执行
        assert!(!UpdateManager::compare_versions(target, "0.1.0").unwrap());
        assert!(!UpdateManager::compare_versions(target, "0.2.0").unwrap());
        assert!(!UpdateManager::compare_versions(target, "0.3.0").unwrap());
    }

    /// 测试复杂配置迁移场景
    #[test]
    fn test_complex_config_migration() {
        // 创建一个模拟的旧配置
        let mut config = serde_yaml::Mapping::new();

        // 添加旧版本字段
        config.insert(
            serde_yaml::Value::String("api_key".to_string()),
            serde_yaml::Value::String("sk-old-key".to_string()),
        );
        config.insert(
            serde_yaml::Value::String("version".to_string()),
            serde_yaml::Value::String("0.0.1".to_string()),
        );

        // 添加一些模型配置
        let models = serde_yaml::Value::Sequence(vec![]);
        config.insert(
            serde_yaml::Value::String("models".to_string()),
            models,
        );

        let mut config_value = serde_yaml::Value::Mapping(config);

        // 执行所有迁移
        let _ = UpdateManager::migrate_to_v0_1_0(&mut config_value);
        let _ = UpdateManager::migrate_to_v0_2_0(&mut config_value);
        let _ = UpdateManager::migrate_to_v0_3_0(&mut config_value);

        // 验证最终状态
        if let serde_yaml::Value::Mapping(map) = config_value {
            // api_key 应该被移除
            assert!(!map.contains_key(&serde_yaml::Value::String("api_key".to_string())));

            // agents 应该被添加
            assert!(map.contains_key(&serde_yaml::Value::String("agents".to_string())));

            // plugins 应该被添加
            assert!(map.contains_key(&serde_yaml::Value::String("plugins".to_string())));

            // 模型应该保留
            assert!(map.contains_key(&serde_yaml::Value::String("models".to_string())));
        }
    }

    /// 测试配置备份和恢复（模拟）
    #[test]
    fn test_config_backup_simulation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        let backup_path = temp_dir.path().join("config.yaml.backup");

        // 创建原始配置
        let original_config = r#"
version: "0.1.0"
models:
  - name: test-model
    provider: openai
"#;

        std::fs::write(&config_path, original_config).unwrap();

        // 模拟备份
        std::fs::copy(&config_path, &backup_path).unwrap();

        // 验证备份存在
        assert!(backup_path.exists());

        // 模拟配置更新
        let updated_config = r#"
version: "0.2.0"
models:
  - name: test-model
    provider: openai
agents: []
"#;
        std::fs::write(&config_path, updated_config).unwrap();

        // 验证配置已更新
        let current_content = std::fs::read_to_string(&config_path).unwrap();
        assert!(current_content.contains("0.2.0"));

        // 验证备份仍是旧版本
        let backup_content = std::fs::read_to_string(&backup_path).unwrap();
        assert!(backup_content.contains("0.1.0"));
    }

    /// 测试版本边界条件
    #[test]
    fn test_version_edge_cases() {
        // 预发布版本 - 这些版本格式包含非数字字符，应该返回错误
        // 根据 SemVer，1.0.0-alpha < 1.0.0，但我们的简单比较函数不支持这种格式
        // 所以我们测试函数能正确处理这些错误情况
        assert!(UpdateManager::compare_versions("1.0.0-alpha", "1.0.0").is_err());
        assert!(UpdateManager::compare_versions("1.0.0-beta", "1.0.0-rc").is_err());

        // 构建元数据 - 同样包含非数字字符
        assert!(UpdateManager::compare_versions("1.0.0+build1", "1.0.0+build2").is_err());

        // 大版本号 - 这些应该正常工作
        assert!(UpdateManager::compare_versions("0.0.0", "999.999.999").unwrap());
    }

    /// 测试空配置迁移
    #[test]
    fn test_empty_config_migration() {
        let mut empty_config = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());

        // 应该能处理空配置
        assert!(UpdateManager::migrate_to_v0_1_0(&mut empty_config).is_ok());
        assert!(UpdateManager::migrate_to_v0_2_0(&mut empty_config).is_ok());
        assert!(UpdateManager::migrate_to_v0_3_0(&mut empty_config).is_ok());
    }

    /// 测试 YAML 值类型保留
    #[test]
    fn test_yaml_type_preservation() {
        let mut config = serde_yaml::Mapping::new();

        // 添加各种类型的值
        config.insert(
            serde_yaml::Value::String("string".to_string()),
            serde_yaml::Value::String("test".to_string()),
        );
        config.insert(
            serde_yaml::Value::String("number".to_string()),
            serde_yaml::Value::Number(42.into()),
        );
        config.insert(
            serde_yaml::Value::String("bool".to_string()),
            serde_yaml::Value::Bool(true),
        );
        config.insert(
            serde_yaml::Value::String("null".to_string()),
            serde_yaml::Value::Null,
        );

        let mut config_value = serde_yaml::Value::Mapping(config);

        // 执行迁移
        let _ = UpdateManager::migrate_to_v0_1_0(&mut config_value);

        // 验证类型被保留
        if let serde_yaml::Value::Mapping(map) = config_value {
            assert!(matches!(map.get(&serde_yaml::Value::String("string".to_string())).unwrap(), serde_yaml::Value::String(_)));
            assert!(matches!(map.get(&serde_yaml::Value::String("number".to_string())).unwrap(), serde_yaml::Value::Number(_)));
            assert!(matches!(map.get(&serde_yaml::Value::String("bool".to_string())).unwrap(), serde_yaml::Value::Bool(_)));
            assert!(matches!(map.get(&serde_yaml::Value::String("null".to_string())).unwrap(), serde_yaml::Value::Null));
        }
    }
}
