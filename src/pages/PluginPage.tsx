import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { pluginApi, pluginMarketApi } from '@/lib/tauri-api'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import {
  Puzzle,
  Download,
  Trash2,
  Power,
  PowerOff,
  Search,
  Star,
  DownloadIcon,
  ChevronRight,
  Grid3X3,
  List,
  RefreshCw,
} from 'lucide-react'
import type { Plugin, MarketPlugin } from '@/types'

type ViewMode = 'grid' | 'list'
type SortOption = 'relevance' | 'downloads' | 'rating' | 'created_at' | 'updated_at'

export function PluginPage() {
  const queryClient = useQueryClient()
  const [activeTab, setActiveTab] = useState('installed')
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedCategory, setSelectedCategory] = useState<string>('all')
  const [sortBy, setSortBy] = useState<SortOption>('relevance')
  const [viewMode, setViewMode] = useState<ViewMode>('grid')
  const [selectedPlugin, setSelectedPlugin] = useState<MarketPlugin | null>(null)
  const [isDetailOpen, setIsDetailOpen] = useState(false)

  // 获取已安装插件
  const { data: installedData, isLoading: isLoadingInstalled } = useQuery({
    queryKey: ['plugins', 'installed'],
    queryFn: () => pluginApi.getAll(),
  })

  // 获取插件分类
  const { data: categoriesData } = useQuery({
    queryKey: ['plugins', 'categories'],
    queryFn: () => pluginMarketApi.getCategories(),
  })

  // 搜索市场插件
  const { data: searchData, isLoading: isLoadingSearch } = useQuery({
    queryKey: ['plugins', 'market', 'search', searchQuery, selectedCategory, sortBy],
    queryFn: () =>
      pluginMarketApi.search({
        query: searchQuery || undefined,
        category: selectedCategory === 'all' ? undefined : selectedCategory,
        sortBy,
        page: 1,
        perPage: 20,
      }),
    enabled: activeTab === 'market',
  })

  // 获取热门插件
  const { data: popularData } = useQuery({
    queryKey: ['plugins', 'market', 'popular'],
    queryFn: () => pluginMarketApi.getPopular(6),
    enabled: activeTab === 'market' && !searchQuery,
  })

  // 获取最新插件
  const { data: latestData } = useQuery({
    queryKey: ['plugins', 'market', 'latest'],
    queryFn: () => pluginMarketApi.getLatest(6),
    enabled: activeTab === 'market' && !searchQuery,
  })

  // 变异操作
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

  const installMutation = useMutation({
    mutationFn: (plugin: MarketPlugin) =>
      pluginApi.install(plugin.id, plugin.download_url),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['plugins'] })
      setIsDetailOpen(false)
    },
  })

  const installedPlugins = installedData || []
  const marketPlugins = searchData?.plugins || []
  const categories = categoriesData || []
  const popularPlugins = popularData || []
  const latestPlugins = latestData || []

  const handlePluginClick = (plugin: MarketPlugin) => {
    setSelectedPlugin(plugin)
    setIsDetailOpen(true)
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">插件中心</h2>
          <p className="text-muted-foreground">
            管理和安装插件以扩展 OpenClaw 功能
          </p>
        </div>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="installed">
            <Puzzle className="mr-2 h-4 w-4" />
            已安装
            {installedPlugins.length > 0 && (
              <Badge variant="secondary" className="ml-2">
                {installedPlugins.length}
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="market">
            <Download className="mr-2 h-4 w-4" />
            插件市场
          </TabsTrigger>
        </TabsList>

        {/* 已安装插件 */}
        <TabsContent value="installed" className="space-y-4">
          {isLoadingInstalled ? (
            <div className="flex h-64 items-center justify-center">
              <RefreshCw className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : installedPlugins.length === 0 ? (
            <EmptyInstalledState onBrowseMarket={() => setActiveTab('market')} />
          ) : (
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {installedPlugins.map((plugin: Plugin) => (
                <InstalledPluginCard
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
        </TabsContent>

        {/* 插件市场 */}
        <TabsContent value="market" className="space-y-6">
          {/* 搜索和筛选 */}
          <div className="flex flex-col gap-4 sm:flex-row">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder="搜索插件..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-10"
              />
            </div>
            <Select value={selectedCategory} onValueChange={setSelectedCategory}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder="选择分类" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部分类</SelectItem>
                {categories.map((cat: { id: string; name: string }) => (
                  <SelectItem key={cat.id} value={cat.id}>
                    {cat.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={sortBy} onValueChange={(v) => setSortBy(v as SortOption)}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder="排序方式" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="relevance">相关度</SelectItem>
                <SelectItem value="downloads">下载量</SelectItem>
                <SelectItem value="rating">评分</SelectItem>
                <SelectItem value="updated_at">最近更新</SelectItem>
              </SelectContent>
            </Select>
            <div className="flex items-center gap-2">
              <Button
                variant={viewMode === 'grid' ? 'default' : 'outline'}
                size="icon"
                onClick={() => setViewMode('grid')}
              >
                <Grid3X3 className="h-4 w-4" />
              </Button>
              <Button
                variant={viewMode === 'list' ? 'default' : 'outline'}
                size="icon"
                onClick={() => setViewMode('list')}
              >
                <List className="h-4 w-4" />
              </Button>
            </div>
          </div>

          {/* 搜索结果或推荐 */}
          {isLoadingSearch ? (
            <div className="flex h-64 items-center justify-center">
              <RefreshCw className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : searchQuery ? (
            <div className={viewMode === 'grid' ? 'grid gap-4 md:grid-cols-2 lg:grid-cols-3' : 'space-y-2'}>
              {marketPlugins.map((plugin: MarketPlugin) => (
                <MarketPluginCard
                  key={plugin.id}
                  plugin={plugin}
                  viewMode={viewMode}
                  onClick={() => handlePluginClick(plugin)}
                />
              ))}
            </div>
          ) : (
            <>
              {/* 热门插件 */}
              {popularPlugins.length > 0 && (
                <section className="space-y-3">
                  <h3 className="text-lg font-semibold">热门插件</h3>
                  <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                    {popularPlugins.map((plugin: MarketPlugin) => (
                      <MarketPluginCard
                        key={plugin.id}
                        plugin={plugin}
                        viewMode="grid"
                        onClick={() => handlePluginClick(plugin)}
                      />
                    ))}
                  </div>
                </section>
              )}

              {/* 最新插件 */}
              {latestPlugins.length > 0 && (
                <section className="space-y-3">
                  <h3 className="text-lg font-semibold">最新发布</h3>
                  <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                    {latestPlugins.map((plugin: MarketPlugin) => (
                      <MarketPluginCard
                        key={plugin.id}
                        plugin={plugin}
                        viewMode="grid"
                        onClick={() => handlePluginClick(plugin)}
                      />
                    ))}
                  </div>
                </section>
              )}
            </>
          )}
        </TabsContent>
      </Tabs>

      {/* 插件详情弹窗 */}
      <Dialog open={isDetailOpen} onOpenChange={setIsDetailOpen}>
        <DialogContent className="max-w-2xl">
          {selectedPlugin && (
            <>
              <DialogHeader>
                <div className="flex items-start gap-4">
                  <div className="flex h-16 w-16 items-center justify-center rounded-lg bg-muted">
                    <Puzzle className="h-8 w-8 text-muted-foreground" />
                  </div>
                  <div className="flex-1">
                    <DialogTitle className="text-xl">{selectedPlugin.name}</DialogTitle>
                    <DialogDescription className="mt-1">
                      v{selectedPlugin.version} by {selectedPlugin.author || 'Unknown'}
                    </DialogDescription>
                    <div className="mt-2 flex flex-wrap gap-2">
                      {selectedPlugin.categories.map((cat) => (
                        <Badge key={cat} variant="secondary">
                          {cat}
                        </Badge>
                      ))}
                    </div>
                  </div>
                </div>
              </DialogHeader>

              <ScrollArea className="max-h-[400px]">
                <div className="space-y-4 pr-4">
                  <p className="text-muted-foreground">
                    {selectedPlugin.description}
                  </p>

                  <div className="grid grid-cols-3 gap-4">
                    <StatCard
                      icon={<DownloadIcon className="h-4 w-4" />}
                      value={formatNumber(selectedPlugin.downloads)}
                      label="下载量"
                    />
                    <StatCard
                      icon={<Star className="h-4 w-4" />}
                      value={selectedPlugin.rating.toFixed(1)}
                      label={`${selectedPlugin.rating_count} 评分`}
                    />
                    <StatCard
                      icon={<Puzzle className="h-4 w-4" />}
                      value={formatSize(selectedPlugin.size_bytes)}
                      label="大小"
                    />
                  </div>

                  {selectedPlugin.changelog && (
                    <>
                      <Separator />
                      <div>
                        <h4 className="mb-2 font-semibold">更新日志</h4>
                        <pre className="whitespace-pre-wrap text-sm text-muted-foreground">
                          {selectedPlugin.changelog}
                        </pre>
                      </div>
                    </>
                  )}

                  <Separator />

                  <div className="flex items-center justify-between text-sm text-muted-foreground">
                    <span>发布于 {formatDate(selectedPlugin.created_at)}</span>
                    <span>更新于 {formatDate(selectedPlugin.updated_at)}</span>
                  </div>
                </div>
              </ScrollArea>

              <div className="flex justify-end gap-2">
                <Button variant="outline" onClick={() => setIsDetailOpen(false)}>
                  取消
                </Button>
                <Button
                  onClick={() => installMutation.mutate(selectedPlugin)}
                  disabled={installMutation.isPending}
                >
                  {installMutation.isPending ? (
                    <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                  ) : (
                    <Download className="mr-2 h-4 w-4" />
                  )}
                  安装
                </Button>
              </div>
            </>
          )}
        </DialogContent>
      </Dialog>
    </div>
  )
}

// 空状态组件
function EmptyInstalledState({ onBrowseMarket }: { onBrowseMarket: () => void }) {
  return (
    <div className="flex h-64 flex-col items-center justify-center text-muted-foreground">
      <Puzzle className="mb-4 h-12 w-12" />
      <p>暂无已安装插件</p>
      <p className="mb-4 text-sm">前往插件市场浏览和安装插件</p>
      <Button onClick={onBrowseMarket}>
        <Download className="mr-2 h-4 w-4" />
        浏览插件市场
      </Button>
    </div>
  )
}

// 已安装插件卡片
interface InstalledPluginCardProps {
  plugin: Plugin
  onEnable: () => void
  onDisable: () => void
  onUninstall: () => void
  isLoading: boolean
}

function InstalledPluginCard({
  plugin,
  onEnable,
  onDisable,
  onUninstall,
  isLoading,
}: InstalledPluginCardProps) {
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

// 市场插件卡片
interface MarketPluginCardProps {
  plugin: MarketPlugin
  viewMode: ViewMode
  onClick: () => void
}

function MarketPluginCard({ plugin, viewMode, onClick }: MarketPluginCardProps) {
  if (viewMode === 'list') {
    return (
      <Card className="cursor-pointer transition-colors hover:bg-muted/50" onClick={onClick}>
        <CardContent className="flex items-center gap-4 p-4">
          <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-muted">
            <Puzzle className="h-6 w-6 text-muted-foreground" />
          </div>
          <div className="flex-1 min-w-0">
            <h4 className="font-semibold truncate">{plugin.name}</h4>
            <p className="text-sm text-muted-foreground truncate">
              {plugin.description}
            </p>
          </div>
          <div className="flex items-center gap-4 text-sm text-muted-foreground">
            <span className="flex items-center gap-1">
              <Star className="h-4 w-4" />
              {plugin.rating.toFixed(1)}
            </span>
            <span className="flex items-center gap-1">
              <DownloadIcon className="h-4 w-4" />
              {formatNumber(plugin.downloads)}
            </span>
          </div>
          <ChevronRight className="h-5 w-5 text-muted-foreground" />
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className="cursor-pointer transition-colors hover:bg-muted/50" onClick={onClick}>
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-3">
            <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-muted">
              <Puzzle className="h-6 w-6 text-muted-foreground" />
            </div>
            <div>
              <CardTitle className="text-base">{plugin.name}</CardTitle>
              <CardDescription>v{plugin.version}</CardDescription>
            </div>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-sm text-muted-foreground line-clamp-2">
          {plugin.description}
        </p>
        <div className="flex flex-wrap gap-1">
          {plugin.tags.slice(0, 3).map((tag) => (
            <Badge key={tag} variant="outline" className="text-xs">
              {tag}
            </Badge>
          ))}
        </div>
        <div className="flex items-center justify-between text-sm text-muted-foreground">
          <span className="flex items-center gap-1">
            <Star className="h-4 w-4" />
            {plugin.rating.toFixed(1)}
          </span>
          <span className="flex items-center gap-1">
            <DownloadIcon className="h-4 w-4" />
            {formatNumber(plugin.downloads)}
          </span>
          <span>{formatSize(plugin.size_bytes)}</span>
        </div>
      </CardContent>
    </Card>
  )
}

// 统计卡片
function StatCard({
  icon,
  value,
  label,
}: {
  icon: React.ReactNode
  value: string
  label: string
}) {
  return (
    <div className="flex flex-col items-center rounded-lg border p-3">
      <div className="mb-1 text-muted-foreground">{icon}</div>
      <div className="text-lg font-semibold">{value}</div>
      <div className="text-xs text-muted-foreground">{label}</div>
    </div>
  )
}

// 工具函数
function formatNumber(num: number): string {
  if (num >= 10000) {
    return (num / 10000).toFixed(1) + 'w'
  }
  if (num >= 1000) {
    return (num / 1000).toFixed(1) + 'k'
  }
  return num.toString()
}

function formatSize(bytes: number): string {
  if (bytes >= 1024 * 1024) {
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  }
  if (bytes >= 1024) {
    return (bytes / 1024).toFixed(1) + ' KB'
  }
  return bytes + ' B'
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}
