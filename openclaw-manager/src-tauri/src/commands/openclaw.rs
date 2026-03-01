use crate::installer::{InstallAllOptions, InstallProgress, OpenClawInstaller, SystemCheckResult};
use crate::models::openclaw::{AgentConfig, InstallResult, InstallStatus, ModelConfig, OpenClawConfig};
use crate::models::ApiResponse;
use crate::services::installer::{InstallMethod, InstallMethodInfo, InstallerService};
use std::sync::Arc;
use tauri::{Emitter, Manager, State, Window};
use tokio::sync::{mpsc, Mutex};

/// 全局安装器状态
pub struct InstallerState {
    pub installer: Arc<Mutex<OpenClawInstaller>>,
    pub service: Arc<Mutex<InstallerService>>,
}

/// 检查 OpenClaw 安装状态
#[tauri::command]
pub async fn check_openclaw_installation(
    state: State<'_, InstallerState>,
) -> Result<ApiResponse<InstallStatus>, String> {
    let installer = state.installer.lock().await;
    match installer.check_installation() {
        Ok(status) => Ok(ApiResponse::success(status)),
        Err(e) => Ok(ApiResponse::error(format!("检查安装状态失败: {}", e))),
    }
}

/// 安装 OpenClaw
#[tauri::command]
pub async fn install_openclaw(
    window: Window,
    _state: State<'_, InstallerState>,
    version: Option<String>,
    _network_preference: Option<String>,
) -> Result<ApiResponse<InstallResult>, String> {
    // 创建进度通道
    let (progress_tx, mut progress_rx) = mpsc::channel::<InstallProgress>(100);

    // 创建安装器
    let installer = OpenClawInstaller::new()
        .map_err(|e| format!("创建安装器失败: {}", e))?
        .with_progress_channel(progress_tx);

    let version_ref = version.as_deref();

    // 在单独任务中发送进度事件
    let window_clone = window.clone();
    tokio::spawn(async move {
        while let Some(progress) = progress_rx.recv().await {
            let _ = window_clone.emit(
                "install-progress",
                serde_json::json!({
                    "stage": progress.stage.to_string(),
                    "percentage": progress.percentage,
                    "message": progress.message,
                }),
            );
        }
    });

    match installer.install(version_ref, None).await {
        Ok(result) => Ok(ApiResponse::success(result)),
        Err(e) => Ok(ApiResponse::error(format!("安装失败: {}", e))),
    }
}

/// 一键安装 OpenClaw（Molili 风格全栈打包）
/// 包含：嵌入式 Runtime + OpenClaw + 国产模型预设
#[tauri::command]
pub async fn install_openclaw_one_click(
    window: Window,
    _state: State<'_, InstallerState>,
    use_offline_package: Option<bool>,
) -> Result<ApiResponse<InstallResult>, String> {
    // 创建进度通道
    let (progress_tx, mut progress_rx) = mpsc::channel::<InstallProgress>(100);

    // 创建安装器
    let installer = OpenClawInstaller::new()
        .map_err(|e| format!("创建安装器失败: {}", e))?
        .with_progress_channel(progress_tx);

    // 在单独任务中发送进度事件
    let window_clone = window.clone();
    tokio::spawn(async move {
        while let Some(progress) = progress_rx.recv().await {
            let _ = window_clone.emit(
                "install-progress",
                serde_json::json!({
                    "stage": progress.stage.to_string(),
                    "percentage": progress.percentage,
                    "message": progress.message,
                }),
            );
        }
    });

    // 构建安装选项
    let options = InstallAllOptions {
        use_offline_package: use_offline_package.unwrap_or(true),
        custom_install_dir: None,
        skip_runtime: false,
    };

    match installer.install_all(options).await {
        Ok(result) => Ok(ApiResponse::success(result)),
        Err(e) => Ok(ApiResponse::error(format!("一键安装失败: {}", e))),
    }
}

