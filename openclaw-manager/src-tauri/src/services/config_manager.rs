//! 配置管理器
//!
//! 提供线程安全的配置管理，支持乐观锁并发控制

#![allow(dead_code)]

use crate::errors::{AppError, ConfigError};
use crate::models::config::ModelConfigFull;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

/// 配置状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigState {
    /// 配置版本号（用于乐观锁）
    pub version: u32,
    /// 最后更新时间
    pub last_updated: u64,
    /// 应用配置
    pub app: AppConfig,
    /// 模型配置
    pub models: Vec<ModelConfig>,
    /// 完整模型配置（包含高级参数）
    pub models_full: Vec<ModelConfigFull>,
    /// 服务配置
    pub services: HashMap<String, ServiceConfig>,
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: String,
    pub language: String,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub check_updates: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            language: "zh-CN".to_string(),
            auto_start: false,
            minimize_to_tray: true,
            check_updates: true,
        }
    }
}

/// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub api_base: Option<String>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<i32>,
    pub enabled: bool,
    pub default: bool,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default".to_string(),
            provider: "openai".to_string(),
            api_base: None,
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: None,
            enabled: true,
            default: true,
        }
    }
}

/// 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub enabled: bool,
    pub port: u16,
    pub auto_start: bool,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 8080,
            auto_start: false,
        }
    }
}

impl Default for ConfigState {
    fn default() -> Self {
        Self {
            version: 1,
            last_updated: current_timestamp(),
            app: AppConfig::default(),
            models: vec![ModelConfig::default()],
            models_full: vec![ModelConfigFull::default()],
            services: {
                let mut map = HashMap::new();
                map.insert("gateway".to_string(), ServiceConfig::default());
                map
            },
        }
    }
}

/// 配置验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self {
            valid: false,
            errors: vec![error.into()],
        }
    }

    pub fn with_errors(errors: Vec<String>) -> Self {
        Self {
            valid: errors.is_empty(),
            errors,
        }
    }
}

/// 应用设置（存储在单独的文件中）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    /// 当前选中的 Agent ID
    pub current_agent_id: Option<String>,
    /// 其他应用级设置
    pub theme: String,
    pub language: String,
}

/// 配置管理器
pub struct ConfigManager {
    state: Arc<RwLock<ConfigState>>,
    config_path: PathBuf,
    settings_path: PathBuf,
}

impl Default for ConfigManager {
    fn default() -> Self {
        let config_dir = dirs::config_dir()
            .map(|p| p.join("openclaw-manager"))
            .unwrap_or_else(|| PathBuf::from("./config"));

        Self {
            state: Arc::new(RwLock::new(ConfigState::default())),
            config_path: config_dir.join("config.yaml"),
            settings_path: config_dir.join("settings.yaml"),
        }
    }
}

