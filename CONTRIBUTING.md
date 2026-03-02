# Contributing to OpenClaw Manager

首先，感谢您考虑为 OpenClaw Manager 做出贡献！正是像您这样的贡献者使这个项目变得更好。

## 目录

- [行为准则](#行为准则)
- [如何贡献](#如何贡献)
  - [报告 Bug](#报告-bug)
  - [建议新功能](#建议新功能)
  - [提交代码](#提交代码)
- [开发指南](#开发指南)
  - [环境设置](#环境设置)
  - [项目结构](#项目结构)
  - [代码规范](#代码规范)
  - [测试](#测试)
- [提交规范](#提交规范)
- [发布流程](#发布流程)

---

## 行为准则

本项目遵循 [Contributor Covenant](https://www.contributor-covenant.org/version/2/1/code_of_conduct/) 行为准则。参与本项目即表示您同意遵守此准则。

### 我们的承诺

- 使用友好和包容的语言
- 尊重不同的观点和经验
- 优雅地接受建设性批评
- 关注对社区最有利的事情
- 对其他社区成员表示同理心

---

## 如何贡献

### 报告 Bug

在提交 Bug 报告之前，请先：

1. **搜索现有 Issues** - 确保该问题尚未被报告
2. **检查最新版本** - 确认问题在最新版本中仍然存在
3. **收集信息** - 准备以下信息以便我们更好地理解问题：
   - 操作系统和版本
   - OpenClaw Manager 版本
   - 重现步骤
   - 预期行为与实际行为
   - 错误日志或截图

**提交 Bug 报告：**

使用我们的 [Bug 报告模板](https://github.com/openclaw/openclaw-manager/issues/new?template=bug_report.md) 创建 Issue，并包含以下信息：

```markdown
**描述**
清晰简洁地描述 Bug

**重现步骤**
1. 进入 '...'
2. 点击 '...'
3. 滚动到 '...'
4. 出现错误

**预期行为**
清晰描述您期望发生的情况

**截图**
如果适用，添加截图帮助解释问题

**环境信息：**
- 操作系统: [例如 macOS 14.0]
- 版本: [例如 1.0.0]
- 安装方式: [例如 Homebrew / DMG]

**附加信息**
任何其他上下文信息
```

### 建议新功能

**提交功能建议：**

1. **搜索现有建议** - 确保该功能尚未被建议
2. **检查路线图** - 查看该功能是否已在计划中
3. **创建 Feature Request**：

使用我们的 [功能请求模板](https://github.com/openclaw/openclaw-manager/issues/new?template=feature_request.md)：

```markdown
**功能描述**
清晰简洁地描述您想要的功能

**问题背景**
描述您遇到的问题或限制

**建议的解决方案**
描述您希望的解决方案

**替代方案**
描述您考虑过的其他替代方案

**附加信息**
任何其他上下文或截图
```

### 提交代码

#### 工作流程

1. **Fork 仓库**
   ```bash
   git clone https://github.com/YOUR_USERNAME/openclaw-manager.git
   cd openclaw-manager/openclaw-manager
   ```

2. **创建分支**
   ```bash
   git checkout -b feature/your-feature-name
   # 或
   git checkout -b fix/your-bug-fix
   ```

3. **进行更改**
   - 编写代码
   - 添加测试
   - 更新文档

4. **提交更改**
   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

5. **推送到 Fork**
   ```bash
   git push origin feature/your-feature-name
   ```

6. **创建 Pull Request**
   - 使用我们的 [PR 模板](https://github.com/openclaw/openclaw-manager/blob/main/.github/PULL_REQUEST_TEMPLATE.md)
   - 描述更改内容和原因
   - 关联相关 Issue

---

## 开发指南

### 环境设置

#### 必要条件

- **Node.js** >= 18.0
- **Rust** >= 1.70
- **npm** >= 9.0
- **Git**

#### 安装步骤

1. **克隆仓库**
   ```bash
   git clone https://github.com/openclaw/openclaw-manager.git
   cd openclaw-manager/openclaw-manager
   ```

2. **安装前端依赖**
   ```bash
   npm install
   ```

3. **安装 Rust 依赖**
   ```bash
   cd src-tauri
   cargo fetch
   cd ..
   ```

4. **验证安装**
   ```bash
   npm run tauri:dev
   ```

### 项目结构

```
openclaw-manager/
├── src/                    # 前端源代码
│   ├── components/         # React 组件
│   ├── pages/             # 页面组件
│   ├── stores/            # Zustand 状态管理
│   ├── lib/               # 工具函数和 API
│   ├── types/             # TypeScript 类型定义
│   └── test/              # 测试文件
├── src-tauri/             # Tauri/Rust 后端
│   ├── src/               # Rust 源代码
│   │   ├── commands/      # Tauri 命令
│   │   ├── services/      # 业务逻辑服务
│   │   ├── models/        # 数据模型
│   │   ├── errors/        # 错误处理
│   │   └── utils/         # 工具函数
│   ├── icons/             # 应用图标
│   └── bundled/           # 捆绑资源
├── docs/                  # 文档
├── e2e/                   # E2E 测试
└── scripts/               # 构建脚本
```

### 代码规范

#### TypeScript/JavaScript

我们使用 ESLint 和 Prettier 来保持代码风格一致：

```bash
# 运行代码检查
npm run lint

# 自动修复问题
npm run lint:fix

# 格式化代码
npm run format

# 检查格式
npm run format:check
```

**规范要点：**
- 使用单引号
- 缩进使用 2 个空格
- 最大行长度 100 字符
- 使用分号
- 尾随逗号（ES5 风格）

#### Rust

我们使用 `rustfmt` 和 `clippy`：

```bash
# 格式化代码
cd src-tauri
cargo fmt

# 运行 clippy
cargo clippy -- -D warnings
```

**规范要点：**
- 遵循 Rust 官方风格指南
- 所有公共 API 必须有文档注释
- 使用 `?` 运算符进行错误传播
- 避免 `unwrap()`，使用适当的错误处理

### 测试

#### 前端测试

```bash
# 运行所有测试
npm run test

# 运行特定测试文件
npm run test -- src/stores/__tests__/appStore.test.ts

# 运行测试并生成覆盖率报告
npm run test:coverage

# 以 UI 模式运行测试
npm run test:ui
```

#### Rust 测试

```bash
cd src-tauri

# 运行所有测试
cargo test

# 运行特定测试
cargo test test_install_directory_creation

# 运行测试并显示输出
cargo test -- --nocapture
```

#### E2E 测试

```bash
# 运行所有 E2E 测试
npx playwright test

# 运行特定测试
npx playwright test dashboard.spec.ts

# 以 UI 模式运行
npx playwright test --ui

# 生成测试报告
npx playwright show-report
```

**测试要求：**
- 新功能必须包含测试
- Bug 修复必须包含回归测试
- 保持测试覆盖率 > 80%

---

## 提交规范

我们使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

### 提交格式

```
<type>(<scope>): <subject>

[optional body]

[optional footer(s)]
```

### 类型说明

| 类型 | 说明 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `docs` | 文档更新 |
| `style` | 代码格式（不影响代码运行的变动）|
| `refactor` | 重构（既不是新增功能，也不是修改 Bug）|
| `perf` | 性能优化 |
| `test` | 添加测试 |
| `chore` | 构建过程或辅助工具的变动 |
| `ci` | CI/CD 相关变动 |

### 示例

```bash
# 新功能
feat(config): add import/export functionality

# Bug 修复
fix(installer): resolve path issue on Windows

# 文档
docs(readme): update installation instructions

# 性能优化
perf(logs): optimize log rendering for large files

# 重构
refactor(api): consolidate error handling
```

---

## 发布流程

### 版本号规范

我们遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/)：

- **MAJOR**: 不兼容的 API 修改
- **MINOR**: 向后兼容的功能新增
- **PATCH**: 向后兼容的问题修复

### 发布步骤

1. **更新版本号**
   - `package.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`

2. **更新 CHANGELOG**
   - 在 `docs/CHANGELOG.md` 中添加新版本

3. **创建发布分支**
   ```bash
   git checkout -b release/v1.0.0
   ```

4. **提交更改**
   ```bash
   git add .
   git commit -m "chore(release): bump version to 1.0.0"
   ```

5. **创建 Pull Request**
   - 标题: `Release v1.0.0`
   - 描述: 包含 CHANGELOG 内容

6. **合并后创建标签**
   ```bash
   git tag -a v1.0.0 -m "Release version 1.0.0"
   git push origin v1.0.0
   ```

7. **GitHub Release**
   - 基于标签创建 Release
   - 上传构建产物
   - 发布 Release Notes

---

## 获取帮助

如果您在贡献过程中需要帮助：

- **GitHub Discussions**: [参与讨论](https://github.com/openclaw/openclaw-manager/discussions)
- **Discord**: [加入我们的 Discord](https://discord.gg/openclaw)（如果有）
- **邮件**: dev@openclaw.io

---

## 致谢

感谢所有为 OpenClaw Manager 做出贡献的人！

特别感谢：
- [Tauri](https://tauri.app/) 团队提供的优秀框架
- [shadcn/ui](https://ui.shadcn.com/) 提供的精美组件
- 所有开源依赖的维护者

---

*最后更新: 2024-03-01*
