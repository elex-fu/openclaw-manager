import { HashRouter, Routes, Route } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MainLayout } from './components/layout/MainLayout'
import { Dashboard } from './pages/Dashboard'
import { InstallWizard } from './pages/InstallWizard'
import { ModelConfigPage } from './pages/ModelConfig'
import { AgentManager } from './pages/AgentManager'
import { Diagnostics } from './pages/Diagnostics'
import { SettingsPage } from './pages/SettingsPage'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5 minutes
      refetchOnWindowFocus: false,
    },
  },
})

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <MainLayout>
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/install" element={<InstallWizard />} />
            <Route path="/models" element={<ModelConfigPage />} />
            <Route path="/agents" element={<AgentManager />} />
            <Route path="/diagnostics" element={<Diagnostics />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Routes>
        </MainLayout>
      </HashRouter>
    </QueryClientProvider>
  )
}

export default App
