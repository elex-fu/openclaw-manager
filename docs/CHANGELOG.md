# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- 计划支持更多模型提供商
- 计划添加插件市场
- 计划支持多语言界面

## [2.0.0] - 2024-03-07

### Added

#### 跨平台支持增强
- **ARM64 架构支持** - 完整支持 Apple Silicon 和 ARM64 Linux/Windows
  - macOS ARM64 (Apple Silicon) 原生支持
  - Linux ARM64 (aarch64) 支持
  - Windows ARM64 支持
  - 条件编译优化单平台构建体积

#### 安全存储改进
- **Linux 加密文件存储降级** - 无 D-Bus 环境自动切换
  - AES-256-GCM 加密算法
  - 机器 ID 绑定（防止文件复制到其他机器）
  - 自动检测 keyring 可用性
  - 无缝降级到加密文件存储

#### 进程管理完善
- **Windows 优雅关闭** - 三级关闭策略
  - Ctrl+Break 信号发送
  - WM_CLOSE 消息发送
  - 强制终止 (taskkill /F /T)
  - 可配置超时时间

- **Unix 信号处理** - 三级信号策略
  - SIGTERM 优雅终止
  - SIGINT 中断信号
  - SIGKILL 强制终止

#### 离线安装优化
- **条件编译支持** - 6 平台独立构建
  - macOS x64 / ARM64
  - Windows x64 / ARM64
  - Linux x64 / ARM64
  - 单平台构建体积减少约 40%

### Changed
- 优化安全存储后端自动选择逻辑
- 改进进程关闭的超时处理
- 增强跨平台构建脚本

### Fixed
- Linux 无 D-Bus 环境无法存储 API 密钥的问题
- Windows 进程强制终止导致数据丢失的问题
- 离线安装包平台检测不准确的问题

## [0.1.1] - 2024-03-03

### Added
- **Agent Management System** - Complete agent configuration and management
  - Create, edit, and delete custom AI agents
  - System prompt editor with syntax highlighting
  - Agent skill assignment and configuration
  - Current agent switching with `setCurrentAgent()`
  - Agent avatar and metadata support

- **Diagnostics and Auto-Fix** - Comprehensive system diagnostics
  - Multi-category health checks (system, openclaw, network, service)
  - Real-time diagnostic results with severity levels
  - One-click auto-fix for common issues
  - Individual issue fixing capability
  - Diagnostic alerts with fix suggestions

- **Settings Page with Secure API Key Storage** - Enhanced security
  - System settings management (theme, language, log level)
  - Secure API key storage using OS keychain
  - Support for multiple provider API keys (OpenAI, Anthropic, Google, Azure)
  - API key encryption at rest
  - Startup settings (auto-start, minimize to tray)

- **Log Viewer with WebSocket Streaming** - Real-time log monitoring
  - Live log streaming via WebSocket
  - Log level filtering (ERROR, WARN, INFO, DEBUG, TRACE)
  - Multi-source log aggregation
  - Log search and filtering
  - Log export to text/JSON/CSV
  - Log statistics and analytics

- **Skill Store** - Skill marketplace and management
  - Browse and search skill marketplace
  - Install/uninstall skills with dependency resolution
  - Enable/disable skills dynamically
  - Skill configuration management
  - Skill update checking
  - Popular and latest skills discovery

- **Plugin Marketplace API** - Plugin discovery and installation
  - Search plugins with filtering and sorting
  - Plugin categories and tags
  - Popular and latest plugin listings
  - Detailed plugin information and ratings
  - One-click plugin installation

- **UI/UX Improvements** - Enhanced user experience
  - Dark mode support with system preference detection
  - Smooth animations and transitions
  - Responsive design improvements
  - Toast notifications for user feedback
  - Loading states and skeleton screens

- **Model Configuration** - Enhanced model management
  - Support for multiple model providers
  - Model connection testing with latency measurement
  - Model prioritization and reordering
  - Default model selection
  - Runtime model configuration with API key injection

- **System Operations** - System information and monitoring
  - System resource monitoring (CPU, memory, disk)
  - Recent activities tracking
  - System environment checking
  - Platform-specific optimizations

- **Update Operations** - Application update management
  - Automatic update checking
  - Download and install updates
  - Offline update support
  - Backup and restore functionality
  - Update progress tracking

### Changed
- Improved error handling with retry mechanisms
- Enhanced API response consistency
- Optimized frontend state management with Zustand
- Better TypeScript type coverage

### Fixed
- Fixed service status detection issues
- Resolved configuration save race conditions
- Fixed plugin installation errors
- Corrected log streaming connection handling

## [1.0.0] - 2024-03-01

### Added
- **初始版本发布** - OpenClaw Manager 首个正式版本
- **一键安装** - 支持自动检测系统环境并安装 OpenClaw
  - 自动检测操作系统和架构
  - 支持 macOS (Intel/Apple Silicon)、Windows、Linux
  - 支持离线安装模式
  - 安装进度实时显示
