# Sidecar UI 测试开发指南

本文档说明如何在前端代码中添加测试支持，以便自动化测试能够稳定地测试 Sidecar 功能。

## 必需的 data-testid 属性

### 安装向导页面 (`src/pages/InstallWizard.tsx`)

```tsx
// Sidecar 安装按钮
<button
  data-testid="sidecar-install"
  onClick={handleSidecarInstall}
>
  Sidecar 安装
</button>

// 安装进度条
<div data-testid="install-progress" role="progressbar">
  <span>{percentage}%</span>
</div>

// 安装阶段文本
<div data-testid="install-stage">
  {stage} - {message}
</div>

// 安装完成标识
<div data-testid="install-complete">
  安装完成
</div>

// 错误提示
<div data-testid="install-error">
  {errorMessage}
</div>

// 重试按钮
<button data-testid="install-retry">
  重试
</button>
```

### 仪表盘页面 (`src/pages/Dashboard.tsx`)

```tsx
// 服务状态卡片
<div data-testid="service-status">
  <span>{status}</span>
</div>

// Sidecar 版本信息
<div data-testid="sidecar-version">
  {version}
</div>

// 启动服务按钮
<button data-testid="start-service">
  启动
</button>

// 停止服务按钮
<button data-testid="stop-service">
  停止
</button>

// 已安装标识
<div data-testid="sidecar-installed">
  已安装
</div>

// 未安装提示
<div data-testid="sidecar-not-installed">
  未安装 Sidecar
</div>
```

## 进度指示实现示例

### 安装进度组件

```tsx
interface InstallProgressProps {
  stage: string;
  percentage: number;
  message: string;
}

export function InstallProgress({ stage, percentage, message }: InstallProgressProps) {
  return (
    <div className="install-progress">
      {/* 进度条 */}
      <div
        data-testid="install-progress"
        role="progressbar"
        aria-valuenow={percentage}
        aria-valuemin={0}
        aria-valuemax={100}
      >
        <div
          className="progress-bar"
          style={{ width: `${percentage}%` }}
        />
        <span className="percentage">{percentage}%</span>
      </div>

      {/* 阶段文本 */}
      <div data-testid="install-stage">
        {stage}
      </div>

      {/* 详细信息 */}
      <div data-testid="install-message">
        {message}
      </div>
    </div>
  );
}
```

### 服务状态组件

```tsx
interface ServiceStatusProps {
  status: 'running' | 'stopped' | 'starting' | 'stopping' | 'error';
  version?: string;
  pid?: number;
}

export function ServiceStatus({ status, version, pid }: ServiceStatusProps) {
  return (
    <div data-testid="service-status" className={`status-${status}`}>
      <span className="status-indicator" />
      <span className="status-text">
        {status === 'running' && '运行中'}
        {status === 'stopped' && '已停止'}
        {status === 'starting' && '启动中'}
        {status === 'stopping' && '停止中'}
        {status === 'error' && '错误'}
      </span>

      {version && (
        <span data-testid="sidecar-version" className="version">
          v{version}
        </span>
      )}

      {pid && (
        <span data-testid="service-pid" className="pid">
          PID: {pid}
        </span>
      )}
    </div>
  );
}
```

## 状态管理集成

### 使用 Zustand store

```tsx
// stores/sidecarStore.ts
import { create } from 'zustand';

interface SidecarState {
  // 安装状态
  installStatus: 'not-installed' | 'installing' | 'installed' | 'error';
  installProgress: number;
  installStage: string;
  installMessage: string;
  installError?: string;

  // 服务状态
  serviceStatus: 'stopped' | 'starting' | 'running' | 'stopping' | 'error';
  servicePid?: number;
  version?: string;

  // Actions
  startInstall: () => Promise<void>;
  retryInstall: () => Promise<void>;
  startService: () => Promise<void>;
  stopService: () => Promise<void>;
}

export const useSidecarStore = create<SidecarState>((set, get) => ({
  installStatus: 'not-installed',
  installProgress: 0,
  installStage: '',
  installMessage: '',
  serviceStatus: 'stopped',

  startInstall: async () => {
    set({ installStatus: 'installing', installProgress: 0 });

    try {
      // 监听进度事件
      const unlisten = await openclawApi.onInstallProgress((progress) => {
        set({
          installProgress: progress.percentage,
          installStage: progress.stage,
          installMessage: progress.message,
        });
      });

      // 调用安装
      const result = await openclawApi.installSidecar();

      unlisten();

      if (result.success) {
        set({ installStatus: 'installed' });
      } else {
        set({
          installStatus: 'error',
          installError: result.message
        });
      }
    } catch (error) {
      set({
        installStatus: 'error',
        installError: String(error)
      });
    }
  },

  retryInstall: async () => {
    set({
      installStatus: 'not-installed',
      installError: undefined
    });
    await get().startInstall();
  },

  startService: async () => {
    set({ serviceStatus: 'starting' });
    try {
      const pid = await serviceApi.startSidecar();
      set({ serviceStatus: 'running', servicePid: pid });
    } catch (error) {
      set({ serviceStatus: 'error' });
    }
  },

  stopService: async () => {
    set({ serviceStatus: 'stopping' });
    try {
      await serviceApi.stopSidecar();
      set({ serviceStatus: 'stopped', servicePid: undefined });
    } catch (error) {
      set({ serviceStatus: 'error' });
    }
  },
}));
```

