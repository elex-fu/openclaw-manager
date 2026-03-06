#!/bin/bash
#
# OpenClaw 本地构建脚本
# 从 /Users/lex/play/openclaw 构建并打包，嵌入到 Manager
#

set -e

OPENCLAW_DIR="/Users/lex/play/openclaw"
MANAGER_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BUILD_DIR="/tmp/openclaw-build-$$"
FILENAME=""
PLATFORM=""
ARCH=""

echo "========================================"
echo "Building OpenClaw (Local Source)"
echo "========================================"
echo "OpenClaw source: $OPENCLAW_DIR"
echo "Manager target:  $MANAGER_DIR"
echo ""

# 检查 OpenClaw 目录
if [ ! -d "$OPENCLAW_DIR" ]; then
    echo "❌ Error: OpenClaw directory not found at $OPENCLAW_DIR"
    exit 1
fi

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
        *) echo "❌ Unsupported OS: $OS"; exit 1 ;;
    esac

    echo "📍 Platform: $PLATFORM-$ARCH"
    echo ""
}

# 构建 OpenClaw
build_openclaw() {
    echo "📦 Step 1: Building OpenClaw..."
    cd "$OPENCLAW_DIR"

    # 检查 pnpm
    if ! command -v pnpm &> /dev/null; then
        echo "❌ Error: pnpm not found"
        echo "   Install with: npm install -g pnpm"
        exit 1
    fi

    # 安装依赖
    if [ ! -d "node_modules" ]; then
        echo "   Installing dependencies..."
        pnpm install
    else
        echo "   Dependencies already installed"
    fi

    # 构建
    echo "   Running pnpm build..."
    pnpm build

    if [ ! -d "dist" ]; then
        echo "❌ Error: Build failed - dist directory not found"
        exit 1
    fi

    echo "   ✓ Build complete"
}

# 打包
package_openclaw() {
    echo ""
    echo "📂 Step 2: Packaging OpenClaw..."

    mkdir -p "$BUILD_DIR/openclaw"
    cd "$OPENCLAW_DIR"

    # 复制必要文件
    echo "   Copying build artifacts..."
    cp -r dist "$BUILD_DIR/openclaw/"
    cp package.json "$BUILD_DIR/openclaw/"
    cp -r assets "$BUILD_DIR/openclaw/" 2>/dev/null || true
    cp -r extensions "$BUILD_DIR/openclaw/" 2>/dev/null || true
    cp -r skills "$BUILD_DIR/openclaw/" 2>/dev/null || true

    # 复制启动脚本
    if [ -f "openclaw.mjs" ]; then
        cp openclaw.mjs "$BUILD_DIR/openclaw/"
    fi

    # 创建启动脚本
    cat > "$BUILD_DIR/openclaw/start.sh" << 'EOF'
#!/bin/bash
# OpenClaw startup script
DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"
export NODE_PATH="$DIR/node_modules"
exec node dist/index.js "$@"
EOF
    chmod +x "$BUILD_DIR/openclaw/start.sh"

    # Windows 启动脚本
    cat > "$BUILD_DIR/openclaw/start.bat" << 'EOF'
@echo off
setlocal
set "DIR=%~dp0"
cd /d "%DIR%"
set "NODE_PATH=%DIR%\node_modules"
node dist/index.js %*
EOF

    # 打包
    cd "$BUILD_DIR"

    if [ "$PLATFORM" = "windows" ]; then
        FILENAME="openclaw-${PLATFORM}-${ARCH}.zip"
        if command -v zip &> /dev/null; then
            zip -r "$FILENAME" openclaw/ -x "*.DS_Store"
        else
            echo "⚠️  zip not found, using tar.gz instead"
            FILENAME="openclaw-${PLATFORM}-${ARCH}.tar.gz"
            tar -czf "$FILENAME" openclaw/ --exclude="*.DS_Store"
        fi
    else
        FILENAME="openclaw-${PLATFORM}-${ARCH}.tar.gz"
        tar -czf "$FILENAME" openclaw/ --exclude="*.DS_Store"
    fi

    echo "   ✓ Packaged: $FILENAME"
    ls -lh "$FILENAME"
}

# 复制到 Manager
copy_to_manager() {
    echo ""
    echo "📋 Step 3: Copying to OpenClaw Manager..."

    mkdir -p "$MANAGER_DIR/src-tauri/bundled"
    cp "$BUILD_DIR/$FILENAME" "$MANAGER_DIR/src-tauri/bundled/"

    echo "   ✓ Copied to src-tauri/bundled/$FILENAME"
    echo ""
    echo "   Bundled directory contents:"
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
    echo ""
    echo "  Development:"
    echo "    npm run tauri:dev"
    echo ""
    echo "  Production build:"
    echo "    npm run tauri:build"
    echo ""
    echo "  Or use the combined script:"
    echo "    ./scripts/build-with-openclaw.sh"
    echo ""
}

main "$@"
