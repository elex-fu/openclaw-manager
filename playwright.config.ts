import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for OpenClaw Manager E2E tests
 *
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './e2e',

  /* Run tests in files in parallel - Disabled for Sidecar tests */
  fullyParallel: false,

  /* Fail the build on CI if you accidentally left test.only in the source code */
  forbidOnly: !!process.env.CI,

  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,

  /* Opt out of parallel tests on CI. Sidecar tests need to run serially */
  workers: process.env.CI ? 1 : 1,

  /* Reporter to use */
  reporter: [
    ['html', { open: 'never' }],
    ['list'],
  ],

  /* Timeout for each test - Sidecar installation needs more time */
  timeout: 300000, // 5 minutes

  /* Expect timeout */
  expect: {
    timeout: 10000,
  },

  /* Shared settings for all the projects below */
  use: {
    /* Base URL to use in actions like `await page.goto('/')` */
    baseURL: 'http://localhost:1420',

    /* Collect trace when retrying the failed test */
    trace: 'on-first-retry',

    /* Screenshot on failure */
    screenshot: 'only-on-failure',

    /* Video recording */
    video: 'on-first-retry',
  },

  /* Configure projects for major browsers */
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },

    // {
    //   name: 'firefox',
    //   use: { ...devices['Desktop Firefox'] },
    // },

    // {
    //   name: 'webkit',
    //   use: { ...devices['Desktop Safari'] },
    // },

    /* Test against mobile viewports */
    // {
    //   name: 'Mobile Chrome',
    //   use: { ...devices['Pixel 5'] },
    // },
    // {
    //   name: 'Mobile Safari',
    //   use: { ...devices['iPhone 12'] },
    // },
  ],

  /* Run your local dev server before starting the tests */
  // Note: Disabled for local testing when dev server is already running
  // Uncomment this for CI/CD where server needs to be started automatically
  // webServer: {
  //   command: 'RUSTC_WRAPPER="" npm run tauri:dev',
  //   url: 'http://localhost:1420',
  //   reuseExistingServer: !process.env.CI,
  //   timeout: 300 * 1000,
  // },
});
