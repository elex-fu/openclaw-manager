use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_type: String,
    pub file_size: i64,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub is_collected: bool,
    pub is_classified: bool,
    pub classification: Option<String>,
    pub custom_attributes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileScanRequest {
    pub path: String,
    pub recursive: bool,
    pub file_types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileScanResult {
    pub files: Vec<FileItem>,
    pub total_count: usize,
    pub total_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFileRequest {
    pub id: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_collected: Option<bool>,
    pub custom_attributes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseFileInfo {
    pub file_name: String,
    pub parsed_data: serde_json::Value,
}