/// 卸载 OpenClaw
#[tauri::command]
pub async fn uninstall_openclaw(
    state: State<'_, InstallerState>,
) -> Result<ApiResponse<bool>, String> {
    let installer = state.installer.lock().await;
    match installer.uninstall() {
        Ok(_) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(format!("卸载失败: {}", e))),
    }
}

/// 获取 OpenClaw 配置
#[tauri::command]
pub async fn get_openclaw_config(
    state: State<'_, InstallerState>,
) -> Result<ApiResponse<OpenClawConfig>, String> {
    let installer = state.installer.lock().await;
    match installer.read_config() {
        Ok(config) => Ok(ApiResponse::success(config)),
        Err(e) => Ok(ApiResponse::error(format!("读取配置失败: {}", e))),
    }
}

/// 更新 OpenClaw 配置
#[tauri::command]
pub async fn update_openclaw_config(
    state: State<'_, InstallerState>,
    config: OpenClawConfig,
) -> Result<ApiResponse<OpenClawConfig>, String> {
    let installer = state.installer.lock().await;
    match installer.write_config(&config) {
        Ok(_) => Ok(ApiResponse::success(config)),
        Err(e) => Ok(ApiResponse::error(format!("保存配置失败: {}", e))),
    }
}

/// 启动 OpenClaw 服务
#[tauri::command]
pub async fn start_openclaw_service(
    state: State<'_, InstallerState>,
) -> Result<ApiResponse<bool>, String> {
    let installer = state.installer.lock().await;
    match installer.start_service().await {
        Ok(_) => Ok(ApiResponse::success(true)),
        Err(e) => Ok(ApiResponse::error(format!("启动服务失败: {}", e))),
    }
}

/// 系统环境检查结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemEnvironmentCheckResult {
    pub checks: Vec<SystemCheckResult>,
    pub can_install: bool,
    pub missing_dependencies: Vec<String>,
}

/// 检查系统环境
#[tauri::command]
pub async fn check_system_environment(
    state: State<'_, InstallerState>,
) -> Result<ApiResponse<SystemEnvironmentCheckResult>, String> {
    let installer = state.installer.lock().await;
    match installer.check_system_environment().await {
        Ok(checks) => {
            let missing: Vec<String> = checks
                .iter()
                .filter(|c| c.required && !c.passed)
                .map(|c| c.name.clone())
                .collect();
            let can_install = missing.is_empty();

            Ok(ApiResponse::success(SystemEnvironmentCheckResult {
                checks,
                can_install,
                missing_dependencies: missing,
            }))
        }
        Err(e) => Ok(ApiResponse::error(format!("系统环境检查失败: {}", e))),
    }
}

/// 执行 OpenClaw 命令
#[tauri::command]
pub async fn execute_openclaw_command(
    state: State<'_, InstallerState>,
    command: String,
    args: Option<Vec<String>>,
) -> Result<ApiResponse<String>, String> {
    let installer = state.installer.lock().await;
    let args_vec = args.unwrap_or_default();

    match installer.execute_command(&command, &args_vec).await {
        Ok(output) => Ok(ApiResponse::success(output)),
        Err(e) => Ok(ApiResponse::error(format!("命令执行失败: {}", e))),
    }
}

/// 离线安装 OpenClaw
#[tauri::command]
pub async fn install_openclaw_offline(
    window: Window,
    state: State<'_, InstallerState>,
) -> Result<ApiResponse<InstallResult>, String> {
    // 创建进度通道
    let (progress_tx, mut progress_rx) = mpsc::channel::<InstallProgress>(100);

    // 在单独任务中发送进度事件
    let window_clone = window.clone();
    tokio::spawn(async move {
        while let Some(progress) = progress_rx.recv().await {
            let _ = window_clone.emit(
                "install-progress",
                serde_json::json!({
                    "stage": progress.stage.to_string(),
                    "percentage": progress.percentage,
                    "message": progress.message,
                }),
            );
        }
    });

    let service: tokio::sync::MutexGuard<'_, InstallerService> = state.service.lock().await;
    log::info!("开始离线安装...");
    match service.install(InstallMethod::Offline, None, Some(progress_tx)).await {
        Ok(result) => {
            log::info!("离线安装成功: {:?}", result);
            Ok(ApiResponse::success(result))
        }
        Err(e) => {
            let error_msg = format!("离线安装失败: {}", e);
            log::error!("{}", error_msg);
            // 打印完整的错误链
            let mut source = e.source();
            while let Some(s) = source {
                log::error!("  原因: {}", s);
                source = s.source();
            }
            Ok(ApiResponse::error(error_msg))
        }
    }
}

