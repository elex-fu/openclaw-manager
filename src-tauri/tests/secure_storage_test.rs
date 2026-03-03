//! 安全存储单元测试
//!
//! 注意：这些测试需要在有密钥链访问权限的环境中运行

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 API Key 保存和读取
    #[test]
    fn test_save_and_get_api_key() {
        let test_provider = "test_provider_openclaw";
        let test_key = "sk-test123456789";

        // 保存 API Key
        let save_result = SecureStorage::save_api_key(test_provider, test_key);

        // 如果密钥链可用，测试应该成功
        if save_result.is_ok() {
            // 读取 API Key
            let get_result = SecureStorage::get_api_key(test_provider);
            assert!(get_result.is_ok(), "应该能读取 API Key");

            let retrieved_key = get_result.unwrap();
            assert!(retrieved_key.is_some(), "应该能获取到 API Key");
            assert_eq!(retrieved_key.unwrap(), test_key, "读取的 API Key 应该与保存的一致");

            // 清理测试数据
            let _ = SecureStorage::delete_api_key(test_provider);
        } else {
            println!("跳过测试：密钥链不可用");
        }
    }

    /// 测试检查 API Key 是否存在
    #[test]
    fn test_has_api_key() {
        let test_provider = "test_has_key_provider";
        let test_key = "sk-test";

        // 初始状态应该不存在
        let has_key_initial = SecureStorage::has_api_key(test_provider);
        if has_key_initial.is_ok() {
            assert_eq!(has_key_initial.unwrap(), false, "初始状态应该不存在 API Key");

            // 保存 API Key
            if SecureStorage::save_api_key(test_provider, test_key).is_ok() {
                // 现在应该存在
                let has_key_after = SecureStorage::has_api_key(test_provider);
                assert!(has_key_after.is_ok());
                assert_eq!(has_key_after.unwrap(), true, "保存后应该存在 API Key");

                // 清理
                let _ = SecureStorage::delete_api_key(test_provider);
            }
        } else {
            println!("跳过测试：密钥链不可用");
        }
    }

    /// 测试删除 API Key
    #[test]
    fn test_delete_api_key() {
        let test_provider = "test_delete_provider";
        let test_key = "sk-test";

        // 先保存
        if SecureStorage::save_api_key(test_provider, test_key).is_ok() {
            // 确认存在
            assert!(SecureStorage::has_api_key(test_provider).unwrap_or(false));

            // 删除
            let delete_result = SecureStorage::delete_api_key(test_provider);
            assert!(delete_result.is_ok(), "应该能删除 API Key");

            // 确认不存在
            let has_key = SecureStorage::has_api_key(test_provider);
            if has_key.is_ok() {
                assert_eq!(has_key.unwrap(), false, "删除后应该不存在 API Key");
            }
        } else {
            println!("跳过测试：密钥链不可用");
        }
    }

    /// 测试获取不存在的 API Key
    #[test]
    fn test_get_nonexistent_api_key() {
        let test_provider = "nonexistent_provider_12345";

        let result = SecureStorage::get_api_key(test_provider);
        if result.is_ok() {
            assert!(result.unwrap().is_none(), "不存在的 API Key 应该返回 None");
        } else {
            println!("跳过测试：密钥链不可用");
        }
    }

    /// 测试多个提供商的 API Key 独立存储
    #[test]
    fn test_multiple_providers() {
        let providers = vec![
            ("test_openai", "sk-openai123"),
            ("test_anthropic", "sk-anthropic456"),
            ("test_deepseek", "sk-deepseek789"),
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
        } else {
            println!("跳过测试：密钥链不可用");
        }
    }

    /// 测试特殊字符的 API Key
    #[test]
    fn test_special_characters_api_key() {
        let test_provider = "test_special_chars";
        // 包含各种特殊字符的 API Key
        let test_key = "sk-test+special=chars&symbols!@#$%^&*()_+-=[]{}|;':\",./<>?";

        if SecureStorage::save_api_key(test_provider, test_key).is_ok() {
            let retrieved = SecureStorage::get_api_key(test_provider).unwrap();
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap(), test_key);

            // 清理
            let _ = SecureStorage::delete_api_key(test_provider);
        } else {
            println!("跳过测试：密钥链不可用");
        }
    }

    /// 测试很长的 API Key
    #[test]
    fn test_long_api_key() {
        let test_provider = "test_long_key";
        let test_key = "sk-".to_string() + &"a".repeat(2000);

        if SecureStorage::save_api_key(test_provider, &test_key).is_ok() {
            let retrieved = SecureStorage::get_api_key(test_provider).unwrap();
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap(), test_key);

            // 清理
            let _ = SecureStorage::delete_api_key(test_provider);
        } else {
            println!("跳过测试：密钥链不可用");
        }
    }
}
