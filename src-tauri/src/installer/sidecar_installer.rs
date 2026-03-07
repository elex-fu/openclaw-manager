//! Sidecar 模式安装器
//!
//! 实现嵌入式 OpenClaw 的自动安装和配置
//! 包含解压、runtime 安装、npm install 等流程

use super::runtime_bundle::{RuntimeBundle, RuntimeType};
use super::{InstallProgress, InstallStage};
use crate::models::openclaw::{InstallResult, InstallStatus};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::mpsc;

/// OpenClaw Sidecar 安装器
pub struct SidecarInstaller {
    install_dir: PathBuf,
    progress_tx: Option<mpsc::Sender<InstallProgress>>,
    runtime_bundle: RuntimeBundle,
}

impl SidecarInstaller {
    /// 创建新的 Sidecar 安装器
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;
        let install_dir = home_dir.join(".openclaw").join("app");

        let runtime_bundle = RuntimeBundle::new()?;

        Ok(Self {
            install_dir,
            progress_tx: None,
            runtime_bundle,
        })
    }

    /// 设置进度发送通道
    pub fn with_progress_channel(mut self, tx: mpsc::Sender<InstallProgress>) -> Self {
        self.progress_tx = Some(tx);
        self
    }

    /// 获取安装目录
    pub fn get_install_dir(&self) -> &Path {
        &self.install_dir
    }

    /// 获取 OpenClaw 主目录
    pub fn get_openclaw_dir(&self) -> PathBuf {
        self.install_dir.join("openclaw")
    }

    /// 获取入口文件路径
    pub fn get_entry_path(&self) -> PathBuf {
        self.get_openclaw_dir().join("dist").join("index.js")
    }

    /// 检查 node_modules 是否存在
    pub async fn check_node_modules(&self) -> bool {
        self.get_openclaw_dir().join("node_modules").exists()
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
    pub async fn check_installation(&self) -> Result<InstallStatus> {
        let openclaw_dir = self.get_openclaw_dir();
        let entry_path = self.get_entry_path();

        // 检查目录和入口文件是否存在
        if !openclaw_dir.exists() || !entry_path.exists() {
            return Ok(InstallStatus::NotInstalled);
        }

        // 检查 node_modules 是否存在
        if !self.check_node_modules().await {
            return Ok(InstallStatus::NeedsDependencies);
        }

        // 尝试获取版本
        match self.get_installed_version().await {
            Ok(version) => Ok(InstallStatus::Installed { version }),
            Err(e) => Ok(InstallStatus::Error {
                message: format!("Found but failed to get version: {}", e),
            }),
        }
    }

    /// 获取已安装的版本
    pub async fn get_installed_version(&self) -> Result<String> {
        let openclaw_dir = self.get_openclaw_dir();
        let version_file = openclaw_dir.join("VERSION");

        if version_file.exists() {
            let version = fs::read_to_string(version_file).await?;
            return Ok(version.trim().to_string());
        }

        // 尝试从 package.json 读取
        let package_json = openclaw_dir.join("package.json");
        if package_json.exists() {
            let content = fs::read_to_string(package_json).await?;
            let json: serde_json::Value = serde_json::from_str(&content)?;
            if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
                return Ok(version.to_string());
            }
        }

        Ok("unknown".to_string())
    }

    /// 完整安装流程
    pub async fn install(&self) -> Result<InstallResult> {
        self.report_progress(InstallStage::Checking, 0.0, "检查安装环境...").await;

        // 创建安装目录
        fs::create_dir_all(&self.install_dir).await?;

        // 步骤1: 解压 OpenClaw  bundled 包
        self.report_progress(InstallStage::ExtractingRuntime, 5.0, "解压 OpenClaw 文件...").await;
        self.extract_openclaw().await?;

        // 步骤2: 安装/检查 Node.js Runtime
        self.report_progress(InstallStage::ExtractingRuntime, 30.0, "准备 Node.js 运行时...").await;
        self.setup_node_runtime().await?;

        // 步骤3: 安装 npm 依赖
        self.report_progress(InstallStage::Installing, 50.0, "安装 npm 依赖（可能需要几分钟）...").await;
        self.install_dependencies().await?;

        // 步骤4: 创建配置
        self.report_progress(InstallStage::Configuring, 90.0, "创建配置文件...").await;
        self.create_default_config().await?;

        // 完成
        let version = self.get_installed_version().await?;
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

    /// 从 bundled 目录解压 OpenClaw
    async fn extract_openclaw(&self) -> Result<()> {
        let source_dir = self.get_bundled_openclaw_path()?;
        let target_dir = self.get_openclaw_dir();

        if !source_dir.exists() {
            return Err(anyhow::anyhow!(
                "OpenClaw bundled 目录不存在: {}。请确保已正确打包应用。",
                source_dir.display()
            ));
        }

        log::info!("解压 OpenClaw: {:?} -> {:?}", source_dir, target_dir);

        // 创建目标目录
        fs::create_dir_all(&target_dir).await?;

        // 复制所有文件
        self.copy_dir_all(&source_dir, &target_dir).await?;

        log::info!("OpenClaw 解压完成");
        Ok(())
    }

    /// 设置 Node.js 运行时
    async fn setup_node_runtime(&self) -> Result<()> {
        // 检查系统是否已有 Node.js 22+
        if let Ok(Some(_)) = self.runtime_bundle.check_system_node().await {
            log::info!("使用系统 Node.js");
            return Ok(());
        }

        // 检查嵌入式 Node.js
        let node_status = self.runtime_bundle.check_node_status().await?;

        if !node_status.installed {
            // 尝试解压嵌入式 Node.js
            if let Ok(()) = self.runtime_bundle.extract_node().await {
                log::info!("嵌入式 Node.js 解压完成");
                return Ok(());
            }

            // 如果没有嵌入式包，尝试下载
            self.report_progress(InstallStage::Downloading, 35.0, "下载 Node.js...").await;
            let mut progress = |msg: &str, pct: f32| {
                let _ = self.report_progress(InstallStage::Downloading, 35.0 + pct * 0.15, msg.to_string());
            };
            self.runtime_bundle.download_and_install_node(&mut progress).await?;
        }

        Ok(())
    }

    /// 安装 npm 依赖
    async fn install_dependencies(&self) -> Result<()> {
        let openclaw_dir = self.get_openclaw_dir();

        if !openclaw_dir.join("package.json").exists() {
            return Err(anyhow::anyhow!("package.json 不存在"));
        }

        let mut progress = |msg: &str, pct: f32| {
            let _ = self.report_progress(InstallStage::Installing, 50.0 + pct * 0.4, msg.to_string());
        };

        self.runtime_bundle.npm_install(&openclaw_dir, &mut progress).await?;

        Ok(())
    }

    /// 创建默认配置
    async fn create_default_config(&self) -> Result<()> {
        let config_dir = dirs::config_dir()
            .map(|d| d.join("openclaw"))
            .unwrap_or_else(|| self.install_dir.join("config"));

        fs::create_dir_all(&config_dir).await?;

        let config_file = config_dir.join("config.yaml");

        if !config_file.exists() {
            let default_config = r#"# OpenClaw 配置文件
# 首次使用请配置 API Keys

# AI 模型配置
models:
  default: openai

# 日志级别
log_level: info

# 服务器配置
server:
  port: 3000
  host: localhost
"#;

            fs::write(&config_file, default_config).await?;
        }

        Ok(())
    }

    /// 获取 bundled OpenClaw 路径
    fn get_bundled_openclaw_path(&self) -> Result<PathBuf> {
        // 1. 检查资源目录（生产环境）
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().context("Failed to get exe dir")?;

        let resource_paths = vec![
            exe_dir.join("../Resources/openclaw"), // macOS app bundle
            exe_dir.join("resources/openclaw"),    // Tauri v2 resources
            exe_dir.join("bundled/openclaw"),      // 开发环境
        ];

        for path in resource_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        // 2. 检查开发环境
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let dev_path = manifest_dir.join("bundled").join("openclaw");
        if dev_path.exists() {
            return Ok(dev_path);
        }

        Err(anyhow::anyhow!(
            "找不到 OpenClaw bundled 目录。请确保应用已正确打包。"
        ))
    }

    /// 复制目录（使用栈实现，避免递归导致的 Send 边界问题）
    async fn copy_dir_all(&self, src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
        let src = src.as_ref().to_path_buf();
        let dst = dst.as_ref().to_path_buf();

        // 使用栈来处理目录
        let mut stack = vec![(src, dst)];

        while let Some((current_src, current_dst)) = stack.pop() {
            fs::create_dir_all(&current_dst).await?;

            let mut entries = fs::read_dir(&current_src).await?;

            while let Some(entry) = entries.next_entry().await? {
                let file_type = entry.file_type().await?;
                let src_path = entry.path();
                let dst_path = current_dst.join(entry.file_name());

                if file_type.is_dir() {
                    // 跳过 node_modules（如果存在）
                    if entry.file_name() == "node_modules" {
                        continue;
                    }
                    stack.push((src_path, dst_path));
                } else {
                    fs::copy(&src_path, &dst_path).await?;
                }
            }
        }

        Ok(())
    }

    /// 卸载 OpenClaw
    pub async fn uninstall(&self) -> Result<()> {
        if self.install_dir.exists() {
            fs::remove_dir_all(&self.install_dir).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidecar_installer_new() {
        let installer = SidecarInstaller::new();
        assert!(installer.is_ok());
    }
}
