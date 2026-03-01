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
  ModelConfig,
  AgentConfig,
  ServiceInfo,
  DiagnosticResult,
  DiagnosticIssue,
} from '@/types';

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
  getAll: () => invoke<ApiResponse<Plugin[]>>('get_plugins'),
  install: (marketItemId: string, downloadUrl: string) =>
    invoke<ApiResponse<Plugin>>('install_plugin', {
      req: { market_item_id: marketItemId, download_url: downloadUrl },
    }),
  uninstall: (id: string) =>
    invoke<ApiResponse<boolean>>('uninstall_plugin', { id }),
  enable: (id: string) =>
    invoke<ApiResponse<Plugin>>('enable_plugin', { id }),
  disable: (id: string) =>
    invoke<ApiResponse<Plugin>>('disable_plugin', { id }),
};

// ==================== System API ====================
export interface SystemCheckResult {
  name: string;
  passed: boolean;
  required: boolean;
  message: string;
}

export const systemApi = {
  getSystemInfo: () =>
    invoke<ApiResponse<SystemInfo>>('get_system_info'),
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
    invoke<ApiResponse<InstallStatus>>('check_openclaw_installation'),

  install: (version?: string, networkPreference?: string, offlinePackagePath?: string) =>
    invoke<ApiResponse<InstallResult>>('install_openclaw', {
      version,
      networkPreference,
      offlinePackagePath,
    }),

  installOffline: () =>
    invoke<ApiResponse<InstallResult>>('install_openclaw_offline'),

  /// 一键安装（Molili 风格全栈打包）
  /// 包含嵌入式 Runtime + OpenClaw + 国产模型预设
  installOneClick: (useOfflinePackage: boolean = true) =>
    invoke<ApiResponse<InstallResult>>('install_openclaw_one_click', {
      useOfflinePackage,
    }),

  getInstallMethods: () =>
    invoke<ApiResponse<InstallMethodInfo[]>>('get_install_methods'),

  uninstall: () => invoke<ApiResponse<boolean>>('uninstall_openclaw'),

  getConfig: () => invoke<ApiResponse<OpenClawConfig>>('get_openclaw_config'),

  updateConfig: (config: OpenClawConfig) =>
    invoke<ApiResponse<OpenClawConfig>>('update_openclaw_config', { config }),

  checkSystemEnvironment: () =>
    invoke<ApiResponse<SystemEnvironmentCheckResult>>('check_system_environment'),

  executeCommand: (command: string, args?: string[]) =>
    invoke<ApiResponse<string>>('execute_openclaw_command', { command, args }),

  onInstallProgress: (callback: (progress: InstallProgress) => void): Promise<UnlistenFn> => {
    return listen<InstallProgress>('install-progress', (event) => {
      callback(event.payload);
    });
  },
};

// ==================== Service API (新增) ====================
export const serviceApi = {
  startService: () =>
    invoke<ApiResponse<boolean>>('start_service'),

  stopService: () =>
    invoke<ApiResponse<boolean>>('stop_service'),

  getServiceStatus: () =>
    invoke<ApiResponse<ServiceInfo>>('get_service_status'),

  healthCheck: () =>
    invoke<ApiResponse<{ healthy: boolean; message?: string }>>('health_check'),
};

// ==================== Secure Storage API (新增) ====================
export const secureStorageApi = {
  saveApiKey: (provider: string, apiKey: string) =>
    invoke<ApiResponse<void>>('save_model_api_key', { provider, apiKey }),

  getApiKey: (provider: string) =>
    invoke<ApiResponse<string | null>>('get_model_api_key', { provider }),

  deleteApiKey: (provider: string) =>
    invoke<ApiResponse<void>>('delete_model_api_key', { provider }),

  hasApiKey: (provider: string) =>
    invoke<ApiResponse<boolean>>('has_model_api_key', { provider }),
};

// ==================== Model API (新增) ====================
export const modelApi = {
  getAllModels: () =>
    invoke<ApiResponse<ModelConfig[]>>('get_all_models'),

  saveModel: (model: ModelConfig) =>
    invoke<ApiResponse<ModelConfig>>('save_model', { model }),

  deleteModel: (id: string) =>
    invoke<ApiResponse<boolean>>('delete_model', { id }),

  setDefaultModel: (id: string) =>
    invoke<ApiResponse<boolean>>('set_default_model', { id }),

  testModelConnection: (modelId: string) =>
    invoke<ApiResponse<{ success: boolean; latency: number; message?: string }>>('test_model_connection', { modelId }),

  reorderModels: (modelIds: string[]) =>
    invoke<ApiResponse<boolean>>('reorder_models', { modelIds }),
};

