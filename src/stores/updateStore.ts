import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { UpdateInfo, UpdateProgress } from '@/lib/tauri-api';

// UpdateProgress type is imported from tauri-api

interface UpdateState {
  // 当前版本信息
  currentVersion: string | null;
  setCurrentVersion: (version: string | null) => void;

  // 最新版本信息
  latestVersion: string | null;
  setLatestVersion: (version: string | null) => void;
  updateInfo: UpdateInfo | null;
  setUpdateInfo: (info: UpdateInfo | null) => void;

  // 更新检查状态
  hasUpdate: boolean;
  setHasUpdate: (hasUpdate: boolean) => void;
  isChecking: boolean;
  setIsChecking: (checking: boolean) => void;
  lastCheckTime: number | null;
  setLastCheckTime: (time: number | null) => void;

  // 下载/安装进度
  progress: UpdateProgress | null;
  setProgress: (progress: UpdateProgress | null) => void;
  updateProgress: (updates: Partial<UpdateProgress>) => void;

  // 更新状态
  updateStatus: 'idle' | 'checking' | 'downloading' | 'installing' | 'completed' | 'error' | 'rollback';
  setUpdateStatus: (status: UpdateState['updateStatus']) => void;

  // 更新设置
  settings: {
    autoCheck: boolean;
    autoDownload: boolean;
    autoInstall: boolean;
    checkInterval: number; // 小时
    notifyOnUpdate: boolean;
    skipVersion: string | null;
  };
  setSettings: (settings: Partial<UpdateState['settings']>) => void;
  setAutoCheck: (enabled: boolean) => void;
  setAutoDownload: (enabled: boolean) => void;
  setAutoInstall: (enabled: boolean) => void;
  setCheckInterval: (hours: number) => void;
  setNotifyOnUpdate: (enabled: boolean) => void;
  setSkipVersion: (version: string | null) => void;

  // 备份相关
  backups: Array<{
    createdAt: string;
    version: string | null;
    path: string;
  }>;
  setBackups: (backups: UpdateState['backups']) => void;
  isCreatingBackup: boolean;
  setIsCreatingBackup: (creating: boolean) => void;

  // 离线更新
  offlinePackagePath: string | null;
  setOfflinePackagePath: (path: string | null) => void;

  // 错误状态
  error: string | null;
  setError: (error: string | null) => void;
  errorDetails: string | null;
  setErrorDetails: (details: string | null) => void;

  // 是否可以取消
  canCancel: boolean;
  setCanCancel: (canCancel: boolean) => void;

  // 是否需要重启
  needsRestart: boolean;
  setNeedsRestart: (needsRestart: boolean) => void;

  // 更新历史
  updateHistory: Array<{
    version: string;
    timestamp: number;
    status: 'success' | 'failed' | 'rolled_back';
    message?: string;
  }>;
  addUpdateHistory: (record: UpdateState['updateHistory'][0]) => void;

  // 计算属性
  isUpdateAvailable: () => boolean;
  isUpdateInProgress: () => boolean;
  shouldCheckForUpdate: () => boolean;

  // 重置
  reset: () => void;
  resetProgress: () => void;
}

const defaultSettings = {
  autoCheck: true,
  autoDownload: false,
  autoInstall: false,
  checkInterval: 24, // 默认24小时检查一次
  notifyOnUpdate: true,
  skipVersion: null,
};

