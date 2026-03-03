//! 技能市场服务
//!
//! 提供技能市场的API客户端，用于搜索、获取技能信息

use crate::errors::app_error::{AppError, NetworkError};
use crate::models::skill::{Skill, SkillCategory, SkillMarketItem, SkillSearchResult};
use crate::services::skill_manager::get_mock_skills;
use tracing::debug;

/// 技能市场客户端
#[derive(Debug, Clone)]
pub struct SkillMarket {
    /// API基础URL
    base_url: String,
    /// 是否使用模拟数据（开发模式）
    use_mock: bool,
}

impl SkillMarket {
    /// 创建新的市场客户端
    pub fn new() -> Self {
        Self {
            base_url: "https://market.openclaw.ai/api/v1".to_string(),
            use_mock: true, // 开发阶段使用模拟数据
        }
    }

    /// 创建带自定义配置的客户端
    pub fn with_config(base_url: impl Into<String>, use_mock: bool) -> Self {
        Self {
            base_url: base_url.into(),
            use_mock,
        }
    }

    /// 搜索技能
    pub async fn search_skills(
        &self,
        query: Option<&str>,
        category: Option<&str>,
        page: usize,
        per_page: usize,
    ) -> Result<SkillSearchResult, AppError> {
        debug!(
            "搜索技能: query={:?}, category={:?}, page={}, per_page={}",
            query, category, page, per_page
        );

        if self.use_mock {
            return self.mock_search_skills(query, category, page, per_page).await;
        }

        // 实际API调用
        let client = reqwest::Client::new();
        let mut request = client.get(format!("{}/skills", self.base_url));

        if let Some(q) = query {
            request = request.query(&[("q", q)]);
        }
        if let Some(c) = category {
            request = request.query(&[("category", c)]);
        }

        request = request
            .query(&[("page", page.to_string())])
            .query(&[("per_page", per_page.to_string())]);

        let response = request
            .send()
            .await
            .map_err(|e| AppError::Network(NetworkError::ConnectionFailed(format!("搜索技能失败: {}", e))))?;

        if !response.status().is_success() {
            return Err(AppError::Network(NetworkError::HttpError {
                code: response.status().as_u16(),
                message: "搜索技能API返回错误".to_string(),
            }));
        }

        let result: SkillSearchResult = response
            .json()
            .await
            .map_err(|e| AppError::Serialization(format!("解析技能搜索结果失败: {}", e)))?;

        Ok(result)
    }

    /// 获取技能详情
    pub async fn get_skill_detail(&self, skill_id: &str) -> Result<Skill, AppError> {
        debug!("获取技能详情: {}", skill_id);

        if self.use_mock {
            return self.mock_get_skill_detail(skill_id).await;
        }

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/skills/{}", self.base_url, skill_id))
            .send()
            .await
            .map_err(|e| AppError::Network(NetworkError::ConnectionFailed(format!("获取技能详情失败: {}", e))))?;

        if !response.status().is_success() {
            return Err(AppError::Network(NetworkError::HttpError {
                code: response.status().as_u16(),
                message: "获取技能详情API返回错误".to_string(),
            }));
        }

        let skill: Skill = response
            .json()
            .await
            .map_err(|e| AppError::Serialization(format!("解析技能详情失败: {}", e)))?;

        Ok(skill)
    }

    /// 获取热门技能
    pub async fn get_popular_skills(&self, limit: usize) -> Result<Vec<SkillMarketItem>, AppError> {
        debug!("获取热门技能: limit={}", limit);

        if self.use_mock {
            return self.mock_get_popular_skills(limit).await;
        }

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/skills/popular", self.base_url))
            .query(&[("limit", limit.to_string())])
            .send()
            .await
            .map_err(|e| AppError::Network(NetworkError::ConnectionFailed(format!("获取热门技能失败: {}", e))))?;

        if !response.status().is_success() {
            return Err(AppError::Network(NetworkError::HttpError {
                code: response.status().as_u16(),
                message: "获取热门技能API返回错误".to_string(),
            }));
        }

        let skills: Vec<SkillMarketItem> = response
            .json()
            .await
            .map_err(|e| AppError::Serialization(format!("解析热门技能失败: {}", e)))?;

        Ok(skills)
    }

    /// 获取最新技能
    pub async fn get_latest_skills(&self, limit: usize) -> Result<Vec<SkillMarketItem>, AppError> {
        debug!("获取最新技能: limit={}", limit);

        if self.use_mock {
            return self.mock_get_latest_skills(limit).await;
        }

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/skills/latest", self.base_url))
            .query(&[("limit", limit.to_string())])
            .send()
            .await
            .map_err(|e| AppError::Network(NetworkError::ConnectionFailed(format!("获取最新技能失败: {}", e))))?;

        if !response.status().is_success() {
            return Err(AppError::Network(NetworkError::HttpError {
                code: response.status().as_u16(),
                message: "获取最新技能API返回错误".to_string(),
            }));
        }

        let skills: Vec<SkillMarketItem> = response
            .json()
            .await
            .map_err(|e| AppError::Serialization(format!("解析最新技能失败: {}", e)))?;

        Ok(skills)
    }

