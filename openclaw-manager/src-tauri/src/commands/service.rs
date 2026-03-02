//! 服务控制相关命令

use crate::errors::ApiResponse;
use crate::services::diagnostics::{DiagnosticResult, DiagnosticService, FixResult};
use crate::services::process_manager::{
    HealthStatus, ProcessManager, ServiceStatus, StartServiceRequest,
};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// 启动服务
#[tauri::command]
pub async fn start_service(
    state: State<'_, Arc<Mutex<ProcessManager>>>,
    request: StartServiceRequest,
) -> Result<ApiResponse<ServiceStatus>, ()> {
    let manager = state.lock().await;
    let result = manager.start_service(request).await;
    Ok(ApiResponse::from_result(result))
}

/// 停止服务
#[tauri::command]
pub async fn stop_service(
    state: State<'_, Arc<Mutex<ProcessManager>>>,
    name: String,
    timeout_secs: Option<u64>,
) -> Result<ApiResponse<()>, ()> {
    let manager = state.lock().await;
    let timeout = timeout_secs.unwrap_or(30);
    let result = manager.stop_service(&name, timeout).await;
    Ok(ApiResponse::from_result(result))
}

/// 获取服务状态
#[tauri::command]
pub async fn get_service_status(
    state: State<'_, Arc<Mutex<ProcessManager>>>,
    name: String,
) -> Result<ApiResponse<Option<ServiceStatus>>, ()> {
    let manager = state.lock().await;
    let status = manager.get_status(&name).await;
    Ok(ApiResponse::success(status))
}

/// 健康检查
#[tauri::command]
pub async fn health_check_service(
    state: State<'_, Arc<Mutex<ProcessManager>>>,
    name: String,
) -> Result<ApiResponse<HealthStatus>, ()> {
    let manager = state.lock().await;
    let result = manager.health_check(&name).await;
    Ok(ApiResponse::from_result(result))
}

/// 运行诊断检查
#[tauri::command]
pub async fn run_diagnostics() -> Result<ApiResponse<DiagnosticResult>, ()> {
    let service = DiagnosticService::new().map_err(|_| ())?;
    let result = service.run_diagnostics().await;
    Ok(ApiResponse::from_result(result))
}

/// 自动修复问题
#[tauri::command]
pub async fn auto_fix_issues(issue_ids: Vec<String>) -> Result<ApiResponse<FixResult>, ()> {
    let service = DiagnosticService::new().map_err(|_| ())?;
    let result = service.auto_fix(issue_ids).await;
    Ok(ApiResponse::from_result(result))
}

/// 修复单个问题
#[tauri::command]
pub async fn fix_issue(issue_name: String) -> Result<ApiResponse<bool>, ()> {
    let service = DiagnosticService::new().map_err(|_| ())?;
    let result = service.auto_fix(vec![issue_name]).await;
    match result {
        Ok(fix_result) => {
            let success = !fix_result.fixed.is_empty();
            Ok(ApiResponse::success(success))
        }
        Err(e) => Ok(ApiResponse::error(e)),
    }
}
