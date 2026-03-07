//! 安全存储单元测试
//!
//! 测试 API Key 保存、读取、删除等功能
//! 覆盖系统密钥链和加密文件存储两种后端

use openclaw_manager::services::secure_storage::{SecureStorage, StorageBackend};

/// 测试 API Key 保存和读取
#[test]
fn test_save_and_get_api_key() {
    let test_provider = "test_provider_openclaw";
    let test_key = "sk-test123456789";

    // 使用加密文件后端进行测试（更可靠）
    let storage = SecureStorage::with_backend(StorageBackend::EncryptedFile);

    // 清理可能存在的旧数据
    let _ = storage.delete_api_key(test_provider);

    // 保存 API Key
    let save_result = storage.save_api_key(test_provider, test_key);
    assert!(save_result.is_ok(), "保存 API Key 应该成功");

    // 读取 API Key
    let get_result = storage.get_api_key(test_provider);
    assert!(get_result.is_ok(), "应该能读取 API Key");

    let retrieved_key = get_result.unwrap();
    assert!(retrieved_key.is_some(), "应该能获取到 API Key");
    assert_eq!(retrieved_key.unwrap(), test_key, "读取的 API Key 应该与保存的一致");

    // 清理测试数据
    let _ = storage.delete_api_key(test_provider);
}

/// 测试检查 API Key 是否存在
#[test]
fn test_has_api_key() {
    let test_provider = "test_has_key_provider";
    let test_key = "sk-test";

    let storage = SecureStorage::with_backend(StorageBackend::EncryptedFile);

    // 清理
    let _ = storage.delete_api_key(test_provider);

    // 初始状态应该不存在
    let has_key_initial = storage.has_api_key(test_provider);
    assert!(has_key_initial.is_ok());
    assert_eq!(has_key_initial.unwrap(), false, "初始状态应该不存在 API Key");

    // 保存 API Key
    let save_result = storage.save_api_key(test_provider, test_key);
    assert!(save_result.is_ok(), "保存应该成功");

    // 现在应该存在
    let has_key_after = storage.has_api_key(test_provider);
    assert!(has_key_after.is_ok());
    assert_eq!(has_key_after.unwrap(), true, "保存后应该存在 API Key");

    // 清理
    let _ = storage.delete_api_key(test_provider);
}

/// 测试删除 API Key
#[test]
fn test_delete_api_key() {
    let test_provider = "test_delete_provider";
    let test_key = "sk-test";

    let storage = SecureStorage::with_backend(StorageBackend::EncryptedFile);

    // 先保存
    let save_result = storage.save_api_key(test_provider, test_key);
    assert!(save_result.is_ok(), "保存应该成功");

    // 确认存在
    assert!(storage.has_api_key(test_provider).unwrap_or(false));

    // 删除
    let delete_result = storage.delete_api_key(test_provider);
    assert!(delete_result.is_ok(), "应该能删除 API Key");

    // 确认不存在
    let has_key = storage.has_api_key(test_provider);
    if has_key.is_ok() {
        assert_eq!(has_key.unwrap(), false, "删除后应该不存在 API Key");
    }
}

/// 测试获取不存在的 API Key
#[test]
fn test_get_nonexistent_api_key() {
    let test_provider = "nonexistent_provider_12345";

    let storage = SecureStorage::with_backend(StorageBackend::EncryptedFile);

    let result = storage.get_api_key(test_provider);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none(), "不存在的 API Key 应该返回 None");
}

/// 测试多个提供商的 API Key 独立存储
#[test]
fn test_multiple_providers() {
    let storage = SecureStorage::with_backend(StorageBackend::EncryptedFile);

    let providers = vec![
        ("test_openai", "sk-openai123"),
        ("test_anthropic", "sk-anthropic456"),
        ("test_deepseek", "sk-deepseek789"),
    ];

    // 清理并保存所有密钥
    for (provider, key) in &providers {
        let _ = storage.delete_api_key(provider);
        let result = storage.save_api_key(provider, key);
        assert!(result.is_ok(), "保存 {} 应该成功", provider);
    }

    // 验证每个密钥都能正确读取
    for (provider, expected_key) in &providers {
        let retrieved = storage.get_api_key(provider).unwrap();
        assert!(retrieved.is_some(), "应该能获取到 {}", provider);
        assert_eq!(retrieved.unwrap(), *expected_key);
    }

    // 清理
    for (provider, _) in &providers {
        let _ = storage.delete_api_key(provider);
    }
}

/// 测试特殊字符的 API Key
#[test]
fn test_special_characters_api_key() {
    let storage = SecureStorage::with_backend(StorageBackend::EncryptedFile);
    let test_provider = "test_special_chars";

    // 清理
    let _ = storage.delete_api_key(test_provider);

    // 包含各种特殊字符的 API Key
    let test_key = "sk-test+special=chars&symbols!@#$%^*()_+-=[]{}|;':\",./<>?";

    let save_result = storage.save_api_key(test_provider, test_key);
    assert!(save_result.is_ok(), "保存特殊字符 API Key 应该成功");

    let retrieved = storage.get_api_key(test_provider).unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), test_key);

    // 清理
    let _ = storage.delete_api_key(test_provider);
}

/// 测试很长的 API Key
#[test]
fn test_long_api_key() {
    let storage = SecureStorage::with_backend(StorageBackend::EncryptedFile);
    let test_provider = "test_long_key";

    // 清理
    let _ = storage.delete_api_key(test_provider);

    let test_key = "sk-".to_string() + &"a".repeat(2000);

    let save_result = storage.save_api_key(test_provider, &test_key);
    assert!(save_result.is_ok(), "保存长 API Key 应该成功");

    let retrieved = storage.get_api_key(test_provider).unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), test_key);

    // 清理
    let _ = storage.delete_api_key(test_provider);
}

