export const mockTauriApi = {
  invoke: vi.fn(),
  listen: vi.fn(() => Promise.resolve(() => {})),
}

// 模拟 Tauri API 响应
export const mockOpenClawInstallation = {
  success: true,
  data: { type: 'Installed', version: '1.0.0' },
  error: null,
}
