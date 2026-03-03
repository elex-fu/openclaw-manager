#!/bin/bash
# 测试离线安装功能

set -e

echo "=== OpenClaw 离线安装测试 ==="
echo ""

# 清理之前的安装
echo "1. 清理之前的安装..."
rm -rf ~/.openclaw

# 检查安装包
echo ""
echo "2. 检查安装包..."
ARCH=$(uname -m)
if [ "$ARCH" = "x86_64" ]; then
    ARCH_NAME="x64"
elif [ "$ARCH" = "arm64" ]; then
    ARCH_NAME="arm64"
else
    echo "不支持的架构: $ARCH"
    exit 1
fi

PACKAGE_FILE="src-tauri/bundled/openclaw-macos-${ARCH_NAME}.tar.gz"
if [ ! -f "$PACKAGE_FILE" ]; then
    echo "错误: 找不到安装包 $PACKAGE_FILE"
    exit 1
fi

echo "找到安装包: $PACKAGE_FILE"
echo "文件大小: $(stat -f%z "$PACKAGE_FILE" 2>/dev/null || stat -c%s "$PACKAGE_FILE" 2>/dev/null) 字节"

# 检查安装包内容
echo ""
echo "3. 检查安装包内容..."
tar -tzf "$PACKAGE_FILE"

# 模拟离线安装流程
echo ""
echo "4. 模拟离线安装流程..."

# 创建临时目录
TEMP_DIR=$(mktemp -d)
echo "临时目录: $TEMP_DIR"

# 解压
echo "解压安装包..."
tar -xzf "$PACKAGE_FILE" -C "$TEMP_DIR"

# 检查解压后的结构
echo "解压后的结构:"
ls -la "$TEMP_DIR"

# 创建安装目录
mkdir -p ~/.openclaw

# 复制文件
echo ""
echo "5. 复制文件到 ~/.openclaw..."
if [ -d "$TEMP_DIR/openclaw" ]; then
    cp -r "$TEMP_DIR/openclaw/"* ~/.openclaw/
else
    cp -r "$TEMP_DIR/"* ~/.openclaw/
fi

# 设置权限
echo "设置可执行权限..."
chmod -R +x ~/.openclaw/bin/

# 检查结果
echo ""
echo "6. 检查安装结果..."
ls -la ~/.openclaw/
ls -la ~/.openclaw/bin/

# 测试运行
echo ""
echo "7. 测试运行 openclaw..."
if ~/.openclaw/bin/openclaw --version; then
    echo ""
    echo "✅ 离线安装测试成功!"
else
    echo ""
    echo "❌ 离线安装测试失败!"
    exit 1
fi

# 清理
rm -rf "$TEMP_DIR"

echo ""
echo "=== 测试完成 ==="
