//! 嵌入式 Runtime 管理器
//!
//! 管理 Node.js 22、Python 3.10 等嵌入式运行环境
//! 实现自动检测、按需解压、环境配置

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::process::Command;

/// Runtime 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeType {
    Node,
    Python,
    Chromium,
}

impl std::fmt::Display for RuntimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeType::Node => write!(f, "node"),
            RuntimeType::Python => write!(f, "python"),
            RuntimeType::Chromium => write!(f, "chromium"),
        }
    }
}

/// Runtime 版本信息
#[derive(Debug, Clone)]
pub struct RuntimeVersion {
    pub runtime_type: RuntimeType,
    pub version: String,
    pub platform: String,
    pub arch: String,
}

/// Runtime 包信息
#[derive(Debug, Clone)]
pub struct RuntimePackage {
    pub version: RuntimeVersion,
    pub archive_name: String,
    pub extract_size: u64,  // 预估解压后大小（字节）
    pub required: bool,     // 是否为必需
}

impl RuntimePackage {
    /// 创建 Node.js 22 包配置
    pub fn node22() -> Self {
        let (platform, arch, ext) = Self::get_platform_info();
        let archive_name = format!("node-v22.0.0-{}-{}{}", platform, arch, ext);

        Self {
            version: RuntimeVersion {
                runtime_type: RuntimeType::Node,
                version: "22.0.0".to_string(),
                platform,
                arch,
            },
            archive_name,
            extract_size: 150 * 1024 * 1024, // ~150MB
            required: false, // Node 是可选的
        }
    }

    /// 创建 Python 3.10 包配置
    pub fn python310() -> Self {
        let (platform, arch, ext) = Self::get_platform_info();
        // Python 构建版本命名
        let archive_name = match platform.as_str() {
            "darwin" => format!("python-3.10.14-macos11-{arch}.tar.gz"),
            "linux" => format!("python-3.10.14-{}-{arch}.tar.gz", platform),
            "win32" => format!("python-3.10.14-embed-{}-{arch}.zip", platform),
            _ => format!("python-3.10.14-{platform}-{arch}.tar.gz"),
        };

        Self {
            version: RuntimeVersion {
                runtime_type: RuntimeType::Python,
                version: "3.10.14".to_string(),
                platform,
                arch,
            },
            archive_name,
            extract_size: 80 * 1024 * 1024, // ~80MB
            required: true, // Python 是必需的（OpenClaw 依赖）
        }
    }

    /// 获取当前平台信息
    fn get_platform_info() -> (String, String, String) {
        let platform = match std::env::consts::OS {
            "macos" => "darwin".to_string(),
            "windows" => "win32".to_string(),
            _ => std::env::consts::OS.to_string(),
        };

        let arch = match std::env::consts::ARCH {
            "x86_64" => "x64".to_string(),
            "aarch64" => "arm64".to_string(),
            _ => std::env::consts::ARCH.to_string(),
        };

        let ext = if platform == "win32" {
            ".zip"
        } else {
            ".tar.gz"
        };

        (platform, arch, ext.to_string())
    }
}

/// Runtime Bundle 管理器
pub struct RuntimeBundle {
    install_dir: PathBuf,
    packages: Vec<RuntimePackage>,
}

impl RuntimeBundle {
    /// 创建新的 RuntimeBundle 实例
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;
        let install_dir = home_dir.join(".openclaw").join("runtime");

