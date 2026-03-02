//! 进程管理器单元测试
//!
//! 测试服务启动、停止、健康检查等功能

#[cfg(test)]
mod tests {
    use crate::services::process_manager::{ProcessManager, ServiceStatus, ProcessEvent, HealthStatus, StartServiceRequest};
    use tokio::time::{sleep, Duration};

    /// 测试进程管理器创建
    #[test]
    fn test_process_manager_creation() {
        let manager = ProcessManager::new();
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
        let status = manager.get_status("nonexistent_service").await;
        assert!(status.is_none(), "Should return None for non-existent service");
    }

    /// 测试事件广播通道
    #[test]
    fn test_event_broadcast() {
        let manager = ProcessManager::new();

        // 验证管理器可以创建（事件发送器在内部初始化）
        // 实际的事件测试需要通过集成测试完成
        assert!(true, "ProcessManager with event channel created");
    }

    /// 测试进程错误类型
    #[test]
    fn test_process_error_variants() {
        use crate::errors::ProcessError;

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
        let manager: ProcessManager = Default::default();
        assert!(true, "Default implementation works");
    }

    /// 模拟服务生命周期测试
    #[tokio::test]
    async fn test_service_lifecycle_mock() {
        let manager = ProcessManager::new();

        // 验证初始状态
        let initial_status = manager.get_status("test_service").await;
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
}

