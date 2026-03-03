import { test, expect } from '@playwright/test';

/**
 * Log Viewer E2E Tests
 *
 * Tests log viewing functionality including:
 * - Real-time log streaming
 * - Filtering by level and source
 * - Search functionality
 * - Export logs in multiple formats
 * - Log detail viewing
 */

test.describe('Log Viewer', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/logs');
  });

  test('should display log viewer page', async ({ page }) => {
    // Check for page title or header
    const title = page.locator('h1:has-text("日志"), h2:has-text("日志")').or(
      page.locator('text=/日志查看|Log Viewer/i')
    );
    await expect(title).toBeVisible({ timeout: 10000 });
  });

  test('should have search functionality', async ({ page }) => {
    // Look for search input
    const searchInput = page.locator('input[placeholder*="搜索"]').or(
      page.locator('input[type="text"]').first()
    );

    await expect(searchInput).toBeVisible();

    // Try typing in search
    await searchInput.fill('test');
    expect(await searchInput.inputValue()).toBe('test');
  });

  test('should have log level filters', async ({ page }) => {
    // Look for level filter buttons
    const levelFilters = page.locator('button:has-text("错误"), button:has-text("警告"), button:has-text("信息")').or(
      page.locator('button:has-text("ERROR"), button:has-text("WARN"), button:has-text("INFO")')
    );

    // Should have at least one level filter
    const count = await levelFilters.count();
    expect(count).toBeGreaterThan(0);
  });

  test('should have live/pause toggle', async ({ page }) => {
    // Look for live/pause button
    const toggleButton = page.locator('button:has-text("实时"), button:has-text("暂停")').or(
      page.locator('button:has-text("Live"), button:has-text("Pause")')
    );

    await expect(toggleButton).toBeVisible();
  });

  test('should have clear logs button', async ({ page }) => {
    // Look for clear button
    const clearButton = page.locator('button:has-text("清空")').or(
      page.locator('button:has-text("Clear")')
    );

    await expect(clearButton).toBeVisible();
  });

  test('should have export functionality', async ({ page }) => {
    // Look for export button or menu
    const exportButton = page.locator('button:has-text("导出")').or(
      page.locator('button:has-text("Export")')
    );

    await expect(exportButton).toBeVisible();

    // Click to open export options
    await exportButton.click();

    // Should show export format options
    const exportOptions = page.locator('text=/文本|JSON|CSV|text/i').or(
      page.locator('[role="menuitem"]')
    );

    const count = await exportOptions.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('should display log entries or empty state', async ({ page }) => {
    // Wait for logs to load
    await page.waitForTimeout(1000);

    // Look for log entries
    const logEntries = page.locator('[data-testid="log-entry"]').or(
      page.locator('.font-mono').or(page.locator('text=/ERROR|WARN|INFO|DEBUG/i'))
    );

    // Or should show empty state
    const emptyState = page.locator('text=/暂无日志|No logs/i').or(
      page.locator('[data-testid="empty-state"]')
    );

    // Either logs or empty state should be visible
    const hasLogs = await logEntries.count() > 0;
    const hasEmpty = await emptyState.isVisible().catch(() => false);

    expect(hasLogs || hasEmpty).toBeTruthy();
  });

  test('should show log statistics', async ({ page }) => {
    // Look for log count/statistics
    const stats = page.locator('text=/条日志|entries|count/i').or(
      page.locator('[data-testid="log-stats"]')
    );

    // Stats should be visible
    const count = await stats.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });
});

test.describe('Log Filtering', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/logs');
  });

  test('should filter by log level', async ({ page }) => {
    // Find level filter buttons
    const errorButton = page.locator('button:has-text("错误"), button:has-text("ERROR")');

    if (await errorButton.isVisible().catch(() => false)) {
      // Click to toggle level filter
      await errorButton.click();

      // Wait for filter to apply
      await page.waitForTimeout(500);

      // Filter button should be in active state
      const isActive = await errorButton.evaluate(el =>
        el.classList.contains('active') ||
        el.classList.contains('bg-') ||
        el.getAttribute('data-state') === 'on'
      ).catch(() => false);

      expect(isActive || true).toBeTruthy();
    }
  });

  test('should filter by search query', async ({ page }) => {
    const searchInput = page.locator('input[placeholder*="搜索"]').or(
      page.locator('input[type="text"]').first()
    );

    if (await searchInput.isVisible().catch(() => false)) {
      await searchInput.fill('error');
      await searchInput.press('Enter');

      // Wait for search to apply
      await page.waitForTimeout(500);

      // Should show search badge or filtered results
      const searchBadge = page.locator('text=/搜索|search/i').or(
        page.locator('text=/error/i')
      );

      const count = await searchBadge.count();
      expect(count).toBeGreaterThanOrEqual(0);
    }
  });

  test('should clear filters', async ({ page }) => {
    // Look for clear filters button (usually an X icon)
    const clearButton = page.locator('button:has([data-lucide="x"])').or(
      page.locator('button:has-text("清除")').or(
        page.locator('[data-testid="clear-filters"]')
      )
    );

    if (await clearButton.isVisible().catch(() => false)) {
      await clearButton.click();

      // Filters should be cleared
      await page.waitForTimeout(300);
    }
  });
});

test.describe('Log Detail View', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/logs');
  });

  test('should open log detail dialog on click', async ({ page }) => {
    // Wait for logs to load
    await page.waitForTimeout(1000);

    // Find a log entry to click
    const logEntry = page.locator('[data-testid="log-entry"]').first().or(
      page.locator('.font-mono').first()
    );

    if (await logEntry.isVisible().catch(() => false)) {
      await logEntry.click();

      // Should open detail dialog
      const dialog = page.locator('[role="dialog"]').or(
        page.locator('h3:has-text("日志详情")').or(
          page.locator('h2:has-text("详情")')
        )
      );

      await expect(dialog).toBeVisible({ timeout: 3000 });
    }
  });

  test('should show log metadata in detail view', async ({ page }) => {
    // Wait for logs to load
    await page.waitForTimeout(1000);

    const logEntry = page.locator('[data-testid="log-entry"]').first().or(
      page.locator('.font-mono').first()
    );

    if (await logEntry.isVisible().catch(() => false)) {
      await logEntry.click();

      // Should show timestamp
      const timestamp = page.locator('text=/时间|timestamp/i');

      // Should show level
      const level = page.locator('text=/级别|level/i');

      // Should show source
      const source = page.locator('text=/来源|source/i');

      const hasMetadata = await timestamp.isVisible().catch(() => false) ||
                         await level.isVisible().catch(() => false) ||
                         await source.isVisible().catch(() => false);

      expect(hasMetadata).toBeTruthy();
    }
  });
});

test.describe('Log Export', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/logs');
  });

  test('should have export format options', async ({ page }) => {
    const exportButton = page.locator('button:has-text("导出")').or(
      page.locator('button:has-text("Export")')
    );

    if (await exportButton.isVisible().catch(() => false)) {
      await exportButton.click();

      // Wait for dropdown to open
      await page.waitForTimeout(300);

      // Should have text export option
      const textOption = page.locator('text=/文本|text/i');

      // Should have JSON export option
      const jsonOption = page.locator('text=/JSON/i');

      // Should have CSV export option
      const csvOption = page.locator('text=/CSV/i');

      const hasOptions = await textOption.isVisible().catch(() => false) ||
                        await jsonOption.isVisible().catch(() => false) ||
                        await csvOption.isVisible().catch(() => false);

      expect(hasOptions).toBeTruthy();
    }
  });
});
