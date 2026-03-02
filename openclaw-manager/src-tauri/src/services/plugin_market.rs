//! 插件市场服务
//!
//! 提供插件市场的搜索、下载等功能
//! 支持模拟实现和真实API调用

use crate::errors::app_error::{AppError, NetworkError};
use serde::{Deserialize, Serialize};

/// 市场API基础URL
const MARKET_API_BASE: &str = "https://market.openclaw.ai/api/v1";

/// 市场插件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub author_avatar: Option<String>,
    pub downloads: i32,
    pub rating: f32,
    pub rating_count: i32,
    pub icon_url: Option<String>,
    pub download_url: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub size_bytes: i64,
    pub min_app_version: Option<String>,
    pub changelog: Option<String>,
}

/// 插件搜索请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPluginsRequest {
    pub query: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub sort_by: Option<SortBy>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

/// 排序选项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    Relevance,
    Downloads,
    Rating,
    CreatedAt,
    UpdatedAt,
}

impl Default for SortBy {
    fn default() -> Self {
        SortBy::Relevance
    }
}

/// 插件搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPluginsResult {
    pub plugins: Vec<MarketPlugin>,
    pub total: i32,
    pub page: i32,
    pub per_page: i32,
    pub has_more: bool,
}

/// 插件分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCategory {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub plugin_count: i32,
}

/// 市场客户端
pub struct MarketClient {
    http_client: reqwest::Client,
    base_url: String,
    use_mock: bool,
}

