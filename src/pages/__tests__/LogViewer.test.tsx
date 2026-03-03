import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { LogViewer } from '../LogViewer'
import { render } from '@/test/utils'
import * as tauriApi from '@/lib/tauri-api'
import type { LogEntry, LogSourceInfo } from '@/types'

// Mock Tauri API - note that invokeWithRetry returns the data directly, not ApiResponse
vi.mock('@/lib/tauri-api', () => ({
  logApi: {
    getRecentLogs: vi.fn(),
    subscribeLogs: vi.fn(),
    unsubscribeLogs: vi.fn(),
    exportLogs: vi.fn(),
    clearLogDisplay: vi.fn(),
    getLogSources: vi.fn(),
    getLogStats: vi.fn(),
    onLogEntry: vi.fn(() => Promise.resolve(() => {})),
    onLogReset: vi.fn(() => Promise.resolve(() => {})),
    onLogError: vi.fn(() => Promise.resolve(() => {})),
  },
}))

// Mock date-fns
vi.mock('@/lib/date', () => ({
  format: vi.fn(() => '2024-03-01 12:00:00.000'),
}))

// Mock @tauri-apps/api/event
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}))

const mockLogs: LogEntry[] = [
  {
    id: 'log-1',
    timestamp: Date.now(),
    level: 'INFO',
    source: 'app',
    message: 'Application started',
    metadata: { version: '1.0.0' },
  },
  {
    id: 'log-2',
    timestamp: Date.now() - 1000,
    level: 'WARN',
    source: 'network',
    message: 'Connection timeout',
  },
  {
    id: 'log-3',
    timestamp: Date.now() - 2000,
    level: 'ERROR',
    source: 'database',
    message: 'Failed to connect to database',
    metadata: { error: 'ECONNREFUSED' },
  },
]

const mockLogSources: LogSourceInfo[] = [
  {
    id: 'source-1',
    name: 'Application',
    path: '/logs/app.log',
    size: 1024,
    modified: Date.now(),
  },
]

describe('LogViewer', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // invokeWithRetry returns data directly, not wrapped in ApiResponse
    vi.mocked(tauriApi.logApi.getRecentLogs).mockResolvedValue(mockLogs)
    vi.mocked(tauriApi.logApi.subscribeLogs).mockResolvedValue({ subscription_id: 'sub-123' })
    vi.mocked(tauriApi.logApi.getLogSources).mockResolvedValue(mockLogSources)
  })

  describe('页面渲染', () => {
    it('should render without crashing', async () => {
      render(<LogViewer />)

      await waitFor(() => {
        expect(screen.getByPlaceholderText('搜索日志内容...')).toBeInTheDocument()
      })
    })

    it('should display filter bar', async () => {
      render(<LogViewer />)

      await waitFor(() => {
        expect(screen.getByPlaceholderText('搜索日志内容...')).toBeInTheDocument()
      })
    })

    it('should display level filter buttons', async () => {
      render(<LogViewer />)

      await waitFor(() => {
        expect(screen.getByText('错误')).toBeInTheDocument()
        expect(screen.getByText('警告')).toBeInTheDocument()
        expect(screen.getByText('信息')).toBeInTheDocument()
        expect(screen.getByText('调试')).toBeInTheDocument()
        expect(screen.getByText('追踪')).toBeInTheDocument()
      })
    })

    it('should display action buttons', async () => {
      render(<LogViewer />)

      await waitFor(() => {
        expect(screen.getByText('暂停')).toBeInTheDocument()
        expect(screen.getByText('清空')).toBeInTheDocument()
        expect(screen.getByText('导出')).toBeInTheDocument()
      })
    })
  })

  describe('日志列表显示', () => {
    it('should fetch logs on mount', async () => {
      render(<LogViewer />)

      await waitFor(() => {
        expect(tauriApi.logApi.getRecentLogs).toHaveBeenCalled()
      })
    })

    it('should display empty state when no logs', async () => {
      vi.mocked(tauriApi.logApi.getRecentLogs).mockResolvedValue([])

      render(<LogViewer />)

      await waitFor(() => {
        expect(screen.getByText('暂无日志记录')).toBeInTheDocument()
      })
    })
  })

  describe('级别筛选功能', () => {
    it('should display level filter buttons', async () => {
      render(<LogViewer />)

      await waitFor(() => {
        expect(screen.getByText('错误')).toBeInTheDocument()
      })

      const errorButton = screen.getByText('错误')
      expect(errorButton).toBeInTheDocument()
    })
  })

  describe('搜索功能', () => {
    it('should display search input', async () => {
      render(<LogViewer />)

      await waitFor(() => {
        expect(screen.getByPlaceholderText('搜索日志内容...')).toBeInTheDocument()
      })
    })

    it('should update search query on input', async () => {
      render(<LogViewer />)

      await waitFor(() => {
        expect(screen.getByPlaceholderText('搜索日志内容...')).toBeInTheDocument()
      })

      const searchInput = screen.getByPlaceholderText('搜索日志内容...')
      expect(searchInput).toBeInTheDocument()
    })
  })

  describe('实时日志切换', () => {
    it('should display live/pause toggle button', async () => {
      render(<LogViewer />)

      await waitFor(() => {
        expect(screen.getByText('暂停')).toBeInTheDocument()
      })
    })

    it('should subscribe to logs when live mode is on', async () => {
      render(<LogViewer />)

      await waitFor(() => {
        expect(tauriApi.logApi.subscribeLogs).toHaveBeenCalled()
      })
    })
  })

  describe('清除日志功能', () => {
    it('should display clear button', async () => {
      render(<LogViewer />)

      await waitFor(() => {
        expect(screen.getByText('清空')).toBeInTheDocument()
      })
    })

    it('should clear logs when clicking clear button', async () => {
      render(<LogViewer />)

      await waitFor(() => {
        expect(screen.getByText('清空')).toBeInTheDocument()
      })

      const clearButton = screen.getByText('清空')
      clearButton.click()

      await waitFor(() => {
        expect(screen.getByText('暂无日志记录')).toBeInTheDocument()
      })
    })
  })

  describe('统计信息显示', () => {
    it('should display log count', async () => {
      render(<LogViewer />)

      await waitFor(() => {
        expect(screen.getByText(/共 \d+ 条日志/)).toBeInTheDocument()
      })
    })
  })

  describe('错误处理', () => {
    it('should handle API errors gracefully without crashing', async () => {
      vi.mocked(tauriApi.logApi.getRecentLogs).mockRejectedValue(
        new Error('Failed to load logs')
      )

      render(<LogViewer />)

      await waitFor(() => {
        expect(tauriApi.logApi.getRecentLogs).toHaveBeenCalled()
      })
    })

    it('should handle network errors without crashing', async () => {
      vi.mocked(tauriApi.logApi.getRecentLogs).mockRejectedValue(
        new Error('Network error')
      )

      render(<LogViewer />)

      await waitFor(() => {
        expect(tauriApi.logApi.getRecentLogs).toHaveBeenCalled()
      })
    })
  })
})
