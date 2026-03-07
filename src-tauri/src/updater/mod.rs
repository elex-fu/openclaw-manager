//! OpenClaw 升级管理模块
//!
//! 提供平滑升级功能：
//! - 版本检查和比较
//! - 自动/手动升级
//! - 配置备份和迁移
//! - 原子替换和回滚

use crate::installer::OpenClawInstaller;
use crate::models::openclaw::InstallResult;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use log::{info, warn, error};

pub mod version;
pub use version::Version;

/// 升级状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateState {
    /// 当前安装的版本
    pub current_version: Option<String>,
    /// 最新可用版本
    pub latest_version: Option<String>,
    /// 是否有可用更新
    pub has_update: bool,
    /// 更新信息
    pub update_info: Option<UpdateInfo>,
}

/// 更新信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// 版本号
    pub version: String,
    /// 发布日期
    pub release_date: String,
    /// 更新日志
    pub changelog: String,
    /// 下载URL
    pub download_url: String,
    /// 文件校验和
    pub checksum: String,
    /// 是否强制更新
    pub mandatory: bool,
    /// 最小支持版本（低于此版本必须更新）
    pub min_supported_version: Option<String>,
}

/// 升级进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgress {
    pub stage: UpdateStage,
    pub percentage: f32,
    pub message: String,
    pub can_cancel: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UpdateStage {
    Checking,
    Downloading,
    BackingUp,
    Installing,
    Migrating,
    CleaningUp,
    Complete,
    Error,
    Rollback,
}

impl std::fmt::Display for UpdateStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateStage::Checking => write!(f, "Checking"),
            UpdateStage::Downloading => write!(f, "Downloading"),
            UpdateStage::BackingUp => write!(f, "BackingUp"),
            UpdateStage::Installing => write!(f, "Installing"),
            UpdateStage::Migrating => write!(f, "Migrating"),
            UpdateStage::CleaningUp => write!(f, "CleaningUp"),
            UpdateStage::Complete => write!(f, "Complete"),
            UpdateStage::Error => write!(f, "Error"),
            UpdateStage::Rollback => write!(f, "Rollback"),
        }
    }
}

/// 升级管理器
pub struct UpdateManager {
    installer: OpenClawInstaller,
    progress_tx: Option<mpsc::Sender<UpdateProgress>>,
    backup_dir: PathBuf,
    temp_dir: PathBuf,
}

impl UpdateManager {
    /// 创建新的升级管理器
    pub fn new() -> Result<Self> {
        let installer = OpenClawInstaller::new()?;
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;

        let backup_dir = home_dir.join(".openclaw").join("backups");
        let temp_dir = home_dir.join(".openclaw").join(".temp");

        // 确保目录存在
        fs::create_dir_all(&backup_dir)?;
        fs::create_dir_all(&temp_dir)?;

        Ok(Self {
            installer,
            progress_tx: None,
            backup_dir,
            temp_dir,
        })
    }

    /// 设置进度通道
    pub fn with_progress_channel(mut self, tx: mpsc::Sender<UpdateProgress>) -> Self {
        self.progress_tx = Some(tx);
        self
    }

    /// 报告进度
    async fn report_progress(&self, stage: UpdateStage, percentage: f32, message: impl Into<String>, can_cancel: bool) {
        if let Some(ref tx) = self.progress_tx {
            let progress = UpdateProgress {
                stage,
                percentage,
                message: message.into(),
                can_cancel,
            };
            let _ = tx.send(progress).await;
        }
    }

    /// 检查更新
    ///
    /// 从远程服务器获取最新版本信息，与当前版本比较
    pub async fn check_update(&self) -> Result<UpdateState> {
        self.report_progress(UpdateStage::Checking, 0.0, "正在检查更新...", true).await;

        // 获取当前版本
        let current_version = match self.installer.get_installed_version() {
            Ok(v) => Some(v),
            Err(_) => None,
        };

        info!("当前版本: {:?}", current_version);

        // 获取最新版本信息（从远程或本地缓存）
        let update_info = self.fetch_latest_version_info().await?;

        // 比较版本
        let has_update = match (&current_version, &update_info) {
            (Some(current), Some(info)) => {
                let current_v = Version::parse(current)?;
                let latest_v = Version::parse(&info.version)?;
                latest_v > current_v
            }
            (None, Some(_)) => true, // 未安装但有新版本
            _ => false,
        };

        let latest_version = update_info.as_ref().map(|i| i.version.clone());

        self.report_progress(UpdateStage::Checking, 100.0,
            if has_update { "发现新版本" } else { "已是最新版本" }, true).await;

        Ok(UpdateState {
            current_version,
            latest_version,
            has_update,
            update_info,
        })
    }

