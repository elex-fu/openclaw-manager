//! 技能管理器
//!
//! 管理技能的安装、卸载、启用/禁用和配置

use crate::errors::app_error::{AppError, ConfigError, NetworkError};
use crate::models::skill::{
    HookType, InstalledSkill, InstallSkillRequest, Skill, SkillCategory, SkillHook,
    UpdateSkillConfigRequest,
};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info};

/// 技能管理器
#[derive(Debug, Clone)]
pub struct SkillManager {
    /// 已安装的技能
    installed_skills: Arc<RwLock<HashMap<String, InstalledSkill>>>,
    /// 技能存储目录
    storage_dir: PathBuf,
    /// 配置存储路径
    config_path: PathBuf,
}

impl SkillManager {
    /// 创建新的技能管理器
    pub fn new() -> Result<Self, AppError> {
        let storage_dir = Self::get_storage_dir()?;
        let config_path = storage_dir.join("skills.json");

        let manager = Self {
            installed_skills: Arc::new(RwLock::new(HashMap::new())),
            storage_dir,
            config_path,
        };

        // 加载已安装的技能
        manager.load_installed_skills()?;

        Ok(manager)
    }

    /// 获取技能存储目录
    fn get_storage_dir() -> Result<PathBuf, AppError> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| AppError::Config(ConfigError::FileNotFound("无法获取主目录".to_string())))?;
        let storage_dir = home_dir.join(".openclaw").join("skills");

        // 确保目录存在
        if !storage_dir.exists() {
            std::fs::create_dir_all(&storage_dir).map_err(|e| {
                AppError::Config(ConfigError::FileNotFound(format!("无法创建技能存储目录: {}", e)))
            })?;
        }

        Ok(storage_dir)
    }

    /// 加载已安装的技能
    fn load_installed_skills(&self) -> Result<(), AppError> {
        if !self.config_path.exists() {
            info!("技能配置文件不存在，初始化为空");
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.config_path).map_err(|e| {
            AppError::Config(ConfigError::FileNotFound(format!("读取技能配置文件失败: {}", e)))
        })?;

        let skills: Vec<InstalledSkill> = serde_json::from_str(&content).map_err(|e| {
            AppError::Config(ConfigError::InvalidFormat(format!("解析技能配置文件失败: {}", e)))
        })?;

        let mut installed = self.installed_skills.write().map_err(|_| {
            AppError::Internal("无法获取技能存储锁".to_string())
        })?;

        for skill in skills {
            installed.insert(skill.id().to_string(), skill);
        }

        info!("已加载 {} 个技能", installed.len());
        Ok(())
    }

    /// 保存已安装的技能
    fn save_installed_skills(&self) -> Result<(), AppError> {
        let installed = self.installed_skills.read().map_err(|_| {
            AppError::Internal("无法获取技能存储锁".to_string())
        })?;

        let skills: Vec<&InstalledSkill> = installed.values().collect();
        let content = serde_json::to_string_pretty(&skills).map_err(|e| {
            AppError::Serialization(format!("序列化技能配置失败: {}", e))
        })?;

        std::fs::write(&self.config_path, content).map_err(|e| {
            AppError::Io(e)
        })?;

        debug!("已保存 {} 个技能", skills.len());
        Ok(())
    }

    /// 获取所有已安装的技能
    pub fn get_all_skills(&self) -> Result<Vec<InstalledSkill>, AppError> {
        let installed = self.installed_skills.read().map_err(|_| {
            AppError::Internal("无法获取技能存储锁".to_string())
        })?;

        let mut skills: Vec<InstalledSkill> = installed.values().cloned().collect();
        // 按名称排序
        skills.sort_by(|a, b| a.name().cmp(b.name()));

        Ok(skills)
    }

    /// 获取已启用的技能
    pub fn get_enabled_skills(&self) -> Result<Vec<InstalledSkill>, AppError> {
        let all = self.get_all_skills()?;
        Ok(all.into_iter().filter(|s| s.is_enabled).collect())
    }

    /// 获取单个技能
    pub fn get_skill(&self, skill_id: &str) -> Result<Option<InstalledSkill>, AppError> {
        let installed = self.installed_skills.read().map_err(|_| {
            AppError::Internal("无法获取技能存储锁".to_string())
        })?;

        Ok(installed.get(skill_id).cloned())
    }

    /// 检查技能是否已安装
    pub fn is_installed(&self, skill_id: &str) -> Result<bool, AppError> {
        let installed = self.installed_skills.read().map_err(|_| {
            AppError::Internal("无法获取技能存储锁".to_string())
        })?;

        Ok(installed.contains_key(skill_id))
    }

    /// 安装技能
    pub async fn install_skill(
        &self,
        request: InstallSkillRequest,
    ) -> Result<InstalledSkill, AppError> {
        let skill_id = request.skill_id;

        info!("开始安装技能: {}", skill_id);

        // 检查是否已安装
        if self.is_installed(&skill_id)? {
            return Err(AppError::Validation(
                format!("技能 '{}' 已安装", skill_id)
            ));
        }

        // 从市场获取技能信息（模拟）
        let skill = self.fetch_skill_from_market(&skill_id, request.version).await?;

        // 创建已安装技能
        let mut installed = InstalledSkill::from_skill(skill);

        // 保存技能文件
        self.save_skill_files(&installed).await?;

        // 添加到存储
        {
            let mut skills = self.installed_skills.write().map_err(|_| {
                AppError::Internal("无法获取技能存储锁".to_string())
            })?;
            skills.insert(skill_id.clone(), installed.clone());
        }

        // 保存配置
        self.save_installed_skills()?;

        info!("技能 '{}' 安装成功", skill_id);
        Ok(installed)
    }

    /// 卸载技能
    pub fn uninstall_skill(&self, skill_id: &str) -> Result<(), AppError> {
        info!("开始卸载技能: {}", skill_id);

        // 检查是否已安装
        if !self.is_installed(skill_id)? {
            return Err(AppError::Validation(
                format!("技能 '{}' 未安装", skill_id)
            ));
        }

        // 从存储中移除
        {
            let mut skills = self.installed_skills.write().map_err(|_| {
                AppError::Internal("无法获取技能存储锁".to_string())
            })?;
            skills.remove(skill_id);
        }

        // 删除技能文件
        self.delete_skill_files(skill_id)?;

        // 保存配置
        self.save_installed_skills()?;

        info!("技能 '{}' 卸载成功", skill_id);
        Ok(())
    }

    /// 启用技能
    pub fn enable_skill(&self, skill_id: &str) -> Result<InstalledSkill, AppError> {
        info!("启用技能: {}", skill_id);

        let mut skill = self
            .get_skill(skill_id)?
            .ok_or_else(|| AppError::Validation(format!("技能 '{}' 不存在", skill_id)))?;

        skill.enable();

        // 更新存储
        {
            let mut skills = self.installed_skills.write().map_err(|_| {
                AppError::Internal("无法获取技能存储锁".to_string())
            })?;
            skills.insert(skill_id.to_string(), skill.clone());
        }

        self.save_installed_skills()?;

        Ok(skill)
    }

    /// 禁用技能
    pub fn disable_skill(&self, skill_id: &str) -> Result<InstalledSkill, AppError> {
        info!("禁用技能: {}", skill_id);

        let mut skill = self
            .get_skill(skill_id)?
            .ok_or_else(|| AppError::Validation(format!("技能 '{}' 不存在", skill_id)))?;

        skill.disable();

        // 更新存储
        {
            let mut skills = self.installed_skills.write().map_err(|_| {
                AppError::Internal("无法获取技能存储锁".to_string())
            })?;
            skills.insert(skill_id.to_string(), skill.clone());
        }

        self.save_installed_skills()?;

        Ok(skill)
    }

    /// 切换技能状态
    pub fn toggle_skill(&self, skill_id: &str, enabled: bool) -> Result<InstalledSkill, AppError> {
        if enabled {
            self.enable_skill(skill_id)
        } else {
            self.disable_skill(skill_id)
        }
    }

    /// 获取技能配置
    pub fn get_skill_config(&self, skill_id: &str) -> Result<serde_json::Value, AppError> {
        let skill = self
            .get_skill(skill_id)?
            .ok_or_else(|| AppError::Validation(format!("技能 '{}' 不存在", skill_id)))?;

        Ok(skill.config)
    }

    /// 更新技能配置
    pub fn update_skill_config(
        &self,
        request: UpdateSkillConfigRequest,
    ) -> Result<InstalledSkill, AppError> {
        let skill_id = request.skill_id;
        info!("更新技能 '{}' 的配置", skill_id);

        let mut skill = self
            .get_skill(&skill_id)?
            .ok_or_else(|| AppError::Validation(format!("技能 '{}' 不存在", skill_id)))?;

        // 验证配置（如果有schema）
        if let Some(schema) = &skill.skill.config_schema {
            self.validate_config(&request.config, schema)?;
        }

        // 更新配置
        skill.update_config(request.config);

        // 更新存储
        {
            let mut skills = self.installed_skills.write().map_err(|_| {
                AppError::Internal("无法获取技能存储锁".to_string())
            })?;
            skills.insert(skill_id.clone(), skill.clone());
        }

        self.save_installed_skills()?;

        info!("技能 '{}' 的配置已更新", skill_id);
        Ok(skill)
    }

    /// 验证配置
    fn validate_config(
        &self,
        config: &serde_json::Value,
        schema: &serde_json::Value,
    ) -> Result<(), AppError> {
        // 简化的配置验证
        // 实际项目中可以使用 jsonschema 库进行完整验证
        if let Some(obj) = config.as_object() {
            if let Some(schema_obj) = schema.get("properties").and_then(|p| p.as_object()) {
                for (key, prop_schema) in schema_obj {
                    if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
                        if required.iter().any(|r| r.as_str() == Some(key)) {
                            if !obj.contains_key(key) {
                                return Err(AppError::Validation(format!(
                                    "缺少必需配置项: {}",
                                    key
                                )));
                            }
                        }
                    }

                    // 类型检查
                    if let Some(config_value) = obj.get(key) {
                        if let Some(prop_type) = prop_schema.get("type").and_then(|t| t.as_str()) {
                            let valid = match prop_type {
                                "string" => config_value.is_string(),
                                "number" => config_value.is_number(),
                                "boolean" => config_value.is_boolean(),
                                "array" => config_value.is_array(),
                                "object" => config_value.is_object(),
                                _ => true,
                            };
                            if !valid {
                                return Err(AppError::Validation(format!(
                                    "配置项 '{}' 类型错误，期望: {}",
                                    key, prop_type
                                )));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 检查技能更新
    pub async fn check_updates(&self) -> Result<Vec<(String, String)>, AppError> {
        let skills = self.get_all_skills()?;
        let mut updates = Vec::new();

        for skill in skills {
            // 模拟检查更新
            if let Some(latest) = self.check_skill_update(&skill.skill).await? {
                updates.push((skill.id().to_string(), latest));
            }
        }

        Ok(updates)
    }

    /// 更新技能
    pub async fn update_skill(&self, skill_id: &str) -> Result<InstalledSkill, AppError> {
        info!("更新技能: {}", skill_id);

        let skill = self
            .get_skill(skill_id)?
            .ok_or_else(|| AppError::Validation(format!("技能 '{}' 不存在", skill_id)))?;

        // 获取最新版本
        let latest_skill = self.fetch_skill_from_market(skill_id, None).await?;

        // 创建新的已安装技能，保留用户配置
        let mut updated = InstalledSkill::from_skill(latest_skill);
        updated.config = skill.config;
        updated.installed_at = skill.installed_at;
        updated.is_enabled = skill.is_enabled;

        // 更新存储
        {
            let mut skills = self.installed_skills.write().map_err(|_| {
                AppError::Internal("无法获取技能存储锁".to_string())
            })?;
            skills.insert(skill_id.to_string(), updated.clone());
        }

        self.save_installed_skills()?;

        info!("技能 '{}' 更新成功", skill_id);
        Ok(updated)
    }

    /// 获取技能分类
    pub fn get_categories(&self) -> Vec<SkillCategory> {
        SkillCategory::predefined()
    }

    /// 按分类获取技能
    pub fn get_skills_by_category(&self, category: &str) -> Result<Vec<InstalledSkill>, AppError> {
        let all = self.get_all_skills()?;

        if category == "all" {
            return Ok(all);
        }

        Ok(all
            .into_iter()
            .filter(|s| {
                s.skill
                    .categories
                    .iter()
                    .any(|c| c == category)
            })
            .collect())
    }

    /// 搜索已安装的技能
    pub fn search_installed_skills(&self, query: &str) -> Result<Vec<InstalledSkill>, AppError> {
        let all = self.get_all_skills()?;
        let query_lower = query.to_lowercase();

        Ok(all
            .into_iter()
            .filter(|s| {
                s.name().to_lowercase().contains(&query_lower)
                    || s.skill
                        .description
                        .to_lowercase()
                        .contains(&query_lower)
                    || s.skill.tags.iter().any(|t| {
                        t.to_lowercase().contains(&query_lower)
                    })
            })
            .collect())
    }

    /// 从市场获取技能信息（模拟实现）
    async fn fetch_skill_from_market(
        &self,
        skill_id: &str,
        version: Option<String>,
    ) -> Result<Skill, AppError> {
        // 这里应该调用实际的技能市场API
        // 目前使用模拟数据
        let mock_skills = get_mock_skills();

        let skill = mock_skills
            .into_iter()
            .find(|s| s.id == skill_id)
            .ok_or_else(|| {
                AppError::NotFound(format!("技能 '{}' 不存在于市场", skill_id))
            })?;

        // 如果指定了版本，这里应该获取特定版本
        let _ = version;

        Ok(skill)
    }

    /// 检查技能更新（模拟实现）
    async fn check_skill_update(&self, skill: &Skill) -> Result<Option<String>, AppError> {
        // 模拟检查更新
        // 实际项目中应该调用市场API
        Ok(None)
    }

    /// 保存技能文件
    async fn save_skill_files(&self, skill: &InstalledSkill) -> Result<(), AppError> {
        let skill_dir = self.storage_dir.join(skill.id());

        if !skill_dir.exists() {
            std::fs::create_dir_all(&skill_dir).map_err(|e| {
                AppError::Config(crate::errors::app_error::ConfigError::FileNotFound(
                    format!("创建技能目录失败: {}", e)
                ))
            })?;
        }

        // 保存技能元数据
        let meta_path = skill_dir.join("skill.json");
        let meta_content = serde_json::to_string_pretty(&skill.skill).map_err(|e| {
            AppError::Serialization(format!("序列化技能元数据失败: {}", e))
        })?;

        std::fs::write(meta_path, meta_content).map_err(|e| {
            AppError::Io(e)
        })?;

        // 保存配置
        let config_path = skill_dir.join("config.json");
        let config_content = serde_json::to_string_pretty(&skill.config).map_err(|e| {
            AppError::Serialization(format!("序列化技能配置失败: {}", e))
        })?;

        std::fs::write(config_path, config_content).map_err(|e| {
            AppError::Io(e)
        })?;

        Ok(())
    }

    /// 删除技能文件
    fn delete_skill_files(&self, skill_id: &str) -> Result<(), AppError> {
        let skill_dir = self.storage_dir.join(skill_id);

        if skill_dir.exists() {
            std::fs::remove_dir_all(&skill_dir).map_err(|e| {
                AppError::Io(e)
            })?;
        }

        Ok(())
    }

    /// 获取技能存储目录
    pub fn get_skill_dir(&self, skill_id: &str) -> PathBuf {
        self.storage_dir.join(skill_id)
    }
}

impl Default for SkillManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            error!("创建SkillManager失败: {}", e);
            Self {
                installed_skills: Arc::new(RwLock::new(HashMap::new())),
                storage_dir: PathBuf::from("."),
                config_path: PathBuf::from("skills.json"),
            }
        })
    }
}

/// 获取模拟技能列表（用于开发测试）
pub fn get_mock_skills() -> Vec<Skill> {
    vec![
        Skill::new("code-assistant", "代码助手", "1.0.0")
            .with_description("智能代码补全、重构建议和错误诊断")
            .with_author("OpenClaw Team")
            .with_categories(vec!["programming".to_string()])
            .with_tags(vec!["code".to_string(), "development".to_string()])
            .with_hook(
                SkillHook::new(HookType::Command, "code_complete", "complete_code")
                    .with_description("代码补全")
                    .with_priority(1),
            )
            .with_hook(
                SkillHook::new(HookType::Command, "code_review", "review_code")
                    .with_description("代码审查")
                    .with_priority(2),
            )
            .with_config_schema(json!({
                "type": "object",
                "properties": {
                    "language": {
                        "type": "string",
                        "title": "默认编程语言",
                        "default": "rust"
                    },
                    "style_guide": {
                        "type": "string",
                        "title": "代码风格指南",
                        "enum": ["google", "microsoft", "custom"],
                        "default": "google"
                    },
                    "max_suggestions": {
                        "type": "number",
                        "title": "最大建议数",
                        "default": 5
                    }
                },
                "required": ["language"]
            }))
            .with_default_config(json!({
                "language": "rust",
                "style_guide": "google",
                "max_suggestions": 5
            })),
        Skill::new("writing-assistant", "写作助手", "1.2.0")
            .with_description("智能写作辅助，包括语法检查、风格建议和翻译")
            .with_author("OpenClaw Team")
            .with_categories(vec!["writing".to_string()])
            .with_tags(vec!["writing".to_string(), "translation".to_string()])
            .with_hook(
                SkillHook::new(HookType::PreProcess, "grammar_check", "check_grammar")
                    .with_description("语法检查")
                    .with_priority(1),
            )
            .with_config_schema(json!({
                "type": "object",
                "properties": {
                    "language": {
                        "type": "string",
                        "title": "目标语言",
                        "default": "zh-CN"
                    },
                    "tone": {
                        "type": "string",
                        "title": "写作风格",
                        "enum": ["formal", "casual", "professional"],
                        "default": "professional"
                    }
                }
            }))
            .with_default_config(json!({
                "language": "zh-CN",
                "tone": "professional"
            })),
        Skill::new("data-analyzer", "数据分析", "0.9.0")
            .with_description("数据可视化、统计分析和报告生成")
            .with_author("DataViz Team")
            .with_categories(vec!["data".to_string()])
            .with_tags(vec!["data".to_string(), "visualization".to_string()])
            .with_hook(
                SkillHook::new(HookType::Command, "analyze_data", "analyze")
                    .with_description("数据分析")
                    .with_priority(1),
            )
            .with_config_schema(json!({
                "type": "object",
                "properties": {
                    "default_chart_type": {
                        "type": "string",
                        "title": "默认图表类型",
                        "enum": ["bar", "line", "pie", "scatter"],
                        "default": "bar"
                    },
                    "auto_visualize": {
                        "type": "boolean",
                        "title": "自动可视化",
                        "default": true
                    }
                }
            })),
        Skill::new("image-processor", "图像处理", "1.0.5")
            .with_description("图像识别、编辑和生成")
            .with_author("VisionAI")
            .with_categories(vec!["image".to_string()])
            .with_tags(vec!["image".to_string(), "ai".to_string()])
            .with_hook(
                SkillHook::new(HookType::Command, "process_image", "process")
                    .with_description("图像处理")
                    .with_priority(1),
            ),
        Skill::new("task-automation", "任务自动化", "2.0.0")
            .with_description("自动化重复任务和工作流")
            .with_author("AutoBot Inc")
            .with_categories(vec!["automation".to_string(), "productivity".to_string()])
            .with_tags(vec!["automation".to_string(), "workflow".to_string()])
            .with_hook(
                SkillHook::new(HookType::Event, "task_trigger", "handle_task")
                    .with_description("任务触发")
                    .with_priority(1),
            )
            .with_config_schema(json!({
                "type": "object",
                "properties": {
                    "auto_run": {
                        "type": "boolean",
                        "title": "自动运行",
                        "default": false
                    },
                    "schedule": {
                        "type": "string",
                        "title": "定时规则",
                        "description": "Cron表达式"
                    }
                }
            })),
        Skill::new("search-enhancer", "搜索增强", "1.1.0")
            .with_description("增强搜索能力，支持多源搜索和结果汇总")
            .with_author("SearchPlus")
            .with_categories(vec!["search".to_string()])
            .with_tags(vec!["search".to_string(), "web".to_string()])
            .with_hook(
                SkillHook::new(HookType::Command, "web_search", "search")
                    .with_description("网络搜索")
                    .with_priority(1),
            )
            .with_config_schema(json!({
                "type": "object",
                "properties": {
                    "search_engine": {
                        "type": "string",
                        "title": "搜索引擎",
                        "enum": ["google", "bing", "duckduckgo"],
                        "default": "google"
                    },
                    "max_results": {
                        "type": "number",
                        "title": "最大结果数",
                        "default": 10
                    }
                }
            })),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_skill_manager_creation() {
        // 使用临时目录测试
        let manager = SkillManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_mock_skills() {
        let skills = get_mock_skills();
        assert!(!skills.is_empty());
        assert_eq!(skills[0].id, "code-assistant");
    }

    #[test]
    fn test_categories() {
        let manager = SkillManager::default();
        let categories = manager.get_categories();
        assert!(!categories.is_empty());
        assert_eq!(categories[0].id, "all");
    }
}
