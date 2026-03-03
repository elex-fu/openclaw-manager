import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ResourceMonitor } from '../ResourceMonitor';
import * as tauriApi from '@/lib/tauri-api';

// Mock the tauri-api module
vi.mock('@/lib/tauri-api', () => ({
  systemApi: {
    getSystemResources: vi.fn(),
  },
}));

describe('ResourceMonitor', () => {
  const mockResources = {
    cpu: {
      usage: 45.5,
      cores: 8,
      name: 'Intel Core i7',
      frequency: 2400,
    },
    memory: {
      used: 8192,
      total: 16384,
      usage: 50,
      available: 8192,
    },
    disk: {
      used: 256,
      total: 512,
      usage: 50,
      free: 256,
    },
    timestamp: Date.now() / 1000,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('renders correctly with title', () => {
    vi.mocked(tauriApi.systemApi.getSystemResources).mockResolvedValueOnce(mockResources);
    render(<ResourceMonitor />);

    expect(screen.getByText('资源监控')).toBeInTheDocument();
    expect(screen.getByText('系统资源使用情况')).toBeInTheDocument();
  });

  it('displays CPU section', () => {
    vi.mocked(tauriApi.systemApi.getSystemResources).mockResolvedValueOnce(mockResources);
    render(<ResourceMonitor />);

    expect(screen.getByText('CPU 使用率')).toBeInTheDocument();
  });

  it('displays memory section', () => {
    vi.mocked(tauriApi.systemApi.getSystemResources).mockResolvedValueOnce(mockResources);
    render(<ResourceMonitor />);

    expect(screen.getByText('内存使用')).toBeInTheDocument();
  });

  it('displays disk section', () => {
    vi.mocked(tauriApi.systemApi.getSystemResources).mockResolvedValueOnce(mockResources);
    render(<ResourceMonitor />);

    expect(screen.getByText('磁盘使用')).toBeInTheDocument();
  });
});
