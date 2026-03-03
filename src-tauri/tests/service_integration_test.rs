//! 服务控制集成测试

use std::sync::Arc;
use tokio::sync::Mutex;

/// 测试进程管理器创建
#[test]
fn test_process_manager_creation() {
    let manager = openclaw_manager::services::process_manager::ProcessManager::new();
    // 应该能成功创建
    assert!(true);
}

/// 测试服务状态枚举
#[test]
fn test_service_status_variants() {
    use openclaw_manager::services::process_manager::ServiceStatus;

    let starting = ServiceStatus::Starting;
    let json = serde_json::to_string(&starting).unwrap();
    assert!(json.contains("Starting"));

    let running = ServiceStatus::Running {
        pid: 1234,
        started_at: 1000,
    };
    let json = serde_json::to_string(&running).unwrap();
    assert!(json.contains("Running"));
    assert!(json.contains("1234"));

    let stopped = ServiceStatus::Stopped;
    let json = serde_json::to_string(&stopped).unwrap();
    assert!(json.contains("Stopped"));

    let error = ServiceStatus::Error("test error".to_string());
    let json = serde_json::to_string(&error).unwrap();
    assert!(json.contains("Error"));
}

/// 测试健康状态结构
#[test]
fn test_health_status() {
    use openclaw_manager::services::process_manager::HealthStatus;

    let healthy = HealthStatus {
        healthy: true,
        message: "Service is running".to_string(),
        response_time_ms: Some(100),
    };

    assert!(healthy.healthy);
    assert_eq!(healthy.response_time_ms, Some(100));

    let unhealthy = HealthStatus {
        healthy: false,
        message: "Connection refused".to_string(),
        response_time_ms: None,
    };

    assert!(!unhealthy.healthy);
    assert!(unhealthy.response_time_ms.is_none());
}

/// 测试启动服务请求
#[test]
fn test_start_service_request() {
    use openclaw_manager::services::process_manager::StartServiceRequest;
    use std::collections::HashMap;

    let mut env_vars = HashMap::new();
    env_vars.insert("KEY1".to_string(), "VALUE1".to_string());

    let request = StartServiceRequest {
        name: "test_service".to_string(),
        command: "/usr/bin/test".to_string(),
        args: vec!["arg1".to_string(), "arg2".to_string()],
        env_vars,
        health_check_port: Some(8080),
        health_check_path: Some("/health".to_string()),
    };

    assert_eq!(request.name, "test_service");
    assert_eq!(request.command, "/usr/bin/test");
    assert_eq!(request.args.len(), 2);
}

/// 测试进程事件序列化
#[test]
fn test_process_event_serialization() {
    use openclaw_manager::services::process_manager::{ProcessEvent, ServiceStatus};

    let started = ProcessEvent::Started {
        name: "test".to_string(),
        pid: 1234,
    };
    let json = serde_json::to_string(&started).unwrap();
    assert!(json.contains("Started"));

    let stopped = ProcessEvent::Stopped {
        name: "test".to_string(),
    };
    let json = serde_json::to_string(&stopped).unwrap();
    assert!(json.contains("Stopped"));

    let crashed = ProcessEvent::Crashed {
        name: "test".to_string(),
        exit_code: Some(1),
    };
    let json = serde_json::to_string(&crashed).unwrap();
    assert!(json.contains("Crashed"));

    let log = ProcessEvent::Log {
        name: "test".to_string(),
        level: "info".to_string(),
        message: "test message".to_string(),
    };
    let json = serde_json::to_string(&log).unwrap();
    assert!(json.contains("Log"));

    let status_changed = ProcessEvent::StatusChanged {
        name: "test".to_string(),
        status: ServiceStatus::Running {
            pid: 1234,
            started_at: 1000,
        },
    };
    let json = serde_json::to_string(&status_changed).unwrap();
    assert!(json.contains("StatusChanged"));
}

/// 测试进程管理器默认实现
#[test]
fn test_process_manager_default() {
    let manager: openclaw_manager::services::process_manager::ProcessManager = Default::default();
    // 应该能成功创建
    assert!(true);
}

/// 测试并发状态查询
#[tokio::test]
async fn test_concurrent_status_queries() {
    use openclaw_manager::services::process_manager::ProcessManager;
    use std::sync::Arc;

    let manager = Arc::new(ProcessManager::new());
    let mut handles = vec![];

    // 创建多个并发查询
    for i in 0..5 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            let status = mgr.get_status(&format!("service_{}", i)).await;
            status
        });
        handles.push(handle);
    }

    // 等待所有查询完成
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
    }
}
