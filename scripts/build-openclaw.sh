#!/bin/bash
#
# OpenClaw 统一打包脚本（优化版）
# 支持 lite 和 full 模式，带版本管理和体积优化
#
# 用法:
#   ./build-openclaw.sh              # 默认 lite 模式（最小体积）
#   ./build-openclaw.sh --full       # 完整模式（清理后的 node_modules）
#   ./build-openclaw.sh --clean      # 清理旧包，只保留最近2个
#   ./build-openclaw.sh --list       # 列出所有包
#

set -e

# 配置
OPENCLAW_DIR="${OPENCLAW_SOURCE:-/Users/lex/play/openclaw}"
MANAGER_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BUNDLED_DIR="$MANAGER_DIR/src-tauri/bundled"
BUILD_DIR=""
MAX_PACKAGE_SIZE_MB=150
KEEP_VERSIONS=2
MODE="lite"
OPTIMIZE=true

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "${BLUE}[STEP]${NC} $1"; }
log_opt() { echo -e "${CYAN}[OPT]${NC} $1"; }

# 进度显示函数
show_progress() {
    local current=$1
    local total=$2
    local label="${3:-Progress}"
    local width=50
    local percentage=$((current * 100 / total))
    local filled=$((current * width / total))
    local empty=$((width - filled))

    printf "\r${BLUE}[PROGRESS]${NC} %s [%s%s] %d%% (%d/%d)" \
        "$label" \
        "$(printf '%*s' $filled '' | tr ' ' '=')" \
        "$(printf '%*s' $empty '' | tr ' ' ' ')" \
        $percentage \
        $current \
        $total

    if [ $current -eq $total ]; then
        echo ""  # 换行
    fi
}

# 显示文件复制进度
copy_with_progress() {
    local src="$1"
    local dst="$2"
    local label="${3:-Copying}"

    if [ ! -d "$src" ]; then
        return 1
    fi

    # 计算总文件数
    local total_files=$(find "$src" -type f 2>/dev/null | wc -l)
    local copied=0

    log_info "$label: 共 $total_files 个文件"

    # 复制文件并显示进度
    find "$src" -type f 2>/dev/null | while read -r file; do
        local rel_path="${file#$src/}"
        local target_file="$dst/$rel_path"
        local target_dir=$(dirname "$target_file")

        mkdir -p "$target_dir"
        cp -f "$file" "$target_file" 2>/dev/null || true

        copied=$((copied + 1))
        if [ $((copied % 10)) -eq 0 ] || [ $copied -eq $total_files ]; then
            show_progress $copied $total_files "$label"
        fi
    done
}

# 显示压缩进度
compress_with_progress() {
    local source_dir="$1"
    local output_file="$2"
    local compress_type="${3:-tar}"

    if [ "$compress_type" = "zip" ]; then
        # zip 压缩显示进度
        local total_files=$(find "$source_dir" -type f | wc -l)
        log_info "压缩中: 共 $total_files 个文件"

        (cd "$source_dir" && zip -r - "$source_dir" 2>/dev/null |
         while read -r line; do
             current=$((current + 1))
             if [ $((current % 50)) -eq 0 ]; then
                 show_progress $current $total_files "Compressing"
             fi
         done) > "$output_file"
    else
        # tar.gz 压缩
        local total_size=$(du -sm "$source_dir" | cut -f1)
        log_info "压缩中: 总大小 ${total_size}MB"

        # 使用 pv 如果有，否则使用 tar
        if command -v pv >/dev/null 2>&1; then
            tar -cf - "$source_dir" 2>/dev/null | pv -s ${total_size}m | gzip > "$output_file"
        else
            # 使用 tar 的 verbose 模式并统计
            (tar -czf "$output_file" "$source_dir" 2>/dev/null &
             TAR_PID=$!

             while kill -0 $TAR_PID 2>/dev/null; do
                 if [ -f "$output_file" ]; then
                     local current_size=$(du -sm "$output_file" 2>/dev/null | cut -f1)
                     show_progress $current_size $total_size "Compressing"
                 fi
                 sleep 0.5
             done
             wait $TAR_PID)
        fi
    fi
}

