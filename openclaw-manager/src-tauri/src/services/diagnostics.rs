//! 诊断服务模块
//!
//! 提供系统环境检查、健康检查和一键修复功能

use crate::errors::AppError;
use crate::installer::OpenClawInstaller;
use crate::models::openclaw::InstallStatus;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::timeout;

/// 检查状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    /// 通过
    Pass,
    /// 警告
    Warning,
    /// 错误
    Error,
}

impl CheckStatus {
    /// 转换为严重程度字符串
    pub fn to_severity(&self) -> &'static str {
        match self {
            CheckStatus::Pass => "info",
            CheckStatus::Warning => "warning",
            CheckStatus::Error => "error",
        }
    }
}

/// 诊断检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCheck {
    /// 检查类别
    pub category: String,
    /// 检查项名称
    pub name: String,
    /// 检查状态
    pub status: CheckStatus,
    /// 状态描述
    pub message: String,
    /// 详细信息
    pub details: Option<String>,
    /// 是否可自动修复
    pub fixable: bool,
    /// 修复建议
    pub fix_suggestion: Option<String>,
}

/// 诊断结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticResult {
    /// 检查项列表
    pub checks: Vec<DiagnosticCheck>,
    /// 是否有错误
    pub has_errors: bool,
    /// 是否有警告
    pub has_warnings: bool,
    /// 检查时间
    pub checked_at: String,
}

/// 修复结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixResult {
    /// 成功修复的检查项名称列表
    pub fixed: Vec<String>,
    /// 修复失败的检查项
    pub failed: Vec<FixFailure>,
}

/// 修复失败信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixFailure {
    /// 检查项名称
    pub name: String,
    /// 错误信息
    pub error: String,
}

/// 诊断服务
pub struct DiagnosticService {
    installer: OpenClawInstaller,
}

impl DiagnosticService {
    /// 创建新的诊断服务实例
    pub fn new() -> Result<Self, AppError> {
        let installer = OpenClawInstaller::new()
            .map_err(|e| AppError::Unknown(format!("Failed to create installer: {}", e)))?;

        Ok(Self { installer })
    }

    /// 运行所有诊断检查
    pub async fn run_diagnostics(&self) -> Result<DiagnosticResult, AppError> {
        let mut checks = Vec::new();
        let mut has_errors = false;
        let mut has_warnings = false;

        // 1. 系统环境检查
        checks.extend(self.check_system_environment().await?);

        // 2. OpenClaw 环境检查
        checks.extend(self.check_openclaw_environment().await?);

        // 3. 网络连通性检查
        checks.extend(self.check_network_connectivity().await?);

        // 4. 服务健康检查
        checks.extend(self.check_service_health().await?);

        // 统计结果
        for check in &checks {
            match check.status {
                CheckStatus::Error => has_errors = true,
                CheckStatus::Warning => has_warnings = true,
                _ => {}
            }
        }

        Ok(DiagnosticResult {
            checks,
            has_errors,
            has_warnings,
            checked_at: chrono::Local::now().to_rfc3339(),
        })
    }

    /// 系统环境检查
    async fn check_system_environment(&self) -> Result<Vec<DiagnosticCheck>, AppError> {
        let mut checks = Vec::new();

        // OS 版本兼容性
        checks.push(self.check_os_compatibility().await);

        // 内存检查
        checks.push(self.check_memory().await);

        // 磁盘空间检查
        checks.push(self.check_disk_space().await);

        // 必要依赖检查
        checks.push(self.check_nodejs().await);
        checks.push(self.check_python().await);
        checks.push(self.check_git().await);

        Ok(checks)
    }

