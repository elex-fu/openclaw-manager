#!/bin/bash
# 测试 OpenClaw 状态检测

echo "=== OpenClaw 状态检测测试 ==="
echo ""

# 1. 检查安装状态
echo "1. 检查安装状态:"
ls -la ~/.openclaw/bin/openclaw 2>/dev/null && echo "✓ 已安装" || echo "✗ 未安装"

# 2. 检查版本
echo ""
echo "2. 检查版本:"
~/.openclaw/bin/openclaw --version 2>/dev/null || echo "无法获取版本"

# 3. 测试进程检测方法
echo ""
echo "3. 测试进程检测方法:"

echo "   方法1: pgrep -x openclaw"
if pgrep -x openclaw > /dev/null 2>&1; then
    echo "   ✓ 找到 openclaw 进程"
else
    echo "   ✗ 未找到 openclaw 进程"
fi

echo ""
echo "   方法2: pgrep -x openclaw-gateway"
if pgrep -x openclaw-gateway > /dev/null 2>&1; then
    echo "   ✓ 找到 openclaw-gateway 进程"
    pgrep -x openclaw-gateway
else
    echo "   ✗ 未找到 openclaw-gateway 进程"
fi

echo ""
echo "   方法3: pgrep -f openclaw"
pgrep -f openclaw | while read pid; do
    echo "   找到 PID: $pid"
done

# 4. 获取进程详细信息
echo ""
echo "4. 进程详细信息:"
echo "   openclaw-gateway:"
ps -o pid,pcpu,pmem,time,command -c openclaw-gateway 2>/dev/null || echo "   无此进程"

echo ""
echo "   所有 openclaw 相关进程:"
ps aux | grep -v grep | grep openclaw | grep -v "openclaw-manager" | grep -v "browser"

echo ""
echo "=== 测试完成 ==="
