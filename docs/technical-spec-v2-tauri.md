# OpenClaw Manager 技术方案 V2（基于 Tauri）

## 一、技术选型对比与决策

### 1.1 为什么选择 Tauri

| 维度 | Electron | Tauri | 说明 |
|------|----------|-------|------|
| **包体积** | 150-300MB | 5-15MB | Tauri 使用系统 WebView |
| **内存占用** | 150-300MB | 50-100MB | Rust 后端更高效 |
| **启动速度** | 3-5s | 1-2s | 冷启动优势明显 |
| **安全性** | 一般 | 强 | Rust 内存安全，默认最小权限 |
| **前端技术** | 任意 | 任意 | 都支持 React/Vue 等 |
| **后端语言** | Node.js | Rust | Rust 学习成本略高 |
| **生态成熟度** | 非常成熟 | 快速发展中 | Electron 生态更丰富 |
| **原生能力** | 丰富 | 足够 | Tauri API 覆盖主要需求 |

**决策结论**：选择 Tauri，因为：
1. 包体积对用户友好（尤其是一键安装场景）
2. 更好的性能和资源占用
3. 更强的安全模型
4. 现代化架构，长期使用价值更高

### 1.2 技术栈总览

```
┌─────────────────────────────────────────────────────────────┐
│                        前端层                                 │
│    React 18 + TypeScript + Tailwind CSS + shadcn/ui        │
│    状态管理: Zustand / Jotai                                 │
│    数据获取: TanStack Query (React Query)                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ Tauri Invoke / Events
                              │
┌─────────────────────────────────────────────────────────────┐
│                      Tauri 运行时层                          │
│    ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐   │
│    │ 命令处理器   │  │ 事件总线    │  │  窗口管理       │   │
│    │ (Commands)  │  │ (Events)    │  │                 │   │
│    └─────────────┘  └─────────────┘  └─────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ Rust 模块调用
                              │
┌─────────────────────────────────────────────────────────────┐
│                     Rust 核心业务层                          │
│    ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐ │
│    │ 配置管理  │ │ 进程管理  │ │ 安装管理  │ │ 系统适配器    │ │
│    │ config   │ │ process  │ │ install  │ │ platform     │ │
│    └──────────┘ └──────────┘ └──────────┘ └──────────────┘ │
│    ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐ │
│    │ 命令执行  │ │ 文件监控  │ │ 安全存储  │ │ 日志管理      │ │
│    │ command  │ │ watcher  │ │ secure   │ │ logging      │ │
│    └──────────┘ └──────────┘ └──────────┘ └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
                              │ 系统调用
                              │
┌─────────────────────────────────────────────────────────────┐
│                       系统层                                 │
│    OpenClaw CLI    配置文件    系统命令    密钥链           │
└─────────────────────────────────────────────────────────────┘
```

---

## 二、核心模块详细设计

### 2.1 项目结构

```
openclaw-manager/
├── src/                          # 前端源码
│   ├── components/               # 通用组件
│   │   ├── ui/                   # 基础 UI (shadcn)
│   │   ├── layout/               # 布局组件
│   │   └── forms/                # 表单组件
│   ├── pages/                    # 页面组件
│   │   ├── Dashboard/            # 仪表盘
│   │   ├── InstallWizard/        # 安装向导
│   │   ├── ModelConfig/          # 模型配置
│   │   ├── AgentManager/         # Agent 管理
│   │   ├── SkillStore/           # 技能商店
│   │   ├── LogsViewer/           # 日志查看
│   │   └── Settings/             # 设置
│   ├── hooks/                    # 自定义 Hooks
│   ├── stores/                   # Zustand 状态管理
│   ├── lib/                      # 工具函数
│   ├── types/                    # TypeScript 类型
│   └── App.tsx                   # 应用入口
├── src-tauri/                    # Rust 后端源码
│   ├── src/
│   │   ├── main.rs               # 入口
│   │   ├── lib.rs                # 库入口
│   │   ├── commands/             # Tauri Commands
│   │   │   ├── mod.rs
│   │   │   ├── config.rs         # 配置相关命令
│   │   │   ├── install.rs        # 安装相关命令
│   │   │   ├── process.rs        # 进程相关命令
│   │   │   ├── model.rs          # 模型相关命令
│   │   │   └── agent.rs          # Agent 相关命令
│   │   ├── services/             # 业务服务层
│   │   │   ├── mod.rs
│   │   │   ├── config_manager.rs
│   │   │   ├── install_manager.rs
│   │   │   ├── process_manager.rs
│   │   │   ├── model_service.rs
│   │   │   ├── agent_service.rs
│   │   │   ├── log_watcher.rs
│   │   │   └── diagnostics.rs
│   │   ├── adapters/             # 适配器层
│   │   │   ├── mod.rs
│   │   │   ├── openclaw_cli.rs   # OpenClaw CLI 适配
│   │   │   └── system.rs         # 系统命令适配
│   │   ├── models/               # 数据模型
│   │   │   ├── mod.rs
│   │   │   ├── config.rs         # 配置结构
│   │   │   ├── agent.rs          # Agent 结构
│   │   │   └── types.rs          # 通用类型
│   │   └── utils/                # 工具模块
│   │       ├── mod.rs
│   │       ├── errors.rs         # 错误定义
│   │       └── helpers.rs        # 辅助函数
│   ├── Cargo.toml
│   └── tauri.conf.json
├── docs/                         # 文档
├── scripts/                      # 脚本
└── package.json
```

### 2.2 Rust 核心模块设计

#### 2.2.1 配置管理器 (ConfigManager)

```rust
// src-tauri/src/services/config_manager.rs
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use notify::{Watcher, RecursiveMode, watcher};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawConfig {
    pub version: u32,
    pub models: ModelsConfig,
    pub agents: Vec<AgentConfig>,
    pub skills: SkillsConfig,
    pub hooks: HooksConfig,
}

#[derive(Debug, Clone)]
pub struct ConfigState {
    pub config: OpenClawConfig,
    pub version: u64,           // 配置版本号，用于冲突检测
    pub last_modified: u64,     // 最后修改时间戳
    pub is_dirty: bool,         // 是否有未保存更改
}

pub struct ConfigManager {
    state: Arc<RwLock<ConfigState>>,
    config_path: PathBuf,
    watcher: Option<Box<dyn Watcher>>,
}

impl ConfigManager {
    pub async fn new() -> Result<Self, ConfigError> {
        let config_path = Self::get_config_path()?;
        let state = Self::load_config(&config_path).await?;

        let manager = Self {
            state: Arc::new(RwLock::new(state)),
            config_path,
            watcher: None,
        };

        manager.start_watching()?;
        Ok(manager)
    }

    // 读取配置（带缓存）
    pub async fn read(&self) -> Result<ConfigState, ConfigError> {
        let state = self.state.read().await;
        Ok(state.clone())
    }

    // 写入配置（带乐观锁）
    pub async fn write(&self, config: OpenClawConfig, expected_version: u64)
        -> Result<(), ConfigError>
    {
        let mut state = self.state.write().await;

        // 乐观锁检查
        if state.version != expected_version {
            return Err(ConfigError::ConcurrentModification);
        }

        // 验证配置
        self.validate(&config)?;

        // 写入文件
        let yaml = serde_yaml::to_string(&config)?;
        tokio::fs::write(&self.config_path, yaml).await?;

        // 更新状态
        state.config = config;
        state.version += 1;
        state.last_modified = current_timestamp();
        state.is_dirty = false;

        Ok(())
    }

    // 导出配置
    pub async fn export(&self, path: PathBuf) -> Result<(), ConfigError> {
        let state = self.state.read().await;
        let export_data = serde_json::to_string_pretty(&state.config)?;
        tokio::fs::write(path, export_data).await?;
        Ok(())
    }

    // 导入配置
    pub async fn import(&self, path: PathBuf) -> Result<OpenClawConfig, ConfigError> {
        let data = tokio::fs::read_to_string(path).await?;
        let config: OpenClawConfig = serde_json::from_str(&data)?;
        self.validate(&config)?;
        Ok(config)
    }

    fn validate(&self, config: &OpenClawConfig) -> Result<(), ConfigError> {
        // 配置验证逻辑
        if config.models.providers.is_empty() {
            return Err(ConfigError::Validation("至少需要配置一个模型提供商".to_string()));
        }
        Ok(())
    }
}
```

#### 2.2.2 进程管理器 (ProcessManager)