/// 测试加密文件存储后端（MVP v2 新增）
#[test]
fn test_encrypted_file_backend() {
    let storage = SecureStorage::with_backend(StorageBackend::EncryptedFile);
    let test_provider = "test_file_backend";
    let test_key = "sk-encrypted-file-test";

    // 清理
    let _ = storage.delete_api_key(test_provider);

    // 测试保存
    let save_result = storage.save_api_key(test_provider, test_key);
    assert!(save_result.is_ok(), "加密文件存储保存应该成功");

    // 测试读取
    let retrieved = storage.get_api_key(test_provider).unwrap();
    assert_eq!(retrieved, Some(test_key.to_string()));

    // 测试存在检查
    assert!(storage.has_api_key(test_provider).unwrap());

    // 测试删除
    let delete_result = storage.delete_api_key(test_provider);
    assert!(delete_result.is_ok());
    assert!(!storage.has_api_key(test_provider).unwrap());
}

/// 测试存储后端自动切换（MVP v2 新增）
#[test]
fn test_storage_backend_auto_selection() {
    // 测试自动选择功能
    let result = SecureStorage::auto();

    // 应该能成功创建（要么用 keyring，要么用加密文件）
    assert!(result.is_ok(), "自动选择存储后端应该成功");

    let storage = result.unwrap();
    let backend = storage.backend();

    // 后端应该是两种之一
    assert!(
        backend == StorageBackend::Keychain || backend == StorageBackend::EncryptedFile,
        "后端应该是 Keychain 或 EncryptedFile"
    );

    // 测试基本功能
    let test_provider = "test_auto_backend";
    let test_key = "sk-auto-test";

    let _ = storage.delete_api_key(test_provider);

    let save_result = storage.save_api_key(test_provider, test_key);
    assert!(save_result.is_ok(), "自动选择的后端应该能保存");

    let retrieved = storage.get_api_key(test_provider).unwrap();
    assert_eq!(retrieved, Some(test_key.to_string()));

    let _ = storage.delete_api_key(test_provider);
}

/// 测试加密文件存储的持久性（MVP v2 新增）
#[test]
fn test_encrypted_file_persistence() {
    let test_provider = "test_persistence";
    let test_key = "sk-persistent-key";

    // 第一个存储实例保存密钥
    {
        let storage = SecureStorage::with_backend(StorageBackend::EncryptedFile);
        let _ = storage.delete_api_key(test_provider);
        let save_result = storage.save_api_key(test_provider, test_key);
        assert!(save_result.is_ok());
    }

    // 第二个存储实例读取密钥（验证持久性）
    {
        let storage = SecureStorage::with_backend(StorageBackend::EncryptedFile);
        let retrieved = storage.get_api_key(test_provider).unwrap();
        assert_eq!(retrieved, Some(test_key.to_string()), "应该能从文件读取持久化的密钥");

        // 清理
        let _ = storage.delete_api_key(test_provider);
    }
}

/// 测试批量保存 API Keys（MVP v2 新增）
#[test]
fn test_batch_save_api_keys() {
    let storage = SecureStorage::with_backend(StorageBackend::EncryptedFile);

    let keys = [
        ("__test_batch1__", "key1"),
        ("__test_batch2__", "key2"),
        ("__test_batch3__", "key3"),
    ];

    // 清理
    for (provider, _) in &keys {
        let _ = storage.delete_api_key(provider);
    }

    // 批量保存
    let result = storage.save_api_keys(&keys);
    assert!(result.is_ok());

    // 验证
    for (provider, expected_key) in &keys {
        let retrieved = storage.get_api_key(provider).unwrap();
        assert_eq!(retrieved, Some(expected_key.to_string()));
    }

    // 清理
    for (provider, _) in &keys {
        let _ = storage.delete_api_key(provider);
    }
}

/// 测试存储测试功能（MVP v2 新增）
#[test]
fn test_storage_test_function() {
    let storage = SecureStorage::with_backend(StorageBackend::EncryptedFile);

    let result = storage.test_storage();
    assert!(result.is_ok(), "存储测试应该通过");
}

/// 测试已知提供商列表
#[test]
fn test_get_known_providers() {
    let providers = SecureStorage::get_known_providers();

    assert!(providers.contains(&"openai"));
    assert!(providers.contains(&"anthropic"));
    assert!(providers.contains(&"google"));
    assert!(providers.contains(&"azure"));
    assert!(providers.contains(&"deepseek"));
}

/// 测试存储后端枚举
#[test]
fn test_storage_backend_enum() {
    assert_ne!(StorageBackend::Keychain, StorageBackend::EncryptedFile);

    // 验证克隆
    let backend1 = StorageBackend::EncryptedFile;
    let backend2 = backend1.clone();
    assert_eq!(backend1, backend2);
}

// 注意：以下测试需要在有密钥链访问权限的环境中运行
// 在 CI 环境中可能会失败，因此标记为 ignore
#[test]
#[ignore]
fn test_keyring_backend() {
    let storage = SecureStorage::with_backend(StorageBackend::Keychain);
    let test_provider = "__test_keyring__";
    let test_key = "sk-keyring-test";

    // 保存
    let save_result = storage.save_api_key(test_provider, test_key);
    assert!(save_result.is_ok());

    // 读取
    let retrieved = storage.get_api_key(test_provider).unwrap();
    assert_eq!(retrieved, Some(test_key.to_string()));

    // 检查存在
    assert!(storage.has_api_key(test_provider).unwrap());

    // 删除
    let delete_result = storage.delete_api_key(test_provider);
    assert!(delete_result.is_ok());

    // 确认删除
    assert!(!storage.has_api_key(test_provider).unwrap());
}
