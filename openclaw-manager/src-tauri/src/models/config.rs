// 允许未使用的代码，这些类型用于向后兼容
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

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
