import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import { 
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { 
  Plus, 
  Edit2, 
  Trash2, 
  Bot,
  Brain,
  CheckCircle,
  User,
  Sparkles
} from 'lucide-react'
import { agentApi } from '@/lib/tauri-api'
import { useConfigStore } from '@/stores/configStore'
import { useAppStore } from '@/stores/appStore'
import type { AgentConfig } from '@/types'

export function AgentManager() {
  const queryClient = useQueryClient()
  const { addNotification } = useAppStore()
  const { setCurrentAgent, agents, currentAgentId } = useConfigStore()
  
  const [editingAgent, setEditingAgent] = useState<AgentConfig | null>(null)
  const [isDialogOpen, setIsDialogOpen] = useState(false)

  // 查询所有 Agents
  const { data: agentsData, isLoading } = useQuery({
    queryKey: ['agents'],
    queryFn: () => agentApi.getAllAgents(),
  })

  const currentAgent = agents.find(a => a.id === currentAgentId)

  const agentsList = agentsData?.data || []

  // 保存 Agent
  const saveMutation = useMutation({
    mutationFn: agentApi.saveAgent,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agents'] })
      setIsDialogOpen(false)
      setEditingAgent(null)
      addNotification({
        
        title: '保存成功',
        message: 'Agent 配置已保存',
        type: 'success',
        
      })
    },
    onError: (error) => {
      addNotification({
        
        title: '保存失败',
        message: String(error),
        type: 'error',
        
      })
    },
  })

  // 删除 Agent
  const deleteMutation = useMutation({
    mutationFn: agentApi.deleteAgent,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agents'] })
      addNotification({
        
        title: '删除成功',
        message: 'Agent 已删除',
        type: 'success',
        
      })
    },
  })

  // 设置当前 Agent
  const setCurrentMutation = useMutation({
    mutationFn: agentApi.setCurrentAgent,
    onSuccess: (_, agentId) => {
      setCurrentAgent(agentId)
      queryClient.invalidateQueries({ queryKey: ['agents'] })
      addNotification({
        title: '切换成功',
        message: '当前 Agent 已更新',
        type: 'success',
      })
    },
  })

  const handleSave = () => {
    if (!editingAgent) return
    saveMutation.mutate(editingAgent)
  }

  const handleAddAgent = () => {
    setEditingAgent({
      id: crypto.randomUUID(),
      name: '',
      description: '',
      modelId: '',
      systemPrompt: '',
      skills: [],
      enabled: true,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    })
    setIsDialogOpen(true)
  }

  const handleEdit = (agent: AgentConfig) => {
    setEditingAgent({ ...agent })
    setIsDialogOpen(true)
  }

  const isCurrentAgent = (agent: AgentConfig) => {
    return currentAgent?.id === agent.id
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Agent 管理</h1>
          <p className="text-muted-foreground">创建和管理 AI Agent 配置</p>
        </div>
        <Button onClick={handleAddAgent}>
          <Plus className="mr-2 h-4 w-4" />
          创建 Agent
        </Button>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {isLoading ? (
          <div className="col-span-full text-center py-8">加载中...</div>
        ) : agentsList.length === 0 ? (
          <Card className="col-span-full">
            <CardContent className="flex flex-col items-center justify-center py-12">
              <Bot className="h-12 w-12 text-muted-foreground mb-4" />
              <p className="text-muted-foreground">尚未创建任何 Agent</p>
              <Button className="mt-4" onClick={handleAddAgent}>
                <Plus className="mr-2 h-4 w-4" />
                创建第一个 Agent
              </Button>
            </CardContent>
          </Card>
        ) : (
          agentsList.map((agent) => (
            <Card 
              key={agent.id} 
              className={isCurrentAgent(agent) ? 'border-primary ring-1 ring-primary' : ''}
            >
              <CardHeader className="pb-3">
                <div className="flex items-start justify-between">
                  <div className="flex items-center gap-2">
                    <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center">
                      {agent.avatar ? (
                        <img src={agent.avatar} alt={agent.name} className="w-8 h-8 rounded-full" />
                      ) : (
                        <User className="h-4 w-4 text-primary" />
                      )}
                    </div>
                    <div>
                      <CardTitle className="text-base">{agent.name}</CardTitle>
                      {isCurrentAgent(agent) && (
                        <Badge variant="default" className="text-xs">
                          <CheckCircle className="mr-1 h-3 w-3" />
                          当前使用
                        </Badge>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center gap-1">
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8"
                      onClick={() => handleEdit(agent)}
                    >
                      <Edit2 className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8"
                      onClick={() => deleteMutation.mutate(agent.id)}
                      disabled={deleteMutation.isPending}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
                <CardDescription className="mt-2">
                  {agent.description || '无描述'}
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <div className="flex items-center gap-2 text-sm">
                    <Brain className="h-4 w-4 text-muted-foreground" />
                    <span className="text-muted-foreground">模型:</span>
                    <span>{agent.modelId || '未设置'}</span>
                  </div>
                  <div className="flex items-center gap-2 text-sm">
                    <Sparkles className="h-4 w-4 text-muted-foreground" />
                    <span className="text-muted-foreground">技能:</span>
                    <span>{agent.skills.length} 个</span>
                  </div>
                </div>

                {!isCurrentAgent(agent) && agent.enabled && (
                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full"
                    onClick={() => setCurrentMutation.mutate(agent.id)}
                    disabled={setCurrentMutation.isPending}
                  >
                    切换到此 Agent
                  </Button>
                )}

                {!agent.enabled && (
                  <Badge variant="secondary" className="w-full justify-center">已禁用</Badge>
                )}
              </CardContent>
            </Card>
          ))
        )}
      </div>

      {/* 编辑/添加 Agent 对话框 */}
      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>{editingAgent?.id ? '编辑 Agent' : '创建 Agent'}</DialogTitle>
            <DialogDescription>
              配置 Agent 的行为和使用的模型
            </DialogDescription>
          </DialogHeader>

          {editingAgent && (
            <div className="grid gap-4 py-4">
              <div className="space-y-2">
                <Label htmlFor="name">名称 *</Label>
                <Input
                  id="name"
                  value={editingAgent.name}
                  onChange={(e) => setEditingAgent({ ...editingAgent, name: e.target.value })}
                  placeholder="例如: 代码助手"
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="description">描述</Label>
                <Input
                  id="description"
                  value={editingAgent.description || ''}
                  onChange={(e) => setEditingAgent({ ...editingAgent, description: e.target.value })}
                  placeholder="简短描述这个 Agent 的用途"
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="modelId">使用模型</Label>
                <select
                  id="modelId"
                  className="w-full h-10 px-3 rounded-md border border-input bg-background"
                  value={editingAgent.modelId}
                  onChange={(e) => setEditingAgent({ ...editingAgent, modelId: e.target.value })}
                >
                  <option value="">选择模型...</option>
                  <option value="default-gpt4">GPT-4</option>
                  <option value="default-claude">Claude 3</option>
                </select>
              </div>

              <div className="space-y-2">
                <Label htmlFor="systemPrompt">系统提示词 (System Prompt)</Label>
                <Textarea
                  id="systemPrompt"
                  value={editingAgent.systemPrompt || ''}
                  onChange={(e) => setEditingAgent({ ...editingAgent, systemPrompt: e.target.value })}
                  placeholder="定义这个 Agent 的角色和行为..."
                  rows={4}
                />
              </div>

              <div className="flex items-center space-x-2">
                <input
                  type="checkbox"
                  id="enabled"
                  checked={editingAgent.enabled}
                  onChange={(e) => setEditingAgent({ ...editingAgent, enabled: e.target.checked })}
                  className="h-4 w-4 rounded border-gray-300"
                />
                <Label htmlFor="enabled">启用此 Agent</Label>
              </div>
            </div>
          )}

          <DialogFooter>
            <Button variant="outline" onClick={() => setIsDialogOpen(false)}>
              取消
            </Button>
            <Button 
              onClick={handleSave} 
              disabled={saveMutation.isPending || !editingAgent?.name}
            >
              {saveMutation.isPending ? '保存中...' : '保存'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