/// 获取可用的安装方法
#[tauri::command]
pub async fn get_install_methods() -> Result<ApiResponse<Vec<InstallMethodInfo>>, String> {
    let methods = InstallerService::get_install_methods();
    Ok(ApiResponse::success(methods))
}

/// 获取 OpenClaw 配置（检查是否已安装）
#[tauri::command]
pub async fn get_openclaw_config_if_installed(
    state: State<'_, InstallerState>,
) -> Result<ApiResponse<Option<OpenClawConfig>>, String> {
    let installer = state.installer.lock().await;

    // 首先检查是否已安装
    match installer.check_installation() {
        Ok(InstallStatus::Installed { .. }) => {
            // 尝试读取配置文件
            let config_path = installer.get_install_dir().join("config.yaml");
            if config_path.exists() {
                match std::fs::read_to_string(&config_path) {
                    Ok(content) => {
                        match OpenClawConfig::from_yaml(&content) {
                            Ok(config) => Ok(ApiResponse::success(Some(config))),
                            Err(e) => Ok(ApiResponse::error(format!("解析配置失败: {}", e))),
                        }
                    }
                    Err(e) => Ok(ApiResponse::error(format!("读取配置失败: {}", e))),
                }
            } else {
                // 返回默认配置
                Ok(ApiResponse::success(Some(OpenClawConfig::default_config())))
            }
        }
        _ => Ok(ApiResponse::success(None)),
    }
}

/// 获取模型列表
#[tauri::command]
pub async fn get_openclaw_models(
    state: State<'_, InstallerState>,
) -> Result<ApiResponse<Vec<ModelConfig>>, String> {
    let config_result = get_openclaw_config_if_installed(state).await?;

    match config_result.data {
        Some(Some(config)) => Ok(ApiResponse::success(config.models)),
        _ => Ok(ApiResponse::success(vec![])),
    }
}

/// 获取 Agent 列表
#[tauri::command]
pub async fn get_openclaw_agents(
    state: State<'_, InstallerState>,
) -> Result<ApiResponse<Vec<AgentConfig>>, String> {
    let config_result = get_openclaw_config_if_installed(state).await?;

    match config_result.data {
        Some(Some(config)) => Ok(ApiResponse::success(config.agents)),
        _ => Ok(ApiResponse::success(vec![])),
    }
}

/// OpenClaw 运行状态
#[derive(Debug, Clone, serde::Serialize)]
pub struct OpenClawRuntimeStatus {
    /// 是否已安装
    pub installed: bool,
    /// 是否有进程在运行
    pub running: bool,
    /// 版本号（如果已安装）
    pub version: Option<String>,
    /// 安装路径（如果已安装）
    pub install_path: Option<String>,
}

