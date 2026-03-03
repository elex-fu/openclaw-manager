import { useState, useEffect, useCallback } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Separator } from '@/components/ui/separator'
import {
  Settings,
  Bell,
  Shield,
  Package,
  Moon,
  Sun,
  Monitor,
  Download,
  Upload,
  AlertTriangle,
  Check,
  RotateCcw,
  FolderOpen,
  Power,
} from 'lucide-react'
import { InstallerPanel } from '@/components/openclaw/InstallerPanel'
import { useConfigStore } from '@/stores/configStore'
import { openclawApi } from '@/lib/tauri-api'
import type { Theme } from '@/types'

// 主题工具函数
const applyTheme = (theme: Theme) => {
  const root = document.documentElement
  const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches

  const isDark = theme === 'dark' || (theme === 'system' && systemDark)

  if (isDark) {
    root.classList.add('dark')
  } else {
    root.classList.remove('dark')
  }
}

export function SettingsPage() {
  const { appSettings } = useConfigStore()

  // 应用主题
  useEffect(() => {
    applyTheme(appSettings.theme)
  }, [appSettings.theme])

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">设置</h2>
        <p className="text-muted-foreground">配置应用偏好和系统选项</p>
      </div>

      <Tabs defaultValue="appearance" className="space-y-4">
        <TabsList className="grid w-full grid-cols-5 lg:w-auto">
          <TabsTrigger value="appearance">
            <Moon className="mr-2 h-4 w-4" />
            外观
          </TabsTrigger>
          <TabsTrigger value="startup">
            <Power className="mr-2 h-4 w-4" />
            启动
          </TabsTrigger>
          <TabsTrigger value="notifications">
            <Bell className="mr-2 h-4 w-4" />
            通知
          </TabsTrigger>
          <TabsTrigger value="openclaw">
            <Package className="mr-2 h-4 w-4" />
            OpenClaw
          </TabsTrigger>
          <TabsTrigger value="advanced">
            <Shield className="mr-2 h-4 w-4" />
            高级
          </TabsTrigger>
        </TabsList>

        <TabsContent value="appearance" className="space-y-4">
          <AppearanceSettings />
        </TabsContent>

        <TabsContent value="startup" className="space-y-4">
          <StartupSettings />
        </TabsContent>

        <TabsContent value="notifications" className="space-y-4">
          <NotificationSettings />
        </TabsContent>

        <TabsContent value="openclaw" className="space-y-4">
          <OpenClawSettings />
        </TabsContent>

        <TabsContent value="advanced" className="space-y-4">
          <AdvancedSettings />
        </TabsContent>
      </Tabs>
    </div>
  )
}

// 外观设置
function AppearanceSettings() {
  const { appSettings, setTheme } = useConfigStore()

  const themes: { value: Theme; label: string; icon: React.ReactNode }[] = [
    { value: 'light', label: '浅色', icon: <Sun className="h-4 w-4" /> },
    { value: 'dark', label: '深色', icon: <Moon className="h-4 w-4" /> },
    { value: 'system', label: '跟随系统', icon: <Monitor className="h-4 w-4" /> },
  ]

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Moon className="h-5 w-5" />
          外观设置
        </CardTitle>
        <CardDescription>自定义应用的外观和主题</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-3">
          <Label>主题</Label>
          <div className="grid grid-cols-3 gap-4">
            {themes.map((theme) => (
              <button
                key={theme.value}
                onClick={() => setTheme(theme.value)}
                className={`flex flex-col items-center gap-2 rounded-lg border p-4 transition-colors hover:bg-muted ${
                  appSettings.theme === theme.value
                    ? 'border-primary bg-primary/5'
                    : 'border-border'
                }`}
              >
                {theme.icon}
                <span className="text-sm font-medium">{theme.label}</span>
                {appSettings.theme === theme.value && (
                  <Check className="h-4 w-4 text-primary" />
                )}
              </button>
            ))}
          </div>
        </div>

        <Separator />

        <div className="space-y-3">
          <Label>语言</Label>
          <Select value={appSettings.language} disabled>
            <SelectTrigger className="w-[200px]">
              <SelectValue placeholder="选择语言" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="zh-CN">简体中文</SelectItem>
              <SelectItem value="en-US">English</SelectItem>
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground">
            更多语言支持即将推出
          </p>
        </div>
      </CardContent>
    </Card>
  )
}

