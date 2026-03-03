//! 进程管理器
//!
//! 提供服务启动、停止、监控和健康检查功能

#![allow(dead_code)]

use crate::errors::{AppError, ProcessError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::System;
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, timeout};

/// 服务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServiceStatus {
    /// 正在启动
    Starting,
    /// 运行中
    Running { pid: u32, started_at: u64 },
    /// 正在停止
    Stopping,
    /// 已停止
    Stopped,
    /// 错误状态
    Error(String),
    /// 已崩溃
    Crashed { exit_code: i32, message: String },
}

/// 进程事件
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ProcessEvent {
    /// 服务启动
    Started { name: String, pid: u32 },
    /// 服务停止
    Stopped { name: String },
    /// 服务崩溃
    Crashed { name: String, exit_code: Option<i32> },
    /// 日志输出
    Log { name: String, level: String, message: String },
    /// 状态变化
    StatusChanged { name: String, status: ServiceStatus },
}

/// 服务信息
#[derive(Debug)]
struct ServiceHandle {
    /// 进程句柄
    process: Child,
    /// 服务信息
    info: ServiceInfo,
    /// 关闭信号发送器
    shutdown_tx: tokio::sync::mpsc::Sender<()>,
    /// 日志收集任务
    _log_task: tokio::task::JoinHandle<()>,
}

/// 服务配置信息
#[derive(Debug, Clone)]
struct ServiceInfo {
    /// 服务名称
    name: String,
    /// 当前状态
    status: ServiceStatus,
    /// 启动命令
    command: String,
    /// 命令参数
    args: Vec<String>,
    /// 环境变量
    env_vars: HashMap<String, String>,
    /// 健康检查端口
    health_check_port: Option<u16>,
    /// 健康检查路径
    health_check_path: Option<String>,
    /// 重启次数
    restart_count: u32,
    /// 最后退出码
    last_exit_code: Option<i32>,
}

/// 健康检查结果
#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub message: String,
    pub response_time_ms: Option<u64>,
}

/// 启动服务请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartServiceRequest {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
    pub health_check_port: Option<u16>,
    pub health_check_path: Option<String>,
}

/// 进程管理器
pub struct ProcessManager {
    /// 运行中的服务
    services: Arc<RwLock<HashMap<String, ServiceHandle>>>,
    /// 事件广播通道
    event_sender: broadcast::Sender<ProcessEvent>,
}

