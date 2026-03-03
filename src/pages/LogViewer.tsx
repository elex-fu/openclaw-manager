import { useState, useEffect, useRef, useCallback, useMemo, memo } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { logApi } from '@/lib/tauri-api';
import type { LogEntry, LogLevel, LogFilter } from '@/types';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  Play,
  Pause,
  Trash2,
  Download,
  Search,
  FileText,
  Info,
  X,
} from 'lucide-react';
import { cn } from '@/lib/utils';
import { format } from '@/lib/date';
import { useVirtualizer } from '@tanstack/react-virtual';

// 日志级别配置
const LOG_LEVELS: { value: LogLevel; label: string; color: string }[] = [
  { value: 'ERROR', label: '错误', color: 'bg-red-500' },
  { value: 'WARN', label: '警告', color: 'bg-yellow-500' },
  { value: 'INFO', label: '信息', color: 'bg-blue-500' },
  { value: 'DEBUG', label: '调试', color: 'bg-gray-500' },
  { value: 'TRACE', label: '追踪', color: 'bg-purple-500' },
];

// 使用 memo 优化日志条目组件
interface LogEntryItemProps {
  log: LogEntry;
  isHighlighted?: boolean;
  searchQuery?: string;
  onClick?: () => void;
}

const LogEntryItem = memo(function LogEntryItem({ log, isHighlighted, searchQuery, onClick }: LogEntryItemProps) {
  const highlightText = useCallback((text: string, query?: string) => {
    if (!query) return text;
    const parts = text.split(new RegExp(`(${query})`, 'gi'));
    return parts.map((part, i) =>
      part.toLowerCase() === query.toLowerCase() ? (
        <mark key={i} className="bg-yellow-500/30 text-yellow-200 rounded px-0.5">
          {part}
        </mark>
      ) : (
        part
      )
    );
  }, []);

  return (
    <div
      onClick={onClick}
      className={cn(
        'flex items-start gap-3 px-4 py-2 text-sm font-mono border-b border-border/50 hover:bg-accent/50 transition-colors cursor-pointer',
        isHighlighted && 'bg-accent',
        log.level === 'ERROR' && 'bg-red-500/5',
        log.level === 'WARN' && 'bg-yellow-500/5'
      )}
    >
      {/* 时间戳 */}
      <div className="text-muted-foreground text-xs shrink-0 w-36">
        {format(new Date(log.timestamp), 'yyyy-MM-dd HH:mm:ss.SSS')}
      </div>

      {/* 级别指示器 */}
      <div className="shrink-0 w-16">
        <Badge
          variant="outline"
          className={cn(
            'text-xs font-medium',
            log.level === 'ERROR' && 'border-red-500/50 text-red-400 bg-red-500/10',
            log.level === 'WARN' && 'border-yellow-500/50 text-yellow-400 bg-yellow-500/10',
            log.level === 'INFO' && 'border-blue-500/50 text-blue-400 bg-blue-500/10',
            log.level === 'DEBUG' && 'border-gray-500/50 text-gray-400 bg-gray-500/10',
            log.level === 'TRACE' && 'border-purple-500/50 text-purple-400 bg-purple-500/10'
          )}
        >
          {log.level}
        </Badge>
      </div>

      {/* 来源 */}
      <div className="text-muted-foreground text-xs shrink-0 w-24 truncate">
        {log.source}
      </div>

      {/* 消息 */}
      <div className="flex-1 break-all text-foreground/90">
        {highlightText(log.message, searchQuery)}
      </div>
    </div>
  );
});

// 使用 memo 优化日志筛选栏组件
interface LogFilterBarProps {
  filter: LogFilter;
  onFilterChange: (filter: LogFilter) => void;
  onExport: (format: 'text' | 'json' | 'csv') => void;
  onClear: () => void;
  isLive: boolean;
  onToggleLive: () => void;
  logCount: number;
}

