import { describe, it, expect, beforeEach } from 'vitest'
import { useConfigStore } from '../configStore'
import { mockModel, mockAgent } from '@/test/mocks/models'

describe('configStore', () => {
  beforeEach(() => {
    useConfigStore.setState({
      models: [],
      agents: [],
      currentAgentId: null,
      apiKeyCache: {},
      services: [],
      gatewayStatus: 'stopped',
    })
  })

  describe('models', () => {
    it('should add model', () => {
      const { addModel } = useConfigStore.getState()
      addModel(mockModel)
      
      const { models } = useConfigStore.getState()
      expect(models).toHaveLength(1)
      expect(models[0].name).toBe('GPT-4')
      expect(models[0].provider).toBe('openai')
    })

    it('should update model', () => {
      const { addModel, updateModel } = useConfigStore.getState()
      addModel(mockModel)
      
      updateModel(mockModel.id, { name: 'GPT-4 Turbo' })
      
      const { models } = useConfigStore.getState()
      expect(models[0].name).toBe('GPT-4 Turbo')
    })

    it('should remove model', () => {
      const { addModel, removeModel } = useConfigStore.getState()
      addModel(mockModel)
      
      removeModel(mockModel.id)
      
      const { models } = useConfigStore.getState()
      expect(models).toHaveLength(0)
    })

    it('should set default model', () => {
      const { addModel, setDefaultModel } = useConfigStore.getState()
      addModel({ ...mockModel, id: '1', isDefault: false })
      addModel({ ...mockModel, id: '2', isDefault: false })
      
      setDefaultModel('2')
      
      const { models } = useConfigStore.getState()
      expect(models[1].isDefault).toBe(true)
      expect(models[0].isDefault).toBe(false)
    })

    it('should reorder models', () => {
      const { addModel, reorderModels } = useConfigStore.getState()
      addModel({ ...mockModel, id: '1', name: 'Model 1' })
      addModel({ ...mockModel, id: '2', name: 'Model 2' })
      addModel({ ...mockModel, id: '3', name: 'Model 3' })
      
      reorderModels(['3', '1', '2'])
      
      const { models } = useConfigStore.getState()
      expect(models[0].id).toBe('3')
      expect(models[1].id).toBe('1')
      expect(models[2].id).toBe('2')
    })
  })

  describe('agents', () => {
    it('should add agent', () => {
      const { addAgent } = useConfigStore.getState()
      addAgent(mockAgent)
      
      const { agents } = useConfigStore.getState()
      expect(agents).toHaveLength(1)
      expect(agents[0].name).toBe('测试 Agent')
    })

    it('should update agent', () => {
      const { addAgent, updateAgent } = useConfigStore.getState()
      addAgent(mockAgent)
      
      updateAgent(mockAgent.id, { name: 'Updated Agent' })
      
      const { agents } = useConfigStore.getState()
      expect(agents[0].name).toBe('Updated Agent')
    })

    it('should remove agent', () => {
      const { addAgent, removeAgent } = useConfigStore.getState()
      addAgent(mockAgent)
      
      removeAgent(mockAgent.id)
      
      const { agents } = useConfigStore.getState()
      expect(agents).toHaveLength(0)
    })

    it('should set current agent', () => {
      const { addAgent, setCurrentAgent } = useConfigStore.getState()
      addAgent(mockAgent)
      
      setCurrentAgent(mockAgent.id)
      
      const { currentAgentId } = useConfigStore.getState()
      expect(currentAgentId).toBe(mockAgent.id)
    })

    it('should reset currentAgentId when removing current agent', () => {
      const { addAgent, setCurrentAgent, removeAgent } = useConfigStore.getState()
      addAgent(mockAgent)
      setCurrentAgent(mockAgent.id)
      
      removeAgent(mockAgent.id)
      
      const { currentAgentId } = useConfigStore.getState()
      expect(currentAgentId).toBeNull()
    })
  })

  describe('apiKeyCache', () => {
    it('should set api key', () => {
      const { setApiKey } = useConfigStore.getState()
      
      setApiKey('openai', 'sk-test-key')
      
      const { apiKeyCache } = useConfigStore.getState()
      expect(apiKeyCache['openai']).toBe('sk-test-key')
    })

    it('should remove api key', () => {
      const { setApiKey, removeApiKey } = useConfigStore.getState()
      setApiKey('openai', 'sk-test-key')
      
      removeApiKey('openai')
      
      const { apiKeyCache } = useConfigStore.getState()
      expect(apiKeyCache['openai']).toBeUndefined()
    })

    it('should clear all api keys', () => {
      const { setApiKey, clearApiKeyCache } = useConfigStore.getState()
      setApiKey('openai', 'sk-test-key-1')
      setApiKey('anthropic', 'sk-test-key-2')
      
      clearApiKeyCache()
      
      const { apiKeyCache } = useConfigStore.getState()
      expect(Object.keys(apiKeyCache)).toHaveLength(0)
    })
  })

  describe('services', () => {
    it('should set services', () => {
      const { setServices } = useConfigStore.getState()
      const services = [{ name: 'gateway', status: 'running' as const }]
      
      setServices(services)
      
      const { services: storedServices } = useConfigStore.getState()
      expect(storedServices).toHaveLength(1)
      expect(storedServices[0].name).toBe('gateway')
    })

    it('should update service', () => {
      const { setServices, updateService } = useConfigStore.getState()
      setServices([{ name: 'gateway', status: 'stopped' as const }])
      
      updateService('gateway', { status: 'running' })
      
      const { services } = useConfigStore.getState()
      expect(services[0].status).toBe('running')
    })
  })

  describe('gateway status', () => {
    it('should set gateway status', () => {
      const { setGatewayStatus } = useConfigStore.getState()
      
      setGatewayStatus('running')
      
      const { gatewayStatus } = useConfigStore.getState()
      expect(gatewayStatus).toBe('running')
    })
  })
})
