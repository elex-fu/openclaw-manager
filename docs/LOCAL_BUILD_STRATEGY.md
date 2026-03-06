# OpenClaw + Manager 本地构建方案（最终版）

## 架构概览

```
┌─────────────────────────────────────────────────────────────────┐
│                   本地开发构建流程                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  /Users/lex/play/openclaw                                       │
│  ┌─────────────────┐                                            │
│  │  OpenClaw       │                                            │
│  │  (Node.js)      │                                            │
│  └────────┬────────┘                                            │
│           │ pnpm install + pnpm build                          │
│           ▼                                                     │
│  ┌─────────────────┐     ┌─────────────────┐                   │
│  │  dist/          │────→│  tar.gz         │                   │
│  │  assets/        │     │  (当前平台)      │                   │
│  │  package.json   │     └────────┬────────┘                   │
│  └─────────────────┘              │                            │
│                                    ▼                            │
│  /Users/lex/play/openclaw-manager                              │
│  ┌─────────────────┐                                            │
│  │  bundled/       │◄──────────────────────────────────────────┘
│  │  (嵌入目录)      │
│  └────────┬────────┘
│           │ cargo build
│           ▼
│  ┌─────────────────┐
│  │  Manager App    │
│  │  (含OpenClaw)   │
│  └─────────────────┘
│
└─────────────────────────────────────────────────────────────────┘
```

## 提供的脚本

### 1. build-openclaw-local.sh
- **作用**: 从 `/Users/lex/play/openclaw` 构建并打包
- **输出**: `src-tauri/bundled/openclaw-{platform}-{arch}.tar.gz`
- **用法**:
  ```bash
  ./scripts/build-openclaw-local.sh
  ```

### 2. build-with-openclaw.sh
- **作用**: 一键构建 OpenClaw + Manager
- **输出**: 完整的 Manager 安装包
- **用法**:
  ```bash
  ./scripts/build-with-openclaw.sh
  ```

## 快速开始

### 首次设置

```bash
# 1. 进入 Manager 目录
cd /Users/lex/play/openclaw-manager

# 2. 安装依赖
npm install

# 3. 构建 OpenClaw 并嵌入
./scripts/build-openclaw-local.sh
```

### 开发模式

```bash
# 启动开发服务器（带热更新）
npm run tauri:dev
```

### 生产构建

```bash
# 完整构建（OpenClaw + Manager）
./scripts/build-with-openclaw.sh

# 或者分步执行
./scripts/build-openclaw-local.sh
npm run tauri:build
```

## CI/CD 方案

### GitHub Actions 工作流

文件: `.github/workflows/build-with-local-openclaw.yml`

**策略**: 在 CI 中从 GitHub 检出 OpenClaw 源码构建

```yaml
触发条件:
  - push tags (v*)
  - workflow_dispatch (手动)

构建步骤:
  1. 检出 Manager
  2. 检出 OpenClaw (openclaw/openclaw)
  3. 构建 OpenClaw (pnpm install + build)
  4. 打包并嵌入
  5. 构建 Manager (6 平台)
  6. 创建 Release
```

**使用**:
```bash
# 本地推送到 GitHub 后触发
git add .
git commit -m "Prepare release"
git tag v0.1.0
git push origin v0.1.0
```

## 平台支持

| 平台 | 本地构建 | CI 构建 | 嵌入文件名 |
|------|---------|---------|-----------|
| macOS x64 | ✅ | ✅ | `openclaw-macos-x64.tar.gz` |
| macOS ARM64 | ✅ | ✅ | `openclaw-macos-arm64.tar.gz` |
| Windows x64 | ❌* | ✅ | `openclaw-windows-x64.zip` |
| Linux x64 | ✅ | ✅ | `openclaw-linux-x64.tar.gz` |
| Linux ARM64 | ❌ | ✅ | `openclaw-linux-arm64.tar.gz` |

*Windows 本地构建需在 Windows 环境运行脚本

## 文件结构

```
openclaw-manager/
├── scripts/
│   ├── build-openclaw-local.sh      # 构建 OpenClaw（本地）
│   └── build-with-openclaw.sh       # 一键完整构建
├── src-tauri/
│   ├── bundled/                     # OpenClaw 包存放（gitignore）
│   └── ...
├── docs/
│   ├── LOCAL_BUILD_STRATEGY.md      # 本文档
│   └── OPENCLAW_BUILD_GUIDE.md      # 详细指南
└── .github/workflows/
    ├── build-release.yml            # 标准发布（假设有 OpenClaw release）
    └── build-with-local-openclaw.yml # 本地源码构建
```

## 注意事项

1. **Node.js 版本**: OpenClaw 需要 Node.js 20+
2. **pnpm**: 确保已安装 `npm install -g pnpm`
3. **构建时间**: OpenClaw 构建约 1-3 分钟，Manager 构建约 5-10 分钟
4. **包大小**: OpenClaw 包含 node_modules，包可能较大（50-100MB）
5. **平台限制**: 本地只能构建当前平台，跨平台需用 CI

## 故障排除

### OpenClaw 构建失败

```bash
cd /Users/lex/play/openclaw
rm -rf node_modules dist
pnpm install
pnpm build
```

### Manager 找不到包

检查 `src-tauri/bundled/` 目录：
```bash
ls -la src-tauri/bundled/
```

文件名必须与 `offline_installer.rs` 中定义的匹配。

### CI 构建失败

1. 确保 GitHub Token 有权限访问 OpenClaw 仓库
2. 检查 OpenClaw 是否已推送到 GitHub
3. 查看 Actions 日志获取详细错误

## 版本管理

### 本地开发
- Manager 和 OpenClaw 版本独立
- 通过 Git 子模块或相对路径管理

### CI/CD
- 从 GitHub 检出 OpenClaw 最新代码
- 每次构建都是最新版本
- 如需特定版本，修改 workflow 中的 checkout ref

## 下一步

1. ✅ 运行 `./scripts/build-openclaw-local.sh` 测试构建
2. ✅ 运行 `npm run tauri:dev` 验证开发模式
3. ✅ 推送代码到 GitHub
4. ✅ 打 tag 触发 CI 构建: `git tag v0.1.0 && git push origin v0.1.0`
