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

        let pid = handle.process.id().ok_or_else(|| {
            ProcessError::StopFailed("Failed to get process ID".to_string())
        })?;

        // 使用平台特定的优雅关闭实现
        let shutdown_result = platform::graceful_shutdown(pid, timeout_secs).await;

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

        shutdown_result
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

/// 平台特定的进程管理实现
mod platform {
    use super::*;

    /// 优雅关闭进程
    ///
    /// 实现三级关闭策略：
    /// 1. 发送优雅关闭信号（Ctrl+Break/WM_CLOSE 或 SIGTERM）
    /// 2. 等待进程退出（带超时）
    /// 3. 强制终止
    pub async fn graceful_shutdown(pid: u32, timeout_secs: u64) -> Result<(), AppError> {
        #[cfg(windows)]
        {
            windows_process::graceful_shutdown(pid, timeout_secs).await
        }
        #[cfg(unix)]
        {
            unix_process::graceful_shutdown(pid, timeout_secs).await
        }
        #[cfg(not(any(windows, unix)))]
        {
            Err(ProcessError::StopFailed("Unsupported platform".to_string()).into())
        }
    }
}

/// Windows 平台进程管理实现
#[cfg(windows)]
mod windows_process {
    use super::*;
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;
    use windows_sys::Win32::Foundation::{BOOL, FALSE, HWND, LPARAM, TRUE, WPARAM};
    use windows_sys::Win32::System::Console::{AttachConsole, FreeConsole, GenerateConsoleCtrlEvent, SetConsoleCtrlHandler, CTRL_BREAK_EVENT};
    use windows_sys::Win32::System::Threading::GetCurrentProcessId;
    use windows_sys::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId, PostMessageW, WM_CLOSE};

    /// Windows 优雅关闭三级策略：
    /// 1. Ctrl+Break 信号
    /// 2. WM_CLOSE 消息
    /// 3. 强制终止 (taskkill /F /T)
    pub async fn graceful_shutdown(pid: u32, timeout_secs: u64) -> Result<(), AppError> {
        log::info!("Attempting graceful shutdown for PID {}", pid);

        // 方案1: Ctrl+Break 信号
        log::info!("Phase 1: Sending Ctrl+Break to PID {}", pid);
        if send_ctrl_break(pid) {
            if wait_for_exit(pid, timeout_secs / 3).await {
                log::info!("Process {} exited after Ctrl+Break", pid);
                return Ok(());
            }
        }

        // 方案2: WM_CLOSE 消息
        log::info!("Phase 2: Sending WM_CLOSE to PID {}", pid);
        if send_wm_close(pid) {
            if wait_for_exit(pid, timeout_secs / 3).await {
                log::info!("Process {} exited after WM_CLOSE", pid);
                return Ok(());
            }
        }

        // 方案3: 强制终止
        log::warn!("Phase 3: Force terminating PID {}", pid);
        force_terminate(pid).await
    }

    /// 发送 Ctrl+Break 信号到指定进程
    ///
    /// 注意：这是 unsafe 操作，需要正确附加到目标进程的控制台
    unsafe fn send_ctrl_break_internal(pid: u32) -> bool {
        // 先禁用当前进程的控制台事件处理
        if SetConsoleCtrlHandler(None, TRUE) == 0 {
            log::warn!("Failed to disable console ctrl handler");
            return false;
        }

        // 附加到目标进程的控制台
        if AttachConsole(pid) == 0 {
            let error = std::io::Error::last_os_error();
            log::warn!("Failed to attach to console of PID {}: {}", pid, error);
            // 恢复控制台事件处理
            SetConsoleCtrlHandler(None, FALSE);
            return false;
        }

        // 发送 Ctrl+Break 事件到当前进程组
        let result = GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, 0);

        // 分离控制台
        FreeConsole();

        // 恢复控制台事件处理
        SetConsoleCtrlHandler(None, FALSE);

        if result == 0 {
            let error = std::io::Error::last_os_error();
            log::warn!("Failed to generate Ctrl+Break event: {}", error);
            false
        } else {
            log::info!("Successfully sent Ctrl+Break to PID {}", pid);
            true
        }
    }

    /// 安全的 Ctrl+Break 发送包装函数
    fn send_ctrl_break(pid: u32) -> bool {
        // 不能向自己发送 Ctrl+Break
        let current_pid = unsafe { GetCurrentProcessId() };
        if pid == current_pid {
            log::warn!("Cannot send Ctrl+Break to self");
            return false;
        }

        unsafe { send_ctrl_break_internal(pid) }
    }

    /// 窗口枚举回调函数使用的上下文结构
    struct WindowEnumContext {
        target_pid: u32,
        found: bool,
    }

    /// 发送 WM_CLOSE 消息到目标进程的所有顶层窗口
    fn send_wm_close(pid: u32) -> bool {
        let mut context = WindowEnumContext {
            target_pid: pid,
            found: false,
        };

        unsafe {
            EnumWindows(
                Some(enum_windows_callback),
                &mut context as *mut _ as LPARAM,
            );
        }

        if context.found {
            log::info!("Sent WM_CLOSE to windows of PID {}", pid);
        } else {
            log::warn!("No windows found for PID {}", pid);
        }

        context.found
    }

    /// 窗口枚举回调函数
    unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let context = &mut *(lparam as *mut WindowEnumContext);
        let mut window_pid: u32 = 0;

        // 获取窗口所属进程ID
        GetWindowThreadProcessId(hwnd, &mut window_pid);

        if window_pid == context.target_pid {
            // 发送 WM_CLOSE 消息
            PostMessageW(hwnd, WM_CLOSE, 0 as WPARAM, 0 as LPARAM);
            context.found = true;
        }

        TRUE // 继续枚举
    }

    /// 等待进程退出
    async fn wait_for_exit(pid: u32, timeout_secs: u64) -> bool {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        while start.elapsed() < timeout {
            // 检查进程是否仍在运行
            if !is_process_running(pid) {
                return true;
            }

            // 短暂等待
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        false
    }

    /// 检查进程是否仍在运行
    fn is_process_running(pid: u32) -> bool {
        use std::process::Command;

        // 使用 tasklist 检查进程是否存在
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains(&pid.to_string())
            }
            Err(_) => false,
        }
    }

    /// 强制终止进程及其子进程
    async fn force_terminate(pid: u32) -> Result<(), AppError> {
        log::warn!("Force terminating PID {} and its children", pid);

        // 使用 taskkill /F /T 强制终止进程树
        let output = Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output()
            .await
            .map_err(|e| ProcessError::TerminateFailed(format!("Failed to execute taskkill: {}", e)))?;

        if output.status.success() {
            log::info!("Successfully force terminated PID {}", pid);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let err_msg = format!("taskkill failed: {}", stderr);
            log::error!("{}", err_msg);
            Err(ProcessError::TerminateFailed(err_msg).into())
        }
    }
}

