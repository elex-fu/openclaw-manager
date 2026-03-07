#!/bin/bash
# 清理 OpenClaw Manager 构建产物和临时文件
#
# 用法:
#   ./cleanup-build.sh              # 默认清理
#   ./cleanup-build.sh --all        # 完整清理（包括 node_modules）
#   ./cleanup-build.sh --packages   # 只清理旧包

set -e

KEEP_VERSIONS=2
CLEAN_ALL=false
CLEAN_PACKAGES_ONLY=false

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

# 解析参数
while [[ $# -gt 0 ]]; do
    case $1 in
        --all)
            CLEAN_ALL=true
            shift
            ;;
        --packages)
            CLEAN_PACKAGES_ONLY=true
            shift
            ;;
        -h|--help)
            echo "清理脚本"
            echo ""
            echo "用法: ./cleanup-build.sh [选项]"
            echo ""
            echo "选项:"
            echo "  --all         完整清理（包括 node_modules）"
            echo "  --packages    只清理旧包"
            echo "  -h, --help    显示帮助"
            exit 0
            ;;
        *)
            log_warn "未知参数: $1"
            shift
            ;;
    esac
done

echo "========================================"
echo "🧹 OpenClaw Manager 清理工具"
echo "========================================"
echo ""

# 只清理包
if [ "$CLEAN_PACKAGES_ONLY" = true ]; then
    log_step "清理旧包（保留最近 $KEEP_VERSIONS 个）..."
    BUNDLED_DIR="src-tauri/bundled"

    if [ -d "$BUNDLED_DIR" ]; then
        all_packages=$(ls -t "$BUNDLED_DIR"/openclaw-*.tar.gz "$BUNDLED_DIR"/openclaw-*.zip 2>/dev/null || true)

        if [ -n "$all_packages" ]; then
            count=$(echo "$all_packages" | wc -l)
            if [ "$count" -gt "$KEEP_VERSIONS" ]; then
                echo "$all_packages" | tail -n +$((KEEP_VERSIONS + 1)) | while read -r pkg; do
                    if [ -f "$pkg" ]; then
                        log_warn "删除: $(basename "$pkg")"
                        rm -f "$pkg"
                    fi
                done
            fi
        fi
    fi
    log_info "✓ 包清理完成"
    exit 0
fi

echo "🔍 分析项目占用空间..."
echo ""

# 显示当前大小
echo "清理前项目总大小:"
du -sh .
echo ""

# 清理 Rust 构建产物
if [ -d "src-tauri/target" ]; then
    log_step "清理 Rust target 目录..."
    TARGET_SIZE=$(du -sh src-tauri/target 2>/dev/null | cut -f1)
    cd src-tauri && cargo clean 2>/dev/null || rm -rf target
    cd ..
    log_info "✓ 已清理 ($TARGET_SIZE)"
fi

# 清理前端构建输出
if [ -d "dist" ]; then
    log_step "清理前端 dist 目录..."
    DIST_SIZE=$(du -sh dist 2>/dev/null | cut -f1)
    rm -rf dist
    log_info "✓ 已清理 ($DIST_SIZE)"
fi

# 清理旧包
log_step "清理旧包（保留最近 $KEEP_VERSIONS 个版本）..."
BUNDLED_DIR="src-tauri/bundled"
if [ -d "$BUNDLED_DIR" ]; then
    all_packages=$(ls -t "$BUNDLED_DIR"/openclaw-*.tar.gz "$BUNDLED_DIR"/openclaw-*.zip 2>/dev/null || true)

    if [ -n "$all_packages" ]; then
        count=$(echo "$all_packages" | wc -l)
        log_info "当前包数量: $count"

        if [ "$count" -gt "$KEEP_VERSIONS" ]; then
            echo "$all_packages" | tail -n +$((KEEP_VERSIONS + 1)) | while read -r pkg; do
                if [ -f "$pkg" ]; then
                    log_warn "删除: $(basename "$pkg")"
                    rm -f "$pkg"
                fi
            done
        fi
    else
        log_info "无包需要清理"
    fi
fi
log_info "✓ 包清理完成"

# 清理 /tmp 残留
echo ""
log_step "清理临时文件..."
TEMP_CLEANED=0

# 清理 /tmp 中的 openclaw 构建目录
for tmp_dir in /tmp/openclaw-build-* /tmp/openclaw-lite-*; do
    if [ -d "$tmp_dir" ]; then
        rm -rf "$tmp_dir"
        TEMP_CLEANED=$((TEMP_CLEANED + 1))
    fi
done

# 清理项目内的临时文件
find . -name "*.log" -type f -delete 2>/dev/null || true
find . -name ".DS_Store" -type f -delete 2>/dev/null || true
find . -name "Thumbs.db" -type f -delete 2>/dev/null || true

if [ "$TEMP_CLEANED" -gt 0 ]; then
    log_info "✓ 已清理 $TEMP_CLEANED 个临时目录"
else
    log_info "✓ 无临时文件需要清理"
fi

# 完整模式：清理 node_modules
if [ "$CLEAN_ALL" = true ]; then
    echo ""
    log_step "清理 node_modules..."
    if [ -d "node_modules" ]; then
        NM_SIZE=$(du -sh node_modules 2>/dev/null | cut -f1)
        rm -rf node_modules
        log_info "✓ 已清理 ($NM_SIZE)"
        log_warn "需要运行 'npm install' 重新安装依赖"
    fi

    # 也清理 src-tauri 的构建缓存
    if [ -d "src-tauri/target" ]; then
        log_step "清理 Rust 构建缓存..."
        rm -rf src-tauri/target
        log_info "✓ 已清理"
    fi
fi

echo ""
echo "========================================"
echo "✅ 清理完成！"
echo "========================================"
echo ""
echo "清理后项目总大小:"
du -sh .
echo ""
echo "💡 提示:"
echo "   - 重新构建前端: npm run build"
echo "   - 重新构建 Rust: cargo build --release (在 src-tauri 目录)"
echo "   - 开发模式: npm run tauri:dev"
if [ "$CLEAN_ALL" = true ]; then
    echo "   - 重新安装依赖: npm install"
fi
echo ""
