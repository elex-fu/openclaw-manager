# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

OpenClaw Manager is a Tauri-based desktop application for managing OpenClaw (an AI gateway/agent system). It provides installation, configuration, model management, agent management, diagnostics, log viewing, skill management, and update capabilities.

**Tech Stack:**
- **Backend:** Rust + Tauri v2
- **Frontend:** React + TypeScript + Tailwind CSS
- **State Management:** Zustand + TanStack Query
- **UI Components:** shadcn/ui + Radix UI primitives
- **Testing:** Vitest (frontend) + Rust built-in test (backend) + Playwright (E2E)

## Common Commands

### Development

```bash
# Run development server (starts Vite dev server + Tauri)
npm run tauri:dev

# Run frontend only (Vite dev server on port 5173)
npm run dev
```

### Building

```bash
# Build production Tauri app
npm run tauri:build

# Build frontend only
npm run build
```

### Testing

```bash
# Run all frontend tests (Vitest)
npm run test

# Run tests with UI
npm run test:ui

# Run tests with coverage
npm run test:coverage

# Run single test file
npm run test -- src/stores/__tests__/appStore.test.ts

# Run Rust tests
cd src-tauri && cargo test

# Run specific Rust test
cd src-tauri && cargo test test_install_directory_creation

# Run E2E tests
npx playwright test

# Run specific E2E test
npx playwright test e2e/installation.spec.ts
```

### Linting

```bash
# Run ESLint
npm run lint
```

## Navigation Structure

The application has 8 navigation items in the sidebar:

1. **Dashboard** - `/` - Main dashboard with system status and quick actions
2. **Model Config** - `/models` - AI model configuration and management
3. **Agent Management** - `/agents` - AI agent creation and configuration
4. **Skill Store** - `/skills` - Browse, install, and manage skills
5. **Diagnostics** - `/diagnostics` - System diagnostics and auto-fix issues
6. **Log Viewer** - `/logs` - View and filter application logs
7. **Update** - `/update` - Check and install updates
8. **Settings** - `/settings` - Application settings

## Architecture

### Frontend Architecture

**Routing:** Uses `HashRouter` from react-router-dom (required for Tauri file-based navigation).

**Pages:** (`src/pages/`)
- `Dashboard.tsx` - Main dashboard
- `InstallWizard.tsx` - Installation wizard
- `ModelConfig.tsx` - Model configuration
- `AgentManager.tsx` - Agent management
- `Diagnostics.tsx` - System diagnostics
- `LogViewer.tsx` - Log viewing (NEW)
- `SkillStore.tsx` - Skill store (NEW)
- `UpdateManager.tsx` - Update management
- `SettingsPage.tsx` - Settings

**API Layer:** (`src/lib/tauri-api.ts`)
- All Tauri command invocations are centralized here
- 12 API namespaces:
  - `configApi` - Configuration management
  - `pluginApi` - Plugin management
  - `pluginMarketApi` - Plugin marketplace
  - `systemApi` - System information and resources
  - `openclawApi` - OpenClaw installation and management
  - `serviceApi` - Service control (start/stop/status)
  - `secureStorageApi` - Secure API key storage
  - `modelApi` - Model configuration
  - `agentApi` - Agent management
  - `diagnosticsApi` - System diagnostics
  - `logApi` - Log viewing and management (NEW)
  - `skillApi` - Skill management (NEW)
  - `updateApi` - Update management (NEW)
- Event listening for progress updates via `listen()` from `@tauri-apps/api/event`

**State Management:**
- **Zustand** for client-side state (`src/stores/appStore.ts`, `src/stores/configStore.ts`, `src/stores/installStore.ts`)
- **TanStack Query** for server-state synchronization (cache, refetching)
- Stores use `persist` middleware for localStorage persistence

**Type Definitions:** (`src/types/index.ts`)
- Shared TypeScript types mirror Rust structs
- Key types: `OpenClawConfig`, `ModelConfig`, `AgentConfig`, `InstallStatus`, `ServiceInfo`, `DiagnosticResult`, `LogEntry`, `Skill`, `InstalledSkill`

### Backend Architecture (Rust)

**Command Pattern:** (`src-tauri/src/commands/`)
- Tauri commands are organized by domain: `openclaw.rs`, `config.rs`, `plugin.rs`, `secure.rs`, `service.rs`
- All commands return `ApiResponse<T>` for consistent error handling
- Commands registered in `main.rs` via `tauri::generate_handler!`

