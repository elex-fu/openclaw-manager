import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { LogEntry, LogLevel, LogSourceInfo } from '@/types';

interface LogState {
  // 日志条目列表
  logEntries: LogEntry[];
  setLogEntries: (entries: LogEntry[]) => void;
  addLogEntry: (entry: LogEntry) => void;
  addLogEntries: (entries: LogEntry[]) => void;
  clearLogEntries: () => void;

  // 日志源
  logSources: LogSourceInfo[];
  setLogSources: (sources: LogSourceInfo[]) => void;
  addLogSource: (source: LogSourceInfo) => void;
  removeLogSource: (sourceId: string) => void;

  // 筛选设置
  filter: {
    levels: LogLevel[];
    sources: string[];
    searchQuery: string;
    startTime: number | null;
    endTime: number | null;
  };
  setFilter: (filter: Partial<LogState['filter']>) => void;
  toggleLevel: (level: LogLevel) => void;
  toggleSource: (source: string) => void;
  setSearchQuery: (query: string) => void;
  resetFilter: () => void;

  // 实时订阅状态
  isSubscribed: boolean;
  subscriptionId: string | null;
  setIsSubscribed: (subscribed: boolean) => void;
  setSubscriptionId: (id: string | null) => void;

  // 自动滚动
  autoScroll: boolean;
  setAutoScroll: (enabled: boolean) => void;

  // 选中的日志条目
  selectedLogId: string | null;
  setSelectedLogId: (id: string | null) => void;

  // 日志统计
  stats: {
    totalCount: number;
    errorCount: number;
    warnCount: number;
    infoCount: number;
    debugCount: number;
  };
  updateStats: (entries: LogEntry[]) => void;

  // 加载状态
  isLoading: boolean;
  setIsLoading: (loading: boolean) => void;
  isExporting: boolean;
  setIsExporting: (exporting: boolean) => void;

  // 错误状态
  error: string | null;
  setError: (error: string | null) => void;

  // 最大显示条目数（性能优化）
  maxEntries: number;
  setMaxEntries: (count: number) => void;

  // 获取筛选后的日志
  getFilteredLogs: () => LogEntry[];

  // 重置
  reset: () => void;
}

const defaultLevels: LogLevel[] = ['ERROR', 'WARN', 'INFO'];
const defaultMaxEntries = 10000;