const LogFilterBar = memo(function LogFilterBar({
  filter,
  onFilterChange,
  onExport,
  onClear,
  isLive,
  onToggleLive,
  logCount,
}: LogFilterBarProps) {
  const [searchInput, setSearchInput] = useState(filter.searchQuery || '');

  const handleSearch = useCallback(() => {
    onFilterChange({ ...filter, searchQuery: searchInput || undefined });
  }, [filter, searchInput, onFilterChange]);

  const toggleLevel = useCallback(
    (level: LogLevel) => {
      const newLevels = filter.levels.includes(level)
        ? filter.levels.filter((l) => l !== level)
        : [...filter.levels, level];
      onFilterChange({ ...filter, levels: newLevels });
    },
    [filter, onFilterChange]
  );

  const clearSearch = useCallback(() => {
    setSearchInput('');
    onFilterChange({ ...filter, searchQuery: undefined });
  }, [filter, onFilterChange]);

  const clearLevels = useCallback(() => {
    onFilterChange({ ...filter, levels: [] });
  }, [filter, onFilterChange]);

  return (
    <div className="flex flex-col gap-3 p-4 bg-card border-b">
      <div className="flex items-center gap-3">
        {/* 搜索框 */}
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="搜索日志内容..."
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            className="pl-9"
          />
        </div>

        {/* 级别筛选 */}
        <div className="flex items-center gap-1">
          {LOG_LEVELS.map((level) => (
            <Button
              key={level.value}
              variant={filter.levels.includes(level.value) ? 'default' : 'outline'}
              size="sm"
              onClick={() => toggleLevel(level.value)}
              className={cn(
                'h-8 px-2 text-xs',
                filter.levels.includes(level.value) &&
                  level.value === 'ERROR' && 'bg-red-500 hover:bg-red-600',
                filter.levels.includes(level.value) &&
                  level.value === 'WARN' && 'bg-yellow-500 hover:bg-yellow-600 text-black',
                filter.levels.includes(level.value) &&
                  level.value === 'INFO' && 'bg-blue-500 hover:bg-blue-600'
              )}
            >
              {level.label}
            </Button>
          ))}
        </div>

        <div className="flex-1" />

        {/* 实时/暂停按钮 */}
        <Button
          variant={isLive ? 'default' : 'outline'}
          size="sm"
          onClick={onToggleLive}
          className="gap-2"
        >
          {isLive ? (
            <>
              <Pause className="h-4 w-4" />
              暂停
            </>
          ) : (
            <>
              <Play className="h-4 w-4" />
              实时
            </>
          )}
        </Button>

        {/* 清空按钮 */}
        <Button variant="outline" size="sm" onClick={onClear} className="gap-2">
          <Trash2 className="h-4 w-4" />
          清空
        </Button>

        {/* 导出菜单 */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm" className="gap-2">
              <Download className="h-4 w-4" />
              导出
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={() => onExport('text')}>
              <FileText className="h-4 w-4 mr-2" />
              导出为文本
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => onExport('json')}>
              <FileText className="h-4 w-4 mr-2" />
              导出为 JSON
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => onExport('csv')}>
              <FileText className="h-4 w-4 mr-2" />
              导出为 CSV
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      {/* 统计信息 */}
      <div className="flex items-center gap-4 text-xs text-muted-foreground">
        <span>共 {logCount} 条日志</span>
        {filter.searchQuery && (
          <Badge variant="secondary" className="gap-1">
            搜索: {filter.searchQuery}
            <X
              className="h-3 w-3 cursor-pointer"
              onClick={clearSearch}
            />
          </Badge>
        )}
        {filter.levels.length > 0 && (
          <Badge variant="secondary" className="gap-1">
            级别: {filter.levels.join(', ')}
            <X
              className="h-3 w-3 cursor-pointer"
              onClick={clearLevels}
            />
          </Badge>
        )}
      </div>
    </div>
  );
});

// 虚拟化日志列表组件
interface VirtualLogListProps {
  logs: LogEntry[];
  searchQuery?: string;
  onLogClick: (log: LogEntry) => void;
  isLive: boolean;
}

