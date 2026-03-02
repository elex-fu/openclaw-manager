//! 技能相关命令
//!
//! 提供技能管理、技能市场相关的Tauri命令

use crate::errors::app_error::AppError;
use crate::models::skill::{
    InstallSkillRequest, InstalledSkill, Skill, SkillCategory, SkillMarketItem,
    SkillSearchResult, ToggleSkillRequest, UpdateSkillConfigRequest,
};
use crate::models::ApiResponse;
use crate::services::skill_manager::SkillManager;
use crate::services::skill_market::SkillMarket;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;

/// 技能状态（用于Tauri状态管理）
pub struct SkillState {
    pub manager: Arc<Mutex<SkillManager>>,
    pub market: Arc<Mutex<SkillMarket>>,
}

impl SkillState {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            manager: Arc::new(Mutex::new(SkillManager::new()?)),
            market: Arc::new(Mutex::new(SkillMarket::new())),
        })
    }
}

impl Default for SkillState {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            log::error!("创建SkillState失败: {}", e);
            Self {
                manager: Arc::new(Mutex::new(SkillManager::default())),
                market: Arc::new(Mutex::new(SkillMarket::default())),
            }
        })
    }
}

// ============ 已安装技能管理命令 ============

/// 获取所有已安装的技能
#[tauri::command]
pub async fn get_skills(state: State<'_, SkillState>) -> Result<ApiResponse<Vec<InstalledSkill>>, String> {
    let manager = state.manager.lock().await;
    match manager.get_all_skills() {
        Ok(skills) => Ok(ApiResponse::success(skills)),
        Err(e) => Ok(ApiResponse::error(format!("获取技能列表失败: {}", e))),
    }
}

/// 获取单个技能详情
#[tauri::command]
pub async fn get_skill(state: State<'_, SkillState>, skill_id: String) -> Result<ApiResponse<Option<InstalledSkill>>, String> {
    let manager = state.manager.lock().await;
    match manager.get_skill(&skill_id) {
        Ok(skill) => Ok(ApiResponse::success(skill)),
        Err(e) => Ok(ApiResponse::error(format!("获取技能详情失败: {}", e))),
    }
}

/// 搜索已安装的技能
#[tauri::command]
pub async fn search_installed_skills(state: State<'_, SkillState>, query: String) -> Result<ApiResponse<Vec<InstalledSkill>>, String> {
    let manager = state.manager.lock().await;
    match manager.search_installed_skills(&query) {
        Ok(skills) => Ok(ApiResponse::success(skills)),
        Err(e) => Ok(ApiResponse::error(format!("搜索技能失败: {}", e))),
    }
}

/// 安装技能
#[tauri::command]
pub async fn install_skill(
    state: State<'_, SkillState>,
    skill_id: String,
) -> Result<ApiResponse<InstalledSkill>, String> {
    let request = InstallSkillRequest {
        skill_id,
        version: None,
    };

    let manager = state.manager.lock().await;
    match manager.install_skill(request).await {
        Ok(skill) => Ok(ApiResponse::success(skill)),
        Err(e) => Ok(ApiResponse::error(format!("安装技能失败: {}", e))),
    }
}

/// 卸载技能
#[tauri::command]
pub async fn uninstall_skill(state: State<'_, SkillState>, skill_id: String) -> Result<ApiResponse<bool>, String> {
    let manager = state.manager.lock().await;
    match manager.uninstall_skill(&skill_id) {
        Ok(_) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(format!("卸载技能失败: {}", e))),
    }
}

/// 启用技能
#[tauri::command]
pub async fn enable_skill(state: State<'_, SkillState>, skill_id: String) -> Result<ApiResponse<InstalledSkill>, String> {
    let manager = state.manager.lock().await;
    match manager.enable_skill(&skill_id) {
        Ok(skill) => Ok(ApiResponse::success(skill)),
        Err(e) => Ok(ApiResponse::error(format!("启用技能失败: {}", e))),
    }
}

/// 禁用技能
#[tauri::command]
pub async fn disable_skill(state: State<'_, SkillState>, skill_id: String) -> Result<ApiResponse<InstalledSkill>, String> {
    let manager = state.manager.lock().await;
    match manager.disable_skill(&skill_id) {
        Ok(skill) => Ok(ApiResponse::success(skill)),
        Err(e) => Ok(ApiResponse::error(format!("禁用技能失败: {}", e))),
    }
}

/// 切换技能状态
#[tauri::command]
pub async fn toggle_skill(
    state: State<'_, SkillState>,
    request: ToggleSkillRequest,
) -> Result<ApiResponse<InstalledSkill>, String> {
    let manager = state.manager.lock().await;
    match manager.toggle_skill(&request.skill_id, request.enabled) {
        Ok(skill) => Ok(ApiResponse::success(skill)),
        Err(e) => Ok(ApiResponse::error(format!("切换技能状态失败: {}", e))),
    }
}

