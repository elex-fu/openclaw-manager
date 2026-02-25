/// Plugin runtime interface
pub trait Runtime {
    fn initialize(&mut self) -> anyhow::Result<()>;
    fn execute(&self, code: &str) -> anyhow::Result<serde_json::Value>;
    fn shutdown(&mut self) -> anyhow::Result<()>;
}

/// Lua runtime for plugins
pub struct LuaRuntime;

impl LuaRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LuaRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// JavaScript runtime for plugins (using quickjs or similar)
pub struct JsRuntime;

impl JsRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// WASM runtime for plugins
pub struct WasmRuntime;

impl WasmRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new()
    }
}