# 显示帮助
show_help() {
    cat << 'EOF'
OpenClaw 优化打包脚本

用法: ./build-openclaw.sh [选项]

选项:
    --full          完整模式（含清理后的 node_modules）
    --lite          轻量模式（最小体积，推荐）
    --no-optimize   禁用体积优化（全量打包）
    --clean         清理旧包，只保留最近 2 个版本
    --list          列出所有已打包的版本
    --max-size N    设置包大小限制（MB，默认 150MB）
    -h, --help      显示帮助

优化内容（默认启用）:
    - 排除 src/ 目录（开发用源码）
    - 排除 .d.ts 类型定义文件
    - 排除 .map source map 文件
    - 排除 docs/ 目录
    - 排除开发配置文件（tsconfig等）
    - 清理 node_modules 中的开发依赖和缓存

示例:
    ./build-openclaw.sh              # lite 模式 + 优化（推荐）
    ./build-openclaw.sh --full       # 完整模式（含生产依赖）
    ./build-openclaw.sh --full --no-optimize  # 全量无优化
EOF
}

# 解析参数
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --full)
                MODE="full"
                shift
                ;;
            --lite)
                MODE="lite"
                shift
                ;;
            --no-optimize)
                OPTIMIZE=false
                shift
                ;;
            --clean)
                clean_old_packages
                exit 0
                ;;
            --list)
                list_packages
                exit 0
                ;;
            --max-size)
                MAX_PACKAGE_SIZE_MB="$2"
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
}

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
        *) log_error "不支持的平台: $OS"; exit 1 ;;
    esac

    log_info "目标平台: $PLATFORM-$ARCH"
}

# 获取版本号
get_version() {
    if [ -f "$OPENCLAW_DIR/package.json" ]; then
        VERSION=$(cd "$OPENCLAW_DIR" && node -p "require('./package.json').version" 2>/dev/null || echo "unknown")
    else
        VERSION="unknown"
    fi
    echo "$VERSION"
}

# 列出所有包
list_packages() {
    log_info "已打包的版本列表"
    log_info "=================="
    echo ""

    if [ ! -d "$BUNDLED_DIR" ] || [ -z "$(ls -A "$BUNDLED_DIR"/*.tar.gz 2>/dev/null)" ]; then
        log_warn "暂无打包文件"
        return
    fi

    ls -lt "$BUNDLED_DIR"/openclaw-*.tar.gz "$BUNDLED_DIR"/openclaw-*.zip 2>/dev/null | while read -r line; do
        echo "  $line"
    done

    echo ""
    log_info "包总数: $(ls "$BUNDLED_DIR"/openclaw-*.tar.gz "$BUNDLED_DIR"/openclaw-*.zip 2>/dev/null | wc -l)"
}

# 清理旧包
clean_old_packages() {
    log_step "清理旧包（保留最近 $KEEP_VERSIONS 个版本）..."

    if [ ! -d "$BUNDLED_DIR" ]; then
        log_warn "bundled 目录不存在"
        return
    fi

    local all_packages
    all_packages=$(ls -t "$BUNDLED_DIR"/openclaw-*.tar.gz "$BUNDLED_DIR"/openclaw-*.zip 2>/dev/null || true)

    if [ -z "$all_packages" ]; then
        log_warn "没有找到需要清理的包"
        return
    fi

    local count
    count=$(echo "$all_packages" | wc -l)

    if [ "$count" -le "$KEEP_VERSIONS" ]; then
        log_info "当前只有 $count 个包，无需清理"
        return
    fi

    local to_delete=$((count - KEEP_VERSIONS))
    log_info "发现 $count 个包，将删除 $to_delete 个旧包"

    # 删除旧的包
    local deleted=0
    echo "$all_packages" | tail -n +$((KEEP_VERSIONS + 1)) | while read -r pkg; do
        if [ -f "$pkg" ]; then
            rm -f "$pkg"
            deleted=$((deleted + 1))
            show_progress $deleted $to_delete "Cleaning"
        fi
    done

    echo ""
    log_info "✓ 清理完成，保留最近 $KEEP_VERSIONS 个版本"
}

# 检查包大小
check_package_size() {
    local pkg_file="$1"
    local size_bytes
    local size_mb

    size_bytes=$(stat -f%z "$pkg_file" 2>/dev/null || stat -c%s "$pkg_file" 2>/dev/null || echo "0")
    size_mb=$((size_bytes / 1024 / 1024))

    echo "$size_mb"
}

