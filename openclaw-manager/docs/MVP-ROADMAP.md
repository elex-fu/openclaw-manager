# OpenClaw Manager MVP 开发执行计划

> 版本: 1.0  
> 创建时间: 2026-02-26  
> 目标: 完成 OpenClaw 一键安装和操作管理 MVP

---

## 一、产品定位调整

### 1.1 核心定位

**OpenClaw Manager** 是一款专注于 OpenClaw 安装、配置和管理的桌面应用。

**目标用户**:  
- 初次接触 OpenClaw 的新用户（一键安装）  
- 需要图形化管理 OpenClaw 的进阶用户

**核心功能**:  
1. 一键安装/卸载 OpenClaw（支持离线包）  
2. 模型配置管理（API Key 安全存储、连接测试）  
3. Agent 管理（创建、切换、模板）  
4. 服务控制（Gateway 启动/停止/监控）  
5. 一键诊断修复

### 1.2 功能裁剪

**删除/废弃功能**:
- ❌ 文件扫描与管理
- ❌ 文件分组系统
- ❌ 文件标签与属性
- ❌ 文件数据库相关代码

**保留功能**:
- ✅ OpenClaw 安装/卸载
- ✅ 系统环境检测
- ✅ 配置管理（YAML）
- ✅ 服务控制（基础）

**新增功能**:
- 🆕 离线安装包支持
- 🆕 安全密钥存储（Keychain）
- 🆕 模型管理 UI
- 🆕 Agent 管理 UI
- 🆕 服务监控仪表盘
- 🆕 诊断修复系统

---

## 二、技术架构升级

### 2.1 Tauri v1 → v2 升级方案

#### 破坏性变更清单

| v1 API | v2 替代方案 | 影响文件 |
|--------|------------|----------|
| `tauri::api::process::Command` | `tauri::process::Command` | `installer/mod.rs` |
| `tauri::SystemTray` | `tauri::tray::TrayIcon` | `main.rs` |
| `tauri::Manager` | `tauri::AppHandle` | 所有 commands |
| `Window` 事件 | `WebviewWindow` 事件 | `main.rs` |
| `tauri::generate_handler` | `tauri::generate_handler` (无变化) | - |

#### Cargo.toml 变更

```toml
[dependencies]
# v1
tauri = { version = "1.6", features = [...] }

# v2
tauri = { version = "2.0", features = [] }
tauri-plugin-shell = "2.0"
tauri-plugin-process = "2.0"
tauri-plugin-notification = "2.0"
tauri-plugin-updater = "2.0"
tauri-plugin-os = "2.0"
tauri-plugin-fs = "2.0"
```

#### 前端 API 变更

```typescript
// v1
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

// v2
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Command } from '@tauri-apps/plugin-shell'
```

### 2.2 项目结构重组

