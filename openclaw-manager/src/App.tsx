import { HashRouter, Routes, Route } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MainLayout } from './components/layout/MainLayout'
import { FileListPage } from './pages/FileListPage'
import { GroupPage } from './pages/GroupPage'
import { PluginPage } from './pages/PluginPage'
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
            <Route path="/" element={<FileListPage />} />
            <Route path="/groups" element={<GroupPage />} />
            <Route path="/plugins" element={<PluginPage />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Routes>
        </MainLayout>
      </HashRouter>
    </QueryClientProvider>
  )
}

export default App