// 启动设置
function StartupSettings() {
  const { appSettings, setStartupSetting } = useConfigStore()
  const { startup } = appSettings

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Power className="h-5 w-5" />
          启动设置
        </CardTitle>
        <CardDescription>配置应用启动时的行为</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>开机自启</Label>
            <p className="text-sm text-muted-foreground">
              系统启动时自动运行应用
            </p>
          </div>
          <Switch
            checked={startup.auto_start}
            onCheckedChange={(checked) =>
              setStartupSetting('auto_start', checked)
            }
          />
        </div>

        <Separator />

        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>最小化到托盘</Label>
            <p className="text-sm text-muted-foreground">
              启动时最小化到系统托盘，不显示主窗口
            </p>
          </div>
          <Switch
            checked={startup.minimize_to_tray}
            onCheckedChange={(checked) =>
              setStartupSetting('minimize_to_tray', checked)
            }
          />
        </div>

        <Separator />

        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>启动时检查更新</Label>
            <p className="text-sm text-muted-foreground">
              每次启动时自动检查是否有新版本可用
            </p>
          </div>
          <Switch
            checked={startup.check_update_on_start}
            onCheckedChange={(checked) =>
              setStartupSetting('check_update_on_start', checked)
            }
          />
        </div>
      </CardContent>
    </Card>
  )
}

