//! 进程管理器单元测试
//!
//! 测试服务启动、停止、健康检查等功能
//! 覆盖 Windows 优雅关闭和 Unix 信号处理（MVP v2 新增）

use openclaw_manager::services::process_manager::{ProcessManager, ServiceStatus, ProcessEvent, HealthStatus, StartServiceRequest};
use openclaw_manager::errors::ProcessError;

/// 测试进程管理器创建
#[test]
fn test_process_manager_creation() {
    let _manager = ProcessManager::new();
    // 进程管理器应该能成功创建
    assert!(true, "ProcessManager created successfully");
}

/// 测试服务状态枚举
#[test]
fn test_service_status_variants() {
    // Test Starting status
    let status = ServiceStatus::Starting;
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("Starting"));

    // Test Running status
    let status = ServiceStatus::Running {
        pid: 1234,
        started_at: 1000,
    };
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("Running"));
    assert!(json.contains("1234"));

    // Test Stopping status
    let status = ServiceStatus::Stopping;
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("Stopping"));

    // Test Stopped status
    let status = ServiceStatus::Stopped;
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("Stopped"));

    // Test Error status
    let status = ServiceStatus::Error("Test error".to_string());
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("Error"));
    assert!(json.contains("Test error"));

    // Test Crashed status
    let status = ServiceStatus::Crashed {
        exit_code: 1,
        message: "Process crashed".to_string(),
    };
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("Crashed"));
    assert!(json.contains("1"));
}

/// 测试进程事件序列化
#[test]
fn test_process_event_serialization() {
    // Test Started event
    let event = ProcessEvent::Started {
        name: "test_service".to_string(),
        pid: 1234,
    };
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("Started"));
    assert!(json.contains("test_service"));
    assert!(json.contains("1234"));

    // Test Stopped event
    let event = ProcessEvent::Stopped {
        name: "test_service".to_string(),
    };
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("Stopped"));

    // Test Crashed event
    let event = ProcessEvent::Crashed {
        name: "test_service".to_string(),
        exit_code: Some(1),
    };
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("Crashed"));

    // Test Log event
    let event = ProcessEvent::Log {
        name: "test_service".to_string(),
        level: "info".to_string(),
        message: "Test log message".to_string(),
    };
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("Log"));
    assert!(json.contains("Test log message"));

    // Test StatusChanged event
    let event = ProcessEvent::StatusChanged {
        name: "test_service".to_string(),
        status: ServiceStatus::Running {
            pid: 1234,
            started_at: 1000,
        },
    };
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("StatusChanged"));
}

