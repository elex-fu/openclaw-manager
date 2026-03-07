/**
 * Sidecar Test Setup
 *
 * Global test setup and teardown for Sidecar E2E tests
 */

import { test as base, expect, Page } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

// Extend test with custom fixtures
export const test = base.extend<{
  sidecarPage: Page;
}>({
  sidecarPage: async ({ page }, use) => {
    // Setup before each test
    await setupSidecarEnvironment(page);
    await use(page);
    // Teardown after each test
    await teardownSidecarEnvironment(page);
  },
});

export { expect };

/**
 * Global setup - runs once before all tests
 */
export async function globalSetup(): Promise<void> {
  console.log('🧹 Starting global test setup...');

  // Ensure clean test environment
  const homeDir = process.env.HOME || process.env.USERPROFILE || '';
  const testDir = path.join(homeDir, '.openclaw', 'test');

  // Create test directory if it doesn't exist
  if (!fs.existsSync(testDir)) {
    fs.mkdirSync(testDir, { recursive: true });
  }

  // Clean up any leftover test artifacts
  cleanupTestArtifacts();

  console.log('✅ Global setup complete');
}

/**
 * Global teardown - runs once after all tests
 */
export async function globalTeardown(): Promise<void> {
  console.log('🧹 Starting global teardown...');

  // Clean up test artifacts
  cleanupTestArtifacts();

  console.log('✅ Global teardown complete');
}

/**
 * Setup for each test
 */
export async function setupSidecarEnvironment(page: Page): Promise<void> {
  console.log('📝 Setting up test environment...');

  // Clear browser state
  await page.context().clearCookies();
  await page.evaluate(() => {
    localStorage.clear();
    sessionStorage.clear();
  });

  // Clear Sidecar installation state
  clearSidecarInstallation();

  console.log('✅ Test environment setup complete');
}

/**
 * Teardown for each test
 */
export async function teardownSidecarEnvironment(page: Page): Promise<void> {
  console.log('🧹 Tearing down test environment...');

  // Clear browser state
  await page.evaluate(() => {
    localStorage.clear();
    sessionStorage.clear();
  });

  // Clear Sidecar installation state
  clearSidecarInstallation();

  console.log('✅ Test environment teardown complete');
}

/**
 * Clear Sidecar installation
 */
function clearSidecarInstallation(): void {
  try {
    const homeDir = process.env.HOME || process.env.USERPROFILE || '';
    const sidecarDir = path.join(homeDir, '.openclaw', 'app');

    if (fs.existsSync(sidecarDir)) {
      fs.rmSync(sidecarDir, { recursive: true, force: true });
      console.log(`[Setup] Cleared Sidecar directory: ${sidecarDir}`);
    }
  } catch (e) {
    console.log('[Setup] No existing Sidecar installation to clear');
  }
}

/**
 * Clean up test artifacts
 */
function cleanupTestArtifacts(): void {
  const homeDir = process.env.HOME || process.env.USERPROFILE || '';
  const artifactsToClean = [
    path.join(homeDir, '.openclaw', 'test'),
    path.join(homeDir, '.openclaw', 'temp'),
  ];

  for (const artifact of artifactsToClean) {
    try {
      if (fs.existsSync(artifact)) {
        fs.rmSync(artifact, { recursive: true, force: true });
        console.log(`[Setup] Cleaned up: ${artifact}`);
      }
    } catch (e) {
      console.log(`[Setup] Failed to clean up: ${artifact}`);
    }
  }
}