```rust
// src-tauri/src/services/process_manager.rs
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::{RwLock, broadcast};
use sysinfo::{ProcessExt, System, SystemExt, get_current_pid};

#[derive(Debug, Clone, Serialize)]
pub enum ServiceStatus {
    Starting,
    Running { pid: u32, started_at: u64 },
    Stopping,
    Stopped,
    Error(String),
    Crashed { exit_code: i32, message: String },
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub status: ServiceStatus,
    pub command: String,
    pub args: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub restart_count: u32,
    pub last_exit_code: Option<i32>,
}

pub struct ProcessManager {
    services: Arc<RwLock<HashMap<String, ServiceHandle>>>,
    event_sender: broadcast::Sender<ProcessEvent>,
    system: Arc<RwLock<System>>,
}

struct ServiceHandle {
    process: Child,
    info: ServiceInfo,
    shutdown_tx: tokio::sync::mpsc::Sender<()>,
}

impl ProcessManager {
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(100);
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            system: Arc::new(RwLock::new(System::new_all())),
        }
    }

    // 启动服务
    pub async fn start_service(
        &self,
        name: String,
        command: String,
        args: Vec<String>,
        env_vars: HashMap<String, String>,
    ) -> Result<ServiceStatus, ProcessError> {
        let mut services = self.services.write().await;

        // 检查是否已存在
        if services.contains_key(&name) {
            return Err(ProcessError::AlreadyRunning);
        }

        // 检查端口占用
        self.check_port_availability(&name).await?;

        // 构建命令
        let mut cmd = Command::new(&command);
        cmd.args(&args)
           .envs(&env_vars)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped())
           .kill_on_drop(true);

        // 启动进程
        let mut process = cmd.spawn()?;
        let pid = process.id().ok_or(ProcessError::StartFailed)?;

        // 创建关闭通道
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel(1);

        let info = ServiceInfo {
            name: name.clone(),
            status: ServiceStatus::Running { pid, started_at: current_timestamp() },
            command: command.clone(),
            args: args.clone(),
            env_vars: env_vars.clone(),
            restart_count: 0,
            last_exit_code: None,
        };

        let handle = ServiceHandle {
            process,
            info: info.clone(),
            shutdown_tx,
        };

        services.insert(name.clone(), handle);
        drop(services); // 释放锁

        // 启动监控任务
        self.spawn_monitor_task(name.clone(), shutdown_rx);

        // 启动日志收集
        self.spawn_log_collector(name.clone());

        self.event_sender.send(ProcessEvent::Started { name, pid }).ok();

        Ok(info.status)
    }

    // 停止服务（优雅关闭）
    pub async fn stop_service(&self, name: &str, timeout_secs: u64)
        -> Result<(), ProcessError>
    {
        let mut services = self.services.write().await;

        let handle = services.get_mut(name)
            .ok_or(ProcessError::NotFound)?;

        // 发送优雅关闭信号
        handle.info.status = ServiceStatus::Stopping;

        // 先尝试 SIGTERM (Windows: taskkill /T /F)
        #[cfg(unix)]
        {
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;

            if let Some(pid) = handle.process.id() {
                signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM).ok();
            }
        }

        #[cfg(windows)]
        {
            // Windows 使用 taskkill /T /PID <pid>
        }

        // 等待进程退出或超时
        let shutdown_tx = handle.shutdown_tx.clone();
        drop(services); // 释放锁

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(timeout_secs)) => {
                // 超时，强制终止
                let mut services = self.services.write().await;
                if let Some(handle) = services.get_mut(name) {
                    handle.process.kill().await.ok();
                }
            }
            _ = shutdown_tx.closed() => {
                // 进程已退出
            }
        }

        services.remove(name);
        self.event_sender.send(ProcessEvent::Stopped { name: name.to_string() }).ok();

        Ok(())
    }

    // 健康检查
    pub async fn health_check(&self, name: &str) -> Result<HealthStatus, ProcessError> {
        let services = self.services.read().await;
        let handle = services.get(name).ok_or(ProcessError::NotFound)?;

        // 检查进程是否存活
        let mut system = self.system.write().await;
        system.refresh_processes();

        let is_alive = handle.process.id()
            .map(|pid| system.process(sysinfo::Pid::from(pid as usize)).is_some())
            .unwrap_or(false);

        if !is_alive {
            return Ok(HealthStatus::Dead);
        }

        // 检查端口响应（如果配置了健康检查端口）
        // TODO: 实现 HTTP 健康检查

        Ok(HealthStatus::Healthy)
    }

    // 获取服务状态
    pub async fn get_status(&self, name: &str) -> Result<ServiceInfo, ProcessError> {
        let services = self.services.read().await;
        services.get(name)
            .map(|h| h.info.clone())
            .ok_or(ProcessError::NotFound)
    }

    // 获取所有服务状态
    pub async fn list_services(&self) -> Vec<ServiceInfo> {
        let services = self.services.read().await;
        services.values().map(|h| h.info.clone()).collect()
    }

    // 订阅进程事件
    pub fn subscribe_events(&self) -> broadcast::Receiver<ProcessEvent> {
        self.event_sender.subscribe()
    }
}
```

#### 2.2.3 安装管理器 (InstallManager)

```rust
// src-tauri/src/services/install_manager.rs
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use reqwest;

#[derive(Debug, Clone, Serialize)]
pub struct InstallProgress {
    pub stage: InstallStage,
    pub progress: f32,           // 0-100
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum InstallStage {
    Detecting,       // 检测环境
    Downloading,     // 下载
    Installing,      // 安装
    Configuring,     // 配置
    Verifying,       // 验证
    Complete,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct SystemRequirement {
    pub item: String,
    pub required_version: String,
    pub installed_version: Option<String>,
    pub is_installed: bool,
    pub is_compatible: bool,
}

pub struct InstallManager {
    progress_sender: mpsc::Sender<InstallProgress>,
    state: Arc<RwLock<InstallState>>,
    http_client: reqwest::Client,
}

#[derive(Debug, Default)]
struct InstallState {
    is_installing: bool,
    checkpoints: Vec<InstallCheckpoint>,
}

#[derive(Debug, Clone)]
struct InstallCheckpoint {
    stage: String,
    timestamp: u64,
    can_rollback: bool,
}

impl InstallManager {
    pub fn new(progress_sender: mpsc::Sender<InstallProgress>) -> Self {
        Self {
            progress_sender,
            state: Arc::new(RwLock::new(InstallState::default())),
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(300))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    // 检测系统环境
    pub async fn check_prerequisites(&self) -> Result<Vec<SystemRequirement>, InstallError> {
        let mut requirements = vec![];

        // 检查 Node.js
        requirements.push(self.check_nodejs().await?);

        // 检查 Git
        requirements.push(self.check_git().await?);

        // 检查 Python (某些技能需要)
        requirements.push(self.check_python().await?);

        // 检查端口占用
        requirements.push(self.check_ports().await?);

        // 检查网络环境
        requirements.push(self.check_network().await?);

        Ok(requirements)
    }

    // 检测网络环境（中国大陆优化）
    async fn check_network(&self) -> Result<SystemRequirement, InstallError> {
        let github_accessible = self.test_url("https://github.com").await;
        let ghproxy_accessible = self.test_url("https://ghproxy.com").await;

        let network_type = if github_accessible {
            NetworkType::Global
        } else if ghproxy_accessible {
            NetworkType::ChinaWithProxy
        } else {
            NetworkType::Offline
        };

        // 保存到状态，后续下载使用
        let mut state = self.state.write().await;
        state.network_type = network_type;

        Ok(SystemRequirement {
            item: "网络环境".to_string(),
            required_version: "可访问 GitHub".to_string(),
            installed_version: Some(format!("{:?}", network_type)),
            is_installed: github_accessible || ghproxy_accessible,
            is_compatible: github_accessible,
        })
    }

    // 自动修复依赖问题
    pub async fn auto_fix(&self, requirements: &[SystemRequirement])
        -> Result<Vec<FixResult>, InstallError>
    {
        let mut results = vec![];

        for req in requirements {
            if req.is_compatible {
                continue;
            }

            let result = match req.item.as_str() {
                "Node.js" => self.install_nodejs().await,
                "Git" => self.install_git().await,
                "端口占用" => self.resolve_port_conflict().await,
                _ => Ok(FixResult::NotSupported),
            };

            results.push(result?);
        }

        Ok(results)
    }

    // 执行安装（带事务和回滚）
    pub async fn install(&self, options: InstallOptions) -> Result<(), InstallError> {
        let mut state = self.state.write().await;
        if state.is_installing {
            return Err(InstallError::AlreadyInstalling);
        }
        state.is_installing = true;
        state.checkpoints.clear();
        drop(state);

        self.report_progress(InstallStage::Detecting, 0.0, "检测系统环境...", None).await;

        // 第1步：环境检测
        let requirements = self.check_prerequisites().await?;
        let all_ready = requirements.iter().all(|r| r.is_compatible);

        if !all_ready && options.auto_fix {
            self.report_progress(InstallStage::Detecting, 10.0, "自动修复依赖...", None).await;
            self.auto_fix(&requirements).await?;
        }

        self.create_checkpoint("prerequisites_check").await;

        // 第2步：下载
        self.report_progress(InstallStage::Downloading, 20.0, "下载 OpenClaw...", None).await;
        let download_url = self.get_download_url().await;
        let temp_path = self.download_with_progress(&download_url, |downloaded, total| {
            let pct = 20.0 + (downloaded as f32 / total as f32) * 30.0;
            self.report_progress(
                InstallStage::Downloading,
                pct,
                &format!("已下载 {}/{} MB", downloaded / 1024 / 1024, total / 1024 / 1024),
                None
            );
        }).await?;

        self.create_checkpoint("download_complete").await;

        // 第3步：安装
        self.report_progress(InstallStage::Installing, 50.0, "安装 OpenClaw...", None).await;
        self.run_install_script(&temp_path).await?;

        self.create_checkpoint("installation_complete").await;

        // 第4步：验证
        self.report_progress(InstallStage::Verifying, 90.0, "验证安装...", None).await;
        self.verify_installation().await?;

        self.report_progress(InstallStage::Complete, 100.0, "安装完成！", None).await;

        let mut state = self.state.write().await;
        state.is_installing = false;

        Ok(())
    }

    // 回滚到指定检查点
    pub async fn rollback(&self, checkpoint: &str) -> Result<(), InstallError> {
        self.report_progress(
            InstallStage::Failed("安装失败，正在回滚...".to_string()),
            0.0,
            "执行回滚...",
            None
        ).await;

        match checkpoint {
            "installation_complete" => {
                // 卸载 OpenClaw
                self.uninstall().await?;
            }
            "download_complete" => {
                // 清理下载文件
                self.cleanup_download().await?;
            }
            _ => {}
        }

        Ok(())
    }

    // 获取推荐镜像源
    fn get_download_url(&self) -> String {
        // 根据网络环境选择最佳镜像
        let mirrors = [
            ("https://github.com/claw/openclaw/releases/latest/download/", NetworkType::Global),
            ("https://ghproxy.com/https://github.com/claw/openclaw/releases/latest/download/", NetworkType::ChinaWithProxy),
            ("https://gitee.com/mirrors/openclaw/releases/download/", NetworkType::China),
        ];

        // 实现镜像选择逻辑...
        mirrors[0].0.to_string()
    }

    async fn report_progress(&self, stage: InstallStage, progress: f32, message: &str, details: Option<&str>) {
        let _ = self.progress_sender.send(InstallProgress {
            stage,
            progress,
            message: message.to_string(),
            details: details.map(|s| s.to_string()),
        }).await;
    }
}
```

#### 2.2.4 命令执行器 (OpenClawCli)

