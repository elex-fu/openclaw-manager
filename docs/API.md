# OpenClaw Manager API 文档

本文档描述了 OpenClaw Manager 的 Tauri 命令 API，供前端开发者和插件开发者参考。

## 目录

- [概述](#概述)
- [Config API](#config-api)
- [Model API](#model-api)
- [Agent API](#agent-api)
- [Diagnostics API](#diagnostics-api)
- [Service API](#service-api)
- [Secure Storage API](#secure-storage-api)
- [OpenClaw API](#openclaw-api)
- [Log API](#log-api)
- [Skill API](#skill-api)
- [Plugin Market API](#plugin-market-api)
- [System API](#system-api)
- [Update API](#update-api)
- [Plugin API](#plugin-api)
- [类型定义](#类型定义)
- [错误处理](#错误处理)
- [事件](#事件)

---

## 概述

OpenClaw Manager 使用 Tauri 的命令系统实现前后端通信。所有命令都通过 `invoke` 函数调用：

```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('command_name', { arg1: value1, arg2: value2 });
```

### API 命名空间

前端使用命名空间 API 对象进行调用（位于 `src/lib/tauri-api.ts`）：

```typescript
import {
  configApi,
  modelApi,
  agentApi,
  diagnosticsApi,
  serviceApi,
  secureStorageApi,
  openclawApi,
  logApi,
  skillApi,
  pluginMarketApi,
  systemApi,
  updateApi,
  pluginApi
} from '@/lib/tauri-api';

// 使用示例
const config = await configApi.get('theme');
```

### 响应格式

所有命令返回统一的响应格式：

```typescript
interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}
```

---

## Config API

配置管理 API，用于管理应用配置项。

### `configApi.get(key)`

获取指定配置项。

**参数**:
```typescript
key: string  // 配置键名
```

**返回**: `Promise<ApiResponse<{ key: string; value: string } | null>>`

```typescript
const response = await configApi.get('theme');
if (response.success && response.data) {
  console.log(response.data.value);
}
```

### `configApi.set(key, value, description?)`

设置配置项。

**参数**:
```typescript
{
  key: string;           // 配置键名
  value: string;         // 配置值
  description?: string;  // 可选描述
}
```

**返回**: `Promise<ApiResponse<{ key: string; value: string }>>`

```typescript
await configApi.set('theme', 'dark', 'UI theme preference');
```

### `configApi.delete(key)`

删除配置项。

**参数**:
```typescript
key: string  // 配置键名
```

**返回**: `Promise<ApiResponse<boolean>>`

---

## Model API

模型配置管理 API，用于管理 AI 模型配置。

### `modelApi.getAllModels()`

获取所有模型配置。

**参数**: 无

**返回**: `Promise<ModelConfig[]>`

```typescript
const models = await modelApi.getAllModels();
```

### `modelApi.saveModel(model)`

保存模型配置。

**参数**:
```typescript
model: ModelConfig  // 模型配置对象
```

**返回**: `Promise<ModelConfig>`

```typescript
const newModel = await modelApi.saveModel({
  id: 'gpt-4',
  name: 'GPT-4',
  provider: 'openai',
  model: 'gpt-4',
  temperature: 0.7,
  enabled: true
});
```

### `modelApi.deleteModel(id)`

删除模型配置。

**参数**:
```typescript
id: string  // 模型 ID
```

**返回**: `Promise<boolean>`

### `modelApi.setDefaultModel(id)`

设置默认模型。

**参数**:
```typescript
id: string  // 模型 ID
```

**返回**: `Promise<boolean>`

### `modelApi.testModelConnection(modelId)`

测试模型连接。

**参数**:
```typescript
modelId: string  // 模型 ID
```

**返回**: `Promise<{ success: boolean; latency: number; message?: string }>`

```typescript
const result = await modelApi.testModelConnection('gpt-4');
console.log(`Connection: ${result.success}, Latency: ${result.latency}ms`);
```

### `modelApi.reorderModels(modelIds)`

重新排序模型（按优先级）。

**参数**:
```typescript
modelIds: string[]  // 模型 ID 数组，按优先级排序
```

**返回**: `Promise<boolean>`

---

## Agent API

Agent 管理 API，用于配置和管理 AI Agent。

### `agentApi.getAllAgents()`

获取所有 Agent 配置。

**参数**: 无

**返回**: `Promise<AgentConfig[]>`

```typescript
const agents = await agentApi.getAllAgents();
```

### `agentApi.saveAgent(agent)`

保存 Agent 配置。

**参数**:
```typescript
agent: AgentConfig  // Agent 配置对象
```

**返回**: `Promise<AgentConfig>`

```typescript
const agent = await agentApi.saveAgent({
  id: 'assistant',
  name: 'AI Assistant',
  description: 'General purpose assistant',
  modelId: 'gpt-4',
  systemPrompt: 'You are a helpful assistant.',
  skills: ['web-search', 'code-execution'],
  enabled: true,
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString()
});
```

### `agentApi.deleteAgent(id)`

删除 Agent。

**参数**:
```typescript
id: string  // Agent ID
```

**返回**: `Promise<boolean>`

### `agentApi.setCurrentAgent(id)`

设置当前使用的 Agent。

**参数**:
```typescript
id: string  // Agent ID
```

**返回**: `Promise<boolean>`

### `agentApi.getCurrentAgent()`

获取当前 Agent。

**参数**: 无

**返回**: `Promise<AgentConfig | null>`

---

## Diagnostics API

系统诊断 API，用于运行诊断检查和自动修复。

### `diagnosticsApi.runDiagnostics()`

运行完整系统诊断。

**参数**: 无

**返回**: `Promise<DiagnosticResult>`

```typescript
interface DiagnosticResult {
  checks: DiagnosticCheck[];
  hasErrors: boolean;
  hasWarnings: boolean;
  checkedAt: string;
}

interface DiagnosticCheck {
  category: 'system' | 'openclaw' | 'network' | 'service';
  name: string;
  status: 'pass' | 'warning' | 'error';
  message: string;
  details?: string;
  fixable: boolean;
  fixSuggestion?: string;
}
```

```typescript
const result = await diagnosticsApi.runDiagnostics();
if (result.hasErrors) {
  console.error('System has errors that need attention');
}
```

### `diagnosticsApi.autoFix(issueNames)`

自动修复指定问题。

**参数**:
```typescript
issueNames: string[]  // 问题名称数组
```

**返回**: `Promise<FixResult>`

```typescript
interface FixResult {
  fixed: string[];           // 成功修复的问题
  failed: FixFailure[];      // 修复失败的问题
}

interface FixFailure {
  name: string;
  error: string;
}
```

### `diagnosticsApi.fixIssue(issueName)`

修复单个问题。

**参数**:
```typescript
issueName: string  // 问题名称
```

**返回**: `Promise<boolean>`

---

## Service API

服务控制 API，用于管理 OpenClaw 服务。

### `serviceApi.startService()`

启动 OpenClaw 服务。

**参数**: 无

**返回**: `Promise<boolean>`

```typescript
const success = await serviceApi.startService();
```

### `serviceApi.stopService()`

停止 OpenClaw 服务。

**参数**: 无

**返回**: `Promise<boolean>`

### `serviceApi.getServiceStatus()`

获取服务状态。

**参数**: 无

**返回**: `Promise<ServiceInfo>`

```typescript
interface ServiceInfo {
  name: string;
  status: 'stopped' | 'starting' | 'running' | 'stopping' | 'error';
  pid?: number;
  uptime?: number;
  error?: string;
}
```

### `serviceApi.healthCheck()`

执行服务健康检查。

**参数**: 无

**返回**: `Promise<{ healthy: boolean; message?: string }>`

---

## Secure Storage API

安全存储 API，用于安全地存储敏感信息（如 API 密钥）。

### `secureStorageApi.saveApiKey(provider, apiKey)`

保存 API 密钥到系统钥匙串。

**参数**:
```typescript
provider: string  // 提供商名称 (e.g., 'openai', 'anthropic')
apiKey: string    // API 密钥
```

**返回**: `Promise<void>`

```typescript
await secureStorageApi.saveApiKey('openai', 'sk-...');
```

### `secureStorageApi.getApiKey(provider)`

获取 API 密钥。

**参数**:
```typescript
provider: string  // 提供商名称
```

**返回**: `Promise<string | null>`

### `secureStorageApi.deleteApiKey(provider)`

删除 API 密钥。

**参数**:
```typescript
provider: string  // 提供商名称
```

**返回**: `Promise<void>`

### `secureStorageApi.hasApiKey(provider)`

检查是否存在 API 密钥。

**参数**:
```typescript
provider: string  // 提供商名称
```

**返回**: `Promise<boolean>`

---

## OpenClaw API

OpenClaw 安装和管理 API。

### `openclawApi.checkInstallation()`

检查 OpenClaw 安装状态。

**参数**: 无

**返回**: `Promise<InstallStatus>`

```typescript
type InstallStatus =
  | { type: 'NotInstalled' }
  | { type: 'Installing'; stage: string; progress: number }
  | { type: 'Installed'; version: string }
  | { type: 'Error'; message: string };
```

### `openclawApi.install(version?, networkPreference?, offlinePackagePath?)`

安装 OpenClaw。

**参数**:
```typescript
version?: string;            // 指定版本
networkPreference?: string;  // 网络偏好
offlinePackagePath?: string; // 离线包路径
```

**返回**: `Promise<InstallResult>`

```typescript
interface InstallResult {
  success: boolean;
  version: string | null;
  message: string;
}
```

### `openclawApi.installOneClick(useOfflinePackage?)`

一键安装 OpenClaw（包含嵌入式 Runtime + 预设配置）。

**参数**:
```typescript
useOfflinePackage?: boolean  // 是否使用离线包，默认 true
```

**返回**: `Promise<InstallResult>`

### `openclawApi.uninstall()`

卸载 OpenClaw。

**参数**: 无

**返回**: `Promise<boolean>`

### `openclawApi.getConfig()`

获取 OpenClaw 配置。

**参数**: 无

**返回**: `Promise<OpenClawConfig>`

### `openclawApi.updateConfig(config)`

更新 OpenClaw 配置。

**参数**:
```typescript
config: OpenClawConfig  // 完整配置对象
```

**返回**: `Promise<OpenClawConfig>`

### `openclawApi.checkSystemEnvironment()`

检查系统环境。

**参数**: 无

**返回**: `Promise<SystemEnvironmentCheckResult>`

```typescript
interface SystemEnvironmentCheckResult {
  checks: SystemCheckResult[];
  can_install: boolean;
  missing_dependencies: string[];
}
```

### `openclawApi.onInstallProgress(callback)`

监听安装进度事件。

**参数**:
```typescript
callback: (progress: InstallProgress) => void
```

**返回**: `Promise<UnlistenFn>`

```typescript
interface InstallProgress {
  stage: 'Checking' | 'Downloading' | 'Installing' | 'Configuring' | 'Complete' | 'Error';
  percentage: number;
  message: string;
}
```

```typescript
const unlisten = await openclawApi.onInstallProgress((progress) => {
  console.log(`[${progress.stage}] ${progress.percentage}%: ${progress.message}`);
});

// 取消监听
unlisten();
```

---

## Log API

日志查看和管理 API，支持 WebSocket 实时流。

### `logApi.getLogSources()`

获取日志源列表。

**参数**: 无

**返回**: `Promise<LogSourceInfo[]>`

```typescript
interface LogSourceInfo {
  id: string;
  name: string;
  path: string;
  size: number;
  modified: number;
}
```

### `logApi.getRecentLogs(params)`

获取最近日志。

**参数**:
```typescript
{
  limit?: number;           // 默认 100
  levels?: LogLevel[];      // 日志级别筛选
  sources?: string[];       // 日志源筛选
  searchQuery?: string;     // 搜索关键词
}
```

**返回**: `Promise<LogEntry[]>`

```typescript
interface LogEntry {
  id: string;
  timestamp: number;
  level: 'ERROR' | 'WARN' | 'INFO' | 'DEBUG' | 'TRACE';
  source: string;
  message: string;
  metadata?: Record<string, unknown>;
}
```

### `logApi.subscribeLogs(params)`

订阅实时日志流。

**参数**:
```typescript
{
  levels: LogLevel[];       // 要监听的日志级别
  sources?: string[];       // 日志源筛选
  searchQuery?: string;     // 搜索关键词
}
```

**返回**: `Promise<{ subscription_id: string }>`

### `logApi.unsubscribeLogs(subscriptionId)`

取消日志订阅。

**参数**:
```typescript
subscriptionId: string  // 订阅 ID
```

**返回**: `Promise<boolean>`

### `logApi.exportLogs(params)`

导出日志。

**参数**:
```typescript
{
  filter: LogFilter;        // 日志过滤器
  format: 'text' | 'json' | 'csv';
  outputPath: string;       // 输出路径
}
```

**返回**: `Promise<string>`  // 导出文件路径

### `logApi.onLogEntry(subscriptionId, callback)`

监听日志条目事件。

**参数**:
```typescript
subscriptionId: string;
callback: (entry: LogEntry) => void;
```

**返回**: `Promise<UnlistenFn>`

### `logApi.onLogReset(subscriptionId, callback)`

监听日志重置事件。

**参数**:
```typescript
subscriptionId: string;
callback: (source: string) => void;
```

**返回**: `Promise<UnlistenFn>`

### `logApi.getLogStats()`

获取日志统计信息。

**参数**: 无

**返回**: `Promise<LogStats>`

---

## Skill API

技能管理 API，用于管理已安装技能和浏览技能市场。

### `skillApi.getAll()`

获取所有已安装技能。

**参数**: 无

**返回**: `Promise<InstalledSkill[]>`

```typescript
interface InstalledSkill extends Skill {
  is_enabled: boolean;
  config: Record<string, unknown>;
  installed_at: string;
  updated_at: string;
  has_update: boolean;
  latest_version?: string;
}
```

### `skillApi.getById(skillId)`

获取技能详情。

**参数**:
```typescript
skillId: string  // 技能 ID
```

**返回**: `Promise<InstalledSkill | null>`

### `skillApi.install(skillId)`

安装技能。

**参数**:
```typescript
skillId: string  // 技能 ID
```

**返回**: `Promise<InstalledSkill>`

### `skillApi.uninstall(skillId)`

卸载技能。

**参数**:
```typescript
skillId: string  // 技能 ID
```

**返回**: `Promise<boolean>`

### `skillApi.enable(skillId)` / `skillApi.disable(skillId)`

启用/禁用技能。

**参数**:
```typescript
skillId: string  // 技能 ID
```

**返回**: `Promise<InstalledSkill>`

### `skillApi.toggle(skillId, enabled)`

切换技能状态。

**参数**:
```typescript
skillId: string;
enabled: boolean;
```

**返回**: `Promise<InstalledSkill>`

### `skillApi.getConfig(skillId)`

获取技能配置。

**参数**:
```typescript
skillId: string  // 技能 ID
```

**返回**: `Promise<Record<string, unknown>>`

### `skillApi.updateConfig(skillId, config)`

更新技能配置。

**参数**:
```typescript
skillId: string;
config: Record<string, unknown>;
```

**返回**: `Promise<InstalledSkill>`

### `skillApi.update(skillId)`

更新技能到最新版本。

**参数**:
```typescript
skillId: string  // 技能 ID
```

**返回**: `Promise<InstalledSkill>`

### `skillApi.checkUpdates()`

检查技能更新。

**参数**: 无

**返回**: `Promise<Array<[string, string]>>`  // [skillId, latestVersion] 数组

### `skillApi.searchMarket(params?)`

搜索技能市场。

**参数**:
```typescript
{
  query?: string;
  category?: string;
  page?: number;
  perPage?: number;
}
```

**返回**: `Promise<SkillSearchResult>`

### `skillApi.getMarketDetail(skillId)`

获取市场技能详情。

**参数**:
```typescript
skillId: string  // 技能 ID
```

**返回**: `Promise<Skill>`

### `skillApi.getPopular(limit?)`

获取热门技能。

**参数**:
```typescript
limit?: number  // 默认 10
```

**返回**: `Promise<SkillMarketItem[]>`

### `skillApi.getLatest(limit?)`

获取最新技能。

**参数**:
```typescript
limit?: number  // 默认 10
```

**返回**: `Promise<SkillMarketItem[]>`

### `skillApi.getCategories()`

获取技能分类。

**参数**: 无

**返回**: `Promise<SkillCategory[]>`

---

## Plugin Market API

插件市场 API，用于浏览和安装插件。

### `pluginMarketApi.search(params?)`

搜索市场插件。

**参数**:
```typescript
{
  query?: string;
  category?: string;
  sortBy?: 'relevance' | 'downloads' | 'rating' | 'created_at' | 'updated_at';
  page?: number;
  perPage?: number;
}
```

**返回**: `Promise<SearchPluginsResult>`

```typescript
interface SearchPluginsResult {
  plugins: MarketPlugin[];
  total: number;
  page: number;
  per_page: number;
  has_more: boolean;
}
```

### `pluginMarketApi.getDetails(pluginId)`

获取插件详情。

**参数**:
```typescript
pluginId: string  // 插件 ID
```

**返回**: `Promise<MarketPlugin>`

```typescript
interface MarketPlugin {
  id: string;
  name: string;
  version: string;
  description?: string;
  author?: string;
  downloads: number;
  rating: number;
  rating_count: number;
  download_url: string;
  categories: string[];
  tags: string[];
  created_at: string;
  updated_at: string;
  size_bytes: number;
}
```

### `pluginMarketApi.getCategories()`

获取插件分类。

**参数**: 无

**返回**: `Promise<PluginCategory[]>`

### `pluginMarketApi.getPopular(limit?)`

获取热门插件。

**参数**:
```typescript
limit?: number  // 默认 10
```

**返回**: `Promise<MarketPlugin[]>`

### `pluginMarketApi.getLatest(limit?)`

获取最新插件。

**参数**:
```typescript
limit?: number  // 默认 10
```

**返回**: `Promise<MarketPlugin[]>`

---

## System API

系统操作 API，用于获取系统信息和资源监控。

### `systemApi.getSystemInfo()`

获取系统信息。

**参数**: 无

**返回**: `Promise<SystemInfo>`

```typescript
interface SystemInfo {
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
```

### `systemApi.getSystemResources()`

获取系统资源使用情况。

**参数**: 无

**返回**: `Promise<SystemResources>`

```typescript
interface SystemResources {
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
```

### `systemApi.getRecentActivities(limit?)`

获取最近活动。

**参数**:
```typescript
limit?: number  // 默认 10
```

**返回**: `Promise<Activity[]>`

```typescript
interface Activity {
  id: string;
  timestamp: number;
  activity_type: 'install' | 'config' | 'service' | 'error';
  message: string;
  details?: string;
}
```

### `systemApi.getDiagnosticAlerts()`

获取诊断告警。

**参数**: 无

**返回**: `Promise<DiagnosticAlert[]>`

```typescript
interface DiagnosticAlert {
  id: string;
  severity: 'info' | 'warning' | 'error';
  title: string;
  message: string;
  fixable: boolean;
  category: string;
}
```

---

## Update API

更新操作 API，用于检查和应用应用更新。

### `updateApi.checkForUpdates()`

检查可用更新。

**参数**: 无

**返回**: `Promise<UpdateState>`

```typescript
interface UpdateState {
  currentVersion: string | null;
  latestVersion: string | null;
  hasUpdate: boolean;
  updateInfo: UpdateInfo | null;
}

interface UpdateInfo {
  version: string;
  releaseDate: string;
  changelog: string;
  downloadUrl: string;
  checksum: string;
  mandatory: boolean;
  minSupportedVersion: string | null;
}
```

### `updateApi.performUpdate(updateInfo)`

执行更新。

**参数**:
```typescript
updateInfo: UpdateInfo  // 更新信息
```

**返回**: `Promise<InstallResult>`

### `updateApi.getBackupList()`

获取备份列表。

**参数**: 无

**返回**: `Promise<BackupMetadata[]>`

```typescript
interface BackupMetadata {
  createdAt: string;
  version: string | null;
  path: string;
}
```

### `updateApi.restoreFromBackup(backupPath)`

从备份恢复。

**参数**:
```typescript
backupPath: string  // 备份路径
```

**返回**: `Promise<void>`

### `updateApi.onUpdateProgress(callback)`

监听更新进度。

**参数**:
```typescript
callback: (progress: UpdateProgress) => void
```

**返回**: `Promise<UnlistenFn>`

```typescript
interface UpdateProgress {
  stage: 'Checking' | 'Downloading' | 'BackingUp' | 'Installing' | 'Migrating' | 'CleaningUp' | 'Complete' | 'Error' | 'Rollback';
  percentage: number;
  message: string;
  canCancel: boolean;
}
```

---

## Plugin API

已安装插件管理 API。

### `pluginApi.getAll()`

获取所有已安装插件。

**参数**: 无

**返回**: `Promise<Plugin[]>`

```typescript
interface Plugin {
  id: string;
  name: string;
  version: string;
  description?: string;
  author?: string;
  plugin_type: string;
  entry_point: string;
  is_enabled: boolean;
  created_at: string;
  updated_at: string;
}
```

### `pluginApi.install(marketItemId, downloadUrl)`

安装插件。

**参数**:
```typescript
marketItemId: string;  // 市场项 ID
downloadUrl: string;   // 下载 URL
```

**返回**: `Promise<Plugin>`

### `pluginApi.uninstall(id)`

卸载插件。

**参数**:
```typescript
id: string  // 插件 ID
```

**返回**: `Promise<boolean>`

### `pluginApi.enable(id)` / `pluginApi.disable(id)`

启用/禁用插件。

**参数**:
```typescript
id: string  // 插件 ID
```

**返回**: `Promise<Plugin>`

### `pluginApi.getConfig(pluginId)`

获取插件配置。

**参数**:
```typescript
pluginId: string  // 插件 ID
```

**返回**: `Promise<{ plugin_id: string; config: Record<string, unknown> } | null>`

### `pluginApi.updateConfig(pluginId, config)`

更新插件配置。

**参数**:
```typescript
pluginId: string;
config: Record<string, unknown>;
```

**返回**: `Promise<{ plugin_id: string; config: Record<string, unknown> }>`

### `pluginApi.getEnabled()`

获取已启用的插件。

**参数**: 无

**返回**: `Promise<Plugin[]>`

### `pluginApi.checkInstalled(pluginId)`

检查插件是否已安装。

**参数**:
```typescript
pluginId: string  // 插件 ID
```

**返回**: `Promise<boolean>`

---

## 类型定义

### ModelConfig

```typescript
interface ModelConfig {
  id: string;
  name: string;
  provider: string;
  api_base?: string;
  model: string;
  temperature: number;
  max_tokens?: number;
  enabled: boolean;
  isDefault?: boolean;
}
```

### AgentConfig

```typescript
interface AgentConfig {
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
```

### OpenClawConfig

```typescript
interface OpenClawConfig {
  version: string;
  name: string;
  models: ModelConfig[];
  defaultModel?: string;
  agents: AgentConfig[];
  skills: SkillConfig[];
  settings: SystemSettings;
}
```

### SkillConfig

```typescript
interface SkillConfig {
  id: string;
  name: string;
  description?: string;
  entryPoint: string;
  config?: Record<string, unknown>;
  enabled: boolean;
}
```

### Skill

```typescript
interface Skill {
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
```

### LogFilter

```typescript
interface LogFilter {
  levels: LogLevel[];
  searchQuery?: string;
  sources?: string[];
  startTime?: number;
  endTime?: number;
}
```

---

## 错误处理

### 错误类型

| 错误代码 | 描述 | 处理建议 |
|---------|------|---------|
| `INSTALL_FAILED` | 安装失败 | 检查磁盘空间和网络连接 |
| `CONFIG_INVALID` | 配置无效 | 检查配置格式和值 |
| `SERVICE_NOT_RUNNING` | 服务未运行 | 启动服务后重试 |
| `API_KEY_MISSING` | API 密钥缺失 | 配置 API 密钥 |
| `NETWORK_ERROR` | 网络错误 | 检查网络连接和代理设置 |
| `PERMISSION_DENIED` | 权限不足 | 以管理员身份运行 |
| `RESOURCE_NOT_FOUND` | 资源未找到 | 检查路径是否正确 |
| `TIMEOUT` | 请求超时 | 检查网络连接或增加超时时间 |

### API 错误类

```typescript
class ApiError extends Error {
  constructor(
    message: string,
    public code?: string,
    public originalError?: unknown,
    public isRetryable?: boolean
  )
}
```

### 错误处理示例

```typescript
import { modelApi, ApiError } from '@/lib/tauri-api';

try {
  const result = await modelApi.testModelConnection('gpt-4');
} catch (error) {
  if (error instanceof ApiError) {
    console.error('Error code:', error.code);
    console.error('Message:', error.message);
    console.error('Retryable:', error.isRetryable);

    if (error.isRetryable) {
      // 可以重试
    }
  }
}
```

### 带重试的调用

API 层提供了自动重试机制：

```typescript
import { invokeWithRetry, invokeWithTimeout } from '@/lib/tauri-api';

// 自动重试（最多 3 次，指数退避）
const result = await invokeWithRetry('command_name', args, {
  maxRetries: 3,
  baseDelay: 1000,
  maxDelay: 30000
});

// 带超时控制
const result = await invokeWithTimeout('command_name', args, 30000);
```

---

## 事件

### 安装进度事件

```typescript
import { listen } from '@tauri-apps/api/event';

listen('install-progress', (event) => {
  const { stage, percentage, message } = event.payload;
  console.log(`[${stage}] ${percentage}%: ${message}`);
});
```

### 日志事件

```typescript
// 监听日志条目
logApi.onLogEntry(subscriptionId, (entry) => {
  console.log(`[${entry.timestamp}] [${entry.level}] ${entry.message}`);
});

// 监听日志重置
logApi.onLogReset(subscriptionId, (source) => {
  console.log(`Log reset for source: ${source}`);
});

// 监听日志错误
logApi.onLogError(subscriptionId, ({ source, error }) => {
  console.error(`Log error from ${source}: ${error}`);
});
```

### 更新进度事件

```typescript
updateApi.onUpdateProgress((progress) => {
  const { stage, percentage, message, canCancel } = progress;
  console.log(`[${stage}] ${percentage}%: ${message}`);
});
```

### 系统资源事件

```typescript
import { listen } from '@tauri-apps/api/event';

listen('system-resources', (event) => {
  const { cpuUsage, memoryUsage } = event.payload;
  updateResourceDisplay(cpuUsage, memoryUsage);
});
```

---

*API 文档版本: 0.1.1*
*最后更新: 2024-03-03*
