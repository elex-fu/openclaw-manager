import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { InstallWizard } from '../InstallWizard'
import { render } from '@/test/utils'
import * as tauriApi from '@/lib/tauri-api'

// Mock Tauri API - invokeWithRetry returns data directly, not ApiResponse
vi.mock('@/lib/tauri-api', () => ({
  openclawApi: {
    installOneClick: vi.fn(),
    onInstallProgress: vi.fn(() => Promise.resolve(() => {})),
  },
}))

// Mock stores
vi.mock('@/stores/appStore', () => ({
  useAppStore: () => ({
    addNotification: vi.fn(),
  }),
}))

// Mock install store with controllable state
const mockSetWizardStep = vi.fn()
const mockAddLog = vi.fn()
const mockSetProgress = vi.fn()
const mockReset = vi.fn()

vi.mock('@/stores/installStore', () => ({
  useInstallStore: () => ({
    wizardStep: 0,
    setWizardStep: mockSetWizardStep,
    logs: [],
    addLog: mockAddLog,
    progress: null,
    setProgress: mockSetProgress,
    reset: mockReset,
  }),
}))

// Mock react-router-dom
const mockNavigate = vi.fn()
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  }
})

describe('InstallWizard', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  describe('页面渲染', () => {
    it('should render without crashing', () => {
      vi.mocked(tauriApi.openclawApi.installOneClick).mockResolvedValue({
        success: true,
        version: '1.0.0',
        message: '安装成功',
      })

      render(<InstallWizard />)

      expect(screen.getByText('安装 OpenClaw')).toBeInTheDocument()
    })

    it('should display page title and description', () => {
      vi.mocked(tauriApi.openclawApi.installOneClick).mockResolvedValue({
        success: true,
        version: '1.0.0',
        message: '安装成功',
      })

      render(<InstallWizard />)

      expect(screen.getByText('安装 OpenClaw')).toBeInTheDocument()
      expect(screen.getByText('按照以下步骤完成安装')).toBeInTheDocument()
    })

    it('should display step indicators', () => {
      vi.mocked(tauriApi.openclawApi.installOneClick).mockResolvedValue({
        success: true,
        version: '1.0.0',
        message: '安装成功',
      })

      render(<InstallWizard />)

      expect(screen.getByText('初始化')).toBeInTheDocument()
      expect(screen.getByText('初始配置')).toBeInTheDocument()
    })

    it('should display step descriptions', () => {
      vi.mocked(tauriApi.openclawApi.installOneClick).mockResolvedValue({
        success: true,
        version: '1.0.0',
        message: '安装成功',
      })

      render(<InstallWizard />)

      expect(screen.getByText('解压运行环境')).toBeInTheDocument()
      expect(screen.getByText('配置模型')).toBeInTheDocument()
    })
  })

  describe('安装步骤导航', () => {
    it('should start at step 0', () => {
      vi.mocked(tauriApi.openclawApi.installOneClick).mockResolvedValue({
        success: true,
        version: '1.0.0',
        message: '安装成功',
      })

      render(<InstallWizard />)

      // Step indicators should be present
      expect(screen.getByText('初始化')).toBeInTheDocument()
    })

    it('should disable back button on first step', () => {
      vi.mocked(tauriApi.openclawApi.installOneClick).mockResolvedValue({
        success: true,
        version: '1.0.0',
        message: '安装成功',
      })

      render(<InstallWizard />)

      const backButton = screen.getByText('上一步')
      expect(backButton).toBeDisabled()
    })
  })

  describe('自动安装', () => {
    it('should start installation automatically on mount', async () => {
      vi.mocked(tauriApi.openclawApi.installOneClick).mockResolvedValue({
        success: true,
        version: '1.0.0',
        message: '安装成功',
      })

      render(<InstallWizard />)

      await waitFor(() => {
        expect(tauriApi.openclawApi.installOneClick).toHaveBeenCalledWith(true)
      }, { timeout: 3000 })
    })
  })

  describe('导航按钮', () => {
    it('should have navigation buttons', () => {
      vi.mocked(tauriApi.openclawApi.installOneClick).mockResolvedValue({
        success: true,
        version: '1.0.0',
        message: '安装成功',
      })

      render(<InstallWizard />)

      expect(screen.getByText('上一步')).toBeInTheDocument()
    })
  })
})
