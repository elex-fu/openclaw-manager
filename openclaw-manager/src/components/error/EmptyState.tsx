import { Package, Search, AlertCircle, type LucideIcon } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

interface EmptyStateProps {
  icon?: LucideIcon
  title?: string
  description?: string
  action?: {
    label: string
    onClick: () => void
  }
  secondaryAction?: {
    label: string
    onClick: () => void
  }
  className?: string
}


export function EmptyState({
  icon: Icon = Package,
  title = '暂无数据',
  description = '当前列表为空',
  action,
  secondaryAction,
  className,
}: EmptyStateProps) {
  return (
    <div
      className={cn(
        'flex flex-col items-center justify-center py-12 px-4 text-center',
        className
      )}
    >
      <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-muted">
        <Icon className="h-8 w-8 text-muted-foreground" />
      </div>
      <h3 className="mb-2 text-lg font-semibold">{title}</h3>
      <p className="mb-6 max-w-sm text-sm text-muted-foreground">{description}</p>
      {(action || secondaryAction) && (
        <div className="flex flex-wrap gap-3">
          {action && (
            <Button onClick={action.onClick}>{action.label}</Button>
          )}
          {secondaryAction && (
            <Button variant="outline" onClick={secondaryAction.onClick}>
              {secondaryAction.label}
            </Button>
          )}
        </div>
      )}
    </div>
  )
}

interface EmptyStateCardProps extends EmptyStateProps {
  cardClassName?: string
}

export function EmptyStateCard({
  cardClassName,
  ...props
}: EmptyStateCardProps) {
  return (
    <div className={cn('rounded-lg border bg-card', cardClassName)}>
      <EmptyState {...props} />
    </div>
  )
}

// Preset empty states for common scenarios
export function EmptySearchState({
  searchTerm,
  onClear,
  className,
}: {
  searchTerm?: string
  onClear?: () => void
  className?: string
}) {
  return (
    <EmptyState
      icon={Search}
      title={searchTerm ? `未找到 "${searchTerm}" 相关结果` : '无搜索结果'}
      description="尝试使用不同的关键词或清除搜索条件"
      action={
        onClear
          ? {
              label: '清除搜索',
              onClick: onClear,
            }
          : undefined
      }
      className={className}
    />
  )
}

export function EmptyListState({
  itemName = '项目',
  onCreate,
  className,
}: {
  itemName?: string
  onCreate?: () => void
  className?: string
}) {
  return (
    <EmptyState
      icon={Package}
      title={`暂无${itemName}`}
      description={`开始创建您的第一个${itemName}`}
      action={
        onCreate
          ? {
              label: `创建${itemName}`,
              onClick: onCreate,
            }
          : undefined
      }
      className={className}
    />
  )
}

export function ErrorState({
  error,
  onRetry,
  className,
}: {
  error?: string
  onRetry?: () => void
  className?: string
}) {
  return (
    <EmptyState
      icon={AlertCircle}
      title="加载失败"
      description={error || '数据加载时发生错误，请稍后重试'}
      action={
        onRetry
          ? {
              label: '重试',
              onClick: onRetry,
            }
          : undefined
      }
      className={className}
    />
  )
}
