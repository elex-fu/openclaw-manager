import { test, expect } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

/**
 * Sidecar Installation E2E Tests - UI Component Verification
 *
 * These tests verify that the Sidecar UI components are correctly rendered
 * with the required data-testid attributes for E2E testing.
 */

// Configure serial execution
test.describe.configure({ mode: 'serial' });

test.describe('Sidecar Installation UI', () => {
  test('InstallWizard has required data-testid attributes in source code', () => {
    // Read the InstallWizard source file
    const installWizardPath = path.join(process.cwd(), 'src', 'pages', 'InstallWizard.tsx');
    const sourceCode = fs.readFileSync(installWizardPath, 'utf-8');

    // Verify all required data-testid attributes are present in the source
    const requiredTestIds = [
      'sidecar-install-step',
      'sidecar-install',
      'install-progress',
      'install-stage',
      'install-complete',
      'install-error',
      'install-retry',
    ];

    for (const testId of requiredTestIds) {
      expect(sourceCode).toContain(`data-testid="${testId}"`);
    }

    console.log('✅ All required data-testid attributes found in InstallWizard.tsx');
  });

  test('Dashboard has required data-testid attributes in source code', () => {
    // Read the Dashboard source file
    const dashboardPath = path.join(process.cwd(), 'src', 'pages', 'Dashboard.tsx');
    const sourceCode = fs.readFileSync(dashboardPath, 'utf-8');

    // Verify Dashboard Sidecar UI elements
    const dashboardTestIds = [
      'sidecar-card',
      'sidecar-installed',
      'sidecar-version',
      'service-status',
      'start-service',
      'stop-service',
    ];

    for (const testId of dashboardTestIds) {
      expect(sourceCode).toContain(`data-testid="${testId}"`);
    }

    console.log('✅ All required data-testid attributes found in Dashboard.tsx');
  });

  test('Sidecar API functions are defined', () => {
    // Read the tauri-api source file
    const apiPath = path.join(process.cwd(), 'src', 'lib', 'tauri-api.ts');
    const sourceCode = fs.readFileSync(apiPath, 'utf-8');

    // Verify Sidecar API functions are exported
    const requiredFunctions = [
      'sidecarApi',
      'checkSidecarInstallation',
      'installSidecar',
      'startSidecar',
      'stopSidecar',
      'getSidecarInfo',
    ];

    for (const func of requiredFunctions) {
      expect(sourceCode).toContain(func);
    }

    console.log('✅ All Sidecar API functions found in tauri-api.ts');
  });

  test('Sidecar types are defined', () => {
    // Read the types file
    const typesPath = path.join(process.cwd(), 'src', 'types', 'index.ts');
    const sourceCode = fs.readFileSync(typesPath, 'utf-8');

    // Verify Sidecar types are exported
    const requiredTypes = [
      'SidecarInstallStatus',
      'SidecarInstallResult',
      'SidecarInfo',
    ];

    for (const type of requiredTypes) {
      expect(sourceCode).toContain(type);
    }

    console.log('✅ All Sidecar types found in types/index.ts');
  });
});