- **可视化配置管理** - 图形化界面管理 OpenClaw 配置
  - 树形配置编辑器
  - 配置验证和错误提示
  - 配置导入/导出功能
  - YAML 语法高亮
- **模型管理** - 管理 AI 模型配置
  - 支持 OpenAI、Anthropic、Azure、Google 等主流提供商
  - 支持本地模型 (Ollama、LM Studio)
  - API 密钥安全存储（系统钥匙串）
  - 模型连接测试
  - 模型优先级设置
- **Agent 管理** - 配置和管理 AI Agent
  - 创建自定义 Agent
  - 系统提示词编辑器
  - Agent 能力配置
  - Agent 调试工具
- **服务控制** - 启动/停止/重启 OpenClaw 服务
  - 服务状态实时监控
  - 自动重启配置
  - 服务健康检查
- **日志查看** - 实时日志监控
  - 日志级别筛选
  - 日志搜索功能
  - 日志导出
  - 自动滚动
- **系统监控** - 资源使用情况监控
  - CPU 使用率
  - 内存使用情况
  - 磁盘空间监控
- **诊断工具** - 系统诊断和故障排查
  - 环境检查
  - 网络连接测试
  - 依赖项检测
  - 一键修复功能
- **插件系统** - 可扩展的插件架构
  - 插件安装/卸载/更新
  - 插件配置管理
  - 支持 Lua、JavaScript、WASM 插件
- **安全存储** - 敏感信息安全管理
  - API 密钥系统级安全存储
  - 配置加密
  - 访问控制
- **自动更新** - 应用自动更新功能
  - 检查更新
  - 自动下载和安装
  - 更新通道选择（稳定版/测试版）
- **备份恢复** - 配置和数据备份
  - 一键备份
  - 选择性恢复
  - 自动备份计划
- **多实例支持** - 管理多个 OpenClaw 实例
  - 实例切换
  - 实例特定配置
- **托盘图标** - 系统托盘集成
  - 快速访问常用功能
  - 服务状态指示
  - 最小化到托盘

### Changed
- 优化安装流程，提升安装速度
- 改进配置编辑器用户体验
- 优化日志显示性能

### Fixed
- 修复某些情况下配置保存失败的问题
- 修复 Windows 下的路径处理问题
- 修复服务状态显示不准确的问题

### Security
- 实现 API 密钥安全存储机制
- 添加配置加密功能
- 实现安全的插件沙箱环境

## [0.9.0] - 2024-02-15

### Added
- Beta 版本发布
- 完整的安装流程实现
- 基础配置管理功能
- 模型管理原型
- 服务控制功能

### Changed
- 重构前端架构
- 优化 Tauri 命令结构

### Fixed
- 修复多个 UI 显示问题
- 修复配置解析错误

## [0.8.0] - 2024-01-20

### Added
- Alpha 版本发布
- 基础界面框架
- 系统检测功能
- 简单的安装流程

### Changed
- 升级至 Tauri v2
- 重构状态管理

## [0.1.0] - 2024-01-01

### Added
- 项目初始化
- 基础架构搭建
- 技术选型确定
- 开发环境配置

---

## 版本说明

### 版本号格式

本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/) 规范：

版本号格式：主版本号.次版本号.修订号（MAJOR.MINOR.PATCH）

- **主版本号（MAJOR）**：当进行不兼容的 API 修改时递增
- **次版本号（MINOR）**：当以向后兼容的方式添加功能时递增
- **修订号（PATCH）**：当进行向后兼容的问题修复时递增

### 预发布版本标识

- `alpha`: 内部测试版本，功能不完整
- `beta`: 公开测试版本，功能基本完整但可能有 Bug
- `rc` (Release Candidate): 候选发布版本，即将正式发布

### 更新类型说明

- **Added**: 新增功能
- **Changed**: 对现有功能的变更
- **Deprecated**: 即将移除的功能
- **Removed**: 已移除的功能
- **Fixed**: 问题修复
- **Security**: 安全相关的修复

---

## 升级指南

### 升级到 1.0.0

从 0.9.x 升级到 1.0.0：

1. **备份配置**
   ```bash
   cp -r ~/.openclaw ~/.openclaw.backup
   ```

2. **更新应用**
   - 通过自动更新功能更新
   - 或从 Releases 页面下载新版本

3. **迁移配置**（如需要）
   - 应用会自动迁移配置
   - 检查配置是否正确加载

4. **验证安装**
   - 启动服务
   - 测试模型连接
   - 检查日志是否有错误

### 降级说明

降级到旧版本时：
1. 备份当前配置
2. 卸载当前版本
3. 安装旧版本
4. 恢复配置（注意配置格式兼容性）

---

## 历史版本归档

旧版本的历史记录可以在 [Releases](https://github.com/openclaw/openclaw-manager/releases) 页面查看。

---

*Changelog 维护者: OpenClaw Team*
*最后更新: 2024-03-03*