# 计算目录大小
calc_dir_size() {
    local dir="$1"
    if [ -d "$dir" ]; then
        du -sh "$dir" 2>/dev/null | cut -f1
    else
        echo "0B"
    fi
}

# 清理 node_modules 中的开发文件和缓存
clean_node_modules() {
    local target_dir="$1"

    log_opt "深度清理 node_modules..."
    local before_size=$(du -sm "$target_dir" 2>/dev/null | cut -f1)

    # 清理任务列表和进度
    local total_tasks=7
    local current_task=0

    # 1. 删除类型定义文件（.d.ts）
    current_task=$((current_task + 1))
    log_opt "  [$current_task/$total_tasks] 删除类型定义文件..."
    find "$target_dir" -name "*.d.ts" -type f -delete 2>/dev/null || true
    find "$target_dir" -name "*.d.ts.map" -type f -delete 2>/dev/null || true
    show_progress $current_task $total_tasks "Cleaning"

    # 2. 删除 source map 文件
    current_task=$((current_task + 1))
    log_opt "  [$current_task/$total_tasks] 删除 source map..."
    find "$target_dir" -name "*.map" -type f -delete 2>/dev/null || true
    show_progress $current_task $total_tasks "Cleaning"

    # 3. 删除 TypeScript 源码（保留 JS）
    current_task=$((current_task + 1))
    log_opt "  [$current_task/$total_tasks] 删除 TypeScript 源码..."
    find "$target_dir" -name "*.ts" -type f -not -name "*.d.ts" -delete 2>/dev/null || true
    show_progress $current_task $total_tasks "Cleaning"

    # 4. 删除测试文件
    current_task=$((current_task + 1))
    log_opt "  [$current_task/$total_tasks] 删除测试文件..."
    find "$target_dir" -name "__tests__" -type d -exec rm -rf {} + 2>/dev/null || true
    find "$target_dir" -name "test" -type d -exec rm -rf {} + 2>/dev/null || true
    find "$target_dir" -name "tests" -type d -exec rm -rf {} + 2>/dev/null || true
    find "$target_dir" -name "*.test.js" -type f -delete 2>/dev/null || true
    find "$target_dir" -name "*.spec.js" -type f -delete 2>/dev/null || true
    find "$target_dir" -name "*.test.ts" -type f -delete 2>/dev/null || true
    find "$target_dir" -name "*.spec.ts" -type f -delete 2>/dev/null || true
    show_progress $current_task $total_tasks "Cleaning"

    # 5. 删除文档和示例
    current_task=$((current_task + 1))
    log_opt "  [$current_task/$total_tasks] 删除文档文件..."
    find "$target_dir" -name "README*" -type f -delete 2>/dev/null || true
    find "$target_dir" -name "CHANGELOG*" -type f -delete 2>/dev/null || true
    find "$target_dir" -name "LICENSE*" -type f -delete 2>/dev/null || true
    find "$target_dir" -name "CONTRIBUTING*" -type f -delete 2>/dev/null || true
    find "$target_dir" -name "example*" -type d -exec rm -rf {} + 2>/dev/null || true
    find "$target_dir" -name "examples" -type d -exec rm -rf {} + 2>/dev/null || true
    find "$target_dir" -name "docs" -type d -exec rm -rf {} + 2>/dev/null || true
    find "$target_dir" -name "doc" -type d -exec rm -rf {} + 2>/dev/null || true
    find "$target_dir" -name "*.md" -type f -delete 2>/dev/null || true
    show_progress $current_task $total_tasks "Cleaning"

    # 6. 删除构建缓存
    current_task=$((current_task + 1))
    log_opt "  [$current_task/$total_tasks] 删除缓存文件..."
    find "$target_dir" -name ".cache" -type d -exec rm -rf {} + 2>/dev/null || true
    find "$target_dir" -name ".DS_Store" -type f -delete 2>/dev/null || true
    find "$target_dir" -name "*.log" -type f -delete 2>/dev/null || true
    show_progress $current_task $total_tasks "Cleaning"

    # 7. 删除平台特定的不必要二进制（只保留当前平台）
    current_task=$((current_task + 1))
    if [ "$OPTIMIZE" = true ]; then
        log_opt "  [$current_task/$total_tasks] 删除其他平台二进制..."
        find "$target_dir" -name "*linux-*" -type d -exec rm -rf {} + 2>/dev/null || true
        find "$target_dir" -name "*linux-*" -type f -delete 2>/dev/null || true
        find "$target_dir" -name "*win32-*" -type d -exec rm -rf {} + 2>/dev/null || true
        find "$target_dir" -name "*win32-*" -type f -delete 2>/dev/null || true
        find "$target_dir" -name "*windows-*" -type d -exec rm -rf {} + 2>/dev/null || true
        find "$target_dir" -name "*windows-*" -type f -delete 2>/dev/null || true
        find "$target_dir" -name "*android-*" -type d -exec rm -rf {} + 2>/dev/null || true
        find "$target_dir" -name "*android-*" -type f -delete 2>/dev/null || true
        find "$target_dir" -name "*freebsd-*" -type d -exec rm -rf {} + 2>/dev/null || true
        find "$target_dir" -name "*freebsd-*" -type f -delete 2>/dev/null || true
    fi
    show_progress $current_task $total_tasks "Cleaning"

    # 8. 删除不必要的开发工具和大文件
    log_opt "  清理开发工具包..."

    # 删除 esbuild 的其他平台二进制（只保留当前平台）
    find "$target_dir" -path "*esbuild*" -name "*esbuild-darwin-arm64*" -prune -o \
        -path "*esbuild*" -name "*esbuild-*" -type f -delete 2>/dev/null || true
    find "$target_dir" -path "*esbuild*" -name "*esbuild-linux*" -type d -exec rm -rf {} + 2>/dev/null || true
    find "$target_dir" -path "*esbuild*" -name "*esbuild-windows*" -type d -exec rm -rf {} + 2>/dev/null || true

    # 删除重复的 canvas 二进制
    find "$target_dir" -path "*@napi-rs*canvas*" -name "*.node" -type f | while read -r f; do
        # 只保留一个版本的 canvas
        if [[ "$f" != *"0.1.94"* ]] && [[ "$f" != *"current"* ]]; then
            rm -f "$f" 2>/dev/null || true
        fi
    done

    local after_size=$(du -sm "$target_dir" 2>/dev/null | cut -f1)
    local saved=$((before_size - after_size))
    log_opt "  ✓ 清理完成: ${before_size}MB → ${after_size}MB (节省 ${saved}MB)"
}

