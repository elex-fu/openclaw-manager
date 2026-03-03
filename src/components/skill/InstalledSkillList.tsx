import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import {
  Code,
  PenTool,
  BarChart,
  Image,
  Zap,
  MessageCircle,
  Search,
  Cpu,
  LayoutGrid,
  Settings,
  Trash2,
  RefreshCw,
  Check,
  X,
} from 'lucide-react'
import type { InstalledSkill } from '@/types'

interface InstalledSkillListProps {
  skills: InstalledSkill[]
  onToggle: (skill: InstalledSkill) => void
  onConfig: (skill: InstalledSkill) => void
  onUninstall: (skillId: string) => void
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

export function InstalledSkillList({
  skills,
  onToggle,
  onConfig,
  onUninstall,
}: InstalledSkillListProps) {
  if (skills.length === 0) {
    return (
      <Card>
        <CardContent className="flex flex-col items-center justify-center py-12">
          <div className="w-16 h-16 rounded-full bg-muted flex items-center justify-center mb-4">
            <LayoutGrid className="h-8 w-8 text-muted-foreground" />
          </div>
          <h3 className="text-lg font-medium mb-2">暂无已安装技能</h3>
          <p className="text-sm text-muted-foreground text-center max-w-sm">
            您还没有安装任何技能。前往技能市场浏览并安装您需要的技能。
          </p>
        </CardContent>
      </Card>
    )
  }

  // 获取主分类图标
  const getCategoryIcon = (skill: InstalledSkill) => {
    const primaryCategory = skill.categories[0]
    return categoryIcons[primaryCategory] || <LayoutGrid className="h-4 w-4" />
  }

  return (
    <TooltipProvider>
      <div className="space-y-3">
        {skills.map((skill) => (
          <Card
            key={skill.id}
            className={`transition-all ${
              skill.is_enabled ? '' : 'opacity-60'
            }`}
          >
            <CardContent className="p-4">
              <div className="flex items-center gap-4">
                {/* 图标 */}
                <div className="w-12 h-12 rounded-lg bg-gradient-to-br from-primary/20 to-primary/10 flex items-center justify-center shrink-0">
                  {skill.icon_url ? (
                    <img
                      src={skill.icon_url}
                      alt={skill.name}
                      className="w-6 h-6 object-contain"
                    />
                  ) : (
                    getCategoryIcon(skill)
                  )}
                </div>

                {/* 信息 */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <h3 className="font-semibold truncate">{skill.name}</h3>
                    {skill.has_update && (
                      <Badge variant="secondary" className="text-xs shrink-0">
                        <RefreshCw className="h-3 w-3 mr-1" />
                        有更新
                      </Badge>
                    )}
                    {!skill.is_enabled && (
                      <Badge variant="outline" className="text-xs shrink-0">
                        已禁用
                      </Badge>
                    )}
                  </div>
                  <p className="text-sm text-muted-foreground truncate">
                    {skill.description}
                  </p>
                  <div className="flex items-center gap-2 mt-1">
                    <span className="text-xs text-muted-foreground">
                      v{skill.version}
                    </span>
                    {skill.categories.slice(0, 2).map((category) => (
                      <Badge key={category} variant="outline" className="text-xs">
                        {category}
                      </Badge>
                    ))}
                  </div>
                </div>

                {/* 操作 */}
                <div className="flex items-center gap-2">
                  {/* 启用/禁用开关 */}
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <div>
                        <Switch
                          checked={skill.is_enabled}
                          onCheckedChange={() => onToggle(skill)}
                        />
                      </div>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>{skill.is_enabled ? '禁用技能' : '启用技能'}</p>
                    </TooltipContent>
                  </Tooltip>

                  {/* 配置按钮 */}
                  {skill.config_schema && (
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => onConfig(skill)}
                        >
                          <Settings className="h-4 w-4" />
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>
                        <p>配置技能</p>
                      </TooltipContent>
                    </Tooltip>
                  )}

                  {/* 卸载按钮 */}
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="text-destructive hover:text-destructive"
                        onClick={() => onUninstall(skill.id)}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>卸载技能</p>
                    </TooltipContent>
                  </Tooltip>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </TooltipProvider>
  )
}
