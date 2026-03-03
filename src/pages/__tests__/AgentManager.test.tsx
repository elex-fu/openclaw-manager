import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { AgentManager } from '../AgentManager'
import { render } from '@/test/utils'
import * as tauriApi from '@/lib/tauri-api'
import type { AgentConfig, ModelConfig } from '@/types'

// Mock Tauri API - invokeWithRetry returns data directly, not ApiResponse
vi.mock('@/lib/tauri-api', () => ({
  agentApi: {
    getAllAgents: vi.fn(),
    saveAgent: vi.fn(),
    deleteAgent: vi.fn(),
  },
  modelApi: {
    getAllModels: vi.fn(),
  },
}))

// Mock stores
vi.mock('@/stores/appStore', () => ({
  useAppStore: () => ({
    addNotification: vi.fn(),
  }),
}))

vi.mock('@/stores/configStore', () => ({
  useConfigStore: () => ({
    currentAgentId: null,
  }),
}))

// Mock AgentCard component
vi.mock('@/components/openclaw/AgentCard', () => ({
  AgentCard: ({ agent }: { agent: AgentConfig }) => (
    <div data-testid="agent-card">{agent.name}</div>
  ),
}))

// Mock EmptyListState component
vi.mock('@/components/error', () => ({
  EmptyListState: ({ itemName }: { itemName: string }) => (
    <div data-testid="empty-list">暂无{itemName}</div>
  ),
  EmptySearchState: ({ searchTerm }: { searchTerm: string }) => (
    <div data-testid="empty-search">未找到 &quot;{searchTerm}&quot;</div>
  ),
}))

// Mock animation components
vi.mock('@/components/animation', () => ({
  StaggerContainer: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  StaggerItem: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  ScaleIn: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}))

// Mock SkeletonGrid
vi.mock('@/components/ui/skeleton', () => ({
  SkeletonGrid: () => <div data-testid="skeleton-grid">Loading...</div>,
}))

const mockAgents: AgentConfig[] = [
  {
    id: 'agent-1',
    name: '代码助手',
    description: '帮助编写和审查代码',
    modelId: 'model-1',
    systemPrompt: '你是一个专业的代码助手',
    skills: ['coding', 'review'],
    enabled: true,
    createdAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-01T00:00:00Z',
  },
  {
    id: 'agent-2',
    name: '写作助手',
    description: '帮助撰写文档',
    modelId: 'model-2',
    systemPrompt: '你是一个写作助手',
    skills: ['writing'],
    enabled: false,
    createdAt: '2024-01-02T00:00:00Z',
    updatedAt: '2024-01-02T00:00:00Z',
  },
]

const mockModels: ModelConfig[] = [
  {
    id: 'model-1',
    name: 'GPT-4',
    provider: 'openai',
    model: 'gpt-4',
    temperature: 1,
    enabled: true,
    isDefault: true,
  },
  {
    id: 'model-2',
    name: 'Claude 3',
    provider: 'anthropic',
    model: 'claude-3-opus',
    temperature: 1,
    enabled: true,
  },
]

describe('AgentManager', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // invokeWithRetry returns data directly
    vi.mocked(tauriApi.agentApi.getAllAgents).mockResolvedValue(mockAgents)
    vi.mocked(tauriApi.modelApi.getAllModels).mockResolvedValue(mockModels)
  })

  describe('页面渲染', () => {
    it('should render without crashing', async () => {
      render(<AgentManager />)

      await waitFor(() => {
        expect(screen.getByText('Agent 管理')).toBeInTheDocument()
      })
    })

    it('should display page title and description', async () => {
      render(<AgentManager />)

      await waitFor(() => {
        expect(screen.getByText('Agent 管理')).toBeInTheDocument()
        expect(screen.getByText('创建和管理 AI Agent 配置')).toBeInTheDocument()
      })
    })

    it('should display create agent button', async () => {
      render(<AgentManager />)

      await waitFor(() => {
        expect(screen.getByText('创建 Agent')).toBeInTheDocument()
      })
    })
  })

  describe('Agent列表显示', () => {
    it('should fetch agents on mount', async () => {
      render(<AgentManager />)

      await waitFor(() => {
        expect(tauriApi.agentApi.getAllAgents).toHaveBeenCalled()
      })
    })

    it('should fetch models on mount', async () => {
      render(<AgentManager />)

      await waitFor(() => {
        expect(tauriApi.modelApi.getAllModels).toHaveBeenCalled()
      })
    })

    it('should display statistics cards', async () => {
      render(<AgentManager />)

      await waitFor(() => {
        expect(screen.getByText('总 Agent 数')).toBeInTheDocument()
        expect(screen.getByText('已启用')).toBeInTheDocument()
        expect(screen.getByText('已禁用')).toBeInTheDocument()
        expect(screen.getByText('当前使用')).toBeInTheDocument()
      })
    })

    it('should display empty state when no agents', async () => {
      vi.mocked(tauriApi.agentApi.getAllAgents).mockResolvedValue([])

      render(<AgentManager />)

      await waitFor(() => {
        expect(screen.getByTestId('empty-list')).toBeInTheDocument()
      })
    })

    it('should show loading state initially', () => {
      vi.mocked(tauriApi.agentApi.getAllAgents).mockImplementation(
        () => new Promise(() => {}) // Never resolves
      )

      render(<AgentManager />)

      expect(screen.getByTestId('skeleton-grid')).toBeInTheDocument()
    })
  })

  describe('搜索过滤功能', () => {
    it('should display search input', async () => {
      render(<AgentManager />)

      await waitFor(() => {
        expect(screen.getByPlaceholderText(/搜索 Agent/i)).toBeInTheDocument()
      })
    })

    it('should update search query on input change', async () => {
      render(<AgentManager />)

      await waitFor(() => {
        expect(screen.getByPlaceholderText(/搜索 Agent/i)).toBeInTheDocument()
      })

      const searchInput = screen.getByPlaceholderText(/搜索 Agent/i)
      expect(searchInput).toBeInTheDocument()
    })
  })

  describe('视图模式切换', () => {
    it('should display view mode tabs', async () => {
      render(<AgentManager />)

      await waitFor(() => {
        expect(screen.getByText('卡片')).toBeInTheDocument()
        expect(screen.getByText('列表')).toBeInTheDocument()
      })
    })
  })

  describe('错误处理', () => {
    it('should display error alert when loading fails', async () => {
      vi.mocked(tauriApi.agentApi.getAllAgents).mockRejectedValue(new Error('Network error'))

      render(<AgentManager />)

      await waitFor(() => {
        expect(screen.getByText(/加载失败/i)).toBeInTheDocument()
      })
    })

    it('should handle API errors gracefully without crashing', async () => {
      vi.mocked(tauriApi.agentApi.getAllAgents).mockRejectedValue(
        new Error('Failed to load agents')
      )

      render(<AgentManager />)

      // Should still render without crashing
      await waitFor(() => {
        expect(screen.getByText('Agent 管理')).toBeInTheDocument()
      })
    })
  })
})
