import { test, expect } from '@playwright/test';
import {
  mockSidecarNotInstalled,
  mockSidecarInstalled,
  mockSidecarInstallSuccess,
  mockSidecarNetworkError,
  mockSidecarServiceControl,
} from './mocks/sidecar-mocks';
import {
  waitForSidecarInstallation,
  waitForProgressComplete,
  clearSidecarInstallation,
  setupSidecarTest,
  teardownSidecarTest,
} from './utils/sidecar-helpers';

/**
 * Sidecar Installation E2E Tests
 *
 * Tests the complete Sidecar installation workflow with improved stability:
 * - Smart waiting logic instead of fixed timeouts
 * - Proper test isolation
 * - Mock-based testing for reliability
 */

// Configure serial execution (Sidecar tests need to run sequentially)
test.describe.configure({ mode: 'serial' });

test.describe('Sidecar Installation', () => {
  test.beforeEach(async ({ page }) => {
    // 1. Clean up browser state
    await page.context().clearCookies();
    await page.evaluate(() => {
      localStorage.clear();
      sessionStorage.clear();
    });

    // 2. Clear filesystem state
    await clearSidecarInstallation(page);

    // 3. Set default mock (not installed)
    await mockSidecarNotInstalled(page);

    // 4. Navigate to install page
    await page.goto('/#/install');
  });

  test.afterEach(async ({ page }) => {
    // Clean up Sidecar state
    await clearSidecarInstallation(page);
  });

  test('should display Sidecar installation option', async ({ page }) => {
    // Wait for wizard to load
    const wizardTitle = page.locator('h1:has-text("安装"), h2:has-text("安装")');
    await expect(wizardTitle).toBeVisible({ timeout: 10000 });

    // Check for Sidecar installation option
    await expect(page.getByTestId('sidecar-install')).toBeVisible({ timeout: 5000 });
    await expect(page.getByTestId('sidecar-install')).toHaveText(/安装 Sidecar|Sidecar|嵌入式安装/);
  });

  test('should start Sidecar installation when clicked', async ({ page }) => {
    // Setup success mock
    await mockSidecarInstallSuccess(page);

    // Click install button
    await page.getByTestId('sidecar-install').click();

    // Use smart wait instead of fixed timeout
    await expect(page.getByTestId('install-progress')).toBeVisible({ timeout: 5000 });
    await expect(page.getByTestId('install-stage')).toContainText(/检查|Checking|下载|Downloading/);
  });

  test('should display Node.js download progress', async ({ page }) => {
    await mockSidecarInstallSuccess(page);
    await page.getByTestId('sidecar-install').click();

    // Wait for progress bar to appear
    await expect(page.getByTestId('install-progress')).toBeVisible({ timeout: 5000 });

    // Wait for progress to reach 100%
    await waitForProgressComplete(page, { timeout: 120000 });
  });

  test('should complete installation and show success', async ({ page }) => {
    await mockSidecarInstallSuccess(page);
    await page.getByTestId('sidecar-install').click();

    // Use smart wait for installation completion
    await waitForSidecarInstallation(page, { timeout: 300000 });

    // Verify success state
    await expect(page.getByTestId('install-complete')).toBeVisible({ timeout: 5000 });
    await expect(page.getByTestId('install-complete')).toContainText(/安装成功|完成|Success/);
  });

  test('should handle network error during Node.js download', async ({ page }) => {
    // Setup network error mock
    await mockSidecarNetworkError(page);
    await page.getByTestId('sidecar-install').click();

    // Wait for error display with extended timeout
    await expect(page.getByTestId('install-error')).toBeVisible({ timeout: 30000 });
    await expect(page.getByTestId('install-error')).toContainText(/网络|Network|错误|Error/);

    // Verify retry button is available
    await expect(page.getByTestId('install-retry')).toBeVisible();
  });

  test('should allow retry after installation failure', async ({ page }) => {
    // First attempt - network error
    await mockSidecarNetworkError(page);
    await page.getByTestId('sidecar-install').click();
    await expect(page.getByTestId('install-error')).toBeVisible({ timeout: 30000 });

    // Second attempt - success
    await mockSidecarInstallSuccess(page);
    await page.getByTestId('install-retry').click();

    // Wait for successful completion
    await waitForSidecarInstallation(page, { timeout: 300000 });
    await expect(page.getByTestId('install-complete')).toBeVisible();
  });
});

