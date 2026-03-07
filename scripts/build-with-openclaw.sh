#!/bin/bash
#
# OpenClaw Manager + OpenClaw 一键完整构建脚本
#

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
MANAGER_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "========================================"
echo "OpenClaw Manager + OpenClaw Full Build"
echo "========================================"
echo ""

# 步骤 1: 构建 OpenClaw
echo "📦 Step 1/3: Building OpenClaw from local source..."
echo ""
if [ ! -f "$SCRIPT_DIR/build-openclaw-local.sh" ]; then
    echo "❌ Error: build-openclaw-local.sh not found"
    exit 1
fi

"$SCRIPT_DIR/build-openclaw-local.sh"

# 步骤 2: 安装 Manager 依赖（如需要）
echo ""
echo "📦 Step 2/3: Checking OpenClaw Manager dependencies..."
cd "$MANAGER_DIR"

if [ ! -d "node_modules" ]; then
    echo "   Installing npm dependencies..."
    npm install
else
    echo "   Dependencies already installed"
fi

echo "   ✓ Dependencies ready"

# 步骤 3: 构建 Manager
echo ""
echo "🔨 Step 3/3: Building OpenClaw Manager..."
echo ""

# 设置环境变量禁用 sccache（如有问题）
export RUSTC_WRAPPER=""

# 执行构建
npm run tauri:build

echo ""
echo "========================================"
echo "✅ Full build complete!"
echo "========================================"
echo ""
echo "Output location:"
echo "  src-tauri/target/release/bundle/"
echo ""
echo "Build artifacts:"
find "$MANAGER_DIR/src-tauri/target/release/bundle" -type f \( \
    -name "*.dmg" -o \
    -name "*.app" -o \
    -name "*.msi" -o \
    -name "*.exe" -o \
    -name "*.deb" -o \
    -name "*.rpm" -o \
    -name "*.AppImage" \
) -exec ls -lh {} \; 2>/dev/null || echo "  (Check bundle directory for outputs)"

echo ""
echo "To run the application:"
echo "  macOS:   open src-tauri/target/release/bundle/dmg/*.dmg"
echo "  Linux:   sudo dpkg -i src-tauri/target/release/bundle/deb/*.deb"
echo "  Windows: Run the .msi or .exe installer"
echo ""
