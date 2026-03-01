import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Separator } from '@/components/ui/separator'
import { Progress } from '@/components/ui/progress'
import { 
  Stethoscope, 
  Play, 
  Wrench, 
  CheckCircle, 
  AlertCircle, 
  AlertTriangle, 
  XCircle,
  Loader2,
  ChevronDown,
  ChevronUp
} from 'lucide-react'
import { diagnosticsApi } from '@/lib/tauri-api'
import { useAppStore } from '@/stores/appStore'
import type { DiagnosticIssue, DiagnosticSeverity } from '@/types'

const severityConfig: Record<DiagnosticSeverity, { icon: React.ReactNode; color: string; label: string }> = {
  info: { icon: <CheckCircle className="h-4 w-4" />, color: 'text-blue-500', label: '信息' },
  warning: { icon: <AlertTriangle className="h-4 w-4" />, color: 'text-yellow-500', label: '警告' },
  error: { icon: <AlertCircle className="h-4 w-4" />, color: 'text-red-500', label: '错误' },
  critical: { icon: <XCircle className="h-4 w-4" />, color: 'text-red-700', label: '严重' },
}

export function Diagnostics() {
  const queryClient = useQueryClient()
  const { addNotification } = useAppStore()
  const [expandedIssues, setExpandedIssues] = useState<Set<string>>(new Set())

  // 查询诊断结果
  const { data: diagnosticData, isLoading, refetch } = useQuery({
    queryKey: ['diagnostics'],
    queryFn: () => diagnosticsApi.runDiagnostics(),
    enabled: false, // 手动触发
  })

  const diagnosticResult = diagnosticData?.data
  const issues = diagnosticResult?.issues || []

  // 运行诊断
  const runDiagnostics = () => {
    refetch()
  }

  // 自动修复
  const fixMutation = useMutation({
    mutationFn: (issue: DiagnosticIssue) => diagnosticsApi.fixIssue(issue),
    onSuccess: (result, issue) => {
      if (result.data) {
        addNotification({
          
          title: '修复成功',
          message: `${issue.name} 已修复`,
          type: 'success',
          
        })
        queryClient.invalidateQueries({ queryKey: ['diagnostics'] })
      } else {
        addNotification({
          
          title: '修复失败',
          message: `${issue.name} 修复失败`,
          type: 'error',
          
        })
      }
    },
  })

  // 一键修复所有可修复问题
  const autoFixMutation = useMutation({
    mutationFn: () => {
      const fixableIssues = issues.filter(i => i.canAutoFix && !i.fixed)
      return diagnosticsApi.autoFix(fixableIssues.map(i => i.id))
    },
    onSuccess: (result) => {
      const { fixed, failed } = result.data || { fixed: [], failed: [] }
      
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
  })

  const toggleExpand = (issueId: string) => {
    const newExpanded = new Set(expandedIssues)
    if (newExpanded.has(issueId)) {
      newExpanded.delete(issueId)
    } else {
      newExpanded.add(issueId)
    }
    setExpandedIssues(newExpanded)
  }

  const hasFixableIssues = issues.some(i => i.canAutoFix && !i.fixed)
  const hasErrors = issues.some(i => i.severity === 'error' || i.severity === 'critical')
  const hasWarnings = issues.some(i => i.severity === 'warning')

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">诊断修复</h1>
          <p className="text-muted-foreground">检查系统环境并自动修复问题</p>
        </div>
        <div className="flex gap-2">
          {hasFixableIssues && (
            <Button 
              onClick={() => autoFixMutation.mutate()}
              disabled={autoFixMutation.isPending}
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
            ) : (
              <Stethoscope className="mr-2 h-4 w-4" />
            )}
            运行诊断
          </Button>
        </div>
      </div>

      {/* 诊断结果摘要 */}
      {diagnosticResult && (
        <>
          {hasErrors ? (
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertTitle>发现问题</AlertTitle>
              <AlertDescription>
                诊断发现 {issues.filter(i => i.severity === 'error' || i.severity === 'critical').length} 个错误，建议立即修复。
              </AlertDescription>
            </Alert>
          ) : hasWarnings ? (
            <Alert className="bg-yellow-50 border-yellow-200">
              <AlertTriangle className="h-4 w-4 text-yellow-600" />
              <AlertTitle className="text-yellow-800">存在警告</AlertTitle>
              <AlertDescription className="text-yellow-700">
                诊断发现 {issues.filter(i => i.severity === 'warning').length} 个警告，建议查看详情。
              </AlertDescription>
            </Alert>
          ) : issues.length > 0 ? (
            <Alert className="bg-green-50 border-green-200">
              <CheckCircle className="h-4 w-4 text-green-600" />
              <AlertTitle className="text-green-800">系统正常</AlertTitle>
              <AlertDescription className="text-green-700">
                未发现严重问题，系统运行正常。
              </AlertDescription>
            </Alert>
          ) : null}
        </>
      )}

      {/* 问题列表 */}
      {!diagnosticResult ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-16">
            <Stethoscope className="h-16 w-16 text-muted-foreground mb-4" />
            <p className="text-muted-foreground text-lg">点击"运行诊断"开始检查系统</p>
            <Button className="mt-6" onClick={runDiagnostics} disabled={isLoading}>
              {isLoading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  诊断中...
                </>
              ) : (
                <>
                  <Play className="mr-2 h-4 w-4" />
                  开始诊断
                </>
              )}
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          {issues.length === 0 ? (
            <Card>
              <CardContent className="flex flex-col items-center justify-center py-12">
                <CheckCircle className="h-12 w-12 text-green-500 mb-4" />
                <p className="text-muted-foreground">未发现任何问题！</p>
              </CardContent>
            </Card>
          ) : (
            issues.map((issue) => {
              const config = severityConfig[issue.severity]
              const isExpanded = expandedIssues.has(issue.id)

              return (
                <Card key={issue.id} className={issue.fixed ? 'opacity-60' : ''}>
                  <CardHeader className="pb-3">
                    <div className="flex items-start justify-between">
                      <div className="flex items-center gap-3">
                        <div className={config.color}>{config.icon}</div>
                        <div>
                          <div className="flex items-center gap-2">
                            <CardTitle className="text-base">{issue.name}</CardTitle>
                            <Badge variant={issue.severity === 'critical' ? 'destructive' : 'secondary'}>
                              {config.label}
                            </Badge>
                            {issue.fixed && (
                              <Badge variant="outline" className="text-green-600">
                                <CheckCircle className="mr-1 h-3 w-3" />
                                已修复
                              </Badge>
                            )}
                          </div>
                          <CardDescription>{issue.description}</CardDescription>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        {issue.canAutoFix && !issue.fixed && (
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() => fixMutation.mutate(issue)}
                            disabled={fixMutation.isPending}
                          >
                            {fixMutation.isPending ? (
                              <Loader2 className="h-4 w-4 animate-spin" />
                            ) : (
                              <>
                                <Wrench className="mr-1 h-3 w-3" />
                                修复
                              </>
                            )}
                          </Button>
                        )}
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => toggleExpand(issue.id)}
                        >
                          {isExpanded ? (
                            <ChevronUp className="h-4 w-4" />
                          ) : (
                            <ChevronDown className="h-4 w-4" />
                          )}
                        </Button>
                      </div>
                    </div>
                  </CardHeader>

                  {isExpanded && (
                    <>
                      <Separator />
                      <CardContent className="pt-4">
                        <div className="space-y-2 text-sm">
                          <div>
                            <span className="text-muted-foreground">问题类别: </span>
                            <span className="capitalize">{issue.category}</span>
                          </div>
                          {issue.fixed && issue.fixMessage && (
                            <div>
                              <span className="text-muted-foreground">修复信息: </span>
                              <span className="text-green-600">{issue.fixMessage}</span>
                            </div>
                          )}
                          {issue.error && (
                            <div>
                              <span className="text-muted-foreground">错误详情: </span>
                              <span className="text-red-600">{issue.error}</span>
                            </div>
                          )}
                        </div>
                      </CardContent>
                    </>
                  )}
                </Card>
              )
            })
          )}
        </div>
      )}
    </div>
  )
}
