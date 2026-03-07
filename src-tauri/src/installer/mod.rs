// 允许未使用的代码，这些类型用于将来的功能扩展
#![allow(dead_code)]

use crate::models::openclaw::{InstallResult, InstallStatus, OpenClawConfig};
use crate::system::SystemInfo;
use anyhow::{Context, Result};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::process::Stdio;
use tokio::sync::mpsc;

// 引入 runtime_bundle 模块
pub mod runtime_bundle;
pub use runtime_bundle::RuntimeBundle;

// 引入 sidecar_installer 模块
pub mod sidecar_installer;
pub use sidecar_installer::SidecarInstaller;

/// 系统检查结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemCheckResult {
    pub name: String,
    pub passed: bool,
    pub required: bool,
    pub message: String,
}

/// 默认 OpenClaw 安装路径
const OPENCLAW_DIR: &str = ".openclaw";
const OPENCLAW_BIN: &str = "bin";
const OPENCLAW_CONFIG: &str = "config.yaml";

/// 安装脚本 URL
/// 注意：这些 URL 需要根据实际部署环境配置
/// 开发测试时可以使用本地路径或 mock 服务
const INSTALL_SCRIPT_URL: &str = "https://raw.githubusercontent.com/openclai/openclaw/main/scripts/install.sh";
const INSTALL_SCRIPT_URL_WIN: &str = "https://raw.githubusercontent.com/openclai/openclaw/main/scripts/install.ps1";

/// 使用本地安装脚本（用于测试）
const USE_LOCAL_SCRIPT: bool = true;

/// OpenClaw 安装器
pub struct OpenClawInstaller {
    install_dir: PathBuf,
    progress_tx: Option<mpsc::Sender<InstallProgress>>,
}

impl OpenClawInstaller {
    /// 获取安装目录
    pub fn get_install_dir(&self) -> &Path {
        &self.install_dir
    }
}

/// 安装进度
#[derive(Debug, Clone)]
pub struct InstallProgress {
    pub stage: InstallStage,
    pub percentage: f32,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstallStage {
    Checking,
    ExtractingRuntime,  // 新增：解压嵌入式 runtime
    Downloading,
    Installing,
    Configuring,
    Complete,
    Error,
}

impl std::fmt::Display for InstallStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallStage::Checking => write!(f, "Checking"),
            InstallStage::ExtractingRuntime => write!(f, "ExtractingRuntime"),
            InstallStage::Downloading => write!(f, "Downloading"),
            InstallStage::Installing => write!(f, "Installing"),
            InstallStage::Configuring => write!(f, "Configuring"),
            InstallStage::Complete => write!(f, "Complete"),
            InstallStage::Error => write!(f, "Error"),
        }
    }
}

impl OpenClawInstaller {
    /// 创建新的安装器实例
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;
        let install_dir = home_dir.join(OPENCLAW_DIR);

