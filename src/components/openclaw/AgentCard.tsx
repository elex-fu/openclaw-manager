import { useState, useCallback, memo } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { agentApi } from '@/lib/tauri-api';
import { useConfigStore, useAppStore } from '@/stores';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Switch } from '@/components/ui/switch';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible';
import {
  Bot,
  Edit2,
  Trash2,
  CheckCircle,
  Loader2,
  Sparkles,
  Wrench,
  ChevronDown,
  ChevronUp,
  Brain,
  MessageSquare,
  PowerOff,
  Clock,
} from 'lucide-react';
import type { AgentConfig } from '@/types';

interface AgentCardProps {
  agent: AgentConfig;
  onEdit?: (agent: AgentConfig) => void;
  onDelete?: (id: string) => void;
  viewMode?: 'card' | 'list';
}

// 使用 memo 优化 AgentCard 组件
export const AgentCard = memo(function AgentCard({ agent, onEdit, onDelete, viewMode = 'card' }: AgentCardProps) {
  const queryClient = useQueryClient();
  const { addNotification } = useAppStore();
  const { currentAgentId, setCurrentAgent } = useConfigStore();
  const [enabled, setEnabled] = useState(agent.enabled);
  const [isExpanded, setIsExpanded] = useState(false);

  const isCurrent = currentAgentId === agent.id;

  // 启用/禁用 Agent - 使用 useCallback
  const enableMutation = useMutation({
    mutationFn: async (value: boolean) => {
      const result = await agentApi.saveAgent({ ...agent, enabled: value });
      if (result.success) {
        setEnabled(value);
      }
      return result;
    },
    onSuccess: (_, value) => {
      queryClient.invalidateQueries({ queryKey: ['agents'] });
      addNotification({
        title: value ? 'Agent 已启用' : 'Agent 已禁用',
        message: `${agent.name} 状态已更新`,
        type: 'success',
      });
    },
    onError: () => {
      setEnabled(!enabled);
      addNotification({
        title: '更新失败',
        message: '无法更新 Agent 状态',
        type: 'error',
      });
    },
  });

  // 切换当前 Agent - 使用 useCallback
  const switchMutation = useMutation({
    mutationFn: () => agentApi.setCurrentAgent(agent.id),
    onSuccess: () => {
      setCurrentAgent(agent.id);
      queryClient.invalidateQueries({ queryKey: ['agents'] });
      addNotification({
        title: 'Agent 已切换',
        message: `当前使用: ${agent.name}`,
        type: 'success',
      });
    },
    onError: (error) => {
      addNotification({
        title: '切换失败',
        message: String(error) || '无法切换到该 Agent',
        type: 'error',
      });
    },
  });

  const handleEnableChange = useCallback((checked: boolean) => {
    setEnabled(checked);
    enableMutation.mutate(checked);
  }, [enableMutation]);

  const handleSwitch = useCallback(() => {
    switchMutation.mutate();
  }, [switchMutation]);

  const handleEdit = useCallback(() => {
    onEdit?.(agent);
  }, [onEdit, agent]);

  const handleDelete = useCallback(() => {
    onDelete?.(agent.id);
  }, [onDelete, agent.id]);

  const handleExpand = useCallback(() => {
    setIsExpanded(prev => !prev);
  }, []);

  // 格式化日期 - 使用 useCallback
  const formatDate = useCallback((dateStr: string) => {
    try {
      const date = new Date(dateStr);
      return date.toLocaleDateString('zh-CN', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      });
    } catch {
      return dateStr;
    }
  }, []);

  // 列表视图
  if (viewMode === 'list') {
    return (
      <Collapsible open={isExpanded} onOpenChange={setIsExpanded}>
        <div
          className={`rounded-lg border transition-colors ${
            isCurrent
              ? 'border-primary bg-primary/5'
              : 'border-border hover:border-muted-foreground/25'
          }`}
        >
          <div className="flex items-center justify-between p-4">
            <div className="flex items-center gap-4">
              <div
                className={`p-2 rounded-lg ${
                  isCurrent
                    ? 'bg-primary text-primary-foreground'
                    : enabled
                    ? 'bg-muted'
                    : 'bg-muted/50 text-muted-foreground'
                }`}
              >
                <Bot className="h-5 w-5" />
              </div>
              <div>
                <div className="flex items-center gap-2">
                  <h4 className="font-medium">{agent.name}</h4>
                  {isCurrent && (
                    <Badge variant="default" className="text-xs">
                      <CheckCircle className="h-3 w-3 mr-1" />
                      当前
                    </Badge>
                  )}
                  {!enabled && (
                    <Badge variant="secondary" className="text-xs">
                      <PowerOff className="h-3 w-3 mr-1" />
                      已禁用
                    </Badge>
                  )}
                </div>
                <p className="text-sm text-muted-foreground">
                  {agent.description || '无描述'}
                </p>
              </div>
            </div>

            <div className="flex items-center gap-4">
              <div className="flex items-center gap-4 text-sm text-muted-foreground">
                <span className="flex items-center gap-1">
                  <Wrench className="h-4 w-4" />
                  {agent.skills.length} 技能
                </span>
                <span className="flex items-center gap-1">
                  <Brain className="h-4 w-4" />
                  {agent.modelId || '默认'}
                </span>
              </div>

              <div className="flex items-center gap-2">
                <Switch
                  checked={enabled}
                  onCheckedChange={handleEnableChange}
                  disabled={enableMutation.isPending}
                />

                <CollapsibleTrigger asChild>
                  <Button variant="ghost" size="icon" className="h-8 w-8">
                    {isExpanded ? (
                      <ChevronUp className="h-4 w-4" />
                    ) : (
                      <ChevronDown className="h-4 w-4" />
                    )}
                  </Button>
                </CollapsibleTrigger>

                {!isCurrent && enabled && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleSwitch}
                    disabled={switchMutation.isPending}
                  >
                    {switchMutation.isPending ? (
                      <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    ) : (
                      <Sparkles className="h-4 w-4 mr-2" />
                    )}
                    切换
                  </Button>
                )}

                {onEdit && (
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8"
                    onClick={handleEdit}
                  >
                    <Edit2 className="h-4 w-4" />
                  </Button>
                )}

                {onDelete && !isCurrent && (
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8 text-destructive hover:text-destructive"
                    onClick={handleDelete}
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                )}
              </div>
            </div>
          </div>

          <CollapsibleContent>
            <Separator />
            <div className="p-4 space-y-3">
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-muted-foreground">ID:</span>
                  <code className="ml-2 text-xs bg-muted px-1 py-0.5 rounded">
                    {agent.id}
                  </code>
                </div>
                <div>
                  <span className="text-muted-foreground">模型:</span>
                  <span className="ml-2">{agent.modelId || '默认模型'}</span>
                </div>
                {agent.createdAt && (
                  <div className="flex items-center gap-1">
                    <Clock className="h-3 w-3 text-muted-foreground" />
                    <span className="text-muted-foreground">创建时间:</span>
                    <span className="ml-2">{formatDate(agent.createdAt)}</span>
                  </div>
                )}
                {agent.updatedAt && (
                  <div className="flex items-center gap-1">
                    <Clock className="h-3 w-3 text-muted-foreground" />
                    <span className="text-muted-foreground">更新时间:</span>
                    <span className="ml-2">{formatDate(agent.updatedAt)}</span>
                  </div>
                )}
              </div>

              {agent.systemPrompt && (
                <div className="space-y-1">
                  <span className="text-sm text-muted-foreground flex items-center gap-1">
                    <MessageSquare className="h-3 w-3" />
                    系统提示词:
                  </span>
                  <div className="text-sm bg-muted p-2 rounded max-h-24 overflow-y-auto">
                    {agent.systemPrompt}
                  </div>
                </div>
              )}

              {agent.skills.length > 0 && (
                <div className="space-y-1">
                  <span className="text-sm text-muted-foreground">技能列表:</span>
                  <div className="flex flex-wrap gap-1">
                    {agent.skills.map((skill) => (
                      <Badge key={skill} variant="outline" className="text-xs">
                        {skill}
                      </Badge>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </CollapsibleContent>
        </div>
      </Collapsible>
    );
  }

  // 卡片视图
  return (
    <Collapsible open={isExpanded} onOpenChange={setIsExpanded}>
      <Card className={`transition-colors ${isCurrent ? 'border-primary ring-1 ring-primary' : ''}`}>
        <CardHeader className="pb-3">
          <div className="flex items-start justify-between">
            <div className="flex items-center gap-3">
              <div
                className={`p-2 rounded-lg ${
                  isCurrent
                    ? 'bg-primary text-primary-foreground'
                    : enabled
                    ? 'bg-muted'
                    : 'bg-muted/50 text-muted-foreground'
                }`}
              >
                <Bot className="h-5 w-5" />
              </div>
              <div>
                <CardTitle className="text-base flex items-center gap-2">
                  {agent.name}
                  {isCurrent && (
                    <Badge variant="default" className="text-xs">
                      <CheckCircle className="h-3 w-3 mr-1" />
                      当前
                    </Badge>
                  )}
                  {!enabled && (
                    <Badge variant="secondary" className="text-xs">
                      <PowerOff className="h-3 w-3 mr-1" />
                      已禁用
                    </Badge>
                  )}
                </CardTitle>
                <CardDescription className="text-xs line-clamp-1">
                  {agent.description || '无描述'}
                </CardDescription>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <Switch
                checked={enabled}
                onCheckedChange={handleEnableChange}
                disabled={enableMutation.isPending}
              />
              <CollapsibleTrigger asChild>
                <Button variant="ghost" size="icon" className="h-8 w-8">
                  {isExpanded ? (
                    <ChevronUp className="h-4 w-4" />
                  ) : (
                    <ChevronDown className="h-4 w-4" />
                  )}
                </Button>
              </CollapsibleTrigger>
            </div>
          </div>
        </CardHeader>

        <CardContent className="space-y-4">
          {/* 基本信息 */}
          <div className="space-y-2 text-sm">
            <div className="flex items-center gap-2">
              <Brain className="h-4 w-4 text-muted-foreground" />
              <span className="text-muted-foreground">模型:</span>
              <span className="truncate">{agent.modelId || '默认模型'}</span>
            </div>
            <div className="flex items-center gap-2">
              <Wrench className="h-4 w-4 text-muted-foreground" />
              <span className="text-muted-foreground">技能:</span>
              <span>{agent.skills.length} 个</span>
            </div>
          </div>

          <CollapsibleContent>
            <Separator className="my-3" />
            <div className="space-y-3 text-sm">
              <div>
                <span className="text-muted-foreground">ID:</span>
                <code className="ml-2 text-xs bg-muted px-1 py-0.5 rounded">
                  {agent.id}
                </code>
              </div>

              {agent.systemPrompt && (
                <div className="space-y-1">
                  <span className="text-muted-foreground flex items-center gap-1">
                    <MessageSquare className="h-3 w-3" />
                    系统提示词:
                  </span>
                  <div className="bg-muted p-2 rounded text-xs max-h-20 overflow-y-auto">
                    {agent.systemPrompt}
                  </div>
                </div>
              )}

              {agent.skills.length > 0 && (
                <div className="space-y-1">
                  <span className="text-muted-foreground">技能:</span>
                  <div className="flex flex-wrap gap-1">
                    {agent.skills.map((skill) => (
                      <Badge key={skill} variant="outline" className="text-xs">
                        {skill}
                      </Badge>
                    ))}
                  </div>
                </div>
              )}

              {(agent.createdAt || agent.updatedAt) && (
                <div className="flex gap-4 text-xs text-muted-foreground">
                  {agent.createdAt && (
                    <span className="flex items-center gap-1">
                      <Clock className="h-3 w-3" />
                      创建于 {formatDate(agent.createdAt)}
                    </span>
                  )}
                </div>
              )}
            </div>
          </CollapsibleContent>

          {/* 操作按钮 */}
          <div className="flex gap-2 pt-2">
            {!isCurrent && enabled && (
              <Button
                variant="default"
                size="sm"
                className="flex-1"
                onClick={handleSwitch}
                disabled={switchMutation.isPending}
              >
                {switchMutation.isPending ? (
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                ) : (
                  <Sparkles className="h-4 w-4 mr-2" />
                )}
                切换到此 Agent
              </Button>
            )}

            {onEdit && (
              <Button
                variant="outline"
                size="sm"
                className="flex-1"
                onClick={handleEdit}
              >
                <Edit2 className="h-4 w-4 mr-2" />
                编辑
              </Button>
            )}

            {onDelete && !isCurrent && (
              <Button
                variant="outline"
                size="sm"
                className="flex-1 text-destructive hover:text-destructive"
                onClick={handleDelete}
              >
                <Trash2 className="h-4 w-4 mr-2" />
                删除
              </Button>
            )}
          </div>
        </CardContent>
      </Card>
    </Collapsible>
  );
});