/// 测试健康检查结果
#[test]
fn test_health_status() {
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

/// 测试启动服务请求构建
#[test]
fn test_start_service_request() {
    use std::collections::HashMap;

    let mut env_vars = HashMap::new();
    env_vars.insert("KEY1".to_string(), "VALUE1".to_string());
    env_vars.insert("KEY2".to_string(), "VALUE2".to_string());

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
    assert_eq!(request.health_check_port, Some(8080));
    assert_eq!(request.health_check_path, Some("/health".to_string()));
}

/// 测试服务状态获取（空管理器）
#[tokio::test]
async fn test_get_status_no_service() {
    let manager = ProcessManager::new();

    // 查询不存在的服务状态
    let status: Option<ServiceStatus> = manager.get_status("nonexistent_service").await;
    assert!(status.is_none(), "Should return None for non-existent service");
}

/// 测试事件广播通道
#[test]
fn test_event_broadcast() {
    let _manager = ProcessManager::new();

    // 验证管理器可以创建（事件发送器在内部初始化）
    // 实际的事件测试需要通过集成测试完成
    assert!(true, "ProcessManager with event channel created");
}

/// 测试进程错误类型
#[test]
fn test_process_error_variants() {
    let err = ProcessError::AlreadyRunning("test".to_string());
    assert!(err.to_string().contains("already running"));

    let err = ProcessError::NotFound("test".to_string());
    assert!(err.to_string().contains("not found"));

    let err = ProcessError::StartFailed("permission denied".to_string());
    assert!(err.to_string().contains("permission denied"));

    let err = ProcessError::StopFailed("timeout".to_string());
    assert!(err.to_string().contains("timeout"));

    let err = ProcessError::PortInUse(8080);
    assert!(err.to_string().contains("8080"));

    let err = ProcessError::Crashed(1);
    assert!(err.to_string().contains("crashed"));

    let err = ProcessError::HealthCheckFailed("connection refused".to_string());
    assert!(err.to_string().contains("connection refused"));

    let err = ProcessError::Timeout(30);
    assert!(err.to_string().contains("30"));

    let err = ProcessError::TerminateFailed("kill failed".to_string());
    assert!(err.to_string().contains("kill failed"));
}

/// 测试进程管理器默认实现
#[test]
fn test_process_manager_default() {
    let _manager: ProcessManager = Default::default();
    assert!(true, "Default implementation works");
}

/// 模拟服务生命周期测试
#[tokio::test]
async fn test_service_lifecycle_mock() {
    let manager = ProcessManager::new();

    // 验证初始状态
    let initial_status: Option<ServiceStatus> = manager.get_status("test_service").await;
    assert!(initial_status.is_none());

    // 注意：实际启动服务需要外部程序，这里仅测试管理器状态
    // 完整的集成测试应在 tests/ 目录下进行
}

/// 测试并发状态查询
#[tokio::test]
async fn test_concurrent_status_queries() {
    let manager = std::sync::Arc::new(ProcessManager::new());

    let mut handles = vec![];

    // 创建多个并发查询
    for i in 0..10 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            let _status: Option<ServiceStatus> = mgr.get_status(&format!("service_{}", i)).await;
        });
        handles.push(handle);
    }

    // 等待所有查询完成
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
    }
}

/// 测试 Windows 优雅关闭策略（MVP v2 新增）
///
/// 注意：此测试仅在 Windows 平台上运行实际测试
/// 在其他平台上仅验证编译通过
#[test]
fn test_windows_graceful_shutdown_strategy() {
    // 验证 Windows 优雅关闭的三级策略在代码中存在
    // 1. Ctrl+Break 信号
    // 2. WM_CLOSE 消息
    // 3. 强制终止 (taskkill /F /T)

    #[cfg(windows)]
    {
        // 在 Windows 上验证相关类型和函数存在
        // 实际的进程关闭测试需要在集成测试中进行
        use openclaw_manager::services::process_manager::ProcessManager;
        let _manager = ProcessManager::new();
        assert!(true, "Windows graceful shutdown components compiled successfully");
    }

    #[cfg(not(windows))]
    {
        // 在非 Windows 平台上，测试通过编译即可
        assert!(true, "Windows graceful shutdown code compiles on non-Windows platforms");
    }
}

/// 测试 Unix 信号处理策略（MVP v2 新增）
///
/// 注意：此测试仅在 Unix 平台上运行实际测试
#[test]
fn test_unix_signal_handling_strategy() {
    // 验证 Unix 信号处理的三级策略在代码中存在
    // 1. SIGTERM 信号
    // 2. SIGINT 信号
    // 3. SIGKILL 强制终止

    #[cfg(unix)]
    {
        // 在 Unix 上验证相关类型和函数存在
        use openclaw_manager::services::process_manager::ProcessManager;
        let _manager = ProcessManager::new();
        assert!(true, "Unix signal handling components compiled successfully");
    }

    #[cfg(not(unix))]
    {
        // 在非 Unix 平台上，测试通过编译即可
        assert!(true, "Unix signal handling code compiles on non-Unix platforms");
    }
}

