import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import { Progress } from '@/components/ui/progress'
import {
  Stethoscope,
  Play,
  Wrench,
  CheckCircle,
  AlertTriangle,
  XCircle,
  Loader2,
  ChevronDown,
  ChevronUp,
  Monitor,
  Settings,
  Globe,
  Activity,
  Cpu,
  HardDrive,
  MemoryStick,
  Server,
  FileJson,
  Network,
  RefreshCw
} from 'lucide-react'
import { diagnosticsApi } from '@/lib/tauri-api'
import { useAppStore } from '@/stores/appStore'
import type { DiagnosticCheck, CheckStatus } from '@/types'

// 状态配置
const statusConfig: Record<CheckStatus, { icon: React.ReactNode; color: string; bgColor: string; label: string }> = {
  pass: { icon: <CheckCircle className="h-4 w-4" />, color: 'text-green-500', bgColor: 'bg-green-50', label: '通过' },
  warning: { icon: <AlertTriangle className="h-4 w-4" />, color: 'text-yellow-500', bgColor: 'bg-yellow-50', label: '警告' },
  error: { icon: <XCircle className="h-4 w-4" />, color: 'text-red-500', bgColor: 'bg-red-50', label: '错误' },
}

// 类别配置
const categoryConfig: Record<string, { icon: React.ReactNode; label: string; description: string }> = {
  system: {
    icon: <Monitor className="h-5 w-5" />,
    label: '系统环境',
    description: '操作系统、内存、磁盘空间和必要依赖检查'
  },
  openclaw: {
    icon: <Settings className="h-5 w-5" />,
    label: 'OpenClaw 环境',
    description: '安装状态、配置有效性和端口占用检查'
  },
  network: {
    icon: <Globe className="h-5 w-5" />,
    label: '网络连通性',
    description: '互联网连接和 API 可访问性检查'
  },
  service: {
    icon: <Activity className="h-5 w-5" />,
    label: '服务健康',
    description: 'OpenClaw 进程状态和 HTTP 健康检查'
  },
}

// 检查项图标映射
const checkIconMap: Record<string, React.ReactNode> = {
  '操作系统兼容性': <Monitor className="h-4 w-4" />,
  '内存检查': <MemoryStick className="h-4 w-4" />,
  '磁盘空间': <HardDrive className="h-4 w-4" />,
  'Node.js 环境': <FileJson className="h-4 w-4" />,
  'Python 环境': <FileJson className="h-4 w-4" />,
  'Git 环境': <FileJson className="h-4 w-4" />,
  '安装状态': <Server className="h-4 w-4" />,
  '配置文件有效性': <FileJson className="h-4 w-4" />,
  '端口 8080 检查': <Network className="h-4 w-4" />,
  '端口 3000 检查': <Network className="h-4 w-4" />,
  '端口 11434 检查': <Network className="h-4 w-4" />,
  '互联网连接': <Globe className="h-4 w-4" />,
  'OpenAI API 连通性': <Globe className="h-4 w-4" />,
  'DeepSeek API 连通性': <Globe className="h-4 w-4" />,
  'OpenClaw 进程状态': <Cpu className="h-4 w-4" />,
  'HTTP 健康检查': <Activity className="h-4 w-4" />,
}

