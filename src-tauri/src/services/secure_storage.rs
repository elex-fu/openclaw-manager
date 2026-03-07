//! 安全存储模块
//!
//! 使用系统密钥链安全存储敏感信息（API Key 等）
//! 在 Linux 环境下 keyring 不可用时自动降级到加密文件存储

#![allow(dead_code)]

use crate::errors::{AppError, SecureStorageError};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use keyring::Entry;
use log::{info, warn};
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

const SERVICE_NAME: &str = "com.openclaw.manager";
const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;

/// 存储后端类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StorageBackend {
    /// 系统密钥链
    Keychain,
    /// 加密文件存储
    EncryptedFile,
}

/// 安全存储管理器
pub struct SecureStorage {
    backend: StorageBackend,
}

impl SecureStorage {
    /// 自动选择最佳存储后端
    ///
    /// 优先尝试 keyring，如果不可用则降级到加密文件存储
    pub fn auto() -> Result<Self, AppError> {
        // 首先尝试 keyring
        match Self::test_keyring() {
            Ok(()) => {
                info!("Using keyring backend for secure storage");
                Ok(Self {
                    backend: StorageBackend::Keychain,
                })
            }
            Err(e) => {
                warn!("Keyring unavailable, falling back to encrypted file storage: {}", e);
                // 验证加密文件存储是否可用
                Self::test_encrypted_file_storage()?;
                info!("Using encrypted file backend for secure storage");
                Ok(Self {
                    backend: StorageBackend::EncryptedFile,
                })
            }
        }
    }

    /// 使用指定的存储后端创建实例
    pub fn with_backend(backend: StorageBackend) -> Self {
        Self { backend }
    }

    /// 获取当前使用的存储后端
    pub fn backend(&self) -> StorageBackend {
        self.backend
    }

    /// 测试 keyring 是否可用
    fn test_keyring() -> Result<(), AppError> {
        let test_provider = "__backend_test__";
        let test_key = "test_key_12345";

        let key = Self::create_key(test_provider);
        let entry = Entry::new(SERVICE_NAME, &key)
            .map_err(|e| SecureStorageError::Keyring(e.to_string()))?;

        // 尝试保存测试密钥
        match entry.set_password(test_key) {
            Ok(()) => {
                // 尝试读取
                let retrieved = entry.get_password();
                // 清理测试密钥
                let _ = entry.delete_password();

                match retrieved {
                    Ok(key) if key == test_key => Ok(()),
                    _ => Err(SecureStorageError::Keyring("Test key mismatch".to_string()).into()),
                }
            }
            Err(e) => Err(SecureStorageError::Keyring(e.to_string()).into()),
        }
    }

