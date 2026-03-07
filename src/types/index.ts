export interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

// 页面路由类型
export type AppPage = 
  | 'dashboard' 
  | 'install-wizard' 
  | 'model-config' 
  | 'agent-manager' 
  | 'diagnostics' 
  | 'settings';

// 通知类型
export interface Notification {
  id: string;
  title: string;
  message: string;
  type: 'info' | 'success' | 'warning' | 'error';
  timestamp: number;
}

// 主题类型
export type Theme = 'light' | 'dark' | 'system';

// 安装状态
export type InstallStatus =
  | { type: 'NotInstalled' }
  | { type: 'Installing'; stage: string; progress: number }
  | { type: 'Installed'; version: string }
  | { type: 'Error'; message: string };

export interface InstallResult {
  success: boolean;
  version: string | null;
  message: string;
}

export interface InstallProgress {
  stage: InstallStage;
  percentage: number;
  message: string;
}

export type InstallStage = 'Checking' | 'Downloading' | 'Installing' | 'Configuring' | 'Complete' | 'Error';

// 安装方式
export type InstallMethod = 'online' | 'offline' | 'oneclick';

// 安装日志
export interface InstallLogEntry {
  timestamp: string;
  level: 'info' | 'success' | 'error' | 'warning';
  message: string;
}

// 系统检查
export interface SystemCheckResult {
  name: string;
  passed: boolean;
  required: boolean;
  message: string;
}

export interface SystemEnvironmentCheckResult {
  checks: SystemCheckResult[];
  can_install: boolean;
  missing_dependencies: string[];
}

// 系统信息
export interface SystemInfo {
  system_type: 'MacOS' | 'Windows' | 'Linux';
  macos_version?: {
    type: 'Catalina' | 'BigSur' | 'Monterey' | 'Ventura' | 'Sonoma' | 'Sequoia' | 'Unknown';
    version_string: string;
  };
  windows_version?: {
    type: 'Windows10' | 'Windows11' | 'Unknown';
    version_string: string;
  };
  linux_distro?: {
    type: 'Ubuntu' | 'Debian' | 'Fedora' | 'Arch' | 'Unknown';
    name: string;
  };
  architecture: 'X86_64' | 'Aarch64' | 'Unknown';
  kernel_version: string;
  hostname: string;
  friendly_name: string;
  install_script: string;
}

// 服务状态
export type ServiceStatus = 
  | 'stopped' 
  | 'starting' 
  | 'running' 
  | 'stopping' 
  | 'error';

export interface ServiceInfo {
  name: string;
  status: ServiceStatus;
  pid?: number;
  uptime?: number;
  error?: string;
}

// 模型提供商
export interface ModelProvider {
  id: string;
  name: string;
  type: 'openai' | 'anthropic' | 'google' | 'azure' | 'local' | 'custom';
  apiBase?: string;
  description?: string;
  icon?: string;
}

// 模型配置
export interface ModelConfig {
  id: string;
  name: string;
  provider: string;
  api_base?: string;
  model: string;
  temperature: number;
  max_tokens?: number;
  enabled: boolean;
  isDefault?: boolean;
  // api_key 不存储在这里，从 Keychain 获取
}

// 运行时模型配置（包含 API Key）
export interface RuntimeModelConfig extends ModelConfig {
  apiKey?: string;
}

// 模型参数配置
export interface ModelParameters {
  temperature: number;      // 0-2，默认1
  max_tokens: number;       // 1-8192，默认2048
  top_p: number;           // 0-1，默认1
  presence_penalty: number; // -2到2，默认0
  frequency_penalty: number; // -2到2，默认0
}

// 模型能力
export interface ModelCapabilities {
  function_calling: boolean;
  vision: boolean;
  streaming: boolean;
  json_mode: boolean;
  max_context_length?: number;
  custom?: Record<string, boolean>;
}

// 完整模型配置（包含高级参数）
export interface ModelConfigFull {
  id: string;
  name: string;
  provider: string;
  api_base?: string;
  model: string;
  priority: number;
  parameters: ModelParameters;
  capabilities: ModelCapabilities;
  enabled: boolean;
  default: boolean;
}

// 连接测试结果
export interface ConnectionTestResult {
  success: boolean;
  latency: number;
  message?: string;
  model_info?: string;
}

// Agent 配置
export interface AgentConfig {
  id: string;
  name: string;
  description?: string;
  modelId: string;
  systemPrompt?: string;
  skills: string[];
  enabled: boolean;
  avatar?: string;
  createdAt: string;
  updatedAt: string;
}

// 技能配置
export interface SkillConfig {
  id: string;
  name: string;
  description?: string;
  entryPoint: string;
  config?: Record<string, unknown>;
  enabled: boolean;
}

// 诊断检查状态
export type CheckStatus = 'pass' | 'warning' | 'error';

// 诊断检查项
export interface DiagnosticCheck {
  category: 'system' | 'openclaw' | 'network' | 'service';
  name: string;
  status: CheckStatus;
  message: string;
  details?: string;
  fixable: boolean;
  fixSuggestion?: string;
}

// 诊断结果
export interface DiagnosticResult {
  checks: DiagnosticCheck[];
  hasErrors: boolean;
  hasWarnings: boolean;
  checkedAt: string;
}

// 修复失败信息
export interface FixFailure {
  name: string;
  error: string;
}

// 修复结果
export interface FixResult {
  fixed: string[];
  failed: FixFailure[];
}