```
openclaw-manager/
├── src/                          # 前端源码
│   ├── components/
│   │   ├── ui/                   # shadcn 基础组件
│   │   ├── layout/               # 布局组件
│   │   │   └── MainLayout.tsx
│   │   └── openclaw/             # OpenClaw 相关组件
│   │       ├── InstallerPanel.tsx
│   │       ├── ModelConfigCard.tsx
│   │       ├── AgentCard.tsx
│   │       ├── ServiceStatus.tsx
│   │       └── DiagnosticPanel.tsx
│   ├── pages/
│   │   ├── Dashboard.tsx         # 仪表盘（首页）
│   │   ├── InstallWizard.tsx     # 安装向导
│   │   ├── ModelConfig.tsx       # 模型配置
│   │   ├── AgentManager.tsx      # Agent 管理
│   │   └── Diagnostics.tsx       # 诊断修复
│   ├── hooks/                    # React Hooks
│   ├── stores/                   # Zustand 状态管理
│   │   ├── appStore.ts
│   │   ├── installStore.ts
│   │   └── configStore.ts
│   ├── lib/
│   │   ├── tauri-api.ts          # Tauri API 封装
│   │   ├── errors.ts             # 错误类型定义
│   │   └── utils.ts
│   ├── types/
│   │   └── index.ts              # TypeScript 类型
│   └── App.tsx
├──
│── src-tauri/                     # Rust 后端
│   ├── src/
│   │   ├── main.rs               # 应用入口
│   │   ├── lib.rs                # 库入口
│   │   ├── commands/             # Tauri Commands
│   │   │   ├── mod.rs
│   │   │   ├── install.rs        # 安装相关
│   │   │   ├── config.rs         # 配置管理
│   │   │   ├── service.rs        # 服务控制
│   │   │   ├── model.rs          # 模型管理
│   │   │   ├── agent.rs          # Agent 管理
│   │   │   └── diagnose.rs       # 诊断修复
│   │   ├── services/             # 业务服务层
│   │   │   ├── mod.rs
│   │   │   ├── installer.rs      # 安装服务（重构后）
│   │   │   ├── config_manager.rs # 配置管理器
│   │   │   ├── process_manager.rs# 进程管理器
│   │   │   ├── secure_storage.rs # 安全存储
│   │   │   └── diagnostics.rs    # 诊断引擎
│   │   ├── models/               # 数据模型
│   │   │   ├── mod.rs
│   │   │   ├── install.rs        # 安装相关类型
│   │   │   ├── config.rs         # 配置类型
│   │   │   ├── model.rs          # 模型类型
│   │   │   ├── agent.rs          # Agent 类型
│   │   │   └── types.rs          # 通用类型
│   │   ├── errors/               # 错误处理
│   │   │   ├── mod.rs            # 统一错误类型
│   │   │   └── app_error.rs
│   │   └── utils/                # 工具模块
│   │       ├── mod.rs
│   │       ├── retry.rs          # 重试机制
│   │       └── platform.rs       # 平台适配
│   ├── capabilities/             # Tauri v2 权限配置
│   │   └── default.json
│   ├── Cargo.toml
│   └── tauri.conf.json
├── tests/                        # 测试目录
│   ├── unit/                     # 单元测试
│   │   ├── rust/                 # Rust 单元测试
│   │   └── ts/                   # TypeScript 单元测试
│   └── e2e/                      # E2E 测试
│       └── specs/
├── scripts/                      # 脚本目录
│   ├── build-offline-package.sh  # 离线包构建脚本
│   └── install/
│       ├── macos/
│       │   └── install.sh
│       └── windows/
│           └── install.ps1
├── docs/
│   └── MVP-ROADMAP.md
└── package.json
```

---

## 三、核心功能技术方案

### 3.1 离线安装包支持

#### 方案选型

| 方案 | 优点 | 缺点 | 决策 |
|------|------|------|------|
| A. 嵌入完整二进制 | 完全离线，速度快 | 包体积大（50MB+） | ✅ 采用 |
| B. 可选镜像源 | 包体积小 | 仍需网络 | 备选 |
| C. 本地缓存 | 首次后离线 | 首次仍需网络 | 辅助 |

#### 实现方案 A：嵌入离线包

```rust
// src-tauri/src/services/installer.rs

/// 离线安装包管理器
pub struct OfflineInstaller {
    /// 嵌入的安装包资源
    embedded_package: Option<&'static [u8]>,
    /// 包元数据
    package_info: PackageInfo,
}

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub version: String,
    pub platform: Platform,
    pub arch: Arch,
    pub checksum: String,
}

impl OfflineInstaller {
    /// 从嵌入资源创建安装器
    pub fn from_embedded() -> Result<Self, InstallError> {
        // 根据平台选择嵌入的资源
        let (package_data, info) = match (std::env::consts::OS, std::env::consts::ARCH) {
            ("macos", "aarch64") => {
                (include_bytes!("../../../bundled/openclaw-macos-arm64.tar.gz"),
                 PackageInfo { version: "1.0.0".into(), platform: Platform::MacOS, arch: Arch::ARM64, ... })
            }
            ("macos", "x86_64") => {
                (include_bytes!("../../../bundled/openclaw-macos-x64.tar.gz"), ...)
            }
            ("windows", _) => {
                (include_bytes!("../../../bundled/openclaw-windows-x64.zip"), ...)
            }
            ("linux", _) => {
                (include_bytes!("../../../bundled/openclaw-linux-x64.tar.gz"), ...)
            }
            _ => return Err(InstallError::UnsupportedPlatform),
        };

        Ok(Self {
            embedded_package: Some(package_data),
            package_info: info,
        })
    }

    /// 执行离线安装
    pub async fn install_offline(&self, target_dir: &Path) -> Result<InstallResult, InstallError> {
        // 1. 验证完整性
        self.verify_checksum()?;
        
        // 2. 解压到临时目录
        let temp_dir = tempfile::tempdir()?;
        self.extract_package(temp_dir.path()).await?;
        
        // 3. 复制到目标目录
        self.copy_to_target(temp_dir.path(), target_dir).await?;
        
        // 4. 创建默认配置
        self.create_default_config(target_dir).await?;
        
        Ok(InstallResult { success: true, ... })
    }
}
```

