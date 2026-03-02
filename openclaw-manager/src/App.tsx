import { HashRouter, Routes, Route, useLocation } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { AnimatePresence } from 'framer-motion'
import { ThemeProvider } from './components/theme'
import { ErrorBoundary } from './components/error'
import { MainLayout } from './components/layout/MainLayout'
import { PageTransition } from './components/animation'
import { Dashboard } from './pages/Dashboard'
import { InstallWizard } from './pages/InstallWizard'
import { ModelConfigPage } from './pages/ModelConfig'
import { AgentManager } from './pages/AgentManager'
import { Diagnostics } from './pages/Diagnostics'
import { SettingsPage } from './pages/SettingsPage'
import { UpdateManager } from './pages/UpdateManager'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5 minutes
      refetchOnWindowFocus: false,
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
              <Dashboard />
            </PageTransition>
          }
        />
        <Route
          path="/install"
          element={
            <PageTransition>
              <InstallWizard />
            </PageTransition>
          }
        />
        <Route
          path="/models"
          element={
            <PageTransition>
              <ModelConfigPage />
            </PageTransition>
          }
        />
        <Route
          path="/agents"
          element={
            <PageTransition>
              <AgentManager />
            </PageTransition>
          }
        />
        <Route
          path="/diagnostics"
          element={
            <PageTransition>
              <Diagnostics />
            </PageTransition>
          }
        />
        <Route
          path="/update"
          element={
            <PageTransition>
              <UpdateManager />
            </PageTransition>
          }
        />
        <Route
          path="/settings"
          element={
            <PageTransition>
              <SettingsPage />
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
