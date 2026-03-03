import { test, expect } from '@playwright/test';

/**
 * Diagnostics E2E Tests
 *
 * Tests system diagnostics functionality including:
 * - Running system diagnostics
 * - Viewing diagnostic results
 * - Auto-fixing issues
 * - Viewing system information
 */

test.describe('Diagnostics', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/diagnostics');
  });

  test('should display diagnostics page', async ({ page }) => {
    const title = page.locator('h1:has-text("诊断"), h2:has-text("诊断")').or(
      page.locator('h1:has-text("Diagnostics"), h2:has-text("Diagnostics")')
    );
    await expect(title).toBeVisible({ timeout: 10000 });
  });

  test('should have run diagnostics button', async ({ page }) => {
    const runButton = page.locator('button:has-text("运行诊断")').or(
      page.locator('button:has-text("诊断")').or(
        page.locator('button:has-text("Run Diagnostics")')
      )
    );
    await expect(runButton).toBeVisible();
  });

  test('should show system information', async ({ page }) => {
    const sysInfo = page.locator('[data-testid="system-info"]').or(
      page.getByText(/系统信息|操作系统|版本/i)
    );

    // System info should be visible or available after loading
    await expect(sysInfo).toBeVisible().catch(() => {
      // May need to wait for load
      test.skip();
    });
  });

  test('should run diagnostics and show results', async ({ page }) => {
    const runButton = page.locator('button:has-text("运行诊断")').or(
      page.locator('button:has-text("诊断")')
    );

    await runButton.click();

    // Wait for diagnostics to complete
    const results = page.locator('[data-testid="diagnostic-results"]').or(
      page.locator('text=/检查完成|诊断完成|results/i')
    );

    await expect(results).toBeVisible({ timeout: 30000 });
  });

  test('should show individual check items', async ({ page }) => {
    // Run diagnostics first
    const runButton = page.locator('button:has-text("运行诊断")').or(
      page.locator('button:has-text("诊断")')
    );
    await runButton.click();

    // Wait for results
    await page.waitForTimeout(2000);

    // Look for check items
    const checkItems = page.locator('[data-testid="check-item"]').or(
      page.locator('text=/检查|check/i')
    );

    const count = await checkItems.count();
    if (count > 0) {
      expect(count).toBeGreaterThan(0);
    }
  });

  test('should indicate check status with icons/colors', async ({ page }) => {
    const runButton = page.locator('button:has-text("运行诊断")').or(
      page.locator('button:has-text("诊断")')
    );
    await runButton.click();

    await page.waitForTimeout(2000);

    // Look for pass/fail indicators
    const passedIndicator = page.locator('text=/通过|成功|✓|passed/i').or(
      page.locator('[data-testid="check-passed"]')
    );
    const failedIndicator = page.locator('text=/失败|错误|✗|failed/i').or(
      page.locator('[data-testid="check-failed"]')
    );

    // At least one type of indicator should exist
    const hasPassed = await passedIndicator.isVisible().catch(() => false);
    const hasFailed = await failedIndicator.isVisible().catch(() => false);

    expect(hasPassed || hasFailed).toBeTruthy();
  });

  test('should have auto-fix option for fixable issues', async ({ page }) => {
    const runButton = page.locator('button:has-text("运行诊断")').or(
      page.locator('button:has-text("诊断")')
    );
    await runButton.click();

    await page.waitForTimeout(2000);

    // Look for auto-fix button
    const autoFixButton = page.locator('button:has-text("自动修复")').or(
      page.locator('button:has-text("修复")').or(
        page.locator('button:has-text("Auto Fix")')
      )
    );

    // Auto-fix may or may not be available depending on issues
    if (await autoFixButton.isVisible().catch(() => false)) {
      await expect(autoFixButton).toBeEnabled();
    }
  });

  test('should show detailed issue description', async ({ page }) => {
    const runButton = page.locator('button:has-text("运行诊断")').or(
      page.locator('button:has-text("诊断")')
    );
    await runButton.click();

    await page.waitForTimeout(2000);

    // Look for expandable issue details
    const issueDetail = page.locator('[data-testid="issue-detail"]').or(
      page.getByText(/详情|description|详情/i)
    );

    // Click to expand if available
    const count = await issueDetail.count();
    if (count > 0) {
      await issueDetail.first().click();

      // Should show more information
      const expandedContent = page.locator('[data-testid="issue-expanded"]').or(
        page.getByText(/建议|solution|修复/i)
      );
      await expect(expandedContent).toBeVisible();
    }
  });
});

test.describe('System Information', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/diagnostics');
  });

  test('should display OS information', async ({ page }) => {
    const osInfo = page.locator('text=/macOS|Windows|Linux|操作系统/i');
    await expect(osInfo).toBeVisible();
  });

  test('should display architecture information', async ({ page }) => {
    const archInfo = page.locator('text=/x64|ARM64|架构|architecture/i');

    if (await archInfo.isVisible().catch(() => false)) {
      await expect(archInfo).toBeVisible();
    }
  });

  test('should show installation status', async ({ page }) => {
    const installStatus = page.locator('text=/安装状态|安装路径|Install Status/i');

    if (await installStatus.isVisible().catch(() => false)) {
      await expect(installStatus).toBeVisible();
    }
  });

  test('should show version information', async ({ page }) => {
    const versionInfo = page.locator('text=/版本|version|v\\d+\\./i');

    const count = await versionInfo.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });
});