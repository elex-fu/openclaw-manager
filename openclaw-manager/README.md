# OpenClaw Manager

基于 Tauri 的文件管理工具，用于整理和分类各种文件。

## 技术栈

- **后端**: Rust + Tauri
- **前端**: React + TypeScript + Tailwind CSS
- **数据库**: SQLite
- **状态管理**: Zustand + TanStack Query
- **UI 组件**: shadcn/ui

## 功能特性

- 📁 文件扫描和管理
- 🏷️ 文件分组和分类
- 🔌 插件系统支持 (Lua/JavaScript/WASM)
- 🎨 现代化 UI 设计
- 🔒 安全沙箱环境
- 📊 文件信息解析

## 开发环境要求

- Node.js 18+
- Rust 1.70+
- npm 或 pnpm

## 安装依赖

```bash
# 进入项目目录
cd openclaw-manager

# 安装前端依赖
npm install

# 安装 Tauri CLI
npm install -g @tauri-apps/cli

# 运行开发服务器
npm run tauri:dev
```

## 构建

```bash
# 构建生产版本
npm run tauri:build
```

## 项目结构

```
openclaw-manager/
├── src/                      # 前端源代码
│   ├── components/          # UI 组件
│   │   ├── ui/             # shadcn/ui 组件
│   │   └── layout/         # 布局组件
│   ├── pages/              # 页面组件
│   ├── hooks/              # 自定义 hooks
│   ├── stores/             # 状态管理
│   ├── types/              # TypeScript 类型
│   ├── lib/                # 工具函数和 API
│   └── assets/             # 静态资源
├── src-tauri/              # Tauri/Rust 源代码
│   └── src/
│       ├── commands/       # Tauri 命令
│       ├── models/         # 数据模型
│       ├── db/            # 数据库模块
│       ├── plugins/       # 插件系统
│       └── utils/         # 工具函数
└── docs/                  # 文档
```

## 许可证

MIT License
