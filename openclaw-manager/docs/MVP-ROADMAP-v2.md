# OpenClaw Manager MVP v2 开发执行计划

> 版本: 2.0
> 创建时间: 2026-02-26
> 目标: 基于 MVP v1 的跨平台兼容性改进方案
> 改进重点: 平台细化、降级方案、构建脚本跨平台支持

---

## 一、v2 改进概要

### 1.1 针对 v1 的改进点

| 改进项 | v1 状态 | v2 改进 | 优先级 |
|--------|---------|---------|--------|
| Windows ARM64 支持 | 通配符匹配 x64 | 独立 ARM64 构建 | 高 |
| Linux ARM64 支持 | 通配符匹配 x64 | 独立 ARM64 构建 | 高 |
| Linux 安全存储降级 | 仅 keyring | 增加加密文件降级 | 高 |
| Windows 进程优雅关闭 | 未实现（占位符） | 完整实现 | 高 |
| 构建脚本跨平台 | 仅 Bash | Bash + PowerShell + Node.js | 中 |
| 条件编译优化 | 全平台资源嵌入 | 单平台构建只嵌入对应资源 | 中 |
| 安装路径策略 | 未明确 | 明确各平台默认路径 | 中 |

### 1.2 保持不变的优秀设计

- ✅ 离线包安装方案（简单、可靠）
- ✅ Tauri v2 架构升级
- ✅ 统一的错误处理体系
- ✅ 安全存储核心设计（macOS/Windows）

---

## 二、离线安装包改进方案

### 2.1 平台矩阵细化

```
支持平台（8个组合）:
├── macOS
│   ├── arm64  (Apple Silicon)
│   └── x64    (Intel)
├── Windows
│   ├── arm64  (Snapdragon X Elite, Surface Pro X)
│   └── x64    (Intel/AMD)
└── Linux
    ├── arm64  (ARM服务器, Asahi Linux)
    └── x64    (Intel/AMD)
```

### 2.2 条件编译优化

```rust
// src-tauri/src/services/installer.rs

impl OfflineInstaller {
    pub fn from_embedded() -> Result<Self, InstallError> {
        let (package_data, info) = match (std::env::consts::OS, std::env::consts::ARCH) {
            // macOS
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            ("macos", "aarch64") => {
                (include_bytes!("../../../bundled/openclaw-macos-arm64.tar.gz"),
                 PackageInfo { platform: Platform::MacOS, arch: Arch::ARM64, .. })
            }
            #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
            ("macos", "x86_64") => {
                (include_bytes!("../../../bundled/openclaw-macos-x64.tar.gz"),
                 PackageInfo { platform: Platform::MacOS, arch: Arch::X64, .. })
            }
            // Windows
            #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
            ("windows", "aarch64") => {
                (include_bytes!("../../../bundled/openclaw-windows-arm64.zip"),
                 PackageInfo { platform: Platform::Windows, arch: Arch::ARM64, .. })
            }
            #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
            ("windows", "x86_64") => {
                (include_bytes!("../../../bundled/openclaw-windows-x64.zip"),
                 PackageInfo { platform: Platform::Windows, arch: Arch::X64, .. })
            }
            // Linux
            #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
            ("linux", "aarch64") => {
                (include_bytes!("../../../bundled/openclaw-linux-arm64.tar.gz"),
                 PackageInfo { platform: Platform::Linux, arch: Arch::ARM64, .. })
            }
            #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
            ("linux", "x86_64") => {
                (include_bytes!("../../../bundled/openclaw-linux-x64.tar.gz"),
                 PackageInfo { platform: Platform::Linux, arch: Arch::X64, .. })
            }
            _ => return Err(InstallError::UnsupportedPlatform),
        };
        // ...
    }
}
```

**优势**：单平台构建时，其他平台的资源不会被嵌入，显著减少二进制体积。

### 2.3 安装路径策略