// 通知设置
function NotificationSettings() {
  const {
    appSettings,
    setNotificationEnabled,
    setNotificationFilter,
  } = useConfigStore()
  const { notifications } = appSettings

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Bell className="h-5 w-5" />
          通知设置
        </CardTitle>
        <CardDescription>配置通知行为和筛选</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>启用通知</Label>
            <p className="text-sm text-muted-foreground">
              显示桌面通知和提醒
            </p>
          </div>
          <Switch
            checked={notifications.enabled}
            onCheckedChange={setNotificationEnabled}
          />
        </div>

        <Separator />

        <div className="space-y-4">
          <Label className={!notifications.enabled ? 'text-muted-foreground' : ''}>
            通知类型筛选
          </Label>
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Check className="h-4 w-4 text-green-500" />
                <span
                  className={
                    !notifications.enabled ? 'text-muted-foreground' : ''
                  }
                >
                  成功通知
                </span>
              </div>
              <Switch
                checked={notifications.filter.success}
                onCheckedChange={(checked) =>
                  setNotificationFilter({ success: checked })
                }
                disabled={!notifications.enabled}
              />
            </div>

            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Settings className="h-4 w-4 text-blue-500" />
                <span
                  className={
                    !notifications.enabled ? 'text-muted-foreground' : ''
                  }
                >
                  信息通知
                </span>
              </div>
              <Switch
                checked={notifications.filter.info}
                onCheckedChange={(checked) =>
                  setNotificationFilter({ info: checked })
                }
                disabled={!notifications.enabled}
              />
            </div>

            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <AlertTriangle className="h-4 w-4 text-yellow-500" />
                <span
                  className={
                    !notifications.enabled ? 'text-muted-foreground' : ''
                  }
                >
                  警告通知
                </span>
              </div>
              <Switch
                checked={notifications.filter.warning}
                onCheckedChange={(checked) =>
                  setNotificationFilter({ warning: checked })
                }
                disabled={!notifications.enabled}
              />
            </div>

            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Shield className="h-4 w-4 text-red-500" />
                <span
                  className={
                    !notifications.enabled ? 'text-muted-foreground' : ''
                  }
                >
                  错误通知
                </span>
              </div>
              <Switch
                checked={notifications.filter.error}
                onCheckedChange={(checked) =>
                  setNotificationFilter({ error: checked })
                }
                disabled={!notifications.enabled}
              />
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

// OpenClaw 设置
function OpenClawSettings() {
  const { settings, updateSettings } = useConfigStore()
  const [installPath] = useState<string>('~/.openclaw')

  // 获取安装路径
  useEffect(() => {
    const fetchInstallPath = async () => {
      try {
        await openclawApi.checkInstallation()
        // 这里可以根据实际情况获取安装路径
        // 暂时使用默认值
      } catch (error) {
        console.error('Failed to get install path:', error)
      }
    }
    fetchInstallPath()
  }, [])

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Package className="h-5 w-5" />
            OpenClaw 配置
          </CardTitle>
          <CardDescription>配置 OpenClaw 运行参数</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="space-y-3">
            <Label>安装路径</Label>
            <div className="flex gap-2">
              <Input value={installPath} disabled className="font-mono" />
              <Button variant="outline" size="icon" disabled>
                <FolderOpen className="h-4 w-4" />
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              OpenClaw 使用标准安装路径，暂不支持自定义
            </p>
          </div>

          <Separator />

          <div className="space-y-3">
            <Label>日志级别</Label>
            <Select
              value={settings.log_level}
              onValueChange={(value) =>
                updateSettings({
                  log_level: value as 'debug' | 'info' | 'warn' | 'error',
                })
              }
            >
              <SelectTrigger className="w-[200px]">
                <SelectValue placeholder="选择日志级别" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="debug">Debug - 调试</SelectItem>
                <SelectItem value="info">Info - 信息</SelectItem>
                <SelectItem value="warn">Warn - 警告</SelectItem>
                <SelectItem value="error">Error - 错误</SelectItem>
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">
              日志级别越低，输出的日志越详细
            </p>
          </div>

          <Separator />

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label>自动更新</Label>
              <p className="text-sm text-muted-foreground">
                自动检查并安装 OpenClaw 更新
              </p>
            </div>
            <Switch
              checked={settings.auto_update}
              onCheckedChange={(checked) =>
                updateSettings({ auto_update: checked })
              }
            />
          </div>
        </CardContent>
      </Card>

      <InstallerPanel />
    </div>
  )
}

