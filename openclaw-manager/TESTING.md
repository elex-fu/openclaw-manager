# Testing Guide

This document describes the testing setup and how to run tests for OpenClaw Manager.

## Test Structure

```
openclaw-manager/
├── e2e/                    # Playwright E2E tests
│   ├── installation.spec.ts
│   ├── model-config.spec.ts
│   ├── agent-manager.spec.ts
│   ├── diagnostics.spec.ts
│   ├── service-control.spec.ts
│   └── settings.spec.ts
├── src/
│   ├── __tests__/         # Frontend unit tests
│   └── stores/__tests__/  # Store tests
└── src-tauri/
    ├── src/               # Rust source with inline tests
    └── tests/             # Rust integration tests
```

## Running Tests

### Frontend Unit Tests (Vitest)

```bash
# Run all tests
npm run test

# Run with UI
npm run test:ui

# Run with coverage
npm run test:coverage

# Run specific test file
npm run test -- src/stores/__tests__/appStore.test.ts
```

### Rust Tests

```bash
cd src-tauri

# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_install_directory_creation
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

# Generate report
npx playwright show-report
```

## E2E Test Coverage

| Test File | Coverage Area | Test Count |
|-----------|--------------|------------|
| installation.spec.ts | Installation flow, wizard, progress | 10 |
| model-config.spec.ts | Model CRUD, provider selection, API key | 12 |
| agent-manager.spec.ts | Agent creation, switching, skills | 14 |
| diagnostics.spec.ts | System diagnostics, auto-fix | 14 |
| service-control.spec.ts | Service start/stop, health check | 7 |
| settings.spec.ts | General settings, advanced options | 11 |

**Total: 68 E2E test cases**

## CI/CD

Tests run automatically on:
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop`

### CI Jobs

1. **Frontend Tests**: ESLint, build, Vitest tests
2. **Rust Tests**: Format check, Clippy, cargo test (multi-platform)
3. **E2E Tests**: Playwright tests on Ubuntu, Windows, macOS
4. **Build Test**: Ensures app builds on all platforms

### Coverage Reporting

Coverage reports are uploaded to:
- Codecov (frontend)
- GitHub Actions artifacts (E2E screenshots and reports)

## Pre-commit Hooks

Husky + lint-staged are configured to run:
- ESLint --fix
- Prettier --write
- Rust fmt and clippy (on .rs files)

```bash
# Install hooks
npm run prepare

# Manual hook test
npx lint-staged
```

## Writing Tests

### E2E Test Pattern

```typescript
import { test, expect } from '@playwright/test';

test.describe('Feature Name', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/route');
  });

  test('should do something', async ({ page }) => {
    // Arrange
    const button = page.locator('button:has-text("Click")');

    // Act
    await button.click();

    // Assert
    await expect(page.locator('.result')).toBeVisible();
  });
});
```

### Rust Test Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = ...;

        // Act
        let result = function_under_test(input);

        // Assert
        assert!(result.is_ok());
    }
}
```

## Troubleshooting

### Playwright browsers not installed

```bash
npx playwright install --with-deps chromium
```

### Rust build failures

```bash
cd src-tauri
cargo clean
cargo update
cargo build
```

### Frontend test failures

```bash
rm -rf node_modules
npm ci
npm run test
```
