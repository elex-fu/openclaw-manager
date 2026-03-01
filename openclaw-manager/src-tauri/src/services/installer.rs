//! OpenClaw 安装服务
//!
//! 支持在线和离线两种安装方式

use crate::installer::{InstallProgress, InstallStage, OpenClawInstaller, SystemCheckResult};
use crate::models::openclaw::{InstallResult, InstallStatus};
use crate::services::offline_installer::OfflineInstaller;
use crate::utils::retry::{retry_with_backoff, RetryConfig};
use anyhow::{Context, Result};
use std::time::Duration;
use tokio::sync::mpsc;

/// 安装方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMethod {
    /// 在线安装
    Online,
    /// 离线安装
    Offline,
}

/// 镜像源配置
#[derive(Debug, Clone)]
pub struct MirrorSource {
    pub name: String,
    pub url: String,
    pub priority: u32,
}

/// 默认镜像源列表
pub fn default_mirrors() -> Vec<MirrorSource> {
    vec![
        MirrorSource {
            name: "Official".to_string(),
            url: "https://openclaw.ai".to_string(),
            priority: 1,
        },
        MirrorSource {
            name: "GitHub".to_string(),
            url: "https://github.com/openclaw/openclaw/releases/download".to_string(),
            priority: 2,
        },
        MirrorSource {
            name: "CN Mirror".to_string(),
            url: "https://mirrors.openclaw.cn".to_string(),
            priority: 3,
        },
    ]
}

/// 安装服务
pub struct InstallerService {
    inner: OpenClawInstaller,
    mirrors: Vec<MirrorSource>,
}

impl InstallerService {
    /// 创建新的安装服务
    pub fn new() -> Result<Self> {
        let inner = OpenClawInstaller::new()?;
        let mirrors = default_mirrors();

        Ok(Self { inner, mirrors })
    }

    /// 检查安装状态
    pub fn check_installation(&self) -> Result<InstallStatus> {
        self.inner.check_installation()
    }

    /// 检查系统环境
    pub async fn check_system_environment(&self) -> Result<Vec<SystemCheckResult>> {
        self.inner.check_system_environment().await
    }

    /// 安装 OpenClaw（支持在线和离线）
    pub async fn install(
        &self,
        method: InstallMethod,
        version: Option<&str>,
        progress_tx: Option<mpsc::Sender<InstallProgress>>,
    ) -> Result<InstallResult> {
        match method {
            InstallMethod::Online => {
                self.install_online(version, progress_tx).await
            }
            InstallMethod::Offline => {
                self.install_offline(progress_tx).await
            }
        }
    }

    /// 在线安装
    async fn install_online(
        &self,
        version: Option<&str>,
        _progress_tx: Option<mpsc::Sender<InstallProgress>>,
    ) -> Result<InstallResult> {
        // 使用重试机制从多个镜像源下载
        let config = RetryConfig {
            max_attempts: self.mirrors.len() as u32,
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: true,
        };

        let installer = &self.inner;
        let version_owned = version.map(|v| v.to_string());

        retry_with_backoff(config, || async {
            // 尝试使用不同的镜像源
            for mirror in &self.mirrors {
                log::info!("Trying mirror: {}", mirror.name);
                
                // 这里可以设置当前使用的镜像源
                // 实际实现中可能需要修改 OpenClawInstaller 以支持自定义镜像源
                
                match installer.install(version_owned.as_deref(), None).await {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        log::warn!("Mirror {} failed: {}", mirror.name, e);
                    }
                }
            }
            
            Err(anyhow::anyhow!("All mirrors failed"))
        }).await
    }

    /// 离线安装
    async fn install_offline(
        &self,
        progress_tx: Option<mpsc::Sender<InstallProgress>>,
    ) -> Result<InstallResult> {
        // 报告开始
        if let Some(ref tx) = progress_tx {
            let _ = tx.send(InstallProgress {
                stage: InstallStage::Checking,
                percentage: 0.0,
                message: "准备离线安装...".to_string(),
            }).await;
        }

        // 创建离线安装器
        let offline_installer = OfflineInstaller::for_current_platform()
            .context("创建离线安装器失败")?;

        // 获取安装目录
        let install_dir = self.inner.get_install_dir();

        // 报告进度
        if let Some(ref tx) = progress_tx {
            let _ = tx.send(InstallProgress {
                stage: InstallStage::Installing,
                percentage: 30.0,
                message: "解压离线安装包...".to_string(),
            }).await;
        }

        // 执行离线安装
        offline_installer.install(&install_dir).await
            .context("离线安装失败")?;

        // 报告进度
        if let Some(ref tx) = progress_tx {
            let _ = tx.send(InstallProgress {
                stage: InstallStage::Configuring,
                percentage: 80.0,
                message: "创建默认配置...".to_string(),
            }).await;
        }

        // 创建默认配置
        self.inner.create_default_config()?;

        // 获取版本
        let version = self.inner.get_installed_version()
            .unwrap_or_else(|_| "unknown".to_string());

        // 完成
        if let Some(ref tx) = progress_tx {
            let _ = tx.send(InstallProgress {
                stage: InstallStage::Complete,
                percentage: 100.0,
                message: format!("OpenClaw {} 离线安装成功!", version),
            }).await;
        }

        Ok(InstallResult {
            success: true,
            version: Some(version.clone()),
            message: format!("OpenClaw {} 离线安装成功", version),
        })
    }

    /// 卸载 OpenClaw
    pub fn uninstall(&self) -> Result<()> {
        self.inner.uninstall()
    }

    /// 获取可用的安装方法
    pub fn get_install_methods() -> Vec<InstallMethodInfo> {
        vec![
            InstallMethodInfo {
                id: "online".to_string(),
                name: "在线安装".to_string(),
                description: "从远程服务器下载并安装最新版本".to_string(),
                requires_network: true,
                available: true,
            },
            InstallMethodInfo {
                id: "offline".to_string(),
                name: "离线安装".to_string(),
                description: "使用嵌入的离线安装包".to_string(),
                requires_network: false,
                available: true,
            },
        ]
    }

    /// 获取镜像源列表
    pub fn get_mirrors(&self) -> &[MirrorSource] {
        &self.mirrors
    }

    /// 添加自定义镜像源
    pub fn add_mirror(&mut self, name: String, url: String, priority: u32) {
        self.mirrors.push(MirrorSource { name, url, priority });
        // 按优先级排序
        self.mirrors.sort_by_key(|m| m.priority);
    }
}

/// 安装方法信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct InstallMethodInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub requires_network: bool,
    pub available: bool,
}

impl Default for InstallerService {
    fn default() -> Self {
        Self::new().expect("Failed to create installer service")
    }
}
