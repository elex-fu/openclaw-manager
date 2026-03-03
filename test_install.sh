#!/bin/bash
# OpenClaw Manager 安装功能测试脚本

set -e

echo "=================================="
echo "OpenClaw Manager 安装功能测试"
echo "=================================="
echo ""

# 1. 检查离线安装包是否存在
echo "1. 检查离线安装包..."
BUNDLED_DIR="./src-tauri/bundled"

for package in \
    "openclaw-macos-arm64.tar.gz" \
    "openclaw-macos-x64.tar.gz" \
    "openclaw-linux-x64.tar.gz" \
    "openclaw-windows-x64.zip"
do
    if [ -f "$BUNDLED_DIR/$package" ]; then
        size=$(stat -f%z "$BUNDLED_DIR/$package" 2>/dev/null || stat -c%s "$BUNDLED_DIR/$package" 2>/dev/null || echo "未知")
        echo "   ✓ $package 存在 ($size 字节)"
    else
        echo "   ✗ $package 不存在"
    fi
done
echo ""

# 2. 检查测试安装脚本
echo "2. 检查测试安装脚本..."
if [ -f "./src-tauri/scripts/install_test.sh" ]; then
    echo "   ✓ install_test.sh 存在"
else
    echo "   ✗ install_test.sh 不存在"
fi
echo ""

# 3. 验证安装包内容
echo "3. 验证安装包内容..."
if command -v tar &>/dev/null; then
    echo "   检查 macOS ARM64 包内容:"
    tar -tzf "$BUNDLED_DIR/openclaw-macos-arm64.tar.gz" | head -5 | sed 's/^/     - /'
fi
echo ""

# 4. 运行 Rust 测试
echo "4. 运行 Rust 编译检查..."
cd src-tauri
if cargo check 2>/dev/null; then
    echo "   ✓ Rust 代码编译通过"
else
    echo "   ✗ Rust 代码编译失败"
fi
cd ..
echo ""

# 5. 模拟离线安装测试
echo "5. 模拟离线安装测试..."
TEST_INSTALL_DIR="/tmp/test_openclaw_install"
rm -rf "$TEST_INSTALL_DIR"
mkdir -p "$TEST_INSTALL_DIR"

# 解压安装包到临时目录
echo "   解压安装包..."
if tar -xzf "$BUNDLED_DIR/openclaw-macos-arm64.tar.gz" -C "$TEST_INSTALL_DIR" 2>/dev/null; then
    echo "   ✓ 安装包解压成功"
else
    echo "   ✗ 安装包解压失败"
fi

# 检查解压后的内容
if [ -d "$TEST_INSTALL_DIR/bin" ]; then
    echo "   ✓ bin 目录存在"
    if [ -f "$TEST_INSTALL_DIR/bin/openclaw" ]; then
        echo "   ✓ openclaw 可执行文件存在"
        # 测试执行
        if chmod +x "$TEST_INSTALL_DIR/bin/openclaw" && "$TEST_INSTALL_DIR/bin/openclaw" --version 2>/dev/null; then
            echo "   ✓ openclaw 可执行文件可以运行"
        else
            echo "   ⚠ openclaw 可执行文件无法运行（可能需要特定环境）"
        fi
    else
        echo "   ✗ openclaw 可执行文件不存在"
    fi
else
    echo "   ✗ bin 目录不存在"
fi

# 清理
rm -rf "$TEST_INSTALL_DIR"
echo ""

echo "=================================="
echo "测试完成"
echo "=================================="
