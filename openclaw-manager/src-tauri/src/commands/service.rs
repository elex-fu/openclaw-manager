//! 服务控制相关命令

use crate::errors::ApiResponse;
use crate::services::process_manager::{
    HealthStatus, ProcessEvent, ProcessManager, ServiceStatus, StartServiceRequest,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Emitter, State};
use tokio::sync::Mutex;

/// 服务状态响应
#[derive(Debug, Serialize)]
pub struct ServiceStatusResponse {
    pub name: String,
    pub status: ServiceStatus,
}

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

/// 强制停止服务
#[tauri::command]
pub async fn kill_service(
    state: State<'_, Arc<Mutex<ProcessManager>>>,
    name: String,
) -> Result<ApiResponse<()>, ()> {
    let manager = state.lock().await;
    let result = manager.kill_service(&name).await;
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

/// 获取所有服务状态
#[tauri::command]
pub async fn get_all_service_status(
    state: State<'_, Arc<Mutex<ProcessManager>>>,
) -> Result<ApiResponse<HashMap<String, ServiceStatus>>, ()> {
    let manager = state.lock().await;
    let status = manager.get_all_status().await;
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

/// 检查服务是否存在
#[tauri::command]
pub async fn has_service(
    state: State<'_, Arc<Mutex<ProcessManager>>>,
    name: String,
) -> Result<ApiResponse<bool>, ()> {
    let manager = state.lock().await;
    let exists = manager.has_service(&name).await;
    Ok(ApiResponse::success(exists))
}

/// 订阅进程事件（通过 Tauri 事件系统）
#[tauri::command]
pub async fn subscribe_process_events(
    state: State<'_, Arc<Mutex<ProcessManager>>>,
    app_handle: tauri::AppHandle,
) -> Result<(), ()> {
    let manager = state.lock().await;
    let mut rx = manager.subscribe();
    drop(manager);

    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let event_type = match &event {
                ProcessEvent::Started { .. } => "service-started",
                ProcessEvent::Stopped { .. } => "service-stopped",
                ProcessEvent::Crashed { .. } => "service-crashed",
                ProcessEvent::Log { .. } => "service-log",
                ProcessEvent::StatusChanged { .. } => "service-status-changed",
            };

            if let Err(e) = app_handle.emit(event_type, event) {
                log::error!("Failed to emit event: {}", e);
                break;
            }
        }
    });

    Ok(())
}