impl ConfigManager {
    /// 创建新的配置管理器（同步版本）
    pub fn new() -> Result<Self, AppError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| ConfigError::FileNotFound("无法获取配置目录".to_string()))?
            .join("openclaw-manager");

        let config_path = config_dir.join("config.yaml");
        let settings_path = config_dir.join("settings.yaml");

        // 确保配置目录存在（同步）
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(AppError::Io)?;
        }

        // 加载或创建默认配置（同步）
        let state = if config_path.exists() {
            Self::load_from_file_sync(&config_path)?
        } else {
            let default = ConfigState::default();
            Self::save_to_file_sync(&config_path, &default)?;
            default
        };

        Ok(Self {
            state: Arc::new(RwLock::new(state)),
            config_path,
            settings_path,
        })
    }

    /// 创建新的配置管理器（异步版本）
    pub async fn new_async(config_path: impl AsRef<Path>) -> Result<Self, AppError> {
        let config_path = config_path.as_ref().to_path_buf();
        let settings_path = config_path.parent()
            .map(|p| p.join("settings.yaml"))
            .unwrap_or_else(|| config_path.with_file_name("settings.yaml"));

        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).await.map_err(AppError::Io)?;
        }

        // 加载或创建默认配置
        let state = if config_path.exists() {
            Self::load_from_file(&config_path).await?
        } else {
            let default = ConfigState::default();
            Self::save_to_file(&config_path, &default).await?;
            default
        };

        Ok(Self {
            state: Arc::new(RwLock::new(state)),
            config_path,
            settings_path,
        })
    }

    /// 从默认路径创建配置管理器
    pub async fn from_default_path() -> Result<Self, AppError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| ConfigError::FileNotFound("无法获取配置目录".to_string()))?
            .join("openclaw-manager");

        let config_path = config_dir.join("config.yaml");
        Self::new_async(config_path).await
    }

    /// 设置当前 Agent
    pub fn set_current_agent(&self, agent_id: &str) -> Result<(), AppError> {
        let mut settings = self.load_settings_sync()?;
        settings.current_agent_id = Some(agent_id.to_string());
        self.save_settings_sync(&settings)?;
        Ok(())
    }

    /// 获取当前 Agent ID
    pub fn get_current_agent(&self) -> String {
        self.load_settings_sync()
            .ok()
            .and_then(|s| s.current_agent_id)
            .unwrap_or_else(|| "default-assistant".to_string())
    }

    /// 加载设置（同步）
    fn load_settings_sync(&self) -> Result<AppSettings, AppError> {
        if self.settings_path.exists() {
            let content = std::fs::read_to_string(&self.settings_path)
                .map_err(|e| ConfigError::FileNotFound(format!("无法读取设置文件: {}", e)))?;
            serde_yaml::from_str(&content)
                .map_err(|e| ConfigError::InvalidFormat(format!("YAML 解析错误: {}", e)).into())
        } else {
            Ok(AppSettings::default())
        }
    }

    /// 保存设置（同步）
    fn save_settings_sync(&self, settings: &AppSettings) -> Result<(), AppError> {
        let yaml = serde_yaml::to_string(settings)
            .map_err(|e| ConfigError::InvalidFormat(format!("序列化错误: {}", e)))?;
        std::fs::write(&self.settings_path, yaml).map_err(AppError::Io)?;
        Ok(())
    }

    /// 从文件加载配置（同步）
    fn load_from_file_sync(path: impl AsRef<Path>) -> Result<ConfigState, AppError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            ConfigError::FileNotFound(format!("无法读取配置文件: {}", e))
        })?;

        serde_yaml::from_str(&content).map_err(|e| {
            ConfigError::InvalidFormat(format!("YAML 解析错误: {}", e)).into()
        })
    }

    /// 保存配置到文件（同步）
    fn save_to_file_sync(
        path: impl AsRef<Path>,
        state: &ConfigState,
    ) -> Result<(), AppError> {
        let yaml = serde_yaml::to_string(state)
            .map_err(|e| ConfigError::InvalidFormat(format!("序列化错误: {}", e)))?;

        std::fs::write(path.as_ref(), yaml).map_err(AppError::Io)?;
        Ok(())
    }

    /// 获取当前配置的只读引用
    pub async fn get_state(&self) -> ConfigState {
        self.state.read().await.clone()
    }

    /// 更新配置（带乐观锁检查）
    ///
    /// # Arguments
    /// * `new_state` - 新的配置状态
    /// * `expected_version` - 期望的当前版本号
    ///
    /// # Returns
    /// * `Ok(())` - 更新成功
    /// * `Err(ConfigError::VersionMismatch)` - 版本冲突
    pub async fn update_state(
        &self,
        new_state: ConfigState,
        expected_version: u32,
    ) -> Result<(), AppError> {
        let mut state = self.state.write().await;

        // 乐观锁检查
        if state.version != expected_version {
            return Err(ConfigError::VersionMismatch {
                expected: expected_version,
                found: state.version,
            }.into());
        }

        // 验证新配置
        let validation = Self::validate(&new_state);
        if !validation.valid {
            return Err(ConfigError::ValidationFailed(validation.errors.join(", ")).into());
        }

        // 更新版本号和时间戳
        let mut new_state = new_state;
        new_state.version = state.version + 1;
        new_state.last_updated = current_timestamp();

        // 保存到文件
        Self::save_to_file(&self.config_path, &new_state).await?;

        // 更新内存状态
        *state = new_state;

        log::info!("Configuration updated to version {}", state.version);
        Ok(())
    }

    /// 部分更新配置
    pub async fn update_partial<F>(
        &self,
        updater: F,
    ) -> Result<(), AppError>
    where
        F: FnOnce(&mut ConfigState),
    {
        let mut state = self.state.write().await;
        let current_version = state.version;

        // 克隆并修改
        let mut new_state = state.clone();
        updater(&mut new_state);

        // 验证
        let validation = Self::validate(&new_state);
        if !validation.valid {
            return Err(ConfigError::ValidationFailed(validation.errors.join(", ")).into());
        }

        // 更新元数据
        new_state.version = current_version + 1;
        new_state.last_updated = current_timestamp();

        // 保存
        Self::save_to_file(&self.config_path, &new_state).await?;

        *state = new_state;
        Ok(())
    }

    /// 验证配置
    pub fn validate(state: &ConfigState) -> ValidationResult {
        let mut errors = Vec::new();

        // 验证模型配置
        for model in &state.models {
            if model.id.is_empty() {
                errors.push("模型 ID 不能为空".to_string());
            }
            if model.provider.is_empty() {
                errors.push(format!("模型 '{}' 的提供商不能为空", model.id));
            }
            if !(0.0..=2.0).contains(&model.temperature) {
                errors.push(format!("模型 '{}' 的 temperature 必须在 0-2 之间", model.id));
            }
        }

        // 验证服务配置
        for (name, service) in &state.services {
            if service.port == 0 {
                errors.push(format!("服务 '{}' 的端口号不能为 0", name));
            }
        }

        ValidationResult::with_errors(errors)
    }

    /// 导出配置到文件
    pub async fn export_to(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(), AppError> {
        let state = self.state.read().await.clone();
        Self::save_to_file(path, &state).await
    }

    /// 从文件导入配置
    pub async fn import_from(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(), AppError> {
        let new_state = Self::load_from_file(path).await?;

        // 验证导入的配置
        let validation = Self::validate(&new_state);
        if !validation.valid {
            return Err(ConfigError::ImportFailed(validation.errors.join(", ")).into());
        }

        // 更新配置
        let expected_version = self.state.read().await.version;
        self.update_state(new_state, expected_version).await
    }

    /// 重置为默认配置
    pub async fn reset_to_default(&self) -> Result<(), AppError> {
        let default = ConfigState::default();
        let expected_version = self.state.read().await.version;
        self.update_state(default, expected_version).await
    }

    /// 获取配置版本号
    pub async fn get_version(&self) -> u32 {
        self.state.read().await.version
    }

    /// 从文件加载配置
    async fn load_from_file(path: impl AsRef<Path>) -> Result<ConfigState, AppError> {
        let content = fs::read_to_string(path.as_ref()).await.map_err(|e| {
            ConfigError::FileNotFound(format!("无法读取配置文件: {}", e))
        })?;

        serde_yaml::from_str(&content).map_err(|e| {
            ConfigError::InvalidFormat(format!("YAML 解析错误: {}", e)).into()
        })
    }

    /// 保存配置到文件
    async fn save_to_file(
        path: impl AsRef<Path>,
        state: &ConfigState,
    ) -> Result<(), AppError> {
        let yaml = serde_yaml::to_string(state)
            .map_err(|e| ConfigError::InvalidFormat(format!("序列化错误: {}", e)))?;

        fs::write(path.as_ref(), yaml).await.map_err(AppError::Io)?;
        Ok(())
    }

    /// 添加模型配置
    pub async fn add_model(&self,
        model: ModelConfig,
    ) -> Result<(), AppError> {
        self.update_partial(|state| {
            // 如果设置为默认，取消其他默认
            if model.default {
                for m in &mut state.models {
                    m.default = false;
                }
            }
            state.models.push(model);
        }).await
    }

    /// 删除模型配置
    pub async fn remove_model(
        &self,
        model_id: &str,
    ) -> Result<(), AppError> {
        self.update_partial(|state| {
            state.models.retain(|m| m.id != model_id);
        }).await
    }

    /// 更新模型配置
    pub async fn update_model(
        &self,
        model_id: &str,
        updater: impl FnOnce(&mut ModelConfig),
    ) -> Result<(), AppError> {
        self.update_partial(|state| {
            if let Some(model) = state.models.iter_mut().find(|m| m.id == model_id) {
                updater(model);
            }
        }).await
    }

    /// 获取默认模型
    pub async fn get_default_model(&self) -> Option<ModelConfig> {
        let state = self.state.read().await;
        state.models.iter().find(|m| m.default).cloned()
            .or_else(|| state.models.first().cloned())
    }

    // ========== 新增方法：完整模型配置支持 ==========

    /// 获取所有模型（完整配置）
    pub async fn get_models_full(&self) -> Result<Vec<ModelConfigFull>, AppError> {
        let state = self.state.read().await;
        Ok(state.models_full.clone())
    }

    /// 保存模型配置（完整版）
    pub async fn save_model_full(&self, model: ModelConfigFull) -> Result<(), AppError> {
        self.update_partial(|state| {
            // 如果设置为默认，取消其他默认
            if model.default {
                for m in &mut state.models_full {
                    m.default = false;
                }
            }

            // 更新或添加模型
            if let Some(index) = state.models_full.iter().position(|m| m.id == model.id) {
                state.models_full[index] = model;
            } else {
                state.models_full.push(model);
            }

            // 同步更新简化版模型列表
            Self::sync_models_list(state);
        }).await
    }

    /// 更新模型优先级（批量）
    pub async fn update_model_priorities(
        &self,
        model_orders: Vec<(String, i32)>,
    ) -> Result<(), AppError> {
        self.update_partial(|state| {
            for (model_id, priority) in model_orders {
                if let Some(model) = state.models_full.iter_mut().find(|m| m.id == model_id) {
                    model.priority = priority;
                }
            }

            // 按优先级排序
            state.models_full.sort_by_key(|m| m.priority);

            // 同步更新简化版模型列表
            Self::sync_models_list(state);
        }).await
    }

    /// 同步简化版模型列表
    fn sync_models_list(state: &mut ConfigState) {
        state.models = state.models_full.iter().map(|m| ModelConfig {
            id: m.id.clone(),
            name: m.name.clone(),
            provider: m.provider.clone(),
            api_base: m.api_base.clone(),
            model: m.model.clone(),
            temperature: m.parameters.temperature,
            max_tokens: Some(m.parameters.max_tokens),
            enabled: m.enabled,
            default: m.default,
        }).collect();
    }
}

/// 获取当前时间戳（秒）
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let state = ConfigState::default();
        let result = ConfigManager::validate(&state);
        assert!(result.valid);
    }

    #[test]
    fn test_validation_invalid_temperature() {
        let mut state = ConfigState::default();
        state.models[0].temperature = 3.0;
        let result = ConfigManager::validate(&state);
        assert!(!result.valid);
    }
}
