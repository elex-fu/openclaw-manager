# OpenClaw 本地构建打包方案

## 概述

本文档描述如何从本地 `/Users/lex/play/openclaw` 构建 OpenClaw，并打包到 OpenClaw Manager 中使用。

## 架构设计

```
┌─────────────────────────────────────────────────────────────────┐
│                    本地开发构建流程                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  /Users/lex/play/openclaw                                       │
│  ┌─────────────────┐                                            │
│  │  OpenClaw 源码  │                                            │
│  │  (Node.js)      │                                            │
│  └────────┬────────┘                                            │
│           │ pnpm build                                          │
│           ▼                                                     │
│  ┌─────────────────┐     ┌─────────────────┐                   │
│  │  dist/          │────→│  tar.gz/zip     │                   │
│  │  (构建输出)      │     │  (平台特定包)    │                   │
│  └─────────────────┘     └────────┬────────┘                   │
│                                    │                            │
│                                    ▼                            │
│  /Users/lex/play/openclaw-manager                              │
│  ┌─────────────────┐     ┌─────────────────┐                   │
│  │  bundled/       │◄────│  copy           │                   │
│  │  (嵌入目录)      │     │                 │                   │
│  └─────────────────┘     └─────────────────┘                   │
│           │                                                      │
│           ▼                                                      │
│  ┌─────────────────┐                                            │
│  │  npm run        │                                            │
│  │  tauri:build    │                                            │
│  └─────────────────┘                                            │
│           │                                                      │
│           ▼                                                      │
│  ┌─────────────────┐                                            │
│  │  Release App    │                                            │
│  │  (含OpenClaw)   │                                            │
│  └─────────────────┘                                            │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## 方案详情

### 方案特点

| 特性 | 说明 |
|------|------|
| 本地源码 | 使用 `/Users/lex/play/openclaw` 本地代码 |
| 单平台构建 | 只构建当前运行平台（快速） |
| 自动嵌入 | 构建完成后自动复制到 bundled/ |
| 开发友好 | 支持热更新，调试方便 |

### 平台支持

| 平台 | 构建输出 | 嵌入文件名 |
|------|---------|-----------|
| macOS x64 | `openclaw-macos-x64.tar.gz` | `openclaw-macos-x64.tar.gz` |
| macOS ARM64 | `openclaw-macos-arm64.tar.gz` | `openclaw-macos-arm64.tar.gz` |
| Windows x64 | `openclaw-windows-x64.zip` | `openclaw-windows-x64.zip` |
| Linux x64 | `openclaw-linux-x64.tar.gz` | `openclaw-linux-x64.tar.gz` |

## 快速开始

### 1. 一键构建（推荐）

```bash
# 在 openclaw-manager 目录下
./scripts/build-with-openclaw.sh
```

### 2. 分步构建

```bash
# 步骤 1: 构建 OpenClaw
./scripts/build-openclaw-local.sh

# 步骤 2: 构建 Manager
npm run tauri:build
```

### 3. 开发模式（带热更新）

```bash
# 先构建并嵌入 OpenClaw
./scripts/build-openclaw-local.sh

# 启动开发服务器
npm run tauri:dev
```

## 详细步骤

### 准备工作

1. **确保 OpenClaw 依赖已安装**
   ```bash
   cd /Users/lex/play/openclaw
   pnpm install
   ```

2. **确保 Manager 依赖已安装**
   ```bash
   cd /Users/lex/play/openclaw-manager
   npm install
   ```

### 构建 OpenClaw

```bash
cd /Users/lex/play/openclaw

# 安装依赖（如未安装）
pnpm install

# 构建项目
pnpm build

# 输出目录: dist/
```

### 打包 OpenClaw

```bash
# 创建临时目录
mkdir -p /tmp/openclaw-package/openclaw
cd /tmp/openclaw-package

# 复制构建输出
cp -r /Users/lex/play/openclaw/dist openclaw/
cp /Users/lex/play/openclaw/package.json openclaw/
cp /Users/lex/play/openclaw/README.md openclaw/ 2>/dev/null || true

# 复制关键依赖（生产依赖）
mkdir -p openclaw/node_modules
cd /Users/lex/play/openclaw
pnpm install --prod --modules-dir /tmp/openclaw-package/openclaw/node_modules

# 打包
cd /tmp/openclaw-package

# macOS/Linux
tar -czf openclaw-macos-$(uname -m).tar.gz openclaw/
# 或
tar -czf openclaw-linux-$(uname -m).tar.gz openclaw/

# Windows (在 Windows 上)
# zip -r openclaw-windows-x64.zip openclaw/
```

### 嵌入到 Manager

```bash
# 创建 bundled 目录
mkdir -p /Users/lex/play/openclaw-manager/src-tauri/bundled

# 复制打包文件
cp /tmp/openclaw-package/openclaw-*.tar.gz \
   /Users/lex/play/openclaw-manager/src-tauri/bundled/

# 验证
ls -la /Users/lex/play/openclaw-manager/src-tauri/bundled/
```

### 构建 Manager

```bash
cd /Users/lex/play/openclaw-manager

# 开发模式
npm run tauri:dev

# 生产构建
npm run tauri:build
```

## 自动化脚本

### 脚本 1: 构建 OpenClaw（本地）

```bash
#!/bin/bash
# scripts/build-openclaw-local.sh

set -e

OPENCLAW_DIR="/Users/lex/play/openclaw"
MANAGER_DIR="/Users/lex/play/openclaw-manager"
BUILD_DIR="/tmp/openclaw-build-$$"

echo "========================================"
echo "Building OpenClaw (Local)"
echo "========================================"
echo ""

