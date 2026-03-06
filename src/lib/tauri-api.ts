import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  ApiResponse,
  InstallStatus,
  InstallResult,
  InstallProgress,
  OpenClawConfig,
  SystemEnvironmentCheckResult,
  SystemInfo,
  Plugin,
  MarketPlugin,
  PluginCategory,
  SearchPluginsResult,
  ModelConfig,
  AgentConfig,
  ServiceInfo,
  DiagnosticResult,
  FixResult,
  LogEntry,
  LogFilter,
  LogLevel,
  LogSourceInfo,
  Skill,
  InstalledSkill,
  SkillCategory,
  SkillMarketItem,
  SkillSearchResult,
} from '@/types';

// ==================== Error Handling ====================

export class ApiError extends Error {
  constructor(
    message: string,
    public code?: string,
    public originalError?: unknown,
    public isRetryable?: boolean
  ) {
    super(message);
    this.name = 'ApiError';
    // 默认网络错误和超时错误是可重试的
    this.isRetryable = isRetryable ?? (
      message.includes('network') ||
      message.includes('timeout') ||
      message.includes('ECONNREFUSED') ||
      message.includes('ETIMEDOUT')
    );
  }
}

export class AbortError extends Error {
  constructor(message = 'Request was aborted') {
    super(message);
    this.name = 'AbortError';
  }
}

// 延迟函数
function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// 判断错误是否可重试
function isRetryableError(error: unknown): boolean {
  if (error instanceof ApiError) {
    return error.isRetryable ?? false;
  }
  if (error instanceof Error) {
    const msg = error.message.toLowerCase();
    return (
      msg.includes('network') ||
      msg.includes('timeout') ||
      msg.includes('econnrefused') ||
      msg.includes('etimedout') ||
      msg.includes('temporarily') ||
      msg.includes('unavailable')
    );
  }
  return false;
}

export function handleApiResponse<T>(response: ApiResponse<T>): T {
  if (!response.success) {
    throw new ApiError(response.error || 'Unknown error');
  }
  if (response.data === null) {
    throw new ApiError('Response data is null');
  }
  return response.data;
}

// 带重试机制的调用函数
export async function invokeWithRetry<T>(
  command: string,
  args?: Record<string, unknown>,
  options: {
    maxRetries?: number;
    baseDelay?: number;
    maxDelay?: number;
    timeoutMs?: number;
    signal?: AbortSignal;
  } = {}
): Promise<T> {
  const {
    maxRetries = 3,
    baseDelay = 1000,
    maxDelay = 30000,
    timeoutMs,
    signal,
  } = options;

  let lastError: Error | undefined;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    // 检查是否已取消
    if (signal?.aborted) {
      throw new AbortError();
    }

    try {
      // 如果指定了超时时间，使用超时包装调用
      const response = timeoutMs
        ? await invokeWithTimeout<ApiResponse<T>>(command, args, timeoutMs)
        : await invoke<ApiResponse<T>>(command, args);
      return handleApiResponse(response);
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));

      // 最后一次尝试，直接抛出错误
      if (attempt === maxRetries) {
        break;
      }

      // 检查是否可重试
      if (!isRetryableError(error)) {
        throw error instanceof ApiError
          ? error
          : new ApiError(lastError.message, undefined, lastError, false);
      }

      // 计算指数退避延迟
      const exponentialDelay = Math.min(
        baseDelay * Math.pow(2, attempt),
        maxDelay
      );
      // 添加随机抖动避免惊群效应
      const jitter = Math.random() * 1000;
      const waitTime = exponentialDelay + jitter;

      console.log(`Retry attempt ${attempt + 1}/${maxRetries} for ${command}, waiting ${Math.round(waitTime)}ms`);

      // 等待后重试
      await delay(waitTime);
    }
  }

  throw lastError instanceof ApiError
    ? lastError
    : new ApiError(
        lastError?.message || 'Max retries exceeded',
        undefined,
        lastError
      );
}