        Ok(Self {
            install_dir,
            packages: vec![
                RuntimePackage::node22(),
                RuntimePackage::python310(),
            ],
        })
    }

    /// 获取安装目录
    pub fn get_install_dir(&self) -> &Path {
        &self.install_dir
    }

    /// 检查所有必需的 runtime 是否已安装
    pub async fn check_all_installed(&self) -> Result<Vec<RuntimeStatus>> {
        let mut statuses = Vec::new();

        for package in &self.packages {
            let status = self.check_runtime(package).await?;
            statuses.push(status);
        }

        Ok(statuses)
    }

    /// 检查特定 runtime 状态
    async fn check_runtime(&self, package: &RuntimePackage) -> Result<RuntimeStatus> {
        let runtime_dir = self.get_runtime_dir(&package.version.runtime_type);

        // 检查是否已经解压安装
        if runtime_dir.exists() {
            // 验证可执行文件是否存在
            let exe_path = self.get_executable_path(&package.version);
            if exe_path.exists() {
                // 尝试获取版本号验证
                match self.get_installed_version(&package.version).await {
                    Ok(version) => {
                        return Ok(RuntimeStatus {
                            package: package.clone(),
                            installed: true,
                            version: Some(version),
                            path: Some(runtime_dir),
                            message: "已安装".to_string(),
                        });
                    }
                    Err(e) => {
                        log::warn!("Runtime found but version check failed: {}", e);
                    }
                }
            }
        }

        // 检查系统是否已有兼容版本
        match self.check_system_runtime(&package.version).await {
            Ok(Some(version)) => {
                return Ok(RuntimeStatus {
                    package: package.clone(),
                    installed: true,
                    version: Some(version),
                    path: None,
                    message: "使用系统版本".to_string(),
                });
            }
            _ => {}
        }

        Ok(RuntimeStatus {
            package: package.clone(),
            installed: false,
            version: None,
            path: None,
            message: if package.required {
                "必需但未安装".to_string()
            } else {
                "可选，未安装".to_string()
            },
        })
    }

    /// 安装所有必需的 runtime
    pub async fn install_required<F>(&self, mut progress_cb: F) -> Result<()>
    where
        F: FnMut(&str, f32),
    {
        let statuses = self.check_all_installed().await?;

        for (idx, status) in statuses.iter().enumerate() {
            if !status.installed && status.package.required {
                let total = statuses.iter().filter(|s| s.package.required).count();
                let current = idx + 1;

                progress_cb(
                    &format!("正在安装 {}...", status.package.version.runtime_type),
                    (current as f32 / total as f32) * 100.0,
                );

                self.extract_runtime(&status.package).await?;
            }
        }

        progress_cb("Runtime 安装完成", 100.0);
        Ok(())
    }

    /// 解压 runtime 包
    async fn extract_runtime(&self, package: &RuntimePackage) -> Result<()> {
        let source_path = self.get_bundled_runtime_path(package)?;
        let target_dir = self.get_runtime_dir(&package.version.runtime_type);

        // 创建目标目录
        fs::create_dir_all(&target_dir).await
            .with_context(|| format!("Failed to create runtime directory: {:?}", target_dir))?;

        // 检查源文件是否存在
        if !source_path.exists() {
            return Err(anyhow::anyhow!(
                "Runtime 包不存在: {}。请确保已下载 {} 并放置到 bundled/runtimes/ 目录",
                source_path.display(),
                package.archive_name
            ));
        }

        log::info!("解压 runtime: {:?} -> {:?}", source_path, target_dir);

        // 根据文件类型解压
        if source_path.extension().map(|e| e == "zip").unwrap_or(false) {
            self.extract_zip(&source_path, &target_dir).await?;
        } else {
            self.extract_tar_gz(&source_path, &target_dir).await?;
        }

        // 设置可执行权限（Unix）
        #[cfg(unix)]
        {
            self.set_executable_permissions(&target_dir, &package.version.runtime_type).await?;
        }

        log::info!("Runtime {:?} 解压完成", package.version.runtime_type);
        Ok(())
    }

    /// 解压 zip 文件
    async fn extract_zip(&self, source: &Path, target: &Path) -> Result<()> {
        use zip::ZipArchive;
        use std::io::Cursor;
        use tokio::task;

        let source = source.to_path_buf();
        let target = target.to_path_buf();

        task::spawn_blocking(move || {
            let file = std::fs::File::open(&source)?;
            let mut archive = ZipArchive::new(file)?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let outpath = target.join(file.name());

                if file.name().ends_with('/') {
                    std::fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(parent) = outpath.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    let mut outfile = std::fs::File::create(&outpath)?;
                    std::io::copy(&mut file, &mut outfile)?;
                }
            }

            Ok::<_, anyhow::Error>(())
        }).await
        .context("Failed to extract zip")??;

        Ok(())
    }

    /// 解压 tar.gz 文件
    async fn extract_tar_gz(&self, source: &Path, target: &Path) -> Result<()> {
        use flate2::read::GzDecoder;
        use tar::Archive;
        use tokio::task;

        let source = source.to_path_buf();
        let target = target.to_path_buf();

        task::spawn_blocking(move || {
            let file = std::fs::File::open(&source)?;
            let gz = GzDecoder::new(file);
            let mut archive = Archive::new(gz);

            archive.unpack(&target)
                .with_context(|| format!("Failed to unpack tar.gz to {:?}", target))?;

            Ok::<_, anyhow::Error>(())
        }).await
        .context("Failed to extract tar.gz")??;

        Ok(())
    }

    /// 设置可执行权限（Unix）
    #[cfg(unix)]
    async fn set_executable_permissions(&self, dir: &Path, runtime_type: &RuntimeType) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let bin_dir = match runtime_type {
            RuntimeType::Node => dir.join("bin"),
            RuntimeType::Python => dir.join("bin"),
            _ => return Ok(()),
        };

        if !bin_dir.exists() {
            return Ok(());
        }

        let mut entries = fs::read_dir(&bin_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                let metadata = fs::metadata(&path).await?;
                let mut permissions = metadata.permissions();
                permissions.set_mode(permissions.mode() | 0o111);
                fs::set_permissions(&path, permissions).await?;
            }
        }

        Ok(())
    }

    /// 获取 bundled runtime 路径
    fn get_bundled_runtime_path(&self, package: &RuntimePackage) -> Result<PathBuf> {
        // 1. 检查资源目录（生产环境）
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().context("Failed to get exe dir")?;

        let resource_paths = vec![
            exe_dir.join("../Resources/runtimes"),     // macOS app bundle
            exe_dir.join("resources/runtimes"),        // Tauri v2 resources
            exe_dir.join("bundled/runtimes"),          // 开发环境
        ];

        for resource_dir in resource_paths {
            let path = resource_dir.join(&package.archive_name);
            if path.exists() {
                return Ok(path);
            }
        }

        // 2. 检查开发环境（src-tauri 目录）
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let dev_path = manifest_dir.join("bundled").join("runtimes").join(&package.archive_name);
        if dev_path.exists() {
            return Ok(dev_path);
        }

        Err(anyhow::anyhow!(
            "找不到 runtime 包: {}。\n请在以下位置之一放置:\n- bundled/runtimes/\n- src-tauri/bundled/runtimes/",
            package.archive_name
        ))
    }

    /// 获取 runtime 安装目录
    fn get_runtime_dir(&self, runtime_type: &RuntimeType) -> PathBuf {
        self.install_dir.join(runtime_type.to_string())
    }

    /// 获取可执行文件路径
    fn get_executable_path(&self, version: &RuntimeVersion) -> PathBuf {
        let runtime_dir = self.get_runtime_dir(&version.runtime_type);
        let bin_dir = runtime_dir.join("bin");

        let exe_name = match version.runtime_type {
            RuntimeType::Node => "node",
            RuntimeType::Python => "python3",
            RuntimeType::Chromium => "chromium",
        };

        #[cfg(windows)]
        let exe_name = format!("{}.exe", exe_name);

        bin_dir.join(exe_name)
    }

    /// 获取已安装 runtime 版本
    async fn get_installed_version(&self, version: &RuntimeVersion) -> Result<String> {
        let exe_path = self.get_executable_path(version);

        let output = Command::new(&exe_path)
            .arg("--version")
            .output()
            .await
            .with_context(|| format!("Failed to get version from {:?}", exe_path))?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Version command failed"));
        }

        let version_str = String::from_utf8_lossy(&output.stdout);
        Ok(version_str.trim().to_string())
    }

    /// 检查系统是否已有兼容的 runtime
    async fn check_system_runtime(&self, version: &RuntimeVersion) -> Result<Option<String>> {
        let cmd = match version.runtime_type {
            RuntimeType::Node => "node",
            RuntimeType::Python => "python3",
            RuntimeType::Chromium => return Ok(None),
        };

        match Command::new(cmd).arg("--version").output().await {
            Ok(output) if output.status.success() => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                let version_str = version_str.trim();

                // 检查版本是否兼容
                if self.is_version_compatible(version, version_str) {
                    return Ok(Some(version_str.to_string()));
                }
            }
            _ => {}
        }

        Ok(None)
    }

    /// 检查版本兼容性
    fn is_version_compatible(&self, required: &RuntimeVersion, found: &str) -> bool {
        // 简单版本检查：提取主版本号
        let found_major = found
            .split('.')
            .next()
            .and_then(|s| s.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse::<u32>().ok())
            .unwrap_or(0);

        let required_major = required
            .version
            .split('.')
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        found_major >= required_major
    }

    /// 设置环境变量
    pub async fn setup_environment(&self) -> Result<()> {
        // 创建环境变量配置文件
        let env_file = self.install_dir.join("env.sh");

        let mut env_content = String::new();
        env_content.push_str("# OpenClaw Runtime Environment\n");
        env_content.push_str("# Source this file: source ~/.openclaw/runtime/env.sh\n\n");

        // 添加每个 runtime 的 PATH
        for package in &self.packages {
            let runtime_dir = self.get_runtime_dir(&package.version.runtime_type);
            let bin_dir = runtime_dir.join("bin");

            if bin_dir.exists() {
                env_content.push_str(&format!(
                    "export PATH=\"{}:$PATH\"\n",
                    bin_dir.to_string_lossy()
                ));
            }
        }

        // 添加 PYTHONPATH（如果需要）
        let python_dir = self.get_runtime_dir(&RuntimeType::Python);
        if python_dir.exists() {
            env_content.push_str(&format!(
                "export PYTHONPATH=\"{}:$PYTHONPATH\"\n",
                python_dir.join("lib/python3.10/site-packages").to_string_lossy()
            ));
        }

        fs::write(&env_file, env_content).await
            .with_context(|| format!("Failed to write env file: {:?}", env_file))?;

        Ok(())
    }

    /// 获取运行时环境的 PATH
    pub fn get_runtime_path(&self) -> Option<String> {
        let mut paths = Vec::new();

        for package in &self.packages {
            let runtime_dir = self.get_runtime_dir(&package.version.runtime_type);
            let bin_dir = runtime_dir.join("bin");

            if bin_dir.exists() {
                paths.push(bin_dir.to_string_lossy().to_string());
            }
        }

        if paths.is_empty() {
            None
        } else {
            Some(paths.join(":"))
        }
    }
}

/// Runtime 状态
#[derive(Debug, Clone)]
pub struct RuntimeStatus {
    pub package: RuntimePackage,
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<PathBuf>,
    pub message: String,
}

impl RuntimeStatus {
    /// 是否可以使用（已安装或系统有兼容版本）
    pub fn is_available(&self) -> bool {
        self.installed
    }

    /// 是否为必需但未安装
    pub fn is_required_missing(&self) -> bool {
        self.package.required && !self.installed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_package_names() {
        let node = RuntimePackage::node22();
        assert!(node.archive_name.contains("node-v22"));

        let python = RuntimePackage::python310();
        assert!(python.archive_name.contains("python-3.10"));
    }

    #[test]
    fn test_version_compatibility() {
        let bundle = RuntimeBundle::new().unwrap();

        let version = RuntimeVersion {
            runtime_type: RuntimeType::Node,
            version: "22.0.0".to_string(),
            platform: "darwin".to_string(),
            arch: "x64".to_string(),
        };

        assert!(bundle.is_version_compatible(&version, "v22.1.0"));
        assert!(bundle.is_version_compatible(&version, "v23.0.0"));
        assert!(!bundle.is_version_compatible(&version, "v18.0.0"));
    }
}