# 检测平台
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$ARCH" in
        x86_64)  ARCH="x64" ;;
        arm64|aarch64) ARCH="arm64" ;;
    esac

    case "$OS" in
        darwin) PLATFORM="macos" ;;
        linux)  PLATFORM="linux" ;;
        mingw*|msys*|cygwin*) PLATFORM="windows" ;;
        *) echo "Unsupported OS: $OS"; exit 1 ;;
    esac

    echo "Platform: $PLATFORM-$ARCH"
}

# 构建 OpenClaw
build_openclaw() {
    echo "📦 Building OpenClaw..."
    cd "$OPENCLAW_DIR"

    # 检查 pnpm
    if ! command -v pnpm &> /dev/null; then
        echo "Error: pnpm not found. Install with: npm install -g pnpm"
        exit 1
    fi

    # 安装依赖
    if [ ! -d "node_modules" ]; then
        echo "  Installing dependencies..."
        pnpm install
    fi

    # 构建
    echo "  Running pnpm build..."
    pnpm build

    echo "  ✓ Build complete"
}

# 打包
package_openclaw() {
    echo ""
    echo "📂 Packaging OpenClaw..."

    mkdir -p "$BUILD_DIR/openclaw"
    cd "$OPENCLAW_DIR"

    # 复制必要文件
    cp -r dist "$BUILD_DIR/openclaw/"
    cp package.json "$BUILD_DIR/openclaw/"
    cp -r assets "$BUILD_DIR/openclaw/" 2>/dev/null || true
    cp -r extensions "$BUILD_DIR/openclaw/" 2>/dev/null || true
    cp -r skills "$BUILD_DIR/openclaw/" 2>/dev/null || true

    # 安装生产依赖
    echo "  Installing production dependencies..."
    pnpm install --prod --modules-dir "$BUILD_DIR/openclaw/node_modules"

    # 打包
    cd "$BUILD_DIR"

    if [ "$PLATFORM" = "windows" ]; then
        # Windows 用 zip
        FILENAME="openclaw-${PLATFORM}-${ARCH}.zip"
        zip -r "$FILENAME" openclaw/
    else
        # macOS/Linux 用 tar.gz
        FILENAME="openclaw-${PLATFORM}-${ARCH}.tar.gz"
        tar -czf "$FILENAME" openclaw/
    fi

    echo "  ✓ Packaged: $FILENAME"
}

# 复制到 Manager
copy_to_manager() {
    echo ""
    echo "📋 Copying to Manager..."

    mkdir -p "$MANAGER_DIR/src-tauri/bundled"
    cp "$BUILD_DIR/$FILENAME" "$MANAGER_DIR/src-tauri/bundled/"

    echo "  ✓ Copied to src-tauri/bundled/$FILENAME"
    echo ""
    ls -lh "$MANAGER_DIR/src-tauri/bundled/"
}

# 清理
cleanup() {
    rm -rf "$BUILD_DIR"
}

# 主流程
main() {
    detect_platform
    build_openclaw
    package_openclaw
    copy_to_manager
    cleanup

    echo ""
    echo "========================================"
    echo "✅ OpenClaw build complete!"
    echo "========================================"
    echo ""
    echo "Next steps:"
    echo "  1. Run: npm run tauri:dev     (development)"
    echo "  2. Run: npm run tauri:build   (production)"
}

main
```

### 脚本 2: 一键完整构建

```bash
#!/bin/bash
# scripts/build-with-openclaw.sh

set -e

echo "========================================"
echo "OpenClaw Manager + OpenClaw Full Build"
echo "========================================"
echo ""

# 步骤 1: 构建 OpenClaw
echo "Step 1/2: Building OpenClaw..."
./scripts/build-openclaw-local.sh

# 步骤 2: 构建 Manager
echo ""
echo "Step 2/2: Building OpenClaw Manager..."
npm run tauri:build

echo ""
echo "========================================"
echo "✅ Full build complete!"
echo "========================================"
echo ""
echo "Output: src-tauri/target/release/bundle/"
ls -lh src-tauri/target/release/bundle/
```

## CI/CD 方案（GitHub Actions）

对于 CI/CD，由于无法访问本地路径，需要采用不同的策略：

### 方案 A: 子模块方式

将 OpenClaw 作为子模块：

```bash
cd /Users/lex/play/openclaw-manager
git submodule add /Users/lex/play/openclaw third-party/openclaw
git submodule update --init
```

然后 GitHub Actions 可以检出子模块并构建。

### 方案 B: 发布到 GitHub

将 OpenClaw 发布到 GitHub Releases，Manager 下载：

```bash
# 在 openclaw 目录
git tag v2026.2.27
git push origin v2026.2.27
# 触发 GitHub Actions 构建并发布
```

## 注意事项

1. **Node.js 版本**: 确保 OpenClaw 和 Manager 使用兼容的 Node.js 版本
2. **依赖体积**: OpenClaw 的 node_modules 可能很大，考虑精简
3. **平台差异**: Windows 构建需要在 Windows 环境或使用 cross-compilation
4. **版本同步**: 建议记录 OpenClaw 的 commit hash 或版本号

## 故障排除

### 问题 1: OpenClaw 构建失败

```bash
# 清理并重新安装
cd /Users/lex/play/openclaw
rm -rf node_modules dist
pnpm install
pnpm build
```

### 问题 2: 嵌入后 Manager 找不到包

检查文件名是否匹配：
- `offline_installer.rs` 中期望的文件名
- `src-tauri/bundled/` 中实际的文件名

### 问题 3: 运行时 OpenClaw 启动失败

检查：
- node_modules 是否正确包含
- 环境变量是否设置
- 日志输出

## 相关文件

- `scripts/build-openclaw-local.sh` - 构建 OpenClaw 本地脚本
- `scripts/build-with-openclaw.sh` - 一键完整构建
- `src-tauri/src/services/offline_installer.rs` - 安装包处理