// 诊断问题类型（保留用于向后兼容）
export type DiagnosticSeverity = 'info' | 'warning' | 'error' | 'critical';

export interface DiagnosticIssue {
  id: string;
  name: string;
  description: string;
  severity: DiagnosticSeverity;
  category: 'environment' | 'service' | 'config' | 'network' | 'permission';
  canAutoFix: boolean;
  fixed?: boolean;
  fixMessage?: string;
  error?: string;
}

// 通知类型筛选
export interface NotificationFilter {
  info: boolean;
  warning: boolean;
  error: boolean;
  success: boolean;
}

// 启动设置
export interface StartupSettings {
  auto_start: boolean;
  minimize_to_tray: boolean;
  check_update_on_start: boolean;
}

// 应用设置
export interface AppSettings {
  theme: Theme;
  language: string;
  startup: StartupSettings;
  notifications: {
    enabled: boolean;
    filter: NotificationFilter;
  };
}

// 系统设置
export interface SystemSettings {
  log_level: 'debug' | 'info' | 'warn' | 'error';
  auto_update: boolean;
  theme: Theme;
  language: string;
  custom_vars: Record<string, string>;
}

// OpenClaw 配置
export interface OpenClawConfig {
  version: string;
  name: string;
  models: ModelConfig[];
  defaultModel?: string;
  agents: AgentConfig[];
  skills: SkillConfig[];
  settings: SystemSettings;
}

// Plugin 类型（保留原有定义）
export interface Plugin {
  id: string;
  name: string;
  version: string;
  description?: string;
  author?: string;
  plugin_type: string;
  entry_point: string;
  is_enabled: boolean;
  config_schema?: string;
  default_config?: string;
  created_at: string;
  updated_at: string;
}

// 市场插件信息
export interface MarketPlugin {
  id: string;
  name: string;
  version: string;
  description?: string;
  author?: string;
  author_avatar?: string;
  downloads: number;
  rating: number;
  rating_count: number;
  icon_url?: string;
  download_url: string;
  categories: string[];
  tags: string[];
  created_at: string;
  updated_at: string;
  size_bytes: number;
  min_app_version?: string;
  changelog?: string;
}

// 插件分类
export interface PluginCategory {
  id: string;
  name: string;
  description?: string;
  icon?: string;
  plugin_count: number;
}

// 插件搜索结果
export interface SearchPluginsResult {
  plugins: MarketPlugin[];
  total: number;
  page: number;
  per_page: number;
  has_more: boolean;
}

// ==================== Log Types ====================

export type LogLevel = 'ERROR' | 'WARN' | 'INFO' | 'DEBUG' | 'TRACE';

export interface LogEntry {
  id: string;
  timestamp: number;
  level: LogLevel;
  source: string;
  message: string;
  metadata?: Record<string, unknown>;
}

export interface LogFilter {
  levels: LogLevel[];
  searchQuery?: string;
  sources?: string[];
  startTime?: number;
  endTime?: number;
}

export interface LogSourceInfo {
  id: string;
  name: string;
  path: string;
  size: number;
  modified: number;
}

export interface LogStats {
  total_size: number;
  source_count: number;
  sources: Array<{
    source: string;
    size: number;
    path: string;
  }>;
}

// ==================== Skill Types ====================

export type HookType = 'pre_process' | 'post_process' | 'command' | 'event' | 'tool';

export interface SkillHook {
  hook_type: HookType;
  trigger: string;
  handler: string;
  description?: string;
  priority: number;
}

export interface Skill {
  id: string;
  name: string;
  description: string;
  author: string;
  version: string;
  categories: string[];
  tags: string[];
  icon_url?: string;
  rating: number;
  downloads: number;
  hooks: SkillHook[];
  config_schema?: Record<string, unknown>;
  default_config?: Record<string, unknown>;
  dependencies: string[];
  created_at: string;
  updated_at: string;
}

export interface InstalledSkill extends Skill {
  is_enabled: boolean;
  config: Record<string, unknown>;
  installed_at: string;
  updated_at: string;
  has_update: boolean;
  latest_version?: string;
}

export interface SkillCategory {
  id: string;
  name: string;
  description?: string;
  icon?: string;
  sort_order: number;
}

export interface SkillSearchResult {
  skills: Skill[];
  total: number;
  page: number;
  per_page: number;
  query?: string;
  category?: string;
}

export interface SkillMarketItem {
  id: string;
  name: string;
  description: string;
  author: string;
  version: string;
  categories: string[];
  tags: string[];
  icon_url?: string;
  rating: number;
  downloads: number;
  download_url: string;
  is_installed: boolean;
  has_update: boolean;
}

export interface ToggleSkillRequest {
  skill_id: string;
  enabled: boolean;
}

export interface UpdateSkillConfigRequest {
  skill_id: string;
  config: Record<string, unknown>;
}

// 技能卡片通用属性接口
export interface SkillCardItem {
  id: string;
  name: string;
  description: string;
  author: string;
  version: string;
  categories: string[];
  tags: string[];
  icon_url?: string;
  rating: number;
  downloads: number;
}

// ==================== Sidecar Types ====================

export type SidecarInstallStatus =
  | { type: 'NotInstalled' }
  | { type: 'Installing'; stage: string; progress: number }
  | { type: 'Installed'; version: string; path: string }
  | { type: 'Error'; message: string };

export interface SidecarInstallResult {
  success: boolean;
  version: string | null;
  message: string;
}

export interface SidecarInfo {
  version: string;
  path: string;
  isRunning: boolean;
  pid?: number;
}