```rust
// src-tauri/src/utils/platform.rs

pub struct InstallPaths;

impl InstallPaths {
    /// 获取默认安装目录（用户级别，无需管理员权限）
    pub fn default_install_dir() -> Result<PathBuf, PlatformError> {
        match std::env::consts::OS {
            "macos" => {
                // ~/Applications/OpenClaw 或 ~/.local/bin/openclaw
                let home = dirs::home_dir().ok_or(PlatformError::HomeDirNotFound)?;
                Ok(home.join("Applications").join("OpenClaw"))
            }
            "windows" => {
                // %LOCALAPPDATA%\Programs\OpenClaw
                let local_app_data = dirs::data_local_dir().ok_or(PlatformError::LocalAppDataNotFound)?;
                Ok(local_app_data.join("Programs").join("OpenClaw"))
            }
            "linux" => {
                // ~/.local/bin/openclaw (符合 XDG 规范)
                let home = dirs::home_dir().ok_or(PlatformError::HomeDirNotFound)?;
                Ok(home.join(".local").join("bin").join("openclaw"))
            }
            _ => Err(PlatformError::UnsupportedPlatform),
        }
    }

    /// 获取系统级安装目录（需要管理员权限，可选）
    pub fn system_install_dir() -> Result<PathBuf, PlatformError> {
        match std::env::consts::OS {
            "macos" => Ok(PathBuf::from("/Applications/OpenClaw")),
            "windows" => Ok(PathBuf::from("C:\\Program Files\\OpenClaw")),
            "linux" => Ok(PathBuf::from("/opt/openclaw")),
            _ => Err(PlatformError::UnsupportedPlatform),
        }
    }
}
```

### 2.4 跨平台构建脚本

#### Node.js 版本（推荐）

```javascript
// scripts/build-offline-package.js

const fs = require('fs');
const path = require('path');
const https = require('https');

const PLATFORMS = [
    { name: 'macos-arm64', ext: 'tar.gz', arch: 'arm64' },
    { name: 'macos-x64', ext: 'tar.gz', arch: 'x64' },
    { name: 'windows-arm64', ext: 'zip', arch: 'arm64' },
    { name: 'windows-x64', ext: 'zip', arch: 'x64' },
    { name: 'linux-arm64', ext: 'tar.gz', arch: 'arm64' },
    { name: 'linux-x64', ext: 'tar.gz', arch: 'x64' },
];

const OUTPUT_DIR = 'bundled';
const VERSION = process.argv[2] || 'latest';

async function download(platform) {
    const filename = `openclaw-${VERSION}-${platform.name}.${platform.ext}`;
    const url = `https://github.com/openclaw/openclaw/releases/download/${VERSION}/${filename}`;
    const outputPath = path.join(OUTPUT_DIR, filename);

    console.log(`Downloading ${platform.name}...`);

    return new Promise((resolve, reject) => {
        const file = fs.createWriteStream(outputPath);
        https.get(url, (response) => {
            if (response.statusCode === 302 || response.statusCode === 301) {
                // Handle redirect
                https.get(response.headers.location, (redirectResponse) => {
                    redirectResponse.pipe(file);
                    file.on('finish', () => {
                        file.close();
                        console.log(`✓ ${filename}`);
                        resolve();
                    });
                }).on('error', reject);
            } else {
                response.pipe(file);
                file.on('finish', () => {
                    file.close();
                    console.log(`✓ ${filename}`);
                    resolve();
                });
            }
        }).on('error', reject);
    });
}

async function main() {
    if (!fs.existsSync(OUTPUT_DIR)) {
        fs.mkdirSync(OUTPUT_DIR, { recursive: true });
    }

    console.log(`Building offline packages for version: ${VERSION}\n`);

    for (const platform of PLATFORMS) {
        try {
            await download(platform);
        } catch (error) {
            console.error(`✗ Failed to download ${platform.name}:`, error.message);
        }
    }

    console.log('\nDone!');
}

main().catch(console.error);
```

#### PowerShell 版本（Windows 原生）

```powershell
# scripts/build-offline-package.ps1

param(
    [string]$Version = "latest"
)

$PLATFORMS = @(
    @{ Name = "macos-arm64"; Ext = "tar.gz" },
    @{ Name = "macos-x64"; Ext = "tar.gz" },
    @{ Name = "windows-arm64"; Ext = "zip" },
    @{ Name = "windows-x64"; Ext = "zip" },
    @{ Name = "linux-arm64"; Ext = "tar.gz" },
    @{ Name = "linux-x64"; Ext = "tar.gz" }
)

