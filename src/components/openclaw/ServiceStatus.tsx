import { useState, useEffect } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { serviceApi } from '@/lib/tauri-api';
import { useConfigStore } from '@/stores';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import {
  Play,
  Square,
  RefreshCw,
  Server,
  Activity,
  AlertCircle,
  CheckCircle,
  Loader2
} from 'lucide-react';

interface ServiceStatusProps {
  showActions?: boolean;
  className?: string;
}

export function ServiceStatus({ showActions = true, className }: ServiceStatusProps) {
  const { gatewayStatus, setGatewayStatus } = useConfigStore();
  const [isLoading, setIsLoading] = useState(false);

  const { data: statusData, refetch } = useQuery({
    queryKey: ['service-status'],
    queryFn: () => serviceApi.getServiceStatus(),
    refetchInterval: 5000, // 每 5 秒刷新一次
  });

  const serviceInfo = statusData;

  useEffect(() => {
    if (serviceInfo) {
      setGatewayStatus(serviceInfo.status);
    }
  }, [serviceInfo, setGatewayStatus]);

  const startMutation = useMutation({
    mutationFn: serviceApi.startService,
    onMutate: () => {
      setIsLoading(true);
      setGatewayStatus('starting');
    },
    onSuccess: (data) => {
      if (data) {
        setGatewayStatus('running');
        refetch();
      } else {
        setGatewayStatus('error');
      }
    },
    onError: () => {
      setGatewayStatus('error');
    },
    onSettled: () => {
      setIsLoading(false);
    },
  });

  const stopMutation = useMutation({
    mutationFn: serviceApi.stopService,
    onMutate: () => {
      setIsLoading(true);
      setGatewayStatus('stopping');
    },
    onSuccess: (data) => {
      if (data) {
        setGatewayStatus('stopped');
        refetch();
      } else {
        setGatewayStatus('error');
      }
    },
    onError: () => {
      setGatewayStatus('error');
    },
    onSettled: () => {
      setIsLoading(false);
    },
  });

  const getStatusConfig = (status: typeof gatewayStatus) => {
    switch (status) {
      case 'running':
        return {
          color: 'bg-green-500',
          bgColor: 'bg-green-50 dark:bg-green-950',
          textColor: 'text-green-700 dark:text-green-300',
          label: '运行中',
          icon: CheckCircle,
        };
      case 'starting':
        return {
          color: 'bg-yellow-500',
          bgColor: 'bg-yellow-50 dark:bg-yellow-950',
          textColor: 'text-yellow-700 dark:text-yellow-300',
          label: '启动中',
          icon: Loader2,
        };
      case 'stopping':
        return {
          color: 'bg-orange-500',
          bgColor: 'bg-orange-50 dark:bg-orange-950',
          textColor: 'text-orange-700 dark:text-orange-300',
          label: '停止中',
          icon: Loader2,
        };
      case 'error':
        return {
          color: 'bg-red-500',
          bgColor: 'bg-red-50 dark:bg-red-950',
          textColor: 'text-red-700 dark:text-red-300',
          label: '错误',
          icon: AlertCircle,
        };
      default:
        return {
          color: 'bg-gray-500',
          bgColor: 'bg-gray-50 dark:bg-gray-950',
          textColor: 'text-gray-700 dark:text-gray-300',
          label: '已停止',
          icon: Server,
        };
    }
  };

  const statusConfig = getStatusConfig(gatewayStatus);
  const StatusIcon = statusConfig.icon;
  const isRunning = gatewayStatus === 'running';
  const isProcessing = gatewayStatus === 'starting' || gatewayStatus === 'stopping' || isLoading;

  return (
    <Card className={className}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className={`p-2 rounded-lg ${statusConfig.bgColor}`}>
              <StatusIcon className={`h-5 w-5 ${statusConfig.textColor} ${isProcessing ? 'animate-spin' : ''}`} />
            </div>
            <div>
              <CardTitle className="text-lg">Gateway 服务</CardTitle>
              <CardDescription>OpenClaw 核心服务</CardDescription>
            </div>
          </div>
          <Badge 
            variant={isRunning ? 'default' : 'secondary'}
            className={`${statusConfig.bgColor} ${statusConfig.textColor} border-0`}
          >
            <span className={`w-2 h-2 rounded-full ${statusConfig.color} mr-2 ${isRunning ? 'animate-pulse' : ''}`} />
            {statusConfig.label}
          </Badge>
        </div>
      </CardHeader>
      
      {showActions && (
        <CardContent className="pt-0">
          <div className="flex items-center gap-3">
            {isRunning ? (
              <Button
                variant="outline"
                size="sm"
                onClick={() => stopMutation.mutate()}
                disabled={isProcessing}
              >
                {isProcessing ? (
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                ) : (
                  <Square className="h-4 w-4 mr-2" />
                )}
                停止服务
              </Button>
            ) : (
              <Button
                variant="default"
                size="sm"
                onClick={() => startMutation.mutate()}
                disabled={isProcessing}
              >
                {isProcessing ? (
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                ) : (
                  <Play className="h-4 w-4 mr-2" />
                )}
                启动服务
              </Button>
            )}
            <Button
              variant="ghost"
              size="sm"
              onClick={() => refetch()}
              disabled={isProcessing}
            >
              <RefreshCw className="h-4 w-4 mr-2" />
              刷新
            </Button>
          </div>
          
          {serviceInfo?.uptime && isRunning && (
            <div className="mt-4 flex items-center gap-2 text-sm text-muted-foreground">
              <Activity className="h-4 w-4" />
              <span>运行时间: {formatUptime(serviceInfo.uptime)}</span>
            </div>
          )}
        </CardContent>
      )}
    </Card>
  );
}

function formatUptime(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;
  
  if (hours > 0) {
    return `${hours}小时 ${minutes}分钟`;
  } else if (minutes > 0) {
    return `${minutes}分钟 ${secs}秒`;
  } else {
    return `${secs}秒`;
  }
}
