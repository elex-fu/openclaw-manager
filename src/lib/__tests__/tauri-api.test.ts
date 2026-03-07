import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import {
  invokeWithRetry,
  invokeWithTimeout,
  ApiError,
  handleApiResponse,
  modelApi,
  serviceApi,
  secureStorageApi,
} from '../tauri-api'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'

const mockInvoke = vi.mocked(invoke)

describe('tauri-api', () => {
  beforeEach(() => {
    mockInvoke.mockClear()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('handleApiResponse', () => {
    it('should return data for successful response', () => {
      const response = { success: true, data: { id: '1', name: 'test' }, error: null }
      expect(handleApiResponse(response)).toEqual({ id: '1', name: 'test' })
    })

    it('should throw ApiError for failed response', () => {
      const response = { success: false, data: null, error: 'Something went wrong' }
      expect(() => handleApiResponse(response)).toThrow(ApiError)
      expect(() => handleApiResponse(response)).toThrow('Something went wrong')
    })

    it('should throw ApiError when data is null', () => {
      const response = { success: true, data: null, error: null }
      expect(() => handleApiResponse(response)).toThrow(ApiError)
      expect(() => handleApiResponse(response)).toThrow('Response data is null')
    })
  })

  describe('invokeWithTimeout', () => {
    it('should return raw response on successful invoke', async () => {
      const mockData = { success: true, data: 'result', error: null }
      mockInvoke.mockResolvedValueOnce(mockData)

      const result = await invokeWithTimeout('test_command', { arg: 'value' }, 5000)
      // invokeWithTimeout returns raw response, not processed data
      expect(result).toEqual(mockData)
      expect(mockInvoke).toHaveBeenCalledWith('test_command', { arg: 'value' })
    })

    it('should throw timeout error when invoke takes too long', async () => {
      mockInvoke.mockImplementationOnce(() => new Promise(resolve => setTimeout(resolve, 10000)))

      await expect(invokeWithTimeout('test_command', {}, 50)).rejects.toThrow('timeout')
    })
  })

  describe('invokeWithRetry', () => {
    it('should return data on first successful attempt', async () => {
      const mockData = { success: true, data: 'result', error: null }
      mockInvoke.mockResolvedValueOnce(mockData)

      const result = await invokeWithRetry('test_command', { arg: 'value' }, { maxRetries: 2 })
      expect(result).toBe('result')
      expect(mockInvoke).toHaveBeenCalledTimes(1)
    })

    it('should retry on failure and eventually succeed', async () => {
      mockInvoke
        .mockRejectedValueOnce(new Error('Network error'))
        .mockRejectedValueOnce(new Error('Network error'))
        .mockResolvedValueOnce({ success: true, data: 'result', error: null })

      const result = await invokeWithRetry('test_command', {}, { maxRetries: 3, baseDelay: 10 })
      expect(result).toBe('result')
      expect(mockInvoke).toHaveBeenCalledTimes(3)
    })

    it('should throw error after max retries exceeded', async () => {
      mockInvoke.mockRejectedValue(new Error('Network error'))

      await expect(invokeWithRetry('test_command', {}, { maxRetries: 2, baseDelay: 10 }))
        .rejects.toThrow('Network error')
      expect(mockInvoke).toHaveBeenCalledTimes(3) // initial + 2 retries
    })

    it('should use timeoutMs option when provided', async () => {
      const mockData = { success: true, data: 'result', error: null }
      mockInvoke.mockResolvedValue(mockData)

      const result = await invokeWithRetry('test_command', {}, { timeoutMs: 5000 })
      expect(result).toBe('result')
    })
  })

  describe('modelApi', () => {
    it('getAllModels should call correct command', async () => {
      const mockModels = [{ id: '1', name: 'GPT-4', provider: 'openai' }]
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockModels, error: null })

      const result = await modelApi.getAllModels()
      expect(mockInvoke).toHaveBeenCalledWith('get_all_models_full', undefined)
      expect(result).toEqual(mockModels)
    })

    it('saveModel should call correct command with model data', async () => {
      const model = { id: '1', name: 'GPT-4', provider: 'openai' }
      mockInvoke.mockResolvedValueOnce({ success: true, data: model, error: null })

      const result = await modelApi.saveModel(model as any)
      expect(mockInvoke).toHaveBeenCalledWith('save_model_full', { model })
      expect(result).toEqual(model)
    })

    it('deleteModel should call correct command with id', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true, data: true, error: null })

      const result = await modelApi.deleteModel('model-1')
      expect(mockInvoke).toHaveBeenCalledWith('delete_model', { id: 'model-1' })
      expect(result).toBe(true)
    })

    it('setDefaultModel should call correct command with id', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true, data: true, error: null })

      const result = await modelApi.setDefaultModel('model-1')
      expect(mockInvoke).toHaveBeenCalledWith('set_default_model', { id: 'model-1' })
      expect(result).toBe(true)
    })

    it('reorderModels should call correct command with modelIds', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true, data: true, error: null })

      const result = await modelApi.reorderModels(['model-2', 'model-1'])
      expect(mockInvoke).toHaveBeenCalledWith('reorder_models', { modelIds: ['model-2', 'model-1'] })
      expect(result).toBe(true)
    })

    it('testModelConnection should call correct command with timeout', async () => {
      const mockResult = { success: true, latency: 100 }
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockResult, error: null })

      const result = await modelApi.testModelConnection('model-1')
      expect(mockInvoke).toHaveBeenCalledWith('test_model_connection', { modelId: 'model-1' })
      expect(result).toEqual(mockResult)
    })
  })

  describe('serviceApi', () => {
    it('startService should call correct command', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true, data: true, error: null })

      const result = await serviceApi.startService()
      expect(mockInvoke).toHaveBeenCalledWith('start_openclaw_service', undefined)
      expect(result).toBe(true)
    })

    it('stopService should call correct command', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true, data: true, error: null })

      const result = await serviceApi.stopService()
      expect(mockInvoke).toHaveBeenCalledWith('stop_openclaw_service', undefined)
      expect(result).toBe(true)
    })

    it('getServiceStatus should call correct command', async () => {
      const mockStatus = { running: true, version: '1.0.0' }
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockStatus, error: null })

      const result = await serviceApi.getServiceStatus()
      expect(mockInvoke).toHaveBeenCalledWith('get_openclaw_service_status', undefined)
      expect(result).toEqual(mockStatus)
    })

    it('healthCheck should call correct command', async () => {
      const mockResult = { healthy: true, message: 'OK' }
      mockInvoke.mockResolvedValueOnce({ success: true, data: mockResult, error: null })

      const result = await serviceApi.healthCheck()
      expect(mockInvoke).toHaveBeenCalledWith('health_check_openclaw_service', undefined)
      expect(result).toEqual(mockResult)
    })
  })

  describe('secureStorageApi', () => {
    it('saveApiKey should call correct command', async () => {
      // void return type uses undefined as data
      mockInvoke.mockResolvedValueOnce({ success: true, data: undefined, error: null })

      await secureStorageApi.saveApiKey('openai', 'sk-test123')
      expect(mockInvoke).toHaveBeenCalledWith('save_api_key', { provider: 'openai', apiKey: 'sk-test123' })
    })

    it('getApiKey should call correct command', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true, data: 'sk-test123', error: null })

      const result = await secureStorageApi.getApiKey('openai')
      expect(mockInvoke).toHaveBeenCalledWith('get_api_key', { provider: 'openai' })
      expect(result).toBe('sk-test123')
    })

    it('deleteApiKey should call correct command', async () => {
      // void return type uses undefined as data
      mockInvoke.mockResolvedValueOnce({ success: true, data: undefined, error: null })

      await secureStorageApi.deleteApiKey('openai')
      expect(mockInvoke).toHaveBeenCalledWith('delete_api_key', { provider: 'openai' })
    })

    it('hasApiKey should call correct command', async () => {
      mockInvoke.mockResolvedValueOnce({ success: true, data: true, error: null })

      const result = await secureStorageApi.hasApiKey('openai')
      expect(mockInvoke).toHaveBeenCalledWith('has_api_key', { provider: 'openai' })
      expect(result).toBe(true)
    })
  })
})