```rust
// src-tauri/src/adapters/openclaw_cli.rs
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use serde::de::DeserializeOwned;

pub struct OpenClawCli {
    binary_path: PathBuf,
    timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl OpenClawCli {
    pub fn new() -> Result<Self, CliError> {
        let binary_path = Self::find_binary()?;
        Ok(Self {
            binary_path,
            timeout_secs: 30,
        })
    }

    // 查找 OpenClaw 可执行文件
    fn find_binary() -> Result<PathBuf, CliError> {
        // 1. 检查环境变量 OPENCLAW_PATH
        if let Ok(path) = std::env::var("OPENCLAW_PATH") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }

        // 2. 检查 PATH
        if let Ok(output) = Command::new("which").arg("openclaw").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Ok(PathBuf::from(path));
            }
        }

        // 3. 检查默认安装路径
        let default_paths = Self::get_default_paths();
        for path in default_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err(CliError::NotFound)
    }

    // 执行命令（带超时和错误处理）
    pub async fn run<T: DeserializeOwned>(
        &self,
        command: &str,
        args: &[&str],
        json_output: bool,
    ) -> Result<CliResult<T>, CliError> {
        let mut cmd_args = vec![command];
        cmd_args.extend_from_slice(args);

        if json_output {
            cmd_args.push("--output");
            cmd_args.push("json");
        }

        let mut command = Command::new(&self.binary_path);
        command.args(&cmd_args)
               .stdout(Stdio::piped())
               .stderr(Stdio::piped());

        // 设置环境变量
        command.env("OPENCLAW_NONINTERACTIVE", "1");

        let result = timeout(
            Duration::from_secs(self.timeout_secs),
            command.output()
        ).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);
                let success = output.status.success();

                let data = if json_output && success {
                    serde_json::from_str(&stdout).ok()
                } else {
                    None
                };

                Ok(CliResult {
                    success,
                    data,
                    error: if success { None } else { Some(stderr.clone()) },
                    stdout,
                    stderr,
                    exit_code,
                })
            }
            Ok(Err(e)) => Err(CliError::ExecutionFailed(e.to_string())),
            Err(_) => Err(CliError::Timeout),
        }
    }

    // 模型相关命令
    pub async fn models_list(&self) -> Result<Vec<ModelInfo>, CliError> {
        let result = self.run::<serde_json::Value>("models", &["list"], true).await?;
        if result.success {
            Ok(serde_json::from_value(result.data.unwrap())?)
        } else {
            Err(CliError::CommandFailed(result.error.unwrap_or_default()))
        }
    }

    pub async fn models_set_default(&self, model_id: &str) -> Result<(), CliError> {
        let result = self.run::<serde_json::Value>(
            "models",
            &["set", model_id],
            true
        ).await?;

        if result.success {
            Ok(())
        } else {
            Err(CliError::CommandFailed(result.error.unwrap_or_default()))
        }
    }

    pub async fn models_add_fallback(&self, model_id: &str) -> Result<(), CliError> {
        let result = self.run::<serde_json::Value>(
            "models",
            &["fallbacks", "add", model_id],
            true
        ).await?;

        if result.success {
            Ok(())
        } else {
            Err(CliError::CommandFailed(result.error.unwrap_or_default()))
        }
    }

    // 网关控制命令
    pub async fn gateway_start(&self) -> Result<(), CliError> {
        self.run::<serde_json::Value>("gateway", &["start"], false).await?;
        Ok(())
    }

    pub async fn gateway_stop(&self) -> Result<(), CliError> {
        self.run::<serde_json::Value>("gateway", &["stop"], false).await?;
        Ok(())
    }

    pub async fn gateway_restart(&self) -> Result<(), CliError> {
        self.run::<serde_json::Value>("gateway", &["restart"], false).await?;
        Ok(())
    }

    pub async fn gateway_status(&self) -> Result<GatewayStatus, CliError> {
        let result = self.run::<GatewayStatus>("gateway", &["status"], true).await?;
        if result.success {
            Ok(result.data.unwrap())
        } else {
            Err(CliError::CommandFailed(result.error.unwrap_or_default()))
        }
    }

    // Agent 管理
    pub async fn agent_list(&self) -> Result<Vec<AgentSummary>, CliError> {
        let result = self.run::<Vec<AgentSummary>>("agent", &["list"], true).await?;
        if result.success {
            Ok(result.data.unwrap())
        } else {
            Err(CliError::CommandFailed(result.error.unwrap_or_default()))
        }
    }

    pub async fn agent_create(&self, name: &str, config: &AgentConfig) -> Result<(), CliError> {
        let config_json = serde_json::to_string(config)?;
        let result = self.run::<serde_json::Value>(
            "agent",
            &["create", name, "--config", &config_json],
            true
        ).await?;

        if result.success {
            Ok(())
        } else {
            Err(CliError::CommandFailed(result.error.unwrap_or_default()))
        }
    }

    pub async fn agent_switch(&self, name: &str) -> Result<(), CliError> {
        let result = self.run::<serde_json::Value>("agent", &["use", name], true).await?;
        if result.success {
            Ok(())
        } else {
            Err(CliError::CommandFailed(result.error.unwrap_or_default()))
        }
    }

    // Skill 管理
    pub async fn skill_list(&self) -> Result<Vec<SkillInfo>, CliError> {
        let result = self.run::<Vec<SkillInfo>>("skill", &["list"], true).await?;
        if result.success {
            Ok(result.data.unwrap())
        } else {
            Err(CliError::CommandFailed(result.error.unwrap_or_default()))
        }
    }

    pub async fn skill_install(&self, skill_name: &str) -> Result<(), CliError> {
        let result = self.run::<serde_json::Value>(
            "skill",
            &["install", skill_name],
            true
        ).await?;

        if result.success {
            Ok(())
        } else {
            Err(CliError::CommandFailed(result.error.unwrap_or_default()))
        }
    }

    pub async fn skill_uninstall(&self, skill_name: &str) -> Result<(), CliError> {
        let result = self.run::<serde_json::Value>(
            "skill",
            &["uninstall", skill_name],
            true
        ).await?;

        if result.success {
            Ok(())
        } else {
            Err(CliError::CommandFailed(result.error.unwrap_or_default()))
        }
    }

    // 版本信息
    pub async fn version(&self) -> Result<String, CliError> {
        let result = self.run::<serde_json::Value>("--version", &[], false).await?;
        Ok(result.stdout.trim().to_string())
    }

    // 检查是否已安装
    pub async fn is_installed(&self) -> bool {
        self.version().await.is_ok()
    }
}
```

#### 2.2.5 日志监控器 (LogWatcher)

```rust
// src-tauri/src/services/log_watcher.rs
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::broadcast;
use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use std::sync::mpsc::channel;
use std::time::Duration;
use regex::Regex;

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: Option<String>,
    pub level: LogLevel,
    pub source: String,
    pub message: String,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
    Unknown,
}

pub struct LogWatcher {
    log_dir: PathBuf,
    event_sender: broadcast::Sender<LogEntry>,
    parsers: Vec<Box<dyn LogParser>>,
}

trait LogParser: Send + Sync {
    fn parse(&self, line: &str) -> Option<LogEntry>;
}

struct DefaultParser;
impl LogParser for DefaultParser {
    fn parse(&self, line: &str) -> Option<LogEntry> {
        // 匹配常见日志格式：2024-01-15 10:30:45 [INFO] module: message
        let re = Regex::new(
            r"^(\d{4}-\d{2}-\d{2}[\sT]\d{2}:\d{2}:\d{2})?\s*\[(\w+)\]\s*(\w+):?\s*(.+)$"
        ).ok()?;

        re.captures(line).map(|caps| {
            LogEntry {
                timestamp: caps.get(1).map(|m| m.as_str().to_string()),
                level: parse_level(caps.get(2).map(|m| m.as_str()).unwrap_or("INFO")),
                source: caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default(),
                message: caps.get(4).map(|m| m.as_str().to_string()).unwrap_or_default(),
                raw: line.to_string(),
            }
        })
    }
}

impl LogWatcher {
    pub fn new(log_dir: PathBuf) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        let parsers: Vec<Box<dyn LogParser>> = vec![
            Box::new(DefaultParser),
        ];

        Self {
            log_dir,
            event_sender,
            parsers,
        }
    }

    // 开始监控日志文件
    pub async fn watch(&self, log_file: &str) -> Result<(), LogError> {
        let path = self.log_dir.join(log_file);

        // 文件监控（检测新内容）
        let (tx, rx) = channel::<DebouncedEvent>();
        let mut watcher = watcher(tx, Duration::from_secs(1))?;
        watcher.watch(&path, RecursiveMode::NonRecursive)?;

        // 初始读取已有内容（最后 N 行）
        self.tail(&path, 100).await?;

        // 事件循环
        tokio::spawn({
            let sender = self.event_sender.clone();
            let parsers = self.parsers.clone();
            let path = path.clone();

            async move {
                let file = File::open(&path).await.expect("Failed to open log file");
                let reader = BufReader::new(file);
                let mut lines = reader.lines();

                // 跳到文件末尾
                // ...

                loop {
                    match lines.next_line().await {
                        Ok(Some(line)) => {
                            // 尝试所有解析器
                            for parser in &parsers {
                                if let Some(entry) = parser.parse(&line) {
                                    let _ = sender.send(entry);
                                    break;
                                }
                            }
                        }
                        Ok(None) => {
                            // 等待新内容
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                        Err(e) => {
                            eprintln!("Error reading log: {}", e);
                            break;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    // 读取最后 N 行
    async fn tail(&self, path: &PathBuf, n: usize) -> Result<Vec<LogEntry>, LogError> {
        // 使用 rev 读取文件最后 N 行
        // 实现略...
        Ok(vec![])
    }

    // 订阅日志事件
    pub fn subscribe(&self) -> broadcast::Receiver<LogEntry> {
        self.event_sender.subscribe()
    }

    // 按级别过滤历史日志
    pub async fn filter_logs(
        &self,
        log_file: &str,
        level: LogLevel,
        since: Option<u64>,
        limit: usize,
    ) -> Result<Vec<LogEntry>, LogError> {
        let path = self.log_dir.join(log_file);
        let file = File::open(&path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut results = vec![];

        while let Some(line) = lines.next_line().await? {
            for parser in &self.parsers {
                if let Some(entry) = parser.parse(&line) {
                    if matches_level(&entry.level, &level) {
                        results.push(entry);
                        if results.len() >= limit {
                            return Ok(results);
                        }
                    }
                    break;
                }
            }
        }

        Ok(results)
    }
}
```

#### 2.2.6 诊断引擎 (Diagnostics)