impl MarketClient {
    /// 创建新的市场客户端
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_url: MARKET_API_BASE.to_string(),
            use_mock: true, // 默认使用模拟数据
        }
    }

    /// 创建使用真实API的客户端
    pub fn with_real_api() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_url: MARKET_API_BASE.to_string(),
            use_mock: false,
        }
    }

    /// 设置是否使用模拟数据
    pub fn set_mock(
        &mut self,
        use_mock: bool,
    ) {
        self.use_mock = use_mock;
    }

    /// 搜索插件
    pub async fn search_plugins(
        &self,
        req: SearchPluginsRequest,
    ) -> Result<SearchPluginsResult, AppError> {
        if self.use_mock {
            return self.mock_search_plugins(req).await;
        }

        let mut url = format!("{}/plugins/search", self.base_url);
        let mut params = vec![];

        if let Some(query) = &req.query {
            params.push(format!("q={}", urlencoding::encode(query)));
        }
        if let Some(category) = &req.category {
            params.push(format!("category={}", category));
        }
        if let Some(sort_by) = &req.sort_by {
            params.push(format!("sort={}", serde_json::to_string(sort_by).unwrap()));
        }
        params.push(format!("page={}", req.page.unwrap_or(1)));
        params.push(format!("per_page={}", req.per_page.unwrap_or(20)));

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Network(NetworkError::ConnectionFailed(e.to_string())))?;

        if !response.status().is_success() {
            return Err(AppError::Network(NetworkError::HttpError {
                code: response.status().as_u16(),
                message: response.status().to_string(),
            }));
        }

        let result = response
            .json::<SearchPluginsResult>()
            .await
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        Ok(result)
    }

    /// 获取插件详情
    pub async fn get_plugin_details(
        &self,
        id: &str,
    ) -> Result<MarketPlugin, AppError> {
        if self.use_mock {
            return self.mock_get_plugin_details(id).await;
        }

        let url = format!("{}/plugins/{}", self.base_url, id);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Network(NetworkError::ConnectionFailed(e.to_string())))?;

        if !response.status().is_success() {
            return Err(AppError::Network(NetworkError::HttpError {
                code: response.status().as_u16(),
                message: response.status().to_string(),
            }));
        }

        let plugin = response
            .json::<MarketPlugin>()
            .await
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        Ok(plugin)
    }

    /// 下载插件
    pub async fn download_plugin(
        &self,
        id: &str,
    ) -> Result<Vec<u8>, AppError> {
        if self.use_mock {
            return self.mock_download_plugin(id).await;
        }

        let plugin = self.get_plugin_details(id).await?;

        let response = self
            .http_client
            .get(&plugin.download_url)
            .send()
            .await
            .map_err(|e| AppError::Network(NetworkError::ConnectionFailed(e.to_string())))?;

        if !response.status().is_success() {
            return Err(AppError::Network(NetworkError::HttpError {
                code: response.status().as_u16(),
                message: response.status().to_string(),
            }));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::Network(NetworkError::ConnectionFailed(e.to_string())))?;

        Ok(bytes.to_vec())
    }

    /// 获取所有分类
    pub async fn get_categories(&self) -> Result<Vec<PluginCategory>, AppError> {
        if self.use_mock {
            return self.mock_get_categories().await;
        }

        let url = format!("{}/categories", self.base_url);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Network(NetworkError::ConnectionFailed(e.to_string())))?;

        if !response.status().is_success() {
            return Err(AppError::Network(NetworkError::HttpError {
                code: response.status().as_u16(),
                message: response.status().to_string(),
            }));
        }

        let categories = response
            .json::<Vec<PluginCategory>>()
            .await
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        Ok(categories)
    }

    /// 获取热门插件
    pub async fn get_popular_plugins(
        &self,
        limit: i32,
    ) -> Result<Vec<MarketPlugin>, AppError> {
        if self.use_mock {
            return self.mock_get_popular_plugins(limit).await;
        }

        let url = format!("{}/plugins/popular?limit={}", self.base_url, limit);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Network(NetworkError::ConnectionFailed(e.to_string())))?;

        if !response.status().is_success() {
            return Err(AppError::Network(NetworkError::HttpError {
                code: response.status().as_u16(),
                message: response.status().to_string(),
            }));
        }

        let plugins = response
            .json::<Vec<MarketPlugin>>()
            .await
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        Ok(plugins)
    }

    /// 获取最新插件
    pub async fn get_latest_plugins(
        &self,
        limit: i32,
    ) -> Result<Vec<MarketPlugin>, AppError> {
        if self.use_mock {
            return self.mock_get_latest_plugins(limit).await;
        }

        let url = format!("{}/plugins/latest?limit={}", self.base_url, limit);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Network(NetworkError::ConnectionFailed(e.to_string())))?;

        if !response.status().is_success() {
            return Err(AppError::Network(NetworkError::HttpError {
                code: response.status().as_u16(),
                message: response.status().to_string(),
            }));
        }

        let plugins = response
            .json::<Vec<MarketPlugin>>()
            .await
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        Ok(plugins)
    }

    // ==================== 模拟实现 ====================

    async fn mock_search_plugins(
        &self,
        req: SearchPluginsRequest,
    ) -> Result<SearchPluginsResult, AppError> {
        // 生成模拟数据
        let all_plugins = self.generate_mock_plugins();

        // 过滤
        let mut filtered: Vec<MarketPlugin> = all_plugins;

        if let Some(query) = &req.query {
            let query_lower = query.to_lowercase();
            filtered = filtered
                .into_iter()
                .filter(|p| {
                    p.name.to_lowercase().contains(&query_lower)
                        || p.description
                            .as_ref()
                            .map(|d| d.to_lowercase().contains(&query_lower))
                            .unwrap_or(false)
                })
                .collect();
        }

        if let Some(category) = &req.category {
            filtered = filtered
                .into_iter()
                .filter(|p| p.categories.contains(category))
                .collect();
        }

        // 排序
        let sort_by = req.sort_by.unwrap_or_default();
        match sort_by {
            SortBy::Downloads => {
                filtered.sort_by(|a, b| b.downloads.cmp(&a.downloads));
            }
            SortBy::Rating => {
                filtered.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap());
            }
            SortBy::CreatedAt => {
                filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
            SortBy::UpdatedAt => {
                filtered.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            }
            _ => {} // Relevance - 保持默认顺序
        }

        let total = filtered.len() as i32;
        let page = req.page.unwrap_or(1);
        let per_page = req.per_page.unwrap_or(20);
        let start = ((page - 1) * per_page) as usize;
        let end = (start + per_page as usize).min(filtered.len());

        let plugins = if start < filtered.len() {
            filtered[start..end].to_vec()
        } else {
            vec![]
        };

        Ok(SearchPluginsResult {
            plugins,
            total,
            page,
            per_page,
            has_more: end < filtered.len(),
        })
    }

    async fn mock_get_plugin_details(
        &self,
        id: &str,
    ) -> Result<MarketPlugin, AppError> {
        let plugins = self.generate_mock_plugins();
        plugins
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| AppError::Network(NetworkError::HttpError {
                code: 404,
                message: "Plugin not found".to_string(),
            }))
    }

    async fn mock_download_plugin(
        &self,
        _id: &str,
    ) -> Result<Vec<u8>, AppError> {
        // 返回模拟的插件包数据（一个空的tar.gz）
        // 在实际场景中，这里应该返回真实的插件包
        Ok(vec![0u8; 1024]) // 模拟1KB数据
    }

    async fn mock_get_categories(&self) -> Result<Vec<PluginCategory>, AppError> {
        Ok(vec![
            PluginCategory {
                id: "productivity".to_string(),
                name: "生产力".to_string(),
                description: Some("提高工作效率的插件".to_string()),
                icon: Some("briefcase".to_string()),
                plugin_count: 15,
            },
            PluginCategory {
                id: "ai-tools".to_string(),
                name: "AI工具".to_string(),
                description: Some("AI辅助功能插件".to_string()),
                icon: Some("brain".to_string()),
                plugin_count: 23,
            },
            PluginCategory {
                id: "automation".to_string(),
                name: "自动化".to_string(),
                description: Some("自动化工作流插件".to_string()),
                icon: Some("zap".to_string()),
                plugin_count: 12,
            },
            PluginCategory {
                id: "integration".to_string(),
                name: "集成".to_string(),
                description: Some("第三方服务集成".to_string()),
                icon: Some("plug".to_string()),
                plugin_count: 18,
            },
            PluginCategory {
                id: "developer".to_string(),
                name: "开发者".to_string(),
                description: Some("开发者工具插件".to_string()),
                icon: Some("code".to_string()),
                plugin_count: 30,
            },
        ])
    }

    async fn mock_get_popular_plugins(
        &self,
        limit: i32,
    ) -> Result<Vec<MarketPlugin>, AppError> {
        let mut plugins = self.generate_mock_plugins();
        plugins.sort_by(|a, b| b.downloads.cmp(&a.downloads));
        plugins.truncate(limit as usize);
        Ok(plugins)
    }

    async fn mock_get_latest_plugins(
        &self,
        limit: i32,
    ) -> Result<Vec<MarketPlugin>, AppError> {
        let mut plugins = self.generate_mock_plugins();
        plugins.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        plugins.truncate(limit as usize);
        Ok(plugins)
    }

    fn generate_mock_plugins(&self) -> Vec<MarketPlugin> {
        vec![
            MarketPlugin {
                id: "smart-reply".to_string(),
                name: "智能回复".to_string(),
                version: "1.2.0".to_string(),
                description: Some("基于AI的智能邮件和消息回复建议".to_string()),
                author: Some("OpenClaw Team".to_string()),
                author_avatar: None,
                downloads: 15420,
                rating: 4.8,
                rating_count: 342,
                icon_url: None,
                download_url: format!("{}/plugins/smart-reply/download", MARKET_API_BASE),
                categories: vec!["ai-tools".to_string(), "productivity".to_string()],
                tags: vec!["ai".to_string(), "email".to_string(), "automation".to_string()],
                created_at: "2024-01-15T08:00:00Z".to_string(),
                updated_at: "2024-03-20T10:30:00Z".to_string(),
                size_bytes: 2457600,
                min_app_version: Some("0.1.0".to_string()),
                changelog: Some("- 改进回复质量\n- 支持更多邮件客户端".to_string()),
            },
            MarketPlugin {
                id: "code-assistant".to_string(),
                name: "代码助手".to_string(),
                version: "2.1.0".to_string(),
                description: Some("智能代码补全和代码审查建议".to_string()),
                author: Some("DevTools Inc".to_string()),
                author_avatar: None,
                downloads: 23100,
                rating: 4.9,
                rating_count: 567,
                icon_url: None,
                download_url: format!("{}/plugins/code-assistant/download", MARKET_API_BASE),
                categories: vec!["developer".to_string(), "ai-tools".to_string()],
                tags: vec!["code".to_string(), "ai".to_string(), "developer".to_string()],
                created_at: "2023-12-01T10:00:00Z".to_string(),
                updated_at: "2024-03-15T14:20:00Z".to_string(),
                size_bytes: 5120000,
                min_app_version: Some("0.1.0".to_string()),
                changelog: Some("- 新增Python支持\n- 优化性能".to_string()),
            },
            MarketPlugin {
                id: "workflow-automation".to_string(),
                name: "工作流自动化".to_string(),
                version: "1.5.0".to_string(),
                description: Some("创建自定义自动化工作流，简化重复任务".to_string()),
                author: Some("Automation Pro".to_string()),
                author_avatar: None,
                downloads: 8900,
                rating: 4.6,
                rating_count: 189,
                icon_url: None,
                download_url: format!("{}/plugins/workflow-automation/download", MARKET_API_BASE),
                categories: vec!["automation".to_string(), "productivity".to_string()],
                tags: vec!["workflow".to_string(), "automation".to_string(), "productivity".to_string()],
                created_at: "2024-02-10T09:00:00Z".to_string(),
                updated_at: "2024-03-18T11:00:00Z".to_string(),
                size_bytes: 1892000,
                min_app_version: Some("0.1.0".to_string()),
                changelog: Some("- 新增条件判断\n- 支持定时触发".to_string()),
            },
            MarketPlugin {
                id: "slack-integration".to_string(),
                name: "Slack集成".to_string(),
                version: "1.0.5".to_string(),
                description: Some("与Slack无缝集成，接收通知和发送消息".to_string()),
                author: Some("Integration Hub".to_string()),
                author_avatar: None,
                downloads: 6700,
                rating: 4.4,
                rating_count: 145,
                icon_url: None,
                download_url: format!("{}/plugins/slack-integration/download", MARKET_API_BASE),
                categories: vec!["integration".to_string()],
                tags: vec!["slack".to_string(), "messaging".to_string(), "notification".to_string()],
                created_at: "2024-01-20T12:00:00Z".to_string(),
                updated_at: "2024-03-10T16:45:00Z".to_string(),
                size_bytes: 980000,
                min_app_version: Some("0.1.0".to_string()),
                changelog: Some("- 修复连接问题\n- 优化消息格式".to_string()),
            },
            MarketPlugin {
                id: "github-tools".to_string(),
                name: "GitHub工具集".to_string(),
                version: "1.3.0".to_string(),
                description: Some("GitHub仓库管理、PR审查和Issue跟踪".to_string()),
                author: Some("GitHub Tools".to_string()),
                author_avatar: None,
                downloads: 18900,
                rating: 4.7,
                rating_count: 423,
                icon_url: None,
                download_url: format!("{}/plugins/github-tools/download", MARKET_API_BASE),
                categories: vec!["developer".to_string(), "integration".to_string()],
                tags: vec!["github".to_string(), "git".to_string(), "developer".to_string()],
                created_at: "2023-11-15T08:30:00Z".to_string(),
                updated_at: "2024-03-22T09:15:00Z".to_string(),
                size_bytes: 3245000,
                min_app_version: Some("0.1.0".to_string()),
                changelog: Some("- 新增PR模板支持\n- 优化审查流程".to_string()),
            },
            MarketPlugin {
                id: "meeting-notes".to_string(),
                name: "会议记录助手".to_string(),
                version: "1.1.0".to_string(),
                description: Some("自动转录会议内容并生成摘要".to_string()),
                author: Some("MeetingAI".to_string()),
                author_avatar: None,
                downloads: 11200,
                rating: 4.5,
                rating_count: 267,
                icon_url: None,
                download_url: format!("{}/plugins/meeting-notes/download", MARKET_API_BASE),
                categories: vec!["productivity".to_string(), "ai-tools".to_string()],
                tags: vec!["meeting".to_string(), "transcription".to_string(), "ai".to_string()],
                created_at: "2024-01-05T14:00:00Z".to_string(),
                updated_at: "2024-03-12T10:30:00Z".to_string(),
                size_bytes: 4567000,
                min_app_version: Some("0.1.0".to_string()),
                changelog: Some("- 支持更多语言\n- 改进摘要质量".to_string()),
            },
            MarketPlugin {
                id: "api-tester".to_string(),
                name: "API测试工具".to_string(),
                version: "2.0.0".to_string(),
                description: Some("快速测试和调试REST API".to_string()),
                author: Some("DevTools Inc".to_string()),
                author_avatar: None,
                downloads: 14500,
                rating: 4.8,
                rating_count: 389,
                icon_url: None,
                download_url: format!("{}/plugins/api-tester/download", MARKET_API_BASE),
                categories: vec!["developer".to_string()],
                tags: vec!["api".to_string(), "testing".to_string(), "developer".to_string()],
                created_at: "2023-10-20T09:00:00Z".to_string(),
                updated_at: "2024-03-25T11:00:00Z".to_string(),
                size_bytes: 2100000,
                min_app_version: Some("0.1.0".to_string()),
                changelog: Some("- 全新UI设计\n- 支持GraphQL".to_string()),
            },
            MarketPlugin {
                id: "document-parser".to_string(),
                name: "文档解析器".to_string(),
                version: "1.4.0".to_string(),
                description: Some("解析PDF、Word等文档并提取关键信息".to_string()),
                author: Some("DocuTech".to_string()),
                author_avatar: None,
                downloads: 7800,
                rating: 4.3,
                rating_count: 198,
                icon_url: None,
                download_url: format!("{}/plugins/document-parser/download", MARKET_API_BASE),
                categories: vec!["productivity".to_string(), "ai-tools".to_string()],
                tags: vec!["document".to_string(), "pdf".to_string(), "parser".to_string()],
                created_at: "2024-02-01T11:00:00Z".to_string(),
                updated_at: "2024-03-20T15:30:00Z".to_string(),
                size_bytes: 6789000,
                min_app_version: Some("0.1.0".to_string()),
                changelog: Some("- 支持更多文档格式\n- 优化解析速度".to_string()),
            },
        ]
    }
}