// ==================== Agent API (新增) ====================
export const agentApi = {
  getAllAgents: () =>
    invoke<ApiResponse<AgentConfig[]>>('get_all_agents'),

  saveAgent: (agent: AgentConfig) =>
    invoke<ApiResponse<AgentConfig>>('save_agent', { agent }),

  deleteAgent: (id: string) =>
    invoke<ApiResponse<boolean>>('delete_agent', { id }),

  setCurrentAgent: (id: string) =>
    invoke<ApiResponse<boolean>>('set_current_agent', { id }),

  getCurrentAgent: () =>
    invoke<ApiResponse<AgentConfig | null>>('get_current_agent'),
};

// ==================== Diagnostics API (新增) ====================
export const diagnosticsApi = {
  runDiagnostics: () =>
    invoke<ApiResponse<DiagnosticResult>>('run_diagnostics'),

  autoFix: (issueIds: string[]) =>
    invoke<ApiResponse<{ fixed: string[]; failed: { id: string; error: string }[] }>>('auto_fix_issues', { issueIds }),

  fixIssue: (issue: DiagnosticIssue) =>
    invoke<ApiResponse<boolean>>('fix_issue', { issue }),
};

// ==================== File API (废弃但保留向后兼容) ====================
export interface FileItem {
  id: string;
  file_name: string;
  file_path: string;
  file_type: string;
  file_size: number;
  description?: string;
  tags?: string;
  is_collected: boolean;
  is_classified: boolean;
  classification?: string;
  custom_attributes?: string;
  created_at: string;
  updated_at: string;
}

export interface FileScanRequest {
  path: string;
  recursive: boolean;
  file_types?: string[];
}

export interface FileScanResult {
  files: FileItem[];
  total_count: number;
  total_size: number;
}

export const fileApi = {
  scan: (req: FileScanRequest) =>
    invoke<ApiResponse<FileScanResult>>('scan_files', { req }),
  getAll: (params?: {
    file_type?: string;
    is_collected?: boolean;
    is_classified?: boolean;
    limit?: number;
    offset?: number;
  }) => invoke<ApiResponse<FileItem[]>>('get_files', params || {}),
  getById: (id: string) =>
    invoke<ApiResponse<FileItem | null>>('get_file_by_id', { id }),
  update: (id: string, data: Partial<FileItem>) =>
    invoke<ApiResponse<FileItem>>('update_file', {
      req: { id, ...data },
    }),
  delete: (id: string) =>
    invoke<ApiResponse<boolean>>('delete_file', { id }),
  parse: (fileName: string) =>
    invoke<ApiResponse<{ file_name: string; parsed_data: unknown }>>('parse_file_info', {
      fileName,
    }),
};

// ==================== Group API (废弃但保留向后兼容) ====================
export interface Group {
  id: string;
  name: string;
  description?: string;
  icon?: string;
  color?: string;
  sort_order: number;
  is_default: boolean;
  created_at: string;
  updated_at: string;
}

export interface GroupWithFiles extends Group {
  files: FileItem[];
  file_count: number;
}

export const groupApi = {
  getAll: (withFiles?: boolean) =>
    invoke<ApiResponse<GroupWithFiles[]>>('get_groups', { withFiles }),
  create: (name: string, description?: string, icon?: string, color?: string) =>
    invoke<ApiResponse<Group>>('create_group', {
      req: { name, description, icon, color },
    }),
  update: (id: string, data: Partial<Group>) =>
    invoke<ApiResponse<Group>>('update_group', {
      req: { id, ...data },
    }),
  delete: (id: string) =>
    invoke<ApiResponse<boolean>>('delete_group', { id }),
  addFile: (groupId: string, fileId: string) =>
    invoke<ApiResponse<boolean>>('add_file_to_group', {
      req: { group_id: groupId, file_id: fileId },
    }),
  removeFile: (groupId: string, fileId: string) =>
    invoke<ApiResponse<boolean>>('remove_file_from_group', {
      groupId,
      fileId,
    }),
};