#### 离线包构建脚本

```bash
#!/bin/bash
# scripts/build-offline-package.sh

set -e

VERSION="${1:-latest}"
PLATFORMS=("macos-arm64" "macos-x64" "windows-x64" "linux-x64")
OUTPUT_DIR="bundled"

mkdir -p $OUTPUT_DIR

for platform in "${PLATFORMS[@]}"; do
    echo "Building offline package for $platform..."
    
    case $platform in
        "macos-arm64")
            curl -L "https://github.com/openclaw/openclaw/releases/download/$VERSION/openclaw-$VERSION-macos-arm64.tar.gz" \
                -o "$OUTPUT_DIR/openclaw-macos-arm64.tar.gz"
            ;;
        "macos-x64")
            curl -L "https://github.com/openclaw/openclaw/releases/download/$VERSION/openclaw-$VERSION-macos-x64.tar.gz" \
                -o "$OUTPUT_DIR/openclaw-macos-x64.tar.gz"
            ;;
        "windows-x64")
            curl -L "https://github.com/openclaw/openclaw/releases/download/$VERSION/openclaw-$VERSION-windows-x64.zip" \
                -o "$OUTPUT_DIR/openclaw-windows-x64.zip"
            ;;
        "linux-x64")
            curl -L "https://github.com/openclaw/openclaw/releases/download/$VERSION/openclaw-$VERSION-linux-x64.tar.gz" \
                -o "$OUTPUT_DIR/openclaw-linux-x64.tar.gz"
            ;;
    esac
done

echo "Offline packages built in $OUTPUT_DIR/"
```

### 3.2 API Key 安全存储（Keychain）

#### 技术选型

使用 `keyring` crate 提供跨平台系统密钥链访问：
- **macOS**: Keychain Services
- **Windows**: Windows Credential Manager
- **Linux**: Secret Service API / libsecret

#### 实现代码

```rust
// src-tauri/src/services/secure_storage.rs

use keyring::Entry;
use thiserror::Error;

const SERVICE_NAME: &str = "com.openclaw.manager";

#[derive(Error, Debug)]
pub enum SecureStorageError {
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
}

pub struct SecureStorage;

impl SecureStorage {
    /// 保存 API Key
    pub fn save_api_key(provider: &str, api_key: &str) -> Result<(), SecureStorageError> {
        let entry = Entry::new(SERVICE_NAME, &format!("api_key_{}", provider))?;
        entry.set_password(api_key)?;
        Ok(())
    }

    /// 获取 API Key
    pub fn get_api_key(provider: &str) -> Result<Option<String>, SecureStorageError> {
        let entry = Entry::new(SERVICE_NAME, &format!("api_key_{}", provider))?;
        match entry.get_password() {
            Ok(key) => Ok(Some(key)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// 删除 API Key
    pub fn delete_api_key(provider: &str) -> Result<(), SecureStorageError> {
        let entry = Entry::new(SERVICE_NAME, &format!("api_key_{}", provider))?;
        entry.delete_password()?;
        Ok(())
    }

    /// 检查是否存在
    pub fn has_api_key(provider: &str) -> Result<bool, SecureStorageError> {
        Ok(Self::get_api_key(provider)?.is_some())
    }
}

// Tauri Command
#[tauri::command]
pub async fn save_model_api_key(
    provider: String,
    api_key: String,
) -> Result<ApiResponse<()>, AppError> {
    SecureStorage::save_api_key(&provider, &api_key)?;
    Ok(ApiResponse::success(()))
}
```

