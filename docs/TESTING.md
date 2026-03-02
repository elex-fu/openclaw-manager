# Testing Documentation

This document describes the testing strategy, structure, and guidelines for the OpenClaw Manager project.

## Overview

OpenClaw Manager uses a multi-layered testing approach:

1. **Frontend Unit Tests** - Vitest + React Testing Library
2. **Backend Unit Tests** - Rust built-in test framework
3. **Backend Integration Tests** - Rust integration tests
4. **E2E Tests** - Playwright for end-to-end testing

## Test Structure

```
openclaw-manager/
├── src/
│   ├── stores/
│   │   └── __tests__/          # Store unit tests
│   │       ├── appStore.test.ts
│   │       ├── configStore.test.ts
│   │       └── installStore.test.ts
│   ├── components/
│   │   └── (component tests can be added in __tests__/)
│   └── test/
│       ├── setup.ts            # Test setup and configuration
│       └── mocks/
│           └── tauri.ts        # Tauri API mocks
├── src-tauri/
│   ├── src/
│   │   ├── installer/
│   │   │   └── mod_test.rs     # Installer unit tests
│   │   ├── services/
│   │   │   ├── offline_installer_test.rs
│   │   │   ├── process_manager_test.rs
│   │   │   ├── secure_storage_test.rs
│   │   │   ├── skill_manager_test.rs
│   │   │   └── plugin_manager_test.rs
│   │   └── updater/
│   │       └── migration_test.rs
│   └── tests/
│       ├── install_test.rs           # Installation integration tests
│       ├── install_integration_test.rs
│       └── updater_test.rs           # Updater integration tests
├── e2e/
│   ├── installation.spec.ts    # Installation flow E2E tests
│   ├── model-config.spec.ts    # Model configuration E2E tests
│   ├── agent-manager.spec.ts   # Agent management E2E tests
│   ├── service-control.spec.ts # Service control E2E tests
│   ├── diagnostics.spec.ts     # Diagnostics E2E tests
│   └── settings.spec.ts        # Settings E2E tests
└── playwright.config.ts        # Playwright configuration
```

## Running Tests

### Frontend Tests (Vitest)

```bash
# Run all frontend tests
npm run test

# Run tests with UI
npm run test:ui

# Run tests with coverage
npm run test:coverage

# Run specific test file
npm run test -- src/stores/__tests__/appStore.test.ts

# Run tests in watch mode
npm run test -- --watch
```

### Backend Tests (Rust)

```bash
# Navigate to tauri directory
cd src-tauri

# Run all Rust tests
cargo test

# Run specific test
cargo test test_install_directory_creation

# Run tests with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test install_test
```

### E2E Tests (Playwright)

```bash
# Run all E2E tests
npx playwright test

# Run specific test file
npx playwright test e2e/installation.spec.ts

# Run with UI mode
npx playwright test --ui

# Run in headed mode (visible browser)
npx playwright test --headed

# Run with debug
npx playwright test --debug

# Run specific test
npx playwright test -g "should display model configuration page"
```

## Frontend Testing

### Store Tests

Store tests verify state management logic using Vitest.

**Location:** `src/stores/__tests__/`

**Key Test Files:**
- `appStore.test.ts` - App-level state (notifications, theme, sidebar, loading)
- `configStore.test.ts` - Configuration state (models, agents, API keys, services)
- `installStore.test.ts` - Installation state (progress, logs, wizard)

**Testing Patterns:**

```typescript
import { describe, it, expect, beforeEach } from 'vitest'
import { useAppStore } from '../appStore'

describe('appStore', () => {
  beforeEach(() => {
    // Reset store state before each test
    useAppStore.setState({
      notifications: [],
      currentPage: 'dashboard',
      theme: 'system',
      sidebarOpen: true,
      isLoading: false,
      globalError: null,
    })
  })

  it('should add notification', () => {
    const { addNotification } = useAppStore.getState()

    addNotification({
      title: 'Test',
      message: 'Test message',
      type: 'info',
    })

    const { notifications } = useAppStore.getState()
    expect(notifications).toHaveLength(1)
    expect(notifications[0].title).toBe('Test')
  })
})
```

