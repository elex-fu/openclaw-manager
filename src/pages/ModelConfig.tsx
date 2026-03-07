import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Switch } from '@/components/ui/switch'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
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
  Trash2,
  Edit2,
  Brain,
  Key,
  TestTube,
  Star,
  GripVertical,
  CheckCircle2,
  XCircle,
  Loader2,
} from 'lucide-react'
import { modelApi, secureStorageApi } from '@/lib/tauri-api'
import { useConfigStore } from '@/stores/configStore'
import { useAppStore } from '@/stores/appStore'
import type { ModelConfig } from '@/types'

// DnD Kit imports
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from '@dnd-kit/core'
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'

// Sortable Model Card Component
interface SortableModelCardProps {
  model: ModelConfig
  onEdit: (model: ModelConfig) => void
  onDelete: (id: string) => void
  onTest: (id: string) => void
  onSetDefault: (id: string) => void
  testStatus: { loading: boolean; result?: { success: boolean; latency: number; message?: string } }
  isDefaultLoading: boolean
}

function SortableModelCard({
  model,
  onEdit,
  onDelete,
  onTest,
  onSetDefault,
  testStatus,
  isDefaultLoading,
}: SortableModelCardProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: model.id })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  }

  return (
    <div ref={setNodeRef} style={style}>
      <Card className={model.isDefault ? 'border-primary' : ''}>
        <CardHeader className="pb-3">
          <div className="flex items-start justify-between">
            <div className="flex items-center gap-2">
              <div
                {...attributes}
                {...listeners}
                className="cursor-grab active:cursor-grabbing p-1 hover:bg-muted rounded"
              >
                <GripVertical className="h-4 w-4 text-muted-foreground" />
              </div>
              <CardTitle>{model.name}</CardTitle>
              {model.isDefault && (
                <Badge variant="default">
                  <Star className="mr-1 h-3 w-3" />
                  默认
                </Badge>
              )}
              {!model.enabled && <Badge variant="secondary">已禁用</Badge>}
            </div>
            <div className="flex items-center gap-2">
              <Button
                variant="ghost"
                size="icon"
                onClick={() => onTest(model.id)}
                disabled={testStatus.loading}
                title="测试连接"
              >
                {testStatus.loading ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : testStatus.result ? (
                  testStatus.result.success ? (
                    <CheckCircle2 className="h-4 w-4 text-green-500" />
                  ) : (
                    <XCircle className="h-4 w-4 text-red-500" />
                  )
                ) : (
                  <TestTube className="h-4 w-4" />
                )}
              </Button>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => onEdit(model)}
                title="编辑"
              >
                <Edit2 className="h-4 w-4" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => onDelete(model.id)}
                title="删除"
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </div>
          </div>
          <CardDescription>
            {model.provider} · {model.model}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <p className="text-muted-foreground">Temperature</p>
              <p className="font-medium">{model.temperature}</p>
            </div>
            <div>
              <p className="text-muted-foreground">Max Tokens</p>
              <p className="font-medium">{model.max_tokens || '默认'}</p>
            </div>
          </div>

          {testStatus.result && (
            <div className={`text-sm p-2 rounded ${testStatus.result.success ? 'bg-green-50 text-green-700' : 'bg-red-50 text-red-700'}`}>
              {testStatus.result.success
                ? `连接成功 - 延迟: ${testStatus.result.latency}ms`
                : `连接失败: ${testStatus.result.message}`}
            </div>
          )}

          {!model.isDefault && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => onSetDefault(model.id)}
              disabled={isDefaultLoading}
            >
              设为默认
            </Button>
          )}
        </CardContent>
      </Card>
    </div>
  )
}

