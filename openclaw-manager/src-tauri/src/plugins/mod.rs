pub mod loader;
pub mod runtime;
pub mod sandbox;

use crate::models::plugin::Plugin;
use std::collections::HashMap;

pub struct PluginManager {
    plugins: HashMap<String, LoadedPlugin>,
}

pub struct LoadedPlugin {
    pub plugin: Plugin,
    pub runtime: PluginRuntime,
}

pub enum PluginRuntime {
    Lua,
    JavaScript,
    Wasm,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn load_plugin(&mut self, plugin: Plugin) -> anyhow::Result<()> {
        // TODO: Implement actual plugin loading based on type
        let runtime = match plugin.plugin_type.as_str() {
            "lua" => PluginRuntime::Lua,
            "javascript" | "js" => PluginRuntime::JavaScript,
            "wasm" => PluginRuntime::Wasm,
            _ => anyhow::bail!("Unsupported plugin type: {}", plugin.plugin_type),
        };

        self.plugins.insert(
            plugin.id.clone(),
            LoadedPlugin { plugin, runtime },
        );

        Ok(())
    }

    pub fn unload_plugin(&mut self, plugin_id: &str) {
        self.plugins.remove(plugin_id);
    }

    pub fn get_plugin(&self, plugin_id: &str) -> Option<&LoadedPlugin> {
        self.plugins.get(plugin_id)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
