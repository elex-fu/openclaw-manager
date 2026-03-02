//! 日志命令
//!
//! 提供日志查看和管理相关的Tauri命令

use crate::errors::app_error::ApiResponse;
use crate::services::log_service::{
    ExportFormat, LogEntry, LogFilter, LogLevel, LogServiceState,
};
use crate::services::log_watcher::{LogWatcherEvent, LogWatcherState};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{Emitter, Manager, State, Window};
use tokio::sync::mpsc;

/// 日志订阅请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeLogsRequest {
    pub levels: Vec<String>,
    pub sources: Option<Vec<String>>,
    pub search_query: Option<String>,
}

/// 日志订阅响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeLogsResponse {
    pub subscription_id: String,
}

/// 获取最近日志请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRecentLogsRequest {
    pub limit: Option<usize>,
    pub levels: Option<Vec<String>>,
    pub sources: Option<Vec<String>>,
    pub search_query: Option<String>,
}

/// 导出日志请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportLogsRequest {
    pub filter: LogFilter,
    pub format: String,
    pub output_path: String,
}

/// 日志源信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSourceInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: i64,
}

/// 获取日志源列表
#[tauri::command]
pub async fn get_log_sources(
    log_state: State<'_, LogServiceState>,
) -> Result<ApiResponse<Vec<LogSourceInfo>>, String> {
    let service = log_state.service.read().await;
    let files = service.get_log_files().await;

    let sources: Vec<LogSourceInfo> = files
        .into_iter()
        .map(|f| LogSourceInfo {
            id: uuid::Uuid::new_v4().to_string(),
            name: f.source.clone(),
            path: f.path,
            size: f.size,
            modified: f.modified,
        })
        .collect();

    Ok(ApiResponse::success(sources))
}

/// 获取最近日志
#[tauri::command]
pub async fn get_recent_logs(
    req: GetRecentLogsRequest,
    log_state: State<'_, LogServiceState>,
) -> Result<ApiResponse<Vec<LogEntry>>, String> {
    let filter = LogFilter {
        levels: req
            .levels
            .unwrap_or_default()
            .into_iter()
            .filter_map(|s| LogLevel::from_str(&s))
            .collect(),
        search_query: req.search_query,
        sources: req.sources,
        start_time: None,
        end_time: None,
    };

    let service = log_state.service.read().await;
    let limit = req.limit.unwrap_or(100);

    match service.get_recent_logs(limit, &filter).await {
        Ok(entries) => Ok(ApiResponse::success(entries)),
        Err(e) => Ok(ApiResponse::error(e)),
    }
}

/// 订阅实时日志
#[tauri::command]
pub async fn subscribe_logs(
    req: SubscribeLogsRequest,
    window: Window,
    log_state: State<'_, LogServiceState>,
    watcher_state: State<'_, LogWatcherState>,
) -> Result<ApiResponse<SubscribeLogsResponse>, String> {
    let subscription_id = uuid::Uuid::new_v4().to_string();

    // 构建筛选器
    let filter = LogFilter {
        levels: req
            .levels
            .into_iter()
            .filter_map(|s| LogLevel::from_str(&s))
            .collect(),
        search_query: req.search_query,
        sources: req.sources,
        start_time: None,
        end_time: None,
    };

    // 启动日志监控
    let service = log_state.service.read().await;
    let watcher_service = watcher_state.service.read().await;

    // 扫描并监控日志文件
    if let Err(e) = watcher_service.start(&*service).await {
        return Ok(ApiResponse::error(e));
    }

    // 创建事件通道
    let (_tx, mut rx) = mpsc::channel(1000);

    // 启动事件转发任务
    let window_clone = window.clone();
    let subscription_id_clone = subscription_id.clone();
    let filter_clone = filter.clone();

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                LogWatcherEvent::NewEntry(entry) => {
                    // 应用筛选器
                    if filter_clone.matches(&entry) {
                        let _ = window_clone.emit(
                            &format!("log-entry-{}", subscription_id_clone),
                            entry,
                        );
                    }
                }
                LogWatcherEvent::FileReset(source) => {
                    let _ = window_clone.emit(
                        &format!("log-reset-{}", subscription_id_clone),
                        source,
                    );
                }
                LogWatcherEvent::Error(source, error) => {
                    let _ = window_clone.emit(
                        &format!("log-error-{}", subscription_id_clone),
                        serde_json::json!({
                            "source": source,
                            "error": error,
                        }),
                    );
                }
            }
        }
    });

    Ok(ApiResponse::success(SubscribeLogsResponse { subscription_id }))
}

