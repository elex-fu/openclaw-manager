import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { agentApi } from '@/lib/tauri-api';
import { useConfigStore, useAppStore } from '@/stores';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Switch } from '@/components/ui/switch';
import { Badge } from '@/components/ui/badge';
import { 
  Bot, 
  Edit2, 
  Trash2, 
  CheckCircle,
  Loader2,
  Sparkles,
  Wrench
} from 'lucide-react';
import type { AgentConfig } from '@/types';

interface AgentCardProps {
  agent: AgentConfig;
  onEdit?: (agent: AgentConfig) => void;
  onDelete?: (id: string) => void;
  viewMode?: 'card' | 'list';
}

export function AgentCard({ agent, onEdit, onDelete, viewMode = 'card' }: AgentCardProps) {
  const queryClient = useQueryClient();
  const { addNotification } = useAppStore();
  const { currentAgentId, setCurrentAgent } = useConfigStore();
  const [enabled, setEnabled] = useState(agent.enabled);

  const isCurrent = currentAgentId === agent.id;

  const enableMutation = useMutation({
    mutationFn: async (value: boolean) => {
      const result = await agentApi.saveAgent({ ...agent, enabled: value });
      if (result.success) {
        setEnabled(value);
      }
      return result;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agents'] });
      addNotification({
        title: enabled ? 'Agent 已启用' : 'Agent 已禁用',
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

  const switchMutation = useMutation({
    mutationFn: () => agentApi.setCurrentAgent(agent.id),
    onSuccess: () => {
      setCurrentAgent(agent.id);
      addNotification({
        title: 'Agent 已切换',
        message: `当前使用: ${agent.name}`,
        type: 'success',
      });
    },
  });

  const handleEnableChange = (checked: boolean) => {
    setEnabled(checked);
    enableMutation.mutate(checked);
  };

  if (viewMode === 'list') {
    return (
      <div className={`flex items-center justify-between p-4 rounded-lg border ${isCurrent ? 'border-primary bg-primary/5' : ''}`}>
        <div className="flex items-center gap-4">
          <div className={`p-2 rounded-lg ${isCurrent ? 'bg-primary text-primary-foreground' : 'bg-muted'}`}>
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
                <Badge variant="secondary" className="text-xs">已禁用</Badge>
              )}
            </div>
            <p className="text-sm text-muted-foreground">{agent.description || '无描述'}</p>
          </div>
        </div>

        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <span className="text-sm text-muted-foreground">技能: {agent.skills.length}</span>
          </div>
          
          <div className="flex items-center gap-2">
            <Switch
              checked={enabled}
              onCheckedChange={handleEnableChange}
              disabled={enableMutation.isPending}
            />
            
            {!isCurrent && enabled && (
              <Button
                variant="outline"
                size="sm"
                onClick={() => switchMutation.mutate()}
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
              <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => onEdit(agent)}>
                <Edit2 className="h-4 w-4" />
              </Button>
            )}

            {onDelete && !isCurrent && (
              <Button 
                variant="ghost" 
                size="icon" 
                className="h-8 w-8 text-destructive hover:text-destructive"
                onClick={() => onDelete(agent.id)}
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            )}
          </div>
        </div>
      </div>
    );
  }

  return (
    <Card className={isCurrent ? 'border-primary' : ''}>
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-3">
            <div className={`p-2 rounded-lg ${isCurrent ? 'bg-primary text-primary-foreground' : 'bg-muted'}`}>
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
              </CardTitle>
              <CardDescription className="text-xs line-clamp-1">
                {agent.description || '无描述'}
              </CardDescription>
            </div>
          </div>
          <Switch
            checked={enabled}
            onCheckedChange={handleEnableChange}
            disabled={enableMutation.isPending}
          />
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        {/* 技能列表 */}
        <div className="flex items-center gap-2">
          <Wrench className="h-4 w-4 text-muted-foreground" />
          <span className="text-sm text-muted-foreground">技能: {agent.skills.length}</span>
        </div>

        {/* 操作按钮 */}
        <div className="flex gap-2 pt-2">
          {!isCurrent && enabled && (
            <Button
              variant="default"
              size="sm"
              className="flex-1"
              onClick={() => switchMutation.mutate()}
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
              onClick={() => onEdit(agent)}
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
              onClick={() => onDelete(agent.id)}
            >
              <Trash2 className="h-4 w-4 mr-2" />
              删除
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