// 带超时控制的调用函数（直接调用invoke，不经过retry）
export async function invokeWithTimeout<T>(
  command: string,
  args?: Record<string, unknown>,
  timeoutMs: number = 30000
): Promise<T> {
  const timeoutPromise = new Promise<never>((_, reject) => {
    setTimeout(() => {
      reject(new ApiError(`Request timeout after ${timeoutMs}ms`, 'TIMEOUT', undefined, true));
    }, timeoutMs);
  });

  const invokePromise = invoke<T>(command, args);

  return Promise.race([invokePromise, timeoutPromise]);
}

// 原始调用函数（保持向后兼容）
export async function invokeWithErrorHandling<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  return invokeWithRetry<T>(command, args, { maxRetries: 0 });
}

// ==================== Config API ====================
export const configApi = {
  get: (key: string) =>
    invoke<ApiResponse<{ key: string; value: string } | null>>('get_config', { key }),
  set: (key: string, value: string, description?: string) =>
    invoke<ApiResponse<{ key: string; value: string }>>('set_config', {
      req: { key, value, description },
    }),
  delete: (key: string) =>
    invoke<ApiResponse<boolean>>('delete_config', { key }),
};

// ==================== Plugin API ====================
export const pluginApi = {
  getAll: () => invokeWithRetry<Plugin[]>('get_plugins', undefined, { maxRetries: 2 }),
  install: (marketItemId: string, downloadUrl: string) =>
    invokeWithRetry<Plugin>('install_plugin', {
      req: { market_item_id: marketItemId, download_url: downloadUrl },
    }, { maxRetries: 2 }),
  uninstall: (id: string) =>
    invokeWithRetry<boolean>('uninstall_plugin', { id }, { maxRetries: 2 }),
  enable: (id: string) =>
    invokeWithRetry<Plugin>('enable_plugin', { id }, { maxRetries: 2 }),
  disable: (id: string) =>
    invokeWithRetry<Plugin>('disable_plugin', { id }, { maxRetries: 2 }),
  getConfig: (pluginId: string) =>
    invokeWithRetry<{ plugin_id: string; config: Record<string, unknown> } | null>('get_plugin_config', { pluginId }, { maxRetries: 2 }),
  updateConfig: (pluginId: string, config: Record<string, unknown>) =>
    invokeWithRetry<{ plugin_id: string; config: Record<string, unknown> }>('update_plugin_config', {
      req: { plugin_id: pluginId, config },
    }, { maxRetries: 2 }),
  getEnabled: () =>
    invokeWithRetry<Plugin[]>('get_enabled_plugins', undefined, { maxRetries: 2 }),
  checkInstalled: (pluginId: string) =>
    invokeWithRetry<boolean>('check_plugin_installed', { pluginId }, { maxRetries: 2 }),
};

// ==================== Plugin Market API ====================
export const pluginMarketApi = {
  search: (params?: {
    query?: string;
    category?: string;
    sortBy?: 'relevance' | 'downloads' | 'rating' | 'created_at' | 'updated_at';
    page?: number;
    perPage?: number;
  }) =>
    invokeWithRetry<SearchPluginsResult>('search_market_plugins', {
      req: {
        query: params?.query,
        category: params?.category,
        sort_by: params?.sortBy,
        page: params?.page,
        per_page: params?.perPage,
      },
    }, { maxRetries: 2 }),
  getDetails: (pluginId: string) =>
    invokeWithRetry<MarketPlugin>('get_market_plugin_details', { pluginId }, { maxRetries: 2 }),
  getCategories: () =>
    invokeWithRetry<PluginCategory[]>('get_plugin_categories', undefined, { maxRetries: 2 }),
  getPopular: (limit?: number) =>
    invokeWithRetry<MarketPlugin[]>('get_popular_plugins', { limit }, { maxRetries: 2 }),
  getLatest: (limit?: number) =>
    invokeWithRetry<MarketPlugin[]>('get_latest_plugins', { limit }, { maxRetries: 2 }),
};

// ==================== System API ====================
export interface SystemCheckResult {
  name: string;
  passed: boolean;
  required: boolean;
  message: string;
}

