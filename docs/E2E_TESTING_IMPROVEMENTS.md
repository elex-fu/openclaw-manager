# E2E 测试改进计划

## 当前状态

### 测试覆盖率
- **总测试数**: 157 个
- **Sidecar 测试**: 18 个（新增）
- **通过**: 5/13 Sidecar 测试
- **需 UI 实现**: 4/13 Sidecar 测试
- **需前置条件**: 4/13 Sidecar 测试

### 执行结果

#### ✅ 已通过测试
1. `should display Node.js download progress` - 进度显示正常
2. `should show npm install progress` - npm 安装进度正常
3. `should navigate to dashboard after Sidecar installation` - 导航正常
4. `should show error when npm install fails` - 错误处理正常
5. `should allow retry after installation failure` - 重试功能正常

#### ❌ 失败测试（需前端 UI 实现）
1. `should display Sidecar installation option` - 缺少 `data-testid="sidecar-install"` 按钮
2. `should start Sidecar installation when clicked` - 缺少 Sidecar 安装按钮
3. `should complete installation and show success` - 缺少安装完成状态标识
4. `should handle network error during Node.js download` - 离线模式测试需优化

#### ⏭️ 跳过测试（前置条件未满足）
- 4 个服务控制测试 - 需要 Sidecar 已安装

---

## 需要提升的方面

### 1. 前端 UI 组件实现

#### 安装向导 (`src/pages/InstallWizard.tsx`)
```tsx
// 需要添加的组件
<button data-testid="sidecar-install" onClick={handleSidecarInstall}>
  Sidecar 安装
</button>

<div data-testid="install-progress" role="progressbar">
  <div style={{ width: `${percentage}%` }} />
  <span>{percentage}%</span>
</div>

<div data-testid="install-stage">{stage}</div>

<div data-testid="install-complete">安装成功</div>

<div data-testid="install-error">{errorMessage}</div>

<button data-testid="install-retry" onClick={handleRetry}>
  重试
</button>
```

#### 仪表盘服务状态 (`src/pages/Dashboard.tsx`)
```tsx
<div data-testid="service-status">{status}</div>

<div data-testid="sidecar-version">{version}</div>

<button data-testid="start-service" onClick={startService}>启动</button>

<button data-testid="stop-service" onClick={stopService}>停止</button>

<div data-testid="sidecar-installed">已安装</div>
```

### 2. 测试稳定性提升

#### 当前问题
- **超时**: 部分测试 5 分钟超时（`test.setTimeout(300000)`）
- **网络依赖**: Node.js 下载依赖外部网络
- **异步等待**: 缺乏可靠的完成状态检测

#### 改进方案

```typescript
// 1. 添加智能等待
export async function waitForSidecarInstallation(
  page: Page,
  options: { timeout?: number; checkInterval?: number } = {}
): Promise<void> {
  const { timeout = 300000, checkInterval = 5000 } = options;

  await page.waitForFunction(
    () => {
      // 检查多种完成条件
      const completeIndicator = document.querySelector('[data-testid="install-complete"]');
      const successText = document.querySelector('.install-success');
      const dashboardLink = document.querySelector('a[href="#/"]');
      return !!(completeIndicator || successText || dashboardLink);
    },
    { timeout }
  );
}

// 2. 使用更可靠的检测方式
export async function waitForProgressComplete(page: Page): Promise<void> {
  // 等待进度条达到 100% 并稳定
  await page.waitForFunction(
    () => {
      const progressText = document.querySelector('[data-testid="install-progress"]')?.textContent;
      if (!progressText) return false;

      const match = progressText.match(/(\d+)%/);
      if (!match) return false;

      return parseInt(match[1], 10) >= 100;
    },
    { timeout: 300000 }
  );

  // 额外等待确保状态更新
  await page.waitForTimeout(1000);
}
```

### 3. Mock 和测试隔离

#### 当前问题
- 测试相互依赖
- Sidecar 安装状态影响后续测试
- 缺乏有效的清理机制

#### 改进方案

```typescript
// 增强 test hooks
test.describe.configure({ mode: 'serial' }); // 确保串行执行

test.beforeEach(async ({ page }) => {
  // 1. 清理 Sidecar 安装
  clearSidecarInstallation();

  // 2. 清理浏览器状态
  await page.context().clearCookies();
  await page.context().clearPermissions();

  // 3. 重置应用状态
  await page.goto('/');
  await page.evaluate(() => {
    localStorage.clear();
    sessionStorage.clear();
  });
});

test.afterEach(async () => {
  // 强制清理
  clearSidecarInstallation();
});
```

### 4. 测试数据准备

#### 添加 Mock API

