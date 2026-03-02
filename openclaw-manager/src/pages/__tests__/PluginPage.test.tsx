import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { PluginPage } from '../PluginPage'
import * as tauriApi from '@/lib/tauri-api'
import type { Plugin, MarketPlugin, PluginCategory, SearchPluginsResult } from '@/types'

// Mock Tauri API
vi.mock('@/lib/tauri-api', () => ({
  pluginApi: {
    getAll: vi.fn(),
    install: vi.fn(),
    uninstall: vi.fn(),
    enable: vi.fn(),
    disable: vi.fn(),
  },
  pluginMarketApi: {
    search: vi.fn(),
    getCategories: vi.fn(),
    getPopular: vi.fn(),
    getLatest: vi.fn(),
    getDetails: vi.fn(),
  },
}))

const mockInstalledPlugins: Plugin[] = [
  {
    id: 'plugin-1',
    name: 'Test Plugin 1',
    version: '1.0.0',
    description: 'A test plugin',
    author: 'Test Author',
    plugin_type: 'lua',
    entry_point: 'main.lua',
    is_enabled: true,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
  {
    id: 'plugin-2',
    name: 'Test Plugin 2',
    version: '2.0.0',
    description: 'Another test plugin',
    author: 'Another Author',
    plugin_type: 'js',
    entry_point: 'main.js',
    is_enabled: false,
    created_at: '2024-01-02T00:00:00Z',
    updated_at: '2024-01-02T00:00:00Z',
  },
]

const mockMarketPlugins: MarketPlugin[] = [
  {
    id: 'market-plugin-1',
    name: 'Market Plugin 1',
    version: '1.0.0',
    description: 'A market plugin',
    author: 'Market Author',
    downloads: 1000,
    rating: 4.5,
    rating_count: 100,
    download_url: 'https://example.com/plugin1',
    categories: ['productivity'],
    tags: ['ai', 'automation'],
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    size_bytes: 1024000,
  },
]

const mockCategories: PluginCategory[] = [
  { id: 'productivity', name: '生产力', plugin_count: 10 },
  { id: 'ai-tools', name: 'AI工具', plugin_count: 20 },
]

const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  })

function renderWithQuery(ui: React.ReactElement) {
  const queryClient = createTestQueryClient()
  return render(
    <QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>
  )
}

