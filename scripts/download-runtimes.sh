#!/bin/bash
# 下载所有平台的嵌入式 Runtime 包

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNTIMES_DIR="$SCRIPT_DIR/../src-tauri/bundled/runtimes"

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

# 创建目录
mkdir -p "$RUNTIMES_DIR"
cd "$RUNTIMES_DIR"

log_info "下载嵌入式 Runtime 包"
log_info "========================"
echo ""

# Node.js 22 版本
NODE_VERSION="22.0.0"
NODE_BASE_URL="https://nodejs.org/dist/v${NODE_VERSION}"

# Python 3.10 版本 (python-build-standalone)
PYTHON_VERSION="3.10.14"
PYTHON_BUILD_DATE="20240107"
PYTHON_BASE_URL="https://github.com/indygreg/python-build-standalone/releases/download/${PYTHON_BUILD_DATE}"

# 定义下载函数
download_nodejs() {
    local platform=$1
    local arch=$2
    local filename="node-v${NODE_VERSION}-${platform}-${arch}.tar.gz"

    if [ -f "$filename" ] && [ -s "$filename" ]; then
        log_warn "$filename 已存在，跳过下载"
        return 0
    fi

    log_step "下载 Node.js ${NODE_VERSION} for ${platform}-${arch}..."

    if curl -fsSL -o "$filename" "${NODE_BASE_URL}/${filename}"; then
        log_info "✓ $filename 下载成功"
        return 0
    else
        log_error "✗ $filename 下载失败"
        return 1
    fi
}

download_python() {
    local platform=$1
    local arch=$2

    local filename_suffix
    case "$platform" in
        darwin)
            if [ "$arch" = "x64" ]; then
                filename_suffix="x86_64-apple-darwin"
            else
                filename_suffix="aarch64-apple-darwin"
            fi
            ;;
        linux)
            if [ "$arch" = "x64" ]; then
                filename_suffix="x86_64-unknown-linux-gnu"
            else
                filename_suffix="aarch64-unknown-linux-gnu"
            fi
            ;;
        windows)
            filename_suffix="x86_64-pc-windows-msvc"
            ;;
    esac

    local filename="cpython-${PYTHON_VERSION}+${PYTHON_BUILD_DATE}-${filename_suffix}-install_only.tar.gz"
    local output_name="python-${PYTHON_VERSION}-${platform}-${arch}.tar.gz"

    if [ -f "$output_name" ] && [ -s "$output_name" ]; then
        log_warn "$output_name 已存在，跳过下载"
        return 0
    fi

    log_step "下载 Python ${PYTHON_VERSION} for ${platform}-${arch}..."

    if curl -fsSL -o "$output_name" "${PYTHON_BASE_URL}/${filename}"; then
        log_info "✓ $output_name 下载成功"
        return 0
    else
        log_error "✗ $output_name 下载失败"
        return 1
    fi
}

# 主下载逻辑
case "${1:-all}" in
    nodejs|node)
        log_step "仅下载 Node.js..."
        download_nodejs "darwin" "arm64"
        download_nodejs "darwin" "x64"
        download_nodejs "linux" "x64"
        ;;
    python|py)
        log_step "仅下载 Python..."
        download_python "darwin" "arm64"
        download_python "darwin" "x64"
        download_python "linux" "x64"
        ;;
    current|local)
        log_step "仅下载当前平台..."
        OS=$(uname -s | tr '[:upper:]' '[:lower:]')
        ARCH=$(uname -m)

        case "$OS" in
            linux) PLATFORM="linux" ;;
            darwin) PLATFORM="darwin" ;;
            *) log_error "不支持的平台: $OS"; exit 1 ;;
        esac

        case "$ARCH" in
            x86_64|amd64) ARCH_NAME="x64" ;;
            arm64|aarch64) ARCH_NAME="arm64" ;;
            *) log_error "不支持的架构: $ARCH"; exit 1 ;;
        esac

        download_nodejs "$PLATFORM" "$ARCH_NAME"
        download_python "$PLATFORM" "$ARCH_NAME"
        ;;
    all|*)
        log_step "下载所有平台 Runtime..."

        # Node.js
        download_nodejs "darwin" "arm64" || true
        download_nodejs "darwin" "x64" || true
        download_nodejs "linux" "x64" || true

        echo ""

        # Python
        download_python "darwin" "arm64" || true
        download_python "darwin" "x64" || true
        download_python "linux" "x64" || true
        ;;
esac

echo ""
log_info "下载完成！"
echo ""
ls -lh "$RUNTIMES_DIR"/*.tar.gz 2>/dev/null || echo "暂无下载文件"
