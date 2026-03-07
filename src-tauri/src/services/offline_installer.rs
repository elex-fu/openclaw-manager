//! 离线安装包支持
//!
//! 支持6个平台组合：
//! - macOS x64 (Intel)
//! - macOS ARM64 (Apple Silicon)
//! - Windows x64
//! - Windows ARM64
//! - Linux x64
//! - Linux ARM64
//!
//! 使用条件编译优化单平台构建体积

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde::{Serialize, Deserialize};

/// 平台类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    MacOS,
    Windows,
    Linux,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::MacOS => write!(f, "macos"),
            Platform::Windows => write!(f, "windows"),
            Platform::Linux => write!(f, "linux"),
        }
    }
}

/// 架构类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Arch {
    X86_64,
    ARM64,
}

impl std::fmt::Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arch::X86_64 => write!(f, "x64"),
            Arch::ARM64 => write!(f, "arm64"),
        }
    }
}

/// 平台资源 trait - 用于条件编译
pub trait PlatformResource {
    /// 获取当前平台的安装包文件名
    fn package_filename() -> &'static str;
    /// 获取当前平台类型
    fn current_platform() -> Platform;
    /// 获取当前架构
    fn current_arch() -> Arch;
}

// macOS x64 支持
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod platform_impl {
    use super::*;
    pub struct CurrentPlatform;
    impl PlatformResource for CurrentPlatform {
        fn package_filename() -> &'static str { "openclaw-macos-x64.tar.gz" }
        fn current_platform() -> Platform { Platform::MacOS }
        fn current_arch() -> Arch { Arch::X86_64 }
    }
}

// macOS ARM64 支持
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod platform_impl {
    use super::*;
    pub struct CurrentPlatform;
    impl PlatformResource for CurrentPlatform {
        fn package_filename() -> &'static str { "openclaw-macos-arm64.tar.gz" }
        fn current_platform() -> Platform { Platform::MacOS }
        fn current_arch() -> Arch { Arch::ARM64 }
    }
}

// Windows x64 支持
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
mod platform_impl {
    use super::*;
    pub struct CurrentPlatform;
    impl PlatformResource for CurrentPlatform {
        fn package_filename() -> &'static str { "openclaw-windows-x64.zip" }
        fn current_platform() -> Platform { Platform::Windows }
        fn current_arch() -> Arch { Arch::X86_64 }
    }
}

// Windows ARM64 支持
#[cfg(all(target_os = "windows", target_arch = "aarch64"))]
mod platform_impl {
    use super::*;
    pub struct CurrentPlatform;
    impl PlatformResource for CurrentPlatform {
        fn package_filename() -> &'static str { "openclaw-windows-arm64.zip" }
        fn current_platform() -> Platform { Platform::Windows }
        fn current_arch() -> Arch { Arch::ARM64 }
    }
}

// Linux x64 支持
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod platform_impl {
    use super::*;
    pub struct CurrentPlatform;
    impl PlatformResource for CurrentPlatform {
        fn package_filename() -> &'static str { "openclaw-linux-x64.tar.gz" }
        fn current_platform() -> Platform { Platform::Linux }
        fn current_arch() -> Arch { Arch::X86_64 }
    }
}

// Linux ARM64 支持
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
mod platform_impl {
    use super::*;
    pub struct CurrentPlatform;
    impl PlatformResource for CurrentPlatform {
        fn package_filename() -> &'static str { "openclaw-linux-arm64.tar.gz" }
        fn current_platform() -> Platform { Platform::Linux }
        fn current_arch() -> Arch { Arch::ARM64 }
    }
}

// 默认实现（用于测试或非目标平台编译）
#[cfg(not(any(
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "windows", target_arch = "x86_64"),
    all(target_os = "windows", target_arch = "aarch64"),
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "linux", target_arch = "aarch64")
)))]
mod platform_impl {
    use super::*;
    pub struct CurrentPlatform;
    impl PlatformResource for CurrentPlatform {
        fn package_filename() -> &'static str { "openclaw-unknown.tar.gz" }
        fn current_platform() -> Platform { Platform::Linux }
        fn current_arch() -> Arch { Arch::X86_64 }
    }
}