/// 检查 OpenClaw 运行状态
#[tauri::command]
pub async fn is_openclaw_running(
    state: State<'_, InstallerState>,
) -> Result<ApiResponse<OpenClawRuntimeStatus>, String> {
    let installer = state.installer.lock().await;

    // 1. 检查是否已安装
    let install_status = match installer.check_installation() {
        Ok(status) => status,
        Err(e) => {
            return Ok(ApiResponse::success(OpenClawRuntimeStatus {
                installed: false,
                running: false,
                version: None,
                install_path: None,
            }));
        }
    };

    let (installed, version) = match install_status {
        InstallStatus::Installed { version } => (true, Some(version)),
        _ => (false, None),
    };

    let install_path = if installed {
        Some(installer.get_install_dir().to_string_lossy().to_string())
    } else {
        None
    };

    // 2. 检查是否有进程在运行
    let process_name = if cfg!(target_os = "windows") {
        "openclaw.exe"
    } else {
        "openclaw"
    };

    // 使用多种方法检测进程
    let running = check_process_running(process_name);

    Ok(ApiResponse::success(OpenClawRuntimeStatus {
        installed,
        running,
        version,
        install_path,
    }))
}

/// 检查进程是否在运行（内部函数）
/// 检查 openclaw 及其相关进程（包括 npm 安装的 node 进程）
fn check_process_running(_process_name: &str) -> bool {
    // OpenClaw 可能有多种运行方式：
    // 1. 原生二进制: openclaw, openclaw-gateway
    // 2. npm 安装: node .../openclaw/dist/index.js

    // 方法1: 检测 npm 安装的 openclaw (node .../openclaw/dist/index.js)
    #[cfg(unix)]
    {
        if let Ok(output) = std::process::Command::new("pgrep")
            .args(["-f", "openclaw/dist/index.js"])
            .output()
        {
            if output.status.success() && !output.stdout.is_empty() {
                log::info!("Found npm-installed openclaw process");
                return true;
            }
        }

        // 方法2: 检测原生二进制进程
        let patterns = ["openclaw", "openclaw-gateway", "openclaw-agent"];
        for pattern in &patterns {
            // 使用 -x 精确匹配主进程名
            if let Ok(output) = std::process::Command::new("pgrep")
                .args(["-x", pattern])
                .output()
            {
                if output.status.success() && !output.stdout.is_empty() {
                    log::info!("Found running process: {}", pattern);
                    return true;
                }
            }

            // 使用 -f 匹配完整命令行
            if let Ok(output) = std::process::Command::new("pgrep")
                .args(["-f", pattern])
                .output()
            {
                if output.status.success() && !output.stdout.is_empty() {
                    // 检查是否不是当前应用进程
                    let pids = String::from_utf8_lossy(&output.stdout);
                    for pid in pids.lines() {
                        if let Ok(pid_num) = pid.trim().parse::<u32>() {
                            // 排除当前进程
                            if pid_num != std::process::id() {
                                log::info!("Found running process (full match): {} (PID: {})", pattern, pid_num);
                                return true;
                            }
                        }
                    }
                }
            }
        }

        // 方法3: 使用 ps + grep 检查
        if let Ok(output) = std::process::Command::new("sh")
            .args(["-c", "ps aux | grep -v grep | grep -E 'openclaw' | grep -v 'openclaw-manager'"])
            .output()
        {
            if !output.stdout.is_empty() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                log::info!("Found openclaw processes:\n{}", output_str);
                return true;
            }
        }
    }

    // 方法4: 使用 pidof
    #[cfg(unix)]
    {
        let patterns = ["openclaw", "openclaw-gateway", "openclaw-agent"];
        for pattern in &patterns {
            if let Ok(output) = std::process::Command::new("pidof")
                .arg(pattern)
                .output()
            {
                if output.status.success() && !output.stdout.is_empty() {
                    log::info!("Found running process with pidof: {}", pattern);
                    return true;
                }
            }
        }
    }

    // Windows: 使用 tasklist
    #[cfg(windows)]
    {
        if let Ok(output) = std::process::Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq openclaw*"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.to_lowercase().contains("openclaw") {
                return true;
            }
        }
    }

    false
}

/// OpenClaw 进程详细信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct OpenClawProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub status: String,
    pub start_time: Option<String>,
}

