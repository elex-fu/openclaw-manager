//! 安全存储模块
//!
//! 使用系统密钥链安全存储敏感信息（API Key 等）

#![allow(dead_code)]

use crate::errors::{AppError, SecureStorageError};
use keyring::Entry;
use log::info;

const SERVICE_NAME: &str = "com.openclaw.manager";

/// 安全存储管理器
pub struct SecureStorage;

impl SecureStorage {
    /// 创建密钥条目键名
    fn create_key(provider: &str) -> String {
        format!("api_key_{}", provider)
    }

    /// 保存 API Key
    ///
    /// # Arguments
    /// * `provider` - 模型提供商名称 (如 "openai", "anthropic")
    /// * `api_key` - API 密钥
    ///
    /// # Example
    /// ```rust,ignore
    /// use openclaw_manager::services::secure_storage::SecureStorage;
    /// SecureStorage::save_api_key("openai", "sk-...").unwrap();
    /// ```
    pub fn save_api_key(provider: &str, api_key: &str) -> Result<(), AppError> {
        let key = Self::create_key(provider);
        let entry = Entry::new(SERVICE_NAME, &key)
            .map_err(|e| SecureStorageError::Keyring(e.to_string()))?;
        
        entry.set_password(api_key)
            .map_err(|e| SecureStorageError::Keyring(e.to_string()))?;
        
        info!("API key saved for provider: {}", provider);
        Ok(())
    }

    /// 获取 API Key
    ///
    /// # Arguments
    /// * `provider` - 模型提供商名称
    ///
    /// # Returns
    /// * `Some(api_key)` - 如果存在
    /// * `None` - 如果不存在
    pub fn get_api_key(provider: &str) -> Result<Option<String>, AppError> {
        let key = Self::create_key(provider);
        let entry = Entry::new(SERVICE_NAME, &key)
            .map_err(|e| SecureStorageError::Keyring(e.to_string()))?;

        match entry.get_password() {
            Ok(api_key) => {
                info!("API key retrieved for provider: {}", provider);
                Ok(Some(api_key))
            }
            Err(keyring::Error::NoEntry) => {
                info!("No API key found for provider: {}", provider);
                Ok(None)
            }
            Err(e) => Err(SecureStorageError::Keyring(e.to_string()).into()),
        }
    }

    /// 删除 API Key
    ///
    /// # Arguments
    /// * `provider` - 模型提供商名称
    pub fn delete_api_key(provider: &str) -> Result<(), AppError> {
        let key = Self::create_key(provider);
        let entry = Entry::new(SERVICE_NAME, &key)
            .map_err(|e| SecureStorageError::Keyring(e.to_string()))?;

        entry.delete_password()
            .map_err(|e| match e {
                keyring::Error::NoEntry => SecureStorageError::ProviderNotFound(provider.to_string()),
                _ => SecureStorageError::Keyring(e.to_string()),
            })?;

        info!("API key deleted for provider: {}", provider);
        Ok(())
    }

    /// 检查是否存在 API Key
    ///
    /// # Arguments
    /// * `provider` - 模型提供商名称
    pub fn has_api_key(provider: &str) -> Result<bool, AppError> {
        match Self::get_api_key(provider) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// 批量保存 API Keys
    ///
    /// # Arguments
    /// * `keys` - 提供商和密钥的元组列表
    pub fn save_api_keys(keys: &[(&str, &str)]) -> Result<(), AppError> {
        for (provider, api_key) in keys {
            Self::save_api_key(provider, api_key)?;
        }
        Ok(())
    }

    /// 获取所有已存储的提供商列表
    ///
    /// 注意：keyring crate 不支持列出条目，这个功能需要额外存储
    /// 返回一个常见提供商列表供前端检查
    pub fn get_known_providers() -> Vec<&'static str> {
        vec![
            "openai",
            "anthropic",
            "google",
            "azure",
            "cohere",
            "mistral",
            "deepseek",
        ]
    }

    /// 测试密钥存储是否可用
    pub fn test_storage() -> Result<(), AppError> {
        let test_provider = "__test_provider__";
        let test_key = "test_key_12345";

        // 尝试保存测试密钥
        Self::save_api_key(test_provider, test_key)?;

        // 尝试读取
        let retrieved = Self::get_api_key(test_provider)?;
        
        // 清理测试密钥
        let _ = Self::delete_api_key(test_provider);

        match retrieved {
            Some(key) if key == test_key => {
                info!("Secure storage test passed");
                Ok(())
            }
            _ => Err(SecureStorageError::Keyring("Test key mismatch".to_string()).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_key() {
        assert_eq!(SecureStorage::create_key("openai"), "api_key_openai");
        assert_eq!(SecureStorage::create_key("anthropic"), "api_key_anthropic");
    }

    // 注意：实际的密钥操作测试需要在有密钥链访问权限的环境中运行
    // 这些测试在 CI 环境中可能会失败
    #[test]
    #[ignore]
    fn test_save_and_get_key() {
        let provider = "__test_openai__";
        let key = "sk-test12345";

        // 保存
        SecureStorage::save_api_key(provider, key).unwrap();

        // 读取
        let retrieved = SecureStorage::get_api_key(provider).unwrap();
        assert_eq!(retrieved, Some(key.to_string()));

        // 检查存在
        assert!(SecureStorage::has_api_key(provider).unwrap());

        // 删除
        SecureStorage::delete_api_key(provider).unwrap();

        // 确认删除
        assert!(!SecureStorage::has_api_key(provider).unwrap());
    }

    #[test]
    fn test_get_known_providers() {
        let providers = SecureStorage::get_known_providers();
        assert!(providers.contains(&"openai"));
        assert!(providers.contains(&"anthropic"));
    }
}
