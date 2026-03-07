import { Card, CardContent, CardHeader } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Star, Download, Check, ChevronRight, Code, PenTool, BarChart, Image, Zap, MessageCircle, Search, Cpu, LayoutGrid } from 'lucide-react'
// 技能卡片通用属性接口
interface SkillCardItem {
  id: string
  name: string
  description: string
  author: string
  version: string
  categories: string[]
  tags: string[]
  icon_url?: string
  rating: number
  downloads: number
}

// 分类图标映射
const categoryIcons: Record<string, React.ReactNode> = {
  programming: <Code className="h-3 w-3" />,
  writing: <PenTool className="h-3 w-3" />,
  data: <BarChart className="h-3 w-3" />,
  image: <Image className="h-3 w-3" />,
  productivity: <Zap className="h-3 w-3" />,
  communication: <MessageCircle className="h-3 w-3" />,
  search: <Search className="h-3 w-3" />,
  automation: <Cpu className="h-3 w-3" />,
}

interface SkillCardProps {
  skill: SkillCardItem
  isInstalled?: boolean
  isEnabled?: boolean
  onInstall?: () => void
  onViewDetail?: () => void
}

export function SkillCard({
  skill,
  isInstalled = false,
  isEnabled = false,
  onInstall,
  onViewDetail,
}: SkillCardProps) {
  // 渲染星级评分
  const renderStars = (rating: number) => {
    return (
      <div className="flex items-center gap-0.5">
        {[1, 2, 3, 4, 5].map((star) => (
          <Star
            key={star}
            className={`h-3 w-3 ${
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
    return categoryIcons[primaryCategory] || <LayoutGrid className="h-3 w-3" />
  }

  return (
    <Card className="group hover:shadow-lg transition-all duration-200 cursor-pointer overflow-hidden">
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-3">
            {/* 图标 */}
            <div className="w-12 h-12 rounded-lg bg-gradient-to-br from-primary/20 to-primary/10 flex items-center justify-center text-2xl">
              {skill.icon_url ? (
                <img
                  src={skill.icon_url}
                  alt={skill.name}
                  className="w-8 h-8 object-contain"
                />
              ) : (
                getCategoryIcon()
              )}
            </div>
            <div>
              <h3 className="font-semibold text-lg leading-tight">{skill.name}</h3>
              <p className="text-sm text-muted-foreground">{skill.author}</p>
            </div>
          </div>
          {/* 状态徽章 */}
          {isInstalled && (
            <Badge variant={isEnabled ? 'default' : 'secondary'}>
              {isEnabled ? (
                <>
                  <Check className="h-3 w-3 mr-1" />
                  已启用
                </>
              ) : (
                '已禁用'
              )}
            </Badge>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        {/* 描述 */}
        <p className="text-sm text-muted-foreground line-clamp-2">
          {skill.description}
        </p>

        {/* 分类和标签 */}
        <div className="flex flex-wrap gap-1">
          {skill.categories.slice(0, 2).map((category) => (
            <Badge key={category} variant="outline" className="text-xs">
              {categoryIcons[category] && (
                <span className="mr-1">{categoryIcons[category]}</span>
              )}
              {category}
            </Badge>
          ))}
          {skill.tags.slice(0, 2).map((tag) => (
            <Badge key={tag} variant="secondary" className="text-xs">
              {tag}
            </Badge>
          ))}
        </div>

        {/* 统计信息 */}
        <div className="flex items-center justify-between text-sm">
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-1">
              {renderStars(skill.rating)}
              <span className="text-muted-foreground">({skill.rating.toFixed(1)})</span>
            </div>
            <div className="flex items-center gap-1 text-muted-foreground">
              <Download className="h-3 w-3" />
              {formatDownloads(skill.downloads)}
            </div>
          </div>
          <span className="text-xs text-muted-foreground">v{skill.version}</span>
        </div>

        {/* 操作按钮 */}
        <div className="flex gap-2 pt-2">
          {isInstalled ? (
            <>
              <Button
                variant="outline"
                size="sm"
                className="flex-1"
                onClick={(e) => {
                  e.stopPropagation()
                  onViewDetail?.()
                }}
              >
                查看详情
                <ChevronRight className="h-4 w-4 ml-1" />
              </Button>
            </>
          ) : (
            <>
              <Button
                variant="outline"
                size="sm"
                className="flex-1"
                onClick={(e) => {
                  e.stopPropagation()
                  onViewDetail?.()
                }}
              >
                详情
              </Button>
              <Button
                size="sm"
                className="flex-1"
                onClick={(e) => {
                  e.stopPropagation()
                  onInstall?.()
                }}
              >
                安装
              </Button>
            </>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