export interface SystemResources {
  cpu: {
    usage: number;
    cores: number;
    name: string;
    frequency: number;
  };
  memory: {
    used: number;
    total: number;
    usage: number;
    available: number;
  };
  disk: {
    used: number;
    total: number;
    usage: number;
    free: number;
  };
  timestamp: number;
}

export interface Activity {
  id: string;
  timestamp: number;
  activity_type: 'install' | 'config' | 'service' | 'error';
  message: string;
  details?: string;
}

export interface DiagnosticAlert {
  id: string;
  severity: 'info' | 'warning' | 'error';
  title: string;
  message: string;
  fixable: boolean;
  category: string;
}

export const systemApi = {
  getSystemInfo: () =>
    invokeWithRetry<SystemInfo>('get_system_info', undefined, { maxRetries: 2 }),
  getSystemResources: () =>
    invokeWithRetry<SystemResources>('get_system_resources', undefined, { maxRetries: 2 }),
  getRecentActivities: (limit?: number) =>
    invokeWithRetry<Activity[]>('get_recent_activities', { limit }, { maxRetries: 2 }),
  getDiagnosticAlerts: () =>
    invokeWithRetry<DiagnosticAlert[]>('get_diagnostic_alerts', undefined, { maxRetries: 2 }),
};

// ==================== OpenClaw Install API ====================
export interface InstallMethodInfo {
  id: string;
  name: string;
  description: string;
  requires_network: boolean;
  available: boolean;
}

export const openclawApi = {
  checkInstallation: () =>
    invokeWithRetry<InstallStatus>('check_openclaw_installation', undefined, { maxRetries: 2 }),

  install: (version?: string, networkPreference?: string, offlinePackagePath?: string) =>
    invokeWithRetry<InstallResult>('install_openclaw', {
      version,
      networkPreference,
      offlinePackagePath,
    }, { maxRetries: 1 }),

  installOffline: () =>
    invokeWithRetry<InstallResult>('install_openclaw_offline', undefined, { maxRetries: 1 }),

  /// 一键安装（Molili 风格全栈打包）
  /// 包含嵌入式 Runtime + OpenClaw + 国产模型预设
  installOneClick: (useOfflinePackage: boolean = true) =>
    invokeWithRetry<InstallResult>('install_openclaw_one_click', {
      useOfflinePackage,
    }, { maxRetries: 1 }),

  getInstallMethods: () =>
    invokeWithRetry<InstallMethodInfo[]>('get_install_methods', undefined, { maxRetries: 2 }),

  uninstall: () => invokeWithRetry<boolean>('uninstall_openclaw', undefined, { maxRetries: 2 }),

  getConfig: () => invokeWithRetry<OpenClawConfig>('get_openclaw_config', undefined, { maxRetries: 2 }),

  updateConfig: (config: OpenClawConfig) =>
    invokeWithRetry<OpenClawConfig>('update_openclaw_config', { config }, { maxRetries: 2 }),

  checkSystemEnvironment: () =>
    invokeWithRetry<SystemEnvironmentCheckResult>('check_system_environment', undefined, { maxRetries: 2 }),

  executeCommand: (command: string, args?: string[]) =>
    invokeWithRetry<string>('execute_openclaw_command', { command, args }, { maxRetries: 2 }),

  onInstallProgress: (callback: (progress: InstallProgress) => void): Promise<UnlistenFn> => {
    return listen<InstallProgress>('install-progress', (event) => {
      callback(event.payload);
    });
  },
};

// ==================== Service API (OpenClaw服务控制) ====================
export const serviceApi = {
  startService: () =>
    invokeWithRetry<boolean>('start_openclaw_service', undefined, { maxRetries: 2 }),

  stopService: () =>
    invokeWithRetry<boolean>('stop_openclaw_service', undefined, { maxRetries: 2 }),

  getServiceStatus: () =>
    invokeWithRetry<ServiceInfo>('get_openclaw_service_status', undefined, { maxRetries: 2 }),

  healthCheck: () =>
    invokeWithRetry<{ healthy: boolean; message?: string }>('health_check_openclaw_service', undefined, { maxRetries: 1 }),
};