$OUTPUT_DIR = "bundled"

if (!(Test-Path $OUTPUT_DIR)) {
    New-Item -ItemType Directory -Path $OUTPUT_DIR | Out-Null
}

Write-Host "Building offline packages for version: $Version`n"

foreach ($platform in $PLATFORMS) {
    $filename = "openclaw-$Version-$($platform.Name).$($platform.Ext)"
    $url = "https://github.com/openclaw/openclaw/releases/download/$Version/$filename"
    $outputPath = Join-Path $OUTPUT_DIR $filename

    Write-Host "Downloading $($platform.Name)..." -NoNewline
    try {
        Invoke-WebRequest -Uri $url -OutFile $outputPath -ErrorAction Stop
        Write-Host " ✓" -ForegroundColor Green
    } catch {
        Write-Host " ✗ Failed" -ForegroundColor Red
    }
}

Write-Host "`nDone!"
```

---

## 三、安全存储改进方案

### 3.1 Linux 降级方案

```rust
// src-tauri/src/services/secure_storage.rs

use keyring::Entry;
use thiserror::Error;

const SERVICE_NAME: &str = "com.openclaw.manager";

#[derive(Error, Debug)]
pub enum SecureStorageError {
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("File storage error: {0}")]
    FileStorage(String),
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
}

/// 安全存储后端枚举
#[derive(Debug, Clone)]
pub enum StorageBackend {
    /// 系统密钥链（首选）
    Keychain,
    /// 加密文件存储（降级方案，用于 Linux 无 D-Bus 环境）
    EncryptedFile,
}

pub struct SecureStorage {
    backend: StorageBackend,
}

impl SecureStorage {
    /// 自动选择最佳后端
    pub fn auto() -> Result<Self, SecureStorageError> {
        // 测试 keyring 是否可用
        match Self::test_keyring() {
            Ok(_) => Ok(Self { backend: StorageBackend::Keychain }),
            Err(_) => {
                log::warn!("Keyring unavailable, falling back to encrypted file storage");
                Ok(Self { backend: StorageBackend::EncryptedFile })
            }
        }
    }

    /// 测试 keyring 是否可用
    fn test_keyring() -> Result<(), SecureStorageError> {
        let entry = Entry::new(SERVICE_NAME, "__test__")?;
        entry.set_password("test")?;
        entry.delete_password()?;
        Ok(())
    }

    pub fn save_api_key(&self, provider: &str, api_key: &str) -> Result<(), SecureStorageError> {
        match self.backend {
            StorageBackend::Keychain => {
                let entry = Entry::new(SERVICE_NAME, &format!("api_key_{}", provider))?;
                entry.set_password(api_key)?;
                Ok(())
            }
            StorageBackend::EncryptedFile => {
                self.save_to_encrypted_file(provider, api_key)
            }
        }
    }

    pub fn get_api_key(&self, provider: &str) -> Result<Option<String>, SecureStorageError> {
        match self.backend {
            StorageBackend::Keychain => {
                let entry = Entry::new(SERVICE_NAME, &format!("api_key_{}", provider))?;
                match entry.get_password() {
                    Ok(key) => Ok(Some(key)),
                    Err(keyring::Error::NoEntry) => Ok(None),
                    Err(e) => Err(e.into()),
                }
            }
            StorageBackend::EncryptedFile => {
                self.read_from_encrypted_file(provider)
            }
        }
    }

