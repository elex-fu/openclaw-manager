import { useState, useEffect, useCallback, useRef } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { openclawApi, serviceApi } from '@/lib/tauri-api'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'
import { 
  Download, 
  Trash2, 
  Play, 
  Activity, 
  LayoutDashboard, 
  Stethoscope,
  Server,
  CheckCircle,
  XCircle,
  AlertCircle,
  Loader2,
  Terminal,
  RefreshCw
} from 'lucide-react'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { InstallProgress, InstallStatus } from '@/types'

interface SystemCheck {
  name: string
  status: 'pending' | 'checking' | 'success' | 'error' | 'warning'
  message?: string
  required: boolean
}

interface LogEntry {
  timestamp: string
  level: 'info' | 'success' | 'error' | 'warning'
  message: string
}

export function InstallerPanel() {
  const queryClient = useQueryClient()
  const [logs, setLogs] = useState<LogEntry[]>([])
  const [progress, setProgress] = useState<InstallProgress | null>(null)
  const scrollRef = useRef<HTMLDivElement>(null)
  const [systemChecks, setSystemChecks] = useState<SystemCheck[]>([
    { name: '操作系统兼容性', status: 'pending', required: true },
    { name: '网络连接', status: 'pending', required: true },
    { name: 'Rust 环境', status: 'pending', required: true },
    { name: 'Node.js 环境', status: 'pending', required: false },
    { name: 'Python 环境', status: 'pending', required: false },
    { name: '磁盘空间', status: 'pending', required: true },
  ])

  const { data: statusData, isLoading: isStatusLoading } = useQuery({
    queryKey: ['openclaw-status'],
    queryFn: () => openclawApi.checkInstallation(),
  })

  const status = statusData?.data
  const isInstalled = status?.type === 'Installed'
  const isInstalling = status?.type === 'Installing'

  // Listen to install progress events
  useEffect(() => {
    let unlisten: UnlistenFn | null = null
    
    const setupListener = async () => {
      unlisten = await listen<InstallProgress>('install-progress', (event) => {
        setProgress(event.payload)
        addLog(event.payload.message, 
          event.payload.percentage === 100 ? 'success' : 
          event.payload.stage === 'Error' ? 'error' : 'info'
        )
      })
    }
    
    setupListener()
    
    return () => {
      if (unlisten) unlisten()
    }
  }, [])

  // Auto-scroll logs
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [logs])

  const addLog = useCallback((message: string, level: LogEntry['level'] = 'info') => {
    setLogs(prev => [...prev, {
      timestamp: new Date().toLocaleTimeString(),
      level,
      message
    }])
  }, [])

  // Check system environment
  const checkSystemMutation = useMutation({
    mutationFn: async () => {
      addLog('开始检查系统环境...', 'info')
      setSystemChecks(prev => prev.map(c => ({ ...c, status: 'checking' as const })))
      
      const results = await openclawApi.checkSystemEnvironment()
      
      setSystemChecks(prev => prev.map((check, index) => {
        const result = results.data?.checks?.[index]
        if (result) {
          addLog(`${check.name}: ${result.passed ? '通过' : '未通过'} - ${result.message}`, 
            result.passed ? 'success' : result.required ? 'error' : 'warning')
          return { ...check, status: result.passed ? 'success' : result.required ? 'error' : 'warning', message: result.message }
        }
        return check
      }))
      
      return results
    }
  })

  const installMutation = useMutation({
    mutationFn: async () => {
      addLog('开始安装 OpenClaw...', 'info')
      setLogs([])
      const result = await openclawApi.install()
      return result
    },
    onSuccess: (data) => {
      if (data.success) {
        addLog('安装完成！', 'success')
        queryClient.invalidateQueries({ queryKey: ['openclaw-status'] })
      } else {
        addLog(`安装失败: ${data.error}`, 'error')
      }
    },
    onError: (error: Error) => {
      addLog(`安装错误: ${error.message}`, 'error')
    }
  })

  const uninstallMutation = useMutation({
    mutationFn: openclawApi.uninstall,
    onSuccess: () => {
      addLog('卸载完成', 'info')
      queryClient.invalidateQueries({ queryKey: ['openclaw-status'] })
    }
  })

  const startServiceMutation = useMutation({
    mutationFn: serviceApi.startService,
    onSuccess: (data) => {
      addLog(data.success ? '服务启动成功' : `启动失败: ${data.error || '未知错误'}`, data.success ? 'success' : 'error')
    }
  })

  const commandMutation = useMutation({
    mutationFn: async ({ command, args }: { command: string, args?: string[] }) => {
      addLog(`执行: openclaw ${command} ${args?.join(' ') || ''}`, 'info')
      const result = await openclawApi.executeCommand(command, args)
      return result
    },
    onSuccess: (data) => {
      if (data.success) {
        addLog(data.data || '命令执行成功', 'success')
      } else {
        addLog(data.error || '命令执行失败', 'error')
      }
    }
  })

  const getStatusIcon = (status: SystemCheck['status']) => {
    switch (status) {
      case 'success': return <CheckCircle className="h-4 w-4 text-green-500" />
      case 'error': return <XCircle className="h-4 w-4 text-red-500" />
      case 'warning': return <AlertCircle className="h-4 w-4 text-yellow-500" />
      case 'checking': return <Loader2 className="h-4 w-4 animate-spin" />
      default: return <AlertCircle className="h-4 w-4 text-gray-400" />
    }
  }

  const getLogColor = (level: LogEntry['level']) => {
    switch (level) {
      case 'success': return 'text-green-600'
      case 'error': return 'text-red-600'
      case 'warning': return 'text-yellow-600'
      default: return 'text-gray-600'
    }
  }

  if (isStatusLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader2 className="h-8 w-8 animate-spin" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Installation Status */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Server className="h-5 w-5" />
                OpenClaw 状态
              </CardTitle>
              <CardDescription>
                {isInstalled 
                  ? `已安装 (版本: ${(status as Extract<InstallStatus, { type: 'Installed' }>).version})`
                  : 'OpenClaw 未安装'
                }
              </CardDescription>
            </div>
            <Badge variant={isInstalled ? 'default' : 'secondary'}>
              {isInstalled ? '已安装' : isInstalling ? '安装中' : '未安装'}
            </Badge>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* System Checks */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <h4 className="text-sm font-medium">系统环境检查</h4>
              <Button 
                variant="outline" 
                size="sm" 
                onClick={() => checkSystemMutation.mutate()}
                disabled={checkSystemMutation.isPending}
              >
                {checkSystemMutation.isPending ? (
                  <Loader2 className="h-4 w-4 animate-spin mr-2" />
                ) : (
                  <RefreshCw className="h-4 w-4 mr-2" />
                )}
                检查环境
              </Button>
            </div>
            <div className="grid gap-2">
              {systemChecks.map((check) => (
                <div 
                  key={check.name}
                  className="flex items-center justify-between p-2 rounded-lg bg-muted/50"
                >
                  <div className="flex items-center gap-2">
                    {getStatusIcon(check.status)}
                    <span className="text-sm">{check.name}</span>
                    {check.required && (
                      <Badge variant="outline" className="text-xs">必需</Badge>
                    )}
                  </div>
                  {check.message && (
                    <span className="text-xs text-muted-foreground">{check.message}</span>
                  )}
                </div>
              ))}
            </div>
          </div>

          {/* Progress Bar */}
          {(isInstalling || progress) && (
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span>{progress?.stage || '准备中'}</span>
                <span>{progress?.percentage || 0}%</span>
              </div>
              <Progress value={progress?.percentage || 0} />
            </div>
          )}

          {/* Action Buttons */}
          <div className="flex flex-wrap gap-2">
            {!isInstalled && !isInstalling && (
              <Button 
                onClick={() => installMutation.mutate()}
                disabled={installMutation.isPending || checkSystemMutation.isPending}
              >
                {installMutation.isPending ? (
                  <Loader2 className="h-4 w-4 animate-spin mr-2" />
                ) : (
                  <Download className="h-4 w-4 mr-2" />
                )}
                安装 OpenClaw
              </Button>
            )}
            
            {isInstalled && (
              <>
                <Button 
                  variant="outline"
                  onClick={() => startServiceMutation.mutate()}
                  disabled={startServiceMutation.isPending}
                >
                  <Play className="h-4 w-4 mr-2" />
                  启动服务
                </Button>
                <Button 
                  variant="outline"
                  onClick={() => commandMutation.mutate({ command: 'gateway', args: ['status'] })}
                  disabled={commandMutation.isPending}
                >
                  <Activity className="h-4 w-4 mr-2" />
                  Gateway 状态
                </Button>
                <Button 
                  variant="outline"
                  onClick={() => commandMutation.mutate({ command: 'gateway', args: ['start'] })}
                  disabled={commandMutation.isPending}
                >
                  <Server className="h-4 w-4 mr-2" />
                  启动 Gateway
                </Button>
                <Button 
                  variant="outline"
                  onClick={() => commandMutation.mutate({ command: 'dashboard' })}
                  disabled={commandMutation.isPending}
                >
                  <LayoutDashboard className="h-4 w-4 mr-2" />
                  控制面板
                </Button>
                <Button 
                  variant="outline"
                  onClick={() => commandMutation.mutate({ command: 'doctor' })}
                  disabled={commandMutation.isPending}
                >
                  <Stethoscope className="h-4 w-4 mr-2" />
                  健康检查
                </Button>
                <Button 
                  variant="destructive"
                  onClick={() => uninstallMutation.mutate()}
                  disabled={uninstallMutation.isPending}
                >
                  <Trash2 className="h-4 w-4 mr-2" />
                  卸载
                </Button>
              </>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Logs */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Terminal className="h-5 w-5" />
            安装日志
          </CardTitle>
        </CardHeader>
        <CardContent>
          <ScrollArea className="h-64 w-full rounded-md border p-4" ref={scrollRef}>
            <div className="space-y-1 font-mono text-sm">
              {logs.length === 0 ? (
                <p className="text-muted-foreground">暂无日志...</p>
              ) : (
                logs.map((log, index) => (
                  <div key={index} className="flex gap-2">
                    <span className="text-muted-foreground">[{log.timestamp}]</span>
                    <span className={getLogColor(log.level)}>{log.message}</span>
                  </div>
                ))
              )}
            </div>
          </ScrollArea>
        </CardContent>
      </Card>
    </div>
  )
}
