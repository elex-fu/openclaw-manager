import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Settings, FolderOpen, Bell, Shield } from 'lucide-react'

export function SettingsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">设置</h2>
        <p className="text-muted-foreground">配置应用偏好和系统选项</p>
      </div>

      <Tabs defaultValue="general" className="space-y-4">
        <TabsList>
          <TabsTrigger value="general">常规</TabsTrigger>
          <TabsTrigger value="scanning">文件扫描</TabsTrigger>
          <TabsTrigger value="notifications">通知</TabsTrigger>
          <TabsTrigger value="advanced">高级</TabsTrigger>
        </TabsList>

        <TabsContent value="general" className="space-y-4">
          <GeneralSettings />
        </TabsContent>

        <TabsContent value="scanning" className="space-y-4">
          <ScanningSettings />
        </TabsContent>

        <TabsContent value="notifications" className="space-y-4">
          <NotificationSettings />
        </TabsContent>

        <TabsContent value="advanced" className="space-y-4">
          <AdvancedSettings />
        </TabsContent>
      </Tabs>
    </div>
  )
}

function GeneralSettings() {
  const [autoStart, setAutoStart] = useState(false)
  const [minimizeToTray, setMinimizeToTray] = useState(true)

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Settings className="h-5 w-5" />
          常规设置
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>开机自启</Label>
            <p className="text-sm text-muted-foreground">
              系统启动时自动运行应用
            </p>
          </div>
          <Switch checked={autoStart} onCheckedChange={setAutoStart} />
        </div>
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>最小化到托盘</Label>
            <p className="text-sm text-muted-foreground">
              关闭窗口时最小化到系统托盘
            </p>
          </div>
          <Switch checked={minimizeToTray} onCheckedChange={setMinimizeToTray} />
        </div>
      </CardContent>
    </Card>
  )
}

function ScanningSettings() {
  const [defaultPath, setDefaultPath] = useState('')
  const [autoScan, setAutoScan] = useState(false)

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <FolderOpen className="h-5 w-5" />
          文件扫描设置
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label>默认扫描路径</Label>
          <div className="flex gap-2">
            <Input
              placeholder="选择默认扫描路径"
              value={defaultPath}
              onChange={(e) => setDefaultPath(e.target.value)}
            />
            <Button variant="outline">浏览</Button>
          </div>
        </div>
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>自动扫描</Label>
            <p className="text-sm text-muted-foreground">
              启动时自动扫描默认路径
            </p>
          </div>
          <Switch checked={autoScan} onCheckedChange={setAutoScan} />
        </div>
      </CardContent>
    </Card>
  )
}

function NotificationSettings() {
  const [enableNotifications, setEnableNotifications] = useState(true)
  const [scanCompleteNotify, setScanCompleteNotify] = useState(true)

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Bell className="h-5 w-5" />
          通知设置
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>启用通知</Label>
            <p className="text-sm text-muted-foreground">
              显示桌面通知
            </p>
          </div>
          <Switch
            checked={enableNotifications}
            onCheckedChange={setEnableNotifications}
          />
        </div>
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>扫描完成通知</Label>
            <p className="text-sm text-muted-foreground">
              文件扫描完成后显示通知
            </p>
          </div>
          <Switch
            checked={scanCompleteNotify}
            onCheckedChange={setScanCompleteNotify}
            disabled={!enableNotifications}
          />
        </div>
      </CardContent>
    </Card>
  )
}

function AdvancedSettings() {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Shield className="h-5 w-5" />
          高级设置
        </CardTitle>
        <CardDescription>
          这些设置仅供高级用户使用
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label>数据库位置</Label>
          <Input value="%APPDATA%/openclaw-manager/openclaw.db" disabled />
        </div>
        <div className="flex gap-2">
          <Button variant="outline">导出数据</Button>
          <Button variant="outline">导入数据</Button>
          <Button variant="destructive">清除所有数据</Button>
        </div>
      </CardContent>
    </Card>
  )
}