```typescript
// e2e/mocks/sidecar-mocks.ts
import { Page } from '@playwright/test';

export async function mockSidecarAPI(page: Page) {
  // Mock 安装状态检查
  await page.route('**/check_sidecar_installation', async (route) => {
    await route.fulfill({
      status: 200,
      body: JSON.stringify({
        success: true,
        data: { type: 'NotInstalled' }
      })
    });
  });

  // Mock 安装进度
  await page.route('**/install_openclaw_sidecar', async (route) => {
    await route.fulfill({
      status: 200,
      body: JSON.stringify({
        success: true,
        data: { version: '2026.2.27', message: '安装成功' }
      })
    });
  });

  // Mock 服务状态
  await page.route('**/get_sidecar_state', async (route) => {
    await route.fulfill({
      status: 200,
      body: JSON.stringify({
        success: true,
        data: { type: 'Stopped' }
      })
    });
  });
}
```

### 5. 视觉回归测试

#### 添加截图对比

```typescript
test('Sidecar installation UI', async ({ page }) => {
  await page.goto('/#/install');

  // 截图对比
  await expect(page).toHaveScreenshot('install-wizard.png');

  await page.click('[data-testid="sidecar-install"]');

  // 进度页面截图
  await expect(page).toHaveScreenshot('install-progress.png', {
    maxDiffPixels: 100
  });
});
```

### 6. 性能测试

#### 添加性能指标收集

```typescript
test('Sidecar installation performance', async ({ page }) => {
  const startTime = Date.now();

  await page.goto('/#/install');
  await page.click('[data-testid="sidecar-install"]');

  // 等待完成
  await page.waitForSelector('[data-testid="install-complete"]', {
    timeout: 300000
  });

  const duration = Date.now() - startTime;

  // 性能断言
  expect(duration).toBeLessThan(180000); // 应在 3 分钟内完成

  // 记录性能数据
  test.info().annotations.push({
    type: 'performance',
    description: `Installation took ${duration}ms`
  });
});
```

### 7. 错误场景覆盖

#### 添加更多错误场景

```typescript
// 磁盘空间不足
test('should handle disk space error', async ({ page }) => {
  // Mock 磁盘空间检查
  await page.route('**/check_disk_space', async (route) => {
    await route.fulfill({
      status: 200,
      body: JSON.stringify({
        success: false,
        error: '磁盘空间不足，需要至少 500MB'
      })
    });
  });

  await page.goto('/#/install');
  await page.click('[data-testid="sidecar-install"]');

  await expect(page.getByText('磁盘空间不足')).toBeVisible();
});

// 权限错误
test('should handle permission error', async ({ page }) => {
  await page.route('**/install_openclaw_sidecar', async (route) => {
    await route.fulfill({
      status: 403,
      body: JSON.stringify({
        success: false,
        error: '无写入权限'
      })
    });
  });

  await page.goto('/#/install');
  await page.click('[data-testid="sidecar-install"]');

  await expect(page.getByText('无写入权限')).toBeVisible();
});

// 网络超时
test('should handle network timeout', async ({ page }) => {
  await page.route('**/download_nodejs', async (route) => {
    await new Promise(resolve => setTimeout(resolve, 31000)); // 超时
    await route.abort('timedout');
  });

  await page.goto('/#/install');
  await page.click('[data-testid="sidecar-install"]');

  await expect(page.getByText('下载超时')).toBeVisible();
});
```

### 8. 跨平台测试

#### 扩展平台支持

```typescript
// playwright.config.ts
projects: [
  {
    name: 'chromium-mac',
    use: { ...devices['Desktop Chrome'], platform: 'darwin' },
  },
  {
    name: 'chromium-linux',
    use: { ...devices['Desktop Chrome'], platform: 'linux' },
  },
  {
    name: 'chromium-windows',
    use: { ...devices['Desktop Chrome'], platform: 'win32' },
  },
]
```

### 9. 测试报告增强

#### 自定义报告

```typescript
// playwright.config.ts
reporter: [
  ['html', { open: 'never' }],
  ['json', { outputFile: 'test-results.json' }],
  ['junit', { outputFile: 'test-results.xml' }],
  ['./custom-reporter.ts']
]
```

### 10. CI/CD 优化

#### 并行执行

```yaml
# .github/workflows/e2e-tests.yml
strategy:
  fail-fast: false
  matrix:
    shard: [1/4, 2/4, 3/4, 4/4]

steps:
  - name: Run E2E tests
    run: npx playwright test --shard=${{ matrix.shard }}
```

---

## 实施优先级

### P0 - 必须实现
1. ✅ 前端 UI 组件添加 `data-testid` 属性
2. ✅ 修复失败的 4 个 Sidecar 测试
3. ✅ 确保所有测试能够独立运行

### P1 - 高优先级
4. 添加 Mock API 支持
5. 优化测试等待逻辑
6. 添加性能基准测试

### P2 - 中优先级
7. 添加视觉回归测试
8. 扩展错误场景覆盖
9. 增强测试报告

### P3 - 低优先级
10. 跨平台测试矩阵
11. 测试并行化
12. 自定义报告器

---

## 验收标准

- [ ] 所有 Sidecar 测试通过
- [ ] 测试执行时间 < 10 分钟
- [ ] 测试稳定性 > 95%（连续 20 次执行通过）
- [ ] 代码覆盖率 > 80%
- [ ] CI/CD 集成完成
- [ ] 文档完整