    /// 测试加密文件存储是否可用
    fn test_encrypted_file_storage() -> Result<(), AppError> {
        // 检查是否可以获取机器 ID
        let _ = Self::get_machine_id()?;

        // 检查配置目录是否可写
        let config_dir = Self::get_config_dir()?;
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| SecureStorageError::EncryptionFailed(format!("Cannot create config dir: {}", e)))?;
        }

        // 测试写入权限
        let test_file = config_dir.join(".write_test");
        match fs::write(&test_file, b"test") {
            Ok(()) => {
                let _ = fs::remove_file(&test_file);
                Ok(())
            }
            Err(e) => Err(SecureStorageError::EncryptionFailed(format!("Config directory not writable: {}", e)).into()),
        }
    }

    /// 创建密钥条目键名
    fn create_key(provider: &str) -> String {
        format!("api_key_{}", provider)
    }

    /// 获取配置目录路径
    fn get_config_dir() -> Result<PathBuf, SecureStorageError> {
        dirs::config_dir()
            .map(|dir| dir.join("openclaw-manager"))
            .ok_or_else(|| SecureStorageError::EncryptionFailed("Cannot determine config directory".to_string()))
    }

    /// 获取加密文件路径
    fn get_encrypted_file_path(provider: &str) -> Result<PathBuf, SecureStorageError> {
        let config_dir = Self::get_config_dir()?;
        Ok(config_dir.join(format!("{}.enc", provider)))
    }

    /// 获取机器 ID
    ///
    /// 首先尝试读取 /etc/machine-id，如果不存在则使用 username + hostname 的组合
    fn get_machine_id() -> Result<String, SecureStorageError> {
        // 尝试读取 /etc/machine-id (systemd-based Linux)
        if let Ok(machine_id) = fs::read_to_string("/etc/machine-id") {
            let id = machine_id.trim();
            if !id.is_empty() {
                return Ok(id.to_string());
            }
        }

        // 尝试读取 /var/lib/dbus/machine-id (fallback)
        if let Ok(machine_id) = fs::read_to_string("/var/lib/dbus/machine-id") {
            let id = machine_id.trim();
            if !id.is_empty() {
                return Ok(id.to_string());
            }
        }

        // 使用 username + hostname 作为 fallback
        let username = whoami::username();
        let hostname = hostname::get()
            .map_err(|e| SecureStorageError::EncryptionFailed(format!("Cannot get hostname: {:?}", e)))?
            .to_string_lossy()
            .to_string();

        if username.is_empty() && hostname.is_empty() {
            return Err(SecureStorageError::EncryptionFailed(
                "Cannot determine machine ID".to_string(),
            ));
        }

        // 使用 SHA-256 生成固定长度的 ID
        use sha2::{Digest, Sha256};
        let combined = format!("{}@{}", username, hostname);
        let hash = Sha256::digest(combined.as_bytes());
        Ok(hex::encode(hash))
    }

    /// 派生加密密钥
    ///
    /// 使用机器 ID 和固定 salt 派生 256 位密钥
    fn derive_key(machine_id: &str) -> [u8; KEY_SIZE] {
        use sha2::{Digest, Sha256};

        // 使用固定 salt 增加安全性
        const SALT: &str = "OpenClawManager_v1.0";
        let combined = format!("{}{}", machine_id, SALT);
        let hash = Sha256::digest(combined.as_bytes());

        let mut key = [0u8; KEY_SIZE];
        key.copy_from_slice(&hash);
        key
    }

    /// 生成随机 nonce
    fn generate_nonce() -> [u8; NONCE_SIZE] {
        use rand::Rng;
        let mut nonce = [0u8; NONCE_SIZE];
        rand::thread_rng().fill(&mut nonce);
        nonce
    }

    /// 保存到加密文件
    fn save_to_encrypted_file(provider: &str, api_key: &str) -> Result<(), AppError> {
        let machine_id = Self::get_machine_id()?;
        let key = Self::derive_key(&machine_id);
        let nonce = Self::generate_nonce();

        // 创建 cipher
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| SecureStorageError::EncryptionFailed(format!("Key init failed: {}", e)))?;

        // 加密数据
        let nonce_obj = Nonce::from_slice(&nonce);
        let encrypted = cipher
            .encrypt(nonce_obj, api_key.as_bytes())
            .map_err(|e| SecureStorageError::EncryptionFailed(format!("Encryption failed: {}", e)))?;

        // 组合 nonce 和加密数据
        let mut data = Vec::with_capacity(NONCE_SIZE + encrypted.len());
        data.extend_from_slice(&nonce);
        data.extend_from_slice(&encrypted);

        // 确保配置目录存在
        let config_dir = Self::get_config_dir()?;
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| SecureStorageError::EncryptionFailed(format!("Create dir failed: {}", e)))?;
        }

        // 写入文件
        let file_path = Self::get_encrypted_file_path(provider)?;
        let mut file = fs::File::create(&file_path)
            .map_err(|e| SecureStorageError::EncryptionFailed(format!("Create file failed: {}", e)))?;

        // 设置文件权限为 0o600 (仅所有者可读写)
        #[cfg(unix)]
        {
            let permissions = std::fs::Permissions::from_mode(0o600);
            file.set_permissions(permissions)
                .map_err(|e| SecureStorageError::EncryptionFailed(format!("Set permissions failed: {}", e)))?;
        }

        file.write_all(&data)
            .map_err(|e| SecureStorageError::EncryptionFailed(format!("Write file failed: {}", e)))?;

        info!("API key saved to encrypted file for provider: {}", provider);
        Ok(())
    }

    /// 从加密文件读取
    fn read_from_encrypted_file(provider: &str) -> Result<Option<String>, AppError> {
        let file_path = Self::get_encrypted_file_path(provider)?;

        // 检查文件是否存在
        if !file_path.exists() {
            return Ok(None);
        }

        // 读取文件内容
        let data = fs::read(&file_path)
            .map_err(|e| SecureStorageError::DecryptionFailed(format!("Read file failed: {}", e)))?;

        // 检查数据长度
        if data.len() < NONCE_SIZE {
            return Err(SecureStorageError::DecryptionFailed("Invalid file format".to_string()).into());
        }

        // 分离 nonce 和加密数据
        let nonce = &data[..NONCE_SIZE];
        let encrypted = &data[NONCE_SIZE..];

        // 派生密钥
        let machine_id = Self::get_machine_id()?;
        let key = Self::derive_key(&machine_id);

        // 创建 cipher
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| SecureStorageError::DecryptionFailed(format!("Key init failed: {}", e)))?;

        // 解密数据
        let nonce_obj = Nonce::from_slice(nonce);
        let decrypted = cipher
            .decrypt(nonce_obj, encrypted)
            .map_err(|e| SecureStorageError::DecryptionFailed(format!("Decryption failed: {}", e)))?;

        // 转换为字符串
        let api_key = String::from_utf8(decrypted)
            .map_err(|e| SecureStorageError::DecryptionFailed(format!("Invalid UTF-8: {}", e)))?;

        info!("API key read from encrypted file for provider: {}", provider);
        Ok(Some(api_key))
    }

    /// 删除加密文件
    fn delete_encrypted_file(provider: &str) -> Result<(), AppError> {
        let file_path = Self::get_encrypted_file_path(provider)?;

        if file_path.exists() {
            fs::remove_file(&file_path)
                .map_err(|e| SecureStorageError::EncryptionFailed(format!("Delete file failed: {}", e)))?;
            info!("Encrypted file deleted for provider: {}", provider);
        }

        Ok(())
    }

    /// 保存 API Key
    ///
    /// # Arguments
    /// * `provider` - 模型提供商名称 (如 "openai", "anthropic")
    /// * `api_key` - API 密钥
    pub fn save_api_key(&self, provider: &str, api_key: &str) -> Result<(), AppError> {
        match self.backend {
            StorageBackend::Keychain => {
                let key = Self::create_key(provider);
                let entry = Entry::new(SERVICE_NAME, &key)
                    .map_err(|e| SecureStorageError::Keyring(e.to_string()))?;

                entry.set_password(api_key)
                    .map_err(|e| SecureStorageError::Keyring(e.to_string()))?;

                info!("API key saved to keyring for provider: {}", provider);
                Ok(())
            }
            StorageBackend::EncryptedFile => Self::save_to_encrypted_file(provider, api_key),
        }
    }

    /// 获取 API Key
    ///
    /// # Arguments
    /// * `provider` - 模型提供商名称
    ///
    /// # Returns
    /// * `Some(api_key)` - 如果存在
    /// * `None` - 如果不存在
    pub fn get_api_key(&self, provider: &str) -> Result<Option<String>, AppError> {
        match self.backend {
            StorageBackend::Keychain => {
                let key = Self::create_key(provider);
                let entry = Entry::new(SERVICE_NAME, &key)
                    .map_err(|e| SecureStorageError::Keyring(e.to_string()))?;

                match entry.get_password() {
                    Ok(api_key) => {
                        info!("API key retrieved from keyring for provider: {}", provider);
                        Ok(Some(api_key))
                    }
                    Err(keyring::Error::NoEntry) => {
                        info!("No API key found in keyring for provider: {}", provider);
                        Ok(None)
                    }
                    Err(e) => Err(SecureStorageError::Keyring(e.to_string()).into()),
                }
            }
            StorageBackend::EncryptedFile => Self::read_from_encrypted_file(provider),
        }
    }

    /// 删除 API Key
    ///
    /// # Arguments
    /// * `provider` - 模型提供商名称
    pub fn delete_api_key(&self, provider: &str) -> Result<(), AppError> {
        match self.backend {
            StorageBackend::Keychain => {
                let key = Self::create_key(provider);
                let entry = Entry::new(SERVICE_NAME, &key)
                    .map_err(|e| SecureStorageError::Keyring(e.to_string()))?;

                entry.delete_password().map_err(|e| match e {
                    keyring::Error::NoEntry => {
                        SecureStorageError::ProviderNotFound(provider.to_string())
                    }
                    _ => SecureStorageError::Keyring(e.to_string()),
                })?;

                info!("API key deleted from keyring for provider: {}", provider);
                Ok(())
            }
            StorageBackend::EncryptedFile => Self::delete_encrypted_file(provider),
        }
    }

    /// 检查是否存在 API Key
    ///
    /// # Arguments
    /// * `provider` - 模型提供商名称
    pub fn has_api_key(&self, provider: &str) -> Result<bool, AppError> {
        match self.get_api_key(provider) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// 批量保存 API Keys
    ///
    /// # Arguments
    /// * `keys` - 提供商和密钥的元组列表
    pub fn save_api_keys(&self, keys: &[(&str, &str)]) -> Result<(), AppError> {
        for (provider, api_key) in keys {
            self.save_api_key(provider, api_key)?;
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
    pub fn test_storage(&self) -> Result<(), AppError> {
        let test_provider = "__test_provider__";
        let test_key = "test_key_12345";

        // 尝试保存测试密钥
        self.save_api_key(test_provider, test_key)?;

        // 尝试读取
        let retrieved = self.get_api_key(test_provider)?;

        // 清理测试密钥
        let _ = self.delete_api_key(test_provider);

        match retrieved {
            Some(key) if key == test_key => {
                info!("Secure storage test passed");
                Ok(())
            }
            _ => Err(SecureStorageError::Keyring("Test key mismatch".to_string()).into()),
        }
    }
}

/// 全局安全存储实例（使用自动选择的后端）
static mut GLOBAL_STORAGE: Option<SecureStorage> = None;
static mut GLOBAL_STORAGE_INIT: bool = false;

impl SecureStorage {
    /// 获取全局安全存储实例
    ///
    /// 首次调用时会自动选择最佳存储后端
    ///
    /// # Safety
    /// 这个函数使用 unsafe 代码来管理全局状态。确保在单线程环境下调用，
    /// 或在多线程环境下使用适当的同步机制。
    pub fn global() -> Result<&'static SecureStorage, AppError> {
        unsafe {
            if !GLOBAL_STORAGE_INIT {
                GLOBAL_STORAGE = Some(SecureStorage::auto()?);
                GLOBAL_STORAGE_INIT = true;
            }
            GLOBAL_STORAGE.as_ref().ok_or_else(|| {
                AppError::SecureStorage(SecureStorageError::EncryptionFailed(
                    "Global storage not initialized".to_string(),
                ))
            })
        }
    }

    /// 重置全局存储实例（主要用于测试）
    #[cfg(test)]
    pub fn reset_global() {
        unsafe {
            GLOBAL_STORAGE = None;
            GLOBAL_STORAGE_INIT = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_create_key() {
        assert_eq!(SecureStorage::create_key("openai"), "api_key_openai");
        assert_eq!(SecureStorage::create_key("anthropic"), "api_key_anthropic");
    }

    #[test]
    fn test_derive_key() {
        let machine_id = "test_machine_id_123";
        let key1 = SecureStorage::derive_key(machine_id);
        let key2 = SecureStorage::derive_key(machine_id);

        // 相同 machine_id 应该产生相同的密钥
        assert_eq!(key1, key2);

        // 密钥长度应该是 32 字节 (256 位)
        assert_eq!(key1.len(), 32);

        // 不同的 machine_id 应该产生不同的密钥
        let key3 = SecureStorage::derive_key("different_machine_id");
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_generate_nonce() {
        let nonce1 = SecureStorage::generate_nonce();
        let nonce2 = SecureStorage::generate_nonce();

        // nonce 长度应该是 12 字节
        assert_eq!(nonce1.len(), 12);
        assert_eq!(nonce2.len(), 12);

        // 随机生成的 nonce 应该不同（概率极低会相同）
        // 这里我们只是检查它们不是全零
        assert!(!nonce1.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_get_machine_id() {
        // 应该能获取到机器 ID
        let machine_id = SecureStorage::get_machine_id();
        assert!(machine_id.is_ok());

        let id = machine_id.unwrap();
        assert!(!id.is_empty());

        // 多次调用应该返回相同的 ID
        let id2 = SecureStorage::get_machine_id().unwrap();
        assert_eq!(id, id2);
    }

    #[test]
    fn test_get_config_dir() {
        let config_dir = SecureStorage::get_config_dir();
        assert!(config_dir.is_ok());

        let path = config_dir.unwrap();
        assert!(path.to_string_lossy().contains("openclaw-manager"));
    }

    #[test]
    fn test_get_encrypted_file_path() {
        let path = SecureStorage::get_encrypted_file_path("openai").unwrap();
        let path_str = path.to_string_lossy();

        assert!(path_str.contains("openclaw-manager"));
        assert!(path_str.ends_with("openai.enc"));
    }

    #[test]
    fn test_encrypted_file_storage() {
        let test_provider = "__test_encrypted__";
        let test_key = "sk-test12345-encrypted";

        // 清理可能存在的旧文件
        let _ = SecureStorage::delete_encrypted_file(test_provider);

        // 测试保存
        let result = SecureStorage::save_to_encrypted_file(test_provider, test_key);
        assert!(result.is_ok(), "Failed to save: {:?}", result.err());

        // 验证文件存在且权限正确
        let file_path = SecureStorage::get_encrypted_file_path(test_provider).unwrap();
        assert!(file_path.exists());

        #[cfg(unix)]
        {
            let metadata = fs::metadata(&file_path).unwrap();
            let permissions = metadata.permissions().mode();
            // 检查文件权限是否为 0o600 (去掉文件类型位)
            assert_eq!(permissions & 0o777, 0o600, "File permissions should be 0o600");
        }

        // 验证文件内容不是明文
        let mut file = fs::File::open(&file_path).unwrap();
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();
        assert!(!contents.windows(test_key.len()).any(|window| window == test_key.as_bytes()));

        // 测试读取
        let retrieved = SecureStorage::read_from_encrypted_file(test_provider);
        assert!(retrieved.is_ok(), "Failed to read: {:?}", retrieved.err());
        assert_eq!(retrieved.unwrap(), Some(test_key.to_string()));

        // 测试删除
        let delete_result = SecureStorage::delete_encrypted_file(test_provider);
        assert!(delete_result.is_ok());
        assert!(!file_path.exists());
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = SecureStorage::read_from_encrypted_file("__nonexistent_provider__");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_storage_backend_enum() {
        assert_eq!(StorageBackend::Keychain as i32, 0);
        assert_eq!(StorageBackend::EncryptedFile as i32, 1);

        assert_ne!(StorageBackend::Keychain, StorageBackend::EncryptedFile);
    }

    #[test]
    fn test_secure_storage_with_backend() {
        let storage = SecureStorage::with_backend(StorageBackend::EncryptedFile);
        assert_eq!(storage.backend(), StorageBackend::EncryptedFile);

        let storage2 = SecureStorage::with_backend(StorageBackend::Keychain);
        assert_eq!(storage2.backend(), StorageBackend::Keychain);
    }

    #[test]
    fn test_get_known_providers() {
        let providers = SecureStorage::get_known_providers();
        assert!(providers.contains(&"openai"));
        assert!(providers.contains(&"anthropic"));
        assert!(providers.contains(&"google"));
        assert!(providers.contains(&"azure"));
        assert!(providers.contains(&"deepseek"));
    }

    // 注意：以下测试需要在有密钥链访问权限的环境中运行
    // 在 CI 环境中可能会失败
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

    #[test]
    #[ignore]
    fn test_encrypted_file_backend_full() {
        let storage = SecureStorage::with_backend(StorageBackend::EncryptedFile);
        let test_provider = "__test_file_backend__";
        let test_key = "sk-file-test";

        // 清理
        let _ = storage.delete_api_key(test_provider);

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

    #[test]
    #[ignore]
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
}
