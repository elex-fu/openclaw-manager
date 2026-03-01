import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { 
  InstallStatus, 
  InstallProgress, 
  InstallMethod, 
  InstallLogEntry,
  SystemCheckResult 
} from '@/types';

interface InstallState {
  // 安装状态
  installStatus: InstallStatus;
  setInstallStatus: (status: InstallStatus) => void;

  // 安装进度
  progress: InstallProgress | null;
  setProgress: (progress: InstallProgress | null) => void;

  // 安装方式
  installMethod: InstallMethod;
  setInstallMethod: (method: InstallMethod) => void;

  // 安装日志
  logs: InstallLogEntry[];
  addLog: (message: string, level?: InstallLogEntry['level']) => void;
  clearLogs: () => void;

  // 系统检查
  systemChecks: SystemCheckResult[];
  setSystemChecks: (checks: SystemCheckResult[]) => void;
  updateSystemCheck: (name: string, check: Partial<SystemCheckResult>) => void;

  // 向导步骤
  wizardStep: number;
  setWizardStep: (step: number) => void;
  nextStep: () => void;
  prevStep: () => void;

  // 离线包路径
  offlinePackagePath: string | null;
  setOfflinePackagePath: (path: string | null) => void;

  // 安装版本
  targetVersion: string;
  setTargetVersion: (version: string) => void;

  // 是否正在安装
  isInstalling: boolean;
  setIsInstalling: (installing: boolean) => void;

  // 重置
  reset: () => void;
}

export const useInstallStore = create<InstallState>()(
  persist(
    (set, get) => ({
      // 初始状态
      installStatus: { type: 'NotInstalled' },
      progress: null,
      installMethod: 'online',
      logs: [],
      systemChecks: [],
      wizardStep: 0,
      offlinePackagePath: null,
      targetVersion: 'latest',
      isInstalling: false,

      // 安装状态
      setInstallStatus: (status) => set({ installStatus: status }),

      // 进度
      setProgress: (progress) => set({ progress }),

      // 安装方式
      setInstallMethod: (method) => set({ installMethod: method }),

      // 日志
      addLog: (message, level = 'info') => {
        const entry: InstallLogEntry = {
          timestamp: new Date().toLocaleTimeString(),
          level,
          message,
        };
        set((state) => ({ logs: [...state.logs, entry] }));
      },
      clearLogs: () => set({ logs: [] }),

      // 系统检查
      setSystemChecks: (checks) => set({ systemChecks: checks }),
      updateSystemCheck: (name, check) =>
        set((state) => ({
          systemChecks: state.systemChecks.map((c) =>
            c.name === name ? { ...c, ...check } : c
          ),
        })),

      // 向导步骤
      setWizardStep: (step) => set({ wizardStep: step }),
      nextStep: () => set((state) => ({ wizardStep: state.wizardStep + 1 })),
      prevStep: () => set((state) => ({ wizardStep: Math.max(0, state.wizardStep - 1) })),

      // 离线包
      setOfflinePackagePath: (path) => set({ offlinePackagePath: path }),

      // 目标版本
      setTargetVersion: (version) => set({ targetVersion: version }),

      // 安装状态
      setIsInstalling: (installing) => set({ isInstalling: installing }),

      // 重置
      reset: () => set({
        installStatus: { type: 'NotInstalled' },
        progress: null,
        installMethod: 'online',
        logs: [],
        systemChecks: [],
        wizardStep: 0,
        offlinePackagePath: null,
        targetVersion: 'latest',
        isInstalling: false,
      }),
    }),
    {
      name: 'install-storage',
      partialize: (state) => ({
        installMethod: state.installMethod,
        targetVersion: state.targetVersion,
        offlinePackagePath: state.offlinePackagePath,
      }),
    }
  )
);
