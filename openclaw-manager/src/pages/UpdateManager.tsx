import { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import { updateApi, type UpdateState, type UpdateInfo, type UpdateProgress, type BackupMetadata } from '@/lib/tauri-api';
import {
  RefreshCw,
  Download,
  RotateCcw,
  CheckCircle2,
  AlertCircle,
  Clock,
  HardDrive,
  FileArchive,
  AlertTriangle,
} from 'lucide-react';

// Dynamic import for Tauri dialog plugin (avoids test issues)
const openDialog = async () => {
  try {
    // Use string literal to prevent vite from pre-resolving
    const moduleName = '@tauri-apps/plugin-dialog';
    const dialog = await import(/* @vite-ignore */ moduleName);
    return dialog.open;
  } catch {
    return null;
  }
};

// Simple date formatter (avoids date-fns dependency in tests)
const formatDate = (dateStr: string) => {
  try {
    const date = new Date(dateStr);
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch {
    return dateStr;
  }
};

interface UpdateStageInfo {
  id: string;
  label: string;
  icon: React.ReactNode;
}

const updateStages: UpdateStageInfo[] = [
  { id: 'Checking', label: '检查更新', icon: <RefreshCw className="h-4 w-4" /> },
  { id: 'Downloading', label: '下载更新', icon: <Download className="h-4 w-4" /> },
  { id: 'BackingUp', label: '备份数据', icon: <HardDrive className="h-4 w-4" /> },
  { id: 'Installing', label: '安装更新', icon: <CheckCircle2 className="h-4 w-4" /> },
  { id: 'Migrating', label: '迁移配置', icon: <RotateCcw className="h-4 w-4" /> },
  { id: 'CleaningUp', label: '清理文件', icon: <HardDrive className="h-4 w-4" /> },
  { id: 'Complete', label: '完成', icon: <CheckCircle2 className="h-4 w-4" /> },
  { id: 'Error', label: '错误', icon: <AlertCircle className="h-4 w-4" /> },
  { id: 'Rollback', label: '回滚', icon: <RotateCcw className="h-4 w-4" /> },
];

export function UpdateManager() {
  const [updateState, setUpdateState] = useState<UpdateState | null>(null);
  const [isChecking, setIsChecking] = useState(false);
  const [isUpdating, setIsUpdating] = useState(false);
  const [progress, setProgress] = useState<UpdateProgress | null>(null);
  const [backups, setBackups] = useState<BackupMetadata[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  // 检查更新
  const checkForUpdates = useCallback(async () => {
    setIsChecking(true);
    setError(null);
    try {
      const response = await updateApi.checkForUpdates();
      if (response.success && response.data) {
        setUpdateState(response.data);
        if (response.data.hasUpdate) {
          setSuccessMessage(`发现新版本: ${response.data.latestVersion}`);
        } else {
          setSuccessMessage('已是最新版本');
        }
      } else {
        setError(response.error || '检查更新失败');
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '检查更新时出错');
    } finally {
      setIsChecking(false);
    }
  }, []);

  // 执行升级
  const performUpdate = async (updateInfo: UpdateInfo) => {
    setIsUpdating(true);
    setError(null);
    setProgress(null);

    try {
      // 监听进度
      const unlisten = await updateApi.onUpdateProgress((p) => {
        setProgress(p);
      });

      const response = await updateApi.performUpdate(updateInfo);

      // 清理监听器
      unlisten();

      if (response.success) {
        setSuccessMessage('升级成功完成！');
        // 重新检查更新状态
        await checkForUpdates();
      } else {
        setError(response.error || '升级失败');
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '升级时出错');
    } finally {
      setIsUpdating(false);
      setProgress(null);
    }
  };

  // 离线升级
  const performOfflineUpdate = async () => {
    try {
      const open = await openDialog();
      if (!open) {
        setError('文件选择器不可用');
        return;
      }

      const selected = await open({
        multiple: false,
        filters: [
          { name: '安装包', extensions: ['tar.gz', 'zip'] },
          { name: '所有文件', extensions: ['*'] },
        ],
      });

      if (selected && typeof selected === 'string') {
        setIsUpdating(true);
        setError(null);
        setProgress(null);

        // 监听进度
        const unlisten = await updateApi.onUpdateProgress((p) => {
          setProgress(p);
        });

        const response = await updateApi.performOfflineUpdate(selected);

        unlisten();

        if (response.success) {
          setSuccessMessage('离线升级成功完成！');
          await checkForUpdates();
        } else {
          setError(response.error || '离线升级失败');
        }
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '离线升级时出错');
    } finally {
      setIsUpdating(false);
      setProgress(null);
    }
  };

  // 加载备份列表
  const loadBackups = useCallback(async () => {
    try {
      const response = await updateApi.getBackupList();
      if (response.success && response.data) {
        setBackups(response.data);
      }
    } catch (e) {
      console.error('加载备份列表失败:', e);
    }
  }, []);

  // 从备份恢复
  const restoreFromBackup = async (backupPath: string) => {
    if (!confirm('确定要从该备份恢复吗？当前配置将被覆盖。')) {
      return;
    }

    setIsUpdating(true);
    setError(null);

    try {
      const response = await updateApi.restoreFromBackup(backupPath);
      if (response.success) {
        setSuccessMessage('从备份恢复成功！');
        await checkForUpdates();
      } else {
        setError(response.error || '恢复备份失败');
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : '恢复备份时出错');
    } finally {
      setIsUpdating(false);
    }
  };

  // 初始加载
  useEffect(() => {
    checkForUpdates();
    loadBackups();
  }, [checkForUpdates, loadBackups]);

  // 清除消息
  useEffect(() => {
    if (error || successMessage) {
      const timer = setTimeout(() => {
        setError(null);
        setSuccessMessage(null);
      }, 5000);
      return () => clearTimeout(timer);
    }
  }, [error, successMessage]);

  const currentStage = updateStages.find(s => s.id === progress?.stage);

  return (
    <div className="container mx-auto py-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">版本升级</h1>
          <p className="text-muted-foreground mt-1">
            检查并安装 OpenClaw 的更新版本
          </p>
        </div>
        <Button
          variant="outline"
          onClick={checkForUpdates}
          disabled={isChecking || isUpdating}
        >
          <RefreshCw className={`mr-2 h-4 w-4 ${isChecking ? 'animate-spin' : ''}`} />
          {isChecking ? '检查中...' : '检查更新'}
        </Button>
      </div>

      {/* 错误提示 */}
      {error && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertTitle>错误</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {/* 成功提示 */}
      {successMessage && (
        <Alert className="bg-green-50 border-green-200">
          <CheckCircle2 className="h-4 w-4 text-green-600" />
          <AlertTitle className="text-green-800">成功</AlertTitle>
          <AlertDescription className="text-green-700">{successMessage}</AlertDescription>
        </Alert>
      )}

      {/* 当前版本状态 */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <CheckCircle2 className="h-5 w-5 text-primary" />
            当前版本
          </CardTitle>
          <CardDescription>
            查看当前安装的 OpenClaw 版本信息
          </CardDescription>
        </CardHeader>
        <CardContent>
          {updateState ? (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">当前版本</span>
                <Badge variant="secondary" className="text-lg px-3 py-1">
                  {updateState.currentVersion || '未安装'}
                </Badge>
              </div>
              {updateState.hasUpdate && (
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground">最新版本</span>
                  <Badge className="bg-green-100 text-green-800 text-lg px-3 py-1">
                    {updateState.latestVersion}
                  </Badge>
                </div>
              )}
            </div>
          ) : (
            <div className="text-center py-8 text-muted-foreground">
              正在加载版本信息...
            </div>
          )}
        </CardContent>
      </Card>

      {/* 可用更新 */}
      {updateState?.hasUpdate && updateState.updateInfo && (
        <Card className="border-green-200 shadow-md">
          <CardHeader className="bg-green-50/50">
            <div className="flex items-center justify-between">
              <div>
                <CardTitle className="flex items-center gap-2 text-green-800">
                  <Download className="h-5 w-5" />
                  发现新版本
                </CardTitle>
                <CardDescription className="text-green-700 mt-1">
                  有新版本可用，建议及时升级以获得最新功能
                </CardDescription>
              </div>
              {updateState.updateInfo.mandatory && (
                <Badge variant="destructive" className="flex items-center gap-1">
                  <AlertTriangle className="h-3 w-3" />
                  强制更新
                </Badge>
              )}
            </div>
          </CardHeader>
          <CardContent className="space-y-4 pt-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <span className="text-muted-foreground text-sm">版本号</span>
                <p className="font-medium text-lg">{updateState.updateInfo.version}</p>
              </div>
              <div>
                <span className="text-muted-foreground text-sm">发布日期</span>
                <p className="font-medium">{updateState.updateInfo.releaseDate}</p>
              </div>
            </div>

            {updateState.updateInfo.changelog && (
              <div>
                <span className="text-muted-foreground text-sm">更新日志</span>
                <ScrollArea className="h-32 mt-2 border rounded-md p-3 bg-muted/30">
                  <pre className="text-sm whitespace-pre-wrap">
                    {updateState.updateInfo.changelog}
                  </pre>
                </ScrollArea>
              </div>
            )}

            <Button
              className="w-full"
              size="lg"
              onClick={() => performUpdate(updateState.updateInfo!)}
              disabled={isUpdating}
            >
              {isUpdating ? (
                <>
                  <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                  升级中...
                </>
              ) : (
                <>
                  <Download className="mr-2 h-4 w-4" />
                  立即升级
                </>
              )}
            </Button>
          </CardContent>
        </Card>
      )}

      {/* 升级进度 */}
      {isUpdating && progress && (
        <Card className="border-primary">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              {currentStage?.icon}
              升级进度
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <Progress value={progress.percentage} className="h-3" />
            <div className="flex items-center justify-between text-sm">
              <span className="font-medium">{currentStage?.label}</span>
              <span className="text-muted-foreground">{Math.round(progress.percentage)}%</span>
            </div>
            <p className="text-muted-foreground text-sm">
              {progress.message}
            </p>
          </CardContent>
        </Card>
      )}

      {/* 离线升级 */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <FileArchive className="h-5 w-5" />
            离线升级
          </CardTitle>
          <CardDescription>
            使用本地安装包进行升级，适用于无网络环境
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Button
            variant="outline"
            onClick={performOfflineUpdate}
            disabled={isUpdating}
            className="w-full"
          >
            <FileArchive className="mr-2 h-4 w-4" />
            选择本地安装包
          </Button>
        </CardContent>
      </Card>

      {/* 备份管理 */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <RotateCcw className="h-5 w-5" />
            备份管理
          </CardTitle>
          <CardDescription>
            查看和管理自动创建的备份，可在升级失败时恢复
          </CardDescription>
        </CardHeader>
        <CardContent>
          {backups.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              暂无备份记录
            </div>
          ) : (
            <div className="space-y-2">
              {backups.map((backup, index) => (
                <div key={index}>
                  <div className="flex items-center justify-between py-3">
                    <div className="flex items-center gap-3">
                      <Clock className="h-4 w-4 text-muted-foreground" />
                      <div>
                        <p className="font-medium">
                          {formatDate(backup.createdAt)}
                        </p>
                        {backup.version && (
                          <p className="text-sm text-muted-foreground">
                            版本: {backup.version}
                          </p>
                        )}
                      </div>
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => restoreFromBackup(backup.path)}
                      disabled={isUpdating}
                    >
                      <RotateCcw className="mr-2 h-4 w-4" />
                      恢复
                    </Button>
                  </div>
                  {index < backups.length - 1 && <Separator />}
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
