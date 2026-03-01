import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { 
  OpenClawConfig, 
  ModelConfig, 
  AgentConfig, 
  ServiceInfo,
  SystemSettings 
} from '@/types';

interface ConfigState {
  // OpenClaw 配置
  config: OpenClawConfig | null;
  setConfig: (config: OpenClawConfig | null) => void;
  updateConfig: (updates: Partial<OpenClawConfig>) => void;

  // 模型配置
  models: ModelConfig[];
  setModels: (models: ModelConfig[]) => void;
  addModel: (model: ModelConfig) => void;
  updateModel: (id: string, updates: Partial<ModelConfig>) => void;
  removeModel: (id: string) => void;
  setDefaultModel: (id: string) => void;
  reorderModels: (modelIds: string[]) => void;

  // Agent 配置
  agents: AgentConfig[];
  setAgents: (agents: AgentConfig[]) => void;
  addAgent: (agent: AgentConfig) => void;
  updateAgent: (id: string, updates: Partial<AgentConfig>) => void;
  removeAgent: (id: string) => void;
  setCurrentAgent: (id: string | null) => void;
  currentAgentId: string | null;

  // 服务状态
  services: ServiceInfo[];
  setServices: (services: ServiceInfo[]) => void;
  updateService: (name: string, updates: Partial<ServiceInfo>) => void;

  // Gateway 状态
  gatewayStatus: 'stopped' | 'starting' | 'running' | 'stopping' | 'error';
  setGatewayStatus: (status: ConfigState['gatewayStatus']) => void;

  // 系统设置
  settings: SystemSettings;
  setSettings: (settings: SystemSettings) => void;
  updateSettings: (updates: Partial<SystemSettings>) => void;

  // 配置加载状态
  isConfigLoaded: boolean;
  setConfigLoaded: (loaded: boolean) => void;

  // API Keys 缓存（内存中，不持久化）
  apiKeyCache: Record<string, string>;
  setApiKey: (providerId: string, key: string) => void;
  removeApiKey: (providerId: string) => void;
  clearApiKeyCache: () => void;
}

const defaultSettings: SystemSettings = {
  log_level: 'info',
  auto_update: true,
  theme: 'system',
  language: 'zh-CN',
  custom_vars: {},
};

export const useConfigStore = create<ConfigState>()(
  persist(
    (set, get) => ({
      // 初始状态
      config: null,
      models: [],
      agents: [],
      currentAgentId: null,
      services: [],
      gatewayStatus: 'stopped',
      settings: defaultSettings,
      isConfigLoaded: false,
      apiKeyCache: {},

      // OpenClaw 配置
      setConfig: (config) => set({ config, isConfigLoaded: true }),
      updateConfig: (updates) =>
        set((state) => ({
          config: state.config ? { ...state.config, ...updates } : null,
        })),

      // 模型管理
      setModels: (models) => set({ models }),
      addModel: (model) =>
        set((state) => ({
          models: [...state.models, model],
        })),
      updateModel: (id, updates) =>
        set((state) => ({
          models: state.models.map((m) =>
            m.id === id ? { ...m, ...updates } : m
          ),
        })),
      removeModel: (id) =>
        set((state) => ({
          models: state.models.filter((m) => m.id !== id),
        })),
      setDefaultModel: (id) =>
        set((state) => ({
          models: state.models.map((m) => ({
            ...m,
            isDefault: m.id === id,
          })),
        })),
      reorderModels: (modelIds) =>
        set((state) => {
          const modelMap = new Map(state.models.map((m) => [m.id, m]));
          return {
            models: modelIds
              .map((id) => modelMap.get(id))
              .filter((m): m is ModelConfig => m !== undefined),
          };
        }),

      // Agent 管理
      setAgents: (agents) => set({ agents }),
      addAgent: (agent) =>
        set((state) => ({
          agents: [...state.agents, agent],
        })),
      updateAgent: (id, updates) =>
        set((state) => ({
          agents: state.agents.map((a) =>
            a.id === id ? { ...a, ...updates } : a
          ),
        })),
      removeAgent: (id) =>
        set((state) => ({
          agents: state.agents.filter((a) => a.id !== id),
          currentAgentId: state.currentAgentId === id ? null : state.currentAgentId,
        })),
      setCurrentAgent: (id) => set({ currentAgentId: id }),

      // 服务状态
      setServices: (services) => set({ services }),
      updateService: (name, updates) =>
        set((state) => ({
          services: state.services.map((s) =>
            s.name === name ? { ...s, ...updates } : s
          ),
        })),

      // Gateway 状态
      setGatewayStatus: (gatewayStatus) => set({ gatewayStatus }),

      // 系统设置
      setSettings: (settings) => set({ settings }),
      updateSettings: (updates) =>
        set((state) => ({
          settings: { ...state.settings, ...updates },
        })),

      // 配置加载状态
      setConfigLoaded: (isConfigLoaded) => set({ isConfigLoaded }),

      // API Key 缓存（仅内存，不持久化）
      setApiKey: (providerId, key) =>
        set((state) => ({
          apiKeyCache: { ...state.apiKeyCache, [providerId]: key },
        })),
      removeApiKey: (providerId) =>
        set((state) => {
          const { [providerId]: _, ...rest } = state.apiKeyCache;
          return { apiKeyCache: rest };
        }),
      clearApiKeyCache: () => set({ apiKeyCache: {} }),
    }),
    {
      name: 'config-storage',
      partialize: (state) => ({
        models: state.models,
        agents: state.agents,
        currentAgentId: state.currentAgentId,
        settings: state.settings,
        // 注意：apiKeyCache 不会被持久化
      }),
    }
  )
);