    /// 从远程获取最新版本信息
    ///
    /// 支持多个镜像源，自动切换：
    /// 1. 官方 API (api.openclaw.ai)
    /// 2. GitHub Releases (api.github.com)
    /// 3. 国内镜像 (mirror.openclaw.cn)
    async fn fetch_latest_version_info(&self) -> Result<Option<UpdateInfo>> {
        // 定义多个版本检查源
        let mirrors = [
            "https://api.openclaw.ai/releases/latest",
            "https://api.github.com/repos/openclaw/openclaw/releases/latest",
            "https://mirror.openclaw.cn/releases/latest",
        ];

        // 尝试从各个源获取版本信息
        for (index, url) in mirrors.iter().enumerate() {
            info!("尝试从源 {} 获取版本信息: {}", index + 1, url);

            match self.fetch_from_source(url).await {
                Ok(Some(info)) => {
                    info!("成功从源 {} 获取版本信息: v{}", index + 1, info.version);
                    // 缓存版本信息
                    if let Err(e) = self.cache_version_info(&info) {
                        warn!("缓存版本信息失败: {}", e);
                    }
                    return Ok(Some(info));
                }
                Ok(None) => {
                    warn!("源 {} 返回空版本信息", index + 1);
                }
                Err(e) => {
                    warn!("从源 {} 获取版本信息失败: {}", index + 1, e);
                }
            }
        }

        // 所有远程源都失败，尝试使用缓存
        warn!("所有远程源都不可用，尝试使用本地缓存");
        match self.load_cached_version_info() {
            Ok(Some(info)) => {
                info!("使用缓存的版本信息: v{}", info.version);
                Ok(Some(info))
            }
            Ok(None) => {
                warn!("没有可用的缓存版本信息");
                Ok(None)
            }
            Err(e) => {
                error!("加载缓存版本信息失败: {}", e);
                Ok(None)
            }
        }
    }

    /// 从指定源获取版本信息
    async fn fetch_from_source(&self, url: &str) -> Result<Option<UpdateInfo>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .context("Failed to build HTTP client")?;

