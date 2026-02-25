import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { fileApi } from '@/lib/tauri-api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { FileText, Search, FolderOpen, RefreshCw } from 'lucide-react'
import { formatFileSize, formatDate, getFileIcon } from '@/lib/utils'
import { FileScanner } from '@/components/FileScanner'
import type { FileItem } from '@/types'

export function FileListPage() {
  const [searchQuery, setSearchQuery] = useState('')
  const [filter, setFilter] = useState('all')

  const { data: filesData, isLoading, refetch } = useQuery({
    queryKey: ['files', filter],
    queryFn: () => fileApi.getAll({
      is_collected: filter === 'collected' ? true : undefined,
      is_classified: filter === 'classified' ? true : undefined,
    }),
  })

  const files = filesData?.data || []

  const filteredFiles = files.filter((file) =>
    file.file_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    file.file_path.toLowerCase().includes(searchQuery.toLowerCase())
  )

  return (
    <div className="space-y-6">
      {/* File Scanner */}
      <FileScanner />

      {/* Stats Cards */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">总文件数</CardTitle>
            <FileText className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{files.length}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">已收录</CardTitle>
            <FolderOpen className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {files.filter((f) => f.is_collected).length}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Filters and Search */}
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <Tabs value={filter} onValueChange={setFilter}>
          <TabsList>
            <TabsTrigger value="all">全部</TabsTrigger>
            <TabsTrigger value="collected">已收录</TabsTrigger>
            <TabsTrigger value="classified">已分类</TabsTrigger>
          </TabsList>
        </Tabs>

        <div className="flex items-center gap-2">
          <div className="relative">
            <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              type="search"
              placeholder="搜索文件..."
              className="w-64 pl-8"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
          <Button variant="outline" size="icon" onClick={() => refetch()}>
            <RefreshCw className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {/* File List */}
      <Card>
        <CardContent className="p-0">
          {isLoading ? (
            <div className="flex h-64 items-center justify-center">
              <RefreshCw className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : filteredFiles.length === 0 ? (
            <div className="flex h-64 flex-col items-center justify-center text-muted-foreground">
              <FileText className="mb-4 h-12 w-12" />
              <p>暂无文件</p>
            </div>
          ) : (
            <div className="divide-y">
              {filteredFiles.map((file) => (
                <FileListItem key={file.id} file={file} />
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}

function FileListItem({ file }: { file: FileItem }) {
  return (
    <div className="flex items-center justify-between p-4 hover:bg-accent/50">
      <div className="flex items-center gap-4">
        <FileText className="h-8 w-8 text-muted-foreground" />
        <div>
          <p className="font-medium">{file.file_name}</p>
          <p className="text-sm text-muted-foreground">{file.file_path}</p>
        </div>
      </div>
      <div className="flex items-center gap-4 text-sm text-muted-foreground">
        <span>{file.file_type.toUpperCase()}</span>
        <span>{formatFileSize(file.file_size)}</span>
        <span>{formatDate(file.created_at)}</span>
        {file.is_collected && (
          <span className="rounded-full bg-green-100 px-2 py-0.5 text-xs text-green-800">
            已收录
          </span>
        )}
      </div>
    </div>
  )
}
