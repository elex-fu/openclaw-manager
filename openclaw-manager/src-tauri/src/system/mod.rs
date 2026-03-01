use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// 系统类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemType {
    MacOS,
    Windows,
    Linux,
}

impl std::fmt::Display for SystemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemType::MacOS => write!(f, "macOS"),
            SystemType::Windows => write!(f, "Windows"),
            SystemType::Linux => write!(f, "Linux"),
        }
    }
}

/// macOS 版本
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacOSVersion {
    Catalina,      // 10.15
    BigSur,        // 11.0
    Monterey,      // 12.0
    Ventura,       // 13.0
    Sonoma,        // 14.0
    Sequoia,       // 15.0
    Unknown(u32),  // 其他版本号
}

impl MacOSVersion {
    /// 从 Darwin 内核版本号获取 macOS 版本
    pub fn from_darwin_version(major: u32, minor: u32) -> Self {
        match (major, minor) {
            (19, _) => MacOSVersion::Catalina,      // 10.15
            (20, _) => MacOSVersion::BigSur,        // 11.0
            (21, _) => MacOSVersion::Monterey,      // 12.0
            (22, _) => MacOSVersion::Ventura,       // 13.0
            (23, _) => MacOSVersion::Sonoma,        // 14.0
            (24, _) => MacOSVersion::Sequoia,       // 15.0
            (major, _) => MacOSVersion::Unknown(major),
        }
    }

    /// 获取版本字符串
    pub fn version_string(&self) -> String {
        match self {
            MacOSVersion::Catalina => "10.15 (Catalina)".to_string(),
            MacOSVersion::BigSur => "11.0 (Big Sur)".to_string(),
            MacOSVersion::Monterey => "12.0 (Monterey)".to_string(),
            MacOSVersion::Ventura => "13.0 (Ventura)".to_string(),
            MacOSVersion::Sonoma => "14.0 (Sonoma)".to_string(),
            MacOSVersion::Sequoia => "15.0 (Sequoia)".to_string(),
            MacOSVersion::Unknown(v) => format!("Unknown (Darwin {})", v),
        }
    }

    /// 获取最低支持的 Rust 版本
    pub fn min_rust_version(&self) -> &'static str {
        match self {
            MacOSVersion::Catalina => "1.70.0",
            MacOSVersion::BigSur => "1.70.0",
            MacOSVersion::Monterey => "1.72.0",
            MacOSVersion::Ventura => "1.72.0",
            MacOSVersion::Sonoma => "1.74.0",
            MacOSVersion::Sequoia => "1.74.0",
            MacOSVersion::Unknown(_) => "1.74.0",
        }
    }

    /// 是否支持 Apple Silicon
    pub fn supports_apple_silicon(&self) -> bool {
        match self {
            MacOSVersion::Catalina => false,
            MacOSVersion::BigSur => true,
            MacOSVersion::Monterey => true,
            MacOSVersion::Ventura => true,
            MacOSVersion::Sonoma => true,
            MacOSVersion::Sequoia => true,
            MacOSVersion::Unknown(_) => true,
        }
    }

    /// 获取推荐的安装脚本
    pub fn install_script_name(&self) -> String {
        match self {
            MacOSVersion::Catalina => "install_macos_10_15.sh".to_string(),
            MacOSVersion::BigSur => "install_macos_11_0.sh".to_string(),
            MacOSVersion::Monterey => "install_macos_12_0.sh".to_string(),
            MacOSVersion::Ventura => "install_macos_13_0.sh".to_string(),
            MacOSVersion::Sonoma => "install_macos_14_0.sh".to_string(),
            MacOSVersion::Sequoia => "install_macos_15_0.sh".to_string(),
            MacOSVersion::Unknown(_) => "install_macos_generic.sh".to_string(),
        }
    }
}

/// Windows 版本
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowsVersion {
    Windows10,
    Windows11,
    Unknown,
}

impl WindowsVersion {
    pub fn version_string(&self) -> String {
        match self {
            WindowsVersion::Windows10 => "Windows 10".to_string(),
            WindowsVersion::Windows11 => "Windows 11".to_string(),
            WindowsVersion::Unknown => "Unknown Windows".to_string(),
        }
    }
}

/// Linux 发行版
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinuxDistro {
    Ubuntu(String),    // 版本号
    Debian(String),
    Fedora(String),
    Arch,
    Unknown(String),
}

impl LinuxDistro {
    pub fn name(&self) -> String {
        match self {
            LinuxDistro::Ubuntu(v) => format!("Ubuntu {}", v),
            LinuxDistro::Debian(v) => format!("Debian {}", v),
            LinuxDistro::Fedora(v) => format!("Fedora {}", v),
            LinuxDistro::Arch => "Arch Linux".to_string(),
            LinuxDistro::Unknown(s) => format!("Unknown ({})", s),
        }
    }
}

/// 系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub system_type: SystemType,
    pub macos_version: Option<MacOSVersion>,
    pub windows_version: Option<WindowsVersion>,
    pub linux_distro: Option<LinuxDistro>,
    pub architecture: Architecture,
    pub kernel_version: String,
    pub hostname: String,
}

/// 系统架构
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Architecture {
    X86_64,
    Aarch64,  // ARM64 / Apple Silicon
    Unknown,
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Architecture::X86_64 => write!(f, "x86_64"),
            Architecture::Aarch64 => write!(f, "aarch64"),
            Architecture::Unknown => write!(f, "unknown"),
        }
    }
}

