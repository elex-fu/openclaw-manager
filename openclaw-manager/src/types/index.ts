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