/// 获取技能配置
#[tauri::command]
pub async fn get_skill_config(
    state: State<'_, SkillState>,
    skill_id: String,
) -> Result<ApiResponse<serde_json::Value>, String> {
    let manager = state.manager.lock().await;
    match manager.get_skill_config(&skill_id) {
        Ok(config) => Ok(ApiResponse::success(config)),
        Err(e) => Ok(ApiResponse::error(format!("获取技能配置失败: {}", e))),
    }
}

/// 更新技能配置
#[tauri::command]
pub async fn update_skill_config(
    state: State<'_, SkillState>,
    request: UpdateSkillConfigRequest,
) -> Result<ApiResponse<InstalledSkill>, String> {
    let manager = state.manager.lock().await;
    match manager.update_skill_config(request) {
        Ok(skill) => Ok(ApiResponse::success(skill)),
        Err(e) => Ok(ApiResponse::error(format!("更新技能配置失败: {}", e))),
    }
}

/// 更新技能到最新版本
#[tauri::command]
pub async fn update_skill(
    state: State<'_, SkillState>,
    skill_id: String,
) -> Result<ApiResponse<InstalledSkill>, String> {
    let manager = state.manager.lock().await;
    match manager.update_skill(&skill_id).await {
        Ok(skill) => Ok(ApiResponse::success(skill)),
        Err(e) => Ok(ApiResponse::error(format!("更新技能失败: {}", e))),
    }
}

/// 检查所有技能更新
#[tauri::command]
pub async fn check_skill_updates(
    state: State<'_, SkillState>,
) -> Result<ApiResponse<Vec<( String, String)>>, String> {
    let manager = state.manager.lock().await;
    match manager.check_updates().await {
        Ok(updates) => Ok(ApiResponse::success(updates)),
        Err(e) => Ok(ApiResponse::error(format!("检查技能更新失败: {}", e))),
    }
}

// ============ 技能市场命令 ============

/// 搜索技能市场
#[tauri::command]
pub async fn search_skills(
    state: State<'_, SkillState>,
    query: Option<String>,
    category: Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
) -> Result<ApiResponse<SkillSearchResult>, String> {
    let market = state.market.lock().await;
    match market.search_skills(
        query.as_deref(),
        category.as_deref(),
        page.unwrap_or(1),
        per_page.unwrap_or(20),
    ).await {
        Ok(skills) => Ok(ApiResponse::success(skills)),
        Err(e) => Ok(ApiResponse::error(format!("搜索技能市场失败: {}", e))),
    }
}

/// 获取技能市场详情
#[tauri::command]
pub async fn get_market_skill_detail(
    state: State<'_, SkillState>,
    skill_id: String,
) -> Result<ApiResponse<Skill>, String> {
    let market = state.market.lock().await;
    match market.get_skill_detail(&skill_id).await {
        Ok(skill) => Ok(ApiResponse::success(skill)),
        Err(e) => Ok(ApiResponse::error(format!("获取技能详情失败: {}", e))),
    }
}

/// 获取热门技能
#[tauri::command]
pub async fn get_popular_skills(
    state: State<'_, SkillState>,
    limit: Option<usize>,
) -> Result<ApiResponse<Vec<SkillMarketItem>>, String> {
    let market = state.market.lock().await;
    match market.get_popular_skills(limit.unwrap_or(10)).await {
        Ok(skills) => Ok(ApiResponse::success(skills)),
        Err(e) => Ok(ApiResponse::error(format!("获取热门技能失败: {}", e))),
    }
}

/// 获取最新技能
#[tauri::command]
pub async fn get_latest_skills(
    state: State<'_, SkillState>,
    limit: Option<usize>,
) -> Result<ApiResponse<Vec<SkillMarketItem>>, String> {
    let market = state.market.lock().await;
    match market.get_latest_skills(limit.unwrap_or(10)).await {
        Ok(skills) => Ok(ApiResponse::success(skills)),
        Err(e) => Ok(ApiResponse::error(format!("获取最新技能失败: {}", e))),
    }
}

/// 获取技能分类
#[tauri::command]
pub async fn get_skill_categories(state: State<'_, SkillState>) -> Result<ApiResponse<Vec<SkillCategory>>, String> {
    let market = state.market.lock().await;
    match market.get_categories().await {
        Ok(categories) => Ok(ApiResponse::success(categories)),
        Err(e) => Ok(ApiResponse::error(format!("获取技能分类失败: {}", e))),
    }
}

/// 检查单个技能更新
#[tauri::command]
pub async fn check_single_skill_update(
    state: State<'_, SkillState>,
    skill_id: String,
    current_version: String,
) -> Result<ApiResponse<Option<String>>, String> {
    let market = state.market.lock().await;
    match market.check_update(&skill_id, &current_version).await {
        Ok(version) => Ok(ApiResponse::success(version)),
        Err(e) => Ok(ApiResponse::error(format!("检查技能更新失败: {}", e))),
    }
}
