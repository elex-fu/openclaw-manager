import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for OpenClaw Manager E2E tests
 *
 * Optimized for stability and reliability:
 * - Smart timeout configuration
 * - Retry strategy for CI
 * - Proper test isolation
 * - Enhanced reporting
 *
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './e2e',

  /* Run tests in files in parallel - Disabled for Sidecar tests */
  fullyParallel: false,

  /* Fail the build on CI if you accidentally left test.only in the source code */
  forbidOnly: !!process.env.CI,

  /* Retry on CI only (2 retries), no retries locally for faster feedback */
  retries: process.env.CI ? 2 : 0,

  /* Opt out of parallel tests. Sidecar tests need to run serially */
  workers: 1,

  /* Reporter to use - HTML, list, and JSON for CI */
  reporter: [
    ['html', { open: 'never' }],
    ['list'],
    ['json', { outputFile: 'test-results.json' }],
  ],

  /* Timeout for each test - Sidecar installation needs more time */
  timeout: 300000, // 5 minutes

  /* Expect timeout - increased for better stability */
  expect: {
    timeout: 15000, // 15 seconds
  },

  /* Shared settings for all the projects below */
  use: {
    /* Base URL to use in actions like `await page.goto('/')` */
    baseURL: process.env.BASE_URL || 'http://localhost:1420',

    /* Collect trace when retrying the failed test */
    trace: 'on-first-retry',

    /* Screenshot on failure */
    screenshot: 'only-on-failure',

    /* Video recording - retain on failure for debugging */
    video: 'retain-on-failure',

    /* Browser context options for stability */
    contextOptions: {
      /* Reduce animations for more stable tests */
      reducedMotion: 'reduce',
    },

    /* Action timeout for clicks, fills, etc. */
    actionTimeout: 10000,

    /* Navigation timeout */
    navigationTimeout: 30000,
  },

  /* Configure projects for major browsers */
  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        /* Disable GPU acceleration for more stable tests */
        launchOptions: {
          args: ['--disable-gpu', '--no-sandbox', '--disable-dev-shm-usage'],
        },
      },
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

  /* Global setup and teardown */
  // globalSetup: require.resolve('./e2e/setup/global-setup'),
  // globalTeardown: require.resolve('./e2e/setup/global-teardown'),

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
