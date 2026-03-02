import { Suspense, lazy } from 'react'
import { HashRouter, Routes, Route, useLocation } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { AnimatePresence } from 'framer-motion'
import { ThemeProvider } from './components/theme'
import { ErrorBoundary } from './components/error'
import { MainLayout } from './components/layout/MainLayout'
import { PageTransition } from './components/animation'
import { PageLoader } from './components/ui/loading'

// 懒加载页面组件 - 按路由分割代码
const Dashboard = lazy(() => import('./pages/Dashboard'))
const InstallWizard = lazy(() => import('./pages/InstallWizard'))
const ModelConfigPage = lazy(() => import('./pages/ModelConfig'))
const AgentManager = lazy(() => import('./pages/AgentManager'))
const Diagnostics = lazy(() => import('./pages/Diagnostics'))
const SettingsPage = lazy(() => import('./pages/SettingsPage'))
const UpdateManager = lazy(() => import('./pages/UpdateManager'))
const LogViewer = lazy(() => import('./pages/LogViewer'))
const SkillStore = lazy(() => import('./pages/SkillStore'))

// 页面加载占位符
function PageSuspense({ children }: { children: React.ReactNode }) {
  return (
    <Suspense fallback={<PageLoader title="加载中..." description="正在准备页面内容" />}>
      {children}
    </Suspense>
  )
}

// 优化 QueryClient 配置
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5 minutes
      gcTime: 1000 * 60 * 30, // 30 minutes (formerly cacheTime)
      refetchOnWindowFocus: false,
      refetchOnReconnect: true,
      retry: (failureCount, error) => {
        // 指数退避重试策略
        if (failureCount >= 3) return false
        // 只对网络错误重试
        if (error instanceof Error && error.message.includes('network')) {
          return true
        }
        return false
      },
      retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
    },
    mutations: {
      retry: 1,
      retryDelay: 1000,
    },
  },
})

// Animated routes wrapper
function AnimatedRoutes() {
  const location = useLocation()

  return (
    <AnimatePresence mode="wait">
      <Routes location={location} key={location.pathname}>
        <Route
          path="/"
          element={
            <PageTransition>
              <PageSuspense>
                <Dashboard />
              </PageSuspense>
            </PageTransition>
          }
        />
        <Route
          path="/install"
          element={
            <PageTransition>
              <PageSuspense>
                <InstallWizard />
              </PageSuspense>
            </PageTransition>
          }
        />
        <Route
          path="/models"
          element={
            <PageTransition>
              <PageSuspense>
                <ModelConfigPage />
              </PageSuspense>
            </PageTransition>
          }
        />
        <Route
          path="/agents"
          element={
            <PageTransition>
              <PageSuspense>
                <AgentManager />
              </PageSuspense>
            </PageTransition>
          }
        />
        <Route
          path="/diagnostics"
          element={
            <PageTransition>
              <PageSuspense>
                <Diagnostics />
              </PageSuspense>
            </PageTransition>
          }
        />
        <Route
          path="/update"
          element={
            <PageTransition>
              <PageSuspense>
                <UpdateManager />
              </PageSuspense>
            </PageTransition>
          }
        />
        <Route
          path="/settings"
          element={
            <PageTransition>
              <PageSuspense>
                <SettingsPage />
              </PageSuspense>
            </PageTransition>
          }
        />
        <Route
          path="/logs"
          element={
            <PageTransition>
              <PageSuspense>
                <LogViewer />
              </PageSuspense>
            </PageTransition>
          }
        />
        <Route
          path="/skills"
          element={
            <PageTransition>
              <PageSuspense>
                <SkillStore />
              </PageSuspense>
            </PageTransition>
          }
        />
      </Routes>
    </AnimatePresence>
  )
}

function App() {
  return (
    <ErrorBoundary>
      <ThemeProvider defaultTheme="system" storageKey="openclaw-theme">
        <QueryClientProvider client={queryClient}>
          <HashRouter>
            <MainLayout>
              <AnimatedRoutes />
            </MainLayout>
          </HashRouter>
        </QueryClientProvider>
      </ThemeProvider>
    </ErrorBoundary>
  )
}

export default App
