import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { ActivityLog } from '../ActivityLog';
import * as tauriApi from '@/lib/tauri-api';

// Mock the tauri-api module
vi.mock('@/lib/tauri-api', () => ({
  systemApi: {
    getRecentActivities: vi.fn(),
  },
}));

describe('ActivityLog', () => {
  const mockActivities = [
    {
      id: '1',
      timestamp: Date.now() / 1000 - 60,
      activity_type: 'service' as const,
      message: 'Gateway 服务已启动',
      details: '服务PID: 12345',
    },
    {
      id: '2',
      timestamp: Date.now() / 1000 - 300,
      activity_type: 'config' as const,
      message: '模型配置已更新',
      details: '模型: GPT-4',
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('renders correctly with title', () => {
    vi.mocked(tauriApi.systemApi.getRecentActivities).mockResolvedValueOnce([]);
    render(<ActivityLog />);

    expect(screen.getByText('最近活动')).toBeInTheDocument();
    expect(screen.getByText('系统操作日志')).toBeInTheDocument();
  });

  it('displays empty state when no activities', async () => {
    vi.mocked(tauriApi.systemApi.getRecentActivities).mockResolvedValueOnce([]);
    render(<ActivityLog />);

    await waitFor(() => {
      expect(screen.getByText('暂无活动记录')).toBeInTheDocument();
    });
  });

  it('displays activities with correct labels', async () => {
    vi.mocked(tauriApi.systemApi.getRecentActivities).mockResolvedValueOnce(mockActivities);
    render(<ActivityLog />);

    await waitFor(() => {
      expect(screen.getByText('Gateway 服务已启动')).toBeInTheDocument();
    });

    expect(screen.getByText('模型配置已更新')).toBeInTheDocument();

    // Check labels
    expect(screen.getByText('服务')).toBeInTheDocument();
    expect(screen.getByText('配置')).toBeInTheDocument();
  });

  it('displays activity details', async () => {
    vi.mocked(tauriApi.systemApi.getRecentActivities).mockResolvedValueOnce(mockActivities);
    render(<ActivityLog />);

    await waitFor(() => {
      expect(screen.getByText('服务PID: 12345')).toBeInTheDocument();
    });

    expect(screen.getByText('模型: GPT-4')).toBeInTheDocument();
  });
});