// ==================== Secure Storage API (新增) ====================
export const secureStorageApi = {
  saveApiKey: (provider: string, apiKey: string) =>
    invokeWithRetry<void>('save_api_key', { provider, apiKey }, { maxRetries: 2 }),

  getApiKey: (provider: string) =>
    invokeWithRetry<string | null>('get_api_key', { provider }, { maxRetries: 2 }),

  deleteApiKey: (provider: string) =>
    invokeWithRetry<void>('delete_api_key', { provider }, { maxRetries: 2 }),

  hasApiKey: (provider: string) =>
    invokeWithRetry<boolean>('has_api_key', { provider }, { maxRetries: 2 }),
};

// ==================== Model API (新增) ====================
export const modelApi = {
  getAllModels: () =>
    invokeWithRetry<ModelConfig[]>('get_all_models_full', undefined, { maxRetries: 2 }),

  saveModel: (model: ModelConfig) =>
    invokeWithRetry<ModelConfig>('save_model_full', { model }, { maxRetries: 2 }),

  deleteModel: (id: string) =>
    invokeWithRetry<boolean>('delete_model', { id }, { maxRetries: 2 }),

  setDefaultModel: (id: string) =>
    invokeWithRetry<boolean>('set_default_model', { id }, { maxRetries: 2 }),

  testModelConnection: (modelId: string) =>
    invokeWithRetry<{ success: boolean; latency: number; message?: string }>('test_model_connection', { modelId }, {
      maxRetries: 1,
      timeoutMs: 30000, // 30秒超时用于连接测试
    }),

  reorderModels: (modelIds: string[]) =>
    invokeWithRetry<boolean>('reorder_models', { modelIds }, { maxRetries: 2 }),
};

// ==================== Agent API (新增) ====================
export const agentApi = {
  getAllAgents: () =>
    invokeWithRetry<AgentConfig[]>('get_all_agents', undefined, { maxRetries: 2 }),

  saveAgent: (agent: AgentConfig) =>
    invokeWithRetry<AgentConfig>('save_agent', { agent }, { maxRetries: 2 }),

  deleteAgent: (id: string) =>
    invokeWithRetry<boolean>('delete_agent', { id }, { maxRetries: 2 }),

  setCurrentAgent: (id: string) =>
    invokeWithRetry<boolean>('set_current_agent', { id }, { maxRetries: 2 }),

  getCurrentAgent: () =>
    invokeWithRetry<AgentConfig | null>('get_current_agent', undefined, { maxRetries: 2 }),
};

// ==================== Diagnostics API (新增) ====================
export const diagnosticsApi = {
  runDiagnostics: () =>
    invokeWithRetry<DiagnosticResult>('run_diagnostics', undefined, { maxRetries: 1 }),

  autoFix: (issueNames: string[]) =>
    invokeWithRetry<FixResult>('auto_fix_issues', { issueIds: issueNames }, { maxRetries: 1 }),

  fixIssue: (issueName: string) =>
    invokeWithRetry<boolean>('fix_issue', { issueName }, { maxRetries: 1 }),
};

// ==================== Log API ====================
export interface LogStats {
  total_size: number;
  source_count: number;
  sources: Array<{
    source: string;
    size: number;
    path: string;
  }>;
}

export interface SubscribeLogsResponse {
  subscription_id: string;
}