/// 安装包信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub version: String,
    pub platform: Platform,
    pub arch: Arch,
    pub checksum: String,
}

/// 离线安装器
pub struct OfflineInstaller {
    package_info: PackageInfo,
}

impl OfflineInstaller {
    /// 创建当前平台的安装器
    pub fn for_current_platform() -> Result<Self> {
        use platform_impl::CurrentPlatform;

        let platform = CurrentPlatform::current_platform();
        let arch = CurrentPlatform::current_arch();

        Ok(Self {
            package_info: PackageInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                platform,
                arch,
                checksum: String::new(),
            },
        })
    }

    /// 获取安装包文件路径
    fn get_package_path(&self) -> Result<PathBuf> {
        use platform_impl::CurrentPlatform;

        // 首先检查资源目录（生产环境）
        let resource_dir = get_resource_dir()?;

        // 使用条件编译获取当前平台的文件名
        let filename = CurrentPlatform::package_filename();

        log::info!("查找安装包: {}", filename);
        log::info!("资源目录: {:?}", resource_dir);

        let path = resource_dir.join(filename);
        log::info!("检查资源目录路径: {:?}, 存在: {}", path, path.exists());
        if path.exists() {
            log::info!("在资源目录找到安装包");
            return Ok(path);
        }

        // 检查开发环境路径（bundled 目录）
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let dev_path = manifest_dir.join("bundled").join(filename);
        log::info!("检查开发路径: {:?}", dev_path);
        log::info!("manifest_dir: {:?}", manifest_dir);

        if dev_path.exists() {
            log::info!("在开发目录找到安装包");
            return Ok(dev_path);
        }

        // 列出 bundled 目录内容以帮助调试
        let bundled_dir = manifest_dir.join("bundled");
        if bundled_dir.exists() {
            log::warn!("bundled 目录存在，内容:");
            if let Ok(entries) = std::fs::read_dir(&bundled_dir) {
                for entry in entries.flatten() {
                    log::warn!("  - {:?}", entry.file_name());
                }
            }
        } else {
            log::error!("bundled 目录不存在: {:?}", bundled_dir);
        }

        Err(anyhow::anyhow!(
            "找不到离线安装包: {}。请在以下位置之一放置安装包:\n- {}\n- {}",
            filename,
            resource_dir.display(),
            dev_path.display()
        ))
    }

    /// 读取安装包数据
    pub async fn read_package_data(&self) -> Result<Vec<u8>> {
        let path = self.get_package_path()?;

        // 检查文件大小
        let metadata = tokio::fs::metadata(&path).await
            .with_context(|| format!("无法读取安装包元数据: {}", path.display()))?;

        if metadata.len() < 1000 {
            // 可能是占位符文件，读取内容检查
            let content = tokio::fs::read_to_string(&path).await
                .with_context(|| format!("无法读取安装包: {}", path.display()))?;

            if content.contains("Placeholder") || content.contains("placeholder") {
                return Err(anyhow::anyhow!(
                    "离线安装包 '{}' 是占位符文件，不是真实的安装包。\n\n\
                    请下载真实的 OpenClaw 二进制文件并放置到正确位置:\n\
                    期望文件: {}\n\
                    路径: {}\n\n\
                    获取方式:\n\
                    1. 从源码编译: git clone https://github.com/openclai/openclaw.git\n\
                    2. 或创建测试包: mkdir -p openclaw/bin && cp openclaw-binary openclaw/bin/ && tar -czf {} openclaw/",
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    path.display(),
                    path.file_name().unwrap_or_default().to_string_lossy()
                ));
            }
        }

        let data = tokio::fs::read(&path).await
            .with_context(|| format!("无法读取安装包: {}", path.display()))?;

        Ok(data)
    }

    /// 执行离线安装
    pub async fn install(&self, target_dir: &Path) -> Result<()> {
        let package_data = self.read_package_data().await?;

        // 创建临时目录
        let temp_dir = tempfile::tempdir()?;
        let temp_path = temp_dir.path();

        // 解压包
        self.extract_package(&package_data, temp_path).await?;

        // 复制到目标目录
        self.copy_to_target(temp_path, target_dir).await?;

        Ok(())
    }

    /// 解压安装包
    ///
    /// 使用条件编译选择解压方式，Windows 使用 zip，其他平台使用 tar.gz
    #[cfg(target_os = "windows")]
    async fn extract_package(&self, data: &[u8], dest: &Path) -> Result<()> {
        use zip::read::ZipArchive;
        use std::io::Cursor;

        let reader = Cursor::new(data);
        let mut archive = ZipArchive::new(reader)?;
        archive.extract(dest)?;

        Ok(())
    }

    /// 解压安装包 (非 Windows 平台 - tar.gz)
    #[cfg(not(target_os = "windows"))]
    async fn extract_package(&self, data: &[u8], dest: &Path) -> Result<()> {
        use flate2::read::GzDecoder;
        use tar::Archive;
        use std::io::Cursor;

        let gz = GzDecoder::new(Cursor::new(data));
        let mut archive = Archive::new(gz);
        archive.unpack(dest)?;

        Ok(())
    }

    /// 复制到目标目录
    /// 源目录结构: temp_dir/openclaw/bin/openclaw
    /// 目标目录: ~/.openclaw/ -> 应该变成 ~/.openclaw/bin/openclaw
    async fn copy_to_target(&self, source: &Path, target: &Path) -> Result<()> {
        tokio::fs::create_dir_all(target).await?;

        // 查找 source 中的 openclaw 子目录
        let openclaw_dir = source.join("openclaw");
        let source_dir = if openclaw_dir.exists() && openclaw_dir.is_dir() {
            &openclaw_dir
        } else {
            source
        };

        log::info!("复制文件从 {:?} 到 {:?}", source_dir, target);

        // 递归复制文件
        use walkdir::WalkDir;

        for entry in WalkDir::new(source_dir) {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(source_dir)?;
            let target_path = target.join(relative_path);

            if path.is_dir() {
                tokio::fs::create_dir_all(&target_path).await?;
            } else {
                if let Some(parent) = target_path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                log::debug!("复制文件: {:?} -> {:?}", path, target_path);
                tokio::fs::copy(path, target_path).await?;
            }
        }

        // 在非 Windows 平台上设置可执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let bin_dir = target.join("bin");
            if bin_dir.exists() {
                let mut entries = tokio::fs::read_dir(&bin_dir).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    if tokio::fs::metadata(&path).await?.is_file() {
                        let metadata = tokio::fs::metadata(&path).await?;
                        let mut permissions = metadata.permissions();
                        permissions.set_mode(permissions.mode() | 0o111); // 添加可执行权限
                        tokio::fs::set_permissions(&path, permissions).await?;
                        log::info!("设置可执行权限: {:?}", path);
                    }
                }
            }
        }

        Ok(())
    }
}