        let response = client
            .get(url)
            .header("User-Agent", "OpenClaw-Manager/0.1.0")
            .send()
            .await
            .context(format!("Failed to fetch from {}", url))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP error: {} from {}",
                response.status(),
                url
            ));
        }

        // 解析响应（支持多种格式）
        let text = response.text().await?;

        // 尝试直接解析为 UpdateInfo
        match serde_json::from_str::<UpdateInfo>(&text) {
            Ok(info) => return Ok(Some(info)),
            Err(_) => {}
        }

        // 尝试解析 GitHub Releases 格式
        match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(json) => {
                if let Some(tag_name) = json.get("tag_name").and_then(|v| v.as_str()) {
                    let version = tag_name.trim_start_matches('v').to_string();
                    let release_date = json
                        .get("published_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let changelog = json
                        .get("body")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    // 查找当前平台的资源
                    let platform_suffix = self.get_platform_suffix();
                    let download_url = json
                        .get("assets")
                        .and_then(|a| a.as_array())
                        .and_then(|assets| {
                            assets.iter().find_map(|asset| {
                                let name = asset.get("name")?.as_str()?;
                                if name.contains(&platform_suffix) {
                                    asset.get("browser_download_url")?.as_str().map(|s| s.to_string())
                                } else {
                                    None
                                }
                            })
                        })
                        .unwrap_or_else(|| format!("https://github.com/openclaw/openclaw/releases/download/{}/openclaw-{}.tar.gz", tag_name, platform_suffix));

                    return Ok(Some(UpdateInfo {
                        version,
                        release_date,
                        changelog,
                        download_url,
                        checksum: String::new(), // GitHub 不提供校验和
                        mandatory: false,
                        min_supported_version: None,
                    }));
                }
            }
            Err(_) => {}
        }

        Err(anyhow::anyhow!("无法解析版本信息"))
    }

    /// 获取当前平台的后缀名
    fn get_platform_suffix(&self) -> String {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        format!("{}-{}", os, arch)
    }

    /// 缓存版本信息到本地
    fn cache_version_info(&self, info: &UpdateInfo) -> Result<()> {
        let cache_path = self.temp_dir.parent().unwrap().join("version_cache.json");
        let json = serde_json::to_string(info)?;
        fs::write(cache_path, json)?;
        Ok(())
    }

    /// 加载缓存的版本信息
    fn load_cached_version_info(&self) -> Result<Option<UpdateInfo>> {
        let cache_path = self.temp_dir.parent().unwrap().join("version_cache.json");

        if !cache_path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(cache_path)?;
        let info: UpdateInfo = serde_json::from_str(&json)?;
        Ok(Some(info))
    }

    /// 执行升级
    ///
    /// 完整的升级流程：
    /// 1. 备份当前安装
    /// 2. 下载新版本
    /// 3. 安装新版本
    /// 4. 迁移配置
    /// 5. 清理临时文件
    pub async fn update(&self, update_info: &UpdateInfo) -> Result<InstallResult> {
        info!("开始升级到版本: {}", update_info.version);

        // 步骤 1: 创建备份
        self.report_progress(UpdateStage::BackingUp, 5.0, "正在创建备份...", true).await;
        let backup_path = self.create_backup().await?;
        info!("备份创建完成: {:?}", backup_path);

        // 步骤 2: 下载新版本
        self.report_progress(UpdateStage::Downloading, 10.0, "正在下载新版本...", true).await;
        let download_path = self.download_update(update_info).await?;
        info!("下载完成: {:?}", download_path);

        // 步骤 3: 验证下载
        self.report_progress(UpdateStage::Downloading, 50.0, "正在验证下载文件...", false).await;
        self.verify_download(&download_path, &update_info.checksum).await?;
        info!("下载验证通过");

        // 步骤 4: 安装新版本（带自动回滚）
        self.report_progress(UpdateStage::Installing, 60.0, "正在安装新版本...", false).await;

        let install_result = match self.install_update(&download_path).await {
            Ok(result) => {
                info!("安装成功");
                result
            }
            Err(e) => {
                error!("安装失败: {}", e);
                self.report_progress(UpdateStage::Rollback, 70.0, "安装失败，正在回滚...", false).await;

                // 自动回滚
                if let Err(rollback_err) = self.rollback(&backup_path).await {
                    error!("回滚失败: {}", rollback_err);
                    return Err(anyhow::anyhow!(
                        "更新失败且回滚失败: {} (回滚错误: {})", e, rollback_err
                    ));
                }

                info!("回滚完成");
                return Err(anyhow::anyhow!("更新失败，已自动回滚到之前版本: {}", e));
            }
        };

        // 步骤 5: 迁移配置
        self.report_progress(UpdateStage::Migrating, 80.0, "正在迁移配置...", false).await;
        if let Err(e) = self.migrate_config(&update_info.version).await {
            warn!("配置迁移出现问题: {}", e);
            // 配置迁移失败不阻止升级完成
        }

        // 步骤 6: 清理临时文件
        self.report_progress(UpdateStage::CleaningUp, 95.0, "正在清理临时文件...", false).await;
        self.cleanup_temp_files().await?;

        // 完成
        self.report_progress(UpdateStage::Complete, 100.0,
            &format!("升级完成！当前版本: {}", update_info.version), false).await;

        Ok(install_result)
    }

    /// 离线升级（使用本地安装包）
    pub async fn update_offline(&self, package_path: &Path) -> Result<InstallResult> {
        info!("开始离线升级: {:?}", package_path);

        // 验证包存在
        if !package_path.exists() {
            return Err(anyhow::anyhow!("安装包不存在: {:?}", package_path));
        }

        // 创建备份
        self.report_progress(UpdateStage::BackingUp, 5.0, "正在创建备份...", true).await;
        let backup_path = self.create_backup().await?;

        // 安装更新（带自动回滚）
        self.report_progress(UpdateStage::Installing, 20.0, "正在安装...", false).await;

        let install_result = match self.install_update(package_path).await {
            Ok(result) => result,
            Err(e) => {
                error!("离线安装失败: {}", e);
                self.report_progress(UpdateStage::Rollback, 50.0, "安装失败，正在回滚...", false).await;

                if let Err(rollback_err) = self.rollback(&backup_path).await {
                    return Err(anyhow::anyhow!(
                        "离线更新失败且回滚失败: {} (回滚错误: {})", e, rollback_err
                    ));
                }

                return Err(anyhow::anyhow!("离线更新失败，已自动回滚: {}", e));
            }
        };

        // 迁移配置
        self.report_progress(UpdateStage::Migrating, 80.0, "正在迁移配置...", false).await;
        let _ = self.migrate_config("unknown").await;

        // 清理
        self.report_progress(UpdateStage::CleaningUp, 95.0, "正在清理...", false).await;
        self.cleanup_temp_files().await?;

        self.report_progress(UpdateStage::Complete, 100.0, "离线升级完成！", false).await;

        Ok(install_result)
    }

    /// 创建备份
    async fn create_backup(&self) -> Result<PathBuf> {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("backup_{}", timestamp);
        let backup_path = self.backup_dir.join(&backup_name);

        fs::create_dir_all(&backup_path)?;

        let install_dir = self.installer.get_install_dir();

        // 备份二进制文件
        let bin_dir = install_dir.join("bin");
        if bin_dir.exists() {
            let backup_bin = backup_path.join("bin");
            fs::create_dir_all(&backup_bin)?;
            self.copy_dir_contents(&bin_dir, &backup_bin)?;
        }

        // 备份配置
        let config_path = install_dir.join("config.yaml");
        if config_path.exists() {
            fs::copy(&config_path, backup_path.join("config.yaml"))?;
        }

        // 备份 VERSION 文件（如果存在）
        let version_path = install_dir.join("VERSION");
        if version_path.exists() {
            fs::copy(&version_path, backup_path.join("VERSION"))?;
        }

        // 记录备份元数据
        let meta = BackupMetadata {
            created_at: chrono::Local::now().to_rfc3339(),
            version: self.installer.get_installed_version().ok(),
            path: backup_path.clone(),
        };

        let meta_json = serde_json::to_string(&meta)?;
        fs::write(backup_path.join(".backup_meta.json"), meta_json)?;

        Ok(backup_path)
    }

    /// 下载更新
    async fn download_update(&self, update_info: &UpdateInfo) -> Result<PathBuf> {
        let filename = format!("openclaw_update_{}.tar.gz", update_info.version);
        let download_path = self.temp_dir.join(&filename);

        // 使用 reqwest 下载
        let response = reqwest::get(&update_info.download_url)
            .await
            .context("Failed to download update")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Download failed with status: {}",
                response.status()
            ));
        }

        let bytes = response.bytes().await?;
        fs::write(&download_path, bytes)?;

        Ok(download_path)
    }

    /// 验证下载文件
    async fn verify_download(&self, path: &Path, expected_checksum: &str) -> Result<()> {
        use sha2::{Sha256, Digest};
        use std::io::Read;

        if expected_checksum.is_empty() {
            warn!("未提供校验和，跳过验证");
            return Ok(());
        }

        let mut file = fs::File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let result = hasher.finalize();
        let actual_checksum = format!("{:x}", result);

        if actual_checksum != expected_checksum {
            return Err(anyhow::anyhow!(
                "Checksum mismatch: expected {}, got {}",
                expected_checksum,
                actual_checksum
            ));
        }

        Ok(())
    }

    /// 安装更新
    async fn install_update(&self, package_path: &Path) -> Result<InstallResult> {
        let install_dir = self.installer.get_install_dir().to_path_buf();

        // 解压新版本的步骤：
        // 1. 先解压到临时目录
        // 2. 停止正在运行的服务
        // 3. 原子替换文件

        let temp_extract_dir = self.temp_dir.join("extract_new");
        if temp_extract_dir.exists() {
            fs::remove_dir_all(&temp_extract_dir)?;
        }
        fs::create_dir_all(&temp_extract_dir)?;

        // 解压
        let output = tokio::process::Command::new("tar")
            .args(["-xzf", package_path.to_str().unwrap(), "-C", temp_extract_dir.to_str().unwrap()])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to extract update package"));
        }

        // 找到解压后的目录（可能有前缀目录如 openclaw/）
        let extracted_content = if temp_extract_dir.join("openclaw").exists() {
            temp_extract_dir.join("openclaw")
        } else {
            temp_extract_dir.clone()
        };

        // 停止服务（如果正在运行）
        self.report_progress(UpdateStage::Installing, 65.0, "正在停止服务...", false).await;
        let _ = self.stop_service().await;

        // 原子替换：
        // 1. 移动当前 bin 到临时位置
        // 2. 移动新 bin 到安装目录
        // 3. 如果失败则恢复

        let bin_dir = install_dir.join("bin");
        let bin_backup = self.temp_dir.join("bin_old");

        if bin_dir.exists() {
            if bin_backup.exists() {
                fs::remove_dir_all(&bin_backup)?;
            }
            fs::rename(&bin_dir, &bin_backup)?;
        }

        // 移动新文件
        let new_bin_dir = extracted_content.join("bin");
        if new_bin_dir.exists() {
            match fs::rename(&new_bin_dir, &bin_dir) {
                Ok(_) => {}
                Err(e) => {
                    // 恢复旧版本
                    if bin_backup.exists() {
                        let _ = fs::rename(&bin_backup, &bin_dir);
                    }
                    return Err(anyhow::anyhow!("Failed to install new version: {}", e));
                }
            }
        }

        // 设置执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let openclaw_bin = bin_dir.join("openclaw");
            if openclaw_bin.exists() {
                let mut perms = fs::metadata(&openclaw_bin)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&openclaw_bin, perms)?;
            }
        }

        // 验证新版本可以运行
        let new_version = self.installer.get_installed_version()?;

        Ok(InstallResult {
            success: true,
            version: Some(new_version),
            message: "更新安装成功".to_string(),
        })
    }

    /// 回滚到备份版本
    async fn rollback(&self, backup_path: &Path) -> Result<()> {
        info!("开始回滚到备份: {:?}", backup_path);

        let install_dir = self.installer.get_install_dir();

        // 恢复二进制文件
        let backup_bin = backup_path.join("bin");
        let install_bin = install_dir.join("bin");

        if backup_bin.exists() {
            if install_bin.exists() {
                fs::remove_dir_all(&install_bin)?;
            }
            self.copy_dir_contents(&backup_bin, &install_bin)?;
        }

        // 恢复配置
        let backup_config = backup_path.join("config.yaml");
        if backup_config.exists() {
            fs::copy(&backup_config, install_dir.join("config.yaml"))?;
        }

        info!("回滚完成");
        Ok(())
    }

    /// 迁移配置
    ///
    /// 根据版本差异自动调整配置格式
    /// 支持从旧版本到新版本的配置迁移
    async fn migrate_config(&self, new_version: &str) -> Result<()> {
        info!("开始配置迁移，目标版本: {}", new_version);

        let config_path = self.installer.get_install_dir().join("config.yaml");
        if !config_path.exists() {
            info!("配置文件不存在，跳过迁移");
            return Ok(());
        }

        // 读取当前配置
        let config_content = fs::read_to_string(&config_path)?;
        let mut config: serde_yaml::Value = serde_yaml::from_str(&config_content)?;

        // 获取当前配置版本（如果没有则假设为 "0.0.0"）
        let current_version = config
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string(); // 克隆为 String 避免借用问题

        info!("当前配置版本: {}, 目标版本: {}", current_version, new_version);

        // 版本比较，执行相应的迁移逻辑
        let needs_migration = Self::compare_versions(&current_version, new_version)?;

        if !needs_migration {
            info!("配置已是最新版本，无需迁移");
            return Ok(());
        }

        // 执行迁移步骤
        let migrations: Vec<(&str, Box<dyn FnOnce(&mut serde_yaml::Value) -> Result<()>>)> = vec![
            ("0.1.0", Box::new(Self::migrate_to_v0_1_0)),
            ("0.2.0", Box::new(Self::migrate_to_v0_2_0)),
            ("0.3.0", Box::new(Self::migrate_to_v0_3_0)),
        ];

        for (version, migration) in migrations {
            if Self::compare_versions(&current_version, version)?
                && !Self::compare_versions(new_version, version)?
            {
                info!("执行迁移到版本 {}", version);
                migration(&mut config)?;
            }
        }

        // 更新配置版本
        if let Some(map) = config.as_mapping_mut() {
            map.insert(
                serde_yaml::Value::String("version".to_string()),
                serde_yaml::Value::String(new_version.to_string()),
            );
        }

        // 备份旧配置
        let backup_path = config_path.with_extension("yaml.backup");
        fs::copy(&config_path, &backup_path)?;
        info!("旧配置已备份到: {:?}", backup_path);

        // 写入新配置
        let new_content = serde_yaml::to_string(&config)?;
        fs::write(&config_path, new_content)?;

        info!("配置迁移完成");
        Ok(())
    }

    /// 比较两个版本号
    /// 返回 true 如果 v1 < v2
    fn compare_versions(v1: &str, v2: &str) -> Result<bool> {
        let parse_version = |v: &str| -> Result<Vec<u32>> {
            v.trim_start_matches('v')
                .split('.')
                .map(|n| n.parse::<u32>().map_err(|e| anyhow::anyhow!("Invalid version: {}", e)))
                .collect::<Result<Vec<_>>>()
        };

        let v1_parts = parse_version(v1)?;
        let v2_parts = parse_version(v2)?;

        for (p1, p2) in v1_parts.iter().zip(v2_parts.iter()) {
            if p1 < p2 {
                return Ok(true);
            }
            if p1 > p2 {
                return Ok(false);
            }
        }

        Ok(v1_parts.len() < v2_parts.len())
    }

    /// 迁移到 v0.1.0
    fn migrate_to_v0_1_0(config: &mut serde_yaml::Value) -> Result<()> {
        info!("执行 v0.1.0 迁移");

        // 添加新的默认字段
        if let Some(map) = config.as_mapping_mut() {
            // 确保 models 字段存在
            if !map.contains_key(&serde_yaml::Value::String("models".to_string())) {
                map.insert(
                    serde_yaml::Value::String("models".to_string()),
                    serde_yaml::Value::Sequence(vec![]),
                );
            }

            // 确保 agents 字段存在
            if !map.contains_key(&serde_yaml::Value::String("agents".to_string())) {
                map.insert(
                    serde_yaml::Value::String("agents".to_string()),
                    serde_yaml::Value::Sequence(vec![]),
                );
            }
        }

        Ok(())
    }

    /// 迁移到 v0.2.0
    fn migrate_to_v0_2_0(config: &mut serde_yaml::Value) -> Result<()> {
        info!("执行 v0.2.0 迁移");

        // 重命名字段示例
        if let Some(map) = config.as_mapping_mut() {
            // 如果存在旧的 api_key 字段，迁移到新的 secure_storage 格式
            if let Some(api_key) = map.remove(&serde_yaml::Value::String("api_key".to_string())) {
                info!("发现旧版 api_key 配置，已移除（应使用安全存储）");
                drop(api_key);
            }
        }

        Ok(())
    }

    /// 迁移到 v0.3.0
    fn migrate_to_v0_3_0(config: &mut serde_yaml::Value) -> Result<()> {
        info!("执行 v0.3.0 迁移");

        // 添加插件配置
        if let Some(map) = config.as_mapping_mut() {
            if !map.contains_key(&serde_yaml::Value::String("plugins".to_string())) {
                map.insert(
                    serde_yaml::Value::String("plugins".to_string()),
                    serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
                );
            }
        }

        Ok(())
    }

    /// 清理临时文件
    async fn cleanup_temp_files(&self) -> Result<()> {
        // 保留最近 5 个备份，删除旧的
        self.rotate_backups(5).await?;

        // 清理临时下载文件
        let entries = fs::read_dir(&self.temp_dir)?;
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("openclaw_update"))
                    .unwrap_or(false)
                {
                    let _ = fs::remove_file(&path);
                }
            }
        }

        Ok(())
    }

    /// 轮换备份，只保留指定数量
    async fn rotate_backups(&self, keep_count: usize) -> Result<()> {
        let mut backups: Vec<_> = fs::read_dir(&self.backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| n.starts_with("backup_"))
                    .unwrap_or(false)
            })
            .collect();

        // 按修改时间排序（最新的在前）
        backups.sort_by(|a, b| {
            let a_time = a.metadata().and_then(|m| m.modified()).ok();
            let b_time = b.metadata().and_then(|m| m.modified()).ok();
            b_time.cmp(&a_time)
        });

        // 删除旧的备份
        if backups.len() > keep_count {
            for entry in &backups[keep_count..] {
                let path = entry.path();
                info!("删除旧备份: {:?}", path);
                let _ = fs::remove_dir_all(&path);
            }
        }

        Ok(())
    }

    /// 停止服务
    ///
    /// 使用 ProcessManager 停止 OpenClaw 服务
    /// 尝试优雅关闭，超时后强制终止
    async fn stop_service(&self) -> Result<()> {
        use crate::services::process_manager::ProcessManager;
        use std::sync::Arc;

        info!("停止 OpenClaw 服务");

        // 创建进程管理器
        let process_manager = Arc::new(ProcessManager::new());

        // 尝试停止 openclaw 服务
        const SERVICE_NAME: &str = "openclaw";
        const STOP_TIMEOUT_SECS: u64 = 30;

        // 检查服务是否正在运行
        match process_manager.get_status(SERVICE_NAME).await {
            Some(status) => {
                info!("服务当前状态: {:?}", status);

                match process_manager
                    .stop_service(SERVICE_NAME, STOP_TIMEOUT_SECS)
                    .await
                {
                    Ok(()) => {
                        info!("服务已优雅停止");
                        Ok(())
                    }
                    Err(e) => {
                        warn!("优雅停止服务失败: {}，尝试强制终止", e);

                        // 强制终止
                        if let Err(kill_err) = process_manager.force_kill(SERVICE_NAME).await {
                            return Err(anyhow::anyhow!(
                                "无法停止服务: {} (强制终止也失败: {})",
                                e,
                                kill_err
                            ));
                        }

                        info!("服务已强制终止");
                        Ok(())
                    }
                }
            }
            None => {
                info!("服务未运行，无需停止");
                Ok(())
            }
        }
    }

    /// 复制目录内容
    fn copy_dir_contents(&self, src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                self.copy_dir_contents(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    /// 获取所有备份列表
    pub fn list_backups(&self) -> Result<Vec<BackupMetadata>> {
        let mut backups = Vec::new();

        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let meta_path = path.join(".backup_meta.json");
                if meta_path.exists() {
                    let json = fs::read_to_string(meta_path)?;
                    let meta: BackupMetadata = serde_json::from_str(&json)?;
                    backups.push(meta);
                }
            }
        }

        // 按时间排序
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }

    /// 从指定备份恢复
    pub async fn restore_from_backup(&self, backup_path: &Path) -> Result<()> {
        info!("从备份恢复: {:?}", backup_path);
        self.rollback(backup_path).await
    }
}

/// 备份元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub created_at: String,
    pub version: Option<String>,
    pub path: PathBuf,
}

impl Default for UpdateManager {
    fn default() -> Self {
        Self::new().expect("Failed to create update manager")
    }
}

/// 获取嵌入式最新版本（用于 bundled 场景）
pub fn get_bundled_latest_version() -> Option<UpdateInfo> {
    // 从 bundled 目录读取版本信息
    let bundled_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bundled");
    let version_file = bundled_dir.join("LATEST_VERSION.json");

    if version_file.exists() {
        if let Ok(json) = fs::read_to_string(version_file) {
            if let Ok(info) = serde_json::from_str::<UpdateInfo>(&json) {
                return Some(info);
            }
        }
    }

    None
}
