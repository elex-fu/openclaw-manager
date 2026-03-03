#!/bin/bash
# 将本地 OpenClaw 项目打包成二进制安装包

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/.."
BUNDLED_DIR="$PROJECT_ROOT/src-tauri/bundled"
OPENCLAW_SOURCE="${OPENCLAW_SOURCE:-/Users/lex/play/openclaw}"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

# 显示帮助
show_help() {
    echo "OpenClaw 二进制打包脚本"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -s, --source DIR    OpenClaw 源码目录 (默认: /Users/lex/play/openclaw)"
    echo "  -o, --output DIR    输出目录 (默认: bundled/)"
    echo "  -p, --platform      目标平台 (macos|linux|windows)"
    echo "  -a, --arch          目标架构 (arm64|x64)"
    echo "  -h, --help          显示帮助"
    echo ""
    echo "Environment Variables:"
    echo "  OPENCLAW_SOURCE     OpenClaw 源码目录"
}

# 解析参数
PLATFORM=""
ARCH=""
while [[ $# -gt 0 ]]; do
    case $1 in
        -s|--source)
            OPENCLAW_SOURCE="$2"
            shift 2
            ;;
        -o|--output)
            BUNDLED_DIR="$2"
            shift 2
            ;;
        -p|--platform)
            PLATFORM="$2"
            shift 2
            ;;
        -a|--arch)
            ARCH="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            log_error "未知参数: $1"
            show_help
            exit 1
            ;;
    esac
done

# 检测平台
if [ -z "$PLATFORM" ]; then
    case "$(uname -s)" in
        Darwin) PLATFORM="macos" ;;
        Linux) PLATFORM="linux" ;;
        MINGW*|MSYS*|CYGWIN*) PLATFORM="windows" ;;
        *) log_error "不支持的平台: $(uname -s)"; exit 1 ;;
    esac
fi

if [ -z "$ARCH" ]; then
    case "$(uname -m)" in
        x86_64|amd64) ARCH="x64" ;;
        arm64|aarch64) ARCH="arm64" ;;
        *) log_error "不支持的架构: $(uname -m)"; exit 1 ;;
    esac
fi

log_info "OpenClaw 二进制打包"
log_info "===================="
echo ""
log_info "源码目录: $OPENCLAW_SOURCE"
log_info "目标平台: $PLATFORM-$ARCH"
log_info "输出目录: $BUNDLED_DIR"
echo ""

# 检查源码目录
if [ ! -d "$OPENCLAW_SOURCE" ]; then
    log_error "OpenClaw 源码目录不存在: $OPENCLAW_SOURCE"
    log_info "请设置 OPENCLAW_SOURCE 环境变量或克隆仓库:"
    log_info "  git clone https://github.com/openclaw/openclaw.git /path/to/openclaw"
    exit 1
fi

# 检查必要的文件
if [ ! -f "$OPENCLAW_SOURCE/package.json" ]; then
    log_error "无效的 OpenClaw 目录: 找不到 package.json"
    exit 1
fi

if [ ! -d "$OPENCLAW_SOURCE/dist" ]; then
    log_warn "找不到 dist 目录，可能需要构建项目"
    log_info "在 OpenClaw 目录运行: npm run build"
fi

# 创建打包目录
PACK_DIR=$(mktemp -d)
mkdir -p "$PACK_DIR/openclaw/bin"
mkdir -p "$PACK_DIR/openclaw/dist"
mkdir -p "$PACK_DIR/openclaw/skills"
mkdir -p "$PACK_DIR/openclaw/extensions"

log_step "复制 OpenClaw 文件..."

# 复制核心文件
cp "$OPENCLAW_SOURCE/package.json" "$PACK_DIR/openclaw/"
cp "$OPENCLAW_SOURCE/openclaw.mjs" "$PACK_DIR/openclaw/"

