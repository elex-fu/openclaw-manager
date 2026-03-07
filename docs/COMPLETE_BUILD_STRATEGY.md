# OpenClaw Manager 完整构建方案（含 OpenClaw 构建）

## 问题分析

当前 OpenClaw 没有预编译的发布包，需要从源码构建：

```
OpenClaw 源码 → 编译 → 打包 → 嵌入 → OpenClaw Manager 构建 → 发布
```

## 方案一：集成构建（推荐）

在 GitHub Actions 中先构建 OpenClaw，再构建 Manager。

### 1.1 工作流程架构

```yaml
Build Pipeline:
├── Job 1: Build OpenClaw (6 platforms)
│   ├── Checkout openclaw repo
│   ├── Build binary for each platform
│   └── Upload as artifacts
│
├── Job 2: Build OpenClaw Manager
│   ├── Download OpenClaw artifacts
│   ├── Copy to bundled/
│   └── Build Tauri app
│
└── Job 3: Create Release
    ├── Collect all artifacts
    └── Create GitHub Release
```

### 1.2 目录结构

```
openclaw-manager/
├── .github/
│   └── workflows/
│       └── build-complete.yml      # 完整构建工作流
├── scripts/
│   ├── build-openclaw.sh           # 构建 OpenClaw 脚本
│   └── package-openclaw.sh         # 打包 OpenClaw 脚本
├── src-tauri/
│   └── bundled/                    # OpenClaw 二进制包存放处
└── third-party/
│   └── openclaw/                   # OpenClaw 源码（可选）
└── docs/
    └── COMPLETE_BUILD_STRATEGY.md  # 本文档
```

---

## 方案二：分离构建（灵活）

OpenClaw 和 Manager 分开构建，通过 Release Assets 关联。

### 2.1 工作流程

**仓库 A: openclaw** (核心程序)
```
推送 tag → 构建 6 平台二进制 → 发布 Release Assets
```

**仓库 B: openclaw-manager** (本仓库)
```
1. 下载 openclaw Release Assets
2. 嵌入到 bundled/
3. 构建 Manager
4. 发布 Release
```

### 2.2 优点

- OpenClaw 更新不需要重新构建 Manager
- 可以单独发布 OpenClaw 更新
- Manager 可以选择嵌入不同版本的 OpenClaw

---

## 方案对比

| 维度 | 方案一：集成构建 | 方案二：分离构建 |
|------|----------------|----------------|
| 复杂度 | 复杂（单流水线） | 中等（两流水线） |
| 构建时间 | 长（15-30分钟） | 中等（5-10分钟） |
| 灵活性 | 低 | 高 |
| 版本控制 | 简单 | 需要版本映射 |
| 维护成本 | 低 | 中等 |
| 推荐场景 | 快速迭代 | 正式发布 |

---

## 推荐：方案二（分离构建）

理由：
1. OpenClaw 作为后端服务，更新频率可能不同于 Manager
2. 用户可能只需要更新 OpenClaw 而不更新 Manager
3. 支持离线场景：Manager 可以检测并下载新版本的 OpenClaw

---

## 详细设计方案（方案二）

### 阶段 1：OpenClaw 构建流程

**触发条件**：`openclaw` 仓库推送 tag

**构建矩阵**：

| 平台 | 编译目标 | 输出格式 |
|------|---------|---------|
| macOS x64 | `x86_64-apple-darwin` | `openclaw-macos-x64.tar.gz` |
| macOS ARM64 | `aarch64-apple-darwin` | `openclaw-macos-arm64.tar.gz` |
| Windows x64 | `x86_64-pc-windows-msvc` | `openclaw-windows-x64.zip` |
| Windows ARM64 | `aarch64-pc-windows-msvc` | `openclaw-windows-arm64.zip` |
| Linux x64 | `x86_64-unknown-linux-gnu` | `openclaw-linux-x64.tar.gz` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `openclaw-linux-arm64.tar.gz` |

**包内容结构**：

```
openclaw/
├── bin/
│   └── openclaw          # 主可执行文件
├── lib/                  # 依赖库
├── config/
│   └── default.yaml      # 默认配置
└── README.md
```

### 阶段 2：Manager 构建流程

**触发条件**：`openclaw-manager` 仓库推送 tag

**步骤**：

1. **解析版本依赖**
   ```yaml
   # version-mapping.json
   {
     "manager_version": "0.1.0",
     "openclaw_version": "0.1.0",
     "min_openclaw_version": "0.1.0"
   }
   ```

2. **下载 OpenClaw 包**
   ```bash
   # 从 openclaw releases 下载
   curl -L https://github.com/openclai/openclaw/releases/download/v0.1.0/openclaw-${PLATFORM}-${ARCH}.tar.gz
   ```

3. **嵌入并构建**
   ```bash
   cp openclaw-*.tar.gz src-tauri/bundled/
   npm run tauri:build
   ```

4. **发布**
   - 上传 Manager 安装包
   - 记录版本映射关系

---

## 本地开发方案

### 快速测试（不构建 OpenClaw）

```bash
# 使用占位符模式
echo "Placeholder" > src-tauri/bundled/openclaw-macos-x64.tar.gz
npm run tauri:dev
```

### 完整本地构建

```bash
# 1. 克隆 OpenClaw
git clone https://github.com/openclai/openclaw.git ../openclaw
cd ../openclaw

# 2. 构建 OpenClaw（当前平台）
cargo build --release
cd ../openclaw-manager

# 3. 打包 OpenClaw
mkdir -p openclaw/bin
cp ../openclaw/target/release/openclaw openclaw/bin/
tar -czf src-tauri/bundled/openclaw-macos-x64.tar.gz openclaw/

# 4. 构建 Manager
npm run tauri:build
```

---

## 需要确认的问题

### 关于 OpenClaw

1. **源码仓库**: `https://github.com/openclai/openclaw` 是否正确？

2. **构建工具**:
   - [ ] Rust/Cargo (纯 Rust 项目)
   - [ ] Go (Go 项目)
   - [ ] 其他语言

3. **构建命令**:
   ```bash
   # 选项 A: 纯 Cargo
   cargo build --release

   # 选项 B: 需要 Makefile
   make build

   # 选项 C: 需要特殊步骤
   ./build.sh
   ```

4. **输出文件**:
   - [ ] 单个二进制文件
   - [ ] 二进制 + 配置文件
   - [ ] 二进制 + 动态库

5. **依赖**:
   - [ ] 静态链接（无依赖）
   - [ ] 需要运行时库

### 关于版本策略

6. **版本同步**:
   - [ ] Manager 和 OpenClaw 版本一致（都是 v0.1.0）
   - [ ] 独立版本（Manager v0.1.0 可以嵌入 OpenClaw v0.2.0）

7. **更新策略**:
   - [ ] 随 Manager 更新 OpenClaw
   - [ ] Manager 可以独立更新 OpenClaw
   - [ ] 两者都支持

请提供 OpenClaw 仓库信息和构建方式，我将生成完整的工作流文件。