/// Unix 平台进程管理实现
#[cfg(unix)]
mod unix_process {
    use super::*;
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid;
    use std::process::Command;

    /// Unix 优雅关闭三级策略：
    /// 1. SIGTERM 信号
    /// 2. SIGINT 信号
    /// 3. SIGKILL 强制终止
    pub async fn graceful_shutdown(pid: u32, timeout_secs: u64) -> Result<(), AppError> {
        log::info!("Attempting graceful shutdown for PID {}", pid);

        let nix_pid = Pid::from_raw(pid as i32);

        // 方案1: SIGTERM 信号
        log::info!("Phase 1: Sending SIGTERM to PID {}", pid);
        match signal::kill(nix_pid, Signal::SIGTERM) {
            Ok(_) => {
                if wait_for_exit(pid, timeout_secs / 3).await {
                    log::info!("Process {} exited after SIGTERM", pid);
                    return Ok(());
                }
            }
            Err(e) => {
                log::warn!("Failed to send SIGTERM to PID {}: {}", pid, e);
            }
        }

        // 方案2: SIGINT 信号
        log::info!("Phase 2: Sending SIGINT to PID {}", pid);
        match signal::kill(nix_pid, Signal::SIGINT) {
            Ok(_) => {
                if wait_for_exit(pid, timeout_secs / 3).await {
                    log::info!("Process {} exited after SIGINT", pid);
                    return Ok(());
                }
            }
            Err(e) => {
                log::warn!("Failed to send SIGINT to PID {}: {}", pid, e);
            }
        }

        // 方案3: SIGKILL 强制终止
        log::warn!("Phase 3: Sending SIGKILL to PID {}", pid);
        match signal::kill(nix_pid, Signal::SIGKILL) {
            Ok(_) => {
                // 给进程一点时间响应 SIGKILL
                tokio::time::sleep(Duration::from_millis(500)).await;
                log::info!("Process {} force killed with SIGKILL", pid);
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("Failed to send SIGKILL to PID {}: {}", pid, e);
                log::error!("{}", err_msg);
                Err(ProcessError::TerminateFailed(err_msg).into())
            }
        }
    }

    /// 等待进程退出
    async fn wait_for_exit(pid: u32, timeout_secs: u64) -> bool {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        while start.elapsed() < timeout {
            // 检查进程是否仍在运行
            if !is_process_running(pid) {
                return true;
            }

            // 短暂等待
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        false
    }

    /// 检查进程是否仍在运行
    fn is_process_running(pid: u32) -> bool {
        // 使用 kill -0 检查进程是否存在
        let output = Command::new("kill")
            .args(["-0", &pid.to_string()])
            .output();

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
}

/// 不支持的平台实现
#[cfg(not(any(windows, unix)))]
mod platform {
    use super::*;

    pub async fn graceful_shutdown(_pid: u32, _timeout_secs: u64) -> Result<(), AppError> {
        Err(ProcessError::StopFailed("Unsupported platform".to_string()).into())
    }
}
