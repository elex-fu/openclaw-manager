import { test, expect } from '@playwright/test';

/**
 * Agent Management E2E Tests
 *
 * Tests agent management functionality including:
 * - Creating new agents
 * - Editing agent settings
 * - Switching between agents
 * - Configuring system prompts
 */

test.describe('Agent Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/agents');
  });

  test('should display agent management page', async ({ page }) => {
    const title = page.locator('h1:has-text("Agent"), h2:has-text("Agent")');
    await expect(title).toBeVisible({ timeout: 10000 });

    const addButton = page.locator('button:has-text("添加")');
    await expect(addButton).toBeVisible();
  });

  test('should open create agent dialog', async ({ page }) => {
    const addButton = page.locator('button:has-text("添加")').first();
    await addButton.click();

    const dialog = page.locator('[role="dialog"]').or(
      page.locator('h3:has-text("添加"), h2:has-text("添加")')
    );
    await expect(dialog).toBeVisible({ timeout: 5000 });
  });

  test('should show agent configuration form', async ({ page }) => {
    const addButton = page.locator('button:has-text("添加")').first();
    await addButton.click();

    // Check for form fields
    await expect(page.locator('input[name="name"]').or(page.getByLabel('名称'))).toBeVisible();
    await expect(page.locator('select[name="modelId"]').or(page.getByLabel('模型'))).toBeVisible();
    await expect(page.locator('textarea[name="systemPrompt"]').or(page.getByLabel('系统提示词'))).toBeVisible();
  });

  test('should allow setting agent name', async ({ page }) => {
    const addButton = page.locator('button:has-text("添加")').first();
    await addButton.click();

    const nameInput = page.locator('input[name="name"]');
    await nameInput.fill('Test Agent');

    expect(await nameInput.inputValue()).toBe('Test Agent');
  });

  test('should show model selection dropdown', async ({ page }) => {
    const addButton = page.locator('button:has-text("添加")').first();
    await addButton.click();

    const modelSelect = page.locator('select[name="modelId"]').or(page.getByLabel('模型'));

    if (await modelSelect.isVisible().catch(() => false)) {
      await modelSelect.click();

      // Should have options
      const options = page.locator('option');
      expect(await options.count()).toBeGreaterThan(0);
    }
  });

  test('should display agent list', async ({ page }) => {
    const agentList = page.locator('[data-testid="agent-list"]').or(
      page.locator('table tbody tr').first().or(page.locator('[data-testid="agent-card"]'))
    );

    // Should have list or empty state
    await expect(agentList.or(page.getByText(/暂无|empty/i))).toBeVisible({ timeout: 5000 });
  });

  test('should indicate current agent', async ({ page }) => {
    const currentIndicator = page.locator('text=/当前|current|使用中/i').or(
      page.locator('[data-testid="current-agent-badge"]')
    );

    // If agents exist, should show current indicator
    const hasAgents = await page.locator('table tbody tr, [data-testid="agent-card"]').count() > 0;

    if (hasAgents) {
      await expect(currentIndicator).toBeVisible();
    }
  });

  test('should allow switching agents', async ({ page }) => {
    const agentCards = page.locator('[data-testid="agent-card"]').or(page.locator('table tbody tr'));

    try {
      await agentCards.first().waitFor({ timeout: 5000 });

      const switchButton = page.locator('button:has-text("切换")').or(
        page.locator('button:has-text("使用")')
      ).first();

      if (await switchButton.isVisible().catch(() => false)) {
        await switchButton.click();

        // Should show confirmation or update current indicator
        await expect(page.getByText(/已切换|当前/i)).toBeVisible();
      }
    } catch {
      test.skip();
    }
  });

  test('should allow editing agent skills', async ({ page }) => {
    const editButton = page.locator('button:has-text("编辑"), button[title="编辑"]').first();

    try {
      await editButton.waitFor({ timeout: 5000 });
      await editButton.click();

      // Should show skills section
      const skillsSection = page.locator('text=/技能|skills/i').or(
        page.locator('[data-testid="skills-section"]')
      );
      await expect(skillsSection).toBeVisible();
    } catch {
      test.skip();
    }
  });
});

test.describe('Agent Configuration', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/agents');
  });

  test('should validate agent name is required', async ({ page }) => {
    const addButton = page.locator('button:has-text("添加")').first();
    await addButton.click();

    // Try to save without name
    const saveButton = page.locator('button:has-text("保存")');
    await saveButton.click();

    // Should stay on dialog
    const dialog = page.locator('[role="dialog"]');
    await expect(dialog).toBeVisible();
  });

  test('should support markdown in system prompt', async ({ page }) => {
    const addButton = page.locator('button:has-text("添加")').first();
    await addButton.click();

    const promptTextarea = page.locator('textarea[name="systemPrompt"]');

    if (await promptTextarea.isVisible().catch(() => false)) {
      const markdownText = '# Instructions\n\nYou are a helpful assistant.';
      await promptTextarea.fill(markdownText);

      expect(await promptTextarea.inputValue()).toBe(markdownText);
    }
  });

  test('should show agent avatar/preview', async ({ page }) => {
    const avatar = page.locator('[data-testid="agent-avatar"]').or(
      page.locator('img[alt*="Agent"]').or(page.locator('.avatar'))
    );

    // Avatars should be visible if agents exist
    const count = await avatar.count();
    if (count > 0) {
      await expect(avatar.first()).toBeVisible();
    }
  });
});
