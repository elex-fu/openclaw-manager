# Sidecar 自动化 UI 测试指南

本文档介绍如何使用 Playwright 自动化测试 Sidecar 安装流程。

## 测试覆盖范围

### 1. 安装流程测试 (`sidecar-installation.spec.ts`)

- **Sidecar 安装选项显示** - 验证安装向导中显示 Sidecar 选项
- **安装启动** - 点击安装按钮后显示进度
- **Node.js 下载进度** - 验证 Node.js 运行时下载进度显示
- **npm install 进度** - 验证依赖安装进度
- **安装完成** - 验证安装成功并创建文件
- **导航到仪表盘** - 安装完成后自动跳转

### 2. 服务控制测试

- **服务状态显示** - 仪表盘显示 Sidecar 服务状态
- **启动服务** - 点击启动按钮启动 Sidecar 服务
- **停止服务** - 点击停止按钮停止服务
- **版本信息显示** - 显示 Sidecar/OpenClaw 版本

### 3. 错误处理测试

- **网络错误处理** - Node.js 下载失败时的回退
- **npm install 失败** - 显示错误和重试按钮
- **重试功能** - 失败后重新尝试安装

## 快速开始

### 安装依赖

```bash
# 安装 Playwright
npm install -D @playwright/test

# 安装浏览器
npx playwright install
```

### 运行测试

```bash
# 运行所有 Sidecar 测试
npx playwright test e2e/sidecar-installation.spec.ts

# 运行特定测试组
npx playwright test e2e/sidecar-installation.spec.ts --grep "Installation Flow"
npx playwright test e2e/sidecar-installation.spec.ts --grep "Service Control"

#  headed 模式（可见浏览器）
npx playwright test e2e/sidecar-installation.spec.ts --headed

# 调试模式
npx playwright test e2e/sidecar-installation.spec.ts --debug
```

### 测试配置

在项目根目录创建 `playwright.config.ts`:

```typescript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false, // Sidecar 测试需要串行执行
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Sidecar 安装测试需要单线程
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:1420',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: 'npm run tauri:dev',
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
});
```

## 测试工具函数

### 辅助函数 (`e2e/utils/sidecar-helpers.ts`)

```typescript
import {
  getSidecarStatus,
  clearSidecarInstallation,
  mockSidecarInstallation,
  waitForSidecarInstallation,
  startSidecarService,
  stopSidecarService,
} from './utils/sidecar-helpers';

// 使用示例
test('example', async ({ page }) => {
  // 清理现有安装
  clearSidecarInstallation();

  // 检查状态
  const status = getSidecarStatus();
  expect(status.installed).toBe(false);

  // 等待安装完成
  await waitForSidecarInstallation(page, { timeout: 300000 });
});
```

## CI/CD 集成

### GitHub Actions 示例

```yaml
name: Sidecar E2E Tests

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  sidecar-e2e:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [macos-latest, ubuntu-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Setup Rust
        uses: dtolnay/rust-action@stable

      - name: Install dependencies
        run: npm ci

      - name: Install Playwright
        run: npx playwright install --with-deps

      - name: Build Tauri app
        run: npm run tauri:build

      - name: Run Sidecar E2E tests
        run: npx playwright test e2e/sidecar-installation.spec.ts

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: playwright-report-${{ matrix.os }}
          path: |
            playwright-report/
            test-results/
```

## 测试数据属性

为了测试稳定性，前端代码需要添加 `data-testid` 属性：

```tsx
// 安装按钮
<button data-testid="sidecar-install" onClick={handleInstall}>
  Sidecar 安装
</button>

// 进度指示器
<div data-testid="install-progress" role="progressbar">
  <span>{percentage}%</span>
</div>

// 安装阶段
<div data-testid="install-stage">{stage}</div>

// 服务状态
<div data-testid="service-status">{status}</div>

// 已安装标识
<div data-testid="sidecar-installed">已安装</div>
```

## 最佳实践

### 1. 测试隔离

每个测试前清理 Sidecar 安装:

```typescript
test.beforeEach(async ({ page }) => {
  clearSidecarInstallation();
  await page.goto('/');
});
```

### 2. 超时设置

Sidecar 安装需要时间，设置合适的超时:

```typescript
test.setTimeout(300000); // 5 分钟
```

### 3. 条件跳过

某些测试需要前置条件:

```typescript
test('service control', async ({ page }) => {
  if (!getSidecarStatus().installed) {
    test.skip('Sidecar installation required');
  }
  // ...
});
```

### 4. 网络模拟

测试离线场景:

```typescript
await page.context().setOffline(true);
// ... 测试离线行为
await page.context().setOffline(false);
```

## 故障排除

### 测试失败常见原因

1. **应用未启动**
   - 确保 `webServer` 配置正确
   - 检查端口是否被占用

2. **Sidecar 安装超时**
   - 增加 `test.setTimeout()`
   - 检查网络连接

3. **元素找不到**
   - 添加 `data-testid` 属性
   - 增加等待时间

4. **文件系统权限**
   - CI 环境中确保有写入 `~/.openclaw` 的权限

### 调试技巧

```bash
# 查看浏览器控制台日志
npx playwright test --debug

# 保存跟踪记录
npx playwright test --trace=on

# 查看报告
npx playwright show-report
```

## 扩展测试

### 添加新的测试场景

```typescript
test('custom scenario', async ({ page }) => {
  // 1. 准备
  clearSidecarInstallation();
  await page.goto('/#/install');

  // 2. 执行操作
  await page.click('[data-testid="sidecar-install"]');

  // 3. 验证结果
  await expect(page.getByText('安装成功')).toBeVisible();

  // 4. 验证文件系统
  expect(getSidecarStatus().installed).toBe(true);
});
```

## 相关文件

- `e2e/sidecar-installation.spec.ts` - 主要测试文件
- `e2e/utils/sidecar-helpers.ts` - 测试工具函数
- `e2e/installation.spec.ts` - 现有安装测试
- `src/pages/InstallWizard.tsx` - 安装向导 UI
- `src/pages/Dashboard.tsx` - 仪表盘 UI

## 参考

- [Playwright 文档](https://playwright.dev/)
- [Tauri 测试指南](https://tauri.app/v1/guides/testing/)
- [CI/CD 最佳实践](../.github/workflows/README.md)
