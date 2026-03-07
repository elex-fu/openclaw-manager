import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMutation, useQueryClient, useQuery } from '@tanstack/react-query'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { Separator } from '@/components/ui/separator'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import {
  CheckCircle,
  Settings,
  ArrowRight,
  ArrowLeft,
  Loader2,
  Sparkles,
  Package,
  AlertCircle,
  RefreshCw
} from 'lucide-react'
import { openclawApi, sidecarApi } from '@/lib/tauri-api'
import { useInstallStore } from '@/stores/installStore'
import { useAppStore } from '@/stores/appStore'
import type { InstallStage, InstallProgress } from '@/types'

const steps = [
  { id: 'install', title: '初始化', description: '解压运行环境' },
  { id: 'sidecar', title: 'Sidecar', description: '安装 Sidecar 组件' },
  { id: 'config', title: '初始配置', description: '配置模型' },
]

export function InstallWizard() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { addNotification } = useAppStore()
  const {
    wizardStep: currentStep,
    setWizardStep: setCurrentStep,
    logs,
    addLog,
    progress,
    setProgress,
    reset
  } = useInstallStore()

  // Sidecar 安装状态
  const [sidecarProgress, setSidecarProgress] = useState<InstallProgress | null>(null)
  const [sidecarError, setSidecarError] = useState<string | null>(null)

  // 查询 Sidecar 安装状态
  const { data: sidecarStatus } = useQuery({
    queryKey: ['sidecar-installation'],
    queryFn: () => sidecarApi.checkSidecarInstallation(),
    enabled: currentStep >= 1,
  })

  // Sidecar 安装 mutation
  const sidecarInstallMutation = useMutation({
    mutationFn: () => sidecarApi.installSidecar(),
    onMutate: () => {
      addLog('🚀 开始安装 Sidecar...', 'info')
      setSidecarError(null)
      setSidecarProgress({
        stage: 'Installing',
        percentage: 0,
        message: '准备安装 Sidecar...'
      })
    },
    onSuccess: (result) => {
      if (result.success) {
        addLog(`🎉 Sidecar 安装成功: ${result.message}`, 'success')
        queryClient.invalidateQueries({ queryKey: ['sidecar-installation'] })
        addNotification({
          title: 'Sidecar 安装完成',
          message: 'Sidecar 组件已成功安装',
          type: 'success',
        })
        setCurrentStep(2) // 自动进入配置步骤
      } else {
        const errorMsg = result.message || 'Sidecar 安装失败'
        addLog(`Sidecar 安装失败: ${errorMsg}`, 'error')
        setSidecarError(errorMsg)
        addNotification({
          title: 'Sidecar 安装失败',
          message: errorMsg,
          type: 'error',
        })
      }
    },
    onError: (error) => {
      const errorMsg = String(error)
      addLog(`Sidecar 安装错误: ${errorMsg}`, 'error')
      setSidecarError(errorMsg)
      addNotification({
        title: 'Sidecar 安装错误',
        message: errorMsg,
        type: 'error',
      })
    },
  })

  // 监听 Sidecar 安装进度
  useEffect(() => {
    let unlisten: (() => void) | undefined

    const setupListener = async () => {
      unlisten = await sidecarApi.onSidecarInstallProgress((prog) => {
        setSidecarProgress({
          stage: prog.stage,
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
  }, [addLog])

  // 一键自动安装
  const installOneClickMutation = useMutation({
    mutationFn: () => openclawApi.installOneClick(true),
    onMutate: () => {
      addLog('🚀 开始自动初始化...', 'info')
    },
    onSuccess: (result) => {
      if (result.success) {
        addLog(`🎉 初始化成功: ${result.message}`, 'success')
        queryClient.invalidateQueries({ queryKey: ['openclaw-installation'] })
        addNotification({
          title: '准备就绪',
          message: 'OpenClaw 已成功初始化',
          type: 'success',
        })
        setCurrentStep(1) // 自动进入配置步骤
      } else {
        const errorMsg = result.message || '未知错误'
        addLog(`初始化失败: ${errorMsg}`, 'error')
        addNotification({
          title: '初始化失败',
          message: errorMsg,
          type: 'error',
        })
      }
    },
    onError: (error) => {
      addLog(`初始化错误: ${String(error)}`, 'error')
      addNotification({
        title: '初始化错误',
        message: String(error),
        type: 'error',
      })
    },
  })

  // 自动开始安装
  useEffect(() => {
    if (currentStep === 0 && !installOneClickMutation.isPending && !installOneClickMutation.isSuccess) {
      installOneClickMutation.mutate()
    }
  }, [currentStep])

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

  const handleNext = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1)
    } else {
      addNotification({
        title: '安装完成',
        message: 'OpenClaw 安装成功',
        type: 'success',
      })
      reset()
      navigate('/')
    }
  }

  const handleBack = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1)
    }
  }

  const renderStepContent = () => {
    switch (currentStep) {
      case 0: // 自动初始化
        return (
          <div className="space-y-6">
            <div className="text-center space-y-2">
              <div className="w-16 h-16 mx-auto bg-primary/10 rounded-full flex items-center justify-center">
                <Sparkles className="w-8 h-8 text-primary animate-pulse" />
              </div>
              <h3 className="text-lg font-medium">正在初始化 OpenClaw</h3>
              <p className="text-sm text-muted-foreground">
                首次启动需要解压运行环境，请稍候...
              </p>
            </div>

            {progress && (
              <div className="space-y-2">
                <Progress value={progress.percentage} className="h-2" />
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">{progress.stage}</span>
                  <span>{Math.round(progress.percentage)}%</span>
                </div>
                <p className="text-sm text-center text-muted-foreground">{progress.message}</p>
              </div>
            )}

            {installOneClickMutation.isPending && (
              <div className="flex justify-center">
                <Loader2 className="h-6 w-6 animate-spin text-primary" />
              </div>
            )}

            {logs.length > 0 && (
              <Card>
                <CardHeader>
                  <CardTitle className="text-sm">初始化日志</CardTitle>
                </CardHeader>
                <CardContent>
                  <ScrollArea className="h-48 w-full rounded-md border p-4">
                    <div className="space-y-1 font-mono text-sm">
                      {logs.map((log, index) => (
                        <div key={index} className="flex gap-2">
                          <span className="text-muted-foreground">[{new Date(log.timestamp).toLocaleTimeString()}]</span>
                          <span className={
                            log.level === 'success' ? 'text-green-600' :
                            log.level === 'error' ? 'text-red-600' :
                            log.level === 'warning' ? 'text-yellow-600' :
                            'text-gray-600'
                          }>
                            {log.message}
                          </span>
                        </div>
                      ))}
                    </div>
                  </ScrollArea>
                </CardContent>
              </Card>
            )}
          </div>
        )

      case 1: // Sidecar 安装
        return (
          <div className="space-y-6" data-testid="sidecar-install-step">
            <div className="text-center space-y-2">
              <div className="w-16 h-16 mx-auto bg-primary/10 rounded-full flex items-center justify-center">
                <Package className="w-8 h-8 text-primary" />
              </div>
              <h3 className="text-lg font-medium">安装 Sidecar 组件</h3>
              <p className="text-sm text-muted-foreground">
                Sidecar 是 OpenClaw 的核心运行组件，需要单独安装
              </p>
            </div>

            {/* 未安装状态 */}
            {sidecarStatus?.type === 'NotInstalled' && !sidecarInstallMutation.isPending && !sidecarProgress && (
              <div className="flex justify-center">
                <Button
                  data-testid="sidecar-install"
                  size="lg"
                  onClick={() => sidecarInstallMutation.mutate()}
                  disabled={sidecarInstallMutation.isPending}
                >
                  {sidecarInstallMutation.isPending ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      安装中...
                    </>
                  ) : (
                    <>
                      <Package className="mr-2 h-4 w-4" />
                      安装 Sidecar
                    </>
                  )}
                </Button>
              </div>
            )}

            {/* 安装中状态 */}
            {(sidecarInstallMutation.isPending || sidecarProgress) && (
              <div data-testid="install-progress" className="space-y-4">
                <div className="space-y-2">
                  <Progress value={sidecarProgress?.percentage || 0} className="h-2" />
                  <div className="flex justify-between text-sm">
                    <span data-testid="install-stage" className="text-muted-foreground">
                      {sidecarProgress?.stage || 'Installing'}
                    </span>
                    <span>{Math.round(sidecarProgress?.percentage || 0)}%</span>
                  </div>
                </div>
                <p className="text-sm text-center text-muted-foreground">
                  {sidecarProgress?.message || '正在安装 Sidecar...'}
                </p>
                <div className="flex justify-center">
                  <Loader2 className="h-6 w-6 animate-spin text-primary" />
                </div>
              </div>
            )}

            {/* 安装完成 */}
            {sidecarStatus?.type === 'Installed' && !sidecarInstallMutation.isPending && (
              <div data-testid="install-complete" className="text-center space-y-4">
                <div className="w-20 h-20 mx-auto bg-green-100 rounded-full flex items-center justify-center">
                  <CheckCircle className="h-10 w-10 text-green-600" />
                </div>
                <div className="space-y-1">
                  <h4 className="text-lg font-semibold text-green-600">安装成功！</h4>
                  <p className="text-sm text-muted-foreground">
                    Sidecar 组件已安装完成，版本: {sidecarStatus.version}
                  </p>
                </div>
                <Button
                  size="lg"
                  onClick={() => setCurrentStep(2)}
                  className="mt-4"
                >
                  继续下一步
                  <ArrowRight className="ml-2 h-4 w-4" />
                </Button>
              </div>
            )}

            {/* 错误状态 */}
            {sidecarError && (
              <Alert data-testid="install-error" variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertTitle>安装失败</AlertTitle>
                <AlertDescription>{sidecarError}</AlertDescription>
              </Alert>
            )}

            {/* 重试按钮 */}
            {sidecarError && (
              <div className="flex justify-center">
                <Button
                  data-testid="install-retry"
                  variant="outline"
                  onClick={() => {
                    setSidecarError(null)
                    sidecarInstallMutation.mutate()
                  }}
                >
                  <RefreshCw className="mr-2 h-4 w-4" />
                  重试安装
                </Button>
              </div>
            )}
          </div>
        )

      case 2: // 初始配置
        return (
          <div className="space-y-4">
            <div className="text-center space-y-2">
              <CheckCircle className="h-16 w-16 text-green-500 mx-auto" />
              <h3 className="text-xl font-medium">初始化成功！</h3>
              <p className="text-muted-foreground">
                OpenClaw 和 Sidecar 已成功初始化。您可以现在进行初始配置，或稍后在设置中配置。
              </p>
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              <Card>
                <CardHeader>
                  <CardTitle>配置模型</CardTitle>
                  <CardDescription>设置 AI 模型提供商和 API Key</CardDescription>
                </CardHeader>
                <CardContent>
                  <Button
                    variant="outline"
                    className="w-full"
                    onClick={() => {
                      reset()
                      navigate('/models')
                    }}
                  >
                    <Settings className="mr-2 h-4 w-4" />
                    前往模型配置
                  </Button>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>立即使用</CardTitle>
                  <CardDescription>跳过初始配置，使用默认设置</CardDescription>
                </CardHeader>
                <CardContent>
                  <Button
                    className="w-full"
                    onClick={() => {
                      reset()
                      navigate('/')
                    }}
                  >
                    开始使用
                  </Button>
                </CardContent>
              </Card>
            </div>
          </div>
        )

      default:
        return null
    }
  }

  return (
    <div className="max-w-3xl mx-auto space-y-6">
      <div className="text-center space-y-2">
        <h1 className="text-3xl font-bold">安装 OpenClaw</h1>
        <p className="text-muted-foreground">按照以下步骤完成安装</p>
      </div>

      {/* 步骤指示器 */}
      <div className="flex items-center justify-between">
        {steps.map((step, index) => (
          <div key={step.id} className="flex items-center">
            <div className="flex flex-col items-center">
              <div 
                className={`w-10 h-10 rounded-full flex items-center justify-center text-sm font-medium ${
                  index < currentStep ? 'bg-green-500 text-white' :
                  index === currentStep ? 'bg-primary text-primary-foreground' :
                  'bg-muted text-muted-foreground'
                }`}
              >
                {index < currentStep ? (
                  <CheckCircle className="h-5 w-5" />
                ) : (
                  index + 1
                )}
              </div>
              <div className="mt-2 text-center">
                <p className="text-sm font-medium">{step.title}</p>
                <p className="text-xs text-muted-foreground">{step.description}</p>
              </div>
            </div>
            {index < steps.length - 1 && (
              <div className={`w-24 h-px mx-4 ${
                index < currentStep ? 'bg-green-500' : 'bg-muted'
              }`} />
            )}
          </div>
        ))}
      </div>

      <Separator />

      {/* 步骤内容 */}
      <Card>
        <CardContent className="pt-6">
          {renderStepContent()}
        </CardContent>
      </Card>

      {/* 导航按钮 */}
      <div className="flex justify-between">
        <Button
          variant="outline"
          onClick={handleBack}
          disabled={currentStep === 0 || installOneClickMutation.isPending || sidecarInstallMutation.isPending}
        >
          <ArrowLeft className="mr-2 h-4 w-4" />
          上一步
        </Button>

        {currentStep < steps.length - 1 && (
          <Button
            onClick={handleNext}
            disabled={
              (currentStep === 0 && !installOneClickMutation.isSuccess) ||
              (currentStep === 1 && sidecarStatus?.type !== 'Installed') ||
              sidecarInstallMutation.isPending
            }
          >
            下一步
            <ArrowRight className="ml-2 h-4 w-4" />
          </Button>
        )}
      </div>
    </div>
  )
}