## 测试场景实现

### 场景 1: 首次安装

```tsx
// 安装向导组件
export function InstallWizard() {
  const {
    installStatus,
    installProgress,
    installStage,
    installMessage,
    installError,
    startInstall,
    retryInstall
  } = useSidecarStore();

  return (
    <div className="install-wizard">
      {installStatus === 'not-installed' && (
        <div data-testid="install-options">
          <button
            data-testid="sidecar-install"
            onClick={startInstall}
          >
            Sidecar 安装
          </button>
        </div>
      )}

      {installStatus === 'installing' && (
        <InstallProgress
          stage={installStage}
          percentage={installProgress}
          message={installMessage}
        />
      )}

      {installStatus === 'installed' && (
        <div data-testid="install-complete">
          <h2>安装成功!</h2>
          <button onClick={() => navigate('/')}>
            进入仪表盘
          </button>
        </div>
      )}

      {installStatus === 'error' && (
        <div data-testid="install-error">
          <p>{installError}</p>
          <button data-testid="install-retry" onClick={retryInstall}>
            重试
          </button>
        </div>
      )}
    </div>
  );
}
```

### 场景 2: 服务控制

```tsx
// 仪表盘服务控制
export function ServiceControl() {
  const {
    serviceStatus,
    servicePid,
    version,
    startService,
    stopService
  } = useSidecarStore();

  return (
    <div className="service-control">
      <ServiceStatus
        status={serviceStatus}
        version={version}
        pid={servicePid}
      />

      {serviceStatus === 'stopped' && (
        <button data-testid="start-service" onClick={startService}>
          启动服务
        </button>
      )}

      {serviceStatus === 'running' && (
        <button data-testid="stop-service" onClick={stopService}>
          停止服务
        </button>
      )}
    </div>
  );
}
```

## 测试验证清单

### 运行测试前检查

- [ ] 所有按钮都有 `data-testid` 属性
- [ ] 进度条有 `role="progressbar"` 和正确的 `aria-*` 属性
- [ ] 状态文本有唯一的 `data-testid`
- [ ] 错误消息可以唯一标识
- [ ] 异步操作有加载状态

### 测试覆盖率

- [ ] 安装按钮点击
- [ ] 进度更新显示
- [ ] 阶段文本变化
- [ ] 安装完成跳转
- [ ] 错误提示显示
- [ ] 重试按钮功能
- [ ] 服务启动/停止
- [ ] 版本信息显示

## 调试技巧

### 使用 Playwright Inspector

```bash
# 调试特定测试
npx playwright test e2e/sidecar-installation.spec.ts --debug

# 逐步执行
npx playwright test e2e/sidecar-installation.spec.ts --headed --timeout=0
```

### 查看测试日志

```typescript
test('example', async ({ page }) => {
  // 启用控制台日志
  page.on('console', msg => console.log(msg.text()));

  // 启用网络日志
  page.on('request', request => console.log('>>', request.method(), request.url()));
  page.on('response', response => console.log('<<', response.status(), response.url()));

  // ... 测试代码
});
```

## 相关文件

- `src/pages/InstallWizard.tsx` - 安装向导
- `src/pages/Dashboard.tsx` - 仪表盘
- `src/stores/sidecarStore.ts` - Sidecar 状态管理
- `src/components/install/InstallProgress.tsx` - 进度组件
- `src/components/service/ServiceStatus.tsx` - 服务状态
- `e2e/sidecar-installation.spec.ts` - E2E 测试
- `e2e/utils/sidecar-helpers.ts` - 测试辅助函数

## 参考

- [Playwright 最佳实践](https://playwright.dev/docs/best-practices)
- [可访问性测试](https://playwright.dev/docs/accessibility-testing)
- [Tauri 前端测试](https://tauri.app/v1/guides/testing/webdriver/introduction/)