export function Diagnostics() {
  const queryClient = useQueryClient()
  const { addNotification } = useAppStore()
  const [expandedChecks, setExpandedChecks] = useState<Set<string>>(new Set())

  // 查询诊断结果
  const { data: diagnosticData, isLoading, refetch } = useQuery({
    queryKey: ['diagnostics'],
    queryFn: async () => {
      const response = await diagnosticsApi.runDiagnostics()
      if (!response.success) {
        throw new Error(response.error || '诊断失败')
      }
      return response.data
    },
    enabled: false, // 手动触发
  })

  const checks = diagnosticData?.checks || []

  // 运行诊断
  const runDiagnostics = () => {
    refetch()
  }

  // 自动修复单个问题
  const fixMutation = useMutation({
    mutationFn: async (checkName: string) => {
      const response = await diagnosticsApi.fixIssue(checkName)
      if (!response.success) {
        throw new Error(response.error || '修复失败')
      }
      return response.data
    },
    onSuccess: (result, checkName) => {
      if (result) {
        addNotification({
          title: '修复成功',
          message: `${checkName} 已修复`,
          type: 'success',
        })
        queryClient.invalidateQueries({ queryKey: ['diagnostics'] })
      } else {
        addNotification({
          title: '修复失败',
          message: `${checkName} 修复失败`,
          type: 'error',
        })
      }
    },
    onError: (error, checkName) => {
      addNotification({
        title: '修复失败',
        message: `${checkName}: ${error instanceof Error ? error.message : '未知错误'}`,
        type: 'error',
      })
    },
  })

  // 一键修复所有可修复问题
  const autoFixMutation = useMutation({
    mutationFn: async () => {
      const fixableChecks = checks.filter(c => c.fixable && c.status !== 'pass')
      if (fixableChecks.length === 0) return { fixed: [], failed: [] }
      const response = await diagnosticsApi.autoFix(fixableChecks.map(c => c.name))
      if (!response.success) {
        throw new Error(response.error || '批量修复失败')
      }
      return response.data
    },
    onSuccess: (result) => {
      const { fixed, failed } = result || { fixed: [], failed: [] }

      if (fixed.length > 0) {
        addNotification({
          title: '批量修复完成',
          message: `成功修复 ${fixed.length} 个问题`,
          type: 'success',
        })
      }

      if (failed.length > 0) {
        addNotification({
          title: '部分修复失败',
          message: `${failed.length} 个问题修复失败`,
          type: 'warning',
        })
      }

      queryClient.invalidateQueries({ queryKey: ['diagnostics'] })
    },
    onError: (error) => {
      addNotification({
        title: '批量修复失败',
        message: error instanceof Error ? error.message : '未知错误',
        type: 'error',
      })
    },
  })

  const toggleExpand = (checkName: string) => {
    const newExpanded = new Set(expandedChecks)
    if (newExpanded.has(checkName)) {
      newExpanded.delete(checkName)
    } else {
      newExpanded.add(checkName)
    }
    setExpandedChecks(newExpanded)
  }

  // 按类别分组检查项
  const groupedChecks = checks.reduce((acc, check) => {
    if (!acc[check.category]) {
      acc[check.category] = []
    }
    acc[check.category].push(check)
    return acc
  }, {} as Record<string, DiagnosticCheck[]>)

  const hasFixableIssues = checks.some(c => c.fixable && c.status !== 'pass')
  const hasErrors = checks.some(c => c.status === 'error')
  const hasWarnings = checks.some(c => c.status === 'warning')
  const passCount = checks.filter(c => c.status === 'pass').length
  const totalCount = checks.length
  const progressPercent = totalCount > 0 ? (passCount / totalCount) * 100 : 0

  // 获取检查项图标
  const getCheckIcon = (checkName: string) => {
    return checkIconMap[checkName] || <Activity className="h-4 w-4" />
  }

  return (
    <div className="space-y-6">
      {/* 头部 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">系统诊断</h1>
          <p className="text-muted-foreground">检查系统环境并自动修复问题</p>
        </div>
        <div className="flex gap-2">
          {diagnosticData && hasFixableIssues && (
            <Button
              onClick={() => autoFixMutation.mutate()}
              disabled={autoFixMutation.isPending || isLoading}
              variant="secondary"
            >
              {autoFixMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Wrench className="mr-2 h-4 w-4" />
              )}
              一键修复
            </Button>
          )}
          <Button onClick={runDiagnostics} disabled={isLoading}>
            {isLoading ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : diagnosticData ? (
              <RefreshCw className="mr-2 h-4 w-4" />
            ) : (
              <Stethoscope className="mr-2 h-4 w-4" />
            )}
            {isLoading ? '诊断中...' : diagnosticData ? '重新诊断' : '运行诊断'}
          </Button>
        </div>
      </div>

      {/* 诊断结果摘要 */}
      {diagnosticData && (
        <>
          {/* 总体状态卡片 */}
          <Card className={hasErrors ? 'border-red-200' : hasWarnings ? 'border-yellow-200' : 'border-green-200'}>
            <CardContent className="pt-6">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-3">
                  {hasErrors ? (
                    <div className="p-2 bg-red-100 rounded-full">
                      <XCircle className="h-6 w-6 text-red-600" />
                    </div>
                  ) : hasWarnings ? (
                    <div className="p-2 bg-yellow-100 rounded-full">
                      <AlertTriangle className="h-6 w-6 text-yellow-600" />
                    </div>
                  ) : (
                    <div className="p-2 bg-green-100 rounded-full">
                      <CheckCircle className="h-6 w-6 text-green-600" />
                    </div>
                  )}
                  <div>
                    <h3 className="font-semibold text-lg">
                      {hasErrors ? '发现问题' : hasWarnings ? '存在警告' : '系统正常'}
                    </h3>
                    <p className="text-muted-foreground text-sm">
                      {hasErrors
                        ? `发现 ${checks.filter(c => c.status === 'error').length} 个错误，建议立即修复`
                        : hasWarnings
                        ? `发现 ${checks.filter(c => c.status === 'warning').length} 个警告，建议查看详情`
                        : '所有检查项均通过，系统运行正常'}
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-2xl font-bold">
                    {passCount}/{totalCount}
                  </div>
                  <div className="text-sm text-muted-foreground">检查通过</div>
                </div>
              </div>
              <Progress value={progressPercent} className="h-2" />
              <div className="flex gap-4 mt-4 text-sm">
                <div className="flex items-center gap-1">
                  <div className="w-3 h-3 rounded-full bg-green-500" />
                  <span>通过 {passCount}</span>
                </div>
                <div className="flex items-center gap-1">
                  <div className="w-3 h-3 rounded-full bg-yellow-500" />
                  <span>警告 {checks.filter(c => c.status === 'warning').length}</span>
                </div>
                <div className="flex items-center gap-1">
                  <div className="w-3 h-3 rounded-full bg-red-500" />
                  <span>错误 {checks.filter(c => c.status === 'error').length}</span>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* 分类检查详情 */}
          <div className="space-y-4">
            {Object.entries(groupedChecks).map(([category, categoryChecks]) => {
              const config = categoryConfig[category] || {
                icon: <Activity className="h-5 w-5" />,
                label: category,
                description: ''
              }
              const categoryPassCount = categoryChecks.filter(c => c.status === 'pass').length
              const categoryHasError = categoryChecks.some(c => c.status === 'error')
              const categoryHasWarning = categoryChecks.some(c => c.status === 'warning')

              return (
                <Card key={category}>
                  <CardHeader>
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <div className={`p-2 rounded-lg ${
                          categoryHasError ? 'bg-red-100 text-red-600' :
                          categoryHasWarning ? 'bg-yellow-100 text-yellow-600' :
                          'bg-green-100 text-green-600'
                        }`}>
                          {config.icon}
                        </div>
                        <div>
                          <CardTitle className="text-lg">{config.label}</CardTitle>
                          <CardDescription>{config.description}</CardDescription>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className={`text-sm font-medium ${
                          categoryHasError ? 'text-red-600' :
                          categoryHasWarning ? 'text-yellow-600' :
                          'text-green-600'
                        }`}>
                          {categoryPassCount}/{categoryChecks.length}
                        </span>
                      </div>
                    </div>
                  </CardHeader>
                  <CardContent className="space-y-2">
                    {categoryChecks.map((check) => {
                      const status = statusConfig[check.status]
                      const isExpanded = expandedChecks.has(check.name)

                      return (
                        <div
                          key={check.name}
                          className={`border rounded-lg transition-colors ${
                            isExpanded ? 'bg-muted/50' : 'hover:bg-muted/30'
                          }`}
                        >
                          <div
                            className="flex items-center justify-between p-3 cursor-pointer"
                            onClick={() => toggleExpand(check.name)}
                          >
                            <div className="flex items-center gap-3">
                              <div className={`p-1.5 rounded ${status.bgColor} ${status.color}`}>
                                {getCheckIcon(check.name)}
                              </div>
                              <div>
                                <div className="flex items-center gap-2">
                                  <span className="font-medium">{check.name}</span>
                                  <Badge variant="outline" className={status.color}>
                                    {status.label}
                                  </Badge>
                                </div>
                                <p className="text-sm text-muted-foreground">{check.message}</p>
                              </div>
                            </div>
                            <div className="flex items-center gap-2">
                              {check.fixable && check.status !== 'pass' && (
                                <Button
                                  size="sm"
                                  variant="outline"
                                  onClick={(e) => {
                                    e.stopPropagation()
                                    fixMutation.mutate(check.name)
                                  }}
                                  disabled={fixMutation.isPending}
                                >
                                  {fixMutation.isPending && fixMutation.variables === check.name ? (
                                    <Loader2 className="h-3 w-3 animate-spin" />
                                  ) : (
                                    <Wrench className="h-3 w-3" />
                                  )}
                                  <span className="ml-1">修复</span>
                                </Button>
                              )}
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={(e) => {
                                  e.stopPropagation()
                                  toggleExpand(check.name)
                                }}
                              >
                                {isExpanded ? (
                                  <ChevronUp className="h-4 w-4" />
                                ) : (
                                  <ChevronDown className="h-4 w-4" />
                                )}
                              </Button>
                            </div>
                          </div>

                          {isExpanded && (
                            <>
                              <Separator />
                              <div className="p-3 space-y-2 text-sm">
                                {check.details && (
                                  <div>
                                    <span className="text-muted-foreground">详细信息: </span>
                                    <span>{check.details}</span>
                                  </div>
                                )}
                                {check.fixSuggestion && (
                                  <div>
                                    <span className="text-muted-foreground">修复建议: </span>
                                    <span className="text-blue-600">{check.fixSuggestion}</span>
                                  </div>
                                )}
                              </div>
                            </>
                          )}
                        </div>
                      )
                    })}
                  </CardContent>
                </Card>
              )
            })}
          </div>
        </>
      )}

      {/* 空状态 */}
      {!diagnosticData && !isLoading && (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-16">
            <Stethoscope className="h-16 w-16 text-muted-foreground mb-4" />
            <p className="text-muted-foreground text-lg mb-2">尚未运行诊断</p>
            <p className="text-muted-foreground text-sm mb-6">点击"运行诊断"开始检查系统环境</p>
            <Button onClick={runDiagnostics}>
              <Play className="mr-2 h-4 w-4" />
              开始诊断
            </Button>
          </CardContent>
        </Card>
      )}

      {/* 加载状态 */}
      {isLoading && (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-16">
            <Loader2 className="h-16 w-16 text-primary animate-spin mb-4" />
            <p className="text-muted-foreground text-lg">正在诊断系统...</p>
            <p className="text-muted-foreground text-sm mt-2">请稍候，这可能需要几秒钟</p>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