    /// 获取技能分类
    pub async fn get_categories(&self) -> Result<Vec<SkillCategory>, AppError> {
        debug!("获取技能分类");

        if self.use_mock {
            return Ok(SkillCategory::predefined());
        }

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/categories", self.base_url))
            .send()
            .await
            .map_err(|e| AppError::Network(NetworkError::ConnectionFailed(format!("获取技能分类失败: {}", e))))?;

        if !response.status().is_success() {
            return Err(AppError::Network(NetworkError::HttpError {
                code: response.status().as_u16(),
                message: "获取技能分类API返回错误".to_string(),
            }));
        }

        let categories: Vec<SkillCategory> = response
            .json()
            .await
            .map_err(|e| AppError::Serialization(format!("解析技能分类失败: {}", e)))?;

        Ok(categories)
    }

    /// 获取技能包下载URL
    pub async fn get_download_url(
        &self,
        skill_id: &str,
        version: Option<&str>,
    ) -> Result<String, AppError> {
        debug!("获取技能下载URL: {}@{:?}", skill_id, version);

        if self.use_mock {
            // 模拟下载URL
            return Ok(format!(
                "https://market.openclaw.ai/download/{}/{}",
                skill_id,
                version.unwrap_or("latest")
            ));
        }

        let client = reqwest::Client::new();
        let mut request = client.get(format!("{}/skills/{}/download", self.base_url, skill_id));

        if let Some(v) = version {
            request = request.query(&[("version", v)]);
        }

        let response = request
            .send()
            .await
            .map_err(|e| AppError::Network(NetworkError::ConnectionFailed(format!("获取下载URL失败: {}", e))))?;

        if !response.status().is_success() {
            return Err(AppError::Network(NetworkError::HttpError {
                code: response.status().as_u16(),
                message: "获取下载URL API返回错误".to_string(),
            }));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::Serialization(format!("解析下载URL失败: {}", e)))?;

        let url = result
            .get("url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| AppError::Serialization("下载URL格式错误".to_string()))?;

        Ok(url.to_string())
    }

    /// 检查技能更新
    pub async fn check_update(
        &self,
        skill_id: &str,
        current_version: &str,
    ) -> Result<Option<String>, AppError> {
        debug!("检查技能更新: {}@{}", skill_id, current_version);

        if self.use_mock {
            return self.mock_check_update(skill_id, current_version).await;
        }

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/skills/{}/check-update", self.base_url, skill_id))
            .query(&[("current_version", current_version)])
            .send()
            .await
            .map_err(|e| AppError::Network(NetworkError::ConnectionFailed(format!("检查技能更新失败: {}", e))))?;

        if !response.status().is_success() {
            return Err(AppError::Network(NetworkError::HttpError {
                code: response.status().as_u16(),
                message: "检查更新API返回错误".to_string(),
            }));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::Serialization(format!("解析更新检查结果失败: {}", e)))?;

        let has_update = result.get("has_update").and_then(|h| h.as_bool()).unwrap_or(false);

        if has_update {
            let latest = result
                .get("latest_version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Ok(latest)
        } else {
            Ok(None)
        }
    }

    // ============ 模拟数据方法 ============

    /// 模拟搜索技能
    async fn mock_search_skills(
        &self,
        query: Option<&str>,
        category: Option<&str>,
        page: usize,
        per_page: usize,
    ) -> Result<SkillSearchResult, AppError> {
        let all_skills = get_mock_skills();

        // 过滤
        let mut filtered: Vec<Skill> = all_skills;

        if let Some(q) = query {
            let q_lower = q.to_lowercase();
            filtered = filtered
                .into_iter()
                .filter(|s| {
                    s.name.to_lowercase().contains(&q_lower)
                        || s.description.to_lowercase().contains(&q_lower)
                        || s.tags.iter().any(|t| t.to_lowercase().contains(&q_lower))
                })
                .collect();
        }

        if let Some(c) = category {
            if c != "all" {
                filtered = filtered
                    .into_iter()
                    .filter(|s| s.categories.iter().any(|cat| cat == c))
                    .collect();
            }
        }

        let total = filtered.len();

        // 分页
        let start = (page - 1) * per_page;
        let end = (start + per_page).min(total);
        let skills = if start < total {
            filtered[start..end].to_vec()
        } else {
            vec![]
        };

        Ok(SkillSearchResult {
            skills,
            total,
            page,
            per_page,
            query: query.map(|s| s.to_string()),
            category: category.map(|s| s.to_string()),
        })
    }

    /// 模拟获取技能详情
    async fn mock_get_skill_detail(&self, skill_id: &str) -> Result<Skill, AppError> {
        let skills = get_mock_skills();
        skills
            .into_iter()
            .find(|s| s.id == skill_id)
            .ok_or_else(|| AppError::NotFound(format!("技能 '{}' 不存在", skill_id)))
    }

    /// 模拟获取热门技能
    async fn mock_get_popular_skills(
        &self,
        limit: usize,
    ) -> Result<Vec<SkillMarketItem>, AppError> {
        let skills = get_mock_skills();
        let mut items: Vec<SkillMarketItem> = skills
            .into_iter()
            .map(|s| SkillMarketItem {
                id: s.id.clone(),
                name: s.name.clone(),
                description: s.description.clone(),
                author: s.author.clone(),
                version: s.version.clone(),
                categories: s.categories.clone(),
                tags: s.tags.clone(),
                icon_url: s.icon_url.clone(),
                rating: s.rating,
                downloads: s.downloads,
                download_url: format!("https://market.openclaw.ai/download/{}", s.id),
                is_installed: false,
                has_update: false,
            })
            .collect();

        // 按下载量排序
        items.sort_by(|a, b| b.downloads.cmp(&a.downloads));
        items.truncate(limit);

        Ok(items)
    }

    /// 模拟获取最新技能
    async fn mock_get_latest_skills(
        &self,
        limit: usize,
    ) -> Result<Vec<SkillMarketItem>, AppError> {
        let skills = get_mock_skills();
        let mut items: Vec<SkillMarketItem> = skills
            .into_iter()
            .map(|s| SkillMarketItem {
                id: s.id.clone(),
                name: s.name.clone(),
                description: s.description.clone(),
                author: s.author.clone(),
                version: s.version.clone(),
                categories: s.categories.clone(),
                tags: s.tags.clone(),
                icon_url: s.icon_url.clone(),
                rating: s.rating,
                downloads: s.downloads,
                download_url: format!("https://market.openclaw.ai/download/{}", s.id),
                is_installed: false,
                has_update: false,
            })
            .collect();

        // 按更新时间排序（模拟）
        items.reverse();
        items.truncate(limit);

        Ok(items)
    }

    /// 模拟检查更新
    async fn mock_check_update(
        &self,
        skill_id: &str,
        current_version: &str,
    ) -> Result<Option<String>, AppError> {
        // 模拟：某些技能有新版本
        if skill_id == "writing-assistant" && current_version == "1.2.0" {
            return Ok(Some("1.3.0".to_string()));
        }
        if skill_id == "task-automation" && current_version == "2.0.0" {
            return Ok(Some("2.1.0".to_string()));
        }
        Ok(None)
    }
}

impl Default for SkillMarket {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_market_search() {
        let market = SkillMarket::with_config("", true);
        let result = market.search_skills(None, None, 1, 10).await;
        assert!(result.is_ok());

        let search_result = result.unwrap();
        assert!(!search_result.skills.is_empty());
    }

    #[tokio::test]
    async fn test_market_search_with_query() {
        let market = SkillMarket::with_config("", true);
        let result = market.search_skills(Some("code"), None, 1, 10).await;
        assert!(result.is_ok());

        let search_result = result.unwrap();
        assert!(!search_result.skills.is_empty());
        // Check that found skills have "code" in name, description, or tags
        assert!(search_result
            .skills
            .iter()
            .any(|s| {
                s.name.to_lowercase().contains("code") ||
                s.description.to_lowercase().contains("code") ||
                s.tags.iter().any(|t| t.to_lowercase().contains("code"))
            }));
    }

    #[tokio::test]
    async fn test_market_get_skill_detail() {
        let market = SkillMarket::with_config("", true);
        let result = market.get_skill_detail("code-assistant").await;
        assert!(result.is_ok());

        let skill = result.unwrap();
        assert_eq!(skill.id, "code-assistant");
    }

    #[tokio::test]
    async fn test_market_get_popular() {
        let market = SkillMarket::with_config("", true);
        let result = market.get_popular_skills(5).await;
        assert!(result.is_ok());

        let skills = result.unwrap();
        assert!(!skills.is_empty());
        assert!(skills.len() <= 5);
    }

    #[tokio::test]
    async fn test_market_check_update() {
        let market = SkillMarket::with_config("", true);

        // 有更新的情况
        let result = market.check_update("writing-assistant", "1.2.0").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("1.3.0".to_string()));

        // 无更新的情况
        let result = market.check_update("code-assistant", "1.0.0").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }
}
