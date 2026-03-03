import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  Star,
  Download,
  Code,
  PenTool,
  BarChart,
  Image,
  Zap,
  MessageCircle,
  Search,
  Cpu,
  LayoutGrid,
  Check,
  Loader2,
  User,
  Calendar,
  GitBranch,
} from 'lucide-react'
import type { Skill } from '@/types'

interface SkillDetailModalProps {
  skill: Skill | null
  isOpen: boolean
  onClose: () => void
  isInstalled: boolean
  isInstalling: boolean
  onInstall: () => void
}

// 分类图标映射
const categoryIcons: Record<string, React.ReactNode> = {
  programming: <Code className="h-4 w-4" />,
  writing: <PenTool className="h-4 w-4" />,
  data: <BarChart className="h-4 w-4" />,
  image: <Image className="h-4 w-4" />,
  productivity: <Zap className="h-4 w-4" />,
  communication: <MessageCircle className="h-4 w-4" />,
  search: <Search className="h-4 w-4" />,
  automation: <Cpu className="h-4 w-4" />,
}

export function SkillDetailModal({
  skill,
  isOpen,
  onClose,
  isInstalled,
  isInstalling,
  onInstall,
}: SkillDetailModalProps) {
  if (!skill) return null

  // 渲染星级评分
  const renderStars = (rating: number) => {
    return (
      <div className="flex items-center gap-0.5">
        {[1, 2, 3, 4, 5].map((star) => (
          <Star
            key={star}
            className={`h-4 w-4 ${
              star <= Math.round(rating)
                ? 'fill-yellow-400 text-yellow-400'
                : 'text-gray-300'
            }`}
          />
        ))}
      </div>
    )
  }

  // 格式化下载数
  const formatDownloads = (downloads: number) => {
    if (downloads >= 10000) {
      return `${(downloads / 10000).toFixed(1)}万`
    }
    if (downloads >= 1000) {
      return `${(downloads / 1000).toFixed(1)}k`
    }
    return downloads.toString()
  }

  // 获取主分类图标
  const getCategoryIcon = () => {
    const primaryCategory = skill.categories[0]
    return categoryIcons[primaryCategory] || <LayoutGrid className="h-4 w-4" />
  }

  // Hook类型显示名称
  const getHookTypeName = (type: string) => {
    const names: Record<string, string> = {
      pre_process: '预处理',
      post_process: '后处理',
      command: '命令',
      event: '事件',
      tool: '工具',
    }
    return names[type] || type
  }

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-hidden">
        <DialogHeader>
          <div className="flex items-start gap-4">
            {/* 图标 */}
            <div className="w-16 h-16 rounded-xl bg-gradient-to-br from-primary/20 to-primary/10 flex items-center justify-center text-3xl shrink-0">
              {skill.icon_url ? (
                <img
                  src={skill.icon_url}
                  alt={skill.name}
                  className="w-10 h-10 object-contain"
                />
              ) : (
                getCategoryIcon()
              )}
            </div>
            <div className="flex-1 min-w-0">
              <DialogTitle className="text-2xl">{skill.name}</DialogTitle>
              <DialogDescription className="mt-1">
                {skill.description}
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        <ScrollArea className="max-h-[60vh]">
          <div className="space-y-6 pr-4">
            {/* 统计信息 */}
            <div className="flex flex-wrap gap-4 text-sm">
              <div className="flex items-center gap-2">
                {renderStars(skill.rating)}
                <span className="text-muted-foreground">({skill.rating.toFixed(1)})</span>
              </div>
              <div className="flex items-center gap-1 text-muted-foreground">
                <Download className="h-4 w-4" />
                {formatDownloads(skill.downloads)} 次下载
              </div>
              <div className="flex items-center gap-1 text-muted-foreground">
                <GitBranch className="h-4 w-4" />
                v{skill.version}
              </div>
              <div className="flex items-center gap-1 text-muted-foreground">
                <User className="h-4 w-4" />
                {skill.author}
              </div>
              <div className="flex items-center gap-1 text-muted-foreground">
                <Calendar className="h-4 w-4" />
                更新于 {new Date(skill.updated_at).toLocaleDateString('zh-CN')}
              </div>
            </div>

            <Separator />

            {/* 分类 */}
            <div>
              <h4 className="text-sm font-medium mb-2">分类</h4>
              <div className="flex flex-wrap gap-2">
                {skill.categories.map((category) => (
                  <Badge key={category} variant="outline">
                    {categoryIcons[category] && (
                      <span className="mr-1">{categoryIcons[category]}</span>
                    )}
                    {category}
                  </Badge>
                ))}
              </div>
            </div>

            {/* 标签 */}
            {skill.tags.length > 0 && (
              <div>
                <h4 className="text-sm font-medium mb-2">标签</h4>
                <div className="flex flex-wrap gap-2">
                  {skill.tags.map((tag) => (
                    <Badge key={tag} variant="secondary">
                      {tag}
                    </Badge>
                  ))}
                </div>
              </div>
            )}

            {/* Hook列表 */}
            {skill.hooks.length > 0 && (
              <div>
                <h4 className="text-sm font-medium mb-2">功能 Hooks</h4>
                <div className="space-y-2">
                  {skill.hooks.map((hook, index) => (
                    <div
                      key={index}
                      className="flex items-center justify-between p-3 rounded-lg bg-muted"
                    >
                      <div>
                        <div className="font-medium">{hook.trigger}</div>
                        {hook.description && (
                          <div className="text-sm text-muted-foreground">
                            {hook.description}
                          </div>
                        )}
                      </div>
                      <Badge variant="outline">{getHookTypeName(hook.hook_type)}</Badge>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* 依赖 */}
            {skill.dependencies.length > 0 && (
              <div>
                <h4 className="text-sm font-medium mb-2">依赖技能</h4>
                <div className="flex flex-wrap gap-2">
                  {skill.dependencies.map((dep) => (
                    <Badge key={dep} variant="outline">
                      {dep}
                    </Badge>
                  ))}
                </div>
              </div>
            )}

            {/* 配置说明 */}
            {skill.config_schema && (
              <div>
                <h4 className="text-sm font-medium mb-2">可配置</h4>
                <p className="text-sm text-muted-foreground">
                  此技能支持自定义配置，安装后可在设置中调整参数。
                </p>
              </div>
            )}
          </div>
        </ScrollArea>

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            关闭
          </Button>
          {isInstalled ? (
            <Button disabled>
              <Check className="h-4 w-4 mr-2" />
              已安装
            </Button>
          ) : (
            <Button onClick={onInstall} disabled={isInstalling}>
              {isInstalling ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  安装中...
                </>
              ) : (
                '安装技能'
              )}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