test.describe('Sidecar Service Control', () => {
  test.beforeEach(async ({ page }) => {
    // Pre-set installed state with specific version
    await mockSidecarInstalled(page, '2026.2.27');
    await mockSidecarServiceControl(page);

    // Navigate to dashboard
    await page.goto('/#/');

    // Verify installed state is shown
    await expect(page.getByTestId('sidecar-installed')).toContainText(/已安装|Installed/, { timeout: 10000 });
  });

  test.afterEach(async ({ page }) => {
    await clearSidecarInstallation(page);
  });

  test('should start Sidecar service', async ({ page }) => {
    await page.getByTestId('start-service').click();

    // Wait for running status with extended timeout
    await expect(page.getByTestId('service-status')).toContainText(/运行中|Running/, { timeout: 15000 });
  });

  test('should stop Sidecar service', async ({ page }) => {
    // First start the service
    await page.getByTestId('start-service').click();
    await expect(page.getByTestId('service-status')).toContainText(/运行中|Running/, { timeout: 15000 });

    // Then stop it
    await page.getByTestId('stop-service').click();

    // Wait for stopped status
    await expect(page.getByTestId('service-status')).toContainText(/已停止|Stopped/, { timeout: 15000 });
  });

  test('should display Sidecar version info', async ({ page }) => {
    // Look for version info on dashboard
    const versionInfo = page.getByTestId('sidecar-version').or(
      page.getByText(/版本|Version.*\d+\.\d+/)
    );

    await expect(versionInfo).toBeVisible({ timeout: 10000 });
  });
});

test.describe('Sidecar Error Handling', () => {
  test.beforeEach(async ({ page }) => {
    await setupSidecarTest(page);
    await page.goto('/#/install');
  });

  test.afterEach(async ({ page }) => {
    await teardownSidecarTest(page);
  });

  test('should handle offline mode gracefully', async ({ page }) => {
    // Simulate offline condition
    await page.context().setOffline(true);

    try {
      await mockSidecarNetworkError(page);
      await page.getByTestId('sidecar-install').click();

      // Should show error or fallback message
      const errorOrFallback = page.getByText(/使用系统 Node|网络错误|离线模式|System Node|Offline/);
      await expect(errorOrFallback).toBeVisible({ timeout: 30000 });
    } finally {
      // Always restore network
      await page.context().setOffline(false);
    }
  });

  test('should show error when npm install fails', async ({ page }) => {
    // Setup mock that simulates npm install failure
    await page.evaluate(() => {
      (window as any).__TAURI__ = {
        ...(window as any).__TAURI__,
        invoke: async (cmd: string, args?: any) => {
          if (cmd === 'install_sidecar') {
            setTimeout(() => {
              (window as any).__TAURI__?.event?.emit?.('install-progress', {
                stage: 'Error',
                percentage: 70,
                message: 'npm install 失败',
                error: true,
              });
            }, 2000);

            return { success: true, data: { started: true } };
          }
          return (window as any).__TAURI__?.__originalInvoke?.(cmd, args);
        },
      };
    });

    await page.getByTestId('sidecar-install').click();

    // Wait for potential error
    const errorMessage = page.getByText(/安装失败|npm install.*error|Error/i);
    await expect(errorMessage).toBeVisible({ timeout: 60000 });

    // Should show retry button
    const retryButton = page.locator('button:has-text("重试"), button:has-text("Retry")');
    await expect(retryButton).toBeVisible();
  });
});