describe('PluginPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(tauriApi.pluginApi.getAll).mockResolvedValue({
      success: true,
      data: mockInstalledPlugins,
      error: null,
    })
    vi.mocked(tauriApi.pluginMarketApi.getCategories).mockResolvedValue({
      success: true,
      data: mockCategories,
      error: null,
    })
    vi.mocked(tauriApi.pluginMarketApi.getPopular).mockResolvedValue({
      success: true,
      data: mockMarketPlugins,
      error: null,
    })
    vi.mocked(tauriApi.pluginMarketApi.getLatest).mockResolvedValue({
      success: true,
      data: [],
      error: null,
    })
    vi.mocked(tauriApi.pluginMarketApi.search).mockResolvedValue({
      success: true,
      data: {
        plugins: mockMarketPlugins,
        total: 1,
        page: 1,
        per_page: 20,
        has_more: false,
      } as SearchPluginsResult,
      error: null,
    })
  })

  it('renders installed plugins tab by default', async () => {
    renderWithQuery(<PluginPage />)

    await waitFor(() => {
      expect(screen.getByText('Test Plugin 1')).toBeInTheDocument()
      expect(screen.getByText('Test Plugin 2')).toBeInTheDocument()
    })
  })

  it('displays correct plugin status badges', async () => {
    renderWithQuery(<PluginPage />)

    await waitFor(() => {
      expect(screen.getByText('已启用')).toBeInTheDocument()
      expect(screen.getByText('已禁用')).toBeInTheDocument()
    })
  })

  it('switches to market tab when clicked', async () => {
    renderWithQuery(<PluginPage />)

    const marketTab = screen.getByText('插件市场')
    fireEvent.click(marketTab)

    await waitFor(() => {
      expect(screen.getByPlaceholderText('搜索插件...')).toBeInTheDocument()
    })
  })

  it('shows empty state when no plugins installed', async () => {
    vi.mocked(tauriApi.pluginApi.getAll).mockResolvedValue({
      success: true,
      data: [],
      error: null,
    })

    renderWithQuery(<PluginPage />)

    await waitFor(() => {
      expect(screen.getByText('暂无已安装插件')).toBeInTheDocument()
      expect(screen.getByText('浏览插件市场')).toBeInTheDocument()
    })
  })

  it('calls enable API when enable button is clicked', async () => {
    vi.mocked(tauriApi.pluginApi.enable).mockResolvedValue({
      success: true,
      data: mockInstalledPlugins[1],
      error: null,
    })

    renderWithQuery(<PluginPage />)

    await waitFor(() => {
      expect(screen.getByText('Test Plugin 2')).toBeInTheDocument()
    })

    const enableButtons = screen.getAllByText('启用')
    fireEvent.click(enableButtons[0])

    await waitFor(() => {
      expect(tauriApi.pluginApi.enable).toHaveBeenCalledWith('plugin-2')
    })
  })

  it('calls disable API when disable button is clicked', async () => {
    vi.mocked(tauriApi.pluginApi.disable).mockResolvedValue({
      success: true,
      data: mockInstalledPlugins[0],
      error: null,
    })

    renderWithQuery(<PluginPage />)

    await waitFor(() => {
      expect(screen.getByText('Test Plugin 1')).toBeInTheDocument()
    })

    const disableButton = screen.getByText('禁用')
    fireEvent.click(disableButton)

    await waitFor(() => {
      expect(tauriApi.pluginApi.disable).toHaveBeenCalledWith('plugin-1')
    })
  })

  it('calls uninstall API when uninstall button is clicked', async () => {
    vi.mocked(tauriApi.pluginApi.uninstall).mockResolvedValue({
      success: true,
      data: true,
      error: null,
    })

    renderWithQuery(<PluginPage />)

    await waitFor(() => {
      expect(screen.getByText('Test Plugin 1')).toBeInTheDocument()
    })

    const uninstallButtons = screen.getAllByText('卸载')
    fireEvent.click(uninstallButtons[0])

    await waitFor(() => {
      expect(tauriApi.pluginApi.uninstall).toHaveBeenCalled()
    })
  })
})

describe('Plugin Market', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(tauriApi.pluginApi.getAll).mockResolvedValue({
      success: true,
      data: [],
      error: null,
    })
    vi.mocked(tauriApi.pluginMarketApi.getCategories).mockResolvedValue({
      success: true,
      data: mockCategories,
      error: null,
    })
    vi.mocked(tauriApi.pluginMarketApi.getPopular).mockResolvedValue({
      success: true,
      data: mockMarketPlugins,
      error: null,
    })
    vi.mocked(tauriApi.pluginMarketApi.getLatest).mockResolvedValue({
      success: true,
      data: [],
      error: null,
    })
    vi.mocked(tauriApi.pluginMarketApi.search).mockResolvedValue({
      success: true,
      data: {
        plugins: mockMarketPlugins,
        total: 1,
        page: 1,
        per_page: 20,
        has_more: false,
      } as SearchPluginsResult,
      error: null,
    })
  })

  it('displays popular plugins in market tab', async () => {
    renderWithQuery(<PluginPage />)

    const marketTab = screen.getByText('插件市场')
    fireEvent.click(marketTab)

    await waitFor(() => {
      expect(screen.getByText('热门插件')).toBeInTheDocument()
      expect(screen.getByText('Market Plugin 1')).toBeInTheDocument()
    })
  })

  it('searches plugins when typing in search box', async () => {
    renderWithQuery(<PluginPage />)

    const marketTab = screen.getByText('插件市场')
    fireEvent.click(marketTab)

    await waitFor(() => {
      expect(screen.getByPlaceholderText('搜索插件...')).toBeInTheDocument()
    })

    const searchInput = screen.getByPlaceholderText('搜索插件...')
    fireEvent.change(searchInput, { target: { value: 'AI' } })

    await waitFor(
      () => {
        expect(tauriApi.pluginMarketApi.search).toHaveBeenCalledWith(
          expect.objectContaining({
            query: 'AI',
          })
        )
      },
      { timeout: 3000 }
    )
  })

  it('displays plugin categories dropdown', async () => {
    renderWithQuery(<PluginPage />)

    const marketTab = screen.getByText('插件市场')
    fireEvent.click(marketTab)

    await waitFor(() => {
      expect(screen.getByText('选择分类')).toBeInTheDocument()
    })
  })
})
