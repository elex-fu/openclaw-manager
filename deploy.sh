#!/bin/bash

# OpenClaw Manager 一键部署脚本
# 支持 macOS 和 Linux

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印函数
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查命令是否存在
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# 检查 Node.js
check_nodejs() {
    print_info "检查 Node.js..."
    if command_exists node; then
        NODE_VERSION=$(node --version)
        print_success "Node.js 已安装: $NODE_VERSION"

        # 检查版本是否 >= 18
        NODE_MAJOR=$(node --version | cut -d'v' -f2 | cut -d'.' -f1)
        if [ "$NODE_MAJOR" -lt 18 ]; then
            print_error "Node.js 版本过低，需要 >= 18.x"
            print_info "请升级 Node.js: https://nodejs.org/"
            exit 1
        fi
    else
        print_error "Node.js 未安装"
        print_info "请安装 Node.js 18+: https://nodejs.org/"
        exit 1
    fi
}

# 检查 Rust
check_rust() {
    print_info "检查 Rust..."
    if command_exists rustc; then
        RUST_VERSION=$(rustc --version)
        print_success "Rust 已安装: $RUST_VERSION"
    elif [ -f "$HOME/.cargo/bin/rustc" ]; then
        export PATH="$HOME/.cargo/bin:$PATH"
        RUST_VERSION=$(rustc --version)
        print_success "Rust 已安装: $RUST_VERSION"
    elif [ -f "/opt/local/bin/rustc" ]; then
        export PATH="/opt/local/bin:$PATH"
        RUST_VERSION=$(rustc --version)
        print_success "Rust 已安装 (MacPorts): $RUST_VERSION"
    else
        print_error "Rust 未安装"
        print_info "请安装 Rust: https://rustup.rs/"
        exit 1
    fi

    # 检查 cargo
    if ! command_exists cargo; then
        print_error "Cargo 未找到"
        print_info "请确保 Rust 完整安装"
        exit 1
    fi

    CARGO_VERSION=$(cargo --version)
    print_success "Cargo 已安装: $CARGO_VERSION"
}

# 安装前端依赖
install_frontend_deps() {
    print_info "安装前端依赖..."
    if [ -d "node_modules" ]; then
        print_warning "node_modules 已存在，跳过安装"
    else
        npm install
        print_success "前端依赖安装完成"
    fi
}

# 构建前端
build_frontend() {
    print_info "构建前端..."
    npm run build
    print_success "前端构建完成"
}

# 检查并安装 Tauri CLI
check_tauri_cli() {
    print_info "检查 Tauri CLI..."
    if ! command_exists tauri; then
        print_info "安装 Tauri CLI..."
        cargo install tauri-cli
        print_success "Tauri CLI 安装完成"
    else
        print_success "Tauri CLI 已安装"
    fi
}

# 构建 Tauri 应用
build_tauri() {
    print_info "构建 Tauri 应用..."
    cd src-tauri
    cargo build --release
    cd ..
    print_success "Tauri 应用构建完成"
}

# 开发模式启动
dev_mode() {
    print_info "启动开发模式..."
    print_info "这将同时启动前端开发服务器和 Tauri 应用"
    npm run tauri:dev
}

# 构建生产版本
build_production() {
    print_info "构建生产版本..."
    npm run tauri:build
    print_success "生产版本构建完成"
    print_info "应用位置: src-tauri/target/release/bundle/"
}

# 主菜单
show_menu() {
    echo ""
    echo "========================================"
    echo "   OpenClaw Manager 部署工具"
    echo "========================================"
    echo ""
    echo "1. 一键部署 (检查环境 + 安装依赖 + 开发模式)"
    echo "2. 仅检查环境"
    echo "3. 仅安装依赖"
    echo "4. 开发模式启动"
    echo "5. 构建生产版本"
    echo "6. 退出"
    echo ""
    read -p "请选择操作 [1-6]: " choice

    case $choice in
        1)
            check_nodejs
            check_rust
            install_frontend_deps
            dev_mode
            ;;
        2)
            check_nodejs
            check_rust
            print_success "环境检查完成"
            ;;
        3)
            check_nodejs
            install_frontend_deps
            ;;
        4)
            dev_mode
            ;;
        5)
            check_nodejs
            check_rust
            install_frontend_deps
            build_production
            ;;
        6)
            print_info "退出"
            exit 0
            ;;
        *)
            print_error "无效选择"
            show_menu
            ;;
    esac
}

# 主函数
main() {
    echo ""
    print_success "OpenClaw Manager 部署脚本"
    echo ""

    # 检查是否在项目根目录
    if [ ! -f "package.json" ] || [ ! -d "src-tauri" ]; then
        print_error "请在项目根目录运行此脚本"
        print_info "当前目录: $(pwd)"
        exit 1
    fi

    # 如果有参数，直接执行对应操作
    case "${1:-}" in
        --check)
            check_nodejs
            check_rust
            ;;
        --deps)
            install_frontend_deps
            ;;
        --dev)
            dev_mode
            ;;
        --build)
            check_nodejs
            check_rust
            install_frontend_deps
            build_production
            ;;
        *)
            show_menu
            ;;
    esac
}

# 运行主函数
main "$@"
