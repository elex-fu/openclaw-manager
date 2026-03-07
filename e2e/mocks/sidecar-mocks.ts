/**
 * Sidecar Mock Utilities for E2E Tests
 *
 * Provides mock implementations for Sidecar-related Tauri commands
 */

import { Page } from '@playwright/test';

export interface MockInstallProgress {
  stage: string;
  percentage: number;
  message: string;
}

/**
 * Mock Sidecar as not installed
 */
export async function mockSidecarNotInstalled(page: Page): Promise<void> {
  await page.evaluate(() => {
    (window as any).__TAURI__ = {
      ...(window as any).__TAURI__,
      invoke: async (cmd: string, args?: any) => {
        if (cmd === 'get_sidecar_status') {
          return {
            success: true,
            data: {
              installed: false,
              version: null,
              nodeVersion: null,
              canInstall: true,
            },
          };
        }
        if (cmd === 'check_installation') {
          return {
            success: true,
            data: {
              installed: false,
              installPath: null,
              version: null,
            },
          };
        }
        // Fall through to original invoke
        return (window as any).__TAURI__?.__originalInvoke?.(cmd, args);
      },
    };
  });
}

/**
 * Mock Sidecar as installed with specific version
 */
export async function mockSidecarInstalled(page: Page, version: string = '2026.2.27'): Promise<void> {
  await page.evaluate((ver) => {
    (window as any).__TAURI__ = {
      ...(window as any).__TAURI__,
      invoke: async (cmd: string, args?: any) => {
        if (cmd === 'get_sidecar_status') {
          return {
            success: true,
            data: {
              installed: true,
              version: ver,
              nodeVersion: '20.11.0',
              canInstall: false,
            },
          };
        }
        if (cmd === 'check_installation') {
          return {
            success: true,
            data: {
              installed: true,
              installPath: '/home/user/.openclaw/app',
              version: ver,
            },
          };
        }
        if (cmd === 'get_service_status') {
          return {
            success: true,
            data: {
              running: false,
              pid: null,
              uptime: 0,
            },
          };
        }
        // Fall through to original invoke
        return (window as any).__TAURI__?.__originalInvoke?.(cmd, args);
      },
    };
  }, version);
}

/**
 * Mock Sidecar installation with success flow
 */
export async function mockSidecarInstallSuccess(page: Page): Promise<void> {
  const progressStages: MockInstallProgress[] = [
    { stage: 'Checking', percentage: 10, message: '检查安装环境...' },
    { stage: 'Downloading', percentage: 30, message: '下载 Node.js...' },
    { stage: 'Extracting', percentage: 50, message: '解压 OpenClaw...' },
    { stage: 'Installing', percentage: 70, message: '运行 npm install...' },
    { stage: 'Finalizing', percentage: 90, message: '完成安装...' },
    { stage: 'Complete', percentage: 100, message: '安装成功' },
  ];

  await page.evaluate((stages) => {
    let currentStage = 0;
    let installStarted = false;

    (window as any).__TAURI__ = {
      ...(window as any).__TAURI__,
      invoke: async (cmd: string, args?: any) => {
        if (cmd === 'install_sidecar') {
          installStarted = true;
          currentStage = 0;

          // Emit progress events
          const emitProgress = () => {
            if (currentStage < stages.length) {
              const progress = stages[currentStage];
              (window as any).__TAURI__?.event?.emit?.('install-progress', progress);
              currentStage++;
              if (currentStage < stages.length) {
                setTimeout(emitProgress, 1000);
              }
            }
          };

          setTimeout(emitProgress, 500);

          return {
            success: true,
            data: { started: true },
          };
        }

        if (cmd === 'get_install_progress') {
          return {
            success: true,
            data: installStarted ? stages[Math.min(currentStage, stages.length - 1)] : null,
          };
        }

        // Fall through to original invoke
        return (window as any).__TAURI__?.__originalInvoke?.(cmd, args);
      },
    };
  }, progressStages);
}

/**
 * Mock Sidecar installation with network error
 */
export async function mockSidecarNetworkError(page: Page): Promise<void> {
  await page.evaluate(() => {
    (window as any).__TAURI__ = {
      ...(window as any).__TAURI__,
      invoke: async (cmd: string, args?: any) => {
        if (cmd === 'install_sidecar') {
          // Emit error progress
          setTimeout(() => {
            (window as any).__TAURI__?.event?.emit?.('install-progress', {
              stage: 'Error',
              percentage: 0,
              message: '网络错误: 无法下载 Node.js',
              error: true,
            });
          }, 1000);

          return {
            success: true,
            data: { started: true },
          };
        }

        if (cmd === 'get_install_progress') {
          return {
            success: true,
            data: {
              stage: 'Error',
              percentage: 0,
              message: '网络错误: 无法下载 Node.js',
              error: true,
            },
          };
        }

        // Fall through to original invoke
        return (window as any).__TAURI__?.__originalInvoke?.(cmd, args);
      },
    };
  });
}

/**
 * Mock Sidecar service control
 */
export async function mockSidecarServiceControl(page: Page): Promise<void> {
  await page.evaluate(() => {
    let serviceRunning = false;
    let servicePid: number | null = null;

    (window as any).__TAURI__ = {
      ...(window as any).__TAURI__,
      invoke: async (cmd: string, args?: any) => {
        if (cmd === 'start_service') {
          serviceRunning = true;
          servicePid = 12345;

          setTimeout(() => {
            (window as any).__TAURI__?.event?.emit?.('service-status', {
              running: true,
              pid: servicePid,
              uptime: 0,
            });
          }, 500);

          return {
            success: true,
            data: { started: true, pid: servicePid },
          };
        }

        if (cmd === 'stop_service') {
          serviceRunning = false;
          servicePid = null;

          setTimeout(() => {
            (window as any).__TAURI__?.event?.emit?.('service-status', {
              running: false,
              pid: null,
              uptime: 0,
            });
          }, 500);

          return {
            success: true,
            data: { stopped: true },
          };
        }

        if (cmd === 'get_service_status') {
          return {
            success: true,
            data: {
              running: serviceRunning,
              pid: servicePid,
              uptime: serviceRunning ? 10 : 0,
            },
          };
        }

        // Fall through to original invoke
        return (window as any).__TAURI__?.__originalInvoke?.(cmd, args);
      },
    };
  });
}

/**
 * Clear all Sidecar mocks
 */
export async function clearSidecarMocks(page: Page): Promise<void> {
  await page.evaluate(() => {
    if ((window as any).__TAURI__?.__originalInvoke) {
      (window as any).__TAURI__.invoke = (window as any).__TAURI__.__originalInvoke;
    }
  });
}
