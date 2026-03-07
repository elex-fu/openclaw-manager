import { useEffect, useState, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  Download,
  Settings,
  Server,
  AlertCircle,
  CheckCircle,
  Clock,
  RefreshCw,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { systemApi } from '@/lib/tauri-api';

interface Activity {
  id: string;
  timestamp: number;
  activity_type: 'install' | 'config' | 'service' | 'error';
  message: string;
  details?: string;
}

const ACTIVITY_ICONS = {
  install: Download,
  config: Settings,
  service: Server,
  error: AlertCircle,
};

const ACTIVITY_COLORS = {
  install: 'bg-blue-50 text-blue-600 dark:bg-blue-950 dark:text-blue-400',
  config: 'bg-purple-50 text-purple-600 dark:bg-purple-950 dark:text-purple-400',
  service: 'bg-green-50 text-green-600 dark:bg-green-950 dark:text-green-400',
  error: 'bg-red-50 text-red-600 dark:bg-red-950 dark:text-red-400',
};

const ACTIVITY_LABELS = {
  install: '安装',
  config: '配置',
  service: '服务',
  error: '错误',
};

function formatTimeAgo(timestamp: number): string {
  const now = Date.now() / 1000;
  const diff = now - timestamp;

  if (diff < 60) {
    return '刚刚';
  } else if (diff < 3600) {
    const minutes = Math.floor(diff / 60);
    return `${minutes} 分钟前`;
  } else if (diff < 86400) {
    const hours = Math.floor(diff / 3600);
    return `${hours} 小时前`;
  } else {
    const days = Math.floor(diff / 86400);
    return `${days} 天前`;
  }
}

function formatDate(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return date.toLocaleString('zh-CN', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function ActivityLog() {
  const [activities, setActivities] = useState<Activity[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const fetchActivities = useCallback(async () => {
    try {
      const data = await systemApi.getRecentActivities(10);
      if (data) {
        setActivities(data);
      }
    } catch (error) {
      console.error('Failed to fetch activities:', error);
    }
  }, []);

  useEffect(() => {
    fetchActivities();

    // Refresh every 30 seconds
    const interval = setInterval(fetchActivities, 30000);
    return () => clearInterval(interval);
  }, [fetchActivities]);

  const handleRefresh = () => {
    setIsLoading(true);
    fetchActivities().finally(() => setIsLoading(false));
  };

  return (
    <Card className="w-full">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-indigo-50 dark:bg-indigo-950">
              <Clock className="h-5 w-5 text-indigo-600 dark:text-indigo-400" />
            </div>
            <div>
              <CardTitle className="text-lg">最近活动</CardTitle>
              <CardDescription>系统操作日志</CardDescription>
            </div>
          </div>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleRefresh}
            disabled={isLoading}
          >
            <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
          </Button>
        </div>
      </CardHeader>

      <CardContent>
        <ScrollArea className="h-[300px] pr-4">
          <div className="relative space-y-4">
            {/* Timeline line */}
            <div className="absolute left-4 top-2 bottom-2 w-px bg-border" />

            {activities.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
                <CheckCircle className="h-8 w-8 mb-2 opacity-50" />
                <p className="text-sm">暂无活动记录</p>
              </div>
            ) : (
              activities.map((activity) => {
                const Icon = ACTIVITY_ICONS[activity.activity_type] || Server;
                const colorClass = ACTIVITY_COLORS[activity.activity_type] || ACTIVITY_COLORS.service;
                const label = ACTIVITY_LABELS[activity.activity_type] || '其他';

                return (
                  <div
                    key={activity.id}
                    className="relative flex gap-4 group"
                  >
                    {/* Timeline dot */}
                    <div
                      className={`relative z-10 flex h-8 w-8 shrink-0 items-center justify-center rounded-full ${colorClass}`}
                    >
                      <Icon className="h-4 w-4" />
                    </div>

                    {/* Content */}
                    <div className="flex-1 space-y-1 pb-4">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          <span className="font-medium text-sm">
                            {activity.message}
                          </span>
                          <Badge variant="outline" className="text-xs">
                            {label}
                          </Badge>
                        </div>
                        <span className="text-xs text-muted-foreground">
                          {formatTimeAgo(activity.timestamp)}
                        </span>
                      </div>

                      {activity.details && (
                        <p className="text-xs text-muted-foreground">
                          {activity.details}
                        </p>
                      )}

                      <p className="text-xs text-muted-foreground">
                        {formatDate(activity.timestamp)}
                      </p>
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
