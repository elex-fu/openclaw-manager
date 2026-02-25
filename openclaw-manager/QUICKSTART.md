# OpenClaw Manager 快速开始指南

## 一键部署

### 方式1: 使用部署脚本 (推荐)

```bash
# 进入项目目录
cd openclaw-manager

# 运行部署脚本
./deploy.sh
```

脚本会引导你完成：
1. 环境检查 (Node.js, Rust)
2. 安装前端依赖
3. 启动开发服务器

### 方式2: 手动部署

#### 1. 环境要求

- **Node.js** >= 18.x
- **Rust** >= 1.70

#### 2. 安装依赖

```bash
# 安装前端依赖
npm install
```

#### 3. 启动开发模式

```bash
# 同时启动前端和 Tauri
npm run tauri:dev
```

或分别启动：

```bash
# 终端1: 启动前端开发服务器
npm run dev

# 终端2: 启动 Tauri
npm run tauri:dev
```

## 功能使用

### 文件扫描

1. 打开应用后，进入"文件列表"页面
2. 在"文件扫描"卡片中选择文件夹
3. 点击"开始扫描"按钮
4. 扫描完成后，文件会显示在列表中

### 分组管理

1. 进入"分组管理"页面
2. 点击"新建分组"创建新分组
3. 可以给分组设置名称、描述、颜色

### 文件分类

1. 在文件列表中，点击文件右侧的"已收录"按钮
2. 文件会被标记为已收录
3. 可以按状态筛选文件

## 构建生产版本

```bash
# 构建生产版本
npm run tauri:build
```

构建完成后，应用位于：
- **macOS**: `src-tauri/target/release/bundle/macos/OpenClaw Manager.app`
- **Windows**: `src-tauri/target/release/bundle/msi/`
- **Linux**: `src-tauri/target/release/bundle/deb/`

## 常见问题

### 1. Rust 未找到

确保 Rust 已安装并添加到 PATH：

```bash
# 检查 Rust
rustc --version
cargo --version

# 如果未找到，手动添加 PATH
export PATH="$HOME/.cargo/bin:$PATH"
# 或 (MacPorts 安装)
export PATH="/opt/local/bin:$PATH"
```

### 2. 前端依赖安装失败

尝试使用国内镜像：

```bash
# 使用淘宝镜像
npm config set registry https://registry.npmmirror.com

# 然后重新安装
npm install
```

### 3. Tauri 构建失败

检查是否安装了必要的系统依赖：

**macOS:**
```bash
xcode-select --install
```

**Linux:**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev libssl-dev
```

## 项目结构

```
openclaw-manager/
├── src/                    # 前端代码 (React + TypeScript)
│   ├── components/        # UI 组件
│   ├── pages/            # 页面
│   └── lib/              # 工具函数
├── src-tauri/             # Rust 后端
│   └── src/
│       ├── commands/     # Tauri 命令
│       ├── models/       # 数据模型
│       └── db/          # 数据库
└── deploy.sh            # 一键部署脚本
```

## 技术支持

如有问题，请查看：
- [Tauri 文档](https://tauri.app/)
- [React 文档](https://react.dev/)