impl ProcessManager {
    /// 创建新的进程管理器
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(100);
        
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
        }
    }

    /// 启动服务
    pub async fn start_service(
        &self,
        request: StartServiceRequest,
    ) -> Result<ServiceStatus, AppError> {
        let name = request.name;
        let mut services = self.services.write().await;

        // 检查是否已存在
        if services.contains_key(&name) {
            return Err(ProcessError::AlreadyRunning(name).into());
        }

        // 检查端口占用
        if let Some(port) = request.health_check_port {
            if self.is_port_in_use(port).await {
                return Err(ProcessError::PortInUse(port).into());
            }
        }

        log::info!("Starting service '{}' with command: {} {:?}", 
            name, request.command, request.args);

        // 更新状态
        self.event_sender
            .send(ProcessEvent::StatusChanged {
                name: name.clone(),
                status: ServiceStatus::Starting,
            })
            .ok();

        // 构建命令
        let mut cmd = Command::new(&request.command);
        cmd.args(&request.args)
            .envs(&request.env_vars)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        // 启动进程
        let mut process = cmd.spawn().map_err(|e| {
            ProcessError::StartFailed(format!("Failed to spawn process: {}", e))
        })?;

        let pid = process.id().ok_or_else(|| {
            ProcessError::StartFailed("Failed to get process ID".to_string())
        })?;

        log::info!("Service '{}' started with PID {}", name, pid);

        // 创建关闭通道
        let (shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel(1);

        let info = ServiceInfo {
            name: name.clone(),
            status: ServiceStatus::Running {
                pid,
                started_at: current_timestamp(),
            },
            command: request.command,
            args: request.args,
            env_vars: request.env_vars,
            health_check_port: request.health_check_port,
            health_check_path: request.health_check_path,
            restart_count: 0,
            last_exit_code: None,
        };

        // 启动日志收集任务
        let log_task = self.spawn_log_collector(
            name.clone(),
            process.stdout.take(),
            process.stderr.take(),
        );

        let handle = ServiceHandle {
            process,
            info: info.clone(),
            shutdown_tx,
            _log_task: log_task,
        };

        services.insert(name.clone(), handle);
        drop(services);

        // 启动监控任务
        self.spawn_monitor_task(name.clone(), shutdown_rx);

        self.event_sender
            .send(ProcessEvent::Started { name, pid })
            .ok();

        Ok(info.status)
    }

    /// 优雅停止服务
    pub async fn stop_service(
        &self,
        name: &str,
        timeout_secs: u64,
    ) -> Result<(), AppError> {
        let mut services = self.services.write().await;

        let handle = services.get_mut(name).ok_or_else(|| {
            ProcessError::NotFound(name.to_string())
        })?;

        // 更新状态
        handle.info.status = ServiceStatus::Stopping;
        self.event_sender
            .send(ProcessEvent::StatusChanged {
                name: name.to_string(),
                status: ServiceStatus::Stopping,
            })
            .ok();

        // 发送优雅关闭信号
        #[cfg(unix)]
        {
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;

            if let Some(pid) = handle.process.id() {
                log::info!("Sending SIGTERM to process {}", pid);
                signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM).ok();
            }
        }

        #[cfg(windows)]
        {
            // Windows: 使用 taskkill /T /PID
            if let Some(pid) = handle.process.id() {
                let _ = Command::new("taskkill")
                    .args([&"/T".to_string(),
                        "/PID".to_string(),
                        pid.to_string(),
                    ])
                    .spawn();
            }
        }

        let shutdown_tx = handle.shutdown_tx.clone();
        drop(services);

        // 等待进程退出或超时
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(timeout_secs)) => {
                log::warn!("Service '{}' stop timeout, forcing kill", name);
                let mut services = self.services.write().await;
                if let Some(handle) = services.get_mut(name) {
                    let _ = handle.process.kill().await;
                }
            }
            _ = shutdown_tx.closed() => {
                log::info!("Service '{}' stopped gracefully", name);
            }
        }

        let mut services = self.services.write().await;
        services.remove(name);

        self.event_sender
            .send(ProcessEvent::Stopped { name: name.to_string() })
            .ok();

        Ok(())
    }

    /// 强制终止服务
    pub async fn force_kill(&self, name: &str) -> Result<(), AppError> {
        let mut services = self.services.write().await;

        let handle = services.get_mut(name).ok_or_else(|| {
            ProcessError::NotFound(name.to_string())
        })?;

        log::warn!("Force killing service '{}'", name);

        // 强制终止进程
        match handle.process.kill().await {
            Ok(()) => {
                log::info!("Service '{}' force killed", name);
            }
            Err(e) => {
                log::error!("Failed to force kill service '{}': {}", name, e);
                return Err(ProcessError::TerminateFailed(e.to_string()).into());
            }
        }

        // 移除服务记录
        services.remove(name);

        self.event_sender
            .send(ProcessEvent::Stopped { name: name.to_string() })
            .ok();

        Ok(())
    }

    /// 获取服务状态
    pub async fn get_status(&self,
        name: &str
    ) -> Option<ServiceStatus> {
        let services = self.services.read().await;
        services.get(name).map(|h| h.info.status.clone())
    }

    /// 健康检查
    pub async fn health_check(
        &self,
        name: &str,
    ) -> Result<HealthStatus, AppError> {
        let services = self.services.read().await;
        let handle = services.get(name).ok_or_else(|| {
            ProcessError::NotFound(name.to_string())
        })?;

        let start_time = std::time::Instant::now();

        // 检查进程是否存活
        if let Some(pid) = handle.process.id() {
            let mut system = System::new_all();
            system.refresh_processes();

            let is_alive = system
                .process(sysinfo::Pid::from(pid as usize))
                .is_some();

            if !is_alive {
                return Ok(HealthStatus {
                    healthy: false,
                    message: "Process is not running".to_string(),
                    response_time_ms: None,
                });
            }
        }

        // HTTP 健康检查
        if let (Some(port), Some(path)) = (
            handle.info.health_check_port,
            handle.info.health_check_path.as_ref(),
        ) {
            let url = format!("http://127.0.0.1:{}{}", port, path);
            
            match timeout(
                Duration::from_secs(5),
                reqwest::get(&url)
            ).await {
                Ok(Ok(response)) => {
                    let response_time = start_time.elapsed().as_millis() as u64;
                    let healthy = response.status().is_success();
                    
                    return Ok(HealthStatus {
                        healthy,
                        message: if healthy {
                            "Service is healthy".to_string()
                        } else {
                            format!("HTTP {}", response.status())
                        },
                        response_time_ms: Some(response_time),
                    });
                }
                Ok(Err(e)) => {
                    return Ok(HealthStatus {
                        healthy: false,
                        message: format!("Health check request failed: {}", e),
                        response_time_ms: None,
                    });
                }
                Err(_) => {
                    return Ok(HealthStatus {
                        healthy: false,
                        message: "Health check timeout".to_string(),
                        response_time_ms: None,
                    });
                }
            }
        }

        // 仅进程检查
        let response_time = start_time.elapsed().as_millis() as u64;
        Ok(HealthStatus {
            healthy: true,
            message: "Process is running".to_string(),
            response_time_ms: Some(response_time),
        })
    }

    // 私有方法

    /// 检查端口是否被占用
    async fn is_port_in_use(&self,
        port: u16
    ) -> bool {
        // 尝试绑定端口，如果失败说明被占用
        tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .is_err()
    }

    /// 启动监控任务
    fn spawn_monitor_task(
        &self,
        name: String,
        mut shutdown_rx: tokio::sync::mpsc::Receiver<()>,
    ) {
        let services = self.services.clone();
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            let mut check_interval = interval(Duration::from_secs(1));

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                    _ = check_interval.tick() => {
                        let mut services = services.write().await;
                        
                        if let Some(handle) = services.get_mut(&name) {
                            match handle.process.try_wait() {
                                Ok(Some(status)) => {
                                    // 进程已退出
                                    let exit_code = status.code().unwrap_or(-1);
                                    handle.info.last_exit_code = Some(exit_code);
                                    handle.info.status = ServiceStatus::Crashed {
                                        exit_code,
                                        message: format!("Process exited with code {}", exit_code),
                                    };

                                    event_sender
                                        .send(ProcessEvent::Crashed {
                                            name: name.clone(),
                                            exit_code: Some(exit_code),
                                        })
                                        .ok();
                                    
                                    services.remove(&name);
                                    break;
                                }
                                Ok(None) => {
                                    // 进程仍在运行
                                }
                                Err(e) => {
                                    log::error!("Error checking process status: {}", e);
                                }
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        });
    }

    /// 启动日志收集任务
    fn spawn_log_collector(
        &self,
        name: String,
        stdout: Option<tokio::process::ChildStdout>,
        stderr: Option<tokio::process::ChildStderr>,
    ) -> tokio::task::JoinHandle<()> {
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};

            // 收集 stdout
            if let Some(stdout) = stdout {
                let sender = event_sender.clone();
                let service_name = name.clone();
                
                tokio::spawn(async move {
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();

                    while let Ok(Some(line)) = lines.next_line().await {
                        sender
                            .send(ProcessEvent::Log {
                                name: service_name.clone(),
                                level: "info".to_string(),
                                message: line,
                            })
                            .ok();
                    }
                });
            }

            // 收集 stderr
            if let Some(stderr) = stderr {
                let sender = event_sender.clone();
                let service_name = name.clone();
                
                tokio::spawn(async move {
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();

                    while let Ok(Some(line)) = lines.next_line().await {
                        sender
                            .send(ProcessEvent::Log {
                                name: service_name.clone(),
                                level: "error".to_string(),
                                message: line,
                            })
                            .ok();
                    }
                });
            }
        })
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取当前时间戳（秒）
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