export const logApi = {
  /** 获取日志源列表 */
  getLogSources: () =>
    invokeWithRetry<LogSourceInfo[]>('get_log_sources', undefined, { maxRetries: 2 }),

  /** 获取最近日志 */
  getRecentLogs: (params: {
    limit?: number;
    levels?: LogLevel[];
    sources?: string[];
    searchQuery?: string;
  }) =>
    invokeWithRetry<LogEntry[]>('get_recent_logs', { req: params }, { maxRetries: 2 }),

  /** 订阅实时日志 */
  subscribeLogs: (params: {
    levels: LogLevel[];
    sources?: string[];
    searchQuery?: string;
  }) =>
    invokeWithRetry<SubscribeLogsResponse>('subscribe_logs', { req: params }, { maxRetries: 2 }),

  /** 取消订阅 */
  unsubscribeLogs: (subscriptionId: string) =>
    invokeWithRetry<boolean>('unsubscribe_logs', { subscriptionId }, { maxRetries: 2 }),

  /** 导出日志 */
  exportLogs: (params: {
    filter: LogFilter;
    format: 'text' | 'json' | 'csv';
    outputPath: string;
  }) =>
    invokeWithRetry<string>('export_logs', { req: params }, { maxRetries: 2 }),

  /** 添加日志源 */
  addLogSource: (path: string, source: string) =>
    invokeWithRetry<boolean>('add_log_source', { path, source }, { maxRetries: 2 }),

  /** 移除日志源 */
  removeLogSource: (sourceId: string) =>
    invokeWithRetry<boolean>('remove_log_source', { sourceId }, { maxRetries: 2 }),

  /** 初始化默认日志源 */
  initDefaultLogSources: () =>
    invokeWithRetry<boolean>('init_default_log_sources', undefined, { maxRetries: 2 }),

  /** 清空日志显示 */
  clearLogDisplay: () =>
    invokeWithRetry<boolean>('clear_log_display', undefined, { maxRetries: 2 }),

  /** 获取日志统计 */
  getLogStats: () =>
    invokeWithRetry<LogStats>('get_log_stats', undefined, { maxRetries: 2 }),

  /** 监听日志条目 */
  onLogEntry: (subscriptionId: string, callback: (entry: LogEntry) => void): Promise<UnlistenFn> => {
    return listen<LogEntry>(`log-entry-${subscriptionId}`, (event) => {
      callback(event.payload);
    });
  },

  /** 监听日志重置 */
  onLogReset: (subscriptionId: string, callback: (source: string) => void): Promise<UnlistenFn> => {
    return listen<string>(`log-reset-${subscriptionId}`, (event) => {
      callback(event.payload);
    });
  },

  /** 监听日志错误 */
  onLogError: (subscriptionId: string, callback: (error: { source: string; error: string }) => void): Promise<UnlistenFn> => {
    return listen<{ source: string; error: string }>(`log-error-${subscriptionId}`, (event) => {
      callback(event.payload);
    });
  },
};

// ==================== Update API ====================
export interface UpdateState {
  currentVersion: string | null;
  latestVersion: string | null;
  hasUpdate: boolean;
  updateInfo: UpdateInfo | null;
}

export interface UpdateInfo {
  version: string;
  releaseDate: string;
  changelog: string;
  downloadUrl: string;
  checksum: string;
  mandatory: boolean;
  minSupportedVersion: string | null;
}

export interface UpdateProgress {
  stage: 'Checking' | 'Downloading' | 'BackingUp' | 'Installing' | 'Migrating' | 'CleaningUp' | 'Complete' | 'Error' | 'Rollback';
  percentage: number;
  message: string;
  canCancel: boolean;
}

export interface BackupMetadata {
  createdAt: string;
  version: string | null;
  path: string;
}

export const updateApi = {
  /** 检查更新 */
  checkForUpdates: () =>
    invokeWithRetry<UpdateState>('check_for_updates', undefined, { maxRetries: 2 }),

  /** 执行升级 */
  performUpdate: (updateInfo: UpdateInfo) =>
    invokeWithRetry<InstallResult>('perform_update', { updateInfo }, { maxRetries: 1 }),

  /** 离线升级 */
  performOfflineUpdate: (packagePath: string) =>
    invokeWithRetry<InstallResult>('perform_offline_update', { packagePath }, { maxRetries: 1 }),

  /** 获取备份列表 */
  getBackupList: () =>
    invokeWithRetry<BackupMetadata[]>('get_backup_list', undefined, { maxRetries: 2 }),

  /** 从备份恢复 */
  restoreFromBackup: (backupPath: string) =>
    invokeWithRetry<void>('restore_from_backup', { backupPath }, { maxRetries: 1 }),

  /** 监听升级进度 */
  onUpdateProgress: (callback: (progress: UpdateProgress) => void) => {
    return listen<UpdateProgress>('update-progress', (event) => {
      callback(event.payload);
    });
  },
};

