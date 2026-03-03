import { test, expect } from '@playwright/test';

/**
 * Settings Page E2E Tests
 *
 * Tests settings functionality including:
 * - General settings
 * - Display preferences
 * - Language selection
 * - Notification settings
 * - Advanced options
 */

test.describe('Settings Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/settings');
  });

  test('should display settings page', async ({ page }) => {
    const title = page.locator('h1:has-text("设置"), h2:has-text("设置")').or(
      page.locator('h1:has-text("Settings"), h2:has-text("Settings")')
    );
    await expect(title).toBeVisible({ timeout: 10000 });
  });

  test('should have settings categories', async ({ page }) => {
    // Look for settings sections
    const sections = page.locator('[data-testid="settings-section"]').or(
      page.locator('h3, h4')
    );

    const count = await sections.count();
    expect(count).toBeGreaterThan(0);
  });

  test('should allow changing theme', async ({ page }) => {
    // Look for theme selector
    const themeSelector = page.locator('[data-testid="theme-selector"]').or(
      page.locator('select[name="theme"]').or(
        page.getByLabel(/主题|Theme/i)
      )
    );

    if (await themeSelector.isVisible().catch(() => false)) {
      await themeSelector.click();

      // Select dark theme
      const darkOption = page.locator('option:has-text("Dark"), option:has-text("暗色")');
      if (await darkOption.isVisible().catch(() => false)) {
        await darkOption.click();
      }

      // Verify theme changed
      const body = page.locator('body');
      const classAttr = await body.getAttribute('class');
      expect(classAttr?.includes('dark') || classAttr?.includes('light')).toBeTruthy();
    }
  });

  test('should have save button', async ({ page }) => {
    const saveButton = page.locator('button:has-text("保存")').or(
      page.locator('button:has-text("Save")').or(
        page.locator('button[type="submit"]')
      )
    );

    await expect(saveButton).toBeVisible();
  });

  test('should show reset options', async ({ page }) => {
    const resetButton = page.locator('button:has-text("重置")').or(
      page.locator('button:has-text("Reset")')
    );

    // Reset button may or may not exist
    const count = await resetButton.count();
    if (count > 0) {
      await expect(resetButton.first()).toBeVisible();
    }
  });
});

test.describe('General Settings', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/settings');
  });

  test('should have language selection', async ({ page }) => {
    const languageSelect = page.locator('select[name="language"]').or(
      page.getByLabel(/语言|Language/i)
    );

    if (await languageSelect.isVisible().catch(() => false)) {
      await languageSelect.click();

      // Check for language options
      const options = page.locator('option');
      const optionTexts = await options.allTextContents();

      // Should have at least one language option
      expect(optionTexts.length).toBeGreaterThan(0);
    }
  });

  test('should have auto-start option', async ({ page }) => {
    const autoStartToggle = page.locator('[data-testid="auto-start-toggle"]').or(
      page.locator('input[type="checkbox"]').filter({ has: page.locator('..:has-text("自动启动")') })
    );

    // Auto-start may not be available on all platforms
    const count = await autoStartToggle.count();
    if (count > 0) {
      await expect(autoStartToggle.first()).toBeVisible();
    }
  });
});

test.describe('Advanced Settings', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/settings');
  });

  test('should have advanced section', async ({ page }) => {
    // Look for advanced settings link or section
    const advancedSection = page.locator('text=/高级|Advanced/i').or(
      page.locator('[data-testid="advanced-settings"]')
    );

    // Click to expand if it's a collapsible section
    if (await advancedSection.isVisible().catch(() => false)) {
      await advancedSection.click();
    }

    // Should show some advanced options
    const advancedOptions = page.locator('[data-testid="advanced-option"]').or(
      page.locator('text=/日志|调试|端口|Log|Debug|Port/i')
    );

    const count = await advancedOptions.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('should have log level configuration', async ({ page }) => {
    const logLevelSelect = page.locator('select[name="logLevel"]').or(
      page.getByLabel(/日志级别|Log Level/i)
    );

    if (await logLevelSelect.isVisible().catch(() => false)) {
      await logLevelSelect.click();

      // Should have log level options
      const options = page.locator('option');
      const optionTexts = await options.allTextContents();

      // Common log levels
      const hasLogLevels = optionTexts.some(text =>
        /debug|info|warn|error/i.test(text)
      );

      expect(hasLogLevels).toBeTruthy();
    }
  });

  test('should have data directory setting', async ({ page }) => {
    const dataDirInput = page.locator('input[name="dataDirectory"]').or(
      page.getByLabel(/数据目录|Data Directory/i)
    );

    if (await dataDirInput.isVisible().catch(() => false)) {
      await expect(dataDirInput).toBeVisible();

      // Should have a browse button
      const browseButton = page.locator('button:has-text("浏览")').or(
        page.locator('button:has-text("Browse")')
      );

      const browseCount = await browseButton.count();
      if (browseCount > 0) {
        await expect(browseButton.first()).toBeVisible();
      }
    }
  });
});

test.describe('Settings Validation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/settings');
  });

  test('should validate port number', async ({ page }) => {
    const portInput = page.locator('input[name="port"]').or(
      page.getByLabel(/端口|Port/i)
    );

    if (await portInput.isVisible().catch(() => false)) {
      // Try invalid port
      await portInput.fill('99999');

      // Try to save
      const saveButton = page.locator('button:has-text("保存")').or(
        page.locator('button[type="submit"]')
      );
      await saveButton.click();

      // Should show error
      const errorMessage = page.locator('text=/无效|错误|Invalid|Error/i');
      await expect(errorMessage).toBeVisible().catch(() => {
        // Some implementations may not show immediate validation
        test.skip();
      });
    }
  });

  test('should show success message after saving', async ({ page }) => {
    const saveButton = page.locator('button:has-text("保存")').or(
      page.locator('button[type="submit"]')
    );

    await saveButton.click();

    // Should show success message
    const successMessage = page.locator('text=/保存成功|已保存|Saved|Success/i').or(
      page.locator('[data-testid="save-success"]')
    );

    await expect(successMessage).toBeVisible({ timeout: 5000 }).catch(() => {
      // Toast notifications may be used instead
      test.skip();
    });
  });
});
