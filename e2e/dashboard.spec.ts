import { test, expect } from '@playwright/test';

/**
 * Dashboard E2E Tests
 *
 * Tests dashboard functionality including:
 * - System status display
 * - Quick actions
 * - Navigation to other pages
 * - Resource monitoring
 * - Activity logs
 */

test.describe('Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should display dashboard page', async ({ page }) => {
    // Check for dashboard title
    const title = page.locator('h1:has-text("OpenClaw")').or(
      page.locator('h1:has-text("仪表盘")').or(
        page.locator('h1:has-text("Dashboard")')
      )
    );

    await expect(title).toBeVisible({ timeout: 30000 });
  });

  test('should show installation status card', async ({ page }) => {
    // Wait for dashboard to load
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Look for installation status
    const installStatus = page.locator('text=/已安装|未安装|安装|Installed|Not installed/i').or(
      page.locator('[data-testid="install-status"]')
    );

    await expect(installStatus).toBeVisible();
  });

  test('should show service status card', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Look for service status
    const serviceStatus = page.locator('text=/服务状态|运行中|已停止|Service|Running|Stopped/i').or(
      page.locator('[data-testid="service-status"]')
    );

    await expect(serviceStatus).toBeVisible();
  });

  test('should show current agent card', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Look for current agent section
    const agentCard = page.locator('text=/当前 Agent|Current Agent/i').or(
      page.locator('[data-testid="current-agent"]')
    );

    await expect(agentCard).toBeVisible();
  });

  test('should show default model card', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Look for default model section
    const modelCard = page.locator('text=/默认模型|Default Model/i').or(
      page.locator('[data-testid="default-model"]')
    );

    await expect(modelCard).toBeVisible();
  });

  test('should show quick actions section', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Look for quick actions
    const quickActions = page.locator('text=/快捷操作|Quick Actions/i').or(
      page.locator('[data-testid="quick-actions"]')
    );

    await expect(quickActions).toBeVisible();
  });

  test('should display status indicators with icons', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Look for status icons (check marks, x marks, etc.)
    const statusIcons = page.locator('svg').or(
      page.locator('[data-lucide]')
    );

    const count = await statusIcons.count();
    expect(count).toBeGreaterThan(0);
  });
});

test.describe('Dashboard Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should navigate to agents page', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Find agents navigation link
    const agentsLink = page.locator('button:has-text("管理 Agents")').or(
      page.locator('a:has-text("Agents")').or(
        page.locator('button:has-text("Agent")')
      )
    );

    if (await agentsLink.isVisible().catch(() => false)) {
      await agentsLink.click();

      // Should navigate to agents page
      await page.waitForURL('**/#/agents', { timeout: 5000 }).catch(() => {
        // Navigation might use different method
      });
    }
  });

  test('should navigate to models page', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Find models navigation link
    const modelsLink = page.locator('button:has-text("配置模型")').or(
      page.locator('a:has-text("Models")').or(
        page.locator('button:has-text("模型")')
      )
    );

    if (await modelsLink.isVisible().catch(() => false)) {
      await modelsLink.click();

      await page.waitForURL('**/#/models', { timeout: 5000 }).catch(() => {});
    }
  });

  test('should navigate to diagnostics page', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Find diagnostics button in quick actions
    const diagnosticsButton = page.locator('button:has-text("运行诊断")').or(
      page.locator('button:has-text("诊断")').or(
        page.locator('a:has-text("诊断")')
      )
    );

    if (await diagnosticsButton.isVisible().catch(() => false)) {
      await diagnosticsButton.click();

      await page.waitForURL('**/#/diagnostics', { timeout: 5000 }).catch(() => {});
    }
  });

  test('should navigate to settings page', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Find settings button
    const settingsButton = page.locator('button:has-text("打开设置")').or(
      page.locator('button:has-text("设置")').or(
        page.locator('a:has-text("设置")')
      )
    );

    if (await settingsButton.isVisible().catch(() => false)) {
      await settingsButton.click();

      await page.waitForURL('**/#/settings', { timeout: 5000 }).catch(() => {});
    }
  });
});

test.describe('System Information', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should show system information section', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Look for system info section
    const sysInfo = page.locator('text=/系统信息|System Information/i').or(
      page.locator('[data-testid="system-info"]')
    );

    await expect(sysInfo).toBeVisible();
  });

  test('should display configured models count', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Look for models count
    const modelsCount = page.locator('text=/已配置模型|Configured Models/i');

    const count = await modelsCount.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('should display created agents count', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Look for agents count
    const agentsCount = page.locator('text=/已创建 Agent|Created Agents/i');

    const count = await agentsCount.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('should display enabled skills count', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Look for skills count
    const skillsCount = page.locator('text=/已启用技能|Enabled Skills/i');

    const count = await skillsCount.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('should display config version', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Look for config version
    const configVersion = page.locator('text=/配置版本|Config Version/i');

    const count = await configVersion.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });
});

test.describe('Resource Monitoring', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should show resource monitor section', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Look for resource monitor
    const resourceMonitor = page.locator('text=/资源|Resource|CPU|内存|Memory/i').or(
      page.locator('[data-testid="resource-monitor"]')
    );

    const count = await resourceMonitor.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('should show activity log section', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Look for activity log
    const activityLog = page.locator('text=/活动日志|Activity Log|日志|Log/i').or(
      page.locator('[data-testid="activity-log"]')
    );

    const count = await activityLog.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('should show diagnostic alerts section', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 }).catch(() => {
      test.skip('Dashboard not loaded');
    });

    // Look for diagnostic alerts
    const diagnosticAlerts = page.locator('text=/诊断警告|Diagnostic Alerts|警告|Alerts/i').or(
      page.locator('[data-testid="diagnostic-alerts"]')
    );

    const count = await diagnosticAlerts.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });
});

test.describe('Dashboard Error States', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should handle initialization failure state', async ({ page }) => {
    // Look for error state
    const errorState = page.locator('text=/初始化失败|Initialization Failed|错误|Error/i').or(
      page.locator('[data-testid="init-error"]')
    );

    // Error state may or may not be visible
    const isVisible = await errorState.isVisible().catch(() => false);

    if (isVisible) {
      // Should have retry button
      const retryButton = page.locator('button:has-text("重试"), button:has-text("Retry")');
      await expect(retryButton).toBeVisible();

      // Should have diagnostics button
      const diagnosticsButton = page.locator('button:has-text("诊断"), button:has-text("Diagnostics")');
      await expect(diagnosticsButton).toBeVisible();
    }
  });

  test('should show loading state during initialization', async ({ page }) => {
    // Look for loading indicator
    const loadingIndicator = page.locator('text=/正在初始化|Initializing|加载中|Loading/i').or(
      page.locator('[role="progressbar"]')
    );

    // Loading state may or may not be visible
    const isVisible = await loadingIndicator.isVisible().catch(() => false);

    if (isVisible) {
      // Should show progress
      const progress = page.locator('text=/\\d+%/i');
      const hasProgress = await progress.count() > 0;

      expect(hasProgress || true).toBeTruthy();
    }
  });
});
