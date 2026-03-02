import { useState, useEffect } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { cn } from '@/lib/utils'
import {
  LayoutDashboard,
  Brain,
  Bot,
  Stethoscope,
  Settings,
  Menu,
  X,
  Package,
  Download,
  ChevronLeft,
  ChevronRight,
  FileText,
  Puzzle,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { ThemeToggle } from '@/components/theme'
import { useAppStore } from '@/stores/appStore'

interface MainLayoutProps {
  children: React.ReactNode
}

const navItems = [
  { path: '/', label: '仪表盘', icon: LayoutDashboard },
  { path: '/models', label: '模型配置', icon: Brain },
  { path: '/agents', label: 'Agent 管理', icon: Bot },
  { path: '/skills', label: '技能商店', icon: Puzzle },
  { path: '/diagnostics', label: '诊断修复', icon: Stethoscope },
  { path: '/logs', label: '日志查看', icon: FileText },
  { path: '/update', label: '版本升级', icon: Download },
  { path: '/settings', label: '设置', icon: Settings },
]

const SIDEBAR_WIDTH = 256
const SIDEBAR_COLLAPSED_WIDTH = 72

export function MainLayout({ children }: MainLayoutProps) {
  const location = useLocation()
  const { sidebarOpen, setSidebarOpen } = useAppStore()
  const [isMobile, setIsMobile] = useState(false)
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false)

  // Detect mobile viewport
  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 1024)
      if (window.innerWidth >= 1024) {
        setMobileMenuOpen(false)
      }
    }

    checkMobile()
    window.addEventListener('resize', checkMobile)
    return () => window.removeEventListener('resize', checkMobile)
  }, [])

  // Close mobile menu on route change
  useEffect(() => {
    setMobileMenuOpen(false)
  }, [location.pathname])

  const toggleSidebar = () => {
    if (isMobile) {
      setMobileMenuOpen(!mobileMenuOpen)
    } else {
      setSidebarOpen(!sidebarOpen)
    }
  }

  const isCollapsed = !sidebarOpen && !isMobile
  const sidebarWidth = isCollapsed ? SIDEBAR_COLLAPSED_WIDTH : SIDEBAR_WIDTH

  return (
    <TooltipProvider delayDuration={0}>
      <div className="flex h-screen w-full bg-background overflow-hidden">
        {/* Mobile Overlay */}
        {isMobile && mobileMenuOpen && (
          <div
            className="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm transition-opacity"
            onClick={() => setMobileMenuOpen(false)}
          />
        )}

        {/* Sidebar */}
        <aside
          className={cn(
            'fixed inset-y-0 left-0 z-50 bg-card border-r transition-all duration-300 ease-in-out flex flex-col',
            isMobile
              ? mobileMenuOpen
                ? 'translate-x-0 w-64'
                : '-translate-x-full w-64'
              : 'translate-x-0'
          )}
          style={{
            width: isMobile ? SIDEBAR_WIDTH : sidebarWidth,
          }}
        >
          {/* Logo */}
          <div className="flex h-16 items-center border-b px-4 shrink-0">
            <div className="flex items-center gap-3 overflow-hidden">
              <div className="w-9 h-9 rounded-lg bg-primary flex items-center justify-center shrink-0">
                <Package className="h-5 w-5 text-primary-foreground" />
              </div>
              <h1
                className={cn(
                  'text-lg font-bold whitespace-nowrap transition-all duration-300',
                  isCollapsed ? 'opacity-0 w-0' : 'opacity-100'
                )}
              >
                OpenClaw
              </h1>
            </div>
          </div>

          {/* Navigation */}
          <nav className="flex-1 space-y-1 p-3 overflow-y-auto scrollbar-thin">
            {navItems.map((item) => {
              const Icon = item.icon
              const isActive = location.pathname === item.path

              const navLink = (
                <Link
                  key={item.path}
                  to={item.path}
                  className={cn(
                    'flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-all duration-200',
                    isActive
                      ? 'bg-primary text-primary-foreground shadow-sm'
                      : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground',
                    isCollapsed && 'justify-center px-2'
                  )}
                >
                  <Icon className="h-5 w-5 shrink-0" />
                  <span
                    className={cn(
                      'whitespace-nowrap transition-all duration-300',
                      isCollapsed ? 'opacity-0 w-0 hidden' : 'opacity-100'
                    )}
                  >
                    {item.label}
                  </span>
                </Link>
              )

              if (isCollapsed) {
                return (
                  <Tooltip key={item.path}>
                    <TooltipTrigger asChild>{navLink}</TooltipTrigger>
                    <TooltipContent side="right">
                      <p>{item.label}</p>
                    </TooltipContent>
                  </Tooltip>
                )
              }

              return navLink
            })}
          </nav>

          {/* Footer */}
          <div className="border-t p-3 shrink-0">
            <div
              className={cn(
                'flex items-center text-xs text-muted-foreground transition-all duration-300',
                isCollapsed ? 'justify-center' : 'justify-between px-2'
              )}
            >
              <span className={cn(isCollapsed && 'hidden')}>v0.1.0</span>
              <span className={cn(isCollapsed && 'hidden')}>MVP</span>
            </div>
          </div>
        </aside>

        {/* Main content */}
        <div
          className="flex flex-1 flex-col min-w-0 transition-all duration-300"
          style={{
            marginLeft: isMobile ? 0 : sidebarWidth,
          }}
        >
          {/* Header */}
          <header className="flex h-16 items-center gap-4 border-b bg-card/50 backdrop-blur-sm px-4 lg:px-6 shrink-0">
            <Button
              variant="ghost"
              size="icon"
              onClick={toggleSidebar}
              className="shrink-0"
            >
              {isMobile ? (
                mobileMenuOpen ? (
                  <X className="h-5 w-5" />
                ) : (
                  <Menu className="h-5 w-5" />
                )
              ) : sidebarOpen ? (
                <ChevronLeft className="h-5 w-5" />
              ) : (
                <ChevronRight className="h-5 w-5" />
              )}
            </Button>

            <h2 className="text-lg font-semibold truncate">
              {navItems.find((item) => item.path === location.pathname)?.label ||
                'OpenClaw Manager'}
            </h2>

            <div className="flex-1" />

            {/* Theme Toggle */}
            <ThemeToggle />
          </header>

          {/* Page content */}
          <main className="flex-1 overflow-auto p-4 lg:p-6 scrollbar-thin">
            <div className="mx-auto max-w-7xl">{children}</div>
          </main>
        </div>
      </div>
    </TooltipProvider>
  )
}
