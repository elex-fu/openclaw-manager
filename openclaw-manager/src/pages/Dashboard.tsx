import { useEffect, useMemo } from 'react'
import { useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import {
  Bot,
  Brain,
  Settings,
  Stethoscope,
  Download,
  CheckCircle,
  XCircle
} from 'lucide-react'
import { openclawApi, serviceApi } from '@/lib/tauri-api'
import { useAppStore } from '@/stores/appStore'
import { useConfigStore } from '@/stores/configStore'
import { ServiceStatus } from '@/components/openclaw/ServiceStatus'

export function Dashboard() {
  const navigate = useNavigate()
  const { addNotification } = useAppStore()
  const { config, agents, currentAgentId, models } = useConfigStore()

  // 计算当前 Agent 和默认模型
  const currentAgent = useMemo(() =>
    agents.find(a => a.id === currentAgentId),
    [agents, currentAgentId]
  )
  const defaultModel = useMemo(() =>
    models.find(m => m.isDefault),
    [models]
  )

  // 查询安装状态
  const { data: installData, isLoading: isInstallLoading } = useQuery({
    queryKey: ['openclaw-installation'],
    queryFn: () => openclawApi.checkInstallation(),
    refetchInterval: 30000,
  })

  // 查询服务状态
  const { data: serviceData } = useQuery({
    queryKey: ['service-status'],
    queryFn: () => serviceApi.getServiceStatus(),
    refetchInterval: 5000,
    enabled: installData?.data?.type === 'Installed',
  })

  const installStatus = installData?.data
  const isInstalled = installStatus?.type === 'Installed'

  useEffect(() => {
    if (installStatus?.type === 'Error') {
      addNotification({
        title: '安装错误',
        message: installStatus.message,
        type: 'error',

      })
    }
  }, [installStatus, addNotification])

  // 未安装时显示安装引导
  if (!isInstalled && !isInstallLoading) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh] space-y-6">
        <div className="text-center space-y-4">
          <div className="w-20 h-20 mx-auto bg-primary/10 rounded-full flex items-center justify-center">
            <Download className="w-10 h-10 text-primary" />
          </div>
          <h1 className="text-3xl font-bold">欢迎使用 OpenClaw</h1>
          <p className="text-muted-foreground max-w-md">
            OpenClaw 是一个强大的 AI Agent 框架。您需要先安装 OpenClaw 才能开始使用。
          </p>
        </div>
        <div className="flex gap-4">
          <Button size="lg" onClick={() => navigate('/install')}>
            <Download className="mr-2 h-5 w-5" />
            安装 OpenClaw
          </Button>
          <Button size="lg" variant="outline" onClick={() => navigate('/settings')}>
            <Settings className="mr-2 h-5 w-5" />
            设置
          </Button>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div>
        <h1 className="text-3xl font-bold">仪表盘</h1>
        <p className="text-muted-foreground">查看 OpenClaw 运行状态</p>
      </div>

      {/* 状态卡片网格 */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {/* OpenClaw 安装状态 */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">OpenClaw 状态</CardTitle>
            <CheckCircle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-2">
              {isInstalled ? (
                <>
                  <CheckCircle className="h-5 w-5 text-green-500" />
                  <span className="text-2xl font-bold">已安装</span>
                </>
              ) : (
                <>
                  <XCircle className="h-5 w-5 text-red-500" />
                  <span className="text-2xl font-bold">未安装</span>
                </>
              )}
            </div>
            {isInstalled && (
              <p className="text-xs text-muted-foreground mt-1">
                版本: {(installStatus as Extract<typeof installStatus, { type: 'Installed' }>).version}
              </p>
            )}
          </CardContent>
        </Card>

        {/* Gateway 服务状态 */}
        <ServiceStatus />

        {/* 当前 Agent */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">当前 Agent</CardTitle>
            <Bot className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {currentAgent?.name || '未设置'}
            </div>
            <p className="text-xs text-muted-foreground mt-1">
              {currentAgent?.description || '使用默认配置'}
            </p>
            <Button
              variant="link"
              className="px-0 mt-2"
              onClick={() => navigate('/agents')}
            >
              管理 Agents →
            </Button>
          </CardContent>
        </Card>

        {/* 默认模型 */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">默认模型</CardTitle>
            <Brain className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {defaultModel?.name || '未设置'}
            </div>
            <p className="text-xs text-muted-foreground mt-1">
              {defaultModel?.provider || '请配置模型'}
            </p>
            <Button
              variant="link"
              className="px-0 mt-2"
              onClick={() => navigate('/models')}
            >
              配置模型 →
            </Button>
          </CardContent>
        </Card>

        {/* 快速操作 */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">快速操作</CardTitle>
            <Settings className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent className="space-y-2">
            <Button
              variant="outline"
              className="w-full justify-start"
              onClick={() => navigate('/diagnostics')}
            >
              <Stethoscope className="mr-2 h-4 w-4" />
              运行诊断
            </Button>
            <Button
              variant="outline"
              className="w-full justify-start"
              onClick={() => navigate('/settings')}
            >
              <Settings className="mr-2 h-4 w-4" />
              打开设置
            </Button>
          </CardContent>
        </Card>
      </div>

      {/* 最近活动 */}
      <Card>
        <CardHeader>
          <CardTitle>系统信息</CardTitle>
          <CardDescription>OpenClaw 配置概览</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm text-muted-foreground">已配置模型</p>
              <p className="text-lg font-medium">{config?.models.length || 0} 个</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">已创建 Agent</p>
              <p className="text-lg font-medium">{config?.agents.length || 0} 个</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">已启用技能</p>
              <p className="text-lg font-medium">
                {config?.skills.filter(s => s.enabled).length || 0} 个
              </p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">配置版本</p>
              <p className="text-lg font-medium">{config?.version || '1.0.0'}</p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