    /// 加密文件存储实现（Linux 降级方案）
    fn save_to_encrypted_file(&self, provider: &str, api_key: &str) -> Result<(), SecureStorageError> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };
        use sha2::{Sha256, Digest};

        let config_dir = dirs::config_dir()
            .ok_or_else(|| SecureStorageError::FileStorage("Config dir not found".into()))?
            .join("openclaw-manager");

        std::fs::create_dir_all(&config_dir)
            .map_err(|e| SecureStorageError::FileStorage(e.to_string()))?;

        // 基于机器 ID 生成加密密钥
        let machine_id = Self::get_machine_id()?;
        let key = Self::derive_key(&machine_id);

        // 加密
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| SecureStorageError::FileStorage(e.to_string()))?;
        let nonce = Self::generate_nonce();
        let ciphertext = cipher.encrypt(&nonce, api_key.as_bytes())
            .map_err(|e| SecureStorageError::FileStorage(e.to_string()))?;

        // 存储：nonce + ciphertext
        let mut data = nonce.to_vec();
        data.extend_from_slice(&ciphertext);

        let file_path = config_dir.join(format!("{}.enc", provider));
        std::fs::write(&file_path, data)
            .map_err(|e| SecureStorageError::FileStorage(e.to_string()))?;

        // 设置文件权限（仅所有者可读写）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&file_path)
                .map_err(|e| SecureStorageError::FileStorage(e.to_string()))?
                .permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&file_path, perms)
                .map_err(|e| SecureStorageError::FileStorage(e.to_string()))?;
        }

        Ok(())
    }

    /// 获取机器唯一标识
    fn get_machine_id() -> Result<String, SecureStorageError> {
        // 优先使用 /etc/machine-id (systemd)
        if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
            return Ok(id.trim().to_string());
        }
        // 降级到主机名 + 用户名组合
        let username = whoami::username();
        let hostname = hostname::get()
            .map_err(|e| SecureStorageError::FileStorage(e.to_string()))?
            .to_string_lossy()
            .to_string();
        Ok(format!("{}@{}", username, hostname))
    }

    fn derive_key(machine_id: &str) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(machine_id.as_bytes());
        hasher.update(SERVICE_NAME.as_bytes());
        hasher.finalize().into()
    }

    fn generate_nonce() -> [u8; 12] {
        use rand::RngCore;
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        nonce
    }
}
```

### 3.2 Linux 依赖文档

```markdown
## Linux 运行依赖

### 完整功能模式（推荐）
需要安装以下包以获得完整的密钥链支持：

- **Ubuntu/Debian**: `libsecret-1-0`, `gnome-keyring` (或 `kwallet` for KDE)
  ```bash
  sudo apt install libsecret-1-0 gnome-keyring
  ```

- **Fedora/RHEL**: `libsecret`, `gnome-keyring`
  ```bash
  sudo dnf install libsecret gnome-keyring
  ```

- **Arch**: `libsecret`, `gnome-keyring`
  ```bash
  sudo pacman -S libsecret gnome-keyring
  ```

### 最小化模式
如果系统没有 D-Bus 或密钥链服务，应用将自动降级到加密文件存储。
此模式下：
- ✅ API Key 仍会被加密存储
- ⚠️ 安全性略低于系统密钥链
- ⚠️ 更换硬件后需要重新配置

### WSL 注意事项
WSL 默认没有 D-Bus，建议：
1. 安装 `dbus-x11` 和 `gnome-keyring`
2. 或在 Windows 侧运行 OpenClaw Manager
```

---

## 四、进程管理改进方案

### 4.1 Windows 优雅关闭实现

```rust
// src-tauri/src/services/process_manager.rs

#[cfg(windows)]
mod windows_process {
    use std::process::Child;
    use windows_sys::Win32::System::Console::{GenerateConsoleCtrlEvent, CTRL_BREAK_EVENT};
    use windows_sys::Win32::Foundation::{BOOL, FALSE};

    /// Windows 优雅关闭进程
    pub fn graceful_shutdown(pid: u32, timeout_secs: u64) -> Result<(), ProcessError> {
        // 方案 1: 发送 Ctrl+Break 信号（如果进程是控制台应用）
        unsafe {
            // 将当前进程附加到目标进程的控制台
            if AttachConsole(pid) != FALSE {
                // 发送 Ctrl+Break (可以被程序捕获并优雅退出)
                if GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pid) != FALSE {
                    // 等待进程退出
                    if wait_for_exit(pid, timeout_secs) {
                        FreeConsole();
                        return Ok(());
                    }
                }
                FreeConsole();
            }
        }