export const useUpdateStore = create<UpdateState>()(
  persist(
    (set, get) => ({
      // 初始状态
      currentVersion: null,
      latestVersion: null,
      updateInfo: null,
      hasUpdate: false,
      isChecking: false,
      lastCheckTime: null,
      progress: null,
      updateStatus: 'idle',
      settings: defaultSettings,
      backups: [],
      isCreatingBackup: false,
      offlinePackagePath: null,
      error: null,
      errorDetails: null,
      canCancel: true,
      needsRestart: false,
      updateHistory: [],

      // 版本信息管理
      setCurrentVersion: (currentVersion) => set({ currentVersion }),
      setLatestVersion: (latestVersion) => set({ latestVersion }),
      setUpdateInfo: (updateInfo) => set({ updateInfo }),

      // 更新检查状态
      setHasUpdate: (hasUpdate) => set({ hasUpdate }),
      setIsChecking: (isChecking) => set({ isChecking }),
      setLastCheckTime: (lastCheckTime) => set({ lastCheckTime }),

      // 进度管理
      setProgress: (progress) => set({ progress }),
      updateProgress: (updates) =>
        set((state) => ({
          progress: state.progress ? { ...state.progress, ...updates } : null,
        })),

      // 更新状态
      setUpdateStatus: (updateStatus) => set({ updateStatus }),

      // 设置管理
      setSettings: (settings) =>
        set((state) => ({
          settings: { ...state.settings, ...settings },
        })),
      setAutoCheck: (autoCheck) =>
        set((state) => ({
          settings: { ...state.settings, autoCheck },
        })),
      setAutoDownload: (autoDownload) =>
        set((state) => ({
          settings: { ...state.settings, autoDownload },
        })),
      setAutoInstall: (autoInstall) =>
        set((state) => ({
          settings: { ...state.settings, autoInstall },
        })),
      setCheckInterval: (checkInterval) =>
        set((state) => ({
          settings: { ...state.settings, checkInterval },
        })),
      setNotifyOnUpdate: (notifyOnUpdate) =>
        set((state) => ({
          settings: { ...state.settings, notifyOnUpdate },
        })),
      setSkipVersion: (skipVersion) =>
        set((state) => ({
          settings: { ...state.settings, skipVersion },
          hasUpdate: skipVersion === state.latestVersion ? false : state.hasUpdate,
        })),

      // 备份管理
      setBackups: (backups) => set({ backups }),
      setIsCreatingBackup: (isCreatingBackup) => set({ isCreatingBackup }),

      // 离线更新
      setOfflinePackagePath: (offlinePackagePath) => set({ offlinePackagePath }),

      // 错误管理
      setError: (error) => set({ error }),
      setErrorDetails: (errorDetails) => set({ errorDetails }),

      // 取消状态
      setCanCancel: (canCancel) => set({ canCancel }),

      // 重启状态
      setNeedsRestart: (needsRestart) => set({ needsRestart }),

      // 更新历史
      addUpdateHistory: (record) =>
        set((state) => ({
          updateHistory: [record, ...state.updateHistory].slice(0, 50), // 最多保留50条
        })),

      // 计算属性方法
      isUpdateAvailable: () => {
        const state = get();
        return (
          state.hasUpdate &&
          state.updateInfo !== null &&
          state.updateInfo.version !== state.settings.skipVersion
        );
      },

      isUpdateInProgress: () => {
        const state = get();
        return (
          state.updateStatus === 'downloading' ||
          state.updateStatus === 'installing'
        );
      },

      shouldCheckForUpdate: () => {
        const state = get();
        if (!state.settings.autoCheck) return false;
        if (state.lastCheckTime === null) return true;
        const intervalMs = state.settings.checkInterval * 60 * 60 * 1000;
        return Date.now() - state.lastCheckTime > intervalMs;
      },

      // 重置进度
      resetProgress: () =>
        set({
          progress: null,
          updateStatus: 'idle',
          error: null,
          errorDetails: null,
          canCancel: true,
        }),

      // 重置所有状态（保留设置）
      reset: () =>
        set({
          currentVersion: null,
          latestVersion: null,
          updateInfo: null,
          hasUpdate: false,
          isChecking: false,
          lastCheckTime: null,
          progress: null,
          updateStatus: 'idle',
          backups: [],
          isCreatingBackup: false,
          offlinePackagePath: null,
          error: null,
          errorDetails: null,
          canCancel: true,
          needsRestart: false,
          // 保留 settings 和 updateHistory
        }),
    }),
    {
      name: 'update-storage',
      partialize: (state) => ({
        // 持久化设置和历史记录
        settings: state.settings,
        updateHistory: state.updateHistory,
        currentVersion: state.currentVersion,
        lastCheckTime: state.lastCheckTime,
        skipVersion: state.settings.skipVersion,
      }),
    }
  )
);

// 选择器 - 优化订阅性能
export const selectCurrentVersion = (state: UpdateState) => state.currentVersion;
export const selectLatestVersion = (state: UpdateState) => state.latestVersion;
export const selectUpdateInfo = (state: UpdateState) => state.updateInfo;
export const selectHasUpdate = (state: UpdateState) => state.hasUpdate;
export const selectIsChecking = (state: UpdateState) => state.isChecking;
export const selectProgress = (state: UpdateState) => state.progress;
export const selectUpdateStatus = (state: UpdateState) => state.updateStatus;
export const selectSettings = (state: UpdateState) => state.settings;
export const selectError = (state: UpdateState) => state.error;
export const selectNeedsRestart = (state: UpdateState) => state.needsRestart;
