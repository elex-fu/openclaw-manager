import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { pluginApi } from '@/lib/tauri-api'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Puzzle, Download, Trash2, Power, PowerOff } from 'lucide-react'
import type { Plugin } from '@/types'

export function PluginPage() {
  const queryClient = useQueryClient()

  const { data: pluginsData, isLoading } = useQuery({
    queryKey: ['plugins'],
    queryFn: () => pluginApi.getAll(),
  })

  const enableMutation = useMutation({
    mutationFn: (id: string) => pluginApi.enable(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['plugins'] }),
  })

  const disableMutation = useMutation({
    mutationFn: (id: string) => pluginApi.disable(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['plugins'] }),
  })

  const uninstallMutation = useMutation({
    mutationFn: (id: string) => pluginApi.uninstall(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['plugins'] }),
  })

  const plugins = pluginsData?.data || []

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">插件中心</h2>
          <p className="text-muted-foreground">
            管理和安装插件以扩展功能
          </p>
        </div>
        <Button>
          <Download className="mr-2 h-4 w-4" />
          浏览插件市场
        </Button>
      </div>

      {isLoading ? (
        <div className="flex h-64 items-center justify-center">
          <div className="text-muted-foreground">加载中...</div>
        </div>
      ) : plugins.length === 0 ? (
        <div className="flex h-64 flex-col items-center justify-center text-muted-foreground">
          <Puzzle className="mb-4 h-12 w-12" />
          <p>暂无已安装插件</p>
          <p className="text-sm">前往插件市场浏览和安装插件</p>
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {plugins.map((plugin) => (
            <PluginCard
              key={plugin.id}
              plugin={plugin}
              onEnable={() => enableMutation.mutate(plugin.id)}
              onDisable={() => disableMutation.mutate(plugin.id)}
              onUninstall={() => uninstallMutation.mutate(plugin.id)}
              isLoading={
                enableMutation.isPending ||
                disableMutation.isPending ||
                uninstallMutation.isPending
              }
            />
          ))}
        </div>
      )}
    </div>
  )
}

interface PluginCardProps {
  plugin: Plugin
  onEnable: () => void
  onDisable: () => void
  onUninstall: () => void
  isLoading: boolean
}

function PluginCard({
  plugin,
  onEnable,
  onDisable,
  onUninstall,
  isLoading,
}: PluginCardProps) {
  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between">
          <div>
            <CardTitle className="text-lg">{plugin.name}</CardTitle>
            <CardDescription>v{plugin.version}</CardDescription>
          </div>
          <Badge variant={plugin.is_enabled ? 'default' : 'secondary'}>
            {plugin.is_enabled ? '已启用' : '已禁用'}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {plugin.description && (
          <p className="text-sm text-muted-foreground">{plugin.description}</p>
        )}
        {plugin.author && (
          <p className="text-xs text-muted-foreground">作者: {plugin.author}</p>
        )}
        <div className="flex gap-2">
          {plugin.is_enabled ? (
            <Button
              variant="outline"
              size="sm"
              onClick={onDisable}
              disabled={isLoading}
            >
              <PowerOff className="mr-2 h-4 w-4" />
              禁用
            </Button>
          ) : (
            <Button size="sm" onClick={onEnable} disabled={isLoading}>
              <Power className="mr-2 h-4 w-4" />
              启用
            </Button>
          )}
          <Button
            variant="ghost"
            size="sm"
            onClick={onUninstall}
            disabled={isLoading}
          >
            <Trash2 className="mr-2 h-4 w-4" />
            卸载
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}