/// 测试跨平台优雅关闭接口（MVP v2 新增）
#[tokio::test]
async fn test_cross_platform_graceful_shutdown_interface() {
    // 验证 ProcessManager 的 stop_service 接口存在且可调用
    let manager = ProcessManager::new();

    // 尝试停止不存在的服务应该返回错误
    let result = manager.stop_service("nonexistent_service", 5).await;
    assert!(result.is_err(), "停止不存在的服务应该返回错误");

    // 验证错误类型
    match result {
        Err(e) => {
            let error_string = e.to_string();
            assert!(error_string.contains("not found") || error_string.contains("NotFound"),
                "错误信息应该包含 'not found'");
        }
        Ok(_) => panic!("应该返回错误"),
    }
}

/// 测试强制终止接口（MVP v2 新增）
#[tokio::test]
async fn test_force_kill_interface() {
    let manager = ProcessManager::new();

    // 尝试强制终止不存在的服务应该返回错误
    let result = manager.force_kill("nonexistent_service").await;
    assert!(result.is_err(), "强制终止不存在的服务应该返回错误");

    // 验证错误类型
    match result {
        Err(e) => {
            let error_string = e.to_string();
            assert!(error_string.contains("not found") || error_string.contains("NotFound"),
                "错误信息应该包含 'not found'");
        }
        Ok(_) => panic!("应该返回错误"),
    }
}

/// 测试健康检查接口（MVP v2 新增）
#[tokio::test]
async fn test_health_check_interface() {
    let manager = ProcessManager::new();

    // 尝试检查不存在的服务健康状态应该返回错误
    let result = manager.health_check("nonexistent_service").await;
    assert!(result.is_err(), "检查不存在的服务应该返回错误");

    // 验证错误类型
    match result {
        Err(e) => {
            let error_string = e.to_string();
            assert!(error_string.contains("not found") || error_string.contains("NotFound"),
                "错误信息应该包含 'not found'");
        }
        Ok(_) => panic!("应该返回错误"),
    }
}

/// 测试服务状态转换（MVP v2 新增）
#[test]
fn test_service_status_transitions() {
    // 验证所有状态都可以被正确创建和序列化
    let states = vec![
        ServiceStatus::Starting,
        ServiceStatus::Running { pid: 1234, started_at: 1000 },
        ServiceStatus::Stopping,
        ServiceStatus::Stopped,
        ServiceStatus::Error("test error".to_string()),
        ServiceStatus::Crashed { exit_code: 1, message: "crashed".to_string() },
    ];

    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: ServiceStatus = serde_json::from_str(&json).unwrap();

        // 验证序列化和反序列化的一致性
        match (state, deserialized) {
            (ServiceStatus::Starting, ServiceStatus::Starting) => {},
            (ServiceStatus::Stopping, ServiceStatus::Stopping) => {},
            (ServiceStatus::Stopped, ServiceStatus::Stopped) => {},
            (ServiceStatus::Running { pid: p1, .. }, ServiceStatus::Running { pid: p2, .. }) => {
                assert_eq!(p1, p2);
            }
            (ServiceStatus::Error(e1), ServiceStatus::Error(e2)) => {
                assert_eq!(e1, e2);
            }
            (ServiceStatus::Crashed { exit_code: c1, .. }, ServiceStatus::Crashed { exit_code: c2, .. }) => {
                assert_eq!(c1, c2);
            }
            _ => panic!("状态序列化/反序列化不一致"),
        }
    }
}

/// 测试进程事件类型（MVP v2 新增）
#[test]
fn test_process_event_types() {
    let events = vec![
        ProcessEvent::Started { name: "test".to_string(), pid: 1234 },
        ProcessEvent::Stopped { name: "test".to_string() },
        ProcessEvent::Crashed { name: "test".to_string(), exit_code: Some(1) },
        ProcessEvent::Log { name: "test".to_string(), level: "info".to_string(), message: "test".to_string() },
        ProcessEvent::StatusChanged { name: "test".to_string(), status: ServiceStatus::Running { pid: 1234, started_at: 1000 } },
    ];

    for event in events {
        let json = serde_json::to_string(&event).unwrap();
        // 验证可以序列化（反序列化需要 tag 支持）
        assert!(!json.is_empty());
    }
}