```rust
// src-tauri/src/services/diagnostics.rs
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticResult {
    pub category: DiagnosticCategory,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub details: Option<HashMap<String, String>>,
    pub auto_fixable: bool,
    pub fix_action: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum DiagnosticCategory {
    Network,
    Configuration,
    Service,
    Dependency,
    Permission,
    Performance,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

pub struct DiagnosticsEngine {
    cli: OpenClawCli,
    config_manager: ConfigManager,
    process_manager: ProcessManager,
}

impl DiagnosticsEngine {
    pub fn new(
        cli: OpenClawCli,
        config_manager: ConfigManager,
        process_manager: ProcessManager,
    ) -> Self {
        Self {
            cli,
            config_manager,
            process_manager,
        }
    }

    // 运行全面诊断
    pub async fn run_full_diagnostics(&self) -> Vec<DiagnosticResult> {
        let mut results = vec![];

        // 1. 网络诊断
        results.extend(self.check_network().await);

        // 2. 配置诊断
        results.extend(self.check_configuration().await);

        // 3. 服务诊断
        results.extend(self.check_services().await);

        // 4. 依赖诊断
        results.extend(self.check_dependencies().await);

        // 5. 权限诊断
        results.extend(self.check_permissions().await);

        // 6. 性能诊断
        results.extend(self.check_performance().await);

        results
    }

    async fn check_network(&self) -> Vec<DiagnosticResult> {
        let mut results = vec![];

        // 检查 GitHub 访问
        let github_ok = self.test_connection("github.com", 443).await;
        results.push(DiagnosticResult {
            category: DiagnosticCategory::Network,
            severity: if github_ok { Severity::Info } else { Severity::Warning },
            title: "GitHub 连接".to_string(),
            description: if github_ok {
                "可以正常访问 GitHub".to_string()
            } else {
                "无法访问 GitHub，可能影响技能安装".to_string()
            },
            details: None,
            auto_fixable: false,
            fix_action: None,
        });

        // 检查 API 端点
        let config = self.config_manager.read().await.ok();
        if let Some(config) = config {
            for provider in &config.config.models.providers {
                let api_ok = self.test_url(&provider.base_url).await;
                results.push(DiagnosticResult {
                    category: DiagnosticCategory::Network,
                    severity: if api_ok { Severity::Info } else { Severity::Error },
                    title: format!("{} API 连接", provider.name),
                    description: if api_ok {
                        format!("{} API 可正常访问", provider.name)
                    } else {
                        format!("无法连接 {} API，请检查网络或配置", provider.name)
                    },
                    details: Some({
                        let mut map = HashMap::new();
                        map.insert("url".to_string(), provider.base_url.clone());
                        map
                    }),
                    auto_fixable: false,
                    fix_action: None,
                });
            }
        }

        results
    }

    async fn check_configuration(&self) -> Vec<DiagnosticResult> {
        let mut results = vec![];

        let config_result = self.config_manager.read().await;

        match config_result {
            Ok(state) => {
                // 检查默认模型
                if state.config.models.default.is_none() {
                    results.push(DiagnosticResult {
                        category: DiagnosticCategory::Configuration,
                        severity: Severity::Error,
                        title: "未设置默认模型".to_string(),
                        description: "必须配置一个默认模型才能正常使用".to_string(),
                        details: None,
                        auto_fixable: true,
                        fix_action: Some("open_model_config".to_string()),
                    });
                }

                // 检查备用模型
                if state.config.models.fallbacks.is_empty() {
                    results.push(DiagnosticResult {
                        category: DiagnosticCategory::Configuration,
                        severity: Severity::Warning,
                        title: "未配置备用模型".to_string(),
                        description: "建议配置备用模型以提高可靠性".to_string(),
                        details: None,
                        auto_fixable: true,
                        fix_action: Some("open_model_config".to_string()),
                    });
                }

                // 检查 API Key
                for provider in &state.config.models.providers {
                    if provider.api_key.is_empty() {
                        results.push(DiagnosticResult {
                            category: DiagnosticCategory::Configuration,
                            severity: Severity::Error,
                            title: format!("{} API Key 未配置", provider.name),
                            description: format!("请在模型配置中添加 {} 的 API Key", provider.name),
                            details: None,
                            auto_fixable: true,
                            fix_action: Some("open_model_config".to_string()),
                        });
                    }
                }
            }
            Err(e) => {
                results.push(DiagnosticResult {
                    category: DiagnosticCategory::Configuration,
                    severity: Severity::Critical,
                    title: "配置文件读取失败".to_string(),
                    description: format!("无法读取配置文件: {}", e),
                    details: None,
                    auto_fixable: false,
                    fix_action: None,
                });
            }
        }

        results
    }

    async fn check_services(&self) -> Vec<DiagnosticResult> {
        let mut results = vec![];

        // 检查网关状态
        match self.process_manager.get_status("gateway").await {
            Ok(info) => {
                let severity = match info.status {
                    ServiceStatus::Running { .. } => Severity::Info,
                    ServiceStatus::Stopped => Severity::Warning,
                    ServiceStatus::Error(_) | ServiceStatus::Crashed { .. } => Severity::Error,
                    _ => Severity::Info,
                };

                results.push(DiagnosticResult {
                    category: DiagnosticCategory::Service,
                    severity,
                    title: "网关服务状态".to_string(),
                    description: format!("当前状态: {:?}", info.status),
                    details: Some({
                        let mut map = HashMap::new();
                        map.insert("restart_count".to_string(), info.restart_count.to_string());
                        map
                    }),
                    auto_fixable: info.status != ServiceStatus::Running { pid: 0, started_at: 0 },
                    fix_action: Some("restart_gateway".to_string()),
                });
            }
            Err(_) => {
                results.push(DiagnosticResult {
                    category: DiagnosticCategory::Service,
                    severity: Severity::Warning,
                    title: "网关未运行".to_string(),
                    description: "OpenClaw 网关服务当前未启动".to_string(),
                    details: None,
                    auto_fixable: true,
                    fix_action: Some("start_gateway".to_string()),
                });
            }
        }

        results
    }

    // 自动修复问题
    pub async fn auto_fix(&self, result: &DiagnosticResult) -> Result<FixResult, DiagnosticsError> {
        if !result.auto_fixable || result.fix_action.is_none() {
            return Err(DiagnosticsError::NotFixable);
        }

        let action = result.fix_action.as_ref().unwrap();

        match action.as_str() {
            "start_gateway" => {
                self.process_manager.start_service(
                    "gateway".to_string(),
                    "openclaw".to_string(),
                    vec!["gateway".to_string(), "start".to_string()],
                    HashMap::new(),
                ).await?;
                Ok(FixResult::Success)
            }
            "restart_gateway" => {
                self.cli.gateway_restart().await?;
                Ok(FixResult::Success)
            }
            "open_model_config" => {
                // 返回需要前端处理的指令
                Ok(FixResult::RequiresUserAction("请打开模型配置页面".to_string()))
            }
            _ => Err(DiagnosticsError::UnknownAction(action.clone())),
        }
    }
}
```

---

## 三、前端架构设计

### 3.1 技术栈

- **框架**: React 18 + TypeScript
- **构建工具**: Vite
- **状态管理**: Zustand（轻量、TypeScript 友好）
- **数据获取**: TanStack Query (React Query) v5
- **UI 组件库**: shadcn/ui（基于 Radix UI + Tailwind）
- **路由**: TanStack Router
- **表单处理**: React Hook Form + Zod
- **图标**: Lucide React

### 3.2 状态管理设计

```typescript
// stores/appStore.ts
import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';

interface AppState {
  // 安装状态
  installState: {
    isInstalled: boolean;
    isInstalling: boolean;
    progress: InstallProgress | null;
    version: string | null;
  };

  // 服务状态
  serviceState: {
    gateway: ServiceStatus | null;
    lastChecked: number;
  };

  // 配置状态
  configState: {
    config: OpenClawConfig | null;
    version: number;
    isDirty: boolean;
    isLoading: boolean;
  };

  // 全局通知
  notifications: Notification[];

  // Actions
  setInstallState: (state: Partial<AppState['installState']>) => void;
  setServiceStatus: (status: ServiceStatus) => void;
  updateConfig: (config: OpenClawConfig, version: number) => void;
  markConfigDirty: (isDirty: boolean) => void;
  addNotification: (notification: Omit<Notification, 'id'>) => void;
  removeNotification: (id: string) => void;
}

export const useAppStore = create<AppState>()(
  subscribeWithSelector(
    immer((set) => ({
      installState: {
        isInstalled: false,
        isInstalling: false,
        progress: null,
        version: null,
      },
      serviceState: {
        gateway: null,
        lastChecked: 0,
      },
      configState: {
        config: null,
        version: 0,
        isDirty: false,
        isLoading: false,
      },
      notifications: [],

      setInstallState: (state) =>
        set((draft) => {
          Object.assign(draft.installState, state);
        }),

      setServiceStatus: (status) =>
        set((draft) => {
          draft.serviceState.gateway = status;
          draft.serviceState.lastChecked = Date.now();
        }),

      updateConfig: (config, version) =>
        set((draft) => {
          draft.configState.config = config;
          draft.configState.version = version;
          draft.configState.isDirty = false;
        }),

      markConfigDirty: (isDirty) =>
        set((draft) => {
          draft.configState.isDirty = isDirty;
        }),

      addNotification: (notification) =>
        set((draft) => {
          draft.notifications.push({
            ...notification,
            id: crypto.randomUUID(),
          });
        }),

      removeNotification: (id) =>
        set((draft) => {
          draft.notifications = draft.notifications.filter((n) => n.id !== id);
        }),
    }))
  )
);

// 持久化安装状态
import { persist } from 'zustand/middleware';

export const usePersistentStore = create(
  persist(
    (set) => ({
      hasCompletedOnboarding: false,
      setOnboardingComplete: () => set({ hasCompletedOnboarding: true }),
    }),
    {
      name: 'openclaw-manager-storage',
    }
  )
);
```

### 3.3 Tauri API 封装

```typescript
// lib/tauri.ts
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// 类型定义
export interface CliResult<T> {
  success: boolean;
  data?: T;
  error?: string;
  stdout: string;
  stderr: string;
  exit_code: number;
}

// Command 调用封装
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    // 统一错误处理
    console.error(`Command ${command} failed:`, error);
    throw new TauriError(error as string);
  }
}

// Event 监听封装
export function watchProcessEvents(
  callback: (event: ProcessEvent) => void
): () => void {
  const unlisten = listen<ProcessEvent>('process-event', (event) => {
    callback(event.payload);
  });

  return () => {
    unlisten.then((fn) => fn());
  };
}

// 具体业务 API
export const openclawApi = {
  // 安装相关
  checkInstallation: () =>
    invokeCommand<{ installed: boolean; version?: string }>('check_installation'),

  startInstall: (options: InstallOptions) =>
    invokeCommand<void>('start_install', { options }),

  subscribeInstallProgress: (callback: (progress: InstallProgress) => void) =>
    listen<InstallProgress>('install-progress', (e) => callback(e.payload)),

  // 配置相关
  getConfig: () =>
    invokeCommand<ConfigState>('get_config'),

  updateConfig: (config: OpenClawConfig, expectedVersion: number) =>
    invokeCommand<void>('update_config', { config, expectedVersion }),

  exportConfig: (path: string) =>
    invokeCommand<void>('export_config', { path }),

  importConfig: (path: string) =>
    invokeCommand<OpenClawConfig>('import_config', { path }),

  // 服务相关
  getServiceStatus: (name: string) =>
    invokeCommand<ServiceInfo>('get_service_status', { name }),

  startService: (name: string, command: string, args: string[]) =>
    invokeCommand<void>('start_service', { name, command, args }),

  stopService: (name: string, timeout?: number) =>
    invokeCommand<void>('stop_service', { name, timeout }),

  // 模型相关
  listModels: () =>
    invokeCommand<ModelInfo[]>('list_models'),

  setDefaultModel: (modelId: string) =>
    invokeCommand<void>('set_default_model', { modelId }),

  testModelConnection: (provider: string, apiKey: string) =>
    invokeConnectionResult>('test_model_connection', { provider, apiKey }),

  // Agent 相关
  listAgents: () =>
    invokeCommand<AgentSummary[]>('list_agents'),

  createAgent: (name: string, config: AgentConfig) =>
    invokeCommand<void>('create_agent', { name, config }),

  switchAgent: (name: string) =>
    invokeCommand<void>('switch_agent', { name }),

  // Skill 相关
  listSkills: () =>
    invokeCommand<SkillInfo[]>('list_skills'),

  installSkill: (name: string) =>
    invokeCommand<void>('install_skill', { name }),

  // 日志相关
  subscribeLogs: (callback: (entry: LogEntry) => void) =>
    listen<LogEntry>('log-entry', (e) => callback(e.payload)),

  getLogs: (options: LogFilterOptions) =>
    invokeCommand<LogEntry[]>('get_logs', { options }),

  // 诊断相关
  runDiagnostics: () =>
    invokeCommand<DiagnosticResult[]>('run_diagnostics'),

  autoFix: (result: DiagnosticResult) =>
    invokeCommand<FixResult>('auto_fix', { result }),
};
```