**State Management:**
- `InstallerState` managed as Tauri state (shared across commands)
- Contains `Arc<Mutex<OpenClawInstaller>>` and `Arc<Mutex<InstallerService>>`

**Service Layer:** (`src-tauri/src/services/`)
- Business logic separated from commands
- Key services: `installer.rs`, `offline_installer.rs`, `process_manager.rs`, `config_manager.rs`, `secure_storage.rs`, `skill_manager.rs`, `plugin_manager.rs`

**Error Handling:** (`src-tauri/src/errors/`)
- Custom error types in `app_error.rs`
- `ApiResponse<T>` struct for frontend-compatible responses

**System Detection:** (`src-tauri/src/system/mod.rs`)
- Platform-specific logic for macOS, Windows, Linux
- System info detection for install script selection

## Key Patterns

### API Response Pattern
All Rust commands return a standardized response:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}
```

### Progress Events
Long-running operations (installation, updates) emit progress events:
```rust
// Backend
window.emit("install-progress", json!({ "stage": "Downloading", "percentage": 50, "message": "..." }))
window.emit("update-progress", json!({ "stage": "Downloading", "percentage": 50, "message": "..." }))

// Frontend
openclawApi.onInstallProgress((progress) => { ... })
updateApi.onUpdateProgress((progress) => { ... })
```

### Log Streaming
Real-time log streaming via event subscription:
```rust
// Backend
window.emit(&format!("log-entry-{}", subscription_id), log_entry)

// Frontend
logApi.onLogEntry(subscriptionId, (entry) => { ... })
```

### Secure Storage
API keys are stored using OS keychain (via `keyring` crate), not in config files. Commands in `secure.rs` handle this.

### File Structure Conventions
- Frontend components use kebab-case files (`main-layout.tsx`) but imports use PascalCase
- Rust modules follow standard snake_case
- UI components in `src/components/ui/` are from shadcn/ui
- Page components in `src/pages/`

## Important Notes

**Path Aliases:**
- `@/` maps to `src/` in both frontend (Vite) and tests (Vitest)

**Tauri Configuration:**
- App identifier: `com.openclaw.manager`
- Tray icon enabled with menu (show/quit)
- Window size: 1200x800 (min: 900x600)

**Testing:**
- Frontend tests use jsdom environment with `@testing-library/react`
- Test setup in `src/test/setup.ts`
- Mock Tauri APIs in `src/test/mocks/tauri.ts`
- E2E tests use Playwright in `e2e/` directory

**Plugin System:** (Partially implemented)
- Plugin runtime in `src-tauri/src/plugins/`
- Supports Lua, JavaScript, and WASM plugins
- Sandbox environment for plugin execution

## Working with Commands

When adding a new Tauri command:

1. Add command function to appropriate file in `src-tauri/src/commands/`
2. Export in `src-tauri/src/commands/mod.rs`
3. Register in `main.rs` invoke handler
4. Add frontend wrapper in `src/lib/tauri-api.ts`
5. Add types to `src/types/index.ts` if needed

Example command signature:
```rust
#[tauri::command]
pub async fn my_command(
    state: State<'_, InstallerState>,
    arg: String,
) -> Result<ApiResponse<ReturnType>, String> {
    // Implementation
}
```

## Testing Structure

### Frontend Tests
- **Unit tests:** `src/stores/__tests__/` - Store tests
- **Component tests:** Can be added in `src/components/**/__tests__/` (not yet implemented)

### Backend Tests
- **Unit tests:** Inline tests in source files (`*_test.rs` modules)
- **Integration tests:** `src-tauri/tests/` directory

### E2E Tests
- **Playwright tests:** `e2e/` directory
- Tests cover: Installation, Model Config, Agent Management, Service Control, Diagnostics, Settings

## New Features (Recent Additions)

### Log Viewer (`/logs`)
- Real-time log streaming with subscription-based architecture
- Filter by level, source, and search query
- Export logs in multiple formats (text, JSON, CSV)
- Add/remove custom log sources

### Skill Store (`/skills`)
- Browse skills from marketplace
- Install/uninstall/enable/disable skills
- Configure skill settings
- Check for skill updates
- Categories and search functionality

### Update Manager (`/update`)
- Check for application updates
- One-click update installation
- Offline update support
- Backup and restore functionality
- Update progress tracking
