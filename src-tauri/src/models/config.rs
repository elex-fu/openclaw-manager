// 允许未使用的代码，这些类型用于向后兼容
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub id: String,
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConfigRequest {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigRequest {
    pub id: String,
    pub value: String,
}

/// 模型参数配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelParameters {
    /// 温度参数 (0-2, 默认 1)
    #[serde(default)]
    pub temperature: f32,
    /// 最大令牌数 (1-8192, 默认 2048)
    #[serde(default)]
    pub max_tokens: i32,
    /// Top P 采样 (0-1, 默认 1)
    #[serde(default = "default_top_p")]
    pub top_p: f32,
    /// 存在惩罚 (-2 到 2, 默认 0)
    #[serde(default)]
    pub presence_penalty: f32,
    /// 频率惩罚 (-2 到 2, 默认 0)
    #[serde(default)]
    pub frequency_penalty: f32,
}

fn default_top_p() -> f32 {
    1.0
}

impl ModelParameters {
    pub fn with_defaults() -> Self {
        Self {
            temperature: 1.0,
            max_tokens: 2048,
            top_p: 1.0,
            presence_penalty: 0.0,
            frequency_penalty: 0.0,
        }
    }
}

/// 模型能力
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelCapabilities {
    /// 支持函数调用
    #[serde(default)]
    pub function_calling: bool,
    /// 支持视觉输入
    #[serde(default)]
    pub vision: bool,
    /// 支持流式输出
    #[serde(default)]
    pub streaming: bool,
    /// 支持 JSON 模式
    #[serde(default)]
    pub json_mode: bool,
    /// 最大上下文长度
    #[serde(default)]
    pub max_context_length: Option<i32>,
    /// 自定义能力标志
    #[serde(default)]
    pub custom: HashMap<String, bool>,
}

/// 模型配置（完整版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfigFull {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub api_base: Option<String>,
    pub model: String,
    /// 优先级顺序（数字越小优先级越高）
    #[serde(default)]
    pub priority: i32,
    /// 模型参数
    #[serde(default)]
    pub parameters: ModelParameters,
    /// 模型能力
    #[serde(default)]
    pub capabilities: ModelCapabilities,
    pub enabled: bool,
    pub default: bool,
}

impl Default for ModelConfigFull {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default".to_string(),
            provider: "openai".to_string(),
            api_base: None,
            model: "gpt-4".to_string(),
            priority: 0,
            parameters: ModelParameters::with_defaults(),
            capabilities: ModelCapabilities::default(),
            enabled: true,
            default: true,
        }
    }
}

/// 连接测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    /// 延迟（毫秒）
    pub latency: u64,
    /// 错误信息（如果失败）
    pub message: Option<String>,
    /// 模型信息（如果成功）
    pub model_info: Option<String>,
}
