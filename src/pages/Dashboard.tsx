import { useEffect, useMemo, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useQuery, useMutation } from '@tanstack/react-query'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { PageLoader } from '@/components/ui/loading'
import { Badge } from '@/components/ui/badge'
import {
  Bot,
  Brain,
  Settings,
  Stethoscope,
  Download,
  CheckCircle,
  XCircle,
  Package,
  Play,
  Square,
  Loader2
} from 'lucide-react'
import { openclawApi, serviceApi, sidecarApi } from '@/lib/tauri-api'
import { useAppStore } from '@/stores/appStore'
import { useConfigStore } from '@/stores/configStore'
import { useInstallStore } from '@/stores/installStore'
import { ServiceStatus } from '@/components/openclaw/ServiceStatus'
import { ResourceMonitor, ActivityLog, DiagnosticAlerts } from '@/components/dashboard'
import { StaggerContainer, StaggerItem, ScaleIn } from '@/components/animation'
import type { InstallStage } from '@/types'

export function Dashboard() {
  const navigate = useNavigate()
  const { addNotification } = useAppStore()
  const { config, agents, currentAgentId, models } = useConfigStore()
  const { progress, setProgress, addLog } = useInstallStore()
  const [autoInstallStarted, setAutoInstallStarted] = useState(false)

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
  const { data: installStatus, isLoading: isInstallLoading, refetch: refetchInstall } = useQuery({
    queryKey: ['openclaw-installation'],
    queryFn: () => openclawApi.checkInstallation(),
    refetchInterval: 30000,
  })

  // 查询 Sidecar 安装状态
  const { data: sidecarStatus, isLoading: isSidecarLoading } = useQuery({
    queryKey: ['sidecar-installation'],
    queryFn: () => sidecarApi.checkSidecarInstallation(),
    enabled: installStatus?.type === 'Installed',
    refetchInterval: 30000,
  })

  const isInstalled = installStatus?.type === 'Installed'

  // 查询服务状态
  useQuery({
    queryKey: ['service-status'],
    queryFn: () => serviceApi.getServiceStatus(),
    refetchInterval: 5000,
    enabled: installStatus?.type === 'Installed',
  })

  // 查询 Sidecar 状态
  const { data: sidecarData, refetch: refetchSidecar } = useQuery({
    queryKey: ['sidecar-status'],
    queryFn: () => sidecarApi.getSidecarInfo(),
    refetchInterval: 10000,
    enabled: isInstalled,
  })

  // 启动 Sidecar mutation
  const startSidecarMutation = useMutation({
    mutationFn: () => sidecarApi.startSidecar(),
    onSuccess: (success) => {
      if (success) {
        addNotification({
          title: 'Sidecar 已启动',
          message: 'Sidecar 服务启动成功',
          type: 'success',
        })
        refetchSidecar()
      } else {
        addNotification({
          title: '启动失败',
          message: '无法启动 Sidecar 服务',
          type: 'error',
        })
      }
    },
    onError: (error) => {
      addNotification({
        title: '启动错误',
        message: String(error),
        type: 'error',
      })
    },
  })

  // 停止 Sidecar mutation
  const stopSidecarMutation = useMutation({
    mutationFn: () => sidecarApi.stopSidecar(),
    onSuccess: (success) => {
      if (success) {
        addNotification({
          title: 'Sidecar 已停止',
          message: 'Sidecar 服务已停止',
          type: 'info',
        })
        refetchSidecar()
      } else {
        addNotification({
          title: '停止失败',
          message: '无法停止 Sidecar 服务',
          type: 'error',
        })
      }
    },
    onError: (error) => {
      addNotification({
        title: '停止错误',
        message: String(error),
        type: 'error',
      })
    },
  })

  // 自动安装 mutation
  const autoInstallMutation = useMutation({
    mutationFn: () => openclawApi.installOneClick(true),
    onMutate: () => {
      addLog('🚀 开始自动初始化...', 'info')
      setAutoInstallStarted(true)
    },
    onSuccess: (result) => {
      if (result.success) {
        addLog(`✅ 初始化完成: ${result.message}`, 'success')
        addNotification({
          title: '准备就绪',
          message: 'OpenClaw 已成功初始化，可以开始使用',
          type: 'success',
        })
        refetchInstall()
      } else {
        const errorMsg = result.message || '未知错误'
        addLog(`❌ 初始化失败: ${errorMsg}`, 'error')
        addNotification({
          title: '初始化失败',
          message: errorMsg,
          type: 'error',
        })
      }
    },
    onError: (error) => {
      addLog(`❌ 初始化错误: ${String(error)}`, 'error')
      addNotification({
        title: '初始化错误',
        message: String(error),
        type: 'error',
      })
    },
  })

  // 监听安装进度
  useEffect(() => {
    let unlisten: (() => void) | undefined

    const setupListener = async () => {
      unlisten = await openclawApi.onInstallProgress((prog) => {
        setProgress({
          stage: prog.stage as InstallStage,
          percentage: prog.percentage,
          message: prog.message,
        })
        addLog(prog.message, 'info')
      })
    }

    setupListener()

    return () => {
      if (unlisten) {
        unlisten()
      }
    }
  }, [addLog, setProgress])

  // 自动安装逻辑
  useEffect(() => {
    if (!isInstalled && !isInstallLoading && !autoInstallStarted && !autoInstallMutation.isPending) {
      // 自动开始安装
      autoInstallMutation.mutate()
    }
  }, [isInstalled, isInstallLoading, autoInstallStarted, autoInstallMutation.isPending])

  useEffect(() => {
    if (installStatus?.type === 'Error') {
      addNotification({
        title: '安装错误',
        message: installStatus.message,
        type: 'error',
      })
    }
  }, [installStatus, addNotification])

  // Sidecar 安装检查：OpenClaw 安装完成后，如果 Sidecar 未安装，跳转到安装向导
  useEffect(() => {
    if (isInstalled && !isSidecarLoading && sidecarStatus?.type === 'NotInstalled') {
      navigate('/install')
    }
  }, [isInstalled, isSidecarLoading, sidecarStatus, navigate])

  // 正在自动安装时显示进度
  if (autoInstallMutation.isPending || (autoInstallStarted && !isInstalled)) {
    return (
      <PageLoader
        title="正在初始化 OpenClaw"
        description="首次启动需要解压运行环境，请稍候..."
      >
        {progress && (
          <div className="w-full max-w-md space-y-3 mt-6">
            <Progress value={progress.percentage} className="h-2" />
            <div className="flex justify-between text-sm text-muted-foreground">
              <span>{progress.stage}</span>
              <span>{Math.round(progress.percentage)}%</span>
            </div>
            <p className="text-sm text-center text-muted-foreground">{progress.message}</p>
          </div>
        )}
      </PageLoader>
    )
  }

  // 安装失败时显示重试选项
  if (autoInstallMutation.isError || (installStatus?.type === 'Error' && autoInstallStarted)) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh] space-y-6">
        <div className="text-center space-y-4">
          <div className="w-20 h-20 mx-auto bg-red-100 rounded-full flex items-center justify-center">
            <XCircle className="w-10 h-10 text-red-500" />
          </div>
          <h1 className="text-3xl font-bold">初始化失败</h1>
          <p className="text-muted-foreground max-w-md">
            OpenClaw 初始化过程中出现问题，请重试或检查日志。
          </p>
        </div>
        <div className="flex gap-4">
          <Button size="lg" onClick={() => autoInstallMutation.mutate()}>
            <Download className="mr-2 h-5 w-5" />
            重试初始化
          </Button>
          <Button size="lg" variant="outline" onClick={() => navigate('/diagnostics')}>
            <Stethoscope className="mr-2 h-5 w-5" />
            运行诊断
          </Button>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <ScaleIn>
        <div>
          <h1 className="text-3xl font-bold">仪表盘</h1>
          <p className="text-muted-foreground">查看 OpenClaw 运行状态</p>
        </div>
      </ScaleIn>

      {/* 状态卡片网格 */}
      <StaggerContainer className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {/* OpenClaw 安装状态 */}
        <StaggerItem>
          <Card className="hover:shadow-md transition-shadow">
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
        </StaggerItem>

        {/* Gateway 服务状态 */}
        <StaggerItem>
          <ServiceStatus />
        </StaggerItem>

        {/* Sidecar 状态 */}
        <StaggerItem>
          <Card className="hover:shadow-md transition-shadow" data-testid="sidecar-card">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Sidecar 状态</CardTitle>
              <Package className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent className="space-y-4">
              {/* 安装状态 */}
              <div className="flex items-center gap-2" data-testid="sidecar-installed">
                {sidecarData ? (
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

              {/* 版本信息 */}
              {sidecarData && (
                <div data-testid="sidecar-version" className="text-xs text-muted-foreground">
                  版本: {sidecarData.version}
                </div>
              )}

              {/* 服务状态 */}
              {sidecarData && (
                <div data-testid="service-status" className="flex items-center gap-2">
                  <Badge variant={sidecarData.isRunning ? "default" : "secondary"}>
                    {sidecarData.isRunning ? '运行中' : '已停止'}
                  </Badge>
                  {sidecarData.pid && (
                    <span className="text-xs text-muted-foreground">PID: {sidecarData.pid}</span>
                  )}
                </div>
              )}

              {/* 操作按钮 */}
              {sidecarData && (
                <div className="flex gap-2">
                  {!sidecarData.isRunning ? (
                    <Button
                      data-testid="start-service"
                      size="sm"
                      className="flex-1"
                      onClick={() => startSidecarMutation.mutate()}
                      disabled={startSidecarMutation.isPending}
                    >
                      {startSidecarMutation.isPending ? (
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      ) : (
                        <Play className="mr-2 h-4 w-4" />
                      )}
                      启动
                    </Button>
                  ) : (
                    <Button
                      data-testid="stop-service"
                      size="sm"
                      variant="destructive"
                      className="flex-1"
                      onClick={() => stopSidecarMutation.mutate()}
                      disabled={stopSidecarMutation.isPending}
                    >
                      {stopSidecarMutation.isPending ? (
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      ) : (
                        <Square className="mr-2 h-4 w-4" />
                      )}
                      停止
                    </Button>
                  )}
                </div>
              )}
            </CardContent>
          </Card>
        </StaggerItem>

        {/* 当前 Agent */}
        <StaggerItem>
          <Card className="hover:shadow-md transition-shadow">
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
        </StaggerItem>

        {/* 默认模型 */}
        <StaggerItem>
          <Card className="hover:shadow-md transition-shadow">
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
        </StaggerItem>

        {/* 快速操作 */}
        <StaggerItem>
          <Card className="hover:shadow-md transition-shadow">
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
        </StaggerItem>
      </StaggerContainer>

      {/* 资源监控和诊断警告 */}
      <ScaleIn delay={0.2}>
        <div className="grid gap-6 lg:grid-cols-2">
          <ResourceMonitor />
          <DiagnosticAlerts />
        </div>
      </ScaleIn>

      {/* 活动日志 */}
      <ScaleIn delay={0.3}>
        <ActivityLog />
      </ScaleIn>

      {/* 系统信息 */}
      <ScaleIn delay={0.4}>
        <Card>
          <CardHeader>
            <CardTitle>系统信息</CardTitle>
            <CardDescription>OpenClaw 配置概览</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div className="p-4 rounded-lg bg-muted/50">
                <p className="text-sm text-muted-foreground">已配置模型</p>
                <p className="text-2xl font-semibold mt-1">{config?.models.length || 0}</p>
              </div>
              <div className="p-4 rounded-lg bg-muted/50">
                <p className="text-sm text-muted-foreground">已创建 Agent</p>
                <p className="text-2xl font-semibold mt-1">{config?.agents.length || 0}</p>
              </div>
              <div className="p-4 rounded-lg bg-muted/50">
                <p className="text-sm text-muted-foreground">已启用技能</p>
                <p className="text-2xl font-semibold mt-1">
                  {config?.skills.filter(s => s.enabled).length || 0}
                </p>
              </div>
              <div className="p-4 rounded-lg bg-muted/50">
                <p className="text-sm text-muted-foreground">配置版本</p>
                <p className="text-2xl font-semibold mt-1">{config?.version || '1.0.0'}</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </ScaleIn>
    </div>
  )
}
