import { useState, useMemo } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import { SkeletonGrid } from '@/components/ui/skeleton'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import {
  Search,
  Code,
  PenTool,
  BarChart,
  Image,
  Zap,
  MessageCircle,
  LayoutGrid,
  Star,
  Check,
  RefreshCw,
  Sparkles,
  Cpu,
  Puzzle,
} from 'lucide-react'
import { skillApi } from '@/lib/tauri-api'
import { useAppStore } from '@/stores/appStore'
import { SkillCard } from '@/components/skill/SkillCard'
import { SkillDetailModal } from '@/components/skill/SkillDetailModal'
import { SkillConfigForm } from '@/components/skill/SkillConfigForm'
import { InstalledSkillList } from '@/components/skill/InstalledSkillList'
import { EmptySearchState } from '@/components/error'
import { StaggerContainer, StaggerItem, ScaleIn } from '@/components/animation'
import type { InstalledSkill, SkillMarketItem, SkillCardItem } from '@/types'

type ViewMode = 'market' | 'installed'
type SortOption = 'popular' | 'latest' | 'rating' | 'name'

// 分类图标映射
const categoryIcons: Record<string, React.ReactNode> = {
  all: <LayoutGrid className="h-4 w-4" />,
  programming: <Code className="h-4 w-4" />,
  writing: <PenTool className="h-4 w-4" />,
  data: <BarChart className="h-4 w-4" />,
  image: <Image className="h-4 w-4" />,
  productivity: <Zap className="h-4 w-4" />,
  communication: <MessageCircle className="h-4 w-4" />,
  search: <Search className="h-4 w-4" />,
  automation: <Cpu className="h-4 w-4" />,
}