const VirtualLogList = memo(function VirtualLogList({
  logs,
  searchQuery,
  onLogClick,
  isLive,
}: VirtualLogListProps) {
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: logs.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 40, // 估计每项高度
    overscan: 5, // 预渲染5项
  });

  const virtualItems = virtualizer.getVirtualItems();

  // 自动滚动到底部
  useEffect(() => {
    if (isLive && logs.length > 0) {
      virtualizer.scrollToIndex(logs.length - 1, { align: 'end' });
    }
  }, [logs.length, isLive, virtualizer]);

  if (logs.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-64 text-muted-foreground">
        <Info className="h-12 w-12 mb-4 opacity-50" />
        <p>暂无日志记录</p>
        <p className="text-sm">调整筛选条件或等待新日志</p>
      </div>
    );
  }

  return (
    <div ref={parentRef} className="h-full overflow-auto">
      <div
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: '100%',
          position: 'relative',
        }}
      >
        {virtualItems.map((virtualItem) => {
          const log = logs[virtualItem.index];
          return (
            <div
              key={log.id}
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                height: `${virtualItem.size}px`,
                transform: `translateY(${virtualItem.start}px)`,
              }}
            >
              <LogEntryItem
                log={log}
                searchQuery={searchQuery}
                onClick={() => onLogClick(log)}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
});

