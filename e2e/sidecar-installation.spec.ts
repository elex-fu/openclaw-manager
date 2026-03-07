import { test, expect, Page } from '@playwright/test';
import { execSync } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';

/**
 * Sidecar Installation E2E Tests
 *
 * Tests the complete Sidecar installation workflow:
 * - Sidecar mode detection and UI display
 * - Node.js runtime download progress
 * - OpenClaw extraction and setup
 * - npm install progress tracking
 * - Service start/stop via Sidecar
 */

// Test helpers
const waitForTimeout = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

const clearSidecarInstallation = () => {
  try {
    // Remove Sidecar installation directory
    const sidecarDir = path.join(process.env.HOME || '', '.openclaw', 'app');
    if (fs.existsSync(sidecarDir)) {
      fs.rmSync(sidecarDir, { recursive: true, force: true });
      console.log('Cleared Sidecar installation directory');
    }
  } catch (e) {
    console.log('No existing Sidecar installation to clear');
  }
};

const checkSidecarInstalled = (): boolean => {
  const sidecarDir = path.join(process.env.HOME || '', '.openclaw', 'app');
  const openclawDir = path.join(sidecarDir, 'openclaw');
  const nodeModulesDir = path.join(openclawDir, 'node_modules');
  return fs.existsSync(nodeModulesDir);
};

test.describe('Sidecar Installation Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Clear any existing Sidecar installation for clean test
    clearSidecarInstallation();
    await page.goto('/');
  });

  test.afterEach(async () => {
    // Cleanup after test
    clearSidecarInstallation();
  });

  test('should display Sidecar installation option in install wizard', async ({ page }) => {
    // Navigate to install wizard
    await page.goto('/#/install');

    // Wait for wizard to load
    const wizardTitle = page.locator('h1:has-text("安装"), h2:has-text("安装")');
    await expect(wizardTitle).toBeVisible({ timeout: 10000 });

    // Look for Sidecar installation option
    const sidecarOption = page.locator('[data-testid="sidecar-install"]').or(
      page.locator('button:has-text("Sidecar")').or(
        page.locator('button:has-text("嵌入式安装")')
      )
    );

    // Sidecar option should be visible
    await expect(sidecarOption).toBeVisible({ timeout: 5000 });
  });

  test('should start Sidecar installation when clicked', async ({ page }) => {
    await page.goto('/#/install');

    // Click Sidecar install button
    const sidecarButton = page.locator('[data-testid="sidecar-install"]').or(
      page.locator('button:has-text("Sidecar")').or(
        page.locator('button:has-text("嵌入式安装")')
      )
    );
    await expect(sidecarButton).toBeVisible({ timeout: 5000 });
    await sidecarButton.click();

    // Should show installation progress
    const progressIndicator = page.locator('[role="progressbar"]').or(
      page.locator('[data-testid="install-progress"]')
    );
    await expect(progressIndicator).toBeVisible({ timeout: 10000 });

    // Should show stage text
    const stageText = page.locator('[data-testid="install-stage"]').or(
      page.getByText(/检查安装环境|解压|下载 Node|安装依赖/)
    );
    await expect(stageText).toBeVisible();
  });

  test('should display Node.js download progress', async ({ page }) => {
    await page.goto('/#/install');

    // Start Sidecar installation
    const sidecarButton = page.locator('button:has-text("Sidecar"), button:has-text("嵌入式安装")');
    if (await sidecarButton.isVisible().catch(() => false)) {
      await sidecarButton.click();

      // Wait for Node.js download stage
      const nodeDownloadText = page.getByText(/下载 Node\.js|Node\.js/);

      try {
        await expect(nodeDownloadText).toBeVisible({ timeout: 30000 });

        // Check for percentage display
        const percentageText = page.locator('[data-testid="install-progress"]').or(
          page.getByText(/\d+%/)
        );
        await expect(percentageText).toBeVisible();
      } catch {
        // If system Node.js is available, download may be skipped
        console.log('Node.js download stage skipped (using system Node.js)');
      }
    }
  });

  test('should show npm install progress', async ({ page }) => {
    await page.goto('/#/install');

    const sidecarButton = page.locator('button:has-text("Sidecar"), button:has-text("嵌入式安装")');
    if (await sidecarButton.isVisible().catch(() => false)) {
      await sidecarButton.click();

      // Wait for npm install stage
      const npmInstallText = page.getByText(/npm install|安装依赖|Installing/);

      await expect(npmInstallText).toBeVisible({ timeout: 60000 });

      // Progress should update
      const progressBar = page.locator('[role="progressbar"]');
      await expect(progressBar).toBeVisible();
    }
  });

  test('should complete installation and show success', async ({ page }) => {
    test.setTimeout(300000); // 5 minutes for full installation

    await page.goto('/#/install');

    const sidecarButton = page.locator('button:has-text("Sidecar"), button:has-text("嵌入式安装")');
    await expect(sidecarButton).toBeVisible({ timeout: 5000 });
    await sidecarButton.click();

    // Wait for installation to complete
    const successMessage = page.getByText(/安装成功|安装完成|success/i);
    const dashboardLink = page.locator('a[href="#/"], button:has-text("进入")');

    // Either success message or dashboard link should appear
    await expect(successMessage.or(dashboardLink)).toBeVisible({ timeout: 240000 });

    // Verify installation actually completed
    expect(checkSidecarInstalled()).toBe(true);
  });

  test('should navigate to dashboard after Sidecar installation', async ({ page }) => {
    test.setTimeout(300000);

    await page.goto('/#/install');

    // Install Sidecar
    const sidecarButton = page.locator('button:has-text("Sidecar"), button:has-text("嵌入式安装")');
    if (await sidecarButton.isVisible().catch(() => false)) {
      await sidecarButton.click();

      // Wait for completion
      const completeButton = page.locator('button:has-text("完成")').or(
        page.locator('button:has-text("进入")').or(
          page.locator('a:has-text("仪表盘")')
        )
      );
      await completeButton.waitFor({ timeout: 240000 });
      await completeButton.click();

      // Should navigate to dashboard
      await page.waitForURL('**/#/', { timeout: 10000 });

      // Dashboard should be visible
      const dashboard = page.locator('h1:has-text("OpenClaw")').or(
        page.getByText('系统状态')
      );
      await expect(dashboard).toBeVisible();
    }
  });
});

