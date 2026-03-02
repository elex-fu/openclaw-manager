use serde::{Deserialize, Serialize};

/// 技能Hook类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HookType {
    /// 预处理Hook
    PreProcess,
    /// 后处理Hook
    PostProcess,
    /// 命令Hook
    Command,
    /// 事件Hook
    Event,
    /// 工具Hook
    Tool,
}

impl Default for HookType {
    fn default() -> Self {
        HookType::Command
    }
}

/// 技能Hook定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillHook {
    /// Hook类型
    pub hook_type: HookType,
    /// 触发器
    pub trigger: String,
    /// 处理器
    pub handler: String,
    /// 描述
    pub description: Option<String>,
    /// 优先级（数字越小优先级越高）
    pub priority: i32,
}

impl SkillHook {
    /// 创建新的Hook
    pub fn new(hook_type: HookType, trigger: impl Into<String>, handler: impl Into<String>) -> Self {
        Self {
            hook_type,
            trigger: trigger.into(),
            handler: handler.into(),
            description: None,
            priority: 0,
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// 技能元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// 技能ID
    pub id: String,
    /// 技能名称
    pub name: String,
    /// 技能描述
    pub description: String,
    /// 作者
    pub author: String,
    /// 版本
    pub version: String,
    /// 分类列表
    pub categories: Vec<String>,
    /// 标签列表
    pub tags: Vec<String>,
    /// 图标URL
    pub icon_url: Option<String>,
    /// 评分（0-5）
    pub rating: f32,
    /// 下载次数
    pub downloads: u32,
    /// Hook列表
    pub hooks: Vec<SkillHook>,
    /// 配置Schema（JSON Schema格式）
    pub config_schema: Option<serde_json::Value>,
    /// 默认配置
    pub default_config: Option<serde_json::Value>,
    /// 依赖的技能ID列表
    pub dependencies: Vec<String>,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

impl Skill {
    /// 创建新的技能
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        let now = chrono::Local::now().to_rfc3339();
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            author: String::new(),
            version: version.into(),
            categories: Vec::new(),
            tags: Vec::new(),
            icon_url: None,
            rating: 0.0,
            downloads: 0,
            hooks: Vec::new(),
            config_schema: None,
            default_config: None,
            dependencies: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// 设置作者
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    /// 设置分类
    pub fn with_categories(mut self, categories: Vec<String>) -> Self {
        self.categories = categories;
        self
    }

    /// 设置标签
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// 设置图标
    pub fn with_icon(mut self, icon_url: impl Into<String>) -> Self {
        self.icon_url = Some(icon_url.into());
        self
    }

    /// 添加Hook
    pub fn with_hook(mut self, hook: SkillHook) -> Self {
        self.hooks.push(hook);
        self
    }

    /// 设置配置Schema
    pub fn with_config_schema(mut self, schema: serde_json::Value) -> Self {
        self.config_schema = Some(schema);
        self
    }

    /// 设置默认配置
    pub fn with_default_config(mut self, config: serde_json::Value) -> Self {
        self.default_config = Some(config);
        self
    }

    /// 添加依赖
    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependencies.push(dependency.into());
        self
    }

    /// 检查是否有配置Schema
    pub fn has_config(&self) -> bool {
        self.config_schema.is_some()
    }

    /// 获取主分类
    pub fn primary_category(&self) -> Option<&str> {
        self.categories.first().map(|s| s.as_str())
    }
}

/// 已安装技能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSkill {
    /// 技能基础信息
    #[serde(flatten)]
    pub skill: Skill,
    /// 是否启用
    pub is_enabled: bool,
    /// 当前配置
    pub config: serde_json::Value,
    /// 安装时间
    pub installed_at: String,
    /// 更新时间
    pub updated_at: String,
    /// 是否有更新
    pub has_update: bool,
    /// 最新版本（如果有更新）
    pub latest_version: Option<String>,
}

impl InstalledSkill {
    /// 从技能创建已安装技能
    pub fn from_skill(skill: Skill) -> Self {
        let now = chrono::Local::now().to_rfc3339();
        let config = skill.default_config.clone().unwrap_or_else(|| serde_json::json!({}));
        Self {
            skill,
            is_enabled: true,
            config,
            installed_at: now.clone(),
            updated_at: now,
            has_update: false,
            latest_version: None,
        }
    }

    /// 获取技能ID
    pub fn id(&self) -> &str {
        &self.skill.id
    }

    /// 获取技能名称
    pub fn name(&self) -> &str {
        &self.skill.name
    }

    /// 更新配置
    pub fn update_config(&mut self, config: serde_json::Value) {
        self.config = config;
        self.updated_at = chrono::Local::now().to_rfc3339();
    }

