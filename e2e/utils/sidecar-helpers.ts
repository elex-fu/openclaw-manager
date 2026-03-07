/**
 * Sidecar E2E Test Helpers
 *
 * Utility functions for testing Sidecar functionality
 */

import { Page, expect } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';
import { execSync } from 'child_process';

export interface SidecarStatus {
  installed: boolean;
  nodeModulesExists: boolean;
  version?: string;
}

export interface PollOptions {
  timeout?: number;
  interval?: number;
  message?: string;
}

/**
 * Get Sidecar installation status
 */
export function getSidecarStatus(): SidecarStatus {
  const homeDir = process.env.HOME || process.env.USERPROFILE || '';
  const sidecarDir = path.join(homeDir, '.openclaw', 'app');
  const openclawDir = path.join(sidecarDir, 'openclaw');
  const nodeModulesDir = path.join(openclawDir, 'node_modules');
  const versionFile = path.join(openclawDir, 'VERSION');

  const installed = fs.existsSync(sidecarDir);
  const nodeModulesExists = fs.existsSync(nodeModulesDir);
  let version: string | undefined;

  if (fs.existsSync(versionFile)) {
    version = fs.readFileSync(versionFile, 'utf-8').trim();
  }

  return {
    installed,
    nodeModulesExists,
    version,
  };
}

/**
 * Clear Sidecar installation from filesystem
 */
export function clearSidecarInstallationFS(): void {
  const homeDir = process.env.HOME || process.env.USERPROFILE || '';
  const sidecarDir = path.join(homeDir, '.openclaw', 'app');

  if (fs.existsSync(sidecarDir)) {
    fs.rmSync(sidecarDir, { recursive: true, force: true });
    console.log(`[Test Helper] Cleared Sidecar directory: ${sidecarDir}`);
  }
}

/**
 * Clear Sidecar installation state from browser
 */
export async function clearSidecarInstallation(page: Page): Promise<void> {
  try {
    // Wait for page to be ready before accessing localStorage
    await page.waitForLoadState('domcontentloaded', { timeout: 5000 }).catch(() => {
      // Page might not be loading, that's ok
    });

    // Clear localStorage and sessionStorage with error handling
    await page.evaluate(() => {
      try {
        localStorage.removeItem('sidecarStatus');
        localStorage.removeItem('installProgress');
        sessionStorage.clear();
      } catch (e) {
        // localStorage access might be restricted, ignore
        console.log('localStorage access failed:', e);
      }
    });
  } catch (e) {
    // Ignore errors from page.evaluate
    console.log('Failed to clear browser state:', e);
  }

  // Clear filesystem installation
  clearSidecarInstallationFS();

  console.log('[Test Helper] Cleared Sidecar installation state');
}

/**
 * Mock Sidecar installation for testing
 */
export function mockSidecarInstallation(version: string = '2026.2.27'): void {
  const homeDir = process.env.HOME || process.env.USERPROFILE || '';
  const sidecarDir = path.join(homeDir, '.openclaw', 'app');
  const openclawDir = path.join(sidecarDir, 'openclaw');
  const nodeModulesDir = path.join(openclawDir, 'node_modules');
  const versionFile = path.join(openclawDir, 'VERSION');

  // Create directory structure
  fs.mkdirSync(nodeModulesDir, { recursive: true });
  fs.writeFileSync(versionFile, version);

  // Create a minimal package.json
  const packageJson = {
    name: 'openclaw',
    version,
    main: 'dist/index.js',
  };
  fs.writeFileSync(
    path.join(openclawDir, 'package.json'),
    JSON.stringify(packageJson, null, 2)
  );

  // Create minimal dist
  const distDir = path.join(openclawDir, 'dist');
  fs.mkdirSync(distDir, { recursive: true });
  fs.writeFileSync(
    path.join(distDir, 'index.js'),
    'console.log("Mock OpenClaw");'
  );

  console.log(`[Test Helper] Created mock Sidecar installation v${version}`);
}

/**
 * Smart polling for a condition
 */
export async function pollForCondition(
  condition: () => boolean | Promise<boolean>,
  options: PollOptions = {}
): Promise<void> {
  const { timeout = 30000, interval = 500, message = 'Condition not met' } = options;
  const startTime = Date.now();

  while (Date.now() - startTime < timeout) {
    if (await condition()) return;
    await new Promise(resolve => setTimeout(resolve, interval));
  }

  throw new Error(`${message} within ${timeout}ms`);
}

/**
 * Wait for Sidecar installation to complete
 */
export async function waitForSidecarInstallation(
  page: Page,
  options: { timeout?: number; checkInterval?: number } = {}
): Promise<void> {
  const { timeout = 300000, checkInterval = 5000 } = options;
  const startTime = Date.now();

  while (Date.now() - startTime < timeout) {
    const status = getSidecarStatus();
    if (status.nodeModulesExists) {
      console.log('[Test Helper] Sidecar installation detected');
      return;
    }

    // Check for completion indicator in UI
    const completeIndicator = page.getByText(/安装成功|安装完成|success/i);
    if (await completeIndicator.isVisible().catch(() => false)) {
      console.log('[Test Helper] Installation completion detected in UI');
      return;
    }

    await page.waitForTimeout(checkInterval);
  }

  throw new Error(`Sidecar installation did not complete within ${timeout}ms`);
}

