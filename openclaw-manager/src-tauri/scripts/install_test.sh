#!/bin/bash
# OpenClaw 测试安装脚本
# 这是一个用于开发和测试的简化安装脚本

set -e

INSTALL_DIR="${1:-$HOME/.openclaw}"
BIN_DIR="$INSTALL_DIR/bin"

echo "开始安装 OpenClaw (测试模式)..."
echo "安装目录: $INSTALL_DIR"

# 创建目录
mkdir -p "$BIN_DIR"

# 创建模拟的 openclaw 可执行文件
cat > "$BIN_DIR/openclaw" << 'EOF'
#!/bin/bash
if [ "$1" = "--version" ]; then
    echo "openclaw 0.1.0 (test)"
    exit 0
fi
echo "OpenClaw CLI (Test Mode)"
echo "Usage: openclaw [command]"
echo ""
echo "Available commands:"
echo "  serve    Start the OpenClaw server"
echo "  config   Manage configuration"
echo "  help     Show help"
exit 0
EOF

# 设置执行权限
chmod +x "$BIN_DIR/openclaw"

echo "安装完成!"
echo "二进制文件位置: $BIN_DIR/openclaw"
echo ""
echo "请将 $BIN_DIR 添加到 PATH:"
echo "  export PATH=\"$BIN_DIR:\$PATH\""