    /// 启用技能
    pub fn enable(&mut self) {
        self.is_enabled = true;
        self.updated_at = chrono::Local::now().to_rfc3339();
    }

    /// 禁用技能
    pub fn disable(&mut self) {
        self.is_enabled = false;
        self.updated_at = chrono::Local::now().to_rfc3339();
    }

    /// 标记有更新
    pub fn mark_update_available(&mut self, latest_version: impl Into<String>) {
        self.has_update = true;
        self.latest_version = Some(latest_version.into());
    }

    /// 清除更新标记
    pub fn clear_update(&mut self) {
        self.has_update = false;
        self.latest_version = None;
    }
}

/// 技能分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCategory {
    /// 分类ID
    pub id: String,
    /// 分类名称
    pub name: String,
    /// 分类描述
    pub description: Option<String>,
    /// 图标
    pub icon: Option<String>,
    /// 排序
    pub sort_order: i32,
}

impl SkillCategory {
    /// 创建新分类
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            icon: None,
            sort_order: 0,
        }
    }

    /// 预定义分类
    pub fn predefined() -> Vec<Self> {
        vec![
            Self::new("all", "全部").with_icon("layout-grid"),
            Self::new("programming", "编程开发").with_icon("code"),
            Self::new("writing", "写作助手").with_icon("pen-tool"),
            Self::new("data", "数据分析").with_icon("bar-chart"),
            Self::new("image", "图像处理").with_icon("image"),
            Self::new("productivity", "效率工具").with_icon("zap"),
            Self::new("communication", "沟通协作").with_icon("message-circle"),
            Self::new("search", "搜索增强").with_icon("search"),
            Self::new("automation", "自动化").with_icon("cpu"),
        ]
    }

    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 设置图标
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// 设置排序
    pub fn with_sort_order(mut self, order: i32) -> Self {
        self.sort_order = order;
        self
    }
}

/// 技能搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSearchResult {
    /// 技能列表
    pub skills: Vec<Skill>,
    /// 总数
    pub total: usize,
    /// 当前页
    pub page: usize,
    /// 每页数量
    pub per_page: usize,
    /// 查询关键词
    pub query: Option<String>,
    /// 分类筛选
    pub category: Option<String>,
}

/// 安装技能请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallSkillRequest {
    /// 技能ID
    pub skill_id: String,
    /// 版本（可选，默认最新）
    pub version: Option<String>,
}

/// 更新技能配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSkillConfigRequest {
    /// 技能ID
    pub skill_id: String,
    /// 配置内容
    pub config: serde_json::Value,
}

/// 切换技能状态请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleSkillRequest {
    /// 技能ID
    pub skill_id: String,
    /// 是否启用
    pub enabled: bool,
}

/// 技能市场项（用于展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMarketItem {
    /// 技能ID
    pub id: String,
    /// 技能名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 作者
    pub author: String,
    /// 版本
    pub version: String,
    /// 分类
    pub categories: Vec<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 图标URL
    pub icon_url: Option<String>,
    /// 评分
    pub rating: f32,
    /// 下载次数
    pub downloads: u32,
    /// 下载URL
    pub download_url: String,
    /// 是否已安装
    pub is_installed: bool,
    /// 是否有更新
    pub has_update: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_creation() {
        let skill = Skill::new("test-skill", "Test Skill", "1.0.0")
            .with_description("A test skill")
            .with_author("Test Author")
            .with_categories(vec!["programming".to_string()]);

        assert_eq!(skill.id, "test-skill");
        assert_eq!(skill.name, "Test Skill");
        assert_eq!(skill.version, "1.0.0");
        assert_eq!(skill.description, "A test skill");
        assert_eq!(skill.author, "Test Author");
        assert_eq!(skill.categories.len(), 1);
    }

    #[test]
    fn test_skill_hook() {
        let hook = SkillHook::new(HookType::Command, "test", "handle_test")
            .with_description("Test hook")
            .with_priority(1);

        assert_eq!(hook.trigger, "test");
        assert_eq!(hook.handler, "handle_test");
        assert_eq!(hook.priority, 1);
    }

    #[test]
    fn test_installed_skill() {
        let skill = Skill::new("test", "Test", "1.0.0");
        let mut installed = InstalledSkill::from_skill(skill);

        assert!(installed.is_enabled);
        assert!(!installed.has_update);

        installed.disable();
        assert!(!installed.is_enabled);

        installed.mark_update_available("1.1.0");
        assert!(installed.has_update);
        assert_eq!(installed.latest_version, Some("1.1.0".to_string()));
    }

    #[test]
    fn test_categories() {
        let categories = SkillCategory::predefined();
        assert!(!categories.is_empty());
        assert_eq!(categories[0].id, "all");
    }
}
