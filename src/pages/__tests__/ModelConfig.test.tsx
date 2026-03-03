import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { ModelConfigPage } from '../ModelConfig'
import { render } from '@/test/utils'
import * as tauriApi from '@/lib/tauri-api'
import type { ModelConfigFull } from '@/types'

// Mock Tauri API - invokeWithRetry returns data directly, not ApiResponse
vi.mock('@/lib/tauri-api', () => ({
  modelApi: {
    getAllModelsFull: vi.fn(),
    saveModelFull: vi.fn(),
    deleteModel: vi.fn(),
    setDefaultModel: vi.fn(),
    testModelConnection: vi.fn(),
    reorderModels: vi.fn(),
  },
  secureStorageApi: {
    saveApiKey: vi.fn(),
    getApiKey: vi.fn(),
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
    setDefaultModel: vi.fn(),
  }),
}))

// Mock DnD Kit
vi.mock('@dnd-kit/core', () => ({
  DndContext: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  closestCenter: vi.fn(),
  KeyboardSensor: vi.fn(),
  PointerSensor: vi.fn(),
  useSensor: vi.fn(() => ({})),
  useSensors: vi.fn(() => ({})),
}))

vi.mock('@dnd-kit/sortable', () => ({
  SortableContext: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  useSortable: vi.fn(() => ({
    attributes: {},
    listeners: {},
    setNodeRef: vi.fn(),
    transform: null,
    transition: null,
    isDragging: false,
  })),
  arrayMove: vi.fn((items) => items),
  verticalListSortingStrategy: {},
  sortableKeyboardCoordinates: {},
}))

vi.mock('@dnd-kit/utilities', () => ({
  CSS: {
    Transform: {
      toString: vi.fn(),
    },
  },
}))

const mockModels: ModelConfigFull[] = [
  {
    id: 'model-1',
    name: 'GPT-4',
    provider: 'openai',
    model: 'gpt-4',
    api_base: 'https://api.openai.com/v1',
    priority: 0,
    parameters: {
      temperature: 1.0,
      max_tokens: 2048,
      top_p: 1.0,
      presence_penalty: 0.0,
      frequency_penalty: 0.0,
    },
    capabilities: {
      function_calling: true,
      vision: false,
      streaming: true,
      json_mode: false,
    },
    enabled: true,
    default: true,
  },
  {
    id: 'model-2',
    name: 'Claude 3',
    provider: 'anthropic',
    model: 'claude-3-opus-20240229',
    api_base: '',
    priority: 1,
    parameters: {
      temperature: 0.7,
      max_tokens: 4096,
      top_p: 1.0,
      presence_penalty: 0.0,
      frequency_penalty: 0.0,
    },
    capabilities: {
      function_calling: true,
      vision: true,
      streaming: true,
      json_mode: true,
    },
    enabled: true,
    default: false,
  },
]

describe('ModelConfig', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // invokeWithRetry returns data directly
    vi.mocked(tauriApi.modelApi.getAllModelsFull).mockResolvedValue(mockModels)
    vi.mocked(tauriApi.secureStorageApi.getApiKey).mockResolvedValue('')
  })

  describe('页面渲染', () => {
    it('should render without crashing', async () => {
      render(<ModelConfigPage />)

      await waitFor(() => {
        expect(screen.getByText('模型配置')).toBeInTheDocument()
      })
    })

    it('should display page title and description', async () => {
      render(<ModelConfigPage />)

      await waitFor(() => {
        expect(screen.getByText('模型配置')).toBeInTheDocument()
        expect(screen.getByText('管理 AI 模型提供商和 API 密钥')).toBeInTheDocument()
      })
    })

    it('should display add model button', async () => {
      render(<ModelConfigPage />)

      await waitFor(() => {
        expect(screen.getByText('添加模型')).toBeInTheDocument()
      })
    })

    it('should display security notice', async () => {
      render(<ModelConfigPage />)

      await waitFor(() => {
        expect(screen.getByText(/API Key 将安全存储/i)).toBeInTheDocument()
      })
    })
  })

  describe('模型列表显示', () => {
    it('should fetch models on mount', async () => {
      render(<ModelConfigPage />)

      await waitFor(() => {
        expect(tauriApi.modelApi.getAllModelsFull).toHaveBeenCalled()
      })
    })

    it('should display empty state when no models', async () => {
      vi.mocked(tauriApi.modelApi.getAllModelsFull).mockResolvedValue([])

      render(<ModelConfigPage />)

      await waitFor(() => {
        expect(screen.getByText('尚未配置任何模型')).toBeInTheDocument()
      })
    })
  })

  describe('添加模型对话框', () => {
    it('should open add model dialog when clicking add button', async () => {
      render(<ModelConfigPage />)

      await waitFor(() => {
        expect(screen.getByText('添加模型')).toBeInTheDocument()
      })

      const addButton = screen.getByText('添加模型')
      addButton.click()

      await waitFor(() => {
        expect(screen.getByText('基本信息')).toBeInTheDocument()
      })
    })
  })

  describe('错误处理', () => {
    it('should handle API errors gracefully without crashing', async () => {
      vi.mocked(tauriApi.modelApi.getAllModelsFull).mockRejectedValue(
        new Error('Failed to load models')
      )

      render(<ModelConfigPage />)

      await waitFor(() => {
        expect(screen.getByText('模型配置')).toBeInTheDocument()
      })
    })

    it('should handle network errors', async () => {
      vi.mocked(tauriApi.modelApi.getAllModelsFull).mockRejectedValue(new Error('Network error'))

      render(<ModelConfigPage />)

      await waitFor(() => {
        expect(screen.getByText('模型配置')).toBeInTheDocument()
      })
    })
  })
})