/**
 * Wait for progress to complete (reach 100%)
 */
export async function waitForProgressComplete(
  page: Page,
  options: { timeout?: number; checkInterval?: number } = {}
): Promise<void> {
  const { timeout = 120000, checkInterval = 1000 } = options;
  const startTime = Date.now();

  while (Date.now() - startTime < timeout) {
    const progressText = await page.locator('[data-testid="install-progress"]').textContent()
      .catch(() => '');

    const match = progressText.match(/(\d+)%/);
    if (match) {
      const percentage = parseInt(match[1], 10);
      if (percentage >= 100) {
        console.log(`[Test Helper] Progress completed: ${percentage}%`);
        return;
      }
    }

    // Also check for completion message
    const completeIndicator = page.getByText(/安装成功|安装完成|success/i);
    if (await completeIndicator.isVisible().catch(() => false)) {
      console.log('[Test Helper] Installation completion detected');
      return;
    }

    await page.waitForTimeout(checkInterval);
  }

  throw new Error(`Progress did not complete within ${timeout}ms`);
}

/**
 * Start Sidecar service via UI
 */
export async function startSidecarService(page: Page): Promise<void> {
  const startButton = page.locator('button:has-text("启动")').or(
    page.locator('[data-testid="start-service"]')
  );

  if (await startButton.isVisible().catch(() => false)) {
    await startButton.click();

    // Wait for running status
    const runningStatus = page.getByText(/运行中|Running/);
    await expect(runningStatus).toBeVisible({ timeout: 30000 });

    console.log('[Test Helper] Sidecar service started');
  } else {
    console.log('[Test Helper] Service already running or button not found');
  }
}

/**
 * Stop Sidecar service via UI
 */
export async function stopSidecarService(page: Page): Promise<void> {
  const stopButton = page.locator('button:has-text("停止")').or(
    page.locator('[data-testid="stop-service"]')
  );

  if (await stopButton.isVisible().catch(() => false)) {
    await stopButton.click();

    // Wait for stopped status
    const stoppedStatus = page.getByText(/已停止|Stopped/);
    await expect(stoppedStatus).toBeVisible({ timeout: 10000 });

    console.log('[Test Helper] Sidecar service stopped');
  } else {
    console.log('[Test Helper] Service already stopped or button not found');
  }
}

/**
 * Wait for progress to reach percentage
 */
export async function waitForProgress(
  page: Page,
  targetPercentage: number,
  timeout: number = 60000
): Promise<void> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeout) {
    const progressText = await page.locator('[data-testid="install-progress"]').textContent()
      .catch(() => '');

    const match = progressText.match(/(\d+)%/);
    if (match) {
      const percentage = parseInt(match[1], 10);
      if (percentage >= targetPercentage) {
        console.log(`[Test Helper] Progress reached ${percentage}%`);
        return;
      }
    }

    await page.waitForTimeout(1000);
  }

  throw new Error(`Progress did not reach ${targetPercentage}% within ${timeout}ms`);
}

/**
 * Verify Sidecar installation via UI
 */
export async function verifySidecarInstallationUI(page: Page): Promise<void> {
  // Check for installed indicators
  const installedIndicators = [
    page.getByText(/已安装|Installed/),
    page.locator('[data-testid="sidecar-installed"]'),
    page.getByText('Sidecar').filter({ hasText: /版本|Version/ }),
  ];

  let found = false;
  for (const indicator of installedIndicators) {
    if (await indicator.isVisible().catch(() => false)) {
      found = true;
      break;
    }
  }

  if (!found) {
    throw new Error('Sidecar installation not verified in UI');
  }

  // Also verify filesystem
  const status = getSidecarStatus();
  if (!status.nodeModulesExists) {
    throw new Error('Sidecar node_modules not found on filesystem');
  }

  console.log('[Test Helper] Sidecar installation verified');
}

/**
 * Setup test environment for Sidecar tests
 */
export async function setupSidecarTest(page: Page): Promise<void> {
  // Clear browser state
  await page.context().clearCookies();
  await page.evaluate(() => {
    localStorage.clear();
    sessionStorage.clear();
  });

  // Clear filesystem state
  clearSidecarInstallationFS();

  console.log('[Test Helper] Sidecar test environment setup complete');
}

/**
 * Teardown test environment after Sidecar tests
 */
export async function teardownSidecarTest(page: Page): Promise<void> {
  // Clear browser state
  await page.evaluate(() => {
    localStorage.clear();
    sessionStorage.clear();
  });

  // Clear filesystem state
  clearSidecarInstallationFS();

  console.log('[Test Helper] Sidecar test environment teardown complete');
}
