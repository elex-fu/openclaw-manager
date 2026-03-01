import type { ModelConfig, AgentConfig } from '@/types'

export const mockModel: ModelConfig = {
  id: 'test-model-1',
  name: 'GPT-4',
  provider: 'openai',
  model: 'gpt-4',
  temperature: 0.7,
  max_tokens: 4096,
  enabled: true,
  isDefault: true,
}

export const mockAgent: AgentConfig = {
  id: 'test-agent-1',
  name: '测试 Agent',
  description: '用于测试的 Agent',
  modelId: 'test-model-1',
  systemPrompt: 'You are a helpful assistant.',
  skills: [],
  enabled: true,
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
}
