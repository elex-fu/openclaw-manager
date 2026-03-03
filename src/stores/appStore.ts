import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import type { AppPage, Notification, Theme } from '@/types';

// 简单的数据压缩（可选）
function compressData(data: string): string {
  // 在生产环境中可以使用 lz-string 等库进行压缩
  // 这里使用简单的 Base64 编码作为示例
  try {
    return btoa(encodeURIComponent(data));
  } catch {
    return data;
  }
}

function decompressData(data: string): string {
  try {
    return decodeURIComponent(atob(data));
  } catch {
    return data;
  }
}

// 自定义存储，支持压缩
const createCompressedStorage = () => {
  return {
    getItem: (name: string): string | null => {
      const item = localStorage.getItem(name);
      if (!item) return null;
      // 尝试解压
      return decompressData(item);
    },
    setItem: (name: string, value: string): void => {
      // 压缩存储
      localStorage.setItem(name, compressData(value));
    },
    removeItem: (name: string): void => {
      localStorage.removeItem(name);
    },
  };
};

interface AppState {
  // 当前页面
  currentPage: AppPage;
  setCurrentPage: (page: AppPage) => void;

  // 全局通知
  notifications: Notification[];
  addNotification: (notification: Omit<Notification, 'id' | 'timestamp'>) => void;
  removeNotification: (id: string) => void;
  clearNotifications: () => void;

  // 主题设置
  theme: Theme;
  setTheme: (theme: Theme) => void;

  // 侧边栏状态
  sidebarOpen: boolean;
  setSidebarOpen: (open: boolean) => void;
  toggleSidebar: () => void;

  // 全局加载状态
  isLoading: boolean;
  setLoading: (loading: boolean) => void;

  // 全局错误
  globalError: string | null;
  setGlobalError: (error: string | null) => void;

  // 性能优化：批量操作
  batchUpdate: (updates: Partial<Omit<AppState, 'batchUpdate'>>) => void;
}

export const useAppStore = create<AppState>()(
  persist(
    (set, get) => ({
      // 初始状态
      currentPage: 'dashboard',
      notifications: [],
      theme: 'system',
      sidebarOpen: true,
      isLoading: false,
      globalError: null,

      // 页面导航
      setCurrentPage: (page) => set({ currentPage: page }),

      // 通知管理 - 优化：限制数量，使用 Set 去重
      addNotification: (notification) => {
        const id = Math.random().toString(36).substring(7);
        const newNotification: Notification = {
          ...notification,
          id,
          timestamp: Date.now(),
        };

        set((state) => {
          // 限制最多 5 个通知
          const notifications = [...state.notifications.slice(-4), newNotification];
          return { notifications };
        });

        // 自动移除通知
        setTimeout(() => {
          get().removeNotification(id);
        }, 5000);
      },

      removeNotification: (id) =>
        set((state) => ({
          notifications: state.notifications.filter((n) => n.id !== id),
        })),

      clearNotifications: () => set({ notifications: [] }),

      // 主题
      setTheme: (theme) => set({ theme }),

      // 侧边栏
      setSidebarOpen: (open) => set({ sidebarOpen: open }),
      toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),

      // 加载状态
      setLoading: (loading) => set({ isLoading: loading }),

      // 全局错误
      setGlobalError: (error) => set({ globalError: error }),

      // 批量更新 - 性能优化
      batchUpdate: (updates) => set(updates),
    }),
    {
      name: 'app-storage',
      storage: createJSONStorage(() => createCompressedStorage()),
      partialize: (state) => ({
        // 只持久化必要的字段
        theme: state.theme,
        sidebarOpen: state.sidebarOpen,
        currentPage: state.currentPage,
      }),
      // 版本控制，用于数据迁移
      version: 1,
      migrate: (persistedState: unknown, version: number) => {
        if (version === 0) {
          // 从版本 0 迁移到版本 1
          return persistedState as Record<string, unknown>;
        }
        return persistedState as Record<string, unknown>;
      },
    }
  )
);

// 选择器 - 优化订阅性能
export const selectTheme = (state: AppState) => state.theme;
export const selectSidebarOpen = (state: AppState) => state.sidebarOpen;
export const selectNotifications = (state: AppState) => state.notifications;
export const selectIsLoading = (state: AppState) => state.isLoading;
export const selectGlobalError = (state: AppState) => state.globalError;