/// 获取资源目录路径
/// 在 Tauri v2 中，资源文件会被复制到以下位置：
/// - macOS: OpenClaw Manager.app/Contents/Resources
/// - Windows: 与可执行文件同目录的 resources 文件夹
/// - Linux: 与可执行文件同目录的 resources 文件夹
fn get_resource_dir() -> Result<PathBuf> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().context("无法获取可执行文件目录")?;

    // 检查是否在 macOS app bundle 中 (../Resources/)
    let macos_resources = exe_dir.join("../Resources");
    if macos_resources.exists() {
        return Ok(macos_resources.canonicalize().unwrap_or(macos_resources));
    }

    // 检查 Tauri v2 资源目录 (resources/)
    let resources_dir = exe_dir.join("resources");
    if resources_dir.exists() {
        return Ok(resources_dir);
    }

    // 检查 bundled 子目录 (开发环境)
    let bundled_dir = exe_dir.join("bundled");
    if bundled_dir.exists() {
        return Ok(bundled_dir);
    }

    // 默认使用可执行文件目录
    Ok(exe_dir.to_path_buf())
}

/// 获取当前平台信息（用于调试和日志）
pub fn current_platform_info() -> (Platform, Arch) {
    use platform_impl::CurrentPlatform;
    (CurrentPlatform::current_platform(), CurrentPlatform::current_arch())
}
