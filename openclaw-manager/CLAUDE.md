# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

OpenClaw Manager is a Tauri-based desktop application for managing OpenClaw (an AI gateway/agent system). It provides installation, configuration, model management, agent management, and diagnostics capabilities.

**Tech Stack:**
- **Backend:** Rust + Tauri v2
- **Frontend:** React + TypeScript + Tailwind CSS
- **State Management:** Zustand + TanStack Query
- **UI Components:** shadcn/ui + Radix UI primitives
- **Testing:** Vitest (frontend) + Rust built-in test (backend)

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
```

### Linting

```bash
# Run ESLint
npm run lint
```

## Architecture

### Frontend Architecture

**Routing:** Uses `HashRouter` from react-router-dom (required for Tauri file-based navigation).

**API Layer:** (`src/lib/tauri-api.ts`)
- All Tauri command invocations are centralized here
- API namespaces: `configApi`, `pluginApi`, `systemApi`, `openclawApi`, `serviceApi`, `secureStorageApi`, `modelApi`, `agentApi`, `diagnosticsApi`
- Event listening for progress updates via `listen()` from `@tauri-apps/api/event`

**State Management:**
- **Zustand** for client-side state (`src/stores/appStore.ts`, `src/stores/configStore.ts`, `src/stores/installStore.ts`)
- **TanStack Query** for server-state synchronization (cache, refetching)
- Stores use `persist` middleware for localStorage persistence

**Type Definitions:** (`src/types/index.ts`)
- Shared TypeScript types mirror Rust structs
- Key types: `OpenClawConfig`, `ModelConfig`, `AgentConfig`, `InstallStatus`, `ServiceInfo`, `DiagnosticResult`

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
- Key services: `installer.rs`, `offline_installer.rs`, `process_manager.rs`, `config_manager.rs`, `secure_storage.rs`

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
Long-running operations (installation) emit progress events:
```rust
// Backend
window.emit("install-progress", json!({ "stage": "Downloading", "percentage": 50, "message": "..." }))

// Frontend
openclawApi.onInstallProgress((progress) => { ... })
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
