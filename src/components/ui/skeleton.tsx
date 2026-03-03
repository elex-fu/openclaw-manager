import { cn } from "@/lib/utils"

function Skeleton({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn("animate-pulse rounded-md bg-muted", className)}
      {...props}
    />
  )
}

interface SkeletonCardProps {
  className?: string
  header?: boolean
  content?: boolean
  footer?: boolean
  rows?: number
}

export function SkeletonCard({
  className,
  header = true,
  content = true,
  footer = false,
  rows = 3,
}: SkeletonCardProps) {
  return (
    <div className={cn("rounded-lg border bg-card p-6 space-y-4", className)}>
      {header && (
        <div className="flex items-center justify-between">
          <Skeleton className="h-5 w-1/3" />
          <Skeleton className="h-4 w-4" />
        </div>
      )}
      {content && (
        <div className="space-y-2">
          {Array.from({ length: rows }).map((_, i) => (
            <Skeleton key={i} className="h-4 w-full" />
          ))}
        </div>
      )}
      {footer && (
        <div className="flex gap-2 pt-2">
          <Skeleton className="h-9 w-24" />
          <Skeleton className="h-9 w-24" />
        </div>
      )}
    </div>
  )
}

interface SkeletonListProps {
  className?: string
  items?: number
  showAvatar?: boolean
}

export function SkeletonList({
  className,
  items = 5,
  showAvatar = true,
}: SkeletonListProps) {
  return (
    <div className={cn("space-y-3", className)}>
      {Array.from({ length: items }).map((_, i) => (
        <div key={i} className="flex items-center gap-4 p-3 rounded-lg border">
          {showAvatar && <Skeleton className="h-10 w-10 rounded-full" />}
          <div className="flex-1 space-y-2">
            <Skeleton className="h-4 w-1/4" />
            <Skeleton className="h-3 w-1/2" />
          </div>
          <Skeleton className="h-8 w-8" />
        </div>
      ))}
    </div>
  )
}

interface SkeletonGridProps {
  className?: string
  columns?: number
  rows?: number
}

export function SkeletonGrid({
  className,
  columns = 3,
  rows = 2,
}: SkeletonGridProps) {
  return (
    <div
      className={cn(
        "grid gap-4",
        columns === 1 && "grid-cols-1",
        columns === 2 && "grid-cols-1 md:grid-cols-2",
        columns === 3 && "grid-cols-1 md:grid-cols-2 lg:grid-cols-3",
        columns === 4 && "grid-cols-1 md:grid-cols-2 lg:grid-cols-4",
        className
      )}
    >
      {Array.from({ length: columns * rows }).map((_, i) => (
        <SkeletonCard key={i} />
      ))}
    </div>
  )
}

interface SkeletonTextProps {
  className?: string
  lines?: number
  lastLineWidth?: string
}

export function SkeletonText({
  className,
  lines = 3,
  lastLineWidth = "60%",
}: SkeletonTextProps) {
  return (
    <div className={cn("space-y-2", className)}>
      {Array.from({ length: lines - 1 }).map((_, i) => (
        <Skeleton key={i} className="h-4 w-full" />
      ))}
      <Skeleton className="h-4" style={{ width: lastLineWidth }} />
    </div>
  )
}

interface SkeletonAvatarProps {
  className?: string
  size?: "sm" | "md" | "lg"
}

const avatarSizes = {
  sm: "h-8 w-8",
  md: "h-10 w-10",
  lg: "h-16 w-16",
}

export function SkeletonAvatar({ className, size = "md" }: SkeletonAvatarProps) {
  return (
    <Skeleton
      className={cn("rounded-full", avatarSizes[size], className)}
    />
  )
}

interface SkeletonPageProps {
  className?: string
  header?: boolean
  contentRows?: number
}

export function SkeletonPage({
  className,
  header = true,
  contentRows = 3,
}: SkeletonPageProps) {
  return (
    <div className={cn("space-y-6", className)}>
      {header && (
        <div className="space-y-2">
          <Skeleton className="h-8 w-1/3" />
          <Skeleton className="h-4 w-1/2" />
        </div>
      )}
      <SkeletonGrid columns={3} rows={contentRows} />
    </div>
  )
}

export { Skeleton }