// 主日志查看页面
export function LogViewer() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [filter, setFilter] = useState<LogFilter>({
    levels: ['ERROR', 'WARN', 'INFO'],
  });
  const [isLive, setIsLive] = useState(true);
  const [subscriptionId, setSubscriptionId] = useState<string | null>(null);
  const [selectedLog, setSelectedLog] = useState<LogEntry | null>(null);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  // 获取初始日志 - 使用优化后的 API
  const { data: initialLogs } = useQuery({
    queryKey: ['recentLogs', filter],
    queryFn: async () => {
      const response = await logApi.getRecentLogs({
        limit: 100,
        levels: filter.levels,
        sources: filter.sources,
        searchQuery: filter.searchQuery,
      });
      return response;
    },
    staleTime: 0, // 日志需要实时性
  });

  // 设置初始日志
  useEffect(() => {
    if (initialLogs) {
      setLogs(initialLogs);
    }
  }, [initialLogs]);

  // 订阅实时日志
  useEffect(() => {
    let isMounted = true;

    const setupSubscription = async () => {
      try {
        // 先取消之前的订阅
        if (subscriptionId) {
          await logApi.unsubscribeLogs(subscriptionId);
        }
        if (unlistenRef.current) {
          unlistenRef.current();
        }

        // 创建新订阅
        const response = await logApi.subscribeLogs({
          levels: filter.levels,
          sources: filter.sources,
          searchQuery: filter.searchQuery,
        });
        const { subscription_id } = response;

        if (!isMounted) return;
        setSubscriptionId(subscription_id);

        // 监听日志事件
        const unlisten = await listen<LogEntry>(
          `log-entry-${subscription_id}`,
          (event) => {
            if (!isMounted) return;
            setLogs((prev) => {
              const newLogs = [...prev, event.payload];
              // 限制内存中的日志数量
              if (newLogs.length > 5000) {
                return newLogs.slice(-5000);
              }
              return newLogs;
            });
          }
        );

        unlistenRef.current = unlisten;
      } catch (error) {
        console.error('Failed to setup log subscription:', error);
      }
    };

    if (isLive) {
      setupSubscription();
    }

    return () => {
      isMounted = false;
      if (unlistenRef.current) {
        unlistenRef.current();
      }
    };
  }, [isLive, filter.levels, filter.sources, filter.searchQuery]);

  // 导出日志
  const exportMutation = useMutation({
    mutationFn: async (format: 'text' | 'json' | 'csv') => {
      // 使用默认下载目录
      const defaultPath = `logs_export_${new Date().toISOString().replace(/[:.]/g, '-')}.${format}`;
      const response = await logApi.exportLogs({
        filter,
        format,
        outputPath: defaultPath,
      });
      return response;
    },
  });

  // 清空日志
  const handleClear = useCallback(() => {
    setLogs([]);
  }, []);

  // 切换实时模式
  const handleToggleLive = useCallback(() => {
    setIsLive((prev) => !prev);
  }, []);

  // 处理筛选变化 - 使用 useCallback
  const handleFilterChange = useCallback((newFilter: LogFilter) => {
    setFilter(newFilter);
  }, []);

  // 处理导出 - 使用 useCallback
  const handleExport = useCallback((format: 'text' | 'json' | 'csv') => {
    exportMutation.mutate(format);
  }, [exportMutation]);

  // 处理日志点击 - 使用 useCallback
  const handleLogClick = useCallback((log: LogEntry) => {
    setSelectedLog(log);
  }, []);

  // 过滤后的日志 - 使用 useMemo
  const filteredLogs = useMemo(() => {
    return logs.filter((log) => {
      // 级别筛选
      if (filter.levels.length > 0 && !filter.levels.includes(log.level)) {
        return false;
      }
      // 搜索筛选
      if (filter.searchQuery) {
        const query = filter.searchQuery.toLowerCase();
        return (
          log.message.toLowerCase().includes(query) ||
          log.source.toLowerCase().includes(query)
        );
      }
      return true;
    });
  }, [logs, filter]);

  return (
    <div className="flex flex-col h-full bg-background">
      {/* 筛选栏 */}
      <LogFilterBar
        filter={filter}
        onFilterChange={handleFilterChange}
        onExport={handleExport}
        onClear={handleClear}
        isLive={isLive}
        onToggleLive={handleToggleLive}
        logCount={filteredLogs.length}
      />

      {/* 虚拟化日志列表 */}
      <div className="flex-1 overflow-hidden">
        <VirtualLogList
          logs={filteredLogs}
          searchQuery={filter.searchQuery}
          onLogClick={handleLogClick}
          isLive={isLive}
        />
      </div>

      {/* 日志详情对话框 */}
      <Dialog
        open={!!selectedLog}
        onOpenChange={() => setSelectedLog(null)}
      >
        <DialogContent className="max-w-3xl max-h-[80vh]">
          <DialogHeader>
            <DialogTitle>日志详情</DialogTitle>
            <DialogDescription>
              查看完整的日志条目信息
            </DialogDescription>
          </DialogHeader>
          {selectedLog && (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <label className="text-muted-foreground">时间戳</label>
                  <p>
                    {format(
                      new Date(selectedLog.timestamp),
                      'yyyy-MM-dd HH:mm:ss.SSS'
                    )}
                  </p>
                </div>
                <div>
                  <label className="text-muted-foreground">级别</label>
                  <Badge
                    variant="outline"
                    className={cn(
                      selectedLog.level === 'ERROR' &&
                        'border-red-500/50 text-red-400',
                      selectedLog.level === 'WARN' &&
                        'border-yellow-500/50 text-yellow-400'
                    )}
                  >
                    {selectedLog.level}
                  </Badge>
                </div>
                <div>
                  <label className="text-muted-foreground">来源</label>
                  <p>{selectedLog.source}</p>
                </div>
                <div>
                  <label className="text-muted-foreground">ID</label>
                  <p className="font-mono text-xs">{selectedLog.id}</p>
                </div>
              </div>
              <Separator />
              <div>
                <label className="text-muted-foreground">消息</label>
                <pre className="mt-2 p-4 bg-muted rounded-lg font-mono text-sm whitespace-pre-wrap">
                  {selectedLog.message}
                </pre>
              </div>
              {selectedLog.metadata && (
                <>
                  <Separator />
                  <div>
                    <label className="text-muted-foreground">元数据</label>
                    <pre className="mt-2 p-4 bg-muted rounded-lg font-mono text-xs">
                      {JSON.stringify(selectedLog.metadata, null, 2)}
                    </pre>
                  </div>
                </>
              )}
            </div>
          )}
          <DialogFooter>
            <Button variant="outline" onClick={() => setSelectedLog(null)}>
              关闭
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