### 3.4 React Query Hooks

```typescript
// hooks/useOpenClaw.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { openclawApi } from '@/lib/tauri';

// 查询 Hooks
export function useInstallationStatus() {
  return useQuery({
    queryKey: ['installation'],
    queryFn: openclawApi.checkInstallation,
    refetchInterval: 30000, // 30秒检查一次
  });
}

export function useConfig() {
  return useQuery({
    queryKey: ['config'],
    queryFn: openclawApi.getConfig,
    staleTime: 0, // 总是获取最新配置
  });
}

export function useServiceStatus(name: string) {
  return useQuery({
    queryKey: ['service', name],
    queryFn: () => openclawApi.getServiceStatus(name),
    refetchInterval: 5000, // 5秒刷新
  });
}

export function useModels() {
  return useQuery({
    queryKey: ['models'],
    queryFn: openclawApi.listModels,
  });
}

export function useAgents() {
  return useQuery({
    queryKey: ['agents'],
    queryFn: openclawApi.listAgents,
  });
}

export function useSkills() {
  return useQuery({
    queryKey: ['skills'],
    queryFn: openclawApi.listSkills,
  });
}

// Mutation Hooks
export function useUpdateConfig() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ config, expectedVersion }: {
      config: OpenClawConfig;
      expectedVersion: number;
    }) => openclawApi.updateConfig(config, expectedVersion),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['config'] });
    },
  });
}

export function useSwitchAgent() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: openclawApi.switchAgent,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agents'] });
      queryClient.invalidateQueries({ queryKey: ['config'] });
    },
  });
}

export function useInstallSkill() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: openclawApi.installSkill,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['skills'] });
    },
  });
}

export function useTestModelConnection() {
  return useMutation({
    mutationFn: openclawApi.testModelConnection,
  });
}
```

---

## 四、UI 页面设计

### 4.1 安装向导流程

```
┌─────────────────────────────────────────────────────────┐
│  Step 1: 欢迎页                                          │
│  - 介绍 OpenClaw Manager                                 │
│  - 显示系统要求                                           │
│  - 开始安装按钮                                           │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│  Step 2: 环境检测                                         │
│  - 逐项检测依赖（Node.js, Git, Python, 端口）             │
│  - 显示检测结果（✓/✗）                                    │
│  - 自动修复按钮（对于可修复的问题）                        │
│  - 网络环境检测（自动选择镜像源）                          │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│  Step 3: 下载安装                                         │
│  - 显示下载进度（带速度显示）                              │
│  - 显示安装进度                                           │
│  - 支持暂停/继续（断点续传）                               │
│  - 失败重试机制                                           │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│  Step 4: 初始化配置                                       │
│  - 选择模型提供商（OpenAI/Claude/GLM等）                   │
│  - 输入 API Key（带验证）                                  │
│  - 测试连接                                               │
│  - 选择默认模型                                           │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│  Step 5: 完成                                             │
│  - 显示配置摘要                                           │
│  - 启动 OpenClaw 选项                                     │
│  - 进入主界面                                             │
└─────────────────────────────────────────────────────────┘
```

### 4.2 主界面布局

```
┌──────────────────────────────────────────────────────────────┐
│  Header: Logo | 当前 Agent | 网关状态 ● | 设置 | 关于          │
├──────────┬───────────────────────────────────────────────────┤
│          │                                                   │
│ Sidebar  │               Main Content Area                   │
│          │                                                   │
│ ┌──────┐ │   ┌─────────────────────────────────────────┐    │
│ │ 📊   │ │   │                                         │    │
│ │ 概览 │ │   │      根据路由显示不同页面内容            │    │
│ └──────┘ │   │                                         │    │
│          │   │  - Dashboard: 仪表盘                    │    │
│ ┌──────┐ │   │  - Models: 模型配置                     │    │
│ │ 🤖   │ │   │  - Agents: Agent 管理                   │    │
│ │ 模型 │ │   │  - Skills: 技能管理                     │    │
│ └──────┘ │   │  - Logs: 日志查看                       │    │
│          │   │  - Diagnose: 诊断修复                   │    │
│ ┌──────┐ │   │                                         │    │
│ │ 👤   │ │   └─────────────────────────────────────────┘    │
│ │ Agent│ │                                                   │
│ └──────┘ │                                                   │
│          │                                                   │
│ ┌──────┐ │                                                   │
│ │ 🧩   │ │                                                   │
│ │ 技能 │ │                                                   │
│ └──────┘ │                                                   │
│          │                                                   │
│ ┌──────┐ │                                                   │
│ │ 📄   │ │                                                   │
│ │ 日志 │ │                                                   │
│ └──────┘ │                                                   │
│          │                                                   │
│ ┌──────┐ │                                                   │
│ │ 🔧   │ │                                                   │
│ │ 诊断 │ │                                                   │
│ └──────┘ │                                                   │
│          │                                                   │
└──────────┴───────────────────────────────────────────────────┘
```

### 4.3 各页面功能清单

#### 仪表盘 (Dashboard)
- [ ] 系统状态卡片（网关运行状态、当前 Agent、默认模型）
- [ ] 快速操作按钮（启动/停止网关、重启服务）
- [ ] 资源使用情况（内存、CPU）
- [ ] 最近活动日志（最近 10 条）
- [ ] 诊断警告提示（如果有问题）

#### 模型配置 (Models)
- [ ] 提供商列表管理（添加/编辑/删除）
- [ ] API Key 输入（带安全显示/隐藏）
- [ ] 连接测试按钮（验证 API Key 有效性）
- [ ] 默认模型选择
- [ ] 备用模型排序管理（拖拽排序）
- [ ] 模型参数配置（温度、最大 token 等）

#### Agent 管理 (Agents)
- [ ] Agent 列表（卡片/列表视图）
- [ ] 创建 Agent 向导（选择模型、技能、Hook 配置）
- [ ] Agent 模板（预设配置）
- [ ] 快速切换 Agent
- [ ] Agent 配置导入/导出
- [ ] Agent 详情编辑

#### 技能商店 (Skills)
- [ ] 已安装技能列表（启用/禁用开关）
- [ ] 技能搜索/筛选
- [ ] 技能详情展示
- [ ] 一键安装/卸载
- [ ] 外部技能商店链接

#### 日志查看 (Logs)
- [ ] 实时日志流（自动滚动）
- [ ] 日志级别筛选（Error/Warning/Info/Debug）
- [ ] 关键字搜索
- [ ] 时间范围筛选
- [ ] 日志导出
- [ ] 高亮显示错误

#### 诊断修复 (Diagnose)
- [ ] 一键全面诊断
- [ ] 诊断结果分类展示
- [ ] 问题严重程度标识
- [ ] 一键自动修复
- [ ] 修复历史记录

---

## 五、关键流程时序图

### 5.1 安装流程

```
用户                前端                   Tauri                   Rust服务                系统
 │                   │                      │                        │                     │
 │──点击安装────────>│                      │                        │                     │
 │                   │──invoke:startInstall─────────────────────────>│                     │
 │                   │                      │                        │                     │
 │                   │                      │<───────────────────────│──检查依赖────────────>│
 │                   │                      │                        │<────────────────────│
 │                   │                      │                        │                     │
 │                   │<──emit:progress─────│<───────────────────────│                     │
 │<──显示进度────────│                      │                        │                     │
 │                   │                      │                        │                     │
 │                   │                      │                        │──下载文件───────────>│
 │                   │                      │                        │<────────────────────│
 │                   │<──emit:progress─────│<───────────────────────│                     │
 │<──更新进度────────│                      │                        │                     │
 │                   │                      │                        │                     │
 │                   │                      │                        │──执行安装脚本───────>│
 │                   │                      │                        │<────────────────────│
 │                   │                      │                        │                     │
 │                   │<──emit:complete─────│<───────────────────────│                     │
 │<──安装完成────────│                      │                        │                     │
```

### 5.2 配置更新流程

```
用户                前端                      Tauri                  ConfigManager           文件系统
 │                   │                         │                        │                     │
 │──修改配置────────>│                         │                        │                     │
 │                   │──invoke:updateConfig────────────────────────────>│                     │
 │                   │                         │                        │                     │
 │                   │                         │                        │──乐观锁检查          │
 │                   │                         │                        │                     │
 │                   │                         │                        │──验证配置────────────>│
 │                   │                         │                        │<────────────────────│
 │                   │                         │                        │                     │
 │                   │                         │                        │──写入文件────────────>│
 │                   │                         │                        │<────────────────────│
 │                   │                         │                        │                     │
 │                   │<──返回成功──────────────│<───────────────────────│                     │
 │<──显示保存成功────│                         │                        │                     │
 │                   │                         │                        │                     │
 │                   │                         │<──emit:configChanged───│──文件监听触发        │
 │                   │<──接收事件──────────────│                        │                     │
 │                   │──重新加载配置──────────>│                        │                     │
```

---

## 六、错误处理策略

### 6.1 错误分类

| 错误类型 | 示例 | 处理方式 |
|---------|------|---------|
| **用户错误** | 无效的 API Key、配置格式错误 | 友好的提示，引导修正 |
| **网络错误** | 下载失败、API 连接超时 | 自动重试，切换镜像源 |
| **系统错误** | 权限不足、端口占用 | 诊断提示，一键修复 |
| **内部错误** | Rust panic、未知异常 | 记录日志，上报错误 |

### 6.2 错误处理架构

```rust
// 统一错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("配置错误: {0}")]
    Config(#[from] ConfigError),

    #[error("进程错误: {0}")]
    Process(#[from] ProcessError),

    #[error("安装错误: {0}")]
    Install(#[from] InstallError),

    #[error("CLI 错误: {0}")]
    Cli(#[from] CliError),

    #[error("网络错误: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("未知错误: {0}")]
    Unknown(String),
}

// 转换为前端友好的错误信息
impl AppError {
    pub fn to_user_message(&self) -> ErrorMessage {
        match self {
            AppError::Config(e) => ErrorMessage {
                title: "配置错误".to_string(),
                description: e.to_string(),
                action: Some("检查配置文件".to_string()),
                severity: ErrorSeverity::Warning,
            },
            AppError::Network(e) => ErrorMessage {
                title: "网络连接失败".to_string(),
                description: "请检查网络连接或尝试切换镜像源".to_string(),
                action: Some("切换镜像源".to_string()),
                severity: ErrorSeverity::Warning,
            },
            _ => ErrorMessage {
                title: "操作失败".to_string(),
                description: self.to_string(),
                action: None,
                severity: ErrorSeverity::Error,
            },
        }
    }
}
```