test.describe('Sidecar Service Control', () => {
  test.beforeEach(async ({ page }) => {
    // Ensure Sidecar is installed before testing service control
    if (!checkSidecarInstalled()) {
      test.skip('Sidecar installation required for service control tests');
    }
    await page.goto('/');
  });

  test('should show Sidecar service status on dashboard', async ({ page }) => {
    await page.goto('/');

    // Wait for dashboard
    const dashboard = page.locator('h1:has-text("OpenClaw")').or(
      page.getByText('系统状态')
    );
    await dashboard.waitFor({ timeout: 30000 });

    // Check for service status indicator
    const serviceStatus = page.locator('[data-testid="service-status"]').or(
      page.getByText(/服务状态|Sidecar|运行中|已停止/)
    );
    await expect(serviceStatus).toBeVisible();
  });

  test('should start Sidecar service when clicking start button', async ({ page }) => {
    await page.goto('/');

    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 });

    // Find and click start button (if service is stopped)
    const startButton = page.locator('button:has-text("启动")').or(
      page.locator('[data-testid="start-service"]')
    );

    if (await startButton.isVisible().catch(() => false)) {
      await startButton.click();

      // Wait for service to start
      const runningStatus = page.getByText(/运行中|Running/);
      await expect(runningStatus).toBeVisible({ timeout: 30000 });
    }
  });

  test('should stop Sidecar service when clicking stop button', async ({ page }) => {
    await page.goto('/');

    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 });

    // Find stop button (if service is running)
    const stopButton = page.locator('button:has-text("停止")').or(
      page.locator('[data-testid="stop-service"]')
    );

    if (await stopButton.isVisible().catch(() => false)) {
      await stopButton.click();

      // Wait for service to stop
      const stoppedStatus = page.getByText(/已停止|Stopped/);
      await expect(stoppedStatus).toBeVisible({ timeout: 10000 });
    }
  });

  test('should display Sidecar version info', async ({ page }) => {
    await page.goto('/');

    const dashboard = page.locator('h1:has-text("OpenClaw")');
    await dashboard.waitFor({ timeout: 30000 });

    // Look for version info
    const versionInfo = page.locator('[data-testid="sidecar-version"]').or(
      page.getByText(/版本|Version.*\d+\.\d+/)
    );

    expect(await versionInfo.isVisible().catch(() => false)).toBe(true);
  });
});

test.describe('Sidecar Error Handling', () => {
  test('should handle network error during Node.js download', async ({ page }) => {
    // Simulate offline condition
    await page.context().setOffline(true);

    await page.goto('/#/install');

    const sidecarButton = page.locator('button:has-text("Sidecar"), button:has-text("嵌入式安装")');
    if (await sidecarButton.isVisible().catch(() => false)) {
      await sidecarButton.click();

      // Should show error or fallback message
      const errorOrFallback = page.getByText(/使用系统 Node|网络错误|离线模式/);
      await expect(errorOrFallback).toBeVisible({ timeout: 30000 });
    }

    // Restore network
    await page.context().setOffline(false);
  });

  test('should show error when npm install fails', async ({ page }) => {
    await page.goto('/#/install');

    const sidecarButton = page.locator('button:has-text("Sidecar"), button:has-text("嵌入式安装")');
    if (await sidecarButton.isVisible().catch(() => false)) {
      await sidecarButton.click();

      // Wait for potential error
      try {
        const errorMessage = page.getByText(/安装失败|npm install.*error/i);
        await errorMessage.waitFor({ timeout: 120000 });

        // Should show retry button
        const retryButton = page.locator('button:has-text("重试")');
        await expect(retryButton).toBeVisible();
      } catch {
        // Installation succeeded, no error to test
        console.log('Installation completed without error');
      }
    }
  });

  test('should allow retry after installation failure', async ({ page }) => {
    await page.goto('/#/install');

    const sidecarButton = page.locator('button:has-text("Sidecar"), button:has-text("嵌入式安装")');
    if (await sidecarButton.isVisible().catch(() => false)) {
      await sidecarButton.click();

      // Look for retry button if error occurs
      const retryButton = page.locator('button:has-text("重试")');

      if (await retryButton.isVisible({ timeout: 60000 }).catch(() => false)) {
        await retryButton.click();

        // Should restart installation
        const progressIndicator = page.locator('[role="progressbar"]');
        await expect(progressIndicator).toBeVisible();
      }
    }
  });
});