# 准备 OpenClaw
prepare_openclaw() {
    log_step "准备 OpenClaw..."

    if [ ! -d "$OPENCLAW_DIR" ]; then
        log_error "OpenClaw 目录不存在: $OPENCLAW_DIR"
        exit 1
    fi

    cd "$OPENCLAW_DIR"

    if [ ! -f "package.json" ]; then
        log_error "找不到 package.json"
        exit 1
    fi

    # 安装依赖（如果需要）
    if [ ! -d "node_modules" ]; then
        log_info "安装依赖..."
        if command -v pnpm &> /dev/null; then
            pnpm install --production=false || npm install
        else
            npm install
        fi
    fi

    # 尝试构建
    log_info "构建 OpenClaw..."
    if command -v pnpm &> /dev/null; then
        pnpm build 2>/dev/null || log_warn "构建有警告，继续打包..."
    else
        npm run build 2>/dev/null || log_warn "构建有警告，继续打包..."
    fi

    log_info "✓ 准备完成"
}

# 复制文件（带优化）
copy_with_optimization() {
    local src="$1"
    local dst="$2"
    local name="$3"

    if [ ! -d "$src" ]; then
        return
    fi

    log_opt "复制 $name/ ..."

    # 创建目标目录
    mkdir -p "$dst"

    # 使用 rsync 进行优化复制
    if command -v rsync &> /dev/null && [ "$OPTIMIZE" = true ]; then
        rsync -a \
            --exclude='node_modules' \
            --exclude='.git' \
            --exclude='.github' \
            --exclude='__tests__' \
            --exclude='test' \
            --exclude='tests' \
            --exclude='*.test.ts' \
            --exclude='*.test.js' \
            --exclude='*.spec.ts' \
            --exclude='*.spec.js' \
            --exclude='tsconfig*.json' \
            --exclude='*.config.ts' \
            --exclude='*.config.js' \
            --exclude='.env*' \
            --exclude='.DS_Store' \
            --exclude='*.log' \
            "$src/" "$dst/" 2>/dev/null || cp -r "$src"/* "$dst/" 2>/dev/null || true
    else
        # 回退到 cp
        cp -r "$src"/* "$dst/" 2>/dev/null || true
    fi
}

# 打包 OpenClaw
package_openclaw() {
    log_step "打包 OpenClaw ($MODE 模式)..."

    if [ "$OPTIMIZE" = true ]; then
        log_info "体积优化: 启用"
    else
        log_warn "体积优化: 禁用"
    fi

    # 创建临时目录
    BUILD_DIR=$(mktemp -d)
    mkdir -p "$BUILD_DIR/openclaw"

    cd "$OPENCLAW_DIR"

    log_info "复制核心文件..."

    # 核心文件（总是复制）
    cp package.json "$BUILD_DIR/openclaw/"
    [ -f "README.md" ] && cp README.md "$BUILD_DIR/openclaw/"
    [ -f "LICENSE" ] && cp LICENSE "$BUILD_DIR/openclaw/"

    # 入口脚本
    if [ -f "openclaw.mjs" ]; then
        cp openclaw.mjs "$BUILD_DIR/openclaw/"
        log_opt "  ✓ openclaw.mjs"
    fi

    # dist/ 目录（构建输出）- 优化：删除 source map
    if [ -d "dist" ]; then
        log_opt "复制 dist/ ..."
        mkdir -p "$BUILD_DIR/openclaw/dist"
        cp -r dist/* "$BUILD_DIR/openclaw/dist/" 2>/dev/null || true

        if [ "$OPTIMIZE" = true ]; then
            # 删除 dist 中的 source map
            find "$BUILD_DIR/openclaw/dist" -name "*.map" -type f -delete 2>/dev/null || true
            # 删除类型定义
            find "$BUILD_DIR/openclaw/dist" -name "*.d.ts" -type f -delete 2>/dev/null || true
            log_opt "  ✓ 已清理 dist 中的开发文件"
        fi
    fi

    # assets/ 目录
    if [ -d "assets" ]; then
        copy_with_optimization "$OPENCLAW_DIR/assets" "$BUILD_DIR/openclaw/assets" "assets"
    fi

    # skills/ 目录
    if [ -d "skills" ]; then
        copy_with_optimization "$OPENCLAW_DIR/skills" "$BUILD_DIR/openclaw/skills" "skills"
    fi

    # extensions/ 目录 - 优化：排除开发文件
    if [ -d "extensions" ]; then
        log_opt "复制 extensions/（清理后）..."
        mkdir -p "$BUILD_DIR/openclaw/extensions"

        for ext in "$OPENCLAW_DIR"/extensions/*/; do
            if [ -d "$ext" ]; then
                local ext_name=$(basename "$ext")
                mkdir -p "$BUILD_DIR/openclaw/extensions/$ext_name"

                # 复制扩展文件（排除开发文件）
                if [ "$OPTIMIZE" = true ]; then
                    rsync -a \
                        --exclude='node_modules' \
                        --exclude='src' \
                        --exclude='tsconfig*.json' \
                        --exclude='*.config.ts' \
                        --exclude='*.test.ts' \
                        --exclude='__tests__' \
                        --exclude='.git' \
                        "$ext/" "$BUILD_DIR/openclaw/extensions/$ext_name/" 2>/dev/null || \
                        cp -r "$ext"/* "$BUILD_DIR/openclaw/extensions/$ext_name/" 2>/dev/null || true
                else
                    cp -r "$ext" "$BUILD_DIR/openclaw/extensions/" 2>/dev/null || true
                fi
            fi
        done
        log_opt "  ✓ extensions 已复制"
    fi

    # 根据模式决定是否复制 src/
    if [ "$MODE" = "full" ] && [ "$OPTIMIZE" = false ]; then
        log_warn "复制 src/ 目录（开发用，会增加 32MB）"
        cp -r src "$BUILD_DIR/openclaw/" 2>/dev/null || true
    else
        log_opt "跳过 src/ 目录（运行时不需要）"
    fi

    # 创建启动脚本
    log_opt "创建启动脚本..."
    cat > "$BUILD_DIR/openclaw/start.sh" << 'EOF'
#!/bin/bash
DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"
export NODE_PATH="$DIR/node_modules"
if [ -f "dist/index.js" ]; then
    exec node dist/index.js "$@"
else
    echo "Error: OpenClaw not built. Run: npm install && npm run build"
    exit 1
fi
EOF
    chmod +x "$BUILD_DIR/openclaw/start.sh"

    # Windows 启动脚本
    cat > "$BUILD_DIR/openclaw/start.bat" << 'EOF'
@echo off
setlocal
set "DIR=%~dp0"
cd /d "%DIR%"
set "NODE_PATH=%DIR%\node_modules"
if exist "dist\index.js" (
    node dist\index.js %*
) else (
    echo Error: OpenClaw not built.
    exit /b 1
)
EOF

    # 创建版本文件
    local version=$(get_version)
    echo "$version" > "$BUILD_DIR/openclaw/VERSION"

    # Full 模式：复制并优化 node_modules
    if [ "$MODE" = "full" ]; then
        log_step "复制 node_modules（完整模式）..."

        if [ -d "node_modules" ]; then
            log_info "原始 node_modules 大小: $(calc_dir_size node_modules)"

            # 复制 node_modules
            if [ "$OPTIMIZE" = true ]; then
                log_opt "使用智能复制策略（保留所有依赖但清理开发文件）..."

                # 创建目标目录
                mkdir -p "$BUILD_DIR/openclaw/node_modules"

                # 方案：复制整个 node_modules 但排除/清理大体积的开发内容
                # 1. 先复制 .pnpm 目录（实际存储所有包的地方）
                if [ -d "node_modules/.pnpm" ]; then
                    log_opt "复制 .pnpm 目录（包存储库）..."
                    mkdir -p "$BUILD_DIR/openclaw/node_modules/.pnpm"

                    # 使用 tar 进行复制，可以显示进度
                    local pnpm_size=$(du -sm node_modules/.pnpm | cut -f1)
                    log_info ".pnpm 原始大小: ${pnpm_size}MB"

                    # 复制 .pnpm 但排除开发工具和平台特定的大文件
                    find node_modules/.pnpm -maxdepth 1 -type d | while read -r pkg_dir; do
                        local pkg_name=$(basename "$pkg_dir")

                        # 跳过开发工具相关的大包
                        case "$pkg_name" in
                            *typescript*native-preview*|*oxlint*|*oxfmt*|*esbuild*darwin*|*rolldown*binding*|*vitest*|*playwright*)
                                log_opt "  跳过开发工具: $pkg_name"
                                continue
                                ;;
                        esac

                        # 复制包目录
                        cp -r "$pkg_dir" "$BUILD_DIR/openclaw/node_modules/.pnpm/" 2>/dev/null || true
                    done

                    # 复制 .pnpm 的元数据文件
                    cp node_modules/.pnpm/modules.yaml "$BUILD_DIR/openclaw/node_modules/.pnpm/" 2>/dev/null || true
                fi

                # 2. 复制顶层依赖的软链接/目录结构
                log_opt "复制顶层依赖链接..."
                for item in node_modules/*; do
                    if [ -d "$item" ]; then
                        local name=$(basename "$item")

                        # 跳过 .pnpm 和 .bin（已处理或不需要）
                        case "$name" in
                            .pnpm|.bin) continue ;;
                        esac

                        # 复制符号链接（相对链接）
                        if [ -L "$item" ]; then
                            cp -P "$item" "$BUILD_DIR/openclaw/node_modules/" 2>/dev/null || true
                        else
                            # 复制目录
                            cp -r "$item" "$BUILD_DIR/openclaw/node_modules/" 2>/dev/null || true
                        fi
                    fi
                done

                # 3. 复制 .bin 目录（启动脚本）
                if [ -d "node_modules/.bin" ]; then
                    log_opt "复制 .bin 启动脚本..."
                    cp -r node_modules/.bin "$BUILD_DIR/openclaw/node_modules/" 2>/dev/null || true
                fi

                # 4. 复制模块元数据
                cp node_modules/.modules.yaml "$BUILD_DIR/openclaw/node_modules/" 2>/dev/null || true

                # 5. 深度清理复制的 node_modules
                log_opt "深度清理开发文件..."
                clean_node_modules "$BUILD_DIR/openclaw/node_modules"

            else
                # 无优化：全量复制
                log_warn "全量复制 node_modules（无优化，体积巨大）..."
                cp -r node_modules "$BUILD_DIR/openclaw/"
            fi

            log_info "复制后大小: $(calc_dir_size "$BUILD_DIR/openclaw/node_modules")"
        else
            log_warn "node_modules 不存在"
        fi
    else
        log_opt "跳过 node_modules（lite 模式）"

        # Lite 模式：创建安装说明
        cat > "$BUILD_DIR/openclaw/INSTALL.md" << EOF
# OpenClaw 安装说明

## 快速开始

1. 确保已安装 Node.js 20+
2. 安装依赖：

\`\`\`bash
cd openclaw
npm install
# 或
pnpm install
\`\`\`

3. 启动 OpenClaw：

\`\`\`bash
./start.sh
# Windows: start.bat
\`\`\`

## 包信息

- 版本: $(get_version)
- 模式: Lite（需安装依赖）
- 平台: $PLATFORM-$ARCH
- 打包时间: $(date +%Y-%m-%d)
EOF
    fi

    # 显示打包前大小统计
    echo ""
    log_info "打包内容统计:"
    du -sh "$BUILD_DIR/openclaw/" 2>/dev/null || echo "  (统计失败)"
    for subdir in dist assets extensions skills node_modules; do
        if [ -d "$BUILD_DIR/openclaw/$subdir" ]; then
            local size=$(du -sh "$BUILD_DIR/openclaw/$subdir" 2>/dev/null | cut -f1)
            echo "  - $subdir/: $size"
        fi
    done

    # 打包
    log_step "创建压缩包..."
    cd "$BUILD_DIR"

    local version=$(get_version)
    local timestamp=$(date +%Y%m%d)
    local base_name="openclaw-v${version}-${PLATFORM}-${ARCH}-${timestamp}"

    # 计算总文件数和大小
    local total_files=$(find openclaw -type f | wc -l)
    local total_size=$(du -sm openclaw | cut -f1)
    log_info "待压缩: $total_files 个文件, 共 ${total_size}MB"

    if [ "$PLATFORM" = "windows" ]; then
        FILENAME="${base_name}.zip"
        if command -v zip &> /dev/null; then
            # 使用 zip 并显示进度
            log_info "正在压缩 (zip)..."
            zip -r "$FILENAME" openclaw/ -x "*.DS_Store" 2>&1 | \
            while IFS= read -r line; do
                if [[ "$line" == "  adding:"* ]]; then
                    current=$((current + 1))
                    if [ $((current % 50)) -eq 0 ]; then
                        show_progress $current $total_files "Compressing"
                    fi
                fi
            done
        else
            FILENAME="${base_name}.tar.gz"
            compress_with_progress "openclaw" "$FILENAME" "tar"
        fi
    else
        FILENAME="${base_name}.tar.gz"
        # 使用 tar 带进度显示
        log_info "正在压缩 (tar.gz)..."
        if command -v pv &> /dev/null; then
            tar -cf - openclaw/ --exclude=".DS_Store" 2>/dev/null | \
                pv -s ${total_size}m | gzip > "$FILENAME"
        else
            # 使用带有进度回调的 tar
            tar -czf "$FILENAME" openclaw/ --exclude=".DS_Store" 2>/dev/null &
            local tar_pid=$!
            local current_size=0

            while kill -0 $tar_pid 2>/dev/null; do
                if [ -f "$FILENAME" ]; then
                    current_size=$(du -sm "$FILENAME" 2>/dev/null | cut -f1)
                    # 压缩后通常会是原来的 20-30%，所以这里调整进度
                    local estimated=$((current_size * 3))
                    if [ $estimated -gt $total_size ]; then
                        estimated=$total_size
                    fi
                    show_progress $estimated $total_size "Compressing"
                fi
                sleep 0.5
            done
            wait $tar_pid
            echo ""  # 换行
        fi
    fi

    # 检查大小
    local size_mb=$(check_package_size "$FILENAME")
    local file_size=$(du -h "$FILENAME" | cut -f1)

    echo ""
    log_info "✓ 压缩完成: $file_size"

    # 大小限制检查
    if [ "$size_mb" -gt "$MAX_PACKAGE_SIZE_MB" ]; then
        log_warn "包大小 ($size_mb MB) 超过限制 ($MAX_PACKAGE_SIZE_MB MB)"
        log_info "建议: 使用 --lite 模式或进一步启用优化"
    fi

    # 确保目标目录存在
    mkdir -p "$BUNDLED_DIR"

    # 移动包到目标目录
    mv "$FILENAME" "$BUNDLED_DIR/"

    log_info "✓ 打包完成: $FILENAME"

    # 清理临时目录
    cleanup
}

# 清理
cleanup() {
    if [ -n "$BUILD_DIR" ] && [ -d "$BUILD_DIR" ]; then
        rm -rf "$BUILD_DIR"
    fi
}

# 复制到 Manager
copy_to_manager() {
    log_step "复制到 Manager..."

    log_info "✓ 包已保存到: $BUNDLED_DIR/$FILENAME"
    echo ""
    log_info "当前包列表:"
    ls -lh "$BUNDLED_DIR"/openclaw-*.tar.gz "$BUNDLED_DIR"/openclaw-*.zip 2>/dev/null | tail -5 || true
}

# 下载 Node.js 运行时
download_nodejs_runtime() {
    log_step "下载 Node.js 运行时..."

    local runtimes_dir="$BUNDLED_DIR/runtimes"
    mkdir -p "$runtimes_dir"

    # 检测平台
    local platform arch ext
    case "$PLATFORM" in
        macos)
            platform="darwin"
            ext=".tar.gz"
            ;;
        linux)
            platform="linux"
            ext=".tar.xz"
            ;;
        windows)
            platform="win32"
            ext=".zip"
            ;;
        *)
            log_error "不支持的平台: $PLATFORM"
            return 1
            ;;
    esac

    case "$ARCH" in
        x64) arch="x64" ;;
        arm64) arch="arm64" ;;
        *) arch="$ARCH" ;;
    esac

    local node_version="22.14.0"  # 使用最新的 LTS 版本
    local archive_name="node-v${node_version}-${platform}-${arch}${ext}"
    local download_url="https://nodejs.org/dist/v${node_version}/${archive_name}"
    local output_file="$runtimes_dir/${archive_name}"

    # 检查是否已存在
    if [ -f "$output_file" ]; then
        log_info "Node.js 运行时已存在: $archive_name"
        return 0
    fi

    log_info "下载: $download_url"
    log_info "保存到: $output_file"

    # 使用 curl 下载
    if ! curl -fsSL --progress-bar "$download_url" -o "$output_file"; then
        log_warn "下载 Node.js 失败，使用系统 Node.js 作为备选"
        return 1
    fi

    log_info "✓ Node.js 运行时下载完成"
}

# 主流程
main() {
    echo "========================================"
    echo "OpenClaw 优化打包脚本"
    echo "========================================"
    echo ""

    parse_args "$@"

    log_info "模式: $MODE"
    log_info "体积优化: $OPTIMIZE"
    log_info "大小限制: ${MAX_PACKAGE_SIZE_MB}MB"
    log_info "OpenClaw 版本: $(get_version)"
    log_info "源码目录: $OPENCLAW_DIR"
    echo ""

    detect_platform
    prepare_openclaw
    package_openclaw

    # 下载 Node.js 运行时（用于 Sidecar 模式）
    download_nodejs_runtime || true

    copy_to_manager

    # 清理旧包
    clean_old_packages

    echo ""
    echo "========================================"
    echo "✅ 打包完成!"
    echo "========================================"
    echo ""
    echo "💡 使用提示:"
    if [ "$MODE" = "lite" ]; then
        echo "  此包为 Lite 模式，用户首次运行需要执行 npm install"
        echo "  如需即开即用，请使用 --full 模式（体积更大）"
    else
        echo "  此包为 Full 模式，可直接运行"
    fi
    echo ""
    echo "下一步:"
    echo "  npm run tauri:dev     (开发模式)"
    echo "  npm run tauri:build   (生产构建)"
    echo ""
}

# 错误处理
trap cleanup EXIT

main "$@"