### Component Testing Guidelines

When adding component tests:

1. Create `__tests__` directory next to the component
2. Use `@testing-library/react` for rendering
3. Mock Tauri APIs using `src/test/mocks/tauri.ts`
4. Test user interactions, not implementation details

**Example:**

```typescript
import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { Button } from '../button'

describe('Button', () => {
  it('should render and handle click', () => {
    const handleClick = vi.fn()
    render(<Button onClick={handleClick}>Click me</Button>)

    fireEvent.click(screen.getByText('Click me'))
    expect(handleClick).toHaveBeenCalled()
  })
})
```

## Backend Testing

### Unit Tests

Unit tests are co-located with source files using the `#[cfg(test)]` module pattern.

**Example:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_directory_creation() {
        use std::fs;
        use std::path::PathBuf;

        let test_dir = PathBuf::from("/tmp/test_openclaw_basic");
        let _ = fs::remove_dir_all(&test_dir);

        // Test directory creation
        let result = fs::create_dir_all(&test_dir);
        assert!(result.is_ok(), "Should create install directory");
    }
}
```

### Integration Tests

Integration tests are in `src-tauri/tests/` and test complete workflows.

**Key Integration Test Files:**
- `install_test.rs` - Installation workflow tests
- `install_integration_test.rs` - Full installation integration
- `updater_test.rs` - Update mechanism tests

## E2E Testing

### E2E Test Coverage

| Feature | Test File | Coverage |
|---------|-----------|----------|
| Installation | `installation.spec.ts` | Initial launch, progress display, wizard navigation |
| Model Config | `model-config.spec.ts` | Add/edit/delete models, provider selection, connection test |
| Agent Management | `agent-manager.spec.ts` | Create/edit agents, switching, system prompts |
| Service Control | `service-control.spec.ts` | Start/stop service, health checks, status display |
| Diagnostics | `diagnostics.spec.ts` | Run diagnostics, view results, auto-fix issues |
| Settings | `settings.spec.ts` | Theme, language, advanced options, validation |

### E2E Testing Patterns

**Navigation Pattern:**

```typescript
test.beforeEach(async ({ page }) => {
  await page.goto('/#/models');
});

