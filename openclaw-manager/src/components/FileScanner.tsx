import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { fileApi } from '@/lib/tauri-api'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { FolderOpen, Scan, Loader2 } from 'lucide-react'
import { open } from '@tauri-apps/api/dialog'

export function FileScanner() {
  const [path, setPath] = useState('')
  const [recursive, setRecursive] = useState(true)
  const queryClient = useQueryClient()

  const scanMutation = useMutation({
    mutationFn: () =>
      fileApi.scan({
        path,
        recursive,
        file_types: ['mp4', 'avi', 'mkv', 'mov', 'mp3', 'wav', 'jpg', 'png', 'pdf', 'txt'],
      }),
    onSuccess: (data) => {
      if (data.success) {
        queryClient.invalidateQueries({ queryKey: ['files'] })
        alert(`扫描完成！找到 ${data.data?.total_count || 0} 个文件`)
      }
    },
  })

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      })
      if (selected && typeof selected === 'string') {
        setPath(selected)
      }
    } catch (e) {
      console.error('Failed to open dialog:', e)
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Scan className="h-5 w-5" />
          文件扫描
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex gap-2">
          <Input
            placeholder="选择文件夹路径"
            value={path}
            onChange={(e) => setPath(e.target.value)}
            className="flex-1"
          />
          <Button variant="outline" onClick={handleSelectFolder}>
            <FolderOpen className="h-4 w-4 mr-2" />
            浏览
          </Button>
        </div>

        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label>递归扫描子文件夹</Label>
            <p className="text-sm text-muted-foreground">
              扫描所选文件夹及其所有子文件夹
            </p>
          </div>
          <Switch checked={recursive} onCheckedChange={setRecursive} />
        </div>

        <Button
          onClick={() => scanMutation.mutate()}
          disabled={!path || scanMutation.isPending}
          className="w-full"
        >
          {scanMutation.isPending ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              扫描中...
            </>
          ) : (
            <>
              <Scan className="mr-2 h-4 w-4" />
              开始扫描
            </>
          )}
        </Button>
      </CardContent>
    </Card>
  )
}