#### 配置结构调整

```rust
// models/config.rs

/// 模型配置（存储在 YAML 中，不含 API Key）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub api_base: Option<String>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<i32>,
    pub enabled: bool,
    // ❌ 不再存储 api_key
}

/// 运行时模型配置（从 Keychain 读取 API Key）
pub struct RuntimeModelConfig {
    pub base: ModelConfig,
    pub api_key: Option<String>,
}
```

### 3.3 统一错误处理

#### 错误类型定义

```rust
// src-tauri/src/errors/app_error.rs

use thiserror::Error;
use serde::Serialize;

#[derive(Error, Debug)]
pub enum AppError {
    // 安装错误
    #[error("Installation failed: {0}")]
    Install(#[from] InstallError),
    
    // 配置错误
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    // 网络错误
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    // 安全存储错误
    #[error("Secure storage error: {0}")]
    SecureStorage(#[from] SecureStorageError),
    
    // 进程错误
    #[error("Process error: {0}")]
    Process(#[from] ProcessError),
    
    // IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    // 未知错误
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl AppError {
    /// 转换为前端友好的错误信息
    pub fn to_user_message(&self) -> UserErrorMessage {
        match self {
            AppError::Install(e) => UserErrorMessage {
                title: "安装失败".to_string(),
                description: e.to_string(),
                action: Some("检查网络连接或尝试离线安装".to_string()),
                severity: ErrorSeverity::Error,
                retryable: true,
            },
            AppError::Network(_) => UserErrorMessage {
                title: "网络连接失败".to_string(),
                description: "无法连接到远程服务器".to_string(),
                action: Some("检查网络设置或切换镜像源".to_string()),
                severity: ErrorSeverity::Warning,
                retryable: true,
            },
            AppError::Config(_) => UserErrorMessage {
                title: "配置错误".to_string(),
                description: self.to_string(),
                action: Some("检查配置文件格式".to_string()),
                severity: ErrorSeverity::Warning,
                retryable: false,
            },
            _ => UserErrorMessage {
                title: "操作失败".to_string(),
                description: self.to_string(),
                action: None,
                severity: ErrorSeverity::Error,
                retryable: false,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UserErrorMessage {
    pub title: String,
    pub description: String,
    pub action: Option<String>,
    pub severity: ErrorSeverity,
    pub retryable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}
```

#### 重试机制实现

```rust
// src-tauri/src/utils/retry.rs

use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

/// 带重试的异步操作
pub async fn retry_with_backoff<F, Fut, T, E>(
    config: RetryConfig,
    operation: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut delay = config.initial_delay;
    
    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                log::warn!("Attempt {} failed: {:?}", attempt, e);
                
                if attempt == config.max_attempts {
                    return Err(e);
                }
                
                sleep(delay).await;
                
                // 指数退避
                delay = std::cmp::min(
                    Duration::from_millis(
                        (delay.as_millis() as f64 * config.backoff_multiplier) as u64
                    ),
                    config.max_delay,
                );
            }
        }
    }
    
    unreachable!()
}

/// 可重试的命令 trait
#[async_trait::async_trait]
pub trait RetryableCommand {
    type Output;
    type Error;
    
    async fn execute(&self) -> Result<Self::Output, Self::Error>;
    
    async fn execute_with_retry(&self, config: RetryConfig) -> Result<Self::Output, Self::Error> {
        retry_with_backoff(config, || self.execute()).await
    }
}
```

### 3.4 进程管理器（ProcessManager）