export function ModelConfigPage() {
  const queryClient = useQueryClient()
  const { addNotification } = useAppStore()
  const { setDefaultModel } = useConfigStore()

  const [editingModel, setEditingModel] = useState<ModelConfig | null>(null)
  const [isDialogOpen, setIsDialogOpen] = useState(false)
  const [apiKeyInput, setApiKeyInput] = useState('')
  const [testStatus, setTestStatus] = useState<Record<string, { loading: boolean; result?: { success: boolean; latency: number; message?: string } }>>({})

  // DnD sensors
  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  )

  // 查询所有模型
  const { data: modelsData, isLoading } = useQuery({
    queryKey: ['models-full'],
    queryFn: () => modelApi.getAllModels(),
  })

  const models = modelsData || []

  // 保存模型
  const saveMutation = useMutation({
    mutationFn: modelApi.saveModel,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['models-full'] })
      setIsDialogOpen(false)
      setEditingModel(null)
      setApiKeyInput('')
      addNotification({
        title: '保存成功',
        message: '模型配置已保存',
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

  // 删除模型
  const deleteMutation = useMutation({
    mutationFn: modelApi.deleteModel,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['models-full'] })
      addNotification({
        title: '删除成功',
        message: '模型已删除',
        type: 'success',
      })
    },
  })

  // 测试连接
  const handleTestConnection = async (modelId: string) => {
    setTestStatus((prev) => ({ ...prev, [modelId]: { loading: true } }))
    try {
      const result = await modelApi.testModelConnection(modelId)
      setTestStatus((prev) => ({
        ...prev,
        [modelId]: { loading: false, result: result || undefined },
      }))
      if (result.success) {
        addNotification({
          title: '连接成功',
          message: `延迟: ${result.latency}ms`,
          type: 'success',
        })
      } else {
        addNotification({
          title: '连接失败',
          message: result.message || '未知错误',
          type: 'error',
        })
      }
    } catch (error) {
      setTestStatus((prev) => ({
        ...prev,
        [modelId]: { loading: false },
      }))
      addNotification({
        title: '连接失败',
        message: String(error),
        type: 'error',
      })
    }
  }

  // 设置默认模型
  const setDefaultMutation = useMutation({
    mutationFn: modelApi.setDefaultModel,
    onSuccess: (_, modelId) => {
      setDefaultModel(modelId)
      queryClient.invalidateQueries({ queryKey: ['models-full'] })
      addNotification({
        title: '设置成功',
        message: '默认模型已更新',
        type: 'success',
      })
    },
  })

  // 拖拽排序
  const handleDragEnd = async (event: DragEndEvent) => {
    const { active, over } = event
    if (!over || active.id === over.id) return

    const oldIndex = models.findIndex((m: ModelConfig) => m.id === active.id)
    const newIndex = models.findIndex((m: ModelConfig) => m.id === over.id)

    const newModels = arrayMove(models, oldIndex, newIndex)

    try {
      await modelApi.reorderModels(newModels.map((m: ModelConfig) => m.id))
      queryClient.invalidateQueries({ queryKey: ['models-full'] })
      addNotification({
        title: '排序已更新',
        message: '模型优先级已更新',
        type: 'success',
      })
    } catch (error) {
      addNotification({
        title: '排序更新失败',
        message: String(error),
        type: 'error',
      })
    }
  }

  const handleSave = async () => {
    if (!editingModel) return

    // 保存 API Key 到安全存储
    if (apiKeyInput) {
      await secureStorageApi.saveApiKey(editingModel.provider, apiKeyInput)
    }

    // 保存模型配置
    saveMutation.mutate(editingModel)
  }

  const handleAddModel = () => {
    setEditingModel({
      id: `model-${Date.now()}`,
      name: '',
      provider: 'openai',
      model: 'gpt-4',
      api_base: '',
      temperature: 1.0,
      max_tokens: 2048,
      enabled: true,
      isDefault: false,
    })
    setApiKeyInput('')
    setIsDialogOpen(true)
  }

  const handleEdit = async (model: ModelConfig) => {
    setEditingModel({ ...model })
    // 获取已保存的 API Key
    const keyResult = await secureStorageApi.getApiKey(model.provider)
    setApiKeyInput(keyResult || '')
    setIsDialogOpen(true)
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">模型配置</h1>
          <p className="text-muted-foreground">管理 AI 模型提供商和 API 密钥</p>
        </div>
        <Button onClick={handleAddModel}>
          <Plus className="mr-2 h-4 w-4" />
          添加模型
        </Button>
      </div>

      <Alert>
        <Key className="h-4 w-4" />
        <AlertDescription>
          API Key 将安全存储在系统密钥链中，不会保存在配置文件里。拖拽卡片可调整模型优先级。
        </AlertDescription>
      </Alert>

      <div className="grid gap-4">
        {isLoading ? (
          <div className="text-center py-8">加载中...</div>
        ) : models.length === 0 ? (
          <Card>
            <CardContent className="flex flex-col items-center justify-center py-12">
              <Brain className="h-12 w-12 text-muted-foreground mb-4" />
              <p className="text-muted-foreground">尚未配置任何模型</p>
              <Button className="mt-4" onClick={handleAddModel}>
                <Plus className="mr-2 h-4 w-4" />
                添加第一个模型
              </Button>
            </CardContent>
          </Card>
        ) : (
          <DndContext
            sensors={sensors}
            collisionDetection={closestCenter}
            onDragEnd={handleDragEnd}
          >
            <SortableContext
              items={models.map((m: ModelConfig) => m.id)}
              strategy={verticalListSortingStrategy}
            >
              <div className="grid gap-4">
                {models.map((model: ModelConfig) => (
                  <SortableModelCard
                    key={model.id}
                    model={model}
                    onEdit={handleEdit}
                    onDelete={(id) => deleteMutation.mutate(id)}
                    onTest={handleTestConnection}
                    onSetDefault={(id) => setDefaultMutation.mutate(id)}
                    testStatus={testStatus[model.id] || { loading: false }}
                    isDefaultLoading={setDefaultMutation.isPending}
                  />
                ))}
              </div>
            </SortableContext>
          </DndContext>
        )}
      </div>

      {/* 编辑/添加模型对话框 */}
      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{editingModel?.id ? '编辑模型' : '添加模型'}</DialogTitle>
            <DialogDescription>
              配置 AI 模型提供商的详细信息
            </DialogDescription>
          </DialogHeader>

          {editingModel && (
            <Tabs defaultValue="basic" className="w-full">
              <TabsList className="grid w-full grid-cols-1">
                <TabsTrigger value="basic">基本信息</TabsTrigger>
              </TabsList>

              <TabsContent value="basic" className="space-y-4 py-4">
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="name">名称</Label>
                    <Input
                      id="name"
                      value={editingModel.name}
                      onChange={(e) =>
                        setEditingModel({ ...editingModel, name: e.target.value })
                      }
                      placeholder="例如: GPT-4"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="provider">提供商</Label>
                    <select
                      id="provider"
                      className="w-full h-10 px-3 rounded-md border border-input bg-background"
                      value={editingModel.provider}
                      onChange={(e) =>
                        setEditingModel({ ...editingModel, provider: e.target.value })
                      }
                    >
                      <option value="openai">OpenAI</option>
                      <option value="anthropic">Anthropic</option>
                      <option value="google">Google</option>
                      <option value="azure">Azure OpenAI</option>
                      <option value="local">本地模型</option>
                    </select>
                  </div>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="model">模型 ID</Label>
                  <Input
                    id="model"
                    value={editingModel.model}
                    onChange={(e) =>
                      setEditingModel({ ...editingModel, model: e.target.value })
                    }
                    placeholder="例如: gpt-4, claude-3-opus-20240229"
                  />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="apiKey">API Key</Label>
                  <Input
                    id="apiKey"
                    type="password"
                    value={apiKeyInput}
                    onChange={(e) => setApiKeyInput(e.target.value)}
                    placeholder="输入 API Key（将安全存储）"
                  />
                  <p className="text-xs text-muted-foreground">
                    API Key 将存储在系统密钥链中，不会保存在配置文件
                  </p>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="apiBase">API Base URL（可选）</Label>
                  <Input
                    id="apiBase"
                    value={editingModel.api_base || ''}
                    onChange={(e) =>
                      setEditingModel({ ...editingModel, api_base: e.target.value })
                    }
                    placeholder="例如: https://api.openai.com/v1"
                  />
                </div>

                <div className="flex items-center space-x-2">
                  <Switch
                    id="enabled"
                    checked={editingModel.enabled}
                    onCheckedChange={(checked: boolean) =>
                      setEditingModel({ ...editingModel, enabled: checked })
                    }
                  />
                  <Label htmlFor="enabled">启用此模型</Label>
                </div>
              </TabsContent>
            </Tabs>
          )}

          <DialogFooter>
            <Button variant="outline" onClick={() => setIsDialogOpen(false)}>
              取消
            </Button>
            <Button onClick={handleSave} disabled={saveMutation.isPending}>
              {saveMutation.isPending ? '保存中...' : '保存'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