    /// 检查操作系统兼容性
    async fn check_os_compatibility(&self) -> DiagnosticCheck {
        let os = std::env::consts::OS;
        let (status, message, details) = match os {
            "macos" => {
                let version = self.get_macos_version().await;
                match version.as_deref() {
                    Some("10.15") | Some("11") | Some("12") | Some("13") | Some("14") | Some("15") => {
                        (CheckStatus::Pass, format!("macOS {} 兼容", version.unwrap_or_default()), None)
                    }
                    Some(v) => (
                        CheckStatus::Warning,
                        format!("macOS {} 可能兼容", v),
                        Some("建议使用 macOS 10.15 (Catalina) 或更高版本".to_string()),
                    ),
                    None => (
                        CheckStatus::Pass,
                        "macOS 系统兼容".to_string(),
                        None,
                    ),
                }
            }
            "linux" => (CheckStatus::Pass, "Linux 系统兼容".to_string(), None),
            "windows" => (CheckStatus::Pass, "Windows 系统兼容".to_string(), None),
            _ => (
                CheckStatus::Error,
                format!("不支持的操作系统: {}", os),
                Some("OpenClaw 支持 macOS、Linux 和 Windows".to_string()),
            ),
        };

        DiagnosticCheck {
            category: "system".to_string(),
            name: "操作系统兼容性".to_string(),
            status,
            message,
            details,
            fixable: false,
            fix_suggestion: None,
        }
    }

