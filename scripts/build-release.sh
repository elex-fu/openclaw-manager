#!/bin/bash
set -e

# OpenClaw Manager 发布构建脚本
# 使用方法: ./build-release.sh [version]

VERSION=${1:-"latest"}
PLATFORMS=("macos-x64" "macos-arm64" "windows-x64" "windows-arm64" "linux-x64" "linux-arm64")

echo "========================================"
echo "OpenClaw Manager Release Builder"
echo "Version: $VERSION"
echo "========================================"
echo

# 1. 下载离线安装包
echo "📦 Step 1: Downloading offline packages..."
npm run build:offline-packages -- --version "$VERSION"

# 2. 复制到 Tauri 资源目录
echo
echo "📂 Step 2: Copying packages to bundled directory..."
mkdir -p src-tauri/bundled
for platform in "${PLATFORMS[@]}"; do
    if [ -f "bundled/openclaw-${VERSION}-${platform}.tar.gz" ]; then
        cp "bundled/openclaw-${VERSION}-${platform}.tar.gz" src-tauri/bundled/
        echo "  ✓ ${platform}"
    elif [ -f "bundled/openclaw-${VERSION}-${platform}.zip" ]; then
        cp "bundled/openclaw-${VERSION}-${platform}.zip" src-tauri/bundled/
        echo "  ✓ ${platform}"
    fi
done

# 3. 构建前端
echo
echo "⚡ Step 3: Building frontend..."
npm run build

# 4. 构建 Tauri 应用
echo
echo "🔨 Step 4: Building Tauri application..."
cd src-tauri
cargo tauri build

# 5. 显示输出
echo
echo "========================================"
echo "✅ Build Complete!"
echo "========================================"
echo
echo "Output files:"
ls -lh target/release/bundle/*
echo
echo "To create a release, push a tag:"
echo "  git tag v$VERSION"
echo "  git push origin v$VERSION"
