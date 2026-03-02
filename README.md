# OpenClaw Manager

<p align="center">
  <img src="./openclaw-manager/src-tauri/icons/icon.png" alt="OpenClaw Manager Logo" width="120">
</p>

<p align="center">
  <strong>OpenClaw 桌面管理工具</strong> - 一站式 AI 网关安装、配置与管理平台
</p>

<p align="center">
  <a href="#功能特性">功能特性</a> •
  <a href="#安装">安装</a> •
  <a href="#快速开始">快速开始</a> •
  <a href="#文档">文档</a> •
  <a href="#技术栈">技术栈</a> •
  <a href="#贡献">贡献</a>
</p>

---

## 简介

**OpenClaw Manager** 是一个基于 Tauri 开发的跨平台桌面应用程序，专为简化 [OpenClaw](https://github.com/openclaw/openclaw) AI 网关的安装、配置和管理而设计。无论您是初次接触 AI 网关的新手，还是需要高效管理多个实例的专业用户，OpenClaw Manager 都能提供直观的可视化界面，让复杂的命令行操作变得简单易懂。

### 核心优势

- **一键安装**：自动检测系统环境，一键完成 OpenClaw 安装
- **可视化配置**：图形化界面管理所有配置项，告别繁琐的 YAML 编辑
- **实时监控**：查看服务状态、资源使用情况和日志输出
- **跨平台支持**：完美支持 macOS、Windows 和 Linux

---

## 功能特性

### 安装管理
- [x] 自动检测操作系统和架构
- [x] 一键下载和安装 OpenClaw
- [x] 支持离线安装模式
- [x] 安装进度实时显示
- [x] 自动配置环境变量

### 配置管理
- [x] 可视化编辑 OpenClaw 配置
- [x] 模型配置管理（添加、编辑、删除）
- [x] Agent 配置管理
- [x] 插件管理
- [x] 配置文件导入/导出
- [x] 配置验证和错误提示

### 服务控制
- [x] 启动/停止 OpenClaw 服务
- [x] 服务状态实时监控
- [x] 系统资源监控（CPU、内存、磁盘）
- [x] 实时日志查看
- [x] 自动重启配置

### 安全与存储
- [x] API 密钥安全存储（系统钥匙串）
- [x] 配置加密存储
- [x] 访问控制管理

### 诊断工具
- [x] 系统环境检查
- [x] 网络连接测试
- [x] 依赖项检测
- [x] 一键修复常见问题

---

## 安装

### 系统要求

| 平台 | 最低版本 | 架构 |
|------|----------|------|
| macOS | 11.0 (Big Sur) | Intel / Apple Silicon |
| Windows | Windows 10 | x64 |
| Linux | Ubuntu 20.04+ | x64 / ARM64 |

### macOS

#### 通过 Homebrew 安装（推荐）

```bash
brew tap openclaw/manager
brew install openclaw-manager
```

#### 通过 DMG 安装

1. 从 [Releases](https://github.com/openclaw/openclaw-manager/releases) 页面下载最新版本的 `.dmg` 文件
2. 双击打开 DMG 文件
3. 将 OpenClaw Manager 拖入 Applications 文件夹
4. 首次运行时，前往 **系统设置 > 隐私与安全性** 允许应用运行

### Windows

#### 通过 Winget 安装（推荐）

```powershell
winget install OpenClaw.Manager
```

#### 通过 MSI 安装

1. 从 [Releases](https://github.com/openclaw/openclaw-manager/releases) 页面下载最新版本的 `.msi` 文件
2. 双击运行安装程序
3. 按照向导完成安装

### Linux

#### 通过 AppImage 安装

```bash
# 下载 AppImage
wget https://github.com/openclaw/openclaw-manager/releases/latest/download/openclaw-manager.AppImage

# 添加执行权限
chmod +x openclaw-manager.AppImage

# 运行
./openclaw-manager.AppImage
```

#### 通过 DEB/RPM 安装

```bash
# Debian/Ubuntu
sudo dpkg -i openclaw-manager_1.0.0_amd64.deb

# RHEL/CentOS/Fedora
sudo rpm -i openclaw-manager-1.0.0.x86_64.rpm
```

---

## 快速开始

### 1. 首次启动

启动 OpenClaw Manager 后，您将看到欢迎界面：

![Welcome Screen](./docs/screenshots/welcome.png)

### 2. 安装 OpenClaw

点击 **"一键安装"** 按钮，OpenClaw Manager 将自动：
- 检测您的系统环境
- 下载适合的 OpenClaw 版本
- 完成安装和配置

![Install Screen](./docs/screenshots/install.png)

### 3. 配置模型

安装完成后，前往 **"模型管理"** 页面：
- 点击 **"添加模型"**
- 选择模型提供商（OpenAI、Anthropic、Azure 等）
- 输入 API 密钥（将安全存储在系统钥匙串中）
- 配置模型参数

![Models Screen](./docs/screenshots/models.png)

### 4. 启动服务

在 **"控制台"** 页面点击 **"启动服务"**，OpenClaw 将开始运行。

### 5. 验证安装

服务启动后，您可以：
- 查看实时日志确认服务正常运行
- 使用内置的诊断工具验证配置
- 测试 API 连接

---

## 截图

### 主界面
<p align="center">
  <img src="./docs/screenshots/dashboard.png" alt="Dashboard" width="800">
</p>

### 配置管理
<p align="center">
  <img src="./docs/screenshots/config.png" alt="Configuration" width="800">
</p>

### 模型管理
<p align="center">
  <img src="./docs/screenshots/models.png" alt="Model Management" width="800">
</p>

### 日志查看
<p align="center">
  <img src="./docs/screenshots/logs.png" alt="Log Viewer" width="800">
</p>

---

## 文档

- [用户指南](./docs/USER_GUIDE.md) - 完整的用户手册
- [常见问题 (FAQ)](./docs/FAQ.md) - 常见问题解答
- [故障排除](./docs/TROUBLESHOOTING.md) - 问题解决指南
- [更新日志](./docs/CHANGELOG.md) - 版本更新记录
- [API 文档](./docs/API.md) - 开发者 API 参考

---

## 技术栈

### 后端
- **Rust** - 高性能系统编程语言
- **Tauri v2** - 跨平台桌面应用框架
- **Tokio** - 异步运行时
- **SQLite** - 本地数据存储

### 前端
- **React 18** - 用户界面库
- **TypeScript** - 类型安全的 JavaScript
- **Tailwind CSS** - 实用优先的 CSS 框架
- **shadcn/ui** - 高质量的 React 组件
- **Zustand** - 轻量级状态管理
- **TanStack Query** - 服务器状态管理

### 开发工具
- **Vite** - 快速的前端构建工具
- **Vitest** - 单元测试框架
- **Playwright** - E2E 测试框架
- **ESLint** - 代码质量检查

---

## 开发

### 环境要求

- **Node.js** >= 18.0
- **Rust** >= 1.70
- **npm** >= 9.0

### 克隆仓库

```bash
git clone https://github.com/openclaw/openclaw-manager.git
cd openclaw-manager/openclaw-manager
```

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
# 同时启动前端开发服务器和 Tauri 应用
npm run tauri:dev
```

### 构建

```bash
# 构建生产版本
npm run tauri:build

# 构建前端（仅 web）
npm run build
```

### 测试

```bash
# 运行单元测试
npm run test

# 运行测试并生成覆盖率报告
npm run test:coverage

# 运行 E2E 测试
npx playwright test

# 运行 Rust 测试
cd src-tauri && cargo test
```

### 代码规范

```bash
# 运行 ESLint
npm run lint
```

---

## 贡献

我们欢迎所有形式的贡献！请阅读我们的 [贡献指南](./CONTRIBUTING.md) 了解如何参与项目。

### 贡献方式

- **提交 Bug** - 使用 [Issue 模板](./.github/ISSUE_TEMPLATE/bug_report.md)
- **功能建议** - 使用 [功能请求模板](./.github/ISSUE_TEMPLATE/feature_request.md)
- **提交代码** - 阅读 [Pull Request 指南](./.github/PULL_REQUEST_TEMPLATE.md)
- **改进文档** - 文档改进同样受欢迎

### 行为准则

本项目遵循 [Contributor Covenant](https://www.contributor-covenant.org/) 行为准则。

---

## 许可证

OpenClaw Manager 采用 [MIT 许可证](./LICENSE) 开源。

```
MIT License

Copyright (c) 2024 OpenClaw Team

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## 致谢

- [Tauri](https://tauri.app/) - 优秀的跨平台桌面应用框架
- [shadcn/ui](https://ui.shadcn.com/) - 精美的 UI 组件库
- [OpenClaw](https://github.com/openclaw/openclaw) - 强大的 AI 网关

---

<p align="center">
  Made with ❤️ by the OpenClaw Team
</p>