impl SystemInfo {
    /// 检测当前系统信息
    pub fn detect() -> Result<Self> {
        let system_type = Self::detect_system_type()?;
        let architecture = Self::detect_architecture();
        let kernel_version = Self::detect_kernel_version()?;
        let hostname = Self::detect_hostname()?;

        let (macos_version, windows_version, linux_distro) = match system_type {
            SystemType::MacOS => (Some(Self::detect_macos_version()?), None, None),
            SystemType::Windows => (None, Some(Self::detect_windows_version()?), None),
            SystemType::Linux => (None, None, Some(Self::detect_linux_distro()?)),
        };

        Ok(SystemInfo {
            system_type,
            macos_version,
            windows_version,
            linux_distro,
            architecture,
            kernel_version,
            hostname,
        })
    }

    /// 检测系统类型
    fn detect_system_type() -> Result<SystemType> {
        let os = std::env::consts::OS;
        match os {
            "macos" => Ok(SystemType::MacOS),
            "windows" => Ok(SystemType::Windows),
            "linux" => Ok(SystemType::Linux),
            _ => Err(anyhow::anyhow!("Unsupported operating system: {}", os)),
        }
    }

    /// 检测系统架构
    fn detect_architecture() -> Architecture {
        let arch = std::env::consts::ARCH;
        match arch {
            "x86_64" => Architecture::X86_64,
            "aarch64" => Architecture::Aarch64,
            _ => Architecture::Unknown,
        }
    }

    /// 检测内核版本
    fn detect_kernel_version() -> Result<String> {
        let output = std::process::Command::new("uname")
            .args(["-r"])
            .output()
            .context("Failed to run uname")?;

        let version = String::from_utf8_lossy(&output.stdout);
        Ok(version.trim().to_string())
    }

    /// 检测主机名
    fn detect_hostname() -> Result<String> {
        let output = std::process::Command::new("hostname")
            .output()
            .context("Failed to run hostname")?;

        let hostname = String::from_utf8_lossy(&output.stdout);
        Ok(hostname.trim().to_string())
    }

    /// 检测 macOS 版本
    fn detect_macos_version() -> Result<MacOSVersion> {
        // 使用 uname -r 获取 Darwin 内核版本
        let output = std::process::Command::new("uname")
            .args(["-r"])
            .output()
            .context("Failed to get Darwin version")?;

        let version_str = String::from_utf8_lossy(&output.stdout);
        let version_parts: Vec<&str> = version_str.trim().split('.').collect();

        if version_parts.is_empty() {
            return Err(anyhow::anyhow!("Invalid Darwin version string"));
        }

        let major = version_parts[0].parse::<u32>()
            .context("Failed to parse Darwin major version")?;
        let minor = version_parts.get(1).unwrap_or(&"0").parse::<u32>().unwrap_or(0);

        Ok(MacOSVersion::from_darwin_version(major, minor))
    }

    /// 检测 Windows 版本
    fn detect_windows_version() -> Result<WindowsVersion> {
        // Windows 版本检测逻辑
        // 在实际实现中，可以使用 registry 或 WMIC
        // 这里简化处理
        Ok(WindowsVersion::Windows11)
    }

    /// 检测 Linux 发行版
    fn detect_linux_distro() -> Result<LinuxDistro> {
        // 读取 /etc/os-release
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            let mut id = None;
            let mut version_id = None;

            for line in content.lines() {
                if line.starts_with("ID=") {
                    id = Some(line.trim_start_matches("ID=").trim_matches('"').to_string());
                }
                if line.starts_with("VERSION_ID=") {
                    version_id = Some(line.trim_start_matches("VERSION_ID=").trim_matches('"').to_string());
                }
            }

            match id.as_deref() {
                Some("ubuntu") => Ok(LinuxDistro::Ubuntu(version_id.unwrap_or_default())),
                Some("debian") => Ok(LinuxDistro::Debian(version_id.unwrap_or_default())),
                Some("fedora") => Ok(LinuxDistro::Fedora(version_id.unwrap_or_default())),
                Some("arch") => Ok(LinuxDistro::Arch),
                Some(other) => Ok(LinuxDistro::Unknown(other.to_string())),
                None => Ok(LinuxDistro::Unknown("unknown".to_string())),
            }
        } else {
            Ok(LinuxDistro::Unknown("unknown".to_string()))
        }
    }

    /// 获取友好的系统名称
    pub fn friendly_name(&self) -> String {
        match self.system_type {
            SystemType::MacOS => {
                if let Some(ref version) = self.macos_version {
                    format!("macOS {}", version.version_string())
                } else {
                    "macOS".to_string()
                }
            }
            SystemType::Windows => {
                if let Some(ref version) = self.windows_version {
                    version.version_string()
                } else {
                    "Windows".to_string()
                }
            }
            SystemType::Linux => {
                if let Some(ref distro) = self.linux_distro {
                    distro.name()
                } else {
                    "Linux".to_string()
                }
            }
        }
    }

    /// 获取安装脚本路径
    pub fn install_script(&self) -> String {
        match self.system_type {
            SystemType::MacOS => {
                if let Some(ref version) = self.macos_version {
                    version.install_script_name()
                } else {
                    "install_macos_generic.sh".to_string()
                }
            }
            SystemType::Windows => "install_windows.ps1".to_string(),
            SystemType::Linux => "install_linux.sh".to_string(),
        }
    }
}

/// 获取系统信息（供前端调用）
#[tauri::command]
pub async fn get_system_info() -> Result<crate::models::ApiResponse<SystemInfo>, String> {
    match SystemInfo::detect() {
        Ok(info) => Ok(crate::models::ApiResponse::success(info)),
        Err(e) => Ok(crate::models::ApiResponse::error(format!("检测系统信息失败: {}", e))),
    }
}
