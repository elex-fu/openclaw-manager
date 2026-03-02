// 允许未使用的代码，这些类型用于将来的功能扩展
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenClaw 配置根结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenClawConfig {
    /// 配置版本
    pub version: String,
    /// 应用名称
    pub name: String,
    /// 模型配置
    pub models: Vec<ModelConfig>,
    /// 默认模型
    pub default_model: Option<String>,
    /// Agent 配置
    pub agents: Vec<AgentConfig>,
    /// 技能配置
    pub skills: Vec<SkillConfig>,
    /// 系统设置
    pub settings: SystemSettings,
}

/// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub api_key: Option<String>,
    pub api_base: Option<String>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<i32>,
    pub enabled: bool,
}

/// Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub model_id: String,
    pub system_prompt: Option<String>,
    pub skills: Vec<String>,
    pub enabled: bool,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

/// 技能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillConfig {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub entry_point: String,
    pub config: Option<serde_json::Value>,
    pub enabled: bool,
}

/// 系统设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSettings {
    pub log_level: String,
    pub auto_update: bool,
    pub theme: String,
    pub language: String,
    pub custom_vars: HashMap<String, String>,
}

impl Default for SystemSettings {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            auto_update: true,
            theme: "system".to_string(),
            language: "zh-CN".to_string(),
            custom_vars: HashMap::new(),
        }
    }
}

/// 安装状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InstallStatus {
    NotInstalled,
    Installing { stage: String, progress: f32 },
    Installed { version: String },
    Error { message: String },
}

/// 安装结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub success: bool,
    pub version: Option<String>,
    pub message: String,
}

/// OpenClaw 版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawVersion {
    pub version: String,
    pub release_date: String,
    pub download_url: String,
    pub checksum: String,
}

impl OpenClawConfig {
    /// 创建默认配置
    pub fn default_config() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            version: "1.0.0".to_string(),
            name: "My OpenClaw".to_string(),
            models: vec![
                ModelConfig {
                    id: "default-gpt4".to_string(),
                    name: "GPT-4".to_string(),
                    provider: "openai".to_string(),
                    api_key: None,
                    api_base: None,
                    model: "gpt-4".to_string(),
                    temperature: 0.7,
                    max_tokens: Some(4096),
                    enabled: true,
                }
            ],
            default_model: Some("default-gpt4".to_string()),
            agents: vec![
                AgentConfig {
                    id: "default-assistant".to_string(),
                    name: "默认助手".to_string(),
                    description: Some("一个通用的 AI 助手".to_string()),
                    model_id: "default-gpt4".to_string(),
                    system_prompt: Some("You are a helpful assistant.".to_string()),
                    skills: vec![],
                    enabled: true,
                    created_at: now.clone(),
                    updated_at: now,
                }
            ],
            skills: vec![],
            settings: SystemSettings::default(),
        }
    }

    /// 序列化为 YAML 字符串
    pub fn to_yaml(&self) -> anyhow::Result<String> {
        serde_yaml::to_string(self).map_err(|e| anyhow::anyhow!("YAML serialize error: {}", e))
    }

    /// 从 YAML 字符串解析
    pub fn from_yaml(yaml: &str) -> anyhow::Result<Self> {
        serde_yaml::from_str(yaml).map_err(|e| anyhow::anyhow!("YAML parse error: {}", e))
    }
}
