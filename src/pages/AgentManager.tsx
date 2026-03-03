import { useState, useMemo, useCallback, memo, useRef } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Separator } from '@/components/ui/separator'
import { SkeletonGrid } from '@/components/ui/skeleton'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import {
  Plus,
  Search,
  Bot,
  AlertCircle,
  LayoutGrid,
  List,
  CheckCircle,
  Sparkles,
} from 'lucide-react'
import { agentApi, modelApi } from '@/lib/tauri-api'
import { useConfigStore } from '@/stores/configStore'
import { useAppStore } from '@/stores/appStore'
import { AgentCard } from '@/components/openclaw/AgentCard'
import { EmptyListState, EmptySearchState } from '@/components/error'
import { StaggerContainer, StaggerItem, ScaleIn } from '@/components/animation'
import { useVirtualizer } from '@tanstack/react-virtual'
import type { AgentConfig } from '@/types'

// 表单验证错误类型
interface FormErrors {
  name?: string
  modelId?: string
  description?: string
  systemPrompt?: string
}

// 验证表单 - 使用 useCallback
function useValidateAgentForm() {
  return useCallback((agent: Partial<AgentConfig>): FormErrors => {
    const errors: FormErrors = {}

    if (!agent.name?.trim()) {
      errors.name = '请输入 Agent 名称'
    } else if (agent.name.length < 2) {
      errors.name = '名称至少需要 2 个字符'
    } else if (agent.name.length > 50) {
      errors.name = '名称不能超过 50 个字符'
    }

    if (!agent.modelId) {
      errors.modelId = '请选择使用的模型'
    }

    if (agent.description && agent.description.length > 200) {
      errors.description = '描述不能超过 200 个字符'
    }

    if (agent.systemPrompt && agent.systemPrompt.length > 4000) {
      errors.systemPrompt = '系统提示词不能超过 4000 个字符'
    }

    return errors
  }, [])
}

// 统计卡片组件 - 使用 memo
const StatCard = memo(function StatCard({
  title,
  value,
  icon: Icon,
  className,
}: {
  title: string
  value: string | number
  icon: React.ElementType
  className?: string
}) {
  return (
    <Card className="hover:shadow-md transition-shadow">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        <Icon className="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div className={`text-2xl font-bold ${className}`}>{value}</div>
      </CardContent>
    </Card>
  )
})

// 虚拟化 Agent 列表组件
interface VirtualAgentListProps {
  agents: AgentConfig[]
  onEdit: (agent: AgentConfig) => void
  onDelete: (id: string) => void
  viewMode: 'card' | 'list'
}

const VirtualAgentList = memo(function VirtualAgentList({
  agents,
  onEdit,
  onDelete,
  viewMode,
}: VirtualAgentListProps) {
  const parentRef = useRef<HTMLDivElement>(null)

  const virtualizer = useVirtualizer({
    count: agents.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => (viewMode === 'card' ? 280 : 120),
    overscan: 3,
  })

  const virtualItems = virtualizer.getVirtualItems()

  if (viewMode === 'card') {
    return (
      <div ref={parentRef} className="h-[600px] overflow-auto">
        <div
          style={{
            height: `${virtualizer.getTotalSize()}px`,
            width: '100%',
            position: 'relative',
          }}
          className="grid gap-4 md:grid-cols-2 lg:grid-cols-3"
        >
          {virtualItems.map((virtualItem) => {
            const agent = agents[virtualItem.index]
            return (
              <div
                key={agent.id}
                style={{
                  position: 'absolute',
                  top: 0,
                  left: `${(virtualItem.index % 3) * 33.33}%`,
                  width: '33.33%',
                  height: `${virtualItem.size}px`,
                  transform: `translateY(${virtualItem.start}px)`,
                  padding: '0 0.5rem',
                }}
              >
                <AgentCard
                  agent={agent}
                  onEdit={onEdit}
                  onDelete={onDelete}
                  viewMode="card"
                />
              </div>
            )
          })}
        </div>
      </div>
    )
  }

  // 列表视图
  return (
    <div ref={parentRef} className="h-[600px] overflow-auto space-y-2">
      <div
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: '100%',
          position: 'relative',
        }}
      >
        {virtualItems.map((virtualItem) => {
          const agent = agents[virtualItem.index]
          return (
            <div
              key={agent.id}
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                height: `${virtualItem.size}px`,
                transform: `translateY(${virtualItem.start}px)`,
              }}
            >
              <AgentCard
                agent={agent}
                onEdit={onEdit}
                onDelete={onDelete}
                viewMode="list"
              />
            </div>
          )
        })}
      </div>
    </div>
  )
})