impl Default for MarketClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_search_plugins() {
        let client = MarketClient::new();

        let req = SearchPluginsRequest {
            query: Some("AI".to_string()),
            category: None,
            tags: None,
            sort_by: Some(SortBy::Downloads),
            page: Some(1),
            per_page: Some(10),
        };

        let result = client.search_plugins(req).await.unwrap();
        assert!(!result.plugins.is_empty());
        assert!(result.total > 0);
    }

    #[tokio::test]
    async fn test_mock_get_plugin_details() {
        let client = MarketClient::new();

        let plugin = client.get_plugin_details("smart-reply").await.unwrap();
        assert_eq!(plugin.id, "smart-reply");
        assert_eq!(plugin.name, "智能回复");
    }

    #[tokio::test]
    async fn test_mock_get_categories() {
        let client = MarketClient::new();

        let categories = client.get_categories().await.unwrap();
        assert!(!categories.is_empty());
        assert!(categories.iter().any(|c| c.id == "ai-tools"));
    }

    #[tokio::test]
    async fn test_mock_get_popular_plugins() {
        let client = MarketClient::new();

        let plugins = client.get_popular_plugins(5).await.unwrap();
        assert_eq!(plugins.len(), 5);

        // 验证按下载量排序
        for i in 1..plugins.len() {
            assert!(plugins[i - 1].downloads >= plugins[i].downloads);
        }
    }
}
