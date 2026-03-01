import { useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { diagnosticsApi } from '@/lib/tauri-api';
import { useAppStore } from '@/stores';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { ScrollArea } from '@/components/ui/scroll-area';
import { 
  Stethoscope, 
  Play, 
  Loader2, 
  CheckCircle, 
  XCircle, 
  AlertCircle, 
  Info,
  Wrench,
  AlertTriangle,
  ShieldAlert,
  RefreshCw
} from 'lucide-react';
import type { DiagnosticIssue, DiagnosticSeverity } from '@/types';

interface DiagnosticPanelProps {
  className?: string;
}

export function DiagnosticPanel({ className }: DiagnosticPanelProps) {
  const { addNotification } = useAppStore();
  const [fixingIssues, setFixingIssues] = useState<Set<string>>(new Set());

  const { data: diagnosticData, refetch, isLoading } = useQuery({
    queryKey: ['diagnostics'],
    queryFn: () => diagnosticsApi.runDiagnostics(),
    enabled: false, // 初始不自动运行
  });

  const result = diagnosticData?.data;

  const runMutation = useMutation({
    mutationFn: () => diagnosticsApi.runDiagnostics(),
  });

  const autoFixMutation = useMutation({
    mutationFn: async (issueIds: string[]) => {
      setFixingIssues(new Set(issueIds));
      const result = await diagnosticsApi.autoFix(issueIds);
      return result;
    },
    onSuccess: (data) => {
      if (data.success) {
        addNotification({
          title: '自动修复完成',
          message: `成功修复 ${data.data?.fixed.length || 0} 个问题`,
          type: 'success',
        });
        refetch();
      }
    },
    onSettled: () => {
      setFixingIssues(new Set());
    },
  });

  const fixSingleMutation = useMutation({
    mutationFn: async (issue: DiagnosticIssue) => {
      setFixingIssues((prev) => new Set([...prev, issue.id]));
      const result = await diagnosticsApi.fixIssue(issue);
      return result;
    },
    onSuccess: (data, issue) => {
      if (data.success) {
        addNotification({
          title: '修复成功',
          message: `${issue.name} 已修复`,
          type: 'success',
        });
        refetch();
      } else {
        addNotification({
          title: '修复失败',
          message: data.error || '无法自动修复此问题',
          type: 'error',
        });
      }
    },
    onSettled: (_, issue) => {
      setFixingIssues((prev) => {
        const next = new Set(prev);
        next.delete(issue.id);
        return next;
      });
    },
  });

  const getSeverityConfig = (severity: DiagnosticSeverity) => {
    switch (severity) {
      case 'critical':
        return {
          color: 'text-red-600',
          bgColor: 'bg-red-50 dark:bg-red-950',
          borderColor: 'border-red-200 dark:border-red-800',
          icon: ShieldAlert,
          label: '严重',
        };
      case 'error':
        return {
          color: 'text-red-500',
          bgColor: 'bg-red-50 dark:bg-red-950',
          borderColor: 'border-red-200 dark:border-red-800',
          icon: XCircle,
          label: '错误',
        };
      case 'warning':
        return {
          color: 'text-yellow-500',
          bgColor: 'bg-yellow-50 dark:bg-yellow-950',
          borderColor: 'border-yellow-200 dark:border-yellow-800',
          icon: AlertTriangle,
          label: '警告',
        };
      default:
        return {
          color: 'text-blue-500',
          bgColor: 'bg-blue-50 dark:bg-blue-950',
          borderColor: 'border-blue-200 dark:border-blue-800',
          icon: Info,
          label: '信息',
        };
    }
  };

  const getCategoryLabel = (category: DiagnosticIssue['category']) => {
    const labels: Record<DiagnosticIssue['category'], string> = {
      environment: '环境',
      service: '服务',
      config: '配置',
      network: '网络',
      permission: '权限',
    };
    return labels[category];
  };

  const autoFixableIssues = result?.issues.filter((i) => i.canAutoFix && !i.fixed) || [];

  return (
    <div className={className}>
      {/* 诊断控制面板 */}
      <Card className="mb-6">
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-lg bg-primary/10">
                <Stethoscope className="h-5 w-5 text-primary" />
              </div>
              <div>
                <CardTitle>系统诊断</CardTitle>
                <CardDescription>检测并修复系统问题</CardDescription>
              </div>
            </div>
            <div className="flex gap-2">
              {result && autoFixableIssues.length > 0 && (
                <Button
                  variant="default"
                  onClick={() => autoFixMutation.mutate(autoFixableIssues.map((i) => i.id))}
                  disabled={autoFixMutation.isPending}
                >
                  {autoFixMutation.isPending ? (
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  ) : (
                    <Wrench className="h-4 w-4 mr-2" />
                  )}
                  一键修复 ({autoFixableIssues.length})
                </Button>
              )}
              <Button
                variant="outline"
                onClick={() => refetch()}
                disabled={isLoading || runMutation.isPending}
              >
                {isLoading || runMutation.isPending ? (
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                ) : result ? (
                  <RefreshCw className="h-4 w-4 mr-2" />
                ) : (
                  <Play className="h-4 w-4 mr-2" />
                )}
                {result ? '重新诊断' : '开始诊断'}
              </Button>
            </div>
          </div>
        </CardHeader>

        {result && (
          <CardContent>
            <div className="grid grid-cols-4 gap-4">
              <div className="p-4 rounded-lg bg-muted">
                <p className="text-sm text-muted-foreground">总问题数</p>
                <p className="text-2xl font-bold">{result.issues.length}</p>
              </div>
              <div className="p-4 rounded-lg bg-red-50 dark:bg-red-950">
                <p className="text-sm text-red-600">严重/错误</p>
                <p className="text-2xl font-bold text-red-600">
                  {result.issues.filter((i) => ['critical', 'error'].includes(i.severity)).length}
                </p>
              </div>
              <div className="p-4 rounded-lg bg-yellow-50 dark:bg-yellow-950">
                <p className="text-sm text-yellow-600">警告</p>
                <p className="text-2xl font-bold text-yellow-600">
                  {result.issues.filter((i) => i.severity === 'warning').length}
                </p>
              </div>
              <div className="p-4 rounded-lg bg-green-50 dark:bg-green-950">
                <p className="text-sm text-green-600">已修复</p>
                <p className="text-2xl font-bold text-green-600">
                  {result.issues.filter((i) => i.fixed).length}
                </p>
              </div>
            </div>
          </CardContent>
        )}
      </Card>

      {/* 诊断结果列表 */}
      {result?.issues.length ? (
        <ScrollArea className="h-[500px] rounded-md border">
          <div className="p-4 space-y-3">
            {result.issues.map((issue) => {
              const config = getSeverityConfig(issue.severity);
              const Icon = config.icon;
              const isFixing = fixingIssues.has(issue.id);

              return (
                <div
                  key={issue.id}
                  className={`p-4 rounded-lg border ${config.bgColor} ${config.borderColor} ${issue.fixed ? 'opacity-60' : ''}`}
                >
                  <div className="flex items-start gap-3">
                    <Icon className={`h-5 w-5 ${config.color} mt-0.5`} />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 flex-wrap">
                        <span className="font-medium">{issue.name}</span>
                        <Badge variant="outline" className="text-xs">
                          {getCategoryLabel(issue.category)}
                        </Badge>
                        <Badge className={`text-xs ${config.color}`}>
                          {config.label}
                        </Badge>
                        {issue.fixed && (
                          <Badge variant="default" className="text-xs">
                            <CheckCircle className="h-3 w-3 mr-1" />
                            已修复
                          </Badge>
                        )}
                      </div>
                      <p className="text-sm text-muted-foreground mt-1">
                        {issue.description}
                      </p>
                      
                      {issue.fixed && issue.fixMessage && (
                        <p className="text-sm text-green-600 mt-2">
                          {issue.fixMessage}
                        </p>
                      )}

                      {issue.error && (
                        <p className="text-sm text-red-600 mt-2">
                          错误: {issue.error}
                        </p>
                      )}

                      {!issue.fixed && issue.canAutoFix && (
                        <div className="mt-3">
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() => fixSingleMutation.mutate(issue)}
                            disabled={isFixing}
                          >
                            {isFixing ? (
                              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                            ) : (
                              <Wrench className="h-4 w-4 mr-2" />
                            )}
                            自动修复
                          </Button>
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </ScrollArea>
      ) : result ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <div className="p-4 rounded-full bg-green-100 dark:bg-green-900">
              <CheckCircle className="h-8 w-8 text-green-600" />
            </div>
            <p className="mt-4 text-lg font-medium">系统状态良好</p>
            <p className="text-muted-foreground">未发现任何问题</p>
          </CardContent>
        </Card>
      ) : null}
    </div>
  );
}