export function SkillStore() {
  const queryClient = useQueryClient()
  const { addNotification } = useAppStore()

  // UI状态
  const [viewMode, setViewMode] = useState<ViewMode>('market')
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedCategory, setSelectedCategory] = useState('all')
  const [sortBy, setSortBy] = useState<SortOption>('popular')
  const [detailSkill, setDetailSkill] = useState<SkillCardItem | null>(null)
  const [configSkill, setConfigSkill] = useState<InstalledSkill | null>(null)
  const [uninstallSkillId, setUninstallSkillId] = useState<string | null>(null)

  // 查询技能分类
  const { data: categoriesData } = useQuery({
    queryKey: ['skillCategories'],
    queryFn: () => skillApi.getCategories(),
  })

  // 查询市场技能
  const { data: marketSkillsData, isLoading: isLoadingMarket } = useQuery({
    queryKey: ['marketSkills', searchQuery, selectedCategory, sortBy],
    queryFn: () =>
      skillApi.searchMarket({
        query: searchQuery || undefined,
        category: selectedCategory === 'all' ? undefined : selectedCategory,
        page: 1,
        perPage: 50,
      }),
  })

  // 查询已安装技能
  const { data: installedSkillsData, isLoading: isLoadingInstalled } = useQuery({
    queryKey: ['installedSkills'],
    queryFn: () => skillApi.getAll(),
  })

  // 查询热门技能
  const { data: popularSkillsData } = useQuery({
    queryKey: ['popularSkills'],
    queryFn: () => skillApi.getPopular(6),
  })

  const categories = categoriesData || []
  const marketSkills = marketSkillsData?.skills || []
  const installedSkills = installedSkillsData || []
  const popularSkills = popularSkillsData || []

  // 安装技能
  const installMutation = useMutation({
    mutationFn: (skillId: string) => skillApi.install(skillId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['installedSkills'] })
      queryClient.invalidateQueries({ queryKey: ['marketSkills'] })
      addNotification({
        title: '安装成功',
        message: '技能已成功安装',
        type: 'success',
      })
      setDetailSkill(null)
    },
    onError: (error) => {
      addNotification({
        title: '安装失败',
        message: String(error) || '请稍后重试',
        type: 'error',
      })
    },
  })

  // 卸载技能
  const uninstallMutation = useMutation({
    mutationFn: (skillId: string) => skillApi.uninstall(skillId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['installedSkills'] })
      queryClient.invalidateQueries({ queryKey: ['marketSkills'] })
      addNotification({
        title: '卸载成功',
        message: '技能已成功卸载',
        type: 'success',
      })
      setUninstallSkillId(null)
    },
    onError: (error) => {
      addNotification({
        title: '卸载失败',
        message: String(error) || '请稍后重试',
        type: 'error',
      })
    },
  })

  // 切换技能状态
  const toggleMutation = useMutation({
    mutationFn: ({ skillId, enabled }: { skillId: string; enabled: boolean }) =>
      skillApi.toggle(skillId, enabled),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['installedSkills'] })
      addNotification({
        title: variables.enabled ? '技能已启用' : '技能已禁用',
        message: variables.enabled ? '技能现在可以使用了' : '技能已被禁用',
        type: 'success',
      })
    },
    onError: (error) => {
      addNotification({
        title: '操作失败',
        message: String(error) || '请稍后重试',
        type: 'error',
      })
    },
  })

  // 更新技能配置
  const updateConfigMutation = useMutation({
    mutationFn: ({ skillId, config }: { skillId: string; config: Record<string, unknown> }) =>
      skillApi.updateConfig(skillId, config),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['installedSkills'] })
      addNotification({
        title: '配置已更新',
        message: '技能配置已成功保存',
        type: 'success',
      })
      setConfigSkill(null)
    },
    onError: (error) => {
      addNotification({
        title: '配置保存失败',
        message: String(error) || '请检查配置格式',
        type: 'error',
      })
    },
  })

  // 检查技能是否已安装
  const isSkillInstalled = (skillId: string) => {
    return installedSkills.some((s: InstalledSkill) => s.id === skillId)
  }

  // 获取已安装技能的启用状态
  const getSkillEnabledStatus = (skillId: string) => {
    const skill = installedSkills.find((s: InstalledSkill) => s.id === skillId)
    return skill?.is_enabled ?? false
  }

  // 统计信息
  const stats = useMemo(() => {
    const total = installedSkills.length
    const enabled = installedSkills.filter((s: InstalledSkill) => s.is_enabled).length
    const withUpdate = installedSkills.filter((s: InstalledSkill) => s.has_update).length
    return { total, enabled, disabled: total - enabled, withUpdate }
  }, [installedSkills])

  // 排序技能
  const sortedSkills = useMemo(() => {
    const skills = [...marketSkills]
    switch (sortBy) {
      case 'popular':
        return skills.sort((a, b) => b.downloads - a.downloads)
      case 'rating':
        return skills.sort((a, b) => b.rating - a.rating)
      case 'name':
        return skills.sort((a, b) => a.name.localeCompare(b.name))
      case 'latest':
      default:
        return skills.sort((a, b) =>
          new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
        )
    }
  }, [marketSkills, sortBy])

  // 处理安装
  const handleInstall = (skill: SkillCardItem) => {
    installMutation.mutate(skill.id)
  }

  // 处理卸载
  const handleUninstall = () => {
    if (uninstallSkillId) {
      uninstallMutation.mutate(uninstallSkillId)
    }
  }

  // 处理切换状态
  const handleToggle = (skill: InstalledSkill) => {
    toggleMutation.mutate({ skillId: skill.id, enabled: !skill.is_enabled })
  }

  // 处理配置保存
  const handleConfigSave = (config: Record<string, unknown>) => {
    if (configSkill) {
      updateConfigMutation.mutate({ skillId: configSkill.id, config })
    }
  }

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <ScaleIn>
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">技能商店</h1>
            <p className="text-muted-foreground">浏览、安装和管理 AI 技能</p>
          </div>
          <Tabs value={viewMode} onValueChange={(v) => setViewMode(v as ViewMode)}>
            <TabsList>
              <TabsTrigger value="market">
                <LayoutGrid className="h-4 w-4 mr-2" />
                技能市场
              </TabsTrigger>
              <TabsTrigger value="installed">
                <Puzzle className="h-4 w-4 mr-2" />
                已安装
                {stats.total > 0 && (
                  <Badge variant="secondary" className="ml-2">
                    {stats.total}
                  </Badge>
                )}
              </TabsTrigger>
            </TabsList>
          </Tabs>
        </div>
      </ScaleIn>

      {/* 统计卡片 */}
      <StaggerContainer className="grid gap-4 md:grid-cols-4">
        <StaggerItem>
          <Card className="hover:shadow-md transition-shadow">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">已安装技能</CardTitle>
              <Puzzle className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stats.total}</div>
            </CardContent>
          </Card>
        </StaggerItem>
        <StaggerItem>
          <Card className="hover:shadow-md transition-shadow">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">已启用</CardTitle>
              <Check className="h-4 w-4 text-green-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-green-600">{stats.enabled}</div>
            </CardContent>
          </Card>
        </StaggerItem>
        <StaggerItem>
          <Card className="hover:shadow-md transition-shadow">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">有更新</CardTitle>
              <RefreshCw className="h-4 w-4 text-blue-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-blue-600">{stats.withUpdate}</div>
            </CardContent>
          </Card>
        </StaggerItem>
        <StaggerItem>
          <Card className="hover:shadow-md transition-shadow">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">可用技能</CardTitle>
              <Sparkles className="h-4 w-4 text-primary" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{marketSkills.length}</div>
            </CardContent>
          </Card>
        </StaggerItem>
      </StaggerContainer>

      {/* 市场视图 */}
      {viewMode === 'market' && (
        <>
          {/* 搜索和筛选 */}
          <ScaleIn delay={0.1}>
            <div className="flex flex-col gap-4 sm:flex-row sm:items-center">
              <div className="relative flex-1">
                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  placeholder="搜索技能名称、描述或标签..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-9"
                />
              </div>
              <Select value={sortBy} onValueChange={(v) => setSortBy(v as SortOption)}>
                <SelectTrigger className="w-[180px]">
                  <SelectValue placeholder="排序方式" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="popular">按热度</SelectItem>
                  <SelectItem value="rating">按评分</SelectItem>
                  <SelectItem value="latest">按最新</SelectItem>
                  <SelectItem value="name">按名称</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </ScaleIn>

          {/* 分类标签 */}
          <ScaleIn delay={0.15}>
            <div className="flex flex-wrap gap-2">
              {categories.map((category: { id: string; name: string }) => (
                <Button
                  key={category.id}
                  variant={selectedCategory === category.id ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setSelectedCategory(category.id)}
                  className="gap-1.5"
                >
                  {categoryIcons[category.id] || <LayoutGrid className="h-4 w-4" />}
                  {category.name}
                </Button>
              ))}
            </div>
          </ScaleIn>

          {/* 热门技能（仅在未搜索时显示） */}
          {!searchQuery && selectedCategory === 'all' && popularSkills.length > 0 && (
            <ScaleIn delay={0.2}>
              <div className="space-y-3">
                <h2 className="text-lg font-semibold flex items-center gap-2">
                  <Star className="h-5 w-5 text-yellow-500" />
                  热门技能
                </h2>
                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                  {popularSkills.map((skill: SkillMarketItem) => (
                    <SkillCard
                      key={skill.id}
                      skill={skill}
                      isInstalled={isSkillInstalled(skill.id)}
                      isEnabled={getSkillEnabledStatus(skill.id)}
                      onInstall={() => handleInstall(skill)}
                      onViewDetail={() => setDetailSkill(skill)}
                    />
                  ))}
                </div>
              </div>
            </ScaleIn>
          )}

          <Separator />

          {/* 技能列表 */}
          <ScaleIn delay={0.25}>
            <div className="space-y-3">
              <h2 className="text-lg font-semibold">
                {searchQuery ? '搜索结果' : '全部技能'}
              </h2>
              {isLoadingMarket ? (
                <SkeletonGrid columns={3} rows={3} />
              ) : sortedSkills.length === 0 ? (
                <EmptySearchState
                  searchTerm={searchQuery}
                  onClear={() => {
                    setSearchQuery('')
                    setSelectedCategory('all')
                  }}
                />
              ) : (
                <StaggerContainer className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                  {sortedSkills.map((skill) => (
                    <StaggerItem key={skill.id}>
                      <SkillCard
                        skill={skill}
                        isInstalled={isSkillInstalled(skill.id)}
                        isEnabled={getSkillEnabledStatus(skill.id)}
                        onInstall={() => handleInstall(skill)}
                        onViewDetail={() => setDetailSkill(skill)}
                      />
                    </StaggerItem>
                  ))}
                </StaggerContainer>
              )}
            </div>
          </ScaleIn>
        </>
      )}

      {/* 已安装视图 */}
      {viewMode === 'installed' && (
        <ScaleIn delay={0.1}>
          {isLoadingInstalled ? (
            <SkeletonGrid columns={1} rows={5} />
          ) : (
            <InstalledSkillList
              skills={installedSkills}
              onToggle={handleToggle}
              onConfig={(skill) => setConfigSkill(skill)}
              onUninstall={(skillId) => setUninstallSkillId(skillId)}
            />
          )}
        </ScaleIn>
      )}

      {/* 技能详情弹窗 */}
      <SkillDetailModal
        skill={detailSkill}
        isOpen={!!detailSkill}
        onClose={() => setDetailSkill(null)}
        isInstalled={detailSkill ? isSkillInstalled(detailSkill.id) : false}
        isInstalling={installMutation.isPending}
        onInstall={() => detailSkill && handleInstall(detailSkill)}
      />

      {/* 技能配置弹窗 */}
      <Dialog open={!!configSkill} onOpenChange={() => setConfigSkill(null)}>
        <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>技能配置</DialogTitle>
            <DialogDescription>
              配置 {configSkill?.name} 的参数
            </DialogDescription>
          </DialogHeader>
          {configSkill && (
            <SkillConfigForm
              skill={configSkill}
              onSubmit={handleConfigSave}
              isLoading={updateConfigMutation.isPending}
            />
          )}
        </DialogContent>
      </Dialog>

      {/* 卸载确认弹窗 */}
      <AlertDialog open={!!uninstallSkillId} onOpenChange={() => setUninstallSkillId(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认卸载</AlertDialogTitle>
            <AlertDialogDescription>
              此操作不可撤销。卸载后，该技能将被永久移除。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleUninstall}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              disabled={uninstallMutation.isPending}
            >
              {uninstallMutation.isPending ? '卸载中...' : '卸载'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
