import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { modelApi, secureStorageApi } from '@/lib/tauri-api';
import { useConfigStore, useAppStore } from '@/stores';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Badge } from '@/components/ui/badge';
import { 
  Key, 
  Check, 
  X, 
  Loader2, 
  TestTube,
  Trash2,
  Edit2,
  Star,
  Server
} from 'lucide-react';
import type { ModelConfig } from '@/types';

interface ModelConfigCardProps {
  model: ModelConfig;
  onEdit?: (model: ModelConfig) => void;
  onDelete?: (id: string) => void;
}

export function ModelConfigCard({ model, onEdit, onDelete }: ModelConfigCardProps) {
  const queryClient = useQueryClient();
  const { addNotification } = useAppStore();
  const { apiKeyCache, setApiKey, removeApiKey } = useConfigStore();
  const [apiKeyInput, setApiKeyInput] = useState('');
  const [showApiKey, setShowApiKey] = useState(false);
  const [isEditingKey, setIsEditingKey] = useState(false);

  const cachedKey = apiKeyCache[model.id];

  const testMutation = useMutation({
    mutationFn: () => modelApi.testModelConnection(model.id),
    onSuccess: (result) => {
      if (result.success && result.data) {
        addNotification({
          title: '连接测试成功',
          message: `延迟: ${result.data.latency}ms`,
          type: 'success',
        });
      } else {
        addNotification({
          title: '连接测试失败',
          message: result.error || '无法连接到模型服务',
          type: 'error',
        });
      }
    },
    onError: (error) => {
      addNotification({
        title: '连接测试失败',
        message: error instanceof Error ? error.message : '未知错误',
        type: 'error',
      });
    },
  });

  const saveKeyMutation = useMutation({
    mutationFn: async () => {
      // 保存到安全存储
      const result = await secureStorageApi.saveApiKey(model.id, apiKeyInput);
      if (result.success) {
        // 同时缓存到内存
        setApiKey(model.id, apiKeyInput);
      }
      return result;
    },
    onSuccess: () => {
      addNotification({
        title: 'API Key 已保存',
        message: '您的 API Key 已安全存储',
        type: 'success',
      });
      setIsEditingKey(false);
      setApiKeyInput('');
    },
    onError: () => {
      addNotification({
        title: '保存失败',
        message: '无法保存 API Key',
        type: 'error',
      });
    },
  });

  const deleteKeyMutation = useMutation({
    mutationFn: async () => {
      const result = await secureStorageApi.deleteApiKey(model.id);
      if (result.success) {
        removeApiKey(model.id);
      }
      return result;
    },
    onSuccess: () => {
      addNotification({
        title: 'API Key 已删除',
        message: 'API Key 已从安全存储中移除',
        type: 'info',
      });
    },
  });

  const defaultMutation = useMutation({
    mutationFn: () => modelApi.setDefaultModel(model.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['models'] });
      addNotification({
        title: '默认模型已更新',
        message: `${model.name} 已设为默认模型`,
        type: 'success',
      });
    },
  });

  const hasApiKey = !!cachedKey;

  return (
    <Card className={model.isDefault ? 'border-primary' : ''}>
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2">
            <div className="p-2 rounded-lg bg-primary/10">
              <Server className="h-4 w-4 text-primary" />
            </div>
            <div>
              <CardTitle className="text-base flex items-center gap-2">
                {model.name}
                {model.isDefault && (
                  <Badge variant="default" className="text-xs">
                    <Star className="h-3 w-3 mr-1" />
                    默认
                  </Badge>
                )}
              </CardTitle>
              <CardDescription className="text-xs">
                {model.provider} / {model.model}
              </CardDescription>
            </div>
          </div>
          <div className="flex items-center gap-1">
            {onEdit && (
              <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => onEdit(model)}>
                <Edit2 className="h-4 w-4" />
              </Button>
            )}
            {onDelete && !model.isDefault && (
              <Button 
                variant="ghost" 
                size="icon" 
                className="h-8 w-8 text-destructive hover:text-destructive"
                onClick={() => onDelete(model.id)}
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            )}
          </div>
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        {/* API Key 设置 */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <Label className="text-sm flex items-center gap-2">
              <Key className="h-4 w-4" />
              API Key
            </Label>
            {hasApiKey && !isEditingKey && (
              <div className="flex items-center gap-1">
                <Badge variant="outline" className="text-xs">
                  <Check className="h-3 w-3 mr-1 text-green-500" />
                  已设置
                </Badge>
                <Button 
                  variant="ghost" 
                  size="sm" 
                  className="h-7 text-xs"
                  onClick={() => setIsEditingKey(true)}
                >
                  修改
                </Button>
                <Button 
                  variant="ghost" 
                  size="sm" 
                  className="h-7 text-xs text-destructive"
                  onClick={() => deleteKeyMutation.mutate()}
                  disabled={deleteKeyMutation.isPending}
                >
                  {deleteKeyMutation.isPending ? (
                    <Loader2 className="h-3 w-3 animate-spin" />
                  ) : (
                    <X className="h-3 w-3" />
                  )}
                </Button>
              </div>
            )}
          </div>

          {isEditingKey ? (
            <div className="flex gap-2">
              <Input
                type={showApiKey ? 'text' : 'password'}
                placeholder="输入 API Key"
                value={apiKeyInput}
                onChange={(e) => setApiKeyInput(e.target.value)}
                className="flex-1"
              />
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowApiKey(!showApiKey)}
              >
                {showApiKey ? '隐藏' : '显示'}
              </Button>
              <Button
                size="sm"
                onClick={() => saveKeyMutation.mutate()}
                disabled={!apiKeyInput || saveKeyMutation.isPending}
              >
                {saveKeyMutation.isPending ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <Check className="h-4 w-4" />
                )}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  setIsEditingKey(false);
                  setApiKeyInput('');
                }}
              >
                <X className="h-4 w-4" />
              </Button>
            </div>
          ) : !hasApiKey ? (
            <Button 
              variant="outline" 
              className="w-full"
              onClick={() => setIsEditingKey(true)}
            >
              <Key className="h-4 w-4 mr-2" />
              设置 API Key
            </Button>
          ) : null}
        </div>

        {/* 模型参数 */}
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div>
            <span className="text-muted-foreground">Temperature</span>
            <p className="font-medium">{model.temperature}</p>
          </div>
          {model.max_tokens && (
            <div>
              <span className="text-muted-foreground">Max Tokens</span>
              <p className="font-medium">{model.max_tokens}</p>
            </div>
          )}
        </div>

        {/* 操作按钮 */}
        <div className="flex gap-2 pt-2">
          <Button
            variant="outline"
            size="sm"
            className="flex-1"
            onClick={() => testMutation.mutate()}
            disabled={testMutation.isPending || !hasApiKey}
          >
            {testMutation.isPending ? (
              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
            ) : (
              <TestTube className="h-4 w-4 mr-2" />
            )}
            连接测试
          </Button>
          
          {!model.isDefault && (
            <Button
              variant="outline"
              size="sm"
              className="flex-1"
              onClick={() => defaultMutation.mutate()}
              disabled={defaultMutation.isPending}
            >
              <Star className="h-4 w-4 mr-2" />
              设为默认
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