export const useLogStore = create<LogState>()(
  persist(
    (set, get) => ({
      // 初始状态
      logEntries: [],
      logSources: [],
      filter: {
        levels: defaultLevels,
        sources: [],
        searchQuery: '',
        startTime: null,
        endTime: null,
      },
      isSubscribed: false,
      subscriptionId: null,
      autoScroll: true,
      selectedLogId: null,
      stats: {
        totalCount: 0,
        errorCount: 0,
        warnCount: 0,
        infoCount: 0,
        debugCount: 0,
      },
      isLoading: false,
      isExporting: false,
      error: null,
      maxEntries: defaultMaxEntries,

      // 日志条目管理
      setLogEntries: (entries) => {
        set({ logEntries: entries });
        get().updateStats(entries);
      },
      addLogEntry: (entry) =>
        set((state) => {
          const newEntries = [...state.logEntries, entry];
          // 限制最大条目数
          if (newEntries.length > state.maxEntries) {
            newEntries.shift();
          }
          // 更新统计
          const stats = { ...state.stats };
          stats.totalCount = newEntries.length;
          switch (entry.level) {
            case 'ERROR':
              stats.errorCount++;
              break;
            case 'WARN':
              stats.warnCount++;
              break;
            case 'INFO':
              stats.infoCount++;
              break;
            case 'DEBUG':
            case 'TRACE':
              stats.debugCount++;
              break;
          }
          return { logEntries: newEntries, stats };
        }),
      addLogEntries: (entries) =>
        set((state) => {
          const newEntries = [...state.logEntries, ...entries];
          // 限制最大条目数
          if (newEntries.length > state.maxEntries) {
            newEntries.splice(0, newEntries.length - state.maxEntries);
          }
          return { logEntries: newEntries };
        }),
      clearLogEntries: () =>
        set({
          logEntries: [],
          stats: {
            totalCount: 0,
            errorCount: 0,
            warnCount: 0,
            infoCount: 0,
            debugCount: 0,
          },
        }),

      // 日志源管理
      setLogSources: (sources) => set({ logSources: sources }),
      addLogSource: (source) =>
        set((state) => ({
          logSources: [...state.logSources, source],
        })),
      removeLogSource: (sourceId) =>
        set((state) => ({
          logSources: state.logSources.filter((s) => s.id !== sourceId),
          filter: {
            ...state.filter,
            sources: state.filter.sources.filter((s) => s !== sourceId),
          },
        })),

      // 筛选管理
      setFilter: (filter) =>
        set((state) => ({
          filter: { ...state.filter, ...filter },
        })),
      toggleLevel: (level) =>
        set((state) => {
          const levels = state.filter.levels.includes(level)
            ? state.filter.levels.filter((l) => l !== level)
            : [...state.filter.levels, level];
          return { filter: { ...state.filter, levels } };
        }),
      toggleSource: (source) =>
        set((state) => {
          const sources = state.filter.sources.includes(source)
            ? state.filter.sources.filter((s) => s !== source)
            : [...state.filter.sources, source];
          return { filter: { ...state.filter, sources } };
        }),
      setSearchQuery: (query) =>
        set((state) => ({
          filter: { ...state.filter, searchQuery: query },
        })),
      resetFilter: () =>
        set({
          filter: {
            levels: defaultLevels,
            sources: [],
            searchQuery: '',
            startTime: null,
            endTime: null,
          },
        }),

      // 订阅状态管理
      setIsSubscribed: (isSubscribed) => set({ isSubscribed }),
      setSubscriptionId: (subscriptionId) => set({ subscriptionId }),

      // 自动滚动
      setAutoScroll: (autoScroll) => set({ autoScroll }),

      // 选中日志
      setSelectedLogId: (selectedLogId) => set({ selectedLogId }),

      // 统计更新
      updateStats: (entries) => {
        const stats = {
          totalCount: entries.length,
          errorCount: entries.filter((e) => e.level === 'ERROR').length,
          warnCount: entries.filter((e) => e.level === 'WARN').length,
          infoCount: entries.filter((e) => e.level === 'INFO').length,
          debugCount: entries.filter((e) => e.level === 'DEBUG' || e.level === 'TRACE').length,
        };
        set({ stats });
      },

      // 加载状态
      setIsLoading: (isLoading) => set({ isLoading }),
      setIsExporting: (isExporting) => set({ isExporting }),

      // 错误状态
      setError: (error) => set({ error }),

      // 最大条目数
      setMaxEntries: (maxEntries) => set({ maxEntries }),

      // 获取筛选后的日志
      getFilteredLogs: () => {
        const state = get();
        return state.logEntries.filter((entry) => {
          // 级别筛选
          if (
            state.filter.levels.length > 0 &&
            !state.filter.levels.includes(entry.level)
          ) {
            return false;
          }

          // 来源筛选
          if (
            state.filter.sources.length > 0 &&
            !state.filter.sources.includes(entry.source)
          ) {
            return false;
          }

          // 时间范围筛选
          if (state.filter.startTime && entry.timestamp < state.filter.startTime) {
            return false;
          }
          if (state.filter.endTime && entry.timestamp > state.filter.endTime) {
            return false;
          }

          // 搜索筛选
          if (state.filter.searchQuery) {
            const query = state.filter.searchQuery.toLowerCase();
            const matchesMessage = entry.message.toLowerCase().includes(query);
            const matchesSource = entry.source.toLowerCase().includes(query);
            const matchesMetadata = entry.metadata
              ? JSON.stringify(entry.metadata).toLowerCase().includes(query)
              : false;
            if (!matchesMessage && !matchesSource && !matchesMetadata) {
              return false;
            }
          }

          return true;
        });
      },

      // 重置所有状态
      reset: () =>
        set({
          logEntries: [],
          logSources: [],
          filter: {
            levels: defaultLevels,
            sources: [],
            searchQuery: '',
            startTime: null,
            endTime: null,
          },
          isSubscribed: false,
          subscriptionId: null,
          autoScroll: true,
          selectedLogId: null,
          stats: {
            totalCount: 0,
            errorCount: 0,
            warnCount: 0,
            infoCount: 0,
            debugCount: 0,
          },
          isLoading: false,
          isExporting: false,
          error: null,
          maxEntries: defaultMaxEntries,
        }),
    }),
    {
      name: 'log-storage',
      partialize: (state) => ({
        // 只持久化筛选设置和显示偏好
        filter: {
          levels: state.filter.levels,
          sources: state.filter.sources,
        },
        autoScroll: state.autoScroll,
        maxEntries: state.maxEntries,
      }),
    }
  )
);

// 选择器 - 优化订阅性能
export const selectLogEntries = (state: LogState) => state.logEntries;
export const selectLogSources = (state: LogState) => state.logSources;
export const selectFilter = (state: LogState) => state.filter;
export const selectIsSubscribed = (state: LogState) => state.isSubscribed;
export const selectAutoScroll = (state: LogState) => state.autoScroll;
export const selectStats = (state: LogState) => state.stats;
export const selectIsLoading = (state: LogState) => state.isLoading;