/// 取消订阅日志
#[tauri::command]
pub async fn unsubscribe_logs(
    subscription_id: String,
    _watcher_state: State<'_, LogWatcherState>,
) -> Result<ApiResponse<bool>, String> {
    // 在实际实现中，这里应该停止对应的事件转发任务
    // 简化处理：返回成功
    log::info!("取消日志订阅: {}", subscription_id);
    Ok(ApiResponse::success(true))
}

/// 导出日志
#[tauri::command]
pub async fn export_logs(
    req: ExportLogsRequest,
    log_state: State<'_, LogServiceState>,
) -> Result<ApiResponse<String>, String> {
    let format = match req.format.as_str() {
        "json" => ExportFormat::Json,
        "csv" => ExportFormat::Csv,
        _ => ExportFormat::Text,
    };

    let service = log_state.service.read().await;

    match service.export_logs(&req.filter, format, &req.output_path).await {
        Ok(_) => Ok(ApiResponse::success(req.output_path)),
        Err(e) => Ok(ApiResponse::error(e)),
    }
}

/// 添加自定义日志源
#[tauri::command]
pub async fn add_log_source(
    path: String,
    source: String,
    log_state: State<'_, LogServiceState>,
    watcher_state: State<'_, LogWatcherState>,
) -> Result<ApiResponse<bool>, String> {
    let path_buf = PathBuf::from(&path);

    // 注册到日志服务
    {
        let service = log_state.service.write().await;
        if let Err(e) = service.register_log_file(path_buf.clone(), source.clone()).await {
            return Ok(ApiResponse::error(e));
        }
    }

    // 添加到监控
    {
        let watcher_service = watcher_state.service.read().await;
        let watcher = watcher_service.watcher();
        if let Err(e) = watcher.watch_file(path_buf, source).await {
            return Ok(ApiResponse::error(e));
        }
    }

    Ok(ApiResponse::success(true))
}

/// 移除日志源
#[tauri::command]
pub async fn remove_log_source(
    source_id: String,
    watcher_state: State<'_, LogWatcherState>,
) -> Result<ApiResponse<bool>, String> {
    let watcher_service = watcher_state.service.read().await;
    let watcher = watcher_service.watcher();

    if let Err(e) = watcher.unwatch_file(&source_id).await {
        return Ok(ApiResponse::error(e));
    }

    Ok(ApiResponse::success(true))
}

/// 初始化默认日志源
#[tauri::command]
pub async fn init_default_log_sources(
    log_state: State<'_, LogServiceState>,
) -> Result<ApiResponse<bool>, String> {
    let service = log_state.service.write().await;

    if let Err(e) = service.scan_default_logs().await {
        return Ok(ApiResponse::error(e));
    }

    Ok(ApiResponse::success(true))
}

/// 清空日志显示（仅清空前端缓存，不影响实际日志文件）
#[tauri::command]
pub async fn clear_log_display() -> Result<ApiResponse<bool>, String> {
    // 这是一个前端操作，后端只需返回成功
    Ok(ApiResponse::success(true))
}

/// 获取日志统计信息
#[tauri::command]
pub async fn get_log_stats(
    log_state: State<'_, LogServiceState>,
) -> Result<ApiResponse<serde_json::Value>, String> {
    let service = log_state.service.read().await;
    let files = service.get_log_files().await;

    let total_size: u64 = files.iter().map(|f| f.size).sum();
    let source_count = files.len();

    let stats = serde_json::json!({
        "total_size": total_size,
        "source_count": source_count,
        "sources": files.iter().map(|f| {
            serde_json::json!({
                "source": f.source,
                "size": f.size,
                "path": f.path,
            })
        }).collect::<Vec<_>>(),
    });

    Ok(ApiResponse::success(stats))
}