---

## 七、安全设计

### 7.1 API Key 存储

```rust
// 使用系统密钥链存储
use keyring::Entry;

pub struct SecureStorage;

impl SecureStorage {
    const SERVICE_NAME: &str = "com.openclaw.manager";

    pub fn save_api_key(provider: &str, api_key: &str) -> Result<(), SecureStorageError> {
        let entry = Entry::new(Self::SERVICE_NAME, provider)?;
        entry.set_password(api_key)?;
        Ok(())
    }

    pub fn get_api_key(provider: &str) -> Result<Option<String>, SecureStorageError> {
        let entry = Entry::new(Self::SERVICE_NAME, provider)?;
        match entry.get_password() {
            Ok(key) => Ok(Some(key)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn delete_api_key(provider: &str) -> Result<(), SecureStorageError> {
        let entry = Entry::new(Self::SERVICE_NAME, provider)?;
        entry.delete_password()?;
        Ok(())
    }
}
```

### 7.2 命令执行安全

```rust
// 严格参数校验，防止命令注入
pub fn sanitize_argument(arg: &str) -> Result<String, ValidationError> {
    // 禁止特殊字符
    if arg.chars().any(|c| matches!(c, ';' | '|' | '&' | '$' | '`' | '>' | '<')) {
        return Err(ValidationError::InvalidCharacters);
    }
    Ok(arg.to_string())
}

// 使用参数列表而非字符串拼接
pub async fn safe_execute(
    program: &str,
    args: &[&str],
) -> Result<Output, ExecutionError> {
    // 验证程序路径
    if !is_allowed_program(program) {
        return Err(ExecutionError::ProgramNotAllowed);
    }

    // 验证所有参数
    let sanitized_args: Result<Vec<_>, _> = args
        .iter()
        .map(|a| sanitize_argument(a))
        .collect();

    let sanitized_args = sanitized_args?;

    // 使用 std::process::Command（非 shell）
    Ok(Command::new(program)
        .args(&sanitized_args)
        .output()
        .await?)
}
```

---

## 八、性能优化

### 8.1 启动优化
- 使用 Vite 进行代码分割
- 延迟加载非首屏组件
- Rust 后端并行初始化

### 8.2 运行时优化
- 配置变更防抖保存（500ms）
- 日志流式传输，前端虚拟滚动
- 服务状态轮询使用 SWR 策略

### 8.3 资源优化
- 使用系统 WebView（Tauri 默认）
- 静态资源懒加载
- 日志自动轮转，防止磁盘占满

---

## 九、测试策略

### 9.1 单元测试 (Rust)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_manager() {
        let manager = ConfigManager::new().await.unwrap();

        // 测试读写
        let config = OpenClawConfig::default();
        manager.write(config.clone(), 0).await.unwrap();

        let state = manager.read().await.unwrap();
        assert_eq!(state.version, 1);
    }
}
```

### 9.2 集成测试
- 安装流程端到端测试
- 配置更新冲突测试
- 进程管理异常测试

### 9.3 E2E 测试
- 使用 Playwright 进行 UI 测试
- 关键用户流程覆盖

---

## 十、部署与分发

### 10.1 打包配置

```json
// tauri.conf.json 关键配置
{
  "build": {
    "beforeBuildCommand": "pnpm build",
    "beforeDevCommand": "pnpm dev",
    "devPath": "http://localhost:5173",
    "distDir": "../dist"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      },
      "dialog": {
        "all": true
      },
      "fs": {
        "all": false,
        "readFile": true,
        "writeFile": true
      }
    },
    "bundle": {
      "active": true,
      "category": "DeveloperTool",
      "copyright": "",
      "deb": {
        "depends": []
      },
      "externalBin": [],
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ],
      "identifier": "com.openclaw.manager",
      "longDescription": "",
      "macOS": {
        "entitlements": null,
        "exceptionDomain": "",
        "frameworks": [],
        "providerShortName": null,
        "signingIdentity": null
      },
      "resources": [],
      "shortDescription": "",
      "targets": "all",
      "windows": {
        "certificateThumbprint": null,
        "digestAlgorithm": "sha256",
        "timestampUrl": ""
      }
    }
  }
}
```

### 10.2 CI/CD 流程

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        platform: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Setup Rust
        uses: dtolnay/rust-action@stable

      - name: Setup pnpm
        uses: pnpm/action-setup@v2
        with:
          version: 8

      - name: Install dependencies
        run: pnpm install

      - name: Build Tauri
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: v__VERSION__
          releaseName: 'v__VERSION__'
          releaseBody: 'See the assets to download this version.'
          releaseDraft: true
          prerelease: false
```

---

## 十一、跨平台兼容性设计

### 11.1 平台特定处理

#### Windows 平台

```rust
// src-tauri/src/platform/windows.rs
use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

pub struct WindowsPlatform;

impl PlatformAdapter for WindowsPlatform {
    // 检查 WebView2 运行时
    fn check_webview2(&self) -> Result<WebView2Status, PlatformError> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let webview2_key = hkcu.open_subkey(
            "Software\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
        );

        match webview2_key {
            Ok(key) => {
                let version: String = key.get_value("pv")?;
                Ok(WebView2Status::Installed(version))
            }
            Err(_) => Ok(WebView2Status::NotInstalled),
        }
    }

    // 安装 WebView2 运行时
    async fn install_webview2(&self) -> Result<(), PlatformError> {
        // 下载 Evergreen Bootstrapper（6MB）
        let url = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";
        let installer = download_file(url, "MicrosoftEdgeWebview2Setup.exe").await?;

        // 静默安装
        Command::new(&installer)
            .args(&["/silent", "/install"])
            .output()
            .await?;

        Ok(())
    }

    // 请求管理员权限
    fn request_elevation(&self, command: &str) -> Result<(), PlatformError> {
        use std::os::windows::process::CommandExt;

        Command::new("powershell")
            .args(&[
                "-Command",
                "Start-Process",
                command,
                "-Verb",
                "runAs"
            ])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()?;

        Ok(())
    }

    // 检查杀毒软件
    fn check_antivirus(&self) -> Vec<AntivirusInfo> {
        // 使用 WMI 查询已安装的杀毒软件
        // 返回可能影响安装的软件列表
        vec![]
    }
}
```

#### macOS 平台

```rust
// src-tauri/src/platform/macos.rs
pub struct MacOSPlatform;

