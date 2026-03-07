import { describe, it, expect, vi, beforeEach } from 'vitest'
import { waitFor } from '@testing-library/react'
import { Dashboard } from '../Dashboard'
import { render } from '@/test/utils'
import * as tauriApi from '@/lib/tauri-api'

// Mock Tauri API
vi.mock('@/lib/tauri-api', () => ({
  openclawApi: {
    checkInstallation: vi.fn(),
    installOneClick: vi.fn(),
    onInstallProgress: vi.fn(() => Promise.resolve(() => {})),
  },
  serviceApi: {
    getServiceStatus: vi.fn(),
  },
}))

describe('Dashboard', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should render without crashing', () => {
    vi.mocked(tauriApi.openclawApi.checkInstallation).mockResolvedValue({
      type: 'Installed',
      version: '1.0.0',
    })

    const { container } = render(<Dashboard />)
    expect(container).toBeInTheDocument()
  })

  it('should call checkInstallation on mount', async () => {
    vi.mocked(tauriApi.openclawApi.checkInstallation).mockResolvedValue({
      type: 'Installed',
      version: '1.0.0',
    })

    render(<Dashboard />)

    await waitFor(() => {
      expect(tauriApi.openclawApi.checkInstallation).toHaveBeenCalled()
    })
  })
})
