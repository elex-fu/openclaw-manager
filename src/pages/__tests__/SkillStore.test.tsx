import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { SkillStore } from '../SkillStore'
import * as tauriApi from '@/lib/tauri-api'
import type { Skill, InstalledSkill, SkillCategory, SkillSearchResult } from '@/types'

// Mock tauri-api
vi.mock('@/lib/tauri-api', () => ({
  skillApi: {
    getAll: vi.fn(),
    getCategories: vi.fn(),
    searchMarket: vi.fn(),
    getPopular: vi.fn(),
    install: vi.fn(),
    uninstall: vi.fn(),
    toggle: vi.fn(),
    updateConfig: vi.fn(),
  },
}))

// Mock stores
vi.mock('@/stores/appStore', () => ({
  useAppStore: () => ({
    addNotification: vi.fn(),
  }),
}))

const mockCategories: SkillCategory[] = [
  { id: 'all', name: '全部', sort_order: 0 },
  { id: 'programming', name: '编程开发', sort_order: 1 },
  { id: 'writing', name: '写作助手', sort_order: 2 },
]

const mockSkills: Skill[] = [
  {
    id: 'code-assistant',
    name: '代码助手',
    description: '智能代码补全',
    author: 'OpenClaw',
    version: '1.0.0',
    categories: ['programming'],
    tags: ['code', 'development'],
    rating: 4.5,
    downloads: 1000,
    hooks: [],
    dependencies: [],
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
  {
    id: 'writing-assistant',
    name: '写作助手',
    description: '智能写作辅助',
    author: 'OpenClaw',
    version: '1.2.0',
    categories: ['writing'],
    tags: ['writing', 'translation'],
    rating: 4.8,
    downloads: 2000,
    hooks: [],
    dependencies: [],
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
]

const mockInstalledSkills: InstalledSkill[] = [
  {
    ...mockSkills[0],
    is_enabled: true,
    config: {},
    installed_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    has_update: false,
  },
]

const mockSearchResult: SkillSearchResult = {
  skills: mockSkills,
  total: 2,
  page: 1,
  per_page: 20,
}

const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  })

const renderWithQueryClient = (ui: React.ReactElement) => {
  const queryClient = createTestQueryClient()
  return render(
    <QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>
  )
}

describe('SkillStore', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(tauriApi.skillApi.getCategories).mockResolvedValue(mockCategories)
    vi.mocked(tauriApi.skillApi.searchMarket).mockResolvedValue(mockSearchResult)
    vi.mocked(tauriApi.skillApi.getAll).mockResolvedValue(mockInstalledSkills)
    vi.mocked(tauriApi.skillApi.getPopular).mockResolvedValue([])
  })

  it('renders skill store page', async () => {
    renderWithQueryClient(<SkillStore />)

    await waitFor(() => {
      expect(screen.getByText('技能商店')).toBeInTheDocument()
    })
  })

  it('displays skill categories', async () => {
    renderWithQueryClient(<SkillStore />)

    await waitFor(() => {
      expect(screen.getByText('全部')).toBeInTheDocument()
      expect(screen.getByText('编程开发')).toBeInTheDocument()
      expect(screen.getByText('写作助手')).toBeInTheDocument()
    })
  })

  it('displays skill cards', async () => {
    renderWithQueryClient(<SkillStore />)

    await waitFor(() => {
      expect(screen.getByText('代码助手')).toBeInTheDocument()
      expect(screen.getByText('写作助手')).toBeInTheDocument()
    })
  })

  it('allows searching skills', async () => {
    renderWithQueryClient(<SkillStore />)

    await waitFor(() => {
      expect(screen.getByPlaceholderText('搜索技能名称、描述或标签...')).toBeInTheDocument()
    })

    const searchInput = screen.getByPlaceholderText('搜索技能名称、描述或标签...')
    fireEvent.change(searchInput, { target: { value: '代码' } })

    await waitFor(() => {
      expect(tauriApi.skillApi.searchMarket).toHaveBeenCalledWith(
        expect.objectContaining({ query: '代码' })
      )
    })
  })

  it('switches between market and installed tabs', async () => {
    renderWithQueryClient(<SkillStore />)

    await waitFor(() => {
      expect(screen.getByText('技能市场')).toBeInTheDocument()
      expect(screen.getByText('已安装')).toBeInTheDocument()
    })

    const installedTab = screen.getByText('已安装')
    fireEvent.click(installedTab)

    await waitFor(() => {
      expect(screen.getByText('代码助手')).toBeInTheDocument()
    })
  })

  it('displays installed skills count', async () => {
    renderWithQueryClient(<SkillStore />)

    await waitFor(() => {
      expect(screen.getByText('1')).toBeInTheDocument()
    })
  })
})

describe('SkillCard', () => {
  it('renders skill information correctly', () => {
    const { SkillCard } = require('@/components/skill/SkillCard')

    render(<SkillCard skill={mockSkills[0]} />)

    expect(screen.getByText('代码助手')).toBeInTheDocument()
    expect(screen.getByText('智能代码补全')).toBeInTheDocument()
    expect(screen.getByText('OpenClaw')).toBeInTheDocument()
  })

  it('shows installed status when skill is installed', () => {
    const { SkillCard } = require('@/components/skill/SkillCard')

    render(<SkillCard skill={mockSkills[0]} isInstalled={true} isEnabled={true} />)

    expect(screen.getByText('已启用')).toBeInTheDocument()
  })

  it('shows install button when skill is not installed', () => {
    const { SkillCard } = require('@/components/skill/SkillCard')

    render(<SkillCard skill={mockSkills[0]} isInstalled={false} />)

    expect(screen.getByText('安装')).toBeInTheDocument()
  })
})

describe('InstalledSkillList', () => {
  it('renders empty state when no skills installed', () => {
    const { InstalledSkillList } = require('@/components/skill/InstalledSkillList')

    render(<InstalledSkillList skills={[]} onToggle={vi.fn()} onConfig={vi.fn()} onUninstall={vi.fn()} />)

    expect(screen.getByText('暂无已安装技能')).toBeInTheDocument()
  })

  it('renders installed skills', () => {
    const { InstalledSkillList } = require('@/components/skill/InstalledSkillList')

    render(
      <InstalledSkillList
        skills={mockInstalledSkills}
        onToggle={vi.fn()}
        onConfig={vi.fn()}
        onUninstall={vi.fn()}
      />
    )

    expect(screen.getByText('代码助手')).toBeInTheDocument()
  })
})