impl PlatformAdapter for MacOSPlatform {
    // 检查是否已授予必要权限
    fn check_permissions(&self) -> Result<PermissionsStatus, PlatformError> {
        use std::process::Command;

        // 检查终端权限
        let output = Command::new("osascript")
            .args(&["-e", r#"
                tell application "System Events"
                    return UI elements enabled
                end tell
            "#])
            .output()?;

        let accessibility_enabled = String::from_utf8_lossy(&output.stdout)
            .trim() == "true";

        Ok(PermissionsStatus {
            accessibility: accessibility_enabled,
            full_disk_access: self.check_full_disk_access(),
        })
    }

    // 引导用户授权
    async fn request_permissions(&self) -> Result<(), PlatformError> {
        // 打开系统偏好设置的安全与隐私页面
        Command::new("open")
            .args(&["x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"])
            .spawn()?;

        // 显示引导对话框
        show_permission_guide().await;

        Ok(())
    }

    // 检查架构（Apple Silicon/Intel）
    fn get_architecture(&self) -> Architecture {
        use std::env::consts::ARCH;
        match ARCH {
            "aarch64" => Architecture::AppleSilicon,
            "x86_64" => Architecture::Intel,
            _ => Architecture::Unknown,
        }
    }
}
```

#### Linux 平台

```rust
// src-tauri/src/platform/linux.rs
pub struct LinuxPlatform;

impl PlatformAdapter for LinuxPlatform {
    // 检测发行版
    fn detect_distro(&self) -> Result<LinuxDistro, PlatformError> {
        if Path::new("/etc/os-release").exists() {
            let content = fs::read_to_string("/etc/os-release")?;

            if content.contains("Ubuntu") {
                return Ok(LinuxDistro::Ubuntu);
            } else if content.contains("Debian") {
                return Ok(LinuxDistro::Debian);
            } else if content.contains("Fedora") {
                return Ok(LinuxDistro::Fedora);
            } else if content.contains("Arch") {
                return Ok(LinuxDistro::Arch);
            }
        }

        Ok(LinuxDistro::Unknown)
    }

    // 获取安装命令
    fn get_install_command(&self, package: &str) -> Vec<String> {
        let distro = self.detect_distro().unwrap_or(LinuxDistro::Unknown);

        match distro {
            LinuxDistro::Ubuntu | LinuxDistro::Debian => vec![
                "sudo".to_string(),
                "apt-get".to_string(),
                "install".to_string(),
                "-y".to_string(),
                package.to_string(),
            ],
            LinuxDistro::Fedora => vec![
                "sudo".to_string(),
                "dnf".to_string(),
                "install".to_string(),
                "-y".to_string(),
                package.to_string(),
            ],
            LinuxDistro::Arch => vec![
                "sudo".to_string(),
                "pacman".to_string(),
                "-S".to_string(),
                "--noconfirm".to_string(),
                package.to_string(),
            ],
            _ => vec![],
        }
    }

    // 检查桌面环境
    fn detect_desktop_env(&self) -> DesktopEnvironment {
        if let Ok(de) = env::var("XDG_CURRENT_DESKTOP") {
            match de.as_str() {
                "GNOME" => DesktopEnvironment::Gnome,
                "KDE" => DesktopEnvironment::Kde,
                "X-Cinnamon" => DesktopEnvironment::Cinnamon,
                _ => DesktopEnvironment::Unknown,
            }
        } else {
            DesktopEnvironment::Unknown
        }
    }
}
```

### 11.2 降级方案设计

当 Tauri 应用无法正常工作时，提供备选方案：

```rust
// src-tauri/src/fallback/mod.rs

pub enum DeploymentMode {
    // 完整桌面应用（首选）
    FullDesktop,
    // Web 界面 + 本地服务
    WebUI,
    // Docker 部署
    Docker,
    // SSH 远程管理
    SshRemote,
    // 纯 CLI 模式
    CliOnly,
}

pub struct FallbackManager;

impl FallbackManager {
    // 检测最佳部署模式
    pub async fn detect_best_mode(&self) -> DeploymentMode {
        // 检查系统兼容性
        if !self.check_system_compatibility().await {
            // 检查 Docker 是否可用
            if self.check_docker_available().await {
                return DeploymentMode::Docker;
            }
            return DeploymentMode::CliOnly;
        }

        // 检查 WebView 可用性
        if !self.check_webview_available().await {
            // 尝试启动本地 Web 服务
            if self.can_start_web_service().await {
                return DeploymentMode::WebUI;
            }
        }

        DeploymentMode::FullDesktop
    }

    // 启动 Web 服务模式
    pub async fn start_web_mode(&self) -> Result<WebModeInfo, FallbackError> {
        // 启动本地 HTTP 服务
        let port = find_available_port(8080..9000).await?;

        // 嵌入静态文件
        start_embedded_server(port).await?;

        // 打开系统浏览器
        open_browser(format!("http://localhost:{}", port));

        Ok(WebModeInfo { port })
    }

    // Docker 部署
    pub async fn deploy_docker(&self) -> Result<DockerInfo, FallbackError> {
        let compose_yaml = r#"
version: '3.8'
services:
  openclaw:
    image: openclaw/openclaw:latest
    ports:
      - "8080:8080"
    volumes:
      - ~/.openclaw:/root/.openclaw
    environment:
      - OPENCLAW_API_KEY=${OPENCLAW_API_KEY}
"#;

        // 写入 docker-compose.yml
        fs::write("docker-compose.yml", compose_yaml).await?;

        // 执行 docker-compose up
        Command::new("docker-compose")
            .args(&["up", "-d"])
            .output()
            .await?;

        Ok(DockerInfo {
            url: "http://localhost:8080".to_string(),
        })
    }
}
```

---

## 十二、小白用户优化设计

### 12.1 三层渐进式架构

```
┌─────────────────────────────────────────────────────────────┐
│                    Level 1: 完全托管模式                     │
│                    （推荐小白用户）                           │
├─────────────────────────────────────────────────────────────┤
│  • 内置 OpenClaw 二进制（无需单独安装）                        │
│  • 自动配置最佳实践（无需手动选择）                            │
│  • 一键启动，无暴露配置选项                                    │
│  • 智能故障恢复（自动重试、切换镜像）                          │
│  • 遇到问题时提供"一键修复"按钮                                │
└─────────────────────────────────────────────────────────────┘
                              ↓ 切换
┌─────────────────────────────────────────────────────────────┐
│                    Level 2: 向导配置模式                     │
│                    （有一定经验的用户）                        │
├─────────────────────────────────────────────────────────────┤
│  • 图形化安装向导（分步骤引导）                                │
│  • 预设配置模板（开发/写作/数据分析场景）                      │
│  • 可视化 Agent 管理（拖拽、表单）                             │
│  • 基础诊断修复（问题检测 + 修复建议）                         │
└─────────────────────────────────────────────────────────────┘
                              ↓ 切换
┌─────────────────────────────────────────────────────────────┐
│                    Level 3: 专家模式                         │
│                    （高级用户）                               │
├─────────────────────────────────────────────────────────────┤
│  • 直接编辑配置文件（YAML 编辑器）                             │
│  • 高级参数调优（温度、token 限制等）                          │
│  • 自定义 Skill 开发（本地路径加载）                           │
│  • 完整日志和调试信息                                        │
│  • 命令行高级操作                                            │
└─────────────────────────────────────────────────────────────┘
```

### 12.2 智能安装流程（故障转移）

```rust
// 安装阶段自动故障转移
pub async fn smart_install(&self) -> Result<InstallResult, InstallError> {
    info!("开始智能安装流程...");

    // 步骤 1: 尝试在线安装（最新版）
    info!("尝试在线安装...");
    match self.online_install().await {
        Ok(result) => return Ok(result),
        Err(e) => {
            warn!("在线安装失败: {}", e);
            self.show_notification("在线安装失败，尝试备用方案...").await;
        }
    }

    // 步骤 2: 尝试镜像源安装
    for (name, mirror) in MIRRORS.iter() {
        info!("尝试镜像源: {}", name);
        match self.install_from_mirror(mirror).await {
            Ok(result) => {
                self.show_notification(&format!("使用 {} 镜像源安装成功", name)).await;
                return Ok(result);
            }
            Err(e) => {
                warn!("镜像源 {} 失败: {}", name, e);
            }
        }
    }

    // 步骤 3: 检查本地离线包
    info!("检查本地离线包...");
    if let Some(offline_path) = self.find_offline_package().await {
        match self.offline_install(&offline_path).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                warn!("离线安装失败: {}", e);
            }
        }
    }

    // 步骤 4: 询问是否使用 Docker
    if self.confirm_dialog(
        "常规安装失败",
        "是否尝试使用 Docker 部署？这需要已安装 Docker。"
    ).await {
        match self.docker_install().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                error!("Docker 安装也失败: {}", e);
            }
        }
    }

    // 所有方法都失败
    Err(InstallError::AllMethodsFailed)
}
```

### 12.3 零配置启动设计

```typescript
// 自动选择最佳配置
async function autoConfigure(): Promise<Configuration> {
  const config: Configuration = {
    // 自动检测最佳模型
    models: {
      default: await detectBestModel(),
      fallbacks: [],
    },
    // 自动选择端口
    server: {
      port: await findAvailablePort(8080, 9000),
    },
    // 根据使用场景预设
    agent: await selectPresetByScenario(),
  };

  return config;
}

// 使用场景检测
async function detectUsageScenario(): Promise<Scenario> {
  // 检测是否安装了代码编辑器
  const hasVSCode = await checkVSCodeInstalled();
  const hasCursor = await checkCursorInstalled();

  if (hasVSCode || hasCursor) {
    return Scenario.Developer;
  }

  // 检测是否有大量文档文件
  const hasManyDocuments = await checkDocumentFiles();
  if (hasManyDocuments) {
    return Scenario.Writer;
  }

  // 默认通用场景
  return Scenario.General;
}
```

### 12.4 一键诊断与修复

```rust
pub struct OneClickFix;

impl OneClickFix {
    pub async fn run(&self) -> Result<FixReport, FixError> {
        let mut report = FixReport::new();

        // 检测常见问题并自动修复
        let checks = vec![
            self.fix_network_issues().await,
            self.fix_permission_issues().await,
            self.fix_port_conflicts().await,
            self.fix_config_errors().await,
            self.fix_service_issues().await,
        ];

        for check in checks {
            match check {
                Ok(fix) => report.add_success(fix),
                Err(e) => report.add_failure(e),
            }
        }

        Ok(report)
    }

    async fn fix_network_issues(&self) -> Result<Fix, FixError> {
        // 检测网络
        if !self.can_access_github().await {
            // 自动切换到可用镜像
            self.switch_to_mirror().await?;
            return Ok(Fix::new("网络", "已切换到可用镜像源"));
        }
        Ok(Fix::new("网络", "网络连接正常"))
    }

    async fn fix_port_conflicts(&self) -> Result<Fix, FixError> {
        let occupied_ports = self.check_ports(&[8080, 8081]).await?;

        if !occupied_ports.is_empty() {
            // 寻找可用端口
            let new_port = self.find_available_port(9000..10000).await?;
            self.update_config_port(new_port).await?;
            return Ok(Fix::new(
                "端口",
                &format!("端口 {} 被占用，已切换到 {}", occupied_ports[0], new_port)
            ));
        }

        Ok(Fix::new("端口", "端口无冲突"))
    }
}
```

### 12.5 远程协助支持

```typescript
// 生成支持包，便于远程协助
async function generateSupportBundle(): Promise<string> {
  const bundle = {
    // 系统信息
    system: {
      os: await getOSInfo(),
      arch: await getArch(),
      resources: await getResourceUsage(),
    },
    // 应用信息
    app: {
      version: APP_VERSION,
      installPath: await getInstallPath(),
      config: await exportConfigSanitized(), // 脱敏
    },
    // 日志
    logs: {
      app: await getAppLogs(100),
      openclaw: await getOpenClawLogs(100),
    },
    // 诊断结果
    diagnostics: await runDiagnostics(),
    // 截图
    screenshots: await captureScreenshots(),
  };

  // 打包为 zip
  const zipPath = await createZip(bundle, `support-bundle-${Date.now()}.zip`);
  return zipPath;
}

