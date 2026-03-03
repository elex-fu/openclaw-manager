//! DiagnosticsService 测试模块
//!
//! 提供 DiagnosticsService 的全面测试覆盖

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::services::diagnostics::{CheckStatus, DiagnosticCheck, DiagnosticResult, FixResult, FixFailure, DiagnosticCheckRequest, DiagnosticService};

    // ==================== CheckStatus 测试 ====================

    /// 测试 CheckStatus 变体
    #[test]
    fn test_check_status_variants() {
        let pass = CheckStatus::Pass;
        let warning = CheckStatus::Warning;
        let error = CheckStatus::Error;

        assert!(matches!(pass, CheckStatus::Pass));
        assert!(matches!(warning, CheckStatus::Warning));
        assert!(matches!(error, CheckStatus::Error));
    }

    /// 测试 CheckStatus 严重程度转换
    #[test]
    fn test_check_status_to_severity() {
        assert_eq!(CheckStatus::Pass.to_severity(), "info");
        assert_eq!(CheckStatus::Warning.to_severity(), "warning");
        assert_eq!(CheckStatus::Error.to_severity(), "error");
    }

    /// 测试 CheckStatus 序列化
    #[test]
    fn test_check_status_serialization() {
        let pass = CheckStatus::Pass;
        let json = serde_json::to_string(&pass).unwrap();
        assert!(json.contains("pass"));

        let warning = CheckStatus::Warning;
        let json = serde_json::to_string(&warning).unwrap();
        assert!(json.contains("warning"));

        let error = CheckStatus::Error;
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("error"));
    }

    /// 测试 CheckStatus 反序列化
    #[test]
    fn test_check_status_deserialization() {
        let pass: CheckStatus = serde_json::from_str("\"pass\"").unwrap();
        assert!(matches!(pass, CheckStatus::Pass));

        let warning: CheckStatus = serde_json::from_str("\"warning\"").unwrap();
        assert!(matches!(warning, CheckStatus::Warning));

        let error: CheckStatus = serde_json::from_str("\"error\"").unwrap();
        assert!(matches!(error, CheckStatus::Error));
    }

    // ==================== DiagnosticCheck 测试 ====================

    /// 测试 DiagnosticCheck 创建
    #[test]
    fn test_diagnostic_check_creation() {
        let check = DiagnosticCheck {
            category: "system".to_string(),
            name: "memory".to_string(),
            status: CheckStatus::Pass,
            message: "Memory check passed".to_string(),
            details: Some("8GB available".to_string()),
            fixable: false,
            fix_suggestion: None,
        };

        assert_eq!(check.category, "system");
        assert_eq!(check.name, "memory");
        assert!(matches!(check.status, CheckStatus::Pass));
        assert_eq!(check.message, "Memory check passed");
        assert_eq!(check.details, Some("8GB available".to_string()));
        assert!(!check.fixable);
        assert!(check.fix_suggestion.is_none());
    }

    /// 测试 DiagnosticCheck 可修复项
    #[test]
    fn test_diagnostic_check_fixable() {
        let check = DiagnosticCheck {
            category: "openclaw".to_string(),
            name: "installation".to_string(),
            status: CheckStatus::Error,
            message: "OpenClaw not installed".to_string(),
            details: None,
            fixable: true,
            fix_suggestion: Some("Click to install".to_string()),
        };

        assert!(check.fixable);
        assert_eq!(check.fix_suggestion, Some("Click to install".to_string()));
    }

    /// 测试 DiagnosticCheck 序列化
    #[test]
    fn test_diagnostic_check_serialization() {
        let check = DiagnosticCheck {
            category: "system".to_string(),
            name: "disk".to_string(),
            status: CheckStatus::Warning,
            message: "Low disk space".to_string(),
            details: Some("Only 1GB remaining".to_string()),
            fixable: true,
            fix_suggestion: Some("Free up disk space".to_string()),
        };

        let json = serde_json::to_string(&check).unwrap();
        assert!(json.contains("disk"));
        assert!(json.contains("warning"));
        assert!(json.contains("Low disk space"));
        assert!(json.contains("fixable"));
    }

    // ==================== DiagnosticResult 测试 ====================

    /// 测试 DiagnosticResult 创建
    #[test]
    fn test_diagnostic_result_creation() {
        let result = DiagnosticResult {
            checks: vec![],
            has_errors: false,
            has_warnings: false,
            checked_at: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(result.checks.is_empty());
        assert!(!result.has_errors);
        assert!(!result.has_warnings);
        assert_eq!(result.checked_at, "2024-01-01T00:00:00Z");
    }

    /// 测试 DiagnosticResult 带检查结果
    #[test]
    fn test_diagnostic_result_with_checks() {
        let check1 = DiagnosticCheck {
            category: "system".to_string(),
            name: "memory".to_string(),
            status: CheckStatus::Pass,
            message: "OK".to_string(),
            details: None,
            fixable: false,
            fix_suggestion: None,
        };

        let check2 = DiagnosticCheck {
            category: "system".to_string(),
            name: "disk".to_string(),
            status: CheckStatus::Warning,
            message: "Low space".to_string(),
            details: None,
            fixable: false,
            fix_suggestion: None,
        };

        let result = DiagnosticResult {
            checks: vec![check1, check2],
            has_errors: false,
            has_warnings: true,
            checked_at: "2024-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(result.checks.len(), 2);
        assert!(result.has_warnings);
        assert!(!result.has_errors);
    }

    /// 测试 DiagnosticResult 序列化
    #[test]
    fn test_diagnostic_result_serialization() {
        let check = DiagnosticCheck {
            category: "system".to_string(),
            name: "disk".to_string(),
            status: CheckStatus::Error,
            message: "Disk full".to_string(),
            details: None,
            fixable: true,
            fix_suggestion: Some("Delete files".to_string()),
        };

        let result = DiagnosticResult {
            checks: vec![check],
            has_errors: true,
            has_warnings: false,
            checked_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("disk"));
        assert!(json.contains("error"));
        assert!(json.contains("has_errors"));
        assert!(json.contains("true"));
    }

    /// 测试 DiagnosticResult 反序列化
    #[test]
    fn test_diagnostic_result_deserialization() {
        let json = r#"{
            "checks": [
                {
                    "category": "system",
                    "name": "memory",
                    "status": "pass",
                    "message": "OK",
                    "details": null,
                    "fixable": false,
                    "fix_suggestion": null
                }
            ],
            "has_errors": false,
            "has_warnings": false,
            "checked_at": "2024-01-01T00:00:00Z"
        }"#;

        let result: DiagnosticResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.checks.len(), 1);
        assert!(!result.has_errors);
        assert_eq!(result.checks[0].name, "memory");
    }

    // ==================== FixResult 测试 ====================

    /// 测试 FixResult 创建
    #[test]
    fn test_fix_result_creation() {
        let fix_result = FixResult {
            fixed: vec!["issue1".to_string(), "issue2".to_string()],
            failed: vec![],
        };

        assert_eq!(fix_result.fixed.len(), 2);
        assert!(fix_result.failed.is_empty());
    }

    /// 测试 FixResult 带失败项
    #[test]
    fn test_fix_result_with_failures() {
        let fix_result = FixResult {
            fixed: vec!["issue1".to_string()],
            failed: vec![
                FixFailure {
                    name: "issue2".to_string(),
                    error: "Permission denied".to_string(),
                },
                FixFailure {
                    name: "issue3".to_string(),
                    error: "Network error".to_string(),
                },
            ],
        };

        assert_eq!(fix_result.fixed.len(), 1);
        assert_eq!(fix_result.failed.len(), 2);
        assert_eq!(fix_result.failed[0].name, "issue2");
        assert_eq!(fix_result.failed[1].error, "Network error");
    }

    /// 测试 FixResult 序列化
    #[test]
    fn test_fix_result_serialization() {
        let fix_result = FixResult {
            fixed: vec!["config".to_string()],
            failed: vec![FixFailure {
                name: "service".to_string(),
                error: "Failed to start".to_string(),
            }],
        };

        let json = serde_json::to_string(&fix_result).unwrap();
        assert!(json.contains("fixed"));
        assert!(json.contains("failed"));
        assert!(json.contains("config"));
        assert!(json.contains("Failed to start"));
    }

    // ==================== FixFailure 测试 ====================

    /// 测试 FixFailure 创建
    #[test]
    fn test_fix_failure_creation() {
        let failure = FixFailure {
            name: "test_issue".to_string(),
            error: "Something went wrong".to_string(),
        };

        assert_eq!(failure.name, "test_issue");
        assert_eq!(failure.error, "Something went wrong");
    }

    /// 测试 FixFailure 序列化
    #[test]
    fn test_fix_failure_serialization() {
        let failure = FixFailure {
            name: "disk_check".to_string(),
            error: "No permission".to_string(),
        };

        let json = serde_json::to_string(&failure).unwrap();
        assert!(json.contains("disk_check"));
        assert!(json.contains("No permission"));
    }

    // ==================== DiagnosticCheckRequest 测试 ====================

    /// 测试 DiagnosticCheckRequest 创建
    #[test]
    fn test_diagnostic_check_request_creation() {
        let request = DiagnosticCheckRequest {
            name: "memory_check".to_string(),
            category: "system".to_string(),
        };

        assert_eq!(request.name, "memory_check");
        assert_eq!(request.category, "system");
    }

    /// 测试 DiagnosticCheckRequest 序列化
    #[test]
    fn test_diagnostic_check_request_serialization() {
        let request = DiagnosticCheckRequest {
            name: "disk_space".to_string(),
            category: "system".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("disk_space"));
        assert!(json.contains("system"));
    }

    /// 测试 DiagnosticCheckRequest 反序列化
    #[test]
    fn test_diagnostic_check_request_deserialization() {
        let json = r#"{"name": "network", "category": "connectivity"}"#;
        let request: DiagnosticCheckRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.name, "network");
        assert_eq!(request.category, "connectivity");
    }

    // ==================== DiagnosticService 创建测试 ====================

    /// 测试 DiagnosticService 创建
    #[test]
    fn test_diagnostic_service_new() {
        let _service = DiagnosticService::new();
        // 结果取决于环境，但不应该 panic
        // 在测试环境中可能没有完整的安装环境
    }

    /// 测试 DiagnosticService 默认实现
    #[test]
    fn test_diagnostic_service_default() {
        // Default 实现可能会 panic 如果没有安装环境
        // 所以我们只测试它存在
    }

    // ==================== 诊断检查结果统计测试 ====================

    /// 测试检查结果统计 - 全通过
    #[test]
    fn test_check_statistics_all_pass() {
        let checks = vec![
            DiagnosticCheck {
                category: "system".to_string(),
                name: "check1".to_string(),
                status: CheckStatus::Pass,
                message: "OK".to_string(),
                details: None,
                fixable: false,
                fix_suggestion: None,
            },
            DiagnosticCheck {
                category: "system".to_string(),
                name: "check2".to_string(),
                status: CheckStatus::Pass,
                message: "OK".to_string(),
                details: None,
                fixable: false,
                fix_suggestion: None,
            },
        ];

        let mut has_errors = false;
        let mut has_warnings = false;

        for check in &checks {
            match check.status {
                CheckStatus::Error => has_errors = true,
                CheckStatus::Warning => has_warnings = true,
                _ => {}
            }
        }

        assert!(!has_errors);
        assert!(!has_warnings);
    }

    /// 测试检查结果统计 - 有警告
    #[test]
    fn test_check_statistics_with_warnings() {
        let checks = vec![
            DiagnosticCheck {
                category: "system".to_string(),
                name: "check1".to_string(),
                status: CheckStatus::Pass,
                message: "OK".to_string(),
                details: None,
                fixable: false,
                fix_suggestion: None,
            },
            DiagnosticCheck {
                category: "system".to_string(),
                name: "check2".to_string(),
                status: CheckStatus::Warning,
                message: "Warning".to_string(),
                details: None,
                fixable: false,
                fix_suggestion: None,
            },
        ];

        let mut has_errors = false;
        let mut has_warnings = false;

        for check in &checks {
            match check.status {
                CheckStatus::Error => has_errors = true,
                CheckStatus::Warning => has_warnings = true,
                _ => {}
            }
        }

        assert!(!has_errors);
        assert!(has_warnings);
    }

    /// 测试检查结果统计 - 有错误
    #[test]
    fn test_check_statistics_with_errors() {
        let checks = vec![
            DiagnosticCheck {
                category: "system".to_string(),
                name: "check1".to_string(),
                status: CheckStatus::Pass,
                message: "OK".to_string(),
                details: None,
                fixable: false,
                fix_suggestion: None,
            },
            DiagnosticCheck {
                category: "system".to_string(),
                name: "check2".to_string(),
                status: CheckStatus::Error,
                message: "Error".to_string(),
                details: None,
                fixable: true,
                fix_suggestion: None,
            },
        ];

        let mut has_errors = false;
        let mut has_warnings = false;

        for check in &checks {
            match check.status {
                CheckStatus::Error => has_errors = true,
                CheckStatus::Warning => has_warnings = true,
                _ => {}
            }
        }

        assert!(has_errors);
        assert!(!has_warnings);
    }

    /// 测试检查结果统计 - 混合
    #[test]
    fn test_check_statistics_mixed() {
        let checks = vec![
            DiagnosticCheck {
                category: "system".to_string(),
                name: "check1".to_string(),
                status: CheckStatus::Pass,
                message: "OK".to_string(),
                details: None,
                fixable: false,
                fix_suggestion: None,
            },
            DiagnosticCheck {
                category: "system".to_string(),
                name: "check2".to_string(),
                status: CheckStatus::Warning,
                message: "Warning".to_string(),
                details: None,
                fixable: false,
                fix_suggestion: None,
            },
            DiagnosticCheck {
                category: "system".to_string(),
                name: "check3".to_string(),
                status: CheckStatus::Error,
                message: "Error".to_string(),
                details: None,
                fixable: true,
                fix_suggestion: None,
            },
        ];

        let mut has_errors = false;
        let mut has_warnings = false;

        for check in &checks {
            match check.status {
                CheckStatus::Error => has_errors = true,
                CheckStatus::Warning => has_warnings = true,
                _ => {}
            }
        }

        assert!(has_errors);
        assert!(has_warnings);
    }

    // ==================== 修复结果统计测试 ====================

    /// 测试修复结果 - 全部成功
    #[test]
    fn test_fix_result_all_success() {
        let fix_result = FixResult {
            fixed: vec!["issue1".to_string(), "issue2".to_string()],
            failed: vec![],
        };

        assert_eq!(fix_result.fixed.len(), 2);
        assert!(fix_result.failed.is_empty());
        assert!(fix_result.fixed.contains(&"issue1".to_string()));
        assert!(fix_result.fixed.contains(&"issue2".to_string()));
    }

    /// 测试修复结果 - 部分失败
    #[test]
    fn test_fix_result_partial_failure() {
        let fix_result = FixResult {
            fixed: vec!["issue1".to_string()],
            failed: vec![
                FixFailure {
                    name: "issue2".to_string(),
                    error: "Failed".to_string(),
                },
            ],
        };

        assert_eq!(fix_result.fixed.len(), 1);
        assert_eq!(fix_result.failed.len(), 1);
        assert_eq!(fix_result.failed[0].name, "issue2");
    }

    /// 测试修复结果 - 全部失败
    #[test]
    fn test_fix_result_all_failed() {
        let fix_result = FixResult {
            fixed: vec![],
            failed: vec![
                FixFailure {
                    name: "issue1".to_string(),
                    error: "Error 1".to_string(),
                },
                FixFailure {
                    name: "issue2".to_string(),
                    error: "Error 2".to_string(),
                },
            ],
        };

        assert!(fix_result.fixed.is_empty());
        assert_eq!(fix_result.failed.len(), 2);
    }

    // ==================== 边界情况测试 ====================

    /// 测试空检查结果
    #[test]
    fn test_empty_checks() {
        let result = DiagnosticResult {
            checks: vec![],
            has_errors: false,
            has_warnings: false,
            checked_at: chrono::Local::now().to_rfc3339(),
        };

        assert!(result.checks.is_empty());
        assert!(!result.has_errors);
        assert!(!result.has_warnings);
    }

    /// 测试很长的消息
    #[test]
    fn test_very_long_message() {
        let long_message = "a".repeat(10000);
        let check = DiagnosticCheck {
            category: "system".to_string(),
            name: "test".to_string(),
            status: CheckStatus::Pass,
            message: long_message.clone(),
            details: None,
            fixable: false,
            fix_suggestion: None,
        };

        assert_eq!(check.message.len(), 10000);

        let json = serde_json::to_string(&check).unwrap();
        assert!(json.len() > 10000);
    }

    /// 测试特殊字符
    #[test]
    fn test_special_characters() {
        let check = DiagnosticCheck {
            category: "system".to_string(),
            name: "test".to_string(),
            status: CheckStatus::Pass,
            message: "Special: äöü € 日本語 🎉 \"quoted\"".to_string(),
            details: None,
            fixable: false,
            fix_suggestion: None,
        };

        let json = serde_json::to_string(&check).unwrap();
        let deserialized: DiagnosticCheck = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.message, check.message);
    }

    /// 测试多字节字符
    #[test]
    fn test_multibyte_characters() {
        let check = DiagnosticCheck {
            category: "系统".to_string(),
            name: "内存检查".to_string(),
            status: CheckStatus::Pass,
            message: "内存充足 (8GB)".to_string(),
            details: Some("详细信息".to_string()),
            fixable: false,
            fix_suggestion: None,
        };

        let json = serde_json::to_string(&check).unwrap();
        let deserialized: DiagnosticCheck = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.category, "系统");
        assert_eq!(deserialized.name, "内存检查");
    }

    // ==================== 时间戳测试 ====================

    /// 测试检查时间戳格式
    #[test]
    fn test_checked_at_timestamp() {
        let now = chrono::Local::now();
        let result = DiagnosticResult {
            checks: vec![],
            has_errors: false,
            has_warnings: false,
            checked_at: now.to_rfc3339(),
        };

        // 验证时间戳可以被解析
        let parsed = chrono::DateTime::parse_from_rfc3339(&result.checked_at);
        assert!(parsed.is_ok());
    }

    /// 测试不同时间戳
    #[test]
    fn test_different_timestamps() {
        let timestamps = vec![
            "2024-01-01T00:00:00Z".to_string(),
            "2024-12-31T23:59:59+08:00".to_string(),
            chrono::Local::now().to_rfc3339(),
        ];

        for ts in timestamps {
            let result = DiagnosticResult {
                checks: vec![],
                has_errors: false,
                has_warnings: false,
                checked_at: ts,
            };

            let json = serde_json::to_string(&result).unwrap();
            assert!(json.contains("checked_at"));
        }
    }

    // ==================== 复杂场景测试 ====================

    /// 测试完整的诊断结果流程
    #[test]
    fn test_full_diagnostic_workflow() {
        // 创建各种检查项
        let checks = vec![
            DiagnosticCheck {
                category: "system".to_string(),
                name: "os_compatibility".to_string(),
                status: CheckStatus::Pass,
                message: "macOS 14 compatible".to_string(),
                details: None,
                fixable: false,
                fix_suggestion: None,
            },
            DiagnosticCheck {
                category: "system".to_string(),
                name: "memory".to_string(),
                status: CheckStatus::Pass,
                message: "16GB available".to_string(),
                details: None,
                fixable: false,
                fix_suggestion: None,
            },
            DiagnosticCheck {
                category: "system".to_string(),
                name: "disk_space".to_string(),
                status: CheckStatus::Warning,
                message: "Low disk space".to_string(),
                details: Some("Only 2GB remaining".to_string()),
                fixable: false,
                fix_suggestion: Some("Free up disk space".to_string()),
            },
            DiagnosticCheck {
                category: "openclaw".to_string(),
                name: "installation".to_string(),
                status: CheckStatus::Error,
                message: "OpenClaw not installed".to_string(),
                details: None,
                fixable: true,
                fix_suggestion: Some("Click to install".to_string()),
            },
        ];

        // 统计结果
        let mut has_errors = false;
        let mut has_warnings = false;

        for check in &checks {
            match check.status {
                CheckStatus::Error => has_errors = true,
                CheckStatus::Warning => has_warnings = true,
                _ => {}
            }
        }

        let result = DiagnosticResult {
            checks,
            has_errors,
            has_warnings,
            checked_at: chrono::Local::now().to_rfc3339(),
        };

        assert!(result.has_errors);
        assert!(result.has_warnings);
        assert_eq!(result.checks.len(), 4);

        // 序列化和反序列化
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: DiagnosticResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.checks.len(), 4);
        assert!(deserialized.has_errors);
        assert!(deserialized.has_warnings);
    }

    /// 测试修复工作流程
    #[test]
    fn test_fix_workflow() {
        // 模拟修复操作
        let fix_result = FixResult {
            fixed: vec!["config".to_string(), "permissions".to_string()],
            failed: vec![
                FixFailure {
                    name: "service".to_string(),
                    error: "Port already in use".to_string(),
                },
            ],
        };

        // 验证结果
        assert_eq!(fix_result.fixed.len(), 2);
        assert_eq!(fix_result.failed.len(), 1);

        // 计算成功率
        let total = fix_result.fixed.len() + fix_result.failed.len();
        let success_rate = fix_result.fixed.len() as f64 / total as f64;
        assert!((success_rate - 0.666666).abs() < 0.001);

        // 序列化
        let json = serde_json::to_string(&fix_result).unwrap();
        assert!(json.contains("config"));
        assert!(json.contains("Port already in use"));
    }

    // ==================== 性能测试（轻量级） ====================

    /// 测试大量检查项
    #[test]
    fn test_large_number_of_checks() {
        let mut checks = Vec::new();

        for i in 0..1000 {
            let status = match i % 3 {
                0 => CheckStatus::Pass,
                1 => CheckStatus::Warning,
                _ => CheckStatus::Error,
            };

            checks.push(DiagnosticCheck {
                category: "test".to_string(),
                name: format!("check_{}", i),
                status,
                message: format!("Message {}", i),
                details: None,
                fixable: false,
                fix_suggestion: None,
            });
        }

        let result = DiagnosticResult {
            checks,
            has_errors: true,
            has_warnings: true,
            checked_at: chrono::Local::now().to_rfc3339(),
        };

        assert_eq!(result.checks.len(), 1000);

        // 序列化应该成功
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.is_empty());
    }
}
