import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { DiagnosticAlerts } from '../DiagnosticAlerts';
import * as tauriApi from '@/lib/tauri-api';
import { BrowserRouter } from 'react-router-dom';

// Mock the tauri-api module
vi.mock('@/lib/tauri-api', () => ({
  systemApi: {
    getDiagnosticAlerts: vi.fn(),
  },
}));

// Mock useNavigate
const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

describe('DiagnosticAlerts', () => {
  const mockAlerts = [
    {
      id: '1',
      severity: 'warning' as const,
      title: '磁盘空间不足',
      message: '磁盘使用率超过 80%，建议清理空间',
      fixable: false,
      category: 'system',
    },
    {
      id: '2',
      severity: 'info' as const,
      title: '配置备份提醒',
      message: '建议定期备份配置以防数据丢失',
      fixable: true,
      category: 'config',
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  const renderWithRouter = (component: React.ReactNode) => {
    return render(<BrowserRouter>{component}</BrowserRouter>);
  };

  it('renders correctly with title', () => {
    vi.mocked(tauriApi.systemApi.getDiagnosticAlerts).mockResolvedValueOnce([]);
    renderWithRouter(<DiagnosticAlerts />);

    expect(screen.getByText('诊断警告')).toBeInTheDocument();
  });

  it('displays empty state when no alerts', async () => {
    vi.mocked(tauriApi.systemApi.getDiagnosticAlerts).mockResolvedValueOnce([]);
    renderWithRouter(<DiagnosticAlerts />);

    await waitFor(() => {
      expect(screen.getByText('未发现任何问题')).toBeInTheDocument();
    });

    expect(screen.getByText('系统运行状况良好')).toBeInTheDocument();
  });

  it('displays alerts with correct information', async () => {
    vi.mocked(tauriApi.systemApi.getDiagnosticAlerts).mockResolvedValueOnce(mockAlerts);
    renderWithRouter(<DiagnosticAlerts />);

    await waitFor(() => {
      expect(screen.getByText('磁盘空间不足')).toBeInTheDocument();
    });

    expect(screen.getByText('配置备份提醒')).toBeInTheDocument();

    // Check category labels
    expect(screen.getByText('系统')).toBeInTheDocument();
    expect(screen.getByText('配置')).toBeInTheDocument();
  });

  it('displays fix button for fixable alerts', async () => {
    vi.mocked(tauriApi.systemApi.getDiagnosticAlerts).mockResolvedValueOnce(mockAlerts);
    renderWithRouter(<DiagnosticAlerts />);

    await waitFor(() => {
      expect(screen.getByText('快速修复')).toBeInTheDocument();
    });
  });

  it('navigates to diagnostics page on button click', async () => {
    vi.mocked(tauriApi.systemApi.getDiagnosticAlerts).mockResolvedValueOnce([]);
    renderWithRouter(<DiagnosticAlerts />);

    await waitFor(() => {
      expect(screen.getByText('查看全部')).toBeInTheDocument();
    });

    const viewAllButton = screen.getByText('查看全部');
    fireEvent.click(viewAllButton);

    expect(mockNavigate).toHaveBeenCalledWith('/diagnostics');
  });
});