    /// 获取 macOS 版本
    async fn get_macos_version(&self) -> Option<String> {
        let output = tokio::process::Command::new("sw_vers")
            .args(["-productVersion"])
            .output()
            .await
            .ok()?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            Some(version.trim().to_string())
        } else {
            None
        }
    }

    /// 检查内存
    async fn check_memory(&self) -> DiagnosticCheck {
        let (status, message, details) = if cfg!(target_os = "macos") {
            match self.get_macos_memory().await {
                Some(mem_mb) if mem_mb >= 4096 => (
                    CheckStatus::Pass,
                    format!("内存充足 ({} MB)", mem_mb),
                    None,
                ),
                Some(mem_mb) if mem_mb >= 2048 => (
                    CheckStatus::Warning,
                    format!("内存较少 ({} MB)", mem_mb),
                    Some("建议至少 4GB 内存以获得最佳性能".to_string()),
                ),
                Some(mem_mb) => (
                    CheckStatus::Error,
                    format!("内存不足 ({} MB)", mem_mb),
                    Some("需要至少 2GB 内存".to_string()),
                ),
                None => (
                    CheckStatus::Warning,
                    "无法检测内存".to_string(),
                    None,
                ),
            }
        } else if cfg!(target_os = "linux") {
            match self.get_linux_memory().await {
                Some(mem_mb) if mem_mb >= 4096 => (
                    CheckStatus::Pass,
                    format!("内存充足 ({} MB)", mem_mb),
                    None,
                ),
                Some(mem_mb) if mem_mb >= 2048 => (
                    CheckStatus::Warning,
                    format!("内存较少 ({} MB)", mem_mb),
                    Some("建议至少 4GB 内存以获得最佳性能".to_string()),
                ),
                Some(mem_mb) => (
                    CheckStatus::Error,
                    format!("内存不足 ({} MB)", mem_mb),
                    Some("需要至少 2GB 内存".to_string()),
                ),
                None => (
                    CheckStatus::Warning,
                    "无法检测内存".to_string(),
                    None,
                ),
            }
        } else {
            (CheckStatus::Pass, "内存检查跳过".to_string(), None)
        };

        DiagnosticCheck {
            category: "system".to_string(),
            name: "内存检查".to_string(),
            status,
            message,
            details,
            fixable: false,
            fix_suggestion: None,
        }
    }

    /// 获取 macOS 内存
    async fn get_macos_memory(&self) -> Option<u64> {
        let output = tokio::process::Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .await
            .ok()?;

        if output.status.success() {
            let bytes: u64 = String::from_utf8_lossy(&output.stdout).trim().parse().ok()?;
            Some(bytes / 1024 / 1024) // Convert to MB
        } else {
            None
        }
    }

    /// 获取 Linux 内存
    async fn get_linux_memory(&self) -> Option<u64> {
        let content = tokio::fs::read_to_string("/proc/meminfo").await.ok()?;
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb: u64 = parts[1].parse().ok()?;
                    return Some(kb / 1024); // Convert to MB
                }
            }
        }
        None
    }

    /// 检查磁盘空间
    async fn check_disk_space(&self) -> DiagnosticCheck {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        let (status, message, details) = if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
            match self.get_unix_disk_space(&home_dir).await {
                Some((available_mb, total_mb)) if available_mb >= 10240 => (
                    CheckStatus::Pass,
                    format!("磁盘空间充足 ({:.1} GB / {:.1} GB)",
                        available_mb as f64 / 1024.0,
                        total_mb as f64 / 1024.0),
                    None,
                ),
                Some((available_mb, total_mb)) if available_mb >= 5120 => (
                    CheckStatus::Warning,
                    format!("磁盘空间较少 ({:.1} GB / {:.1} GB)",
                        available_mb as f64 / 1024.0,
                        total_mb as f64 / 1024.0),
                    Some("建议至少保留 10GB 可用空间".to_string()),
                ),
                Some((available_mb, total_mb)) => (
                    CheckStatus::Error,
                    format!("磁盘空间不足 ({:.1} GB / {:.1} GB)",
                        available_mb as f64 / 1024.0,
                        total_mb as f64 / 1024.0),
                    Some("需要至少 5GB 可用空间".to_string()),
                ),
                None => (
                    CheckStatus::Warning,
                    "无法检测磁盘空间".to_string(),
                    None,
                ),
            }
        } else {
            (CheckStatus::Pass, "磁盘空间检查跳过".to_string(), None)
        };

        DiagnosticCheck {
            category: "system".to_string(),
            name: "磁盘空间".to_string(),
            status,
            message,
            details,
            fixable: false,
            fix_suggestion: None,
        }
    }

    /// 获取 Unix 磁盘空间
    async fn get_unix_disk_space(&self, path: &PathBuf) -> Option<(u64, u64)> {
        let output = tokio::process::Command::new("df")
            .args(["-k", path.to_str().unwrap_or(".")])
            .output()
            .await
            .ok()?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = output_str.lines().collect();
            if lines.len() >= 2 {
                let parts: Vec<&str> = lines[1].split_whitespace().collect();
                if parts.len() >= 4 {
                    let total_kb: u64 = parts[1].parse().ok()?;
                    let available_kb: u64 = parts[3].parse().ok()?;
                    return Some((available_kb / 1024, total_kb / 1024)); // Convert to MB
                }
            }
        }
        None
    }

    /// 检查 Node.js
    async fn check_nodejs(&self) -> DiagnosticCheck {
        let result = tokio::process::Command::new("node")
            .args(["--version"])
            .output()
            .await;

        let (status, message, fixable, fix_suggestion) = match result {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                (CheckStatus::Pass, format!("Node.js 已安装 ({})", version), false, None)
            }
            _ => (
                CheckStatus::Warning,
                "Node.js 未安装（可选依赖）".to_string(),
                false,
                Some("某些功能可能需要 Node.js，建议从 https://nodejs.org 安装".to_string()),
            ),
        };

        DiagnosticCheck {
            category: "system".to_string(),
            name: "Node.js 环境".to_string(),
            status,
            message,
            details: None,
            fixable,
            fix_suggestion,
        }
    }

    /// 检查 Python
    async fn check_python(&self) -> DiagnosticCheck {
        let result = tokio::process::Command::new("python3")
            .args(["--version"])
            .output()
            .await;

        let result = if result.is_err() {
            tokio::process::Command::new("python")
                .args(["--version"])
                .output()
                .await
        } else {
            result
        };

        let (status, message, fixable, fix_suggestion) = match result {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                (CheckStatus::Pass, format!("Python 已安装 ({})", version), false, None)
            }
            _ => (
                CheckStatus::Warning,
                "Python 未安装（可选依赖）".to_string(),
                false,
                Some("某些插件可能需要 Python，建议从 https://python.org 安装".to_string()),
            ),
        };

        DiagnosticCheck {
            category: "system".to_string(),
            name: "Python 环境".to_string(),
            status,
            message,
            details: None,
            fixable,
            fix_suggestion,
        }
    }

    /// 检查 Git
    async fn check_git(&self) -> DiagnosticCheck {
        let result = tokio::process::Command::new("git")
            .args(["--version"])
            .output()
            .await;

        let (status, message, fixable, fix_suggestion) = match result {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                (CheckStatus::Pass, format!("Git 已安装 ({})", version), false, None)
            }
            _ => (
                CheckStatus::Warning,
                "Git 未安装（可选依赖）".to_string(),
                false,
                Some("某些功能可能需要 Git".to_string()),
            ),
        };

        DiagnosticCheck {
            category: "system".to_string(),
            name: "Git 环境".to_string(),
            status,
            message,
            details: None,
            fixable,
            fix_suggestion,
        }
    }

    /// OpenClaw 环境检查
    async fn check_openclaw_environment(&self) -> Result<Vec<DiagnosticCheck>, AppError> {
        let mut checks = Vec::new();

        // 安装状态检查
        checks.push(self.check_installation_status().await?);

        // 配置文件有效性检查
        checks.push(self.check_config_validity().await?);

        // 端口占用检查
        checks.push(self.check_port(8080, "OpenClaw 默认端口").await);
        checks.push(self.check_port(3000, "开发服务器端口").await);
        checks.push(self.check_port(11434, "Ollama 端口").await);

        Ok(checks)
    }

    /// 检查安装状态
    async fn check_installation_status(&self) -> Result<DiagnosticCheck, AppError> {
        let install_status = self.installer.check_installation()
            .map_err(|e| AppError::Unknown(format!("Failed to check installation: {}", e)))?;

        let (status, message, details, fixable, fix_suggestion) = match install_status {
            InstallStatus::Installed { version } => (
                CheckStatus::Pass,
                format!("OpenClaw 已安装 (版本: {})", version),
                None,
                false,
                None,
            ),
            InstallStatus::NotInstalled => (
                CheckStatus::Error,
                "OpenClaw 未安装".to_string(),
                Some("请先安装 OpenClaw".to_string()),
                true,
                Some("点击安装按钮进行安装".to_string()),
            ),
            InstallStatus::Installing { stage, .. } => (
                CheckStatus::Warning,
                format!("OpenClaw 正在安装中 ({})", stage),
                None,
                false,
                Some("请等待安装完成".to_string()),
            ),
            InstallStatus::Error { message: err_msg } => (
                CheckStatus::Error,
                format!("OpenClaw 安装状态异常: {}", err_msg),
                None,
                true,
                Some("尝试重新安装 OpenClaw".to_string()),
            ),
        };

        Ok(DiagnosticCheck {
            category: "openclaw".to_string(),
            name: "安装状态".to_string(),
            status,
            message,
            details,
            fixable,
            fix_suggestion,
        })
    }

    /// 检查配置文件有效性
    async fn check_config_validity(&self) -> Result<DiagnosticCheck, AppError> {
        let config_result = self.installer.read_config();

        let (status, message, details, fixable, fix_suggestion) = match config_result {
            Ok(config) => {
                let model_count = config.models.len();
                (
                    CheckStatus::Pass,
                    format!("配置文件有效 ({} 个模型配置)", model_count),
                    None,
                    false,
                    None,
                )
            }
            Err(e) => (
                CheckStatus::Error,
                "配置文件无效".to_string(),
                Some(format!("错误: {}", e)),
                true,
                Some("尝试重置配置文件".to_string()),
            ),
        };

        Ok(DiagnosticCheck {
            category: "openclaw".to_string(),
            name: "配置文件有效性".to_string(),
            status,
            message,
            details,
            fixable,
            fix_suggestion,
        })
    }

    /// 检查端口占用
    async fn check_port(&self, port: u16, description: &str) -> DiagnosticCheck {
        let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();

        let is_available = match TcpListener::bind(addr).await {
            Ok(listener) => {
                // 成功绑定说明端口可用，关闭监听器
                drop(listener);
                true
            }
            Err(_) => false,
        };

        let (status, message, details, fixable, fix_suggestion) = if is_available {
            (
                CheckStatus::Pass,
                format!("端口 {} ({}) 可用", port, description),
                None,
                false,
                None,
            )
        } else {
            (
                CheckStatus::Warning,
                format!("端口 {} ({}) 已被占用", port, description),
                Some("如果该服务未启动，可能需要关闭占用该端口的程序".to_string()),
                false,
                Some(format!("查找并关闭占用端口 {} 的程序", port)),
            )
        };

        DiagnosticCheck {
            category: "openclaw".to_string(),
            name: format!("端口 {} 检查", port),
            status,
            message,
            details,
            fixable,
            fix_suggestion,
        }
    }

    /// 网络连通性检查
    async fn check_network_connectivity(&self) -> Result<Vec<DiagnosticCheck>, AppError> {
        let mut checks = Vec::new();

        // 检查互联网连接
        checks.push(self.check_internet_connection().await);

        // 检查常见 API 可访问性
        checks.push(self.check_api_access("https://api.openai.com", "OpenAI API").await);
        checks.push(self.check_api_access("https://api.deepseek.com", "DeepSeek API").await);

        Ok(checks)
    }

    /// 检查互联网连接
    async fn check_internet_connection(&self) -> DiagnosticCheck {
        let urls = vec![
            "https://www.google.com",
            "https://www.baidu.com",
            "https://www.cloudflare.com",
        ];

        let mut connected = false;
        let mut last_error = None;

        for url in urls {
            match timeout(
                Duration::from_secs(5),
                reqwest::get(url)
            ).await {
                Ok(Ok(response)) => {
                    if response.status().is_success() {
                        connected = true;
                        break;
                    } else {
                        last_error = Some(format!("HTTP {}", response.status()));
                    }
                }
                Ok(Err(e)) => {
                    last_error = Some(e.to_string());
                }
                Err(_) => {
                    last_error = Some("连接超时".to_string());
                }
            }
        }

        let (status, message, details) = if connected {
            (
                CheckStatus::Pass,
                "互联网连接正常".to_string(),
                None,
            )
        } else {
            (
                CheckStatus::Warning,
                "互联网连接异常".to_string(),
                last_error,
            )
        };

        DiagnosticCheck {
            category: "network".to_string(),
            name: "互联网连接".to_string(),
            status,
            message,
            details,
            fixable: false,
            fix_suggestion: Some("检查网络连接和代理设置".to_string()),
        }
    }

    /// 检查 API 可访问性
    async fn check_api_access(&self, url: &str, name: &str) -> DiagnosticCheck {
        let result = timeout(
            Duration::from_secs(5),
            reqwest::get(url)
        ).await;

        let (status, message, details) = match result {
            Ok(Ok(response)) => {
                if response.status().is_success() || response.status().as_u16() == 401 {
                    // 401 表示 API 可达但无认证
                    (
                        CheckStatus::Pass,
                        format!("{} 可访问", name),
                        None,
                    )
                } else {
                    (
                        CheckStatus::Warning,
                        format!("{} 返回状态 {}", name, response.status()),
                        None,
                    )
                }
            }
            Ok(Err(e)) => (
                CheckStatus::Warning,
                format!("{} 访问失败", name),
                Some(e.to_string()),
            ),
            Err(_) => (
                CheckStatus::Warning,
                format!("{} 连接超时", name),
                Some("可能是网络问题或 API 不可用".to_string()),
            ),
        };

        DiagnosticCheck {
            category: "network".to_string(),
            name: format!("{} 连通性", name),
            status,
            message,
            details,
            fixable: false,
            fix_suggestion: Some("检查网络连接和防火墙设置".to_string()),
        }
    }

    /// 服务健康检查
    async fn check_service_health(&self) -> Result<Vec<DiagnosticCheck>, AppError> {
        let mut checks = Vec::new();

        // OpenClaw 进程状态
        checks.push(self.check_openclaw_process().await?);

        // HTTP 健康检查
        checks.push(self.check_http_health().await);

        Ok(checks)
    }

    /// 检查 OpenClaw 进程
    async fn check_openclaw_process(&self) -> Result<DiagnosticCheck, AppError> {
        // 尝试查找 openclaw 进程
        let result = if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
            tokio::process::Command::new("pgrep")
                .args(["-f", "openclaw"])
                .output()
                .await
        } else {
            // Windows
            tokio::process::Command::new("tasklist")
                .args(["/FI", "IMAGENAME eq openclaw.exe"])
                .output()
                .await
        };

        let (status, message, details, fixable, fix_suggestion) = match result {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.trim().is_empty() || output_str.contains("No tasks") {
                    (
                        CheckStatus::Warning,
                        "OpenClaw 服务未运行".to_string(),
                        None,
                        true,
                        Some("点击启动服务".to_string()),
                    )
                } else {
                    (
                        CheckStatus::Pass,
                        "OpenClaw 服务正在运行".to_string(),
                        None,
                        false,
                        None,
                    )
                }
            }
            _ => (
                CheckStatus::Warning,
                "无法检测 OpenClaw 进程状态".to_string(),
                None,
                false,
                None,
            ),
        };

        Ok(DiagnosticCheck {
            category: "service".to_string(),
            name: "OpenClaw 进程状态".to_string(),
            status,
            message,
            details,
            fixable,
            fix_suggestion,
        })
    }

    /// HTTP 健康检查
    async fn check_http_health(&self) -> DiagnosticCheck {
        // 尝试访问本地 OpenClaw 服务
        let result = timeout(
            Duration::from_secs(3),
            reqwest::get("http://127.0.0.1:8080/health")
        ).await;

        let (status, message, details, fixable, fix_suggestion) = match result {
            Ok(Ok(response)) if response.status().is_success() => (
                CheckStatus::Pass,
                "HTTP 健康检查通过".to_string(),
                None,
                false,
                None,
            ),
            Ok(Ok(response)) => (
                CheckStatus::Warning,
                format!("HTTP 健康检查返回 {}", response.status()),
                None,
                false,
                Some("检查服务日志".to_string()),
            ),
            Ok(Err(e)) => (
                CheckStatus::Warning,
                "无法连接到 OpenClaw 服务".to_string(),
                Some(format!("错误: {}", e)),
                true,
                Some("启动 OpenClaw 服务".to_string()),
            ),
            Err(_) => (
                CheckStatus::Warning,
                "HTTP 健康检查超时".to_string(),
                Some("服务可能未启动或响应缓慢".to_string()),
                true,
                Some("启动或重启 OpenClaw 服务".to_string()),
            ),
        };

        DiagnosticCheck {
            category: "service".to_string(),
            name: "HTTP 健康检查".to_string(),
            status,
            message,
            details,
            fixable,
            fix_suggestion,
        }
    }

    /// 自动修复问题
    pub async fn auto_fix(&self, check_names: Vec<String>) -> Result<FixResult, AppError> {
        let mut fixed = Vec::new();
        let mut failed = Vec::new();

        for name in check_names {
            match self.fix_check(&name).await {
                Ok(()) => fixed.push(name),
                Err(e) => failed.push(FixFailure {
                    name,
                    error: e.to_string(),
                }),
            }
        }

        Ok(FixResult { fixed, failed })
    }

    /// 修复单个检查项
    async fn fix_check(&self, name: &str) -> Result<(), AppError> {
        match name {
            "安装状态" => self.fix_installation().await,
            "配置文件有效性" => self.fix_config().await,
            "OpenClaw 进程状态" => self.fix_service().await,
            "HTTP 健康检查" => self.fix_service().await,
            _ => Err(AppError::Unknown(format!("无法自动修复: {}", name))),
        }
    }

    /// 修复安装
    async fn fix_installation(&self) -> Result<(), AppError> {
        // 触发安装流程
        // 实际实现中应该调用安装服务
        Err(AppError::Unknown("请使用安装向导进行安装".to_string()))
    }

    /// 修复配置
    async fn fix_config(&self) -> Result<(), AppError> {
        self.installer.create_default_config()
            .map_err(|e| AppError::Unknown(format!("创建默认配置失败: {}", e)))?;
        Ok(())
    }

    /// 修复服务
    async fn fix_service(&self) -> Result<(), AppError> {
        // 尝试启动服务
        self.installer.start_service().await
            .map_err(|e| AppError::Unknown(format!("启动服务失败: {}", e)))?;
        Ok(())
    }
}

impl Default for DiagnosticService {
    fn default() -> Self {
        Self::new().expect("Failed to create diagnostic service")
    }
}

/// 诊断检查请求（用于前端传入）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCheckRequest {
    pub name: String,
    pub category: String,
}