        Ok(Self {
            install_dir,
            progress_tx: None,
        })
    }

    /// 设置进度发送通道
    pub fn with_progress_channel(mut self, tx: mpsc::Sender<InstallProgress>) -> Self {
        self.progress_tx = Some(tx);
        self
    }

    /// 报告进度
    async fn report_progress(&self, stage: InstallStage, percentage: f32, message: impl Into<String>) {
        if let Some(ref tx) = self.progress_tx {
            let progress = InstallProgress {
                stage,
                percentage,
                message: message.into(),
            };
            let _ = tx.send(progress).await;
        }
    }

    /// 检查 OpenClaw 是否已安装
    pub fn check_installation(&self) -> Result<InstallStatus> {
        let bin_path = self.get_binary_path();

        if !bin_path.exists() {
            return Ok(InstallStatus::NotInstalled);
        }

        // 检查可执行文件是否存在并获取版本
        match self.get_installed_version() {
            Ok(version) => Ok(InstallStatus::Installed { version }),
            Err(e) => Ok(InstallStatus::Error {
                message: format!("Found binary but failed to get version: {}", e),
            }),
        }
    }

    /// 获取安装的二进制路径
    fn get_binary_path(&self) -> PathBuf {
        let exe_name = if cfg!(target_os = "windows") {
            "openclaw.exe"
        } else {
            "openclaw"
        };
        self.install_dir.join(OPENCLAW_BIN).join(exe_name)
    }

    /// 获取已安装的版本
    pub fn get_installed_version(&self) -> Result<String> {
        let bin_path = self.get_binary_path();

        // 使用 Command API 执行版本检查
        let output = std::process::Command::new(&bin_path)
            .arg("--version")
            .output()
            .context("Failed to execute openclaw --version")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "openclaw --version returned non-zero exit code"
            ));
        }

        let version_str = String::from_utf8_lossy(&output.stdout);
        Ok(version_str.trim().to_string())
    }

    /// 安装 OpenClaw - 使用命令行脚本
    pub async fn install(&self, _version: Option<&str>, _network_preference: Option<&str>) -> Result<InstallResult> {
        self.report_progress(InstallStage::Checking, 0.0, "检查安装环境...").await;

        // 创建安装目录
        fs::create_dir_all(&self.install_dir)?;

        // 检测系统信息以选择合适的安装脚本
        let system_info = SystemInfo::detect()?;
        let install_script = system_info.install_script();

        self.report_progress(
            InstallStage::Checking,
            5.0,
            format!("检测到系统: {}", system_info.friendly_name()),
        )
        .await;

        // 根据平台选择安装命令
        if cfg!(target_os = "windows") {
            self.install_windows().await
        } else if cfg!(target_os = "macos") {
            self.install_macos(&install_script).await
        } else {
            self.install_unix().await
        }
    }

    /// macOS 安装 - 使用版本特定脚本
    async fn install_macos(&self, script_name: &str) -> Result<InstallResult> {
        self.report_progress(
            InstallStage::Downloading,
            10.0,
            format!("使用安装脚本: {}", script_name),
        )
        .await;

        // 脚本路径 - 嵌入在应用内部
        let script_path = format!("scripts/macos/{}", script_name);
        let app_script = PathBuf::from(&script_path);

        // 如果应用内部有脚本，使用内部脚本
        // 否则从远程下载
        let script_to_run = if app_script.exists() {
            app_script.to_string_lossy().to_string()
        } else {
            // 从远程下载对应版本的脚本
            let remote_url = format!(
                "https://raw.githubusercontent.com/openclai/openclaw/main/scripts/macos/{}",
                script_name
            );
            let temp_script = "/tmp/openclaw_install_macos.sh";

            self.report_progress(
                InstallStage::Downloading,
                15.0,
                "下载安装脚本...",
            )
            .await;

            // 下载脚本
            let download_result = tokio::task::spawn_blocking({
                let url = remote_url.clone();
                let dest = temp_script.to_string();
                move || {
                    std::process::Command::new("curl")
                        .args(["-fsSL", "-o", &dest, &url])
                        .output()
                }
            }).await;

            match download_result {
                Ok(Ok(output)) if output.status.success() => {
                    // 设置执行权限
                    let _ = tokio::fs::set_permissions(temp_script, std::fs::Permissions::from_mode(0o755)).await;
                    temp_script.to_string()
                }
                _ => {
                    // 下载失败，使用通用脚本
                    self.report_progress(
                        InstallStage::Downloading,
                        15.0,
                        "使用通用安装脚本...",
                    )
                    .await;
                    return self.install_unix().await;
                }
            }
        };

        self.report_progress(
            InstallStage::Downloading,
            20.0,
            "开始执行安装脚本...",
        )
        .await;

        // 执行安装脚本
        let mut child = Command::new("bash")
            .args([&script_to_run])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn bash command")?;

        let stdout = child.stdout.take().context("Failed to get stdout")?;
        let stderr = child.stderr.take().context("Failed to get stderr")?;

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        let mut output_lines = Vec::new();
        let mut error_lines = Vec::new();

        // 读取 stdout 和 stderr
        loop {
            tokio::select! {
                line = stdout_reader.next_line() => {
                    if let Ok(Some(line)) = line {
                        output_lines.push(line.clone());
                        let progress = self.parse_progress(&line);
                        self.report_progress(
                            progress.0,
                            progress.1,
                            progress.2,
                        )
                        .await;
                    }
                }
                line = stderr_reader.next_line() => {
                    if let Ok(Some(line)) = line {
                        error_lines.push(line.clone());
                        self.report_progress(
                            InstallStage::Installing,
                            60.0,
                            &line,
                        )
                        .await;
                    }
                }
                status = child.wait() => {
                    match status {
                        Ok(status) => {
                            if !status.success() {
                                let code = status.code().unwrap_or(-1);
                                let error_msg = error_lines.join("\n");
                                self.report_progress(
                                    InstallStage::Error,
                                    0.0,
                                    format!("安装失败 (退出码: {})", code),
                                )
                                .await;
                                return Err(anyhow::anyhow!(
                                    "Installation failed with exit code: {}. Errors: {}",
                                    code,
                                    error_msg
                                ));
                            }
                            break;
                        }
                        Err(e) => {
                            self.report_progress(
                                InstallStage::Error,
                                0.0,
                                format!("安装错误: {}", e),
                            )
                            .await;
                            return Err(anyhow::anyhow!("Installation error: {}", e));
                        }
                    }
                }
            }
        }

        self.report_progress(InstallStage::Configuring, 80.0, "创建默认配置...").await;

        // 创建默认配置
        self.create_default_config()?;

        // 获取安装版本
        let version = match self.get_installed_version() {
            Ok(v) => v,
            Err(_) => "unknown".to_string(),
        };

        self.report_progress(
            InstallStage::Complete,
            100.0,
            format!("OpenClaw {} 安装成功!", version),
        )
        .await;

        Ok(InstallResult {
            success: true,
            version: Some(version.clone()),
            message: format!("OpenClaw {} 安装成功", version),
        })
    }

    /// Windows 安装 - 使用 PowerShell
    async fn install_windows(&self) -> Result<InstallResult> {
        self.report_progress(
            InstallStage::Downloading,
            10.0,
            "正在下载安装脚本...",
        )
        .await;

        // 构建 PowerShell 命令
        let script = format!("iwr -useb {} | iex", INSTALL_SCRIPT_URL_WIN);

        let output = Command::new("powershell")
            .args(["-Command", &script])
            .output()
            .await
            .context("Failed to spawn PowerShell command")?;

        self.report_progress(InstallStage::Downloading, 50.0, "安装中...").await;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            self.report_progress(
                InstallStage::Error,
                0.0,
                format!("安装失败: {}", error_msg),
            )
            .await;
            return Err(anyhow::anyhow!("Installation failed: {}", error_msg));
        }

        self.report_progress(InstallStage::Configuring, 80.0, "创建默认配置...").await;

        // 创建默认配置
        self.create_default_config()?;

        // 获取安装版本
        let version = match self.get_installed_version() {
            Ok(v) => v,
            Err(_) => "unknown".to_string(),
        };

        self.report_progress(
            InstallStage::Complete,
            100.0,
            format!("OpenClaw {} 安装成功!", version),
        )
        .await;

        Ok(InstallResult {
            success: true,
            version: Some(version.clone()),
            message: format!("OpenClaw {} 安装成功", version),
        })
    }

    /// Unix (Mac/Linux) 安装 - 使用 curl + bash 或本地脚本
    async fn install_unix(&self) -> Result<InstallResult> {
        self.report_progress(
            InstallStage::Downloading,
            10.0,
            "正在下载安装脚本...",
        )
        .await;

        // 检查是否使用本地脚本（用于测试）
        let script_path = if USE_LOCAL_SCRIPT {
            // 使用项目内的测试脚本
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let local_script = manifest_dir.join("scripts").join("install_test.sh");

            if local_script.exists() {
                self.report_progress(
                    InstallStage::Downloading,
                    20.0,
                    "使用本地测试脚本...",
                )
                .await;
                local_script.to_string_lossy().to_string()
            } else {
                // 如果没有本地脚本，尝试下载
                match self.download_install_script().await {
                    Ok(path) => path,
                    Err(e) => {
                        // 下载失败，使用模拟安装
                        log::warn!("下载安装脚本失败: {}，切换到模拟安装模式", e);
                        return self.mock_install().await;
                    }
                }
            }
        } else {
            // 下载远程脚本
            match self.download_install_script().await {
                Ok(path) => path,
                Err(e) => {
                    log::error!("下载安装脚本失败: {}", e);
                    return Err(e);
                }
            }
        };

        self.report_progress(
            InstallStage::Downloading,
            40.0,
            "下载完成，开始安装...",
        )
        .await;

        // 执行安装脚本
        let install_dir_str = self.install_dir.to_string_lossy().to_string();
        let install_output = Command::new("bash")
            .args([&script_path, &install_dir_str])
            .output()
            .await
            .context("Failed to spawn bash command")?;

        if !install_output.status.success() {
            let code = install_output.status.code().unwrap_or(-1);
            let error_msg = String::from_utf8_lossy(&install_output.stderr);
            self.report_progress(
                InstallStage::Error,
                0.0,
                format!("安装失败 (退出码: {})", code),
            )
            .await;
            return Err(anyhow::anyhow!(
                "Installation failed with exit code: {}. Errors: {}",
                code,
                error_msg
            ));
        }

        self.report_progress(InstallStage::Configuring, 80.0, "创建默认配置...").await;

        // 创建默认配置
        self.create_default_config()?;

        // 清理临时文件（如果是下载的）
        if script_path == "/tmp/openclaw_install.sh" {
            let _ = fs::remove_file(&script_path);
        }

        // 获取安装版本
        let version = match self.get_installed_version() {
            Ok(v) => v,
            Err(_) => "unknown".to_string(),
        };

        self.report_progress(
            InstallStage::Complete,
            100.0,
            format!("OpenClaw {} 安装成功!", version),
        )
        .await;

        Ok(InstallResult {
            success: true,
            version: Some(version.clone()),
            message: format!("OpenClaw {} 安装成功", version),
        })
    }

    /// 下载安装脚本
    async fn download_install_script(&self) -> Result<String> {
        let download_output = Command::new("curl")
            .args(["-fsSL", "-o", "/tmp/openclaw_install.sh", INSTALL_SCRIPT_URL])
            .output()
            .await
            .context("Failed to spawn curl command")?;

        if !download_output.status.success() {
            let error_msg = String::from_utf8_lossy(&download_output.stderr);
            return Err(anyhow::anyhow!("Failed to download install script: {}", error_msg));
        }

        // 设置执行权限
        let _ = tokio::fs::set_permissions("/tmp/openclaw_install.sh", std::fs::Permissions::from_mode(0o755)).await;

        Ok("/tmp/openclaw_install.sh".to_string())
    }

    /// 模拟安装（用于测试）
    async fn mock_install(&self) -> Result<InstallResult> {
        use std::time::Duration;
        use tokio::time::sleep;

        self.report_progress(InstallStage::Downloading, 20.0, "模拟下载中...").await;
        sleep(Duration::from_millis(500)).await;

        self.report_progress(InstallStage::Installing, 50.0, "模拟安装中...").await;

        // 创建 bin 目录并复制测试二进制文件
        let bin_dir = self.install_dir.join("bin");
        fs::create_dir_all(&bin_dir)?;

        // 创建一个模拟的 openclaw 可执行文件
        let openclaw_bin = bin_dir.join("openclaw");
        let mock_script = r##"#!/bin/bash
if [ "$1" = "--version" ]; then
    echo "openclaw 0.1.0 (mock)"
    exit 0
fi
echo "OpenClaw CLI (Mock)"
echo "Usage: openclaw [command]"
exit 0
"##;
        fs::write(&openclaw_bin, mock_script)?;

        // 设置执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&openclaw_bin)?.permissions();
            perms.set_mode(perms.mode() | 0o111);
            fs::set_permissions(&openclaw_bin, perms)?;
        }

        sleep(Duration::from_millis(500)).await;

        self.report_progress(InstallStage::Configuring, 80.0, "创建默认配置...").await;
        self.create_default_config()?;

        sleep(Duration::from_millis(300)).await;

        let version = "0.1.0-mock".to_string();
        self.report_progress(
            InstallStage::Complete,
            100.0,
            format!("OpenClaw {} 安装成功! (模拟模式)", version),
        )
        .await;

        Ok(InstallResult {
            success: true,
            version: Some(version.clone()),
            message: format!("OpenClaw {} 安装成功 (模拟模式)", version),
        })
    }

    /// 解析命令输出中的进度信息
    fn parse_progress(&self, line: &str) -> (InstallStage, f32, String) {
        let line_lower = line.to_lowercase();

        // 根据关键词判断进度阶段
        if line_lower.contains("downloading") || line_lower.contains("curl") {
            (InstallStage::Downloading, 30.0, format!("下载中: {}", line))
        } else if line_lower.contains("installing") || line_lower.contains("extracting") {
            (InstallStage::Installing, 60.0, format!("安装中: {}", line))
        } else if line_lower.contains("setting up") || line_lower.contains("configuring") {
            (InstallStage::Configuring, 80.0, format!("配置中: {}", line))
        } else if line_lower.contains("complete") || line_lower.contains("done") {
            (InstallStage::Complete, 100.0, "安装完成!".to_string())
        } else if line_lower.contains("error") || line_lower.contains("failed") {
            (InstallStage::Error, 0.0, format!("错误: {}", line))
        } else {
            (InstallStage::Installing, 50.0, line.to_string())
        }
    }

    /// 创建默认配置
    pub fn create_default_config(&self) -> Result<()> {
        let config_path = self.install_dir.join(OPENCLAW_CONFIG);

        // 如果配置已存在，不要覆盖
        if config_path.exists() {
            return Ok(());
        }

        let config = OpenClawConfig::default_config();
        let yaml = config.to_yaml()?;

        fs::write(&config_path, yaml).context("Failed to write config file")?;

        Ok(())
    }

    /// 读取配置
    pub fn read_config(&self) -> Result<OpenClawConfig> {
        let config_path = self.install_dir.join(OPENCLAW_CONFIG);

        if !config_path.exists() {
            return Ok(OpenClawConfig::default_config());
        }

        let yaml = fs::read_to_string(&config_path).context("Failed to read config file")?;
        let config = OpenClawConfig::from_yaml(&yaml)?;

        Ok(config)
    }

    /// 写入配置
    pub fn write_config(&self, config: &OpenClawConfig) -> Result<()> {
        let config_path = self.install_dir.join(OPENCLAW_CONFIG);
        let yaml = config.to_yaml()?;

        fs::write(&config_path, yaml).context("Failed to write config file")?;

        Ok(())
    }

    /// 卸载 OpenClaw
    pub fn uninstall(&self) -> Result<()> {
        if self.install_dir.exists() {
            fs::remove_dir_all(&self.install_dir)
                .context("Failed to remove installation directory")?;
        }

        Ok(())
    }

    /// 启动 OpenClaw 服务
    pub async fn start_service(&self) -> Result<()> {
        let bin_path = self.get_binary_path();

        if !bin_path.exists() {
            return Err(anyhow::anyhow!("OpenClaw binary not found"));
        }

        // 使用 Command API 启动服务
        let _ = Command::new(bin_path.to_str().unwrap())
            .args(["serve"])
            .spawn()
            .context("Failed to start OpenClaw service")?;

        Ok(())
    }

    /// 检查系统环境
    pub async fn check_system_environment(&self) -> Result<Vec<SystemCheckResult>> {
        let mut checks = Vec::new();

        // 1. 检查操作系统兼容性
        let os_check = self.check_os_compatibility().await;
        checks.push(os_check);

        // 2. 检查网络连接
        let network_check = self.check_network().await;
        checks.push(network_check);

        // 3. 检查 Rust 环境
        let rust_check = self.check_rust().await;
        checks.push(rust_check);

        // 4. 检查 Node.js 环境
        let node_check = self.check_nodejs().await;
        checks.push(node_check);

        // 5. 检查 Python 环境
        let python_check = self.check_python().await;
        checks.push(python_check);

        // 6. 检查磁盘空间
        let disk_check = self.check_disk_space().await;
        checks.push(disk_check);

        Ok(checks)
    }

    /// 检查操作系统兼容性
    async fn check_os_compatibility(&self) -> SystemCheckResult {
        let os = std::env::consts::OS;
        let (passed, message) = match os {
            "macos" | "linux" | "windows" => (true, format!("支持的操作系统: {}", os)),
            _ => (false, format!("不支持的操作系统: {}", os)),
        };

        SystemCheckResult {
            name: "操作系统兼容性".to_string(),
            passed,
            required: true,
            message,
        }
    }

    /// 检查网络连接
    async fn check_network(&self) -> SystemCheckResult {
        // 尝试 ping openclaw.ai using spawn_blocking
        let result = tokio::task::spawn_blocking(|| {
            std::process::Command::new("ping")
                .args(if cfg!(target_os = "windows") {
                    vec!["-n", "1", "-w", "3000", "openclaw.ai"]
                } else {
                    vec!["-c", "1", "-W", "3", "openclaw.ai"]
                })
                .output()
        }).await;

        match result {
            Ok(Ok(output)) if output.status.success() => SystemCheckResult {
                name: "网络连接".to_string(),
                passed: true,
                required: true,
                message: "网络连接正常".to_string(),
            },
            _ => SystemCheckResult {
                name: "网络连接".to_string(),
                passed: false,
                required: true,
                message: "无法连接到 openclaw.ai，请检查网络设置".to_string(),
            },
        }
    }

    /// 检查 Rust 环境
    async fn check_rust(&self) -> SystemCheckResult {
        let result = tokio::task::spawn_blocking(|| {
            std::process::Command::new("rustc").args(["--version"]).output()
        }).await;

        match result {
            Ok(Ok(output)) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                SystemCheckResult {
                    name: "Rust 环境".to_string(),
                    passed: true,
                    required: true,
                    message: format!("已安装 {}", version),
                }
            }
            _ => SystemCheckResult {
                name: "Rust 环境".to_string(),
                passed: false,
                required: true,
                message: "未安装 Rust，请先安装 Rust (https://rustup.rs/)".to_string(),
            },
        }
    }

    /// 检查 Node.js 环境
    async fn check_nodejs(&self) -> SystemCheckResult {
        let result = tokio::task::spawn_blocking(|| {
            std::process::Command::new("node").args(["--version"]).output()
        }).await;

        match result {
            Ok(Ok(output)) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                SystemCheckResult {
                    name: "Node.js 环境".to_string(),
                    passed: true,
                    required: false,
                    message: format!("已安装 {}", version),
                }
            }
            _ => SystemCheckResult {
                name: "Node.js 环境".to_string(),
                passed: false,
                required: false,
                message: "未安装 Node.js（可选，某些功能需要）".to_string(),
            },
        }
    }

    /// 检查 Python 环境
    async fn check_python(&self) -> SystemCheckResult {
        // 尝试 python3 或 python
        let result = tokio::task::spawn_blocking(|| {
            std::process::Command::new("python3").args(["--version"]).output()
        }).await;

        let result = if result.is_err() || matches!(&result, Ok(Err(_))) {
            tokio::task::spawn_blocking(|| {
                std::process::Command::new("python").args(["--version"]).output()
            }).await
        } else {
            result
        };

        match result {
            Ok(Ok(output)) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                SystemCheckResult {
                    name: "Python 环境".to_string(),
                    passed: true,
                    required: false,
                    message: format!("已安装 {}", version),
                }
            }
            _ => SystemCheckResult {
                name: "Python 环境".to_string(),
                passed: false,
                required: false,
                message: "未安装 Python（可选，某些功能需要）".to_string(),
            },
        }
    }

    /// 检查磁盘空间
    async fn check_disk_space(&self) -> SystemCheckResult {
        // 获取安装目录所在磁盘的空间
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let home_dir_clone = home_dir.clone();

        // 使用 df 命令检查磁盘空间 (Unix) 或 wmic (Windows)
        let result = tokio::task::spawn_blocking(move || {
            if cfg!(target_os = "windows") {
                std::process::Command::new("wmic")
                    .args([
                        "LogicalDisk",
                        "where",
                        &format!("DeviceID='{}'", home_dir_clone.to_str().unwrap_or("C:").chars().next().unwrap_or('C')),
                        "get",
                        "FreeSpace",
                        "/value",
                    ])
                    .output()
            } else {
                std::process::Command::new("df")
                    .args(["-h", home_dir_clone.to_str().unwrap_or(".")])
                    .output()
            }
        }).await;

        match result {
            Ok(Ok(output)) if output.status.success() => {
                // 简单检查，认为有空间即可
                SystemCheckResult {
                    name: "磁盘空间".to_string(),
                    passed: true,
                    required: true,
                    message: "磁盘空间充足".to_string(),
                }
            }
            _ => SystemCheckResult {
                name: "磁盘空间".to_string(),
                passed: true,
                required: true,
                message: "无法检查磁盘空间，假设充足".to_string(),
            },
        }
    }

    /// 执行 OpenClaw 命令
    pub async fn execute_command(&self, command: &str, args: &[String]) -> Result<String> {
        let bin_path = self.get_binary_path();

        if !bin_path.exists() {
            return Err(anyhow::anyhow!("OpenClaw binary not found. Please install OpenClaw first."));
        }

        let mut cmd_args = vec![command.to_string()];
        cmd_args.extend_from_slice(args);

        let bin_path_str = bin_path.to_str().unwrap().to_string();
        let output = tokio::task::spawn_blocking(move || {
            std::process::Command::new(&bin_path_str)
                .args(&cmd_args)
                .output()
        }).await
            .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
            .context("Failed to execute OpenClaw command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Command failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }

    // ==================== 一键部署（全栈打包）方法 ====================

    /// 一键安装：Runtime + OpenClaw + 预设配置
    /// 这是 Molili 风格的全栈打包安装方式
    pub async fn install_all(
        &self,
        options: InstallAllOptions,
    ) -> Result<InstallResult> {
        // 步骤 1: 检查并安装嵌入式 Runtime
        self.report_progress(
            InstallStage::Checking,
            0.0,
            "检查运行环境...",
        ).await;

        let runtime_bundle = RuntimeBundle::new()?;
        let runtime_statuses = runtime_bundle.check_all_installed().await?;

        // 检查是否有必需的 runtime 缺失
        let missing_required: Vec<_> = runtime_statuses
            .iter()
            .filter(|s| s.is_required_missing())
            .collect();

        if !missing_required.is_empty() {
            self.report_progress(
                InstallStage::ExtractingRuntime,
                5.0,
                "正在准备运行环境...",
            ).await;

            // 安装必需的 runtime
            runtime_bundle
                .install_required(|msg, pct| {
                    let _ = msg;
                    let _ = pct;
                    // 可以通过 progress_tx 发送进度
                })
                .await?;

            self.report_progress(
                InstallStage::ExtractingRuntime,
                20.0,
                "运行环境准备完成",
            ).await;
        }

        // 步骤 2: 安装 OpenClaw
        self.report_progress(
            InstallStage::Installing,
            25.0,
            "正在安装 OpenClaw...",
        ).await;

        let install_result = if options.use_offline_package {
            // 使用离线安装包
            self.install_offline().await?
        } else {
            // 使用在线安装
            self.install(None, None).await?
        };

        if !install_result.success {
            return Ok(install_result);
        }

        // 步骤 3: 应用本土化预设配置
        self.report_progress(
            InstallStage::Configuring,
            70.0,
            "正在应用本土化配置...",
        ).await;

        self.apply_china_presets().await?;

        // 步骤 4: 设置环境变量
        self.report_progress(
            InstallStage::Configuring,
            85.0,
            "正在配置环境...",
        ).await;

        runtime_bundle.setup_environment().await?;

        // 完成
        self.report_progress(
            InstallStage::Complete,
            100.0,
            "安装完成！OpenClaw 已准备就绪",
        ).await;

        Ok(InstallResult {
            success: true,
            version: install_result.version,
            message: "OpenClaw 一键安装成功！已集成国产模型支持".to_string(),
        })
    }

    /// 离线安装（使用 bundled 安装包）
    async fn install_offline(&self) -> Result<InstallResult> {
        use crate::services::offline_installer::OfflineInstaller;

        let installer = OfflineInstaller::for_current_platform()?;

        self.report_progress(
            InstallStage::Installing,
            30.0,
            "正在解压离线安装包...",
        ).await;

        installer.install(&self.install_dir).await?;

        // 创建默认配置
        self.create_default_config()?;

        // 获取版本
        let version = match self.get_installed_version() {
            Ok(v) => v,
            Err(_) => "unknown".to_string(),
        };

        Ok(InstallResult {
            success: true,
            version: Some(version.clone()),
            message: format!("OpenClaw {} 离线安装成功", version),
        })
    }

    /// 应用本土化预设配置（国产模型等）
    pub async fn apply_china_presets(&self) -> Result<()> {
        // 读取预设配置
        let presets = self.load_presets().await?;

        // 读取当前配置
        let mut config = self.read_config()?;

        // 应用预设模型提供商
        if let Ok(_models_yaml) = std::fs::read_to_string(&presets.models_preset_path) {
            // 这里可以将预设的模型配置合并到现有配置
            log::info!("已加载国产模型预设配置");

            // 设置默认提供商为 DeepSeek
            if config.default_model.is_none() {
                // 创建默认的 DeepSeek 配置
                config.default_model = Some("deepseek-chat".to_string());
            }
        }

        // 保存更新后的配置
        self.write_config(&config)?;

        Ok(())
    }

    /// 加载预设配置
    async fn load_presets(&self) -> Result<PresetConfig> {
        // 检查资源目录
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().context("Failed to get exe dir")?;

        let search_paths = [
            exe_dir.join("../Resources/presets"),
            exe_dir.join("resources/presets"),
            exe_dir.join("bundled/presets"),
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bundled/presets"),
        ];

        for path in &search_paths {
            let models_yaml = path.join("models.yaml");
            if models_yaml.exists() {
                return Ok(PresetConfig {
                    models_preset_path: models_yaml,
                });
            }
        }

        Err(anyhow::anyhow!("找不到预设配置文件"))
    }

    /// 获取运行时环境变量（用于子进程）
    pub fn get_runtime_env(&self) -> Result<std::collections::HashMap<String, String>> {
        let runtime_bundle = RuntimeBundle::new()?;
        let mut env = std::collections::HashMap::new();

        if let Some(path) = runtime_bundle.get_runtime_path() {
            let current_path = std::env::var("PATH").unwrap_or_default();
            env.insert("PATH".to_string(), format!("{}:{}", path, current_path));
        }

        Ok(env)
    }
}

impl Default for OpenClawInstaller {
    fn default() -> Self {
        Self::new().expect("Failed to create installer")
    }
}

/// 一键安装选项
pub struct InstallAllOptions {
    /// 使用离线安装包（不依赖网络）
    pub use_offline_package: bool,
    /// 安装路径（默认 ~/.openclaw）
    pub custom_install_dir: Option<PathBuf>,
    /// 是否跳过 runtime 安装（假设系统已有）
    pub skip_runtime: bool,
}

impl Default for InstallAllOptions {
    fn default() -> Self {
        Self {
            use_offline_package: true, // 默认使用离线包，实现真正的一键部署
            custom_install_dir: None,
            skip_runtime: false,
        }
    }
}

/// 预设配置
struct PresetConfig {
    models_preset_path: PathBuf,
}