// 脱敏处理配置
function sanitizeConfig(config: OpenClawConfig): OpenClawConfig {
  return {
    ...config,
    models: {
      ...config.models,
      providers: config.models.providers.map(p => ({
        ...p,
        apiKey: p.apiKey ? '***REDACTED***' : '', // 隐藏 API Key
      })),
    },
  };
}
```

---

## 十三、系统要求与兼容性矩阵

### 13.1 最低系统要求

| 平台 | 最低版本 | 最低内存 | 最低磁盘 | 其他要求 |
|------|---------|---------|---------|---------|
| Windows | Windows 10 (1903+) | 4GB | 500MB | WebView2 Runtime |
| macOS | macOS 11 (Big Sur) | 4GB | 500MB | - |
| Linux | Ubuntu 20.04 / Fedora 35 | 4GB | 500MB | WebKit2GTK 4.0 |

### 13.2 测试矩阵

| 平台 | 版本 | 架构 | 测试优先级 | 说明 |
|------|------|------|-----------|------|
| Windows 11 | 23H2 | x64 | P0 | 主力平台 |
| Windows 10 | 22H2 | x64 | P0 | 兼容性测试 |
| macOS Sonoma | 14.x | ARM64 | P0 | Apple Silicon |
| macOS Ventura | 13.x | x64 | P1 | Intel Mac |
| Ubuntu | 22.04 LTS | x64 | P0 | 主力 Linux |
| Ubuntu | 20.04 LTS | x64 | P1 | 兼容性测试 |
| Fedora | 39 | x64 | P2 | 新特性测试 |
| Debian | 12 | x64 | P2 | 稳定性测试 |

### 13.3 已知限制

| 限制 | 说明 | 解决方案 |
|------|------|---------|
| Windows 7/8 | 不支持 | 建议使用 Web 模式或 Docker |
| 纯离线环境 | 无法下载 Skill | 提供离线安装包 |
| 32位系统 | 不提供原生支持 | 使用 Web 模式 |
| 无管理员权限 | 无法安装到系统目录 | 提供便携版 |

---

## 十四、详细开发计划

### Phase 1: 基础框架搭建（Week 1-2）

#### Week 1: 项目初始化与环境配置
- [ ] **Day 1-2: 项目脚手架搭建**
  - [ ] 初始化 Tauri v2 + React + TypeScript 项目
  - [ ] 配置 pnpm workspace
  - [ ] 配置 ESLint + Prettier + TypeScript 严格模式
  - [ ] 配置 Git hooks (lint-staged + husky)

- [ ] **Day 3-4: UI 组件库配置**
  - [ ] 初始化 shadcn/ui
  - [ ] 配置 Tailwind CSS 主题
  - [ ] 创建基础布局组件 (Layout, Sidebar, Header)
  - [ ] 创建通用 UI 组件库封装

- [ ] **Day 5: 状态管理配置**
  - [ ] 配置 Zustand + 持久化
  - [ ] 定义核心状态类型
  - [ ] 创建 AppStore, ConfigStore, ServiceStore

#### Week 2: 核心架构实现
- [ ] **Day 1-2: Rust 基础模块**
  - [ ] 创建错误处理体系 (AppError)
  - [ ] 实现日志系统 (tracing + 文件日志)
  - [ ] 创建平台抽象层 (PlatformAdapter trait)
  - [ ] 实现 Windows/macOS/Linux 平台适配器

- [ ] **Day 3-4: Tauri Commands 框架**
  - [ ] 定义所有 IPC 命令接口
  - [ ] 实现命令路由和错误转换
  - [ ] 创建前端 API 封装层
  - [ ] 配置 CORS 和安全策略

- [ ] **Day 5: 前端路由与布局**
  - [ ] 配置 TanStack Router
  - [ ] 实现主布局 (Sidebar + Main Content)
  - [ ] 创建空页面占位符
  - [ ] 实现路由守卫和权限控制

**Phase 1 交付物**: 可运行的空壳应用，具备基础架构

---

### Phase 2: 安装与配置系统（Week 3-5）

#### Week 3: 安装管理器
- [ ] **Day 1-2: 系统检测模块**
  - [ ] 实现系统要求检测 (OS, 内存, 磁盘)
  - [ ] 实现网络环境检测
  - [ ] 实现端口占用检测
  - [ ] 实现依赖检查 (Node.js, Git, Python)

- [ ] **Day 3-4: 下载与安装**
  - [ ] 实现多镜像源管理
  - [ ] 实现下载进度追踪 (带断点续传)
  - [ ] 实现安装脚本执行器
  - [ ] 实现事务性安装 (带回滚)

- [ ] **Day 5: 安装向导 UI**
  - [ ] 创建安装向导页面框架
  - [ ] 实现环境检测步骤 UI
  - [ ] 实现下载进度展示
  - [ ] 实现安装完成页面

#### Week 4: 配置管理器
- [ ] **Day 1-2: 配置读写**
  - [ ] 实现 YAML 配置解析
  - [ ] 实现配置验证 (Schema)
  - [ ] 实现乐观锁并发控制
  - [ ] 实现配置文件监听

- [ ] **Day 3-4: 配置同步**
  - [ ] 实现前后端配置同步
  - [ ] 实现配置变更事件通知
  - [ ] 实现配置导入/导出
  - [ ] 实现配置备份/恢复

- [ ] **Day 5: API Key 安全存储**
  - [ ] 集成 keyring (系统密钥链)
  - [ ] 实现 API Key 加密存储
  - [ ] 实现安全的读取接口

#### Week 5: OpenClaw CLI 适配器
- [ ] **Day 1-2: CLI 发现与调用**
  - [ ] 实现 CLI 自动发现
  - [ ] 实现命令执行封装
  - [ ] 实现超时和错误处理
  - [ ] 实现 JSON 输出解析

- [ ] **Day 3-4: 版本适配**
  - [ ] 设计版本适配器架构
  - [ ] 实现 v1.x CLI 适配器
  - [ ] 实现命令兼容性层

- [ ] **Day 5: 集成测试**
  - [ ] 编写安装流程 E2E 测试
  - [ ] 测试各平台安装兼容性
  - [ ] 修复发现的问题

**Phase 2 交付物**: 完整的安装和配置系统，可独立完成 OpenClaw 安装

---

### Phase 3: 核心功能实现（Week 6-9）

#### Week 6: 进程管理与服务控制
- [ ] **Day 1-2: 进程管理器**
  - [ ] 实现进程生命周期管理
  - [ ] 实现优雅关闭机制
  - [ ] 实现健康检查
  - [ ] 实现自动重启策略

- [ ] **Day 3-4: 服务控制面板**
  - [ ] 创建仪表盘页面
  - [ ] 实现服务状态展示
  - [ ] 实现启动/停止/重启控制
  - [ ] 实现实时状态更新 (WebSocket/Events)

- [ ] **Day 5: 日志系统**
  - [ ] 实现日志文件监控
  - [ ] 实现日志解析和结构化
  - [ ] 实现实时日志流推送
  - [ ] 创建日志查看器页面

#### Week 7: 模型配置管理
- [ ] **Day 1-2: 模型提供商管理**
  - [ ] 实现提供商 CRUD
  - [ ] 实现 API Key 输入组件（安全显示）
  - [ ] 实现连接测试功能

- [ ] **Day 3-4: 模型配置页面**
  - [ ] 创建模型配置页面
  - [ ] 实现默认模型选择
  - [ ] 实现备用模型排序 (拖拽)
  - [ ] 实现模型参数配置

- [ ] **Day 5: 连接测试与验证**
  - [ ] 实现 API 连接测试
  - [ ] 实现余额查询（支持的平台）
  - [ ] 实现错误诊断

#### Week 8: Agent 管理
- [ ] **Day 1-2: Agent CRUD**
  - [ ] 实现 Agent 列表查询
  - [ ] 实现 Agent 创建向导
  - [ ] 实现 Agent 配置编辑
  - [ ] 实现 Agent 切换逻辑

- [ ] **Day 3-4: Agent 配置页面**
  - [ ] 创建 Agent 管理页面
  - [ ] 实现 Agent 卡片/列表视图
  - [ ] 实现配置模板选择
  - [ ] 实现导入/导出功能

- [ ] **Day 5: Agent 模板系统**
  - [ ] 设计预设模板 (Developer/Writer/Analyst)
  - [ ] 实现模板应用逻辑
  - [ ] 允许用户保存自定义模板

#### Week 9: 技能管理
- [ ] **Day 1-2: 技能列表与搜索**
  - [ ] 实现已安装技能列表
  - [ ] 实现技能搜索/筛选
  - [ ] 实现技能启用/禁用

- [ ] **Day 3-4: 技能商店集成**
  - [ ] 创建技能商店页面
  - [ ] 实现技能详情展示
  - [ ] 实现一键安装/卸载

- [ ] **Day 5: 技能配置**
  - [ ] 实现技能参数配置
  - [ ] 实现本地技能加载

**Phase 3 交付物**: 完整的 OpenClaw 管理功能，支持模型、Agent、技能的全面管理

---

### Phase 4: 诊断与优化（Week 10-11）

#### Week 10: 诊断引擎
- [ ] **Day 1-2: 诊断检测器**
  - [ ] 实现网络诊断
  - [ ] 实现配置诊断
  - [ ] 实现服务诊断
  - [ ] 实现依赖诊断

- [ ] **Day 3-4: 自动修复**
  - [ ] 实现一键诊断页面
  - [ ] 实现自动修复逻辑
  - [ ] 实现修复结果展示
  - [ ] 实现手动修复引导

- [ ] **Day 5: 健康监控**
  - [ ] 实现后台健康检查
  - [ ] 实现问题预警通知
  - [ ] 实现系统资源监控

#### Week 11: 性能优化与体验改进
- [ ] **Day 1-2: 性能优化**
  - [ ] 前端代码分割和懒加载
  - [ ] 配置保存防抖优化
  - [ ] 日志虚拟滚动优化
  - [ ] Rust 端性能优化

- [ ] **Day 3-4: 用户体验**
  - [ ] 完善错误提示和引导
  - [ ] 添加操作确认和撤销
  - [ ] 优化加载状态和骨架屏
  - [ ] 添加键盘快捷键支持

- [ ] **Day 5: 降级方案实现**
  - [ ] 实现 Web 模式支持
  - [ ] 实现 Docker 部署引导
  - [ ] 实现故障转移逻辑

**Phase 4 交付物**: 具备自诊断和自修复能力的稳定系统

---

### Phase 5: 测试与发布（Week 12-13）

#### Week 12: 全面测试
- [ ] **Day 1-2: 单元测试**
  - [ ] Rust 核心模块单元测试
  - [ ] 前端组件单元测试 (Vitest)
  - [ ] 关键 Hooks 测试

- [ ] **Day 3-4: 集成测试**
  - [ ] 安装流程 E2E 测试
  - [ ] 配置管理集成测试
  - [ ] 服务控制集成测试

- [ ] **Day 5: 跨平台测试**
  - [ ] Windows 10/11 测试
  - [ ] macOS Intel/ARM 测试
  - [ ] Ubuntu/Fedora 测试

#### Week 13: 打包与发布
- [ ] **Day 1-2: 打包配置**
  - [ ] 配置代码签名
  - [ ] 配置自动更新
  - [ ] 优化包体积

- [ ] **Day 3-4: CI/CD**
  - [ ] 配置 GitHub Actions
  - [ ] 配置自动发布
  - [ ] 配置更新服务器

- [ ] **Day 5: 文档与发布**
  - [ ] 编写用户文档
  - [ ] 编写 FAQ
  - [ ] 发布 v1.0.0

**Phase 5 交付物**: 正式发布的 v1.0.0 版本，支持自动更新

---

## 十五、风险评估与缓解

### 15.1 技术风险

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| Tauri v2 API 变更 | 中 | 高 | 关注 RC 版本，使用稳定特性，预留适配层 |
| OpenClaw CLI 变更 | 高 | 高 | 实现版本适配器，支持多版本兼容 |
| WebView 兼容性问题 | 中 | 中 | 充分测试各平台 WebView 版本，提供降级方案 |
| 网络不稳定 | 高 | 中 | 多镜像源，断点续传，离线包支持 |

### 15.2 项目风险

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| 开发周期延期 | 中 | 中 | 分阶段交付，MVP 优先，灵活调整范围 |
| 跨平台问题复杂 | 中 | 高 | 尽早开始多平台测试，预留缓冲时间 |
| 用户体验不达预期 | 低 | 高 | 早期用户测试，持续迭代优化 |

---

## 十六、附录

### 16.1 参考资源

- [Tauri v2 官方文档](https://tauri.app/)
- [Tauri API 参考](https://tauri.app/reference/)
- [shadcn/ui 组件库](https://ui.shadcn.com/)
- [TanStack Query v5](https://tanstack.com/query/latest)
- [Zustand 文档](https://docs.pmnd.rs/zustand)
- [Rust Async Book](https://rust-lang.github.io/async-book/)

### 16.2 相关工具

| 工具 | 用途 |
|------|------|
| `cargo-tauri` | Tauri CLI 工具 |
| `trunk` | Rust WebAssembly 构建（如需要） |
| `cargo-watch` | Rust 开发热重载 |
| `vite` | 前端构建工具 |
| `playwright` | E2E 测试 |
| `vitest` | 前端单元测试 |

---

**文档版本**: 2.1
**最后更新**: 2024-01
**状态**: 待评审
**下一步**: 技术方案评审通过后进入 Phase 1 开发
