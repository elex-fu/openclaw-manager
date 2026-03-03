import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import { SettingsPage } from '../SettingsPage'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { AppSettings, SystemSettings } from '@/types'

// Mock the config store
const mockSetTheme = vi.fn()
const mockSetStartupSetting = vi.fn()
const mockSetNotificationEnabled = vi.fn()
const mockSetNotificationFilter = vi.fn()
const mockUpdateSettings = vi.fn()
const mockResetAllSettings = vi.fn()

const mockAppSettings: AppSettings = {
  theme: 'system',
  language: 'zh-CN',
  startup: {
    auto_start: false,
    minimize_to_tray: true,
    check_update_on_start: true,
  },
  notifications: {
    enabled: true,
    filter: {
      info: true,
      warning: true,
      error: true,
      success: true,
    },
  },
}

const mockSystemSettings: SystemSettings = {
  log_level: 'info',
  auto_update: true,
  theme: 'system',
  language: 'zh-CN',
  custom_vars: {},
}

vi.mock('@/stores/configStore', () => ({
  useConfigStore: () => ({
    appSettings: mockAppSettings,
    settings: mockSystemSettings,
    setTheme: mockSetTheme,
    setStartupSetting: mockSetStartupSetting,
    setNotificationEnabled: mockSetNotificationEnabled,
    setNotificationFilter: mockSetNotificationFilter,
    updateSettings: mockUpdateSettings,
    resetAllSettings: mockResetAllSettings,
    models: [],
    agents: [],
  }),
}))

// Mock Tauri API
vi.mock('@/lib/tauri-api', () => ({
  openclawApi: {
    checkInstallation: vi.fn().mockResolvedValue({
      success: true,
      data: { type: 'Installed', version: '1.0.0' },
    }),
  },
}))

// Mock the InstallerPanel component
vi.mock('@/components/openclaw/InstallerPanel', () => ({
  InstallerPanel: () => <div data-testid="installer-panel">Installer Panel</div>,
}))

const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  })

const renderWithProviders = (component: React.ReactNode) => {
  const queryClient = createTestQueryClient()
  return render(
    <QueryClientProvider client={queryClient}>{component}</QueryClientProvider>
  )
}

describe('SettingsPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders settings page with tabs', () => {
    renderWithProviders(<SettingsPage />)

    expect(screen.getByText('设置')).toBeInTheDocument()
    expect(screen.getByText('配置应用偏好和系统选项')).toBeInTheDocument()

    // Check all tabs are present
    expect(screen.getByRole('tab', { name: /外观/i })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: /启动/i })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: /通知/i })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: /OpenClaw/i })).toBeInTheDocument()
    expect(screen.getByRole('tab', { name: /高级/i })).toBeInTheDocument()
  })

  it('renders appearance settings by default', () => {
    renderWithProviders(<SettingsPage />)

    expect(screen.getByText('外观设置')).toBeInTheDocument()
    expect(screen.getByText('自定义应用的外观和主题')).toBeInTheDocument()

    // Check theme options
    expect(screen.getByText('浅色')).toBeInTheDocument()
    expect(screen.getByText('深色')).toBeInTheDocument()
    expect(screen.getByText('跟随系统')).toBeInTheDocument()
  })

  it('renders theme selection buttons', () => {
    renderWithProviders(<SettingsPage />)

    // Theme buttons should be clickable
    const lightButton = screen.getByText('浅色').closest('button')
    const darkButton = screen.getByText('深色').closest('button')
    const systemButton = screen.getByText('跟随系统').closest('button')

    expect(lightButton).toBeInTheDocument()
    expect(darkButton).toBeInTheDocument()
    expect(systemButton).toBeInTheDocument()
  })

  it('renders language selection', () => {
    renderWithProviders(<SettingsPage />)

    expect(screen.getByText('语言')).toBeInTheDocument()
    expect(screen.getByText('更多语言支持即将推出')).toBeInTheDocument()
  })
})
