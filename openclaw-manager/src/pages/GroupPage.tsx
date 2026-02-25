import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { groupApi } from '@/lib/tauri-api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Label } from '@/components/ui/label'
import { FolderOpen, Plus, Trash2 } from 'lucide-react'
import type { Group } from '@/types'

export function GroupPage() {
  const [newGroupName, setNewGroupName] = useState('')
  const [newGroupDesc, setNewGroupDesc] = useState('')
  const [dialogOpen, setDialogOpen] = useState(false)
  const queryClient = useQueryClient()

  const { data: groupsData, isLoading } = useQuery({
    queryKey: ['groups'],
    queryFn: () => groupApi.getAll(true),
  })

  const createGroupMutation = useMutation({
    mutationFn: (name: string) => groupApi.create(name, newGroupDesc),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['groups'] })
      setNewGroupName('')
      setNewGroupDesc('')
      setDialogOpen(false)
    },
  })

  const deleteGroupMutation = useMutation({
    mutationFn: (id: string) => groupApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['groups'] })
    },
  })

  const groups = groupsData?.data || []

  const handleCreateGroup = () => {
    if (newGroupName.trim()) {
      createGroupMutation.mutate(newGroupName.trim())
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">分组管理</h2>
        <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="mr-2 h-4 w-4" />
              新建分组
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>新建分组</DialogTitle>
              <DialogDescription>
                创建一个新的文件分组来组织您的文件
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-4 py-4">
              <div className="space-y-2">
                <Label htmlFor="name">分组名称</Label>
                <Input
                  id="name"
                  placeholder="输入分组名称"
                  value={newGroupName}
                  onChange={(e) => setNewGroupName(e.target.value)}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="description">描述（可选）</Label>
                <Input
                  id="description"
                  placeholder="输入分组描述"
                  value={newGroupDesc}
                  onChange={(e) => setNewGroupDesc(e.target.value)}
                />
              </div>
            </div>
            <DialogFooter>
              <Button
                onClick={handleCreateGroup}
                disabled={createGroupMutation.isPending || !newGroupName.trim()}
              >
                {createGroupMutation.isPending ? '创建中...' : '创建'}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      {isLoading ? (
        <div className="flex h-64 items-center justify-center">
          <div className="text-muted-foreground">加载中...</div>
        </div>
      ) : groups.length === 0 ? (
        <div className="flex h-64 flex-col items-center justify-center text-muted-foreground">
          <FolderOpen className="mb-4 h-12 w-12" />
          <p>暂无分组</p>
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {groups.map((group) => (
            <GroupCard
              key={group.id}
              group={group}
              onDelete={() => deleteGroupMutation.mutate(group.id)}
              isDeleting={deleteGroupMutation.isPending}
            />
          ))}
        </div>
      )}
    </div>
  )
}

interface GroupCardProps {
  group: {
    id: string
    name: string
    description?: string
    file_count: number
    is_default?: boolean
  }
  onDelete: () => void
  isDeleting: boolean
}

function GroupCard({ group, onDelete, isDeleting }: GroupCardProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <div className="flex items-center gap-2">
          <FolderOpen className="h-5 w-5 text-primary" />
          <CardTitle className="text-lg">{group.name}</CardTitle>
        </div>
        {!group.is_default && (
          <Button
            variant="ghost"
            size="icon"
            onClick={onDelete}
            disabled={isDeleting}
          >
            <Trash2 className="h-4 w-4 text-destructive" />
          </Button>
        )}
      </CardHeader>
      <CardContent>
        {group.description && (
          <p className="text-sm text-muted-foreground mb-2">
            {group.description}
          </p>
        )}
        <p className="text-sm">
          <span className="font-medium">{group.file_count}</span> 个文件
        </p>
      </CardContent>
    </Card>
  )
}
