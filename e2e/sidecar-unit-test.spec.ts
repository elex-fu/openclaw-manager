import { test, expect } from '@playwright/test';
import { getSidecarStatus, clearSidecarInstallation, mockSidecarInstallation } from './utils/sidecar-helpers';

/**
 * Sidecar Helper Function Unit Tests
 *
 * These tests verify the test helper functions work correctly
 * without needing the full Tauri application.
 */

test.describe('Sidecar Helper Functions', () => {
  test.beforeEach(() => {
    // Clean up before each test
    clearSidecarInstallation();
  });

  test.afterEach(() => {
    // Clean up after each test
    clearSidecarInstallation();
  });

  test('should report not installed when Sidecar is not present', () => {
    const status = getSidecarStatus();

    expect(status.installed).toBe(false);
    expect(status.nodeModulesExists).toBe(false);
    expect(status.version).toBeUndefined();
  });

  test('should create mock Sidecar installation', () => {
    mockSidecarInstallation('2026.2.27');

    const status = getSidecarStatus();

    expect(status.installed).toBe(true);
    expect(status.nodeModulesExists).toBe(true);
    expect(status.version).toBe('2026.2.27');
  });

  test('should clear Sidecar installation', () => {
    // First create a mock installation
    mockSidecarInstallation('2026.2.27');

    // Verify it's created
    let status = getSidecarStatus();
    expect(status.installed).toBe(true);

    // Clear it
    clearSidecarInstallation();

    // Verify it's cleared
    status = getSidecarStatus();
    expect(status.installed).toBe(false);
    expect(status.nodeModulesExists).toBe(false);
  });

  test('should support custom version in mock installation', () => {
    mockSidecarInstallation('2026.3.1-beta');

    const status = getSidecarStatus();

    expect(status.version).toBe('2026.3.1-beta');
  });
});

test.describe('Sidecar UI Test Setup', () => {
  test('playwright browser should launch', async ({ page }) => {
    // Simple test to verify browser launches
    await page.goto('about:blank');
    const title = await page.title();
    expect(title).toBe('');
  });

  test('page screenshot should work', async ({ page }) => {
    await page.goto('about:blank');
    const screenshot = await page.screenshot();
    expect(screenshot).toBeTruthy();
    expect(screenshot.length).toBeGreaterThan(0);
  });
});