```rust
// src-tauri/src/services/process_manager.rs

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::{RwLock, broadcast};
use sysinfo::{ProcessExt, System, SystemExt};

#[derive(Debug, Clone, Serialize)]
pub enum ServiceStatus {
    Starting,
    Running { pid: u32, started_at: u64 },
    Stopping,
    Stopped,
    Error(String),
    Crashed { exit_code: i32, message: String },
}

pub struct ProcessManager {
    services: Arc<RwLock<HashMap<String, ServiceHandle>>>,
    event_sender: broadcast::Sender<ProcessEvent>,
}

struct ServiceHandle {
    process: Child,
    info: ServiceInfo,
    shutdown_tx: tokio::sync::mpsc::Sender<()>,
}

impl ProcessManager {
    /// 启动服务
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
        drop(services);

        // 启动监控任务
        self.spawn_monitor_task(name.clone(), shutdown_rx);

        // 启动日志收集
        self.spawn_log_collector(name.clone());

        self.event_sender.send(ProcessEvent::Started { name, pid }).ok();

        Ok(info.status)
    }

    /// 优雅停止服务
    pub async fn stop_service(&self, name: &str, timeout_secs: u64) -> Result<(), ProcessError> {
        let mut services = self.services.write().await;

        let handle = services.get_mut(name).ok_or(ProcessError::NotFound)?;
        handle.info.status = ServiceStatus::Stopping;

        // 发送优雅关闭信号
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

        let shutdown_tx = handle.shutdown_tx.clone();
        drop(services);

        // 等待进程退出或超时
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

    /// 健康检查
    pub async fn health_check(&self, name: &str) -> Result<HealthStatus, ProcessError> {
        let services = self.services.read().await;
        let handle = services.get(name).ok_or(ProcessError::NotFound)?;

        // 检查进程是否存活
        let mut system = System::new_all();
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
}
```

---

## 四、开发阶段规划

### Phase 0: 基础准备（1 天）

#### 任务清单

- [ ] **0.1** 删除文件管理相关代码
  - [ ] 删除 `src/commands/file.rs`
  - [ ] 删除 `src/commands/group.rs`
  - [ ] 删除 `src/models/file.rs`
  - [ ] 删除 `src/models/group.rs`
  - [ ] 删除 `src/db/` 目录
  - [ ] 清理前端 `FileListPage.tsx`, `GroupPage.tsx`
  - [ ] 更新 `main.rs` 中的 invoke_handler

- [ ] **0.2** 创建文档结构
  - [ ] 创建本执行计划文档
  - [ ] 创建 API 文档模板
  - [ ] 创建 CHANGELOG.md

- [ ] **0.3** 项目初始化检查
  - [ ] 确认 Rust 版本 >= 1.70
  - [ ] 确认 Node.js 版本 >= 18
  - [ ] 运行基础构建测试

### Phase 1: Tauri v2 升级（2 天）

#### 任务清单

- [ ] **1.1** 更新 Cargo.toml
  ```toml
  [dependencies]
  tauri = { version = "2.0", features = [] }
  tauri-build = { version = "2.0", features = [] }
  
  tauri-plugin-shell = "2.0"
  tauri-plugin-process = "2.0"
  tauri-plugin-notification = "2.0"
  tauri-plugin-os = "2.0"
  ```

- [ ] **1.2** 更新前端依赖
  ```bash
  npm uninstall @tauri-apps/api @tauri-apps/cli
  npm install @tauri-apps/api@^2.0.0 @tauri-apps/cli@^2.0.0
  npm install @tauri-apps/plugin-shell @tauri-apps/plugin-process
  ```

- [ ] **1.3** 迁移 API 调用
  - [ ] 更新 `lib/tauri-api.ts`
  - [ ] 替换所有 `@tauri-apps/api/tauri` 为 `@tauri-apps/api/core`
  - [ ] 更新 `listen` 导入路径

- [ ] **1.4** 迁移 Rust 代码
  - [ ] 更新 `main.rs` 中的 tray 初始化
  - [ ] 更新所有 `Window` 引用为 `WebviewWindow`
  - [ ] 更新 `Command` 导入路径
  - [ ] 更新 `Manager` 使用方式

- [ ] **1.5** 更新配置文件
  - [ ] 创建 `capabilities/default.json`
  - [ ] 更新 `tauri.conf.json` 到 v2 格式
  - [ ] 移除旧版 `allowlist` 配置

- [ ] **1.6** 测试验证
  - [ ] 运行 `cargo check`
  - [ ] 运行 `npm run tauri:dev`
  - [ ] 验证基础功能正常

