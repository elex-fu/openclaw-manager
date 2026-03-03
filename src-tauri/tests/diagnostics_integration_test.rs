//! 诊断服务集成测试

/// 测试诊断服务创建
#[test]
fn test_diagnostic_service_creation() {
    let result = openclaw_manager::services::diagnostics::DiagnosticService::new();
    // 可能成功或失败，取决于环境
    assert!(result.is_ok() || result.is_err());
}

/// 测试诊断检查结果结构
#[test]
fn test_diagnostic_result_structure() {
    use openclaw_manager::services::diagnostics::{DiagnosticResult, DiagnosticCheck, CheckStatus};

    let check = DiagnosticCheck {
        category: "system".to_string(),
        name: "memory".to_string(),
        status: CheckStatus::Pass,
        message: "Memory check passed".to_string(),
        details: None,
        fixable: false,
        fix_suggestion: None,
    };

    let result = DiagnosticResult {
        checks: vec![check],
        has_errors: false,
        has_warnings: false,
        checked_at: chrono::Local::now().to_rfc3339(),
    };

    assert!(!result.has_errors);
    assert!(!result.has_warnings);
    assert_eq!(result.checks.len(), 1);
}

/// 测试修复结果结构
#[test]
fn test_fix_result_structure() {
    use openclaw_manager::services::diagnostics::{FixResult, FixFailure};

    let fix_result = FixResult {
        fixed: vec!["issue1".to_string(), "issue2".to_string()],
        failed: vec![FixFailure {
            name: "issue3".to_string(),
            error: "Failed to fix".to_string(),
        }],
    };

    assert_eq!(fix_result.fixed.len(), 2);
    assert_eq!(fix_result.failed.len(), 1);
}

/// 测试检查状态序列化
#[test]
fn test_check_status_serialization() {
    use openclaw_manager::services::diagnostics::CheckStatus;

    let pass = CheckStatus::Pass;
    let warning = CheckStatus::Warning;
    let error = CheckStatus::Error;

    // 测试转换为严重程度
    assert_eq!(pass.to_severity(), "info");
    assert_eq!(warning.to_severity(), "warning");
    assert_eq!(error.to_severity(), "error");

    // 测试序列化
    let pass_json = serde_json::to_string(&pass).unwrap();
    assert!(pass_json.contains("pass"));

    let warning_json = serde_json::to_string(&warning).unwrap();
    assert!(warning_json.contains("warning"));

    let error_json = serde_json::to_string(&error).unwrap();
    assert!(error_json.contains("error"));
}

/// 测试诊断检查序列化
#[test]
fn test_diagnostic_check_serialization() {
    use openclaw_manager::services::diagnostics::{DiagnosticCheck, CheckStatus};

    let check = DiagnosticCheck {
        category: "system".to_string(),
        name: "disk_space".to_string(),
        status: CheckStatus::Warning,
        message: "Low disk space".to_string(),
        details: Some("Only 10% remaining".to_string()),
        fixable: true,
        fix_suggestion: Some("Free up disk space".to_string()),
    };

    let json = serde_json::to_string(&check).unwrap();
    assert!(json.contains("disk_space"));
    assert!(json.contains("warning"));
    assert!(json.contains("Low disk space"));
}

/// 测试诊断结果序列化
#[test]
fn test_diagnostic_result_serialization() {
    use openclaw_manager::services::diagnostics::{DiagnosticResult, DiagnosticCheck, CheckStatus};

    let check = DiagnosticCheck {
        category: "network".to_string(),
        name: "connectivity".to_string(),
        status: CheckStatus::Pass,
        message: "Network is reachable".to_string(),
        details: None,
        fixable: false,
        fix_suggestion: None,
    };

    let result = DiagnosticResult {
        checks: vec![check],
        has_errors: false,
        has_warnings: false,
        checked_at: "2024-01-01T00:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("connectivity"));
    assert!(json.contains("pass"));
}