export function AgentManager() {
  const queryClient = useQueryClient()
  const { addNotification } = useAppStore()
  const { currentAgentId } = useConfigStore()
  const validateAgentForm = useValidateAgentForm()

  // UI 状态
  const [viewMode, setViewMode] = useState<'card' | 'list'>('card')
  const [searchQuery, setSearchQuery] = useState('')
  const [isDialogOpen, setIsDialogOpen] = useState(false)
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false)
  const [agentToDelete, setAgentToDelete] = useState<string | null>(null)
  const [formErrors, setFormErrors] = useState<FormErrors>({})

  // 编辑状态
  const [editingAgent, setEditingAgent] = useState<Partial<AgentConfig> | null>(null)
  const [isEditing, setIsEditing] = useState(false)

  // 查询所有 Agents - 使用优化配置
  const { data: agentsData, isLoading: isLoadingAgents, error: agentsError } = useQuery({
    queryKey: ['agents'],
    queryFn: () => agentApi.getAllAgents(),
    staleTime: 1000 * 30, // 30秒
    gcTime: 1000 * 60 * 5, // 5分钟
  })

  // 查询所有模型
  const { data: modelsData, isLoading: isLoadingModels } = useQuery({
    queryKey: ['models'],
    queryFn: () => modelApi.getAllModels(),
    staleTime: 1000 * 60 * 5, // 5分钟
  })

  const agentsList = agentsData?.data || []
  const modelsList = modelsData?.data || []

  // 过滤 Agents - 使用 useMemo
  const filteredAgents = useMemo(() => {
    if (!searchQuery.trim()) return agentsList
    const query = searchQuery.toLowerCase()
    return agentsList.filter(
      (agent) =>
        agent.name.toLowerCase().includes(query) ||
        agent.description?.toLowerCase().includes(query) ||
        agent.skills.some((skill) => skill.toLowerCase().includes(query))
    )
  }, [agentsList, searchQuery])

  // 统计信息 - 使用 useMemo
  const stats = useMemo(() => {
    const total = agentsList.length
    const enabled = agentsList.filter((a) => a.enabled).length
    const disabled = total - enabled
    return { total, enabled, disabled }
  }, [agentsList])

  // 保存 Agent
  const saveMutation = useMutation({
    mutationFn: agentApi.saveAgent,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agents'] })
      setIsDialogOpen(false)
      setEditingAgent(null)
      setFormErrors({})
      addNotification({
        title: isEditing ? '更新成功' : '创建成功',
        message: isEditing ? 'Agent 配置已更新' : '新 Agent 已创建',
        type: 'success',
      })
    },
    onError: (error) => {
      addNotification({
        title: '保存失败',
        message: String(error) || '请检查输入并重试',
        type: 'error',
      })
    },
  })

  // 删除 Agent
  const deleteMutation = useMutation({
    mutationFn: agentApi.deleteAgent,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agents'] })
      setIsDeleteDialogOpen(false)
      setAgentToDelete(null)
      addNotification({
        title: '删除成功',
        message: 'Agent 已删除',
        type: 'success',
      })
    },
    onError: (error) => {
      addNotification({
        title: '删除失败',
        message: String(error) || '无法删除 Agent',
        type: 'error',
      })
    },
  })

  // 处理保存 - 使用 useCallback
  const handleSave = useCallback(() => {
    if (!editingAgent) return

    // 验证表单
    const errors = validateAgentForm(editingAgent)
    if (Object.keys(errors).length > 0) {
      setFormErrors(errors)
      return
    }

    // 构建完整的 Agent 配置
    const now = new Date().toISOString()
    const agentToSave: AgentConfig = {
      id: editingAgent.id || crypto.randomUUID(),
      name: editingAgent.name!.trim(),
      description: editingAgent.description?.trim() || undefined,
      modelId: editingAgent.modelId!,
      systemPrompt: editingAgent.systemPrompt?.trim() || undefined,
      skills: editingAgent.skills || [],
      enabled: editingAgent.enabled ?? true,
      createdAt: editingAgent.createdAt || now,
      updatedAt: now,
    }

    saveMutation.mutate(agentToSave)
  }, [editingAgent, validateAgentForm, saveMutation])

  // 打开创建对话框 - 使用 useCallback
  const handleAddAgent = useCallback(() => {
    setIsEditing(false)
    setEditingAgent({
      name: '',
      description: '',
      modelId: modelsList.length > 0 ? modelsList[0].id : '',
      systemPrompt: '',
      skills: [],
      enabled: true,
    })
    setFormErrors({})
    setIsDialogOpen(true)
  }, [modelsList])

  // 打开编辑对话框 - 使用 useCallback
  const handleEdit = useCallback((agent: AgentConfig) => {
    setIsEditing(true)
    setEditingAgent({ ...agent })
    setFormErrors({})
    setIsDialogOpen(true)
  }, [])

  // 打开删除确认对话框 - 使用 useCallback
  const handleDeleteClick = useCallback((id: string) => {
    setAgentToDelete(id)
    setIsDeleteDialogOpen(true)
  }, [])

  // 确认删除 - 使用 useCallback
  const handleConfirmDelete = useCallback(() => {
    if (agentToDelete) {
      deleteMutation.mutate(agentToDelete)
    }
  }, [agentToDelete, deleteMutation])

  // 更新编辑状态 - 使用 useCallback
  const updateEditingAgent = useCallback((updates: Partial<AgentConfig>) => {
    setEditingAgent((prev) => (prev ? { ...prev, ...updates } : null))
    // 清除相关错误
    setFormErrors((prev) => {
      const newErrors = { ...prev }
      Object.keys(updates).forEach((key) => {
        delete newErrors[key as keyof FormErrors]
      })
      return newErrors
    })
  }, [])

  // 清除搜索 - 使用 useCallback
  const clearSearch = useCallback(() => {
    setSearchQuery('')
  }, [])

  // 当前 Agent
  const currentAgent = agentsList.find((a) => a.id === currentAgentId)

  // 判断是否使用虚拟滚动（当列表项超过20时使用）
  const useVirtualScroll = filteredAgents.length > 20

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <ScaleIn>
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">Agent 管理</h1>
            <p className="text-muted-foreground">创建和管理 AI Agent 配置</p>
          </div>
          <Button onClick={handleAddAgent} disabled={isLoadingModels} loading={isLoadingModels}>
            <Plus className="mr-2 h-4 w-4" />
            创建 Agent
          </Button>
        </div>
      </ScaleIn>

      {/* 统计卡片 */}
      <StaggerContainer className="grid gap-4 md:grid-cols-4">
        <StaggerItem>
          <StatCard
            title="总 Agent 数"
            value={stats.total}
            icon={Bot}
          />
        </StaggerItem>
        <StaggerItem>
          <StatCard
            title="已启用"
            value={stats.enabled}
            icon={CheckCircle}
            className="text-green-600"
          />
        </StaggerItem>
        <StaggerItem>
          <StatCard
            title="已禁用"
            value={stats.disabled}
            icon={Bot}
            className="text-muted-foreground"
          />
        </StaggerItem>
        <StaggerItem>
          <Card className="hover:shadow-md transition-shadow">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">当前使用</CardTitle>
              <Sparkles className="h-4 w-4 text-primary" />
            </CardHeader>
            <CardContent>
              <div className="text-sm font-medium truncate">
                {currentAgent?.name || '未设置'}
              </div>
            </CardContent>
          </Card>
        </StaggerItem>
      </StaggerContainer>

      {/* 错误提示 */}
      {agentsError && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertTitle>加载失败</AlertTitle>
          <AlertDescription>
            无法加载 Agent 列表，请检查网络连接或稍后重试。
          </AlertDescription>
        </Alert>
      )}

      {/* 搜索和视图切换 */}
      <ScaleIn delay={0.2}>
        <div className="flex items-center gap-4">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="搜索 Agent 名称、描述或技能..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-9"
            />
          </div>
          <Tabs value={viewMode} onValueChange={(v) => setViewMode(v as 'card' | 'list')}>
            <TabsList>
              <TabsTrigger value="card">
                <LayoutGrid className="h-4 w-4 mr-2" />
                卡片
              </TabsTrigger>
              <TabsTrigger value="list">
                <List className="h-4 w-4 mr-2" />
                列表
              </TabsTrigger>
            </TabsList>
          </Tabs>
        </div>
      </ScaleIn>

      {/* Agent 列表 */}
      {isLoadingAgents ? (
        <SkeletonGrid columns={3} rows={2} />
      ) : filteredAgents.length === 0 ? (
        searchQuery ? (
          <EmptySearchState
            searchTerm={searchQuery}
            onClear={clearSearch}
          />
        ) : (
          <EmptyListState
            itemName="Agent"
            onCreate={handleAddAgent}
          />
        )
      ) : useVirtualScroll ? (
        // 使用虚拟滚动（大数据量）
        <VirtualAgentList
          agents={filteredAgents}
          onEdit={handleEdit}
          onDelete={handleDeleteClick}
          viewMode={viewMode}
        />
      ) : (
        // 普通列表（小数据量）
        <TabsContent value={viewMode} className="mt-0">
          {viewMode === 'card' ? (
            <StaggerContainer className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {filteredAgents.map((agent) => (
                <StaggerItem key={agent.id}>
                  <AgentCard
                    agent={agent}
                    onEdit={handleEdit}
                    onDelete={handleDeleteClick}
                    viewMode="card"
                  />
                </StaggerItem>
              ))}
            </StaggerContainer>
          ) : (
            <StaggerContainer className="space-y-2">
              {filteredAgents.map((agent) => (
                <StaggerItem key={agent.id}>
                  <AgentCard
                    agent={agent}
                    onEdit={handleEdit}
                    onDelete={handleDeleteClick}
                    viewMode="list"
                  />
                </StaggerItem>
              ))}
            </StaggerContainer>
          )}
        </TabsContent>
      )}

      {/* 编辑/添加 Agent 对话框 */}
      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{isEditing ? '编辑 Agent' : '创建 Agent'}</DialogTitle>
            <DialogDescription>
              {isEditing
                ? '修改 Agent 的配置信息'
                : '配置新 Agent 的行为和使用的模型'}
            </DialogDescription>
          </DialogHeader>

          {editingAgent && (
            <div className="grid gap-4 py-4">
              {/* 名称 */}
              <div className="space-y-2">
                <Label htmlFor="name">
                  名称 <span className="text-destructive">*</span>
                </Label>
                <Input
                  id="name"
                  value={editingAgent.name || ''}
                  onChange={(e) => updateEditingAgent({ name: e.target.value })}
                  placeholder="例如: 代码助手"
                  className={formErrors.name ? 'border-destructive' : ''}
                />
                {formErrors.name && (
                  <p className="text-sm text-destructive">{formErrors.name}</p>
                )}
              </div>

              {/* 描述 */}
              <div className="space-y-2">
                <Label htmlFor="description">描述</Label>
                <Input
                  id="description"
                  value={editingAgent.description || ''}
                  onChange={(e) => updateEditingAgent({ description: e.target.value })}
                  placeholder="简短描述这个 Agent 的用途"
                  className={formErrors.description ? 'border-destructive' : ''}
                />
                {formErrors.description && (
                  <p className="text-sm text-destructive">{formErrors.description}</p>
                )}
                <p className="text-xs text-muted-foreground">
                  {editingAgent.description?.length || 0}/200 字符
                </p>
              </div>

              {/* 模型选择 */}
              <div className="space-y-2">
                <Label htmlFor="modelId">
                  使用模型 <span className="text-destructive">*</span>
                </Label>
                <Select
                  value={editingAgent.modelId || ''}
                  onValueChange={(value) => updateEditingAgent({ modelId: value })}
                >
                  <SelectTrigger className={formErrors.modelId ? 'border-destructive' : ''}>
                    <SelectValue placeholder="选择模型..." />
                  </SelectTrigger>
                  <SelectContent>
                    {modelsList.length === 0 ? (
                      <SelectItem value="" disabled>
                        暂无可用模型，请先配置模型
                      </SelectItem>
                    ) : (
                      modelsList
                        .filter((m) => m.enabled)
                        .map((model) => (
                          <SelectItem key={model.id} value={model.id}>
                            {model.name} ({model.provider})
                          </SelectItem>
                        ))
                    )}
                  </SelectContent>
                </Select>
                {formErrors.modelId && (
                  <p className="text-sm text-destructive">{formErrors.modelId}</p>
                )}
              </div>

              {/* 系统提示词 */}
              <div className="space-y-2">
                <Label htmlFor="systemPrompt">系统提示词 (System Prompt)</Label>
                <Textarea
                  id="systemPrompt"
                  value={editingAgent.systemPrompt || ''}
                  onChange={(e) => updateEditingAgent({ systemPrompt: e.target.value })}
                  placeholder="定义这个 Agent 的角色和行为..."
                  rows={4}
                  className={formErrors.systemPrompt ? 'border-destructive' : ''}
                />
                {formErrors.systemPrompt && (
                  <p className="text-sm text-destructive">{formErrors.systemPrompt}</p>
                )}
                <p className="text-xs text-muted-foreground">
                  {editingAgent.systemPrompt?.length || 0}/4000 字符
                </p>
              </div>

              {/* 启用状态 */}
              <div className="flex items-center space-x-2">
                <input
                  type="checkbox"
                  id="enabled"
                  checked={editingAgent.enabled}
                  onChange={(e) => updateEditingAgent({ enabled: e.target.checked })}
                  className="h-4 w-4 rounded border-gray-300"
                />
                <Label htmlFor="enabled">启用此 Agent</Label>
              </div>

              <Separator />

              {/* 提示信息 */}
              <div className="text-sm text-muted-foreground">
                <p>提示:</p>
                <ul className="list-disc list-inside space-y-1 mt-1">
                  <li>Agent 名称用于在列表中识别</li>
                  <li>系统提示词定义了 Agent 的角色和行为</li>
                  <li>禁用的 Agent 不会被使用</li>
                </ul>
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
              loading={saveMutation.isPending}
              loadingText={isEditing ? '保存中...' : '创建中...'}
            >
              {isEditing ? '保存修改' : '创建 Agent'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 删除确认对话框 */}
      <AlertDialog open={isDeleteDialogOpen} onOpenChange={setIsDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              此操作不可撤销。删除后，该 Agent 的配置将被永久移除。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={() => setAgentToDelete(null)}>
              取消
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={handleConfirmDelete}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              disabled={deleteMutation.isPending}
            >
              {deleteMutation.isPending ? '删除中...' : '删除'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
