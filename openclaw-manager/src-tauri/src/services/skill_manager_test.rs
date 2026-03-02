//! 技能管理器测试

#[cfg(test)]
mod tests {
    use crate::models::skill::{
        HookType, InstalledSkill, InstallSkillRequest, Skill, SkillCategory, SkillHook,
        UpdateSkillConfigRequest,
    };
    use crate::services::skill_manager::{get_mock_skills, SkillManager};

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

    #[test]
    fn test_mock_skills() {
        let skills = get_mock_skills();
        assert!(!skills.is_empty());
        assert_eq!(skills[0].id, "code-assistant");

        // 检查是否有配置schema的技能
        let with_config = skills.iter().filter(|s| s.has_config()).count();
        assert!(with_config > 0);
    }

    #[tokio::test]
    async fn test_skill_manager_creation() {
        // 使用临时目录测试
        let manager = SkillManager::new();
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_skill_install_and_uninstall() {
        let manager = SkillManager::new().unwrap();

        // 安装技能
        let request = InstallSkillRequest {
            skill_id: "code-assistant".to_string(),
            version: None,
        };

        let result = manager.install_skill(request).await;
        assert!(result.is_ok());

        let installed = result.unwrap();
        assert_eq!(installed.id(), "code-assistant");
        assert!(installed.is_enabled);

        // 检查是否已安装
        assert!(manager.is_installed("code-assistant").unwrap());

        // 获取所有技能
        let all_skills = manager.get_all_skills().unwrap();
        assert_eq!(all_skills.len(), 1);

        // 卸载技能
        let uninstall_result = manager.uninstall_skill("code-assistant");
        assert!(uninstall_result.is_ok());

        // 检查是否已卸载
        assert!(!manager.is_installed("code-assistant").unwrap());
    }

    #[tokio::test]
    async fn test_skill_toggle() {
        let manager = SkillManager::new().unwrap();

        // 安装技能
        let request = InstallSkillRequest {
            skill_id: "code-assistant".to_string(),
            version: None,
        };
        manager.install_skill(request).await.unwrap();

        // 禁用技能
        let disabled = manager.disable_skill("code-assistant").unwrap();
        assert!(!disabled.is_enabled);

        // 启用技能
        let enabled = manager.enable_skill("code-assistant").unwrap();
        assert!(enabled.is_enabled);

        // 切换技能状态
        let toggled = manager.toggle_skill("code-assistant", false).unwrap();
        assert!(!toggled.is_enabled);

        // 清理
        manager.uninstall_skill("code-assistant").unwrap();
    }

    #[tokio::test]
    async fn test_skill_config() {
        let manager = SkillManager::new().unwrap();

        // 安装有配置的技能
        let request = InstallSkillRequest {
            skill_id: "code-assistant".to_string(),
            version: None,
        };
        manager.install_skill(request).await.unwrap();

        // 获取配置
        let config = manager.get_skill_config("code-assistant").unwrap();
        assert!(config.is_object());

        // 更新配置
        let new_config = serde_json::json!({
            "language": "python",
            "style_guide": "microsoft",
            "max_suggestions": 10
        });

        let update_request = UpdateSkillConfigRequest {
            skill_id: "code-assistant".to_string(),
            config: new_config.clone(),
        };

        let updated = manager.update_skill_config(update_request).unwrap();
        assert_eq!(updated.config, new_config);

        // 验证配置已更新
        let saved_config = manager.get_skill_config("code-assistant").unwrap();
        assert_eq!(saved_config["language"], "python");

        // 清理
        manager.uninstall_skill("code-assistant").unwrap();
    }

    #[tokio::test]
    async fn test_skill_search() {
        let manager = SkillManager::new().unwrap();

        // 安装多个技能
        let skills_to_install = vec!["code-assistant", "writing-assistant", "data-analyzer"];
        for skill_id in &skills_to_install {
            let request = InstallSkillRequest {
                skill_id: skill_id.to_string(),
                version: None,
            };
            manager.install_skill(request).await.unwrap();
        }

        // 搜索技能
        let results = manager.search_installed_skills("code").unwrap();
        assert!(results.iter().any(|s| s.id() == "code-assistant"));

        let results = manager.search_installed_skills("writing").unwrap();
        assert!(results.iter().any(|s| s.id() == "writing-assistant"));

        // 按分类获取
        let programming_skills = manager.get_skills_by_category("programming").unwrap();
        assert!(programming_skills.iter().any(|s| s.id() == "code-assistant"));

        // 清理
        for skill_id in &skills_to_install {
            manager.uninstall_skill(skill_id).unwrap();
        }
    }

    #[test]
    fn test_skill_categories() {
        let manager = SkillManager::default();
        let categories = manager.get_categories();

        assert!(!categories.is_empty());
        assert!(categories.iter().any(|c| c.id == "all"));
        assert!(categories.iter().any(|c| c.id == "programming"));
        assert!(categories.iter().any(|c| c.id == "writing"));
    }
}
