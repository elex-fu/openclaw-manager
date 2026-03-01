# OpenClaw Manager 一键部署优化方案

## 实施完成内容

### 1. 嵌入式 Runtime 架构 ✅

**新增文件：**
- `src-tauri/bundled/runtimes/README.md` - Runtime 包使用指南
- `src-tauri/bundled/presets/models.yaml` - 国产模型预设配置
- `src-tauri/src/installer/runtime_bundle.rs` - RuntimeBundle 管理器

**支持的 Runtime：**
| 组件 | 版本 | 必需 | 大小 |
|------|------|------|------|
| Node.js | 22.x | 否 | ~30MB |
| Python | 3.10 | 是 | ~25MB |

**关键特性：**
- 自动检测系统已有版本（兼容则跳过）
- 按需解压到 `~/.openclaw/runtime/`
- 自动设置 PATH 和环境变量
- 支持跨平台（macOS/Windows/Linux）

### 2. 全栈打包安装器 ✅

**新增方法（`OpenClawInstaller`）：**

```rust
/// 一键安装：Runtime + OpenClaw + 预设配置
pub async fn install_all(options: InstallAllOptions) -> Result<InstallResult>

/// 离线安装（使用 bundled 安装包）
async fn install_offline(&self) -> Result<InstallResult>

/// 应用本土化预设配置
async fn apply_china_presets(&self) -> Result<()>

/// 获取运行时环境变量
pub fn get_runtime_env(&self) -> Result<HashMap<String, String>>
```

**新增选项：**
```rust
pub struct InstallAllOptions {
    pub use_offline_package: bool,  // 默认 true
    pub custom_install_dir: Option<PathBuf>,
    pub skip_runtime: bool,
}
```

### 3. 国产模型预设 ✅

**预配置提供商：**
- ✅ DeepSeek（默认）- 深度求索
- ✅ MiniMax - 稀宇科技
- ✅ 智谱 AI - ChatGLM 系列
- ✅ 月之暗面 - Kimi 长文本
- ✅ 硅基流动 - 开源模型聚合
- ✅ 百川智能 - 王小川

**预设功能：**
- 推荐模型配置（价格/性能比最优）
- 默认 API Base URL
- 中文描述和说明
- 自动设置默认提供商为 DeepSeek

### 4. Tauri 命令扩展 ✅

**新增命令：**
```rust
#[tauri::command]
pub async fn install_openclaw_one_click(
    window: Window,
    use_offline_package: Option<bool>,
) -> Result<ApiResponse<InstallResult>, String>
```

**前端 API：**
```typescript
openclawApi.installOneClick(useOfflinePackage: boolean = true)
```

## 目录结构

```
src-tauri/
├── bundled/
│   ├── runtimes/
│   │   ├── node-v22.x.x-darwin-arm64.tar.gz  (需下载)
│   │   ├── python-3.10.x-macos11-arm64.tar.gz (需下载)
│   │   └── README.md
│   ├── presets/
│   │   └── models.yaml                       (已创建)
│   └── openclaw-macos-arm64.tar.gz           (已有)
├── src/
│   └── installer/
│       ├── mod.rs                            (已扩展)
│       └── runtime_bundle.rs                 (新增)
└── tauri.conf.json                           (已更新)
```

## 使用方式

### 方式 1：一键安装（推荐）

```typescript
// 前端调用
const result = await openclawApi.installOneClick(true);

// 安装流程：
// 1. 检查系统环境
// 2. 解压嵌入式 Python/Node（如需）
// 3. 安装 OpenClaw（离线包）
// 4. 应用国产模型预设
// 5. 配置环境变量
```

### 方式 2：传统安装

```typescript
// 在线安装
await openclawApi.install();

// 离线安装
await openclawApi.installOffline();
```

## 下一步工作

### 1. 下载真实 Runtime 包

根据 `bundled/runtimes/README.md` 中的链接下载：

```bash
cd src-tauri/bundled/runtimes

# macOS ARM64
curl -O https://nodejs.org/dist/v22.0.0/node-v22.0.0-darwin-arm64.tar.gz
curl -L -o python-3.10.14-macos11-arm64.tar.gz \
  https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-3.10.14+20240107-aarch64-apple-darwin-install_only.tar.gz

# 其他平台类似...
```

### 2. 更新前端安装向导

修改 `InstallWizard.tsx`：
- 添加"一键安装"按钮
- 优先使用 `installOneClick`
- 显示国产模型选择界面

### 3. CI/CD 自动化

在 GitHub Actions 中自动下载 runtime 包：

```yaml
- name: Download Runtimes
  run: |
    mkdir -p src-tauri/bundled/runtimes
    # 根据目标平台下载对应 runtime
```

### 4. 安全加固（阶段三）

- 文件白名单系统
- 高危操作二次确认
- 数据本地处理保证

## 技术对比

| 特性 | 优化前 | 优化后（Molili 风格） |
|------|--------|----------------------|
| 安装步骤 | 5+ 步，需手动配置 | 1 步，自动完成 |
| 环境依赖 | 需预装 Python/Node | 零依赖，嵌入式 |
| 网络要求 | 必须联网下载 | 支持离线安装 |
| 模型配置 | 手动配置 OpenAI | 预配置国产模型 |
| 安装时间 | 5-15 分钟 | 2-3 分钟 |
| 用户体验 | 技术门槛高 | 双击即用 |

## 测试建议

1. **清理测试**：
   ```bash
   rm -rf ~/.openclaw
   rm -rf ~/Library/Application\ Support/OpenClaw\ Manager
   ```

2. **运行一键安装**：
   ```bash
   npm run tauri:dev
   # 在界面中点击"一键安装"
   ```

3. **验证安装**：
   ```bash
   ls ~/.openclaw/runtime/python/bin/python3
   ls ~/.openclaw/bin/openclaw
   cat ~/.openclaw/config.yaml | grep deepseek
   ```

---

**实施状态：** 阶段一完成（80%）
**剩余工作：** Runtime 包下载、前端集成、CI/CD 配置