### Phase 2: 核心架构重构（2 天）

#### 任务清单

- [ ] **2.1** 错误处理体系
  - [ ] 创建 `src/errors/app_error.rs`
  - [ ] 定义所有错误类型
  - [ ] 实现 `to_user_message()` 方法
  - [ ] 创建 `src/utils/retry.rs`
  - [ ] 实现重试机制

- [ ] **2.2** 安全存储模块
  - [ ] 添加 `keyring` 依赖
  - [ ] 创建 `src/services/secure_storage.rs`
  - [ ] 实现 save/get/delete API
  - [ ] 添加 Tauri commands

- [ ] **2.3** 配置管理器重构
  - [ ] 创建 `src/services/config_manager.rs`
  - [ ] 实现乐观锁并发控制
  - [ ] 实现配置验证
  - [ ] 实现配置导入/导出

- [ ] **2.4** 进程管理器
  - [ ] 创建 `src/services/process_manager.rs`
  - [ ] 实现服务启动/停止
  - [ ] 实现健康检查
  - [ ] 实现优雅关闭

- [ ] **2.5** 安装器重构
  - [ ] 重构 `src/services/installer.rs`
  - [ ] 支持离线包安装
  - [ ] 支持多镜像源
  - [ ] 集成重试机制

### Phase 3: 前端重构（2 天）

#### 任务清单

- [ ] **3.1** 页面结构调整
  - [ ] 删除 `FileListPage.tsx`
  - [ ] 删除 `GroupPage.tsx`
  - [ ] 创建 `Dashboard.tsx`（仪表盘）
  - [ ] 创建 `InstallWizard.tsx`（安装向导）
  - [ ] 创建 `ModelConfig.tsx`（模型配置）
  - [ ] 创建 `AgentManager.tsx`（Agent 管理）
  - [ ] 更新 `App.tsx` 路由

- [ ] **3.2** 状态管理重构
  - [ ] 重构 `stores/appStore.ts`
  - [ ] 创建 `stores/installStore.ts`
  - [ ] 创建 `stores/configStore.ts`
  - [ ] 实现持久化

- [ ] **3.3** 组件开发
  - [ ] 创建 `ServiceStatus.tsx`（服务状态卡片）
  - [ ] 创建 `ModelConfigCard.tsx`（模型配置卡片）
  - [ ] 创建 `AgentCard.tsx`（Agent 卡片）
  - [ ] 创建 `DiagnosticPanel.tsx`（诊断面板）
  - [ ] 重构 `InstallerPanel.tsx`

- [ ] **3.4** 错误处理 UI
  - [ ] 创建错误提示组件
  - [ ] 实现重试按钮
  - [ ] 集成全局错误边界

### Phase 4: 离线安装包（1 天）

#### 任务清单

- [ ] **4.1** 离线包构建脚本
  - [ ] 创建 `scripts/build-offline-package.sh`
  - [ ] 实现多平台下载
  - [ ] 添加校验和生成

- [ ] **4.2** 离线安装实现
  - [ ] 实现 `OfflineInstaller` 结构
  - [ ] 实现嵌入资源加载
  - [ ] 实现解压安装逻辑
  - [ ] 添加进度事件

- [ ] **4.3** 镜像源管理
  - [ ] 创建镜像源配置
  - [ ] 实现自动选择最佳镜像
  - [ ] 实现手动切换镜像

### Phase 5: 测试体系（2 天）

#### 任务清单

- [ ] **5.1** Rust 单元测试
  - [ ] 测试 `secure_storage.rs`
  - [ ] 测试 `config_manager.rs`
  - [ ] 测试 `retry.rs`
  - [ ] 测试 `installer.rs`

- [ ] **5.2** 前端单元测试
  - [ ] 配置 Vitest
  - [ ] 测试 stores
  - [ ] 测试 API 封装
  - [ ] 测试工具函数

- [ ] **5.3** E2E 测试
  - [ ] 配置 Playwright
  - [ ] 编写安装流程测试
  - [ ] 编写配置管理测试
  - [ ] 编写服务控制测试