test('should display model configuration page', async ({ page }) => {
  const title = page.locator('h1:has-text("模型"), h2:has-text("模型")');
  await expect(title).toBeVisible({ timeout: 10000 });
});
```

**Conditional Testing:**

```typescript
test('should allow editing existing model', async ({ page }) => {
  try {
    await modelList.waitFor({ timeout: 5000 });
    // Test editing...
  } catch {
    // No models to edit - skip test
    test.skip();
  }
});
```

**Form Testing:**

```typescript
test('should open add model dialog', async ({ page }) => {
  const addButton = page.locator('button:has-text("添加")').first();
  await addButton.click();

  const dialog = page.locator('[role="dialog"]');
  await expect(dialog).toBeVisible({ timeout: 5000 });

  // Check form fields
  await expect(page.locator('input[name="name"]')).toBeVisible();
  await expect(page.locator('select[name="provider"]')).toBeVisible();
});
```

### Adding New E2E Tests

When adding tests for new features:

1. Create new `.spec.ts` file in `e2e/` directory
2. Use descriptive test and group names
3. Add `data-testid` attributes to components for reliable selection
4. Handle both success and empty states
5. Use `test.skip()` for tests that depend on specific state

## Testing New Features

### Log Viewer Tests

When testing the Log Viewer feature:

```typescript
// e2e/log-viewer.spec.ts
test.describe('Log Viewer', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/logs');
  });

  test('should display log viewer page', async ({ page }) => {
    const title = page.locator('h1:has-text("日志"), h2:has-text("日志")');
    await expect(title).toBeVisible({ timeout: 10000 });
  });

  test('should filter logs by level', async ({ page }) => {
    const levelFilter = page.locator('[data-testid="level-filter"]');
    await levelFilter.click();
    await page.locator('text=ERROR').click();
    // Verify filtered results
  });

  test('should search logs', async ({ page }) => {
    const searchInput = page.locator('input[placeholder*="搜索"]').or(
      page.locator('[data-testid="log-search"]')
    );
    await searchInput.fill('error');
    await expect(page.locator('[data-testid="log-entry"]')).toBeVisible();
  });
});
```

### Skill Store Tests

When testing the Skill Store feature:

```typescript
// e2e/skill-store.spec.ts
test.describe('Skill Store', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/skills');
  });

  test('should display skill store page', async ({ page }) => {
    const title = page.locator('h1:has-text("技能"), h2:has-text("技能")');
    await expect(title).toBeVisible({ timeout: 10000 });
  });

  test('should search skills', async ({ page }) => {
    const searchInput = page.locator('input[placeholder*="搜索"]');
    await searchInput.fill('weather');
    await expect(page.locator('[data-testid="skill-card"]')).toBeVisible();
  });

  test('should show skill categories', async ({ page }) => {
    const categories = page.locator('[data-testid="skill-category"]');
    expect(await categories.count()).toBeGreaterThan(0);
  });
});
```

## Test Data and Mocks

### Frontend Mocks

Mock data is defined in `src/test/mocks/`:

```typescript
// src/test/mocks/models.ts
export const mockModel = {
  id: 'test-model-1',
  name: 'GPT-4',
  provider: 'openai',
  model: 'gpt-4',
  apiKey: null,
  apiBase: null,
  temperature: 0.7,
  maxTokens: 2048,
  enabled: true,
  isDefault: false,
};

export const mockAgent = {
  id: 'test-agent-1',
  name: 'Test Agent',
  modelId: 'test-model-1',
  systemPrompt: 'You are a helpful assistant.',
  skills: [],
  enabled: true,
};
```

### Tauri API Mocks

```typescript
// src/test/mocks/tauri.ts
export const mockInvoke = vi.fn((command: string, args?: unknown) => {
  switch (command) {
    case 'get_all_models':
      return Promise.resolve([mockModel]);
    case 'get_all_agents':
      return Promise.resolve([mockAgent]);
    default:
      return Promise.resolve(null);
  }
});
```

## Coverage Goals

| Layer | Target Coverage |
|-------|-----------------|
| Store Tests | 80%+ |
| Utility Functions | 80%+ |
| Component Tests | 60%+ |
| E2E Tests | Critical paths covered |

## Troubleshooting

### Common Issues

**Tests fail with "window not defined":**
- Ensure `jsdom` environment is configured in `vitest.config.ts`

**Tauri API calls fail in tests:**
- Mock the Tauri APIs using `vi.mock('@tauri-apps/api/core')`

**E2E tests timeout:**
- Increase timeout in `playwright.config.ts`
- Use `test.setTimeout()` for specific slow tests

**Rust tests fail with permission errors:**
- Run with `cargo test -- --test-threads=1` to avoid conflicts

### Debug Commands

```bash
# Run Vitest with debug output
DEBUG=vitest:* npm run test

# Run Playwright with debug
PWDEBUG=1 npx playwright test

# Run Rust tests with backtrace
RUST_BACKTRACE=1 cargo test
```

## CI/CD Integration

Tests are run in CI on every pull request:

1. Frontend unit tests (`npm run test`)
2. Backend unit tests (`cargo test`)
3. E2E tests (`npx playwright test`)

All tests must pass before merging.

## Best Practices

1. **Write tests first** for new features (TDD approach)
2. **Test behavior, not implementation** - Focus on what users see
3. **Use descriptive test names** - Tests document expected behavior
4. **Keep tests independent** - Each test should be runnable alone
5. **Clean up after tests** - Reset state, remove test files
6. **Use data-testid** for reliable E2E selectors
7. **Mock external dependencies** - Don't call real APIs in tests
