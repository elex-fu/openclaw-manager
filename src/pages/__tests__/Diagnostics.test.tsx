import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { Diagnostics } from '../Diagnostics'
import { render } from '@/test/utils'
import * as tauriApi from '@/lib/tauri-api'
import type { DiagnosticResult, DiagnosticCheck } from '@/types'

// Mock Tauri API - invokeWithRetry returns data directly, not ApiResponse
vi.mock('@/lib/tauri-api', () => ({
  diagnosticsApi: {
    runDiagnostics: vi.fn(),
    fixIssue: vi.fn(),
    autoFix: vi.fn(),
  },
}))

// Mock stores
vi.mock('@/stores/appStore', () => ({
  useAppStore: () => ({
    addNotification: vi.fn(),
  }),
}))

const mockChecks: DiagnosticCheck[] = [
  {
    category: 'system',
    name: '操作系统兼容性',
    status: 'pass',
    message: '系统版本兼容',
    details: 'macOS 14.0',
    fixable: false,
  },
  {
    category: 'openclaw',
    name: '安装状态',
    status: 'error',
    message: 'OpenClaw 未安装',
    fixSuggestion: '运行安装向导',
    fixable: true,
  },
]

const mockDiagnosticResult: DiagnosticResult = {
  checks: mockChecks,
  hasErrors: true,
  hasWarnings: false,
  checkedAt: new Date().toISOString(),
}

describe('Diagnostics', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('页面渲染', () => {
    it('should render without crashing', () => {
      render(<Diagnostics />)

      expect(screen.getByText('系统诊断')).toBeInTheDocument()
    })

    it('should display page title and description', () => {
      render(<Diagnostics />)

      expect(screen.getByText('系统诊断')).toBeInTheDocument()
      expect(screen.getByText('检查系统环境并自动修复问题')).toBeInTheDocument()
    })

    it('should display run diagnostics button', () => {
      render(<Diagnostics />)

      expect(screen.getByText('运行诊断')).toBeInTheDocument()
    })
  })

  describe('诊断项目列表', () => {
    it('should display empty state before running diagnostics', () => {
      render(<Diagnostics />)

      expect(screen.getByText('尚未运行诊断')).toBeInTheDocument()
    })
  })

  describe('运行诊断功能', () => {
    it('should call runDiagnostics API when clicking run button', async () => {
      vi.mocked(tauriApi.diagnosticsApi.runDiagnostics).mockResolvedValue(mockDiagnosticResult)

      render(<Diagnostics />)

      const runButton = screen.getByText('运行诊断')
      runButton.click()

      await waitFor(() => {
        expect(tauriApi.diagnosticsApi.runDiagnostics).toHaveBeenCalled()
      })
    })

    it('should show loading state while running diagnostics', async () => {
      vi.mocked(tauriApi.diagnosticsApi.runDiagnostics).mockImplementation(
        () => new Promise(() => {}) // Never resolves
      )

      render(<Diagnostics />)

      const runButton = screen.getByText('运行诊断')
      runButton.click()

      await waitFor(() => {
        expect(screen.getByText('诊断中...')).toBeInTheDocument()
      })
    })
  })

  describe('错误处理', () => {
    it('should handle API errors gracefully without crashing', async () => {
      vi.mocked(tauriApi.diagnosticsApi.runDiagnostics).mockRejectedValue(
        new Error('Failed to run diagnostics')
      )

      render(<Diagnostics />)

      const runButton = screen.getByText('运行诊断')
      runButton.click()

      await waitFor(() => {
        expect(tauriApi.diagnosticsApi.runDiagnostics).toHaveBeenCalled()
      })
    })

    it('should handle network errors without crashing', async () => {
      vi.mocked(tauriApi.diagnosticsApi.runDiagnostics).mockRejectedValue(
        new Error('Network error')
      )

      render(<Diagnostics />)

      const runButton = screen.getByText('运行诊断')
      runButton.click()

      await waitFor(() => {
        expect(tauriApi.diagnosticsApi.runDiagnostics).toHaveBeenCalled()
      })
    })
  })
})