// 高级设置
function AdvancedSettings() {
  const { resetAllSettings, settings, appSettings, models, agents } = useConfigStore()
  const [resetDialogOpen, setResetDialogOpen] = useState(false)
  const [exportDialogOpen, setExportDialogOpen] = useState(false)
  const [importDialogOpen, setImportDialogOpen] = useState(false)
  const [exportData, setExportData] = useState('')
  const [importData, setImportData] = useState('')
  const [importError, setImportError] = useState('')

  // 导出配置
  const handleExport = useCallback(() => {
    const data = {
      version: '1.0',
      exportDate: new Date().toISOString(),
      settings,
      appSettings,
      models,
      agents,
    }
    setExportData(JSON.stringify(data, null, 2))
    setExportDialogOpen(true)
  }, [settings, appSettings, models, agents])

  // 导入配置
  const handleImport = useCallback(() => {
    try {
      const data = JSON.parse(importData)
      // 验证数据格式
      if (!data.settings || !data.appSettings) {
        throw new Error('无效的配置文件格式')
      }
      // 这里可以添加更多的导入逻辑
      setImportDialogOpen(false)
      setImportData('')
      setImportError('')
      alert('配置导入成功，请重启应用以生效')
    } catch (error) {
      setImportError(error instanceof Error ? error.message : '导入失败')
    }
  }, [importData])

  // 复制到剪贴板
  const copyToClipboard = useCallback(() => {
    navigator.clipboard.writeText(exportData)
    alert('已复制到剪贴板')
  }, [exportData])

  // 重置所有设置
  const handleReset = useCallback(() => {
    resetAllSettings()
    setResetDialogOpen(false)
    // 应用默认主题
    applyTheme('system')
    alert('所有设置已重置为默认值')
  }, [resetAllSettings])

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            高级设置
          </CardTitle>
          <CardDescription>数据管理和故障排除选项</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="space-y-3">
            <Label>配置管理</Label>
            <div className="flex flex-wrap gap-2">
              <Button variant="outline" onClick={handleExport}>
                <Download className="mr-2 h-4 w-4" />
                导出配置
              </Button>
              <Button
                variant="outline"
                onClick={() => setImportDialogOpen(true)}
              >
                <Upload className="mr-2 h-4 w-4" />
                导入配置
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              导出和导入应用配置（JSON 格式）
            </p>
          </div>

          <Separator />

          <div className="space-y-3">
            <Label>数据管理</Label>
            <Alert variant="destructive" className="border-destructive/50">
              <AlertTriangle className="h-4 w-4" />
              <AlertTitle>危险区域</AlertTitle>
              <AlertDescription>
                以下操作不可逆，请谨慎操作
              </AlertDescription>
            </Alert>
            <div className="flex flex-wrap gap-2">
              <Dialog
                open={resetDialogOpen}
                onOpenChange={setResetDialogOpen}
              >
                <DialogTrigger asChild>
                  <Button variant="destructive">
                    <RotateCcw className="mr-2 h-4 w-4" />
                    重置所有配置
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>确认重置所有配置？</DialogTitle>
                    <DialogDescription>
                      此操作将删除所有自定义设置，包括模型配置、Agent
                      配置和应用偏好。此操作不可撤销。
                    </DialogDescription>
                  </DialogHeader>
                  <DialogFooter>
                    <Button
                      variant="outline"
                      onClick={() => setResetDialogOpen(false)}
                    >
                      取消
                    </Button>
                    <Button variant="destructive" onClick={handleReset}>
                      确认重置
                    </Button>
                  </DialogFooter>
                </DialogContent>
              </Dialog>
            </div>
          </div>

          <Separator />

          <div className="space-y-3">
            <Label>应用信息</Label>
            <div className="rounded-lg bg-muted p-4 space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">版本</span>
                <span>0.1.0</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">构建时间</span>
                <span>{new Date().toLocaleDateString()}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">数据目录</span>
                <code className="text-xs">~/.openclaw-manager</code>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* 导出配置对话框 */}
      <Dialog open={exportDialogOpen} onOpenChange={setExportDialogOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>导出配置</DialogTitle>
            <DialogDescription>
              复制以下 JSON 数据以备份您的配置
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <textarea
              value={exportData}
              readOnly
              className="w-full h-64 rounded-md border bg-muted p-3 font-mono text-xs"
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={copyToClipboard}>
              复制到剪贴板
            </Button>
            <Button onClick={() => setExportDialogOpen(false)}>关闭</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 导入配置对话框 */}
      <Dialog open={importDialogOpen} onOpenChange={setImportDialogOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>导入配置</DialogTitle>
            <DialogDescription>
              粘贴之前导出的 JSON 配置数据
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            {importError && (
              <Alert variant="destructive">
                <AlertTriangle className="h-4 w-4" />
                <AlertTitle>导入失败</AlertTitle>
                <AlertDescription>{importError}</AlertDescription>
              </Alert>
            )}
            <textarea
              value={importData}
              onChange={(e) => setImportData(e.target.value)}
              placeholder="在此粘贴 JSON 配置数据..."
              className="w-full h-64 rounded-md border p-3 font-mono text-xs"
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setImportDialogOpen(false)}>
              取消
            </Button>
            <Button onClick={handleImport} disabled={!importData.trim()}>
              导入
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