# 复制 dist 目录
if [ -d "$OPENCLAW_SOURCE/dist" ]; then
    cp -r "$OPENCLAW_SOURCE/dist"/* "$PACK_DIR/openclaw/dist/"
    log_info "✓ 复制 dist 目录"
fi

# 复制 skills (排除 node_modules)
if [ -d "$OPENCLAW_SOURCE/skills" ]; then
    rsync -a --exclude='node_modules' "$OPENCLAW_SOURCE/skills/" "$PACK_DIR/openclaw/skills/" 2>/dev/null || \
        cp -r "$OPENCLAW_SOURCE/skills"/* "$PACK_DIR/openclaw/skills/" 2>/dev/null || true
    log_info "✓ 复制 skills 目录"
fi

# 复制 extensions (排除 node_modules)
if [ -d "$OPENCLAW_SOURCE/extensions" ]; then
    rsync -a --exclude='node_modules' "$OPENCLAW_SOURCE/extensions/" "$PACK_DIR/openclaw/extensions/" 2>/dev/null || \
        find "$OPENCLAW_SOURCE/extensions" -type d -name 'node_modules' -prune -o -type f -exec cp --parents {} "$PACK_DIR/openclaw/extensions/" \; 2>/dev/null || true
    log_info "✓ 复制 extensions 目录 (排除 node_modules)"
fi

# 复制 assets
if [ -d "$OPENCLAW_SOURCE/assets" ]; then
    cp -r "$OPENCLAW_SOURCE/assets" "$PACK_DIR/openclaw/"
    log_info "✓ 复制 assets 目录"
fi

# 复制其他必要文件
for file in README.md LICENSE CHANGELOG.md; do
    if [ -f "$OPENCLAW_SOURCE/$file" ]; then
        cp "$OPENCLAW_SOURCE/$file" "$PACK_DIR/openclaw/"
    fi
done

# 创建启动脚本
log_step "创建启动脚本..."

case "$PLATFORM" in
    macos|linux)
        cat > "$PACK_DIR/openclaw/bin/openclaw" << 'SCRIPT_EOF'
#!/bin/bash
# OpenClaw 启动脚本

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OPENCLAW_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# 使用系统 Node.js 或嵌入的 Node.js
if command -v node &> /dev/null; then
    NODE_CMD="node"
else
    # 尝试使用嵌入的 Node.js
    if [ -f "$OPENCLAW_DIR/../runtime/node/bin/node" ]; then
        NODE_CMD="$OPENCLAW_DIR/../runtime/node/bin/node"
    else
        echo "Error: Node.js not found. Please install Node.js 22 or use OpenClaw Manager to install."
        exit 1
    fi
fi

# 运行 OpenClaw
exec "$NODE_CMD" "$OPENCLAW_DIR/openclaw.mjs" "$@"
SCRIPT_EOF
        chmod +x "$PACK_DIR/openclaw/bin/openclaw"
        ;;

    windows)
        cat > "$PACK_DIR/openclaw/bin/openclaw.bat" << 'SCRIPT_EOF'
@echo off
setlocal enabledelayedexpansion

set "OPENCLAW_DIR=%~dp0.."

where node >nul 2>nul
if %errorlevel% == 0 (
    set "NODE_CMD=node"
) else (
    if exist "%OPENCLAW_DIR%\..\runtime\node\node.exe" (
        set "NODE_CMD=%OPENCLAW_DIR%\..\runtime\node\node.exe"
    ) else (
        echo Error: Node.js not found. Please install Node.js 22 or use OpenClaw Manager to install.
        exit /b 1
    )
)

"%NODE_CMD%" "%OPENCLAW_DIR%\openclaw.mjs" %*
SCRIPT_EOF
        ;;
esac

# 获取版本
VERSION=$(cd "$OPENCLAW_SOURCE" && node -p "require('./package.json').version" 2>/dev/null || echo "unknown")
log_info "OpenClaw 版本: $VERSION"

# 创建版本文件
echo "$VERSION" > "$PACK_DIR/openclaw/VERSION"

# 打包
log_step "打包..."

mkdir -p "$BUNDLED_DIR"
OUTPUT_NAME="openclaw-${PLATFORM}-${ARCH}"

case "$PLATFORM" in
    windows)
        (cd "$PACK_DIR" && zip -r "$BUNDLED_DIR/${OUTPUT_NAME}.zip" openclaw/)
        log_info "✓ 创建: ${OUTPUT_NAME}.zip"
        ;;
    *)
        (cd "$PACK_DIR" && tar -czf "$BUNDLED_DIR/${OUTPUT_NAME}.tar.gz" openclaw/)
        log_info "✓ 创建: ${OUTPUT_NAME}.tar.gz"
        ;;
esac

# 显示文件大小
FILE_SIZE=$(du -h "$BUNDLED_DIR/${OUTPUT_NAME}".* | cut -f1)
log_info "包大小: $FILE_SIZE"

# 清理
rm -rf "$PACK_DIR"

echo ""
log_info "打包完成！"
log_info "输出文件: $BUNDLED_DIR/${OUTPUT_NAME}.tar.gz (或 .zip)"
log_info ""
log_info "安装包结构:"
log_info "  openclaw/"
log_info "    ├── bin/openclaw"
log_info "    ├── dist/"
log_info "    ├── skills/"
log_info "    ├── extensions/"
log_info "    ├── assets/"
log_info "    ├── package.json"
log_info "    └── openclaw.mjs"
