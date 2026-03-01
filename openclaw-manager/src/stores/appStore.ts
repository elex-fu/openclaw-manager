import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { AppPage, Notification, Theme } from '@/types';

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

      // 通知管理
      addNotification: (notification) => {
        const id = Math.random().toString(36).substring(7);
        const newNotification: Notification = {
          ...notification,
          id,
          timestamp: Date.now(),
        };
        set((state) => ({
          notifications: [...state.notifications.slice(-4), newNotification],
        }));

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
    }),
    {
      name: 'app-storage',
      partialize: (state) => ({
        theme: state.theme,
        sidebarOpen: state.sidebarOpen,
      }),
    }
  )
);
