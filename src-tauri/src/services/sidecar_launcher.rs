//! Sidecar 启动器
//!
//! 使用 Tauri Command API 启动 OpenClaw Node.js 进程
//! 管理进程生命周期、日志输出和错误处理

use crate::installer::SidecarInstaller;
use anyhow::{Context, Result};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

/// Sidecar 进程状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum SidecarState {
    Stopped,
    Starting,
    Running { pid: u32 },
    Error(String),
}

/// Sidecar 启动器
pub struct SidecarLauncher {
    installer: SidecarInstaller,
    process: Arc<Mutex<Option<Child>>>,
    state: Arc<Mutex<SidecarState>>,
}

impl SidecarLauncher {
    /// 创建新的 Sidecar 启动器
    pub fn new() -> Result<Self> {
        let installer = SidecarInstaller::new()?;

        Ok(Self {
            installer,
            process: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(SidecarState::Stopped)),
        })
    }

    /// 获取当前状态
    pub async fn get_state(&self) -> SidecarState {
        self.state.lock().await.clone()
    }

    /// 检查 OpenClaw 是否已安装
    pub async fn is_installed(&self) -> Result<bool> {
        match self.installer.check_installation().await? {
            crate::models::openclaw::InstallStatus::Installed { .. } => Ok(true),
            _ => Ok(false),
        }
    }

    /// 启动 OpenClaw 服务
    pub async fn start(&self) -> Result<u32> {
        // 检查是否已运行
        {
            let state = self.state.lock().await;
            if matches!(*state, SidecarState::Running { .. }) {
                return Err(anyhow::anyhow!("OpenClaw 已经在运行中"));
            }
        }

        // 设置状态为启动中
        *self.state.lock().await = SidecarState::Starting;

        // 获取安装目录
        let openclaw_dir = self.installer.get_openclaw_dir();
        let entry_path = self.installer.get_entry_path();

        // 检查入口文件
        if !entry_path.exists() {
            *self.state.lock().await = SidecarState::Error("OpenClaw 未安装".to_string());
            return Err(anyhow::anyhow!("OpenClaw entry file not found: {:?}", entry_path));
        }

        // 获取 Node.js 路径
        let node_path = self.get_node_path().await?;

        log::info!("启动 OpenClaw: node={:?}, entry={:?}", node_path, entry_path);

        // 启动进程
        let mut child = Command::new(&node_path)
            .arg(&entry_path)
            .arg("serve")
            .current_dir(&openclaw_dir)
            .env("NODE_PATH", openclaw_dir.join("node_modules"))
            .env("OPENCLAW_HOME", &openclaw_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn OpenClaw process")?;

        let pid = child.id().context("Failed to get process ID")?;

        // 设置 stdout/stderr 日志
        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(Self::log_stdout(BufReader::new(stdout).lines()));
        }

        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(Self::log_stderr(BufReader::new(stderr).lines()));
        }

        // 保存进程
        *self.process.lock().await = Some(child);

        // 更新状态
        *self.state.lock().await = SidecarState::Running { pid };

        log::info!("OpenClaw 启动成功，PID: {}", pid);
        Ok(pid)
    }

    /// 停止 OpenClaw 服务
    pub async fn stop(&self) -> Result<()> {
        let mut process = self.process.lock().await;

        if let Some(mut child) = process.take() {
            // 尝试优雅停止
            match child.kill().await {
                Ok(_) => {
                    log::info!("OpenClaw 进程已停止");
                }
                Err(e) => {
                    log::warn!("停止 OpenClaw 进程失败: {}", e);
                }
            }

            // 等待进程退出
            let _ = child.wait().await;
        }

        *self.state.lock().await = SidecarState::Stopped;
        Ok(())
    }

    /// 重启 OpenClaw 服务
    pub async fn restart(&self) -> Result<u32> {
        self.stop().await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        self.start().await
    }

    /// 执行 OpenClaw CLI 命令
    pub async fn execute_command(&self, args: &[String]) -> Result<String> {
        let openclaw_dir = self.installer.get_openclaw_dir();
        let entry_path = self.installer.get_entry_path();

        if !entry_path.exists() {
            return Err(anyhow::anyhow!("OpenClaw 未安装"));
        }

        let node_path = self.get_node_path().await?;

        log::debug!("执行命令: node {:?} {}", entry_path, args.join(" "));

        let output = Command::new(&node_path)
            .arg(&entry_path)
            .args(args)
            .current_dir(&openclaw_dir)
            .env("NODE_PATH", openclaw_dir.join("node_modules"))
            .env("OPENCLAW_HOME", &openclaw_dir)
            .output()
            .await
            .context("Failed to execute OpenClaw command")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "命令执行失败: {}\nstdout: {}\nstderr: {}",
                output.status,
                stdout,
                stderr
            ));
        }

        Ok(stdout.to_string())
    }

    /// 获取 Node.js 路径
    async fn get_node_path(&self) -> Result<std::path::PathBuf> {
        // 1. 优先使用嵌入式 Node.js
        let runtime_bundle = crate::installer::RuntimeBundle::new()?;
        let embedded_node = runtime_bundle.get_node_path().await;

        if let Ok(path) = embedded_node {
            if path.exists() {
                return Ok(path);
            }
        }

        // 2. 使用系统 Node.js
        match Command::new("node").arg("--version").output().await {
            Ok(output) if output.status.success() => Ok(std::path::PathBuf::from("node")),
            _ => Err(anyhow::anyhow!(
                "Node.js 未找到。请先安装 Node.js 22+ 或重新安装应用。"
            )),
        }
    }

    /// 获取版本
    pub async fn get_version(&self) -> Result<String> {
        self.installer.get_installed_version().await
    }

    /// 处理 stdout 日志
    async fn log_stdout(mut lines: tokio::io::Lines<BufReader<tokio::process::ChildStdout>>) {
        while let Ok(Some(line)) = lines.next_line().await {
            log::info!("[OpenClaw] {}", line);
        }
    }

    /// 处理 stderr 日志
    async fn log_stderr(mut lines: tokio::io::Lines<BufReader<tokio::process::ChildStderr>>) {
        while let Ok(Some(line)) = lines.next_line().await {
            log::warn!("[OpenClaw] {}", line);
        }
    }
}

impl Clone for SidecarLauncher {
    fn clone(&self) -> Self {
        Self {
            installer: SidecarInstaller::new().expect("Failed to create SidecarInstaller"),
            process: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(SidecarState::Stopped)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidecar_launcher_new() {
        let launcher = SidecarLauncher::new();
        assert!(launcher.is_ok());
    }
}
