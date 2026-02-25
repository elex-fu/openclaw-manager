use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: i32,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupWithFiles {
    #[serde(flatten)]
    pub group: Group,
    pub files: Vec<super::file::FileItem>,
    pub file_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFileToGroupRequest {
    pub group_id: String,
    pub file_id: String,
}
