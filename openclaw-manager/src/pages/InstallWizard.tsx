import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  CheckCircle,
  XCircle,
  AlertCircle,
  Download,
  Package,
  Settings,
  ArrowRight,
  ArrowLeft,
  Loader2,
  Terminal,
  Sparkles
} from 'lucide-react'
import { openclawApi, type InstallMethodInfo } from '@/lib/tauri-api'
import { useInstallStore } from '@/stores/installStore'
import { useAppStore } from '@/stores/appStore'
import type { SystemCheckResult, InstallStage, InstallMethod } from '@/types'

const steps = [
  { id: 'check', title: '环境检测', description: '检查系统环境' },
  { id: 'method', title: '选择方式', description: '选择安装方式' },
  { id: 'install', title: '安装', description: '安装 OpenClaw' },
  { id: 'config', title: '初始配置', description: '配置模型' },
]

export function InstallWizard() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { addNotification } = useAppStore()
  const { 
    wizardStep: currentStep, 
    setWizardStep: setCurrentStep, 
    setInstallMethod, 
    installMethod,
    logs,
    addLog,
    progress,
    setProgress,
    reset 
  } = useInstallStore()

  const [systemChecks, setSystemChecks] = useState<SystemCheckResult[]>([])
  const [isChecking, setIsChecking] = useState(false)

  // 获取可用的安装方法
  const { data: installMethodsData } = useQuery({
    queryKey: ['install-methods'],
    queryFn: openclawApi.getInstallMethods,
  })

  const installMethods = installMethodsData?.data || []

  // 检查系统环境
  const checkMutation = useMutation({
    mutationFn: openclawApi.checkSystemEnvironment,
    onMutate: () => {
      setIsChecking(true)
      addLog('开始检查系统环境...', 'info')
    },
    onSuccess: (result) => {
      setIsChecking(false)
      if (result.data) {
        setSystemChecks(result.data.checks)
        result.data.checks.forEach(check => {
          addLog(
            `${check.name}: ${check.passed ? '通过' : '未通过'} - ${check.message}`,
            check.passed ? 'success' : check.required ? 'error' : 'warning'
          )
        })
        
        if (result.data.can_install) {
          addLog('系统环境检查通过，可以继续安装', 'success')
        } else {
          addLog(`缺少必要依赖: ${result.data.missing_dependencies.join(', ')}`, 'error')
        }
      }
    },
  })

  // 在线安装
  const installOnlineMutation = useMutation({
    mutationFn: () => openclawApi.install(undefined, undefined, undefined),
    onSuccess: (result) => {
      if (result.data?.success) {
        addLog(`安装成功: ${result.data.message}`, 'success')
        queryClient.invalidateQueries({ queryKey: ['openclaw-installation'] })
      } else {
        addLog(`安装失败: ${result.data?.message || '未知错误'}`, 'error')
      }
    },
    onError: (error) => {
      addLog(`安装错误: ${String(error)}`, 'error')
    },
  })

  // 离线安装
  const installOfflineMutation = useMutation({
    mutationFn: () => openclawApi.installOffline(),
    onSuccess: (result) => {
      console.log('离线安装结果:', result)
      if (result.data?.success) {
        addLog(`离线安装成功: ${result.data.message}`, 'success')
        queryClient.invalidateQueries({ queryKey: ['openclaw-installation'] })
      } else {
        const errorMsg = result.error || result.data?.message || '未知错误'
        addLog(`离线安装失败: ${errorMsg}`, 'error')
        console.error('离线安装失败详情:', result)
      }
    },
    onError: (error) => {
      addLog(`离线安装错误: ${String(error)}`, 'error')
      console.error('离线安装异常:', error)
    },
  })

  // 一键安装（Molili 风格全栈打包）
  const installOneClickMutation = useMutation({
    mutationFn: () => openclawApi.installOneClick(true),
    onSuccess: (result) => {
      console.log('一键安装结果:', result)
      if (result.data?.success) {
        addLog(`🎉 一键安装成功: ${result.data.message}`, 'success')
        queryClient.invalidateQueries({ queryKey: ['openclaw-installation'] })
        addNotification({
          title: '安装完成',
          message: 'OpenClaw 已成功安装，已配置国产模型支持',
          type: 'success',
        })
      } else {
        const errorMsg = result.error || result.data?.message || '未知错误'
        addLog(`一键安装失败: ${errorMsg}`, 'error')
        console.error('一键安装失败详情:', result)
        addNotification({
          title: '安装失败',
          message: errorMsg,
          type: 'error',
        })
      }
    },
    onError: (error) => {
      addLog(`一键安装错误: ${String(error)}`, 'error')
      console.error('一键安装异常:', error)
      addNotification({
        title: '安装错误',
        message: String(error),
        type: 'error',
      })
    },
  })

  // 监听安装进度
  useEffect(() => {
    let unlisten: (() => void) | undefined

    const setupListener = async () => {
      unlisten = await openclawApi.onInstallProgress((progress) => {
        setProgress({
          stage: progress.stage as InstallStage,
          percentage: progress.percentage,
          message: progress.message,
        })
        addLog(progress.message, 'info')
      })
    }

    setupListener()

    return () => {
      if (unlisten) {
        unlisten()
      }
    }
  }, [addLog, setProgress])

  const handleStartInstall = () => {
    console.log('开始安装，选择的安装方式:', installMethod)

    if (installMethod === 'oneclick') {
      addLog('🚀 开始一键安装...', 'info')
      addLog('正在解压嵌入式运行环境...', 'info')
      installOneClickMutation.mutate()
    } else if (installMethod === 'online') {
      addLog('开始在线安装...', 'info')
      installOnlineMutation.mutate()
    } else if (installMethod === 'offline') {
      addLog('开始离线安装...', 'info')
      installOfflineMutation.mutate()
    } else {
      addLog('错误: 未选择安装方式', 'error')
    }
  }

  const isInstalling = installOnlineMutation.isPending || installOfflineMutation.isPending || installOneClickMutation.isPending
  const isInstallSuccess = installOnlineMutation.isSuccess || installOfflineMutation.isSuccess || installOneClickMutation.isSuccess

  const canProceed = () => {
    switch (currentStep) {
      case 0: // check
        return systemChecks.some(c => c.passed && c.required)
      case 1: // method
        return installMethod !== null
      case 2: // install
        return isInstallSuccess
      default:
        return true
    }
  }

  const handleNext = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1)
    } else {
      // 完成
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
      case 0: // 环境检测
        return (
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-lg font-medium">系统环境检查</h3>
              <Button 
                onClick={() => checkMutation.mutate()} 
                disabled={isChecking}
                variant="outline"
              >
                {isChecking ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    检查中...
                  </>
                ) : (
                  <>
                    <Terminal className="mr-2 h-4 w-4" />
                    开始检查
                  </>
                )}
              </Button>
            </div>

            {systemChecks.length > 0 && (
              <div className="space-y-2">
                {systemChecks.map((check) => (
                  <div 
                    key={check.name}
                    className="flex items-center justify-between p-3 rounded-lg bg-muted/50"
                  >
                    <div className="flex items-center gap-2">
                      {check.passed ? (
                        <CheckCircle className="h-5 w-5 text-green-500" />
                      ) : check.required ? (
                        <XCircle className="h-5 w-5 text-red-500" />
                      ) : (
                        <AlertCircle className="h-5 w-5 text-yellow-500" />
                      )}
                      <span>{check.name}</span>
                      {check.required && (
                        <Badge variant="outline">必需</Badge>
                      )}
                    </div>
                    <span className="text-sm text-muted-foreground">{check.message}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        )

      case 1: // 选择安装方式
        return (
          <div className="space-y-4">
            <h3 className="text-lg font-medium">选择安装方式</h3>

            {/* 一键安装选项（推荐） */}
            <Card
              className={`cursor-pointer transition-colors border-primary bg-primary/5 ${installMethod === 'oneclick' ? 'ring-2 ring-primary' : ''}`}
              onClick={() => setInstallMethod('oneclick' as InstallMethod)}
            >
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <div className="p-2 rounded-full bg-primary/10">
                      <Sparkles className="h-5 w-5 text-primary" />
                    </div>
                    <div>
                      <CardTitle className="text-primary">🚀 一键安装（推荐）</CardTitle>
                      <CardDescription className="mt-1">
                        Molili 风格全栈打包，零依赖开箱即用
                      </CardDescription>
                    </div>
                  </div>
                  <Badge variant="default">推荐</Badge>
                </div>
              </CardHeader>
              <CardContent>
                <ul className="text-sm space-y-2 text-muted-foreground">
                  <li className="flex items-center gap-2">
                    <CheckCircle className="h-4 w-4 text-green-500" />
                    <span>嵌入式 Python 3.10 + Node.js 22，无需手动安装环境</span>
                  </li>
                  <li className="flex items-center gap-2">
                    <CheckCircle className="h-4 w-4 text-green-500" />
                    <span>预配置国产大模型（DeepSeek、MiniMax、智谱等）</span>
                  </li>
                  <li className="flex items-center gap-2">
                    <CheckCircle className="h-4 w-4 text-green-500" />
                    <span>无需网络，2-3 分钟完成安装</span>
                  </li>
                  <li className="flex items-center gap-2">
                    <CheckCircle className="h-4 w-4 text-green-500" />
                    <span>支持离线安装包自动更新</span>
                  </li>
                </ul>
              </CardContent>
            </Card>

            <div className="relative">
              <div className="absolute inset-0 flex items-center">
                <span className="w-full border-t" />
              </div>
              <div className="relative flex justify-center text-xs uppercase">
                <span className="bg-background px-2 text-muted-foreground">
                  高级选项
                </span>
              </div>
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              {installMethods.map((method: InstallMethodInfo) => (
                <Card 
                  key={method.id}
                  className={`cursor-pointer transition-colors ${installMethod === method.id ? 'border-primary' : ''} ${!method.available ? 'opacity-50 cursor-not-allowed' : ''}`}
                  onClick={() => method.available && setInstallMethod(method.id as InstallMethod)}
                >
                  <CardHeader>
                    <div className="flex items-center gap-2">
                      {method.id === 'online' ? (
                        <Download className="h-5 w-5" />
                      ) : (
                        <Package className="h-5 w-5" />
                      )}
                      <CardTitle>{method.name}</CardTitle>
                    </div>
                    <CardDescription>
                      {method.description}
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <ul className="text-sm space-y-1 text-muted-foreground">
                      <li>{method.requires_network ? '✓ 需要网络连接' : '✓ 无需网络连接'}</li>
                      <li>{method.id === 'online' ? '✓ 自动下载最新版本' : '✓ 安装速度快'}</li>
                      <li>{method.id === 'online' ? '✓ 安装包体积较小' : '✓ 应用体积较大'}</li>
                    </ul>
                    {!method.available && (
                      <Badge variant="secondary" className="mt-2">暂不可用</Badge>
                    )}
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        )

      case 2: // 安装
        return (
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-lg font-medium">安装进度</h3>
              {!isInstalling && !isInstallSuccess && (
                <Button onClick={handleStartInstall} size="lg" className={installMethod === 'oneclick' ? 'bg-primary' : ''}>
                  {installMethod === 'oneclick' ? (
                    <>
                      <Sparkles className="mr-2 h-4 w-4" />
                      🚀 开始一键安装
                    </>
                  ) : installMethod === 'offline' ? (
                    <>
                      <Package className="mr-2 h-4 w-4" />
                      开始离线安装
                    </>
                  ) : (
                    <>
                      <Download className="mr-2 h-4 w-4" />
                      开始在线安装
                    </>
                  )}
                </Button>
              )}
            </div>

            {isInstalling && progress && (
              <div className="space-y-2">
                <Progress value={progress.percentage} />
                <div className="flex justify-between text-sm">
                  <span>{progress.stage}</span>
                  <span>{progress.percentage}%</span>
                </div>
              </div>
            )}

            <Card>
              <CardHeader>
                <CardTitle className="text-sm">安装日志</CardTitle>
              </CardHeader>
              <CardContent>
                <ScrollArea className="h-64 w-full rounded-md border p-4">
                  <div className="space-y-1 font-mono text-sm">
                    {logs.length === 0 ? (
                      <p className="text-muted-foreground">等待开始安装...</p>
                    ) : (
                      logs.map((log, index) => (
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
                      ))
                    )}
                  </div>
                </ScrollArea>
              </CardContent>
            </Card>
          </div>
        )

      case 3: // 初始配置
        return (
          <div className="space-y-4">
            <div className="text-center space-y-2">
              <CheckCircle className="h-16 w-16 text-green-500 mx-auto" />
              <h3 className="text-xl font-medium">安装成功！</h3>
              <p className="text-muted-foreground">
                OpenClaw 已成功安装。您可以现在进行初始配置，或稍后在设置中配置。
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
                  <CardTitle>稍后配置</CardTitle>
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
                    完成
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
          disabled={currentStep === 0}
        >
          <ArrowLeft className="mr-2 h-4 w-4" />
          上一步
        </Button>

        {currentStep < 3 && (
          <Button
            onClick={handleNext}
            disabled={!canProceed()}
          >
            下一步
            <ArrowRight className="ml-2 h-4 w-4" />
          </Button>
        )}
      </div>
    </div>
  )
}
