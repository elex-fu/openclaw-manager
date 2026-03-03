//! 安全存储集成测试

/// 测试安全存储基本操作
#[test]
fn test_secure_storage_basic() {
    use openclaw_manager::services::secure_storage::SecureStorage;

    let test_provider = "__test_integration__";
    let test_key = "sk-test12345";

    // 保存 API Key
    let save_result = SecureStorage::save_api_key(test_provider, test_key);

    // 如果密钥链可用，测试完整流程
    if save_result.is_ok() {
        // 检查是否存在
        let has_result = SecureStorage::has_api_key(test_provider);
        assert!(has_result.is_ok());
        assert!(has_result.unwrap());

        // 读取 API Key
        let get_result = SecureStorage::get_api_key(test_provider);
        assert!(get_result.is_ok());
        assert_eq!(get_result.unwrap(), Some(test_key.to_string()));

        // 删除 API Key
        let delete_result = SecureStorage::delete_api_key(test_provider);
        assert!(delete_result.is_ok());

        // 确认已删除
        let has_result = SecureStorage::has_api_key(test_provider);
        assert!(has_result.is_ok());
        assert!(!has_result.unwrap());
    } else {
        println!("密钥链不可用，跳过测试");
    }
}

/// 测试获取不存在的 API Key
#[test]
fn test_get_nonexistent_key() {
    use openclaw_manager::services::secure_storage::SecureStorage;

    let result = SecureStorage::get_api_key("__nonexistent_provider_12345__");

    if result.is_ok() {
        assert_eq!(result.unwrap(), None);
    }
}

/// 测试多个提供商的 API Key 独立存储
#[test]
fn test_multiple_providers() {
    use openclaw_manager::services::secure_storage::SecureStorage;

    let providers = vec![
        ("__test_openai__", "sk-openai123"),
        ("__test_anthropic__", "sk-anthropic456"),
    ];

    // 保存所有密钥
    let mut all_saved = true;
    for (provider, key) in &providers {
        if SecureStorage::save_api_key(provider, key).is_err() {
            all_saved = false;
            break;
        }
    }

    if all_saved {
        // 验证每个密钥都能正确读取
        for (provider, expected_key) in &providers {
            let retrieved = SecureStorage::get_api_key(provider).unwrap();
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap(), *expected_key);
        }

        // 清理
        for (provider, _) in &providers {
            let _ = SecureStorage::delete_api_key(provider);
        }
    }
}

/// 测试特殊字符的 API Key
#[test]
fn test_special_characters_key() {
    use openclaw_manager::services::secure_storage::SecureStorage;

    let test_provider = "__test_special__";
    // 包含各种特殊字符的 API Key
    let test_key = "sk-test+special=chars&symbols!@#$%^&*()_+-=[]{}|;':\",./<>?";

    if SecureStorage::save_api_key(test_provider, test_key).is_ok() {
        let retrieved = SecureStorage::get_api_key(test_provider).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), test_key);

        // 清理
        let _ = SecureStorage::delete_api_key(test_provider);
    }
}

/// 测试很长的 API Key
#[test]
fn test_long_api_key() {
    use openclaw_manager::services::secure_storage::SecureStorage;

    let test_provider = "__test_long__";
    let test_key = "sk-".to_string() + &"a".repeat(2000);

    if SecureStorage::save_api_key(test_provider, &test_key).is_ok() {
        let retrieved = SecureStorage::get_api_key(test_provider).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), test_key);

        // 清理
        let _ = SecureStorage::delete_api_key(test_provider);
    }
}

/// 测试已知提供商列表
#[test]
fn test_known_providers() {
    use openclaw_manager::services::secure_storage::SecureStorage;

    let providers = SecureStorage::get_known_providers();
    assert!(!providers.is_empty());
    assert!(providers.contains(&"openai"));
    assert!(providers.contains(&"anthropic"));
    assert!(providers.contains(&"google"));
}

/// 测试批量保存 API Keys
#[test]
fn test_save_api_keys_batch() {
    use openclaw_manager::services::secure_storage::SecureStorage;

    let keys = vec![
        ("__test_batch1__", "key1"),
        ("__test_batch2__", "key2"),
    ];

    let result = SecureStorage::save_api_keys(&keys);

    if result.is_ok() {
        // 验证所有密钥都已保存
        for (provider, expected_key) in &keys {
            let retrieved = SecureStorage::get_api_key(provider).unwrap();
            assert_eq!(retrieved, Some(expected_key.to_string()));
        }

        // 清理
        for (provider, _) in &keys {
            let _ = SecureStorage::delete_api_key(provider);
        }
    }
}