// ==================== Skill API ====================
export const skillApi = {
  // 已安装技能管理
  getAll: () =>
    invokeWithRetry<InstalledSkill[]>('get_skills', undefined, { maxRetries: 2 }),

  getById: (skillId: string) =>
    invokeWithRetry<InstalledSkill | null>('get_skill', { skillId }, { maxRetries: 2 }),

  searchInstalled: (query: string) =>
    invokeWithRetry<InstalledSkill[]>('search_installed_skills', { query }, { maxRetries: 2 }),

  install: (skillId: string) =>
    invokeWithRetry<InstalledSkill>('install_skill', { skillId }, { maxRetries: 1 }),

  uninstall: (skillId: string) =>
    invokeWithRetry<boolean>('uninstall_skill', { skillId }, { maxRetries: 2 }),

  enable: (skillId: string) =>
    invokeWithRetry<InstalledSkill>('enable_skill', { skillId }, { maxRetries: 2 }),

  disable: (skillId: string) =>
    invokeWithRetry<InstalledSkill>('disable_skill', { skillId }, { maxRetries: 2 }),

  toggle: (skillId: string, enabled: boolean) =>
    invokeWithRetry<InstalledSkill>('toggle_skill', {
      request: { skill_id: skillId, enabled },
    }, { maxRetries: 2 }),

  getConfig: (skillId: string) =>
    invokeWithRetry<Record<string, unknown>>('get_skill_config', { skillId }, { maxRetries: 2 }),

  updateConfig: (skillId: string, config: Record<string, unknown>) =>
    invokeWithRetry<InstalledSkill>('update_skill_config', {
      request: { skill_id: skillId, config },
    }, { maxRetries: 2 }),

  update: (skillId: string) =>
    invokeWithRetry<InstalledSkill>('update_skill', { skillId }, { maxRetries: 1 }),

  checkUpdates: () =>
    invokeWithRetry<Array<[string, string]>>('check_skill_updates', undefined, { maxRetries: 2 }),

  // 技能市场
  searchMarket: (params?: {
    query?: string;
    category?: string;
    page?: number;
    perPage?: number;
  }) =>
    invokeWithRetry<SkillSearchResult>('search_skills', {
      query: params?.query,
      category: params?.category,
      page: params?.page ?? 1,
      per_page: params?.perPage ?? 20,
    }, { maxRetries: 2 }),

  getMarketDetail: (skillId: string) =>
    invokeWithRetry<Skill>('get_market_skill_detail', { skillId }, { maxRetries: 2 }),

  getPopular: (limit?: number) =>
    invokeWithRetry<SkillMarketItem[]>('get_popular_skills', { limit: limit ?? 10 }, { maxRetries: 2 }),

  getLatest: (limit?: number) =>
    invokeWithRetry<SkillMarketItem[]>('get_latest_skills', { limit: limit ?? 10 }, { maxRetries: 2 }),

  getCategories: () =>
    invokeWithRetry<SkillCategory[]>('get_skill_categories', undefined, { maxRetries: 2 }),

  checkSingleUpdate: (skillId: string, currentVersion: string) =>
    invokeWithRetry<string | null>('check_single_skill_update', {
      skillId,
      currentVersion,
    }, { maxRetries: 2 }),
};

// ==================== Network Status ====================
// 网络状态管理
type NetworkStatus = 'online' | 'offline' | 'unknown';

class NetworkManager {
  private status: NetworkStatus = 'unknown';
  private listeners: Set<(status: NetworkStatus) => void> = new Set();

  constructor() {
    // 监听网络状态变化
    if (typeof window !== 'undefined' && 'ononline' in window) {
      window.addEventListener('online', () => this.setStatus('online'));
      window.addEventListener('offline', () => this.setStatus('offline'));
      this.status = navigator.onLine ? 'online' : 'offline';
    }
  }

  private setStatus(status: NetworkStatus) {
    if (this.status !== status) {
      this.status = status;
      this.listeners.forEach(listener => listener(status));
    }
  }

  getStatus(): NetworkStatus {
    return this.status;
  }

  isOnline(): boolean {
    return this.status === 'online';
  }

  subscribe(listener: (status: NetworkStatus) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }
}

export const networkManager = new NetworkManager();
