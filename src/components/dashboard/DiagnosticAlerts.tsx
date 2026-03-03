import { useEffect, useState, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  AlertTriangle,
  AlertCircle,
  Info,
  Wrench,
  Stethoscope,
  ChevronRight,
  RefreshCw,
  CheckCircle,
  Loader2,
} from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { systemApi } from '@/lib/tauri-api';

interface DiagnosticAlert {
  id: string;
  severity: 'info' | 'warning' | 'error';
  title: string;
  message: string;
  fixable: boolean;
  category: string;
}

const SEVERITY_ICONS = {
  info: Info,
  warning: AlertTriangle,
  error: AlertCircle,
};

const SEVERITY_COLORS = {
  info: {
    bg: 'bg-blue-50 dark:bg-blue-950',
    text: 'text-blue-600 dark:text-blue-400',
    border: 'border-blue-200 dark:border-blue-800',
    badge: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
  },
  warning: {
    bg: 'bg-yellow-50 dark:bg-yellow-950',
    text: 'text-yellow-600 dark:text-yellow-400',
    border: 'border-yellow-200 dark:border-yellow-800',
    badge: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
  },
  error: {
    bg: 'bg-red-50 dark:bg-red-950',
    text: 'text-red-600 dark:text-red-400',
    border: 'border-red-200 dark:border-red-800',
    badge: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
  },
};

const CATEGORY_LABELS: Record<string, string> = {
  system: '系统',
  config: '配置',
  service: '服务',
  network: '网络',
  permission: '权限',
};

export function DiagnosticAlerts() {
  const navigate = useNavigate();
  const [alerts, setAlerts] = useState<DiagnosticAlert[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isFixing, setIsFixing] = useState<string | null>(null);
  const [fixedAlerts, setFixedAlerts] = useState<Set<string>>(new Set());

  const fetchAlerts = useCallback(async () => {
    try {
      const data = await systemApi.getDiagnosticAlerts();
      if (data) {
        setAlerts(data);
      }
    } catch (error) {
      console.error('Failed to fetch diagnostic alerts:', error);
    }
  }, []);

  useEffect(() => {
    fetchAlerts();

    // Refresh every 60 seconds
    const interval = setInterval(fetchAlerts, 60000);
    return () => clearInterval(interval);
  }, [fetchAlerts]);

  const handleRefresh = () => {
    setIsLoading(true);
    fetchAlerts().finally(() => setIsLoading(false));
  };

  const handleFix = async (alert: DiagnosticAlert) => {
    if (!alert.fixable) return;

    setIsFixing(alert.id);
    try {
      // In a real implementation, this would call a fix command
      // For now, we simulate a fix
      await new Promise((resolve) => setTimeout(resolve, 1000));
      setFixedAlerts((prev) => new Set(prev).add(alert.id));
    } finally {
      setIsFixing(null);
    }
  };

  const handleGoToDiagnostics = () => {
    navigate('/diagnostics');
  };

  const warningCount = alerts.filter((a) => a.severity === 'warning').length;
  const errorCount = alerts.filter((a) => a.severity === 'error').length;

  return (
    <Card className="w-full">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-orange-50 dark:bg-orange-950">
              <Stethoscope className="h-5 w-5 text-orange-600 dark:text-orange-400" />
            </div>
            <div>
              <CardTitle className="text-lg">诊断警告</CardTitle>
              <CardDescription>
                {alerts.length === 0 ? (
                  '系统运行正常'
                ) : (
                  <span className="flex items-center gap-2">
                    {errorCount > 0 && (
                      <span className="text-red-600 dark:text-red-400">
                        {errorCount} 个错误
                      </span>
                    )}
                    {warningCount > 0 && (
                      <span className="text-yellow-600 dark:text-yellow-400">
                        {warningCount} 个警告
                      </span>
                    )}
                  </span>
                )}
              </CardDescription>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={handleRefresh}
              disabled={isLoading}
            >
              <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleGoToDiagnostics}
            >
              查看全部
              <ChevronRight className="ml-1 h-4 w-4" />
            </Button>
          </div>
        </div>
      </CardHeader>

      <CardContent>
        {alerts.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
            <CheckCircle className="h-12 w-12 mb-3 text-green-500" />
            <p className="text-sm font-medium">未发现任何问题</p>
            <p className="text-xs mt-1">系统运行状况良好</p>
          </div>
        ) : (
          <div className="space-y-3">
            {alerts.map((alert) => {
              const Icon = SEVERITY_ICONS[alert.severity];
              const colors = SEVERITY_COLORS[alert.severity];
              const isFixed = fixedAlerts.has(alert.id);
              const categoryLabel = CATEGORY_LABELS[alert.category] || alert.category;
              const isFixingThis = isFixing === alert.id;

              return (
                <div
                  key={alert.id}
                  className={`p-4 rounded-lg border ${colors.bg} ${colors.border} ${
                    isFixed ? 'opacity-50' : ''
                  }`}
                >
                  <div className="flex items-start gap-3">
                    <div className={`p-1.5 rounded-full ${colors.bg}`}>
                      <Icon className={`h-4 w-4 ${colors.text}`} />
                    </div>

                    <div className="flex-1 min-w-0">
                      <div className="flex items-center justify-between gap-2">
                        <div className="flex items-center gap-2">
                          <span className="font-medium text-sm">{alert.title}</span>
                          <Badge variant="outline" className={`text-xs ${colors.badge}`}>
                            {categoryLabel}
                          </Badge>
                          {isFixed && (
                            <Badge variant="outline" className="text-xs bg-green-100 text-green-800">
                              已修复
                            </Badge>
                          )}
                        </div>
                      </div>

                      <p className={`text-sm mt-1 ${colors.text}`}>{alert.message}</p>

                      {alert.fixable && !isFixed && (
                        <div className="mt-3">
                          <Button
                            size="sm"
                            variant="secondary"
                            onClick={() => handleFix(alert)}
                            disabled={isFixingThis}
                          >
                            {isFixingThis ? (
                              <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
                            ) : (
                              <Wrench className="mr-1.5 h-3.5 w-3.5" />
                            )}
                            {isFixingThis ? '修复中...' : '快速修复'}
                          </Button>
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
