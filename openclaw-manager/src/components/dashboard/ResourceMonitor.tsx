import { useEffect, useState, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { Activity, Cpu, HardDrive, MemoryStick, RefreshCw } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  AreaChart,
  Area,
} from 'recharts';
import { systemApi } from '@/lib/tauri-api';

interface SystemResources {
  cpu: {
    usage: number;
    cores: number;
    name: string;
    frequency: number;
  };
  memory: {
    used: number;
    total: number;
    usage: number;
    available: number;
  };
  disk: {
    used: number;
    total: number;
    usage: number;
    free: number;
  };
  timestamp: number;
}

interface CpuDataPoint {
  time: string;
  usage: number;
}

const REFRESH_INTERVAL = 5000; // 5 seconds
const MAX_DATA_POINTS = 60; // 5 minutes of data

export function ResourceMonitor() {
  const [resources, setResources] = useState<SystemResources | null>(null);
  const [cpuHistory, setCpuHistory] = useState<CpuDataPoint[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());

  const fetchResources = useCallback(async () => {
    try {
      const data = await systemApi.getSystemResources();
      if (data) {
        setResources(data);
        setLastUpdate(new Date());

        // Update CPU history
        setCpuHistory(prev => {
          const now = new Date();
          const timeStr = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`;
          const newPoint: CpuDataPoint = {
            time: timeStr,
            usage: Math.round(data!.cpu.usage * 10) / 10,
          };
          const newHistory = [...prev, newPoint];
          if (newHistory.length > MAX_DATA_POINTS) {
            return newHistory.slice(newHistory.length - MAX_DATA_POINTS);
          }
          return newHistory;
        });
      }
    } catch (error) {
      console.error('Failed to fetch system resources:', error);
    }
  }, []);

  useEffect(() => {
    // Initial fetch
    fetchResources();

    // Set up interval
    const interval = setInterval(fetchResources, REFRESH_INTERVAL);

    return () => clearInterval(interval);
  }, [fetchResources]);

  const handleRefresh = () => {
    setIsLoading(true);
    fetchResources().finally(() => setIsLoading(false));
  };

  const getUsageColor = (usage: number) => {
    if (usage >= 90) return 'text-red-500';
    if (usage >= 70) return 'text-yellow-500';
    return 'text-green-500';
  };

  const getProgressColor = (usage: number) => {
    if (usage >= 90) return 'bg-red-500';
    if (usage >= 70) return 'bg-yellow-500';
    return 'bg-green-500';
  };

  return (
    <Card className="w-full">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-blue-50 dark:bg-blue-950">
              <Activity className="h-5 w-5 text-blue-600 dark:text-blue-400" />
            </div>
            <div>
              <CardTitle className="text-lg">资源监控</CardTitle>
              <CardDescription>
                系统资源使用情况
                <span className="ml-2 text-xs text-muted-foreground">
                  更新于 {lastUpdate.toLocaleTimeString()}
                </span>
              </CardDescription>
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

      <CardContent className="space-y-6">
        {/* CPU Usage Chart */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Cpu className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm font-medium">CPU 使用率</span>
            </div>
            <div className="flex items-center gap-2">
              <Badge variant="secondary" className="text-xs">
                {resources?.cpu.cores || '-'} 核
              </Badge>
              <span className={`text-sm font-bold ${getUsageColor(resources?.cpu.usage || 0)}`}>
                {resources ? `${Math.round(resources.cpu.usage)}%` : '-'}
              </span>
            </div>
          </div>
          <div className="h-32">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={cpuHistory}>
                <defs>
                  <linearGradient id="cpuGradient" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3} />
                    <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                <XAxis
                  dataKey="time"
                  tick={{ fontSize: 10 }}
                  interval="preserveStartEnd"
                  tickCount={5}
                />
                <YAxis
                  domain={[0, 100]}
                  tick={{ fontSize: 10 }}
                  tickFormatter={(value) => `${value}%`}
                />
                <Tooltip
                  contentStyle={{ fontSize: 12 }}
                  formatter={(value: number) => [`${value}%`, 'CPU']}
                />
                <Area
                  type="monotone"
                  dataKey="usage"
                  stroke="#3b82f6"
                  strokeWidth={2}
                  fill="url(#cpuGradient)"
                  isAnimationActive={false}
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Memory Usage */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <MemoryStick className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm font-medium">内存使用</span>
            </div>
            <span className={`text-sm font-bold ${getUsageColor(resources?.memory.usage || 0)}`}>
              {resources ? `${Math.round(resources.memory.usage)}%` : '-'}
            </span>
          </div>
          <Progress
            value={resources?.memory.usage || 0}
            className="h-2"
          />
          <div className="flex justify-between text-xs text-muted-foreground">
            <span>
              已用: {resources ? `${Math.round(resources.memory.used / 1024 * 10) / 10} GB` : '-'}
            </span>
            <span>
              总计: {resources ? `${Math.round(resources.memory.total / 1024 * 10) / 10} GB` : '-'}
            </span>
          </div>
        </div>

        {/* Disk Usage */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <HardDrive className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm font-medium">磁盘使用</span>
            </div>
            <span className={`text-sm font-bold ${getUsageColor(resources?.disk.usage || 0)}`}>
              {resources ? `${Math.round(resources.disk.usage)}%` : '-'}
            </span>
          </div>
          <Progress
            value={resources?.disk.usage || 0}
            className="h-2"
          />
          <div className="flex justify-between text-xs text-muted-foreground">
            <span>
              已用: {resources ? `${resources.disk.used} GB` : '-'}
            </span>
            <span>
              可用: {resources ? `${resources.disk.free} GB` : '-'}
            </span>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
