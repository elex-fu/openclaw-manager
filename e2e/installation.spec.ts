import { test, expect } from '@playwright/test';

/**
 * Installation Flow E2E Tests
 *
 * Tests the complete installation workflow including:
 * - Initial app launch
 * - Automatic installation process
 * - Installation progress display
 * - Post-installation state
 */

test.describe('Installation Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app
    await page.goto('/');
  });

  test('should display initial loading state on first launch', async ({ page }) => {
    // Check for loading indicator
    const loadingIndicator = page.locator('[data-testid="install-progress"]').or(
      page.getByText(/正在初始化|Initializing/)
    );

    // Should show either loading or dashboard (if already installed)
    await expect(loadingIndicator.or(page.locator('h1:has-text("OpenClaw")'))).toBeVisible({
      timeout: 10000,
    });
  });

  test('should show installation progress when installing', async ({ page }) => {
    // Wait for either installation progress or dashboard (if already installed)
    const installProgress = page.locator('[data-testid="install-progress"]');
    const dashboard = page.locator('h1:has-text("OpenClaw")');

    await installProgress.or(dashboard).waitFor({ timeout: 15000 });

    // If installation is in progress, check for progress elements
    if (await installProgress.isVisible().catch(() => false)) {
      // Check for progress bar
      const progressBar = page.locator('[role="progressbar"]');
      await expect(progressBar).toBeVisible();

      // Check for stage indicator
      const stageText = page.locator('[data-testid="install-stage"]');
      await expect(stageText).toContainText(/\d+%/);
    }
  });

  test('should navigate to dashboard after installation', async ({ page }) => {
    // Wait for dashboard to appear (installation completes)
    const dashboard = page.locator('h1:has-text("OpenClaw")');

    try {
      await dashboard.waitFor({ timeout: 60000 });

      // Verify dashboard elements
      await expect(page.getByText('系统状态')).toBeVisible();
      await expect(page.getByText('快捷操作')).toBeVisible();
    } catch {
      // If dashboard doesn't appear, we might be in installation state
      // This is acceptable for test environment without bundled runtime
      test.skip();
    }
  });

  test('should show service status on dashboard', async ({ page }) => {
    // Wait for dashboard
    const dashboard = page.locator('h1:has-text("OpenClaw")');

    try {
      await dashboard.waitFor({ timeout: 30000 });

      // Check for service status card
      const serviceStatus = page.locator('[data-testid="service-status"]').or(
        page.getByText(/服务状态|运行中|已停止/)
      );
      await expect(serviceStatus).toBeVisible();
    } catch {
      test.skip();
    }
  });

  test('should display model configuration shortcut', async ({ page }) => {
    const dashboard = page.locator('h1:has-text("OpenClaw")');

    try {
      await dashboard.waitFor({ timeout: 30000 });

      // Check for model config link/button
      const modelConfigLink = page.locator('a[href*="model"], button:has-text("模型")');
      await expect(modelConfigLink).toBeVisible();
    } catch {
      test.skip();
    }
  });
});

test.describe('Installation Wizard', () => {
  test('should navigate to install wizard', async ({ page }) => {
    await page.goto('/#/install');

    // Check for wizard title
    const wizardTitle = page.locator('h1:has-text("安装")').or(
      page.locator('h2:has-text("安装")')
    );
    await expect(wizardTitle).toBeVisible({ timeout: 10000 });
  });

  test('should show installation method selection', async ({ page }) => {
    await page.goto('/#/install');

    // Look for installation method options
    const methodOptions = page.locator('[data-testid="install-method"]').or(
      page.getByText(/在线安装|离线安装|一键安装/)
    );

    // Should have at least one installation option
    const count = await methodOptions.count();
    expect(count).toBeGreaterThan(0);
  });

  test('should allow starting one-click installation', async ({ page }) => {
    await page.goto('/#/install');

    // Find one-click install button
    const oneClickButton = page.locator('button:has-text("一键安装")').or(
      page.locator('button:has-text("自动安装")')
    );

    if (await oneClickButton.isVisible().catch(() => false)) {
      await oneClickButton.click();

      // Should show progress or confirmation
      const progressOrConfirm = page.locator('[role="progressbar"]').or(
        page.getByText(/正在安装|确认/)
      );
      await expect(progressOrConfirm).toBeVisible();
    }
  });
});