        // 方案 2: 发送 WM_CLOSE 消息（如果是 GUI 应用）
        if send_wm_close(pid) {
            if wait_for_exit(pid, timeout_secs) {
                return Ok(());
            }
        }

        // 方案 3: 强制终止
        force_terminate(pid)
    }

    unsafe fn AttachConsole(pid: u32) -> BOOL {
        use windows_sys::Win32::System::Console::AttachConsole as AC;
        AC(pid)
    }

    unsafe fn FreeConsole() -> BOOL {
        use windows_sys::Win32::System::Console::FreeConsole as FC;
        FC()
    }

    fn send_wm_close(pid: u32) -> bool {
        use windows_sys::Win32::UI::WindowsAndMessaging::{EnumWindows, PostMessageW, WM_CLOSE};
        use windows_sys::Win32::Foundation::{LPARAM, HWND};
        use windows_sys::Win32::System::Threading::GetWindowThreadProcessId;

        struct EnumData {
            target_pid: u32,
            found: bool,
        }

        unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let data = &mut *(lparam as *mut EnumData);
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);

            if pid == data.target_pid {
                PostMessageW(hwnd, WM_CLOSE, 0, 0);
                data.found = true;
                // 继续枚举，因为可能有多个窗口
            }
            1 // TRUE - 继续枚举
        }

        let mut data = EnumData { target_pid: pid, found: false };
        unsafe {
            EnumWindows(Some(enum_callback), &mut data as *mut _ as LPARAM);
        }
        data.found
    }

    fn wait_for_exit(pid: u32, timeout_secs: u64) -> bool {
        use std::time::{Duration, Instant};
        use sysinfo::{System, SystemExt, ProcessExt};

        let start = Instant::now();
        let mut system = System::new_all();

        while start.elapsed() < Duration::from_secs(timeout_secs) {
            system.refresh_processes();
            if system.process(sysinfo::Pid::from(pid as usize)).is_none() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        false
    }

    fn force_terminate(pid: u32) -> Result<(), ProcessError> {
        use std::os::windows::process::CommandExt;
        use std::process::Command;

        // 使用 taskkill /F /T /PID
        let output = Command::new("taskkill")
            .args(&["/F", "/T", "/PID", &pid.to_string()])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output()
            .map_err(|e| ProcessError::TerminateFailed(e.to_string()))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(ProcessError::TerminateFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
    }
}
```

### 4.2 Unix 信号处理改进

```rust
// src-tauri/src/services/process_manager.rs

#[cfg(unix)]
mod unix_process {
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid;

    /// Unix 优雅关闭进程
    pub fn graceful_shutdown(pid: u32, timeout_secs: u64) -> Result<(), ProcessError> {
        let pid = Pid::from_raw(pid as i32);

        // 1. 发送 SIGTERM (可被捕获)
        signal::kill(pid, Signal::SIGTERM)
            .map_err(|e| ProcessError::SignalFailed(e.to_string()))?;

        // 2. 等待进程退出
        if wait_for_exit(pid, timeout_secs) {
            return Ok(());
        }

        // 3. 超时后发送 SIGKILL (强制终止)
        signal::kill(pid, Signal::SIGKILL)
            .map_err(|e| ProcessError::TerminateFailed(e.to_string()))?;

        Ok(())
    }

    fn wait_for_exit(pid: Pid, timeout_secs: u64) -> bool {
        use std::time::{Duration, Instant};
        use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
        use nix::errno::Errno;

        let start = Instant::now();

        while start.elapsed() < Duration::from_secs(timeout_secs) {
            match waitpid(pid, Some(WaitPidFlag::WNOHANG)) {
                Ok(WaitStatus::Exited(_, _)) | Ok(WaitStatus::Signaled(_, _, _)) => {
                    return true;
                }
                Ok(_) => {
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(Errno::ECHILD) => {
                    // 进程不存在或不是子进程，检查是否存在
                    return !process_exists(pid);
                }
                Err(_) => {
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
        false
    }

    fn process_exists(pid: Pid) -> bool {
        use nix::sys::signal::Signal;
        signal::kill(pid, None).is_ok()
    }
}
```

---

## 五、项目结构调整

```
openclaw-manager/
├── src/
│   ├── components/
│   │   ├── ui/                   # shadcn 基础组件
│   │   ├── layout/               # 布局组件
│   │   └── openclaw/             # OpenClaw 相关组件
│   ├── pages/
│   ├── hooks/
│   ├── stores/
│   ├── lib/
│   ├── types/
│   └── App.tsx
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── install.rs
│   │   │   ├── config.rs
│   │   │   ├── service.rs
│   │   │   ├── model.rs
│   │   │   ├── agent.rs
│   │   │   └── diagnose.rs
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── installer.rs      # 离线安装（v2: 条件编译优化）
│   │   │   ├── config_manager.rs
│   │   │   ├── process_manager.rs # v2: Windows优雅关闭实现
│   │   │   ├── secure_storage.rs  # v2: Linux降级方案
│   │   │   └── diagnostics.rs
│   │   ├── models/
│   │   ├── errors/
│   │   └── utils/
│   │       ├── mod.rs
│   │       ├── retry.rs
│   │       └── platform.rs        # v2: 新增平台路径工具
│   ├── capabilities/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── scripts/                       # v2: 跨平台构建脚本
│   ├── build-offline-package.js   # Node.js（跨平台推荐）
│   ├── build-offline-package.sh   # Bash（Unix）
│   ├── build-offline-package.ps1  # PowerShell（Windows）
│   └── install/
│       ├── macos/
│       ├── windows/
│       └── linux/
├── docs/
│   ├── MVP-ROADMAP.md            # v1 原始文档
│   ├── MVP-ROADMAP-v2.md         # 本文档
│   └── LINUX-DEPENDENCIES.md     # v2: Linux依赖说明
└── package.json
```

---

## 六、开发阶段规划（v2）

### Phase 0: 基础设施（1 天）

- [ ] **0.1** 创建跨平台构建脚本
  - [ ] 实现 `scripts/build-offline-package.js`
  - [ ] 实现 `scripts/build-offline-package.ps1`
  - [ ] 更新 `scripts/build-offline-package.sh`
  - [ ] 添加 Node.js 脚本到 package.json scripts

- [ ] **0.2** 创建平台工具模块
  - [ ] 创建 `src/utils/platform.rs`
  - [ ] 实现 `InstallPaths::default_install_dir()`
  - [ ] 实现 `InstallPaths::system_install_dir()`

### Phase 1: 离线安装优化（2 天）

- [ ] **1.1** 添加平台矩阵支持
  - [ ] 更新 `Platform` 和 `Arch` 枚举（增加 ARM64）
  - [ ] 更新 `from_embedded()` 使用条件编译
  - [ ] 测试各平台构建体积

- [ ] **1.2** 集成安装路径策略
  - [ ] 更新 `install_offline()` 使用默认路径
  - [ ] 添加自定义路径选项
  - [ ] 测试各平台路径权限

### Phase 2: 安全存储改进（2 天）

- [ ] **2.1** 实现 Linux 降级方案
  - [ ] 添加 `aes-gcm` 依赖
  - [ ] 实现 `StorageBackend` 枚举
  - [ ] 实现加密文件存储
  - [ ] 实现自动后端选择

- [ ] **2.2** 创建 Linux 依赖文档
  - [ ] 编写 `docs/LINUX-DEPENDENCIES.md`
  - [ ] 添加 WSL 说明

- [ ] **2.3** 测试安全存储
  - [ ] 测试 keyring 模式
  - [ ] 测试降级模式
  - [ ] 测试后端切换

### Phase 3: 进程管理完善（2 天）

- [ ] **3.1** 实现 Windows 优雅关闭
  - [ ] 添加 `windows-sys` 依赖
  - [ ] 实现 `windows_process::graceful_shutdown()`
  - [ ] 测试 Ctrl+Break 信号
  - [ ] 测试 WM_CLOSE 信号

- [ ] **3.2** 改进 Unix 信号处理
  - [ ] 完善 `unix_process::graceful_shutdown()`
  - [ ] 添加子进程等待逻辑
  - [ ] 测试 SIGTERM/SIGKILL 流程

- [ ] **3.3** 整合进程管理
  - [ ] 更新 `stop_service()` 调用平台特定实现
  - [ ] 添加权限处理（Windows）
  - [ ] 测试跨平台进程管理

### Phase 4: 集成测试（2 天）

- [ ] **4.1** 跨平台构建测试
  - [ ] macOS ARM64 构建
  - [ ] macOS x64 构建
  - [ ] Windows ARM64 构建
  - [ ] Windows x64 构建
  - [ ] Linux ARM64 构建
  - [ ] Linux x64 构建

- [ ] **4.2** 功能测试
  - [ ] 离线安装流程（各平台）
  - [ ] 安全存储（keyring + 降级）
  - [ ] 进程启动/停止

- [ ] **4.3** 文档更新
  - [ ] 更新 README.md
  - [ ] 更新 API 文档
  - [ ] 更新 CHANGELOG.md

---

## 七、技术依赖更新

### 7.1 Cargo.toml 新增依赖

```toml
[dependencies]
# 安全存储 - Linux 加密降级
aes-gcm = "0.10"
rand = "0.8"
sha2 = "0.10"
whoami = "1.4"
hostname = "0.3"

# Windows API
[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.52", features = [
    "Win32_System_Console",
    "Win32_System_Threading",
    "Win32_UI_WindowsAndMessaging",
    "Win32_Foundation",
] }

# Unix 信号
[target.'cfg(unix)'.dependencies]
nix = { version = "0.27", features = ["signal", "process"] }

# 目录工具（已存在，确认版本）
dirs = "5.0"
```

### 7.2 package.json 新增脚本

```json
{
  "scripts": {
    "build:offline-packages": "node scripts/build-offline-package.js",
    "build:offline-packages:ps": "powershell -ExecutionPolicy Bypass -File scripts/build-offline-package.ps1",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build"
  }
}
```

---

## 八、验收标准（v2）

### 8.1 功能完整性

- [ ] 支持 6 个平台组合（macOS/Windows/Linux × x64/ARM64）
- [ ] Linux 安全存储自动降级
- [ ] Windows 进程优雅关闭完整实现
- [ ] 单平台构建体积优化（仅嵌入对应资源）

### 8.2 兼容性

- [ ] macOS 10.15+ (Intel/Apple Silicon)
- [ ] Windows 10 1809+ / Windows 11 (x64/ARM64)
- [ ] Ubuntu 20.04+ / Debian 11+ / Fedora 35+ (x64/ARM64)
- [ ] WSL2 可用（降级模式）

### 8.3 文档

- [ ] Linux 依赖文档完整
- [ ] 跨平台构建脚本可用
- [ ] API 文档更新

---

## 九、风险与缓解（v2）

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| Windows ARM64 构建环境复杂 | 中 | 中 | 使用 GitHub Actions 自动化构建 |
| Linux 加密存储安全性争议 | 低 | 中 | 明确文档说明，建议生产环境使用 keyring |
| Windows API 变更 | 低 | 中 | 使用稳定的 windows-sys crate |
| 构建脚本跨平台差异 | 中 | 低 | 提供 Node.js 作为统一方案 |

---

## 十、与 v1 的对比总结

| 维度 | MVP v1 | MVP v2 | 改进 |
|------|--------|--------|------|
| 支持平台 | 4 个 | 6 个 | +Windows ARM64, +Linux ARM64 |
| Linux 安全存储 | keyring only | keyring + 加密降级 | 支持 WSL/最小化系统 |
| Windows 进程关闭 | 未实现 | 完整实现 | Ctrl+Break + WM_CLOSE + 强制终止 |
| 构建脚本 | Bash only | Bash + PS + Node.js | 真正的跨平台构建 |
| 构建体积 | 全平台资源 | 单平台资源 | 减少约 75% 单平台体积 |
| 安装路径 | 未明确 | 明确各平台默认 | 用户级安装，无需管理员 |

---

*本文档基于 MVP v1 的跨平台兼容性评估创建，重点解决 v1 中识别的平台细化和降级方案问题。*
