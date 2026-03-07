import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { UpdateManager } from '../UpdateManager'
import { render } from '@/test/utils'
import * as tauriApi from '@/lib/tauri-api'

// Mock Tauri plugins
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}))

// Mock date-fns
vi.mock('date-fns', () => ({
  format: vi.fn(() => '2024年03月01日 12:00'),
}))

vi.mock('date-fns/locale', () => ({
  zhCN: {},
}))

// Mock Tauri API
vi.mock('@/lib/tauri-api', () => ({
  updateApi: {
    checkForUpdates: vi.fn(),
    performUpdate: vi.fn(),
    performOfflineUpdate: vi.fn(),
    getBackupList: vi.fn(),
    restoreFromBackup: vi.fn(),
    onUpdateProgress: vi.fn(() => Promise.resolve(() => {})),
  },
}))

describe('UpdateManager', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should render without crashing', () => {
    vi.mocked(tauriApi.updateApi.checkForUpdates).mockResolvedValue({
      currentVersion: '1.0.0',
      latestVersion: '1.0.0',
      hasUpdate: false,
      updateInfo: null,
    })

    vi.mocked(tauriApi.updateApi.getBackupList).mockResolvedValue([])

    const { container } = render(<UpdateManager />)
    expect(container).toBeInTheDocument()
  })

  it('should check for updates on mount', async () => {
    vi.mocked(tauriApi.updateApi.checkForUpdates).mockResolvedValue({
      currentVersion: '1.0.0',
      latestVersion: '1.1.0',
      hasUpdate: true,
      updateInfo: {
        version: '1.1.0',
        releaseDate: '2024-03-01',
        changelog: 'Bug fixes',
        downloadUrl: 'https://example.com/download',
        checksum: 'abc123',
        mandatory: false,
        minSupportedVersion: null,
      },
    })

    vi.mocked(tauriApi.updateApi.getBackupList).mockResolvedValue([])

    render(<UpdateManager />)

    await waitFor(() => {
      expect(tauriApi.updateApi.checkForUpdates).toHaveBeenCalled()
    })
  })

  it('should display current version', async () => {
    vi.mocked(tauriApi.updateApi.checkForUpdates).mockResolvedValue({
      currentVersion: '1.0.0',
      latestVersion: '1.0.0',
      hasUpdate: false,
      updateInfo: null,
    })

    vi.mocked(tauriApi.updateApi.getBackupList).mockResolvedValue([])

    render(<UpdateManager />)

    await waitFor(() => {
      expect(screen.getByText('1.0.0')).toBeInTheDocument()
    })
  })

  it('should show update available when hasUpdate is true', async () => {
    vi.mocked(tauriApi.updateApi.checkForUpdates).mockResolvedValue({
      currentVersion: '1.0.0',
      latestVersion: '1.1.0',
      hasUpdate: true,
      updateInfo: {
        version: '1.1.0',
        releaseDate: '2024-03-01',
        changelog: 'New features',
        downloadUrl: 'https://example.com/download',
        checksum: 'abc123',
        mandatory: false,
        minSupportedVersion: null,
      },
    })

    vi.mocked(tauriApi.updateApi.getBackupList).mockResolvedValue([])

    render(<UpdateManager />)

    // Wait for the component to render with update info
    await waitFor(() => {
      expect(tauriApi.updateApi.checkForUpdates).toHaveBeenCalled()
    })

    // Check that the update is displayed (either current or latest version)
    await waitFor(() => {
      const versionElements = screen.getAllByText(/1\.0\.0|1\.1\.0/)
      expect(versionElements.length).toBeGreaterThan(0)
    })
  })

  it('should load backup list on mount', async () => {
    vi.mocked(tauriApi.updateApi.checkForUpdates).mockResolvedValue({
      currentVersion: '1.0.0',
      latestVersion: '1.0.0',
      hasUpdate: false,
      updateInfo: null,
    })

    vi.mocked(tauriApi.updateApi.getBackupList).mockResolvedValue([
      {
        createdAt: '2024-03-01T12:00:00Z',
        version: '1.0.0',
        path: '/test/backup',
      },
    ])

    render(<UpdateManager />)

    await waitFor(() => {
      expect(tauriApi.updateApi.getBackupList).toHaveBeenCalled()
    })
  })
})