- [ ] **5.4** CI/CD
  - [ ] 创建 `.github/workflows/test.yml`
  - [ ] 配置自动化测试
  - [ ] 配置构建检查

### Phase 6: 集成与优化（1 天）

#### 任务清单

- [ ] **6.1** 功能整合测试
  - [ ] 端到端安装流程测试
  - [ ] 配置管理完整测试
  - [ ] 服务控制测试
  - [ ] 错误处理测试

- [ ] **6.2** 性能优化
  - [ ] 前端代码分割
  - [ ] 配置加载优化
  - [ ] 事件处理优化

- [ ] **6.3** 文档完善
  - [ ] 更新 README.md
  - [ ] 添加 API 文档
  - [ ] 添加用户指南

- [ ] **6.4** 发布准备
  - [ ] 版本号更新
  - [ ] 构建测试
  - [ ] 发布检查清单

---

## 五、技术细节附录

### A. Tauri v2 Capability 配置

```json
// src-tauri/capabilities/default.json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capability",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "shell:allow-execute",
    "dialog:allow-open",
    "dialog:allow-save",
    "fs:allow-read",
    "fs:allow-write",
    "fs:allow-read-dir",
    "notification:default",
    "os:default",
    "process:default"
  ]
}
```

### B. 前端错误处理示例

```typescript
// lib/errors.ts
export class AppError extends Error {
  constructor(
    message: string,
    public code: string,
    public severity: 'info' | 'warning' | 'error' | 'critical',
    public retryable: boolean,
    public action?: string
  ) {
    super(message)
  }
}

// hooks/useErrorHandler.ts
export function useErrorHandler() {
  const handleError = (error: unknown) => {
    if (error instanceof AppError) {
      toast.error(error.message, {
        action: error.retryable ? {
          label: '重试',
          onClick: () => retryOperation()
        } : undefined
      })
    }
  }
  
  return { handleError }
}
```

### C. 模型配置数据结构

```typescript
// types/model.ts
export interface ModelProvider {
  id: string
  name: string
  type: 'openai' | 'anthropic' | 'google' | 'local'
  apiBase?: string
  // apiKey 不存储在这里，从 Keychain 获取
}

export interface ModelConfig {
  id: string
  providerId: string
  model: string
  temperature: number
  maxTokens?: number
  enabled: boolean
}

export interface RuntimeModelConfig extends ModelConfig {
  apiKey?: string  // 运行时从 Keychain 获取
}
```

---

## 六、验收标准

### MVP 完成检查清单

- [ ] 用户可以一键安装 OpenClaw（离线包）
- [ ] 用户可以配置模型（API Key 安全存储）
- [ ] 用户可以创建/切换 Agent
- [ ] 用户可以启动/停止 Gateway 服务
- [ ] 用户可以进行一键诊断修复
- [ ] 所有错误都有友好提示
- [ ] 核心功能有单元测试覆盖
- [ ] 主要流程有 E2E 测试

---

## 七、风险与缓解

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| Tauri v2 升级破坏性变更 | 高 | 中 | 仔细对照迁移文档，分步验证 |
| keyring 跨平台兼容性 | 中 | 高 | 充分测试各平台，准备降级方案 |
| 离线包体积过大 | 中 | 中 | 考虑可选下载模式 |
| 开发时间超预期 | 中 | 中 | 分阶段交付，MVP 优先 |

---

## 八、开发检查点

每个 Phase 完成后，对照此文档进行检查确认：

1. **Phase 0**: 文件管理代码是否完全删除？
2. **Phase 1**: Tauri v2 是否成功升级？所有 API 是否正常工作？
3. **Phase 2**: 错误处理体系是否统一？安全存储是否可用？
4. **Phase 3**: 前端页面是否符合新设计？组件是否可复用？
5. **Phase 4**: 离线安装是否成功？镜像切换是否正常？
6. **Phase 5**: 测试覆盖率是否达标？CI/CD 是否配置完成？
7. **Phase 6**: MVP 功能是否完整？是否达到验收标准？

---

*本文档是 MVP 开发的技术指南，开发过程中如有变更需同步更新文档。*