/// 获取 OpenClaw 进程详细信息
#[tauri::command]
pub async fn get_openclaw_process_info() -> Result<ApiResponse<Vec<OpenClawProcessInfo>>, String> {
    let mut all_processes = Vec::new();

    // 使用 ps 命令获取详细信息
    #[cfg(unix)]
    {
        // 首先检测 npm 安装的 openclaw (node .../openclaw/dist/index.js)
        if let Ok(output) = std::process::Command::new("pgrep")
            .args(["-a", "-f", "openclaw/dist/index.js"])
            .output()
        {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(pid) = parts[0].parse::<u32>() {
                            // 排除当前应用进程
                            if pid != std::process::id() {
                                let cmd = parts[1..].join(" ");
                                all_processes.push(OpenClawProcessInfo {
                                    pid,
                                    name: "openclaw (npm)".to_string(),
                                    cpu_percent: 0.0,
                                    memory_mb: 0,
                                    status: "running".to_string(),
                                    start_time: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        // 然后检测原生二进制进程
        let process_patterns = ["openclaw", "openclaw-gateway", "openclaw-agent"];
        for pattern in &process_patterns {
            // macOS: ps -o pid,pcpu,pmem,time,command -c <pattern>
            let ps_args = if cfg!(target_os = "macos") {
                vec!["-o", "pid,pcpu,pmem,time,command", "-c", pattern]
            } else {
                vec!["-o", "pid,pcpu,pmem,time,etime,comm", "-C", pattern]
            };

            match std::process::Command::new("ps")
                .args(&ps_args)
                .output()
            {
                Ok(result) => {
                    if !result.status.success() {
                        continue;
                    }

                    let output_str = String::from_utf8_lossy(&result.stdout);
                    if output_str.trim().is_empty() {
                        continue;
                    }

                    log::info!("ps output for {}: {}", pattern, output_str);

                    // 解析 ps 输出
                    // 格式: PID %CPU %MEM TIME COMMAND
                    for line in output_str.lines().skip(1) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 5 {
                            if let Ok(pid) = parts[0].parse::<u32>() {
                                let cpu = parts.get(1)
                                    .and_then(|s| s.parse::<f32>().ok())
                                    .unwrap_or(0.0);
                                let mem = parts.get(2)
                                    .and_then(|s| s.parse::<f32>().ok())
                                    .unwrap_or(0.0) as u64;

                                // 获取进程名（最后一部分）
                                let name = parts.last().unwrap_or(&pattern).to_string();

                                all_processes.push(OpenClawProcessInfo {
                                    pid,
                                    name,
                                    cpu_percent: cpu,
                                    memory_mb: mem,
                                    status: "running".to_string(),
                                    start_time: parts.get(3).map(|s| s.to_string()),
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to run ps command for {}: {}", pattern, e);
                }
            }
        }

        // 如果没有找到，尝试使用 pgrep 获取所有 openclaw 相关进程
        if all_processes.is_empty() {
            if let Ok(output) = std::process::Command::new("pgrep")
                .args(["-a", "openclaw"])
                .output()
            {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    for line in output_str.lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            if let Ok(pid) = parts[0].parse::<u32>() {
                                let name = parts[1..].join(" ");
                                all_processes.push(OpenClawProcessInfo {
                                    pid,
                                    name,
                                    cpu_percent: 0.0,
                                    memory_mb: 0,
                                    status: "running".to_string(),
                                    start_time: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(windows)]
    {
        // Windows: 使用 tasklist 检查所有 openclaw 相关进程
        for pattern in &process_patterns {
            if let Ok(output) = std::process::Command::new("tasklist")
                .args(["/FI", &format!("IMAGENAME eq {}*", pattern), "/FO", "CSV"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines().skip(1) {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 2 {
                        if let Ok(pid) = parts[1].trim_matches('"').parse::<u32>() {
                            all_processes.push(OpenClawProcessInfo {
                                pid,
                                name: pattern.to_string(),
                                cpu_percent: 0.0,
                                memory_mb: 0,
                                status: "running".to_string(),
                                start_time: None,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(ApiResponse::success(all_processes))
}
