#!/bin/bash
# 下载或创建 OpenClaw 二进制文件用于离线安装

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUNDLED_DIR="$SCRIPT_DIR/../src-tauri/bundled"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 确保目录存在
mkdir -p "$BUNDLED_DIR"

# OpenClaw GitHub releases 地址
# 注意：这里使用示例URL，实际使用时需要替换为真实的OpenClaw下载地址
GITHUB_REPO="openclai/openclaw"

echo "=== OpenClaw 离线安装包下载/构建脚本 ==="
echo ""

# 检测当前平台
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    linux)
        PLATFORM="linux"
        ;;
    darwin)
        PLATFORM="macos"
        ;;
    mingw*|msys*|cygwin*)
        PLATFORM="windows"
        ;;
    *)
        echo "不支持的平台: $OS"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        ARCH_NAME="x64"
        ;;
    arm64|aarch64)
        ARCH_NAME="arm64"
        ;;
    *)
        echo "不支持的架构: $ARCH"
        exit 1
        ;;
esac

echo "检测到平台: $PLATFORM, 架构: $ARCH_NAME"
echo ""

# 由于 OpenClaw 可能还没有正式发布，我们创建一个示例安装包
echo "注意：目前 OpenClaw 还没有官方发布版本。"
echo "你可以通过以下方式获取二进制文件："
echo ""
echo "1. 从源码编译:"
echo "   git clone https://github.com/openclai/openclaw.git"
echo "   cd openclaw"
echo "   cargo build --release"
echo ""
echo "2. 手动下载二进制文件并放入 bundled 目录"
echo ""
echo "3. 使用在线安装方式"
echo ""

# 检查是否已有真实的安装包（通过检查文件大小和内容）
if [ -f "$BUNDLED_DIR/openclaw-${PLATFORM}-${ARCH_NAME}.tar.gz" ]; then
    FILE_SIZE=$(stat -f%z "$BUNDLED_DIR/openclaw-${PLATFORM}-${ARCH_NAME}.tar.gz" 2>/dev/null || stat -c%s "$BUNDLED_DIR/openclaw-${PLATFORM}-${ARCH_NAME}.tar.gz" 2>/dev/null || echo "0")
    if [ "$FILE_SIZE" -gt 1000 ]; then
        # 检查是否为占位符
        if ! head -c 100 "$BUNDLED_DIR/openclaw-${PLATFORM}-${ARCH_NAME}.tar.gz" | grep -q "Placeholder"; then
            echo "✓ 找到已存在的安装包 ($FILE_SIZE 字节)"
            exit 0
        fi
    fi
fi

# 创建一个空的占位符安装包（包含说明）
echo "创建占位符安装包..."

# 创建一个最小有效的 tar.gz 文件
TEMP_DIR=$(mktemp -d)
mkdir -p "$TEMP_DIR/openclaw/bin"

# 创建一个简单的说明文件
cat > "$TEMP_DIR/openclaw/README.txt" << 'EOF'
OpenClaw 离线安装包
====================

此目录应包含 OpenClaw 可执行文件。

文件结构:
- bin/openclaw (或 openclaw.exe on Windows)
- config/ (配置目录)

请从以下位置获取 OpenClaw 二进制文件:
1. 官方发布: https://github.com/openclai/openclaw/releases
2. 自行编译: git clone https://github.com/openclai/openclaw.git

将下载或编译的文件放入此目录后重新打包为 tar.gz (或 Windows 的 zip) 格式。
EOF

# 创建 platform/arch 特定的 tar.gz
case "$PLATFORM" in
    windows)
        # Windows 使用 zip
        (cd "$TEMP_DIR" && zip -r "$BUNDLED_DIR/openclaw-windows-x64.zip" openclaw/)
        echo "✓ 创建了 Windows 安装包占位符: openclaw-windows-x64.zip"
        ;;
    *)
        # macOS 和 Linux 使用 tar.gz
        (cd "$TEMP_DIR" && tar -czf "$BUNDLED_DIR/openclaw-${PLATFORM}-${ARCH_NAME}.tar.gz" openclaw/)
        echo "✓ 创建了 ${PLATFORM} 安装包占位符: openclaw-${PLATFORM}-${ARCH_NAME}.tar.gz"
        ;;
esac

# 清理
rm -rf "$TEMP_DIR"

echo ""
echo "=== 下一步 ==="
echo "1. 从源码编译或下载 OpenClaw 二进制文件"
echo "2. 将二进制文件放入正确的目录结构"
echo "3. 重新打包安装包"
echo ""
echo "示例结构:"
echo "  bundled/"
echo "    ├── openclaw-macos-arm64.tar.gz"
echo "    │   └── openclaw/"
echo "    │       └── bin/"
echo "    │           └── openclaw"
echo "    ├── openclaw-macos-x64.tar.gz"
echo "    └── ..."
