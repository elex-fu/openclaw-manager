/// Security sandbox for plugin execution
pub struct Sandbox {
    allowed_apis: Vec<String>,
    memory_limit: usize,
    timeout_ms: u64,
}

impl Sandbox {
    pub fn new() -> Self {
        Self {
            allowed_apis: vec![
                "config".to_string(),
                "file".to_string(),
                "group".to_string(),
                "notification".to_string(),
                "ui".to_string(),
            ],
            memory_limit: 64 * 1024 * 1024, // 64MB
            timeout_ms: 5000,                // 5 seconds
        }
    }

    pub fn with_memory_limit(mut self, limit: usize) -> Self {
        self.memory_limit = limit;
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn allow_api(mut self, api: &str) -> Self {
        if !self.allowed_apis.contains(&api.to_string()) {
            self.allowed_apis.push(api.to_string());
        }
        self
    }

    pub fn deny_api(mut self, api: &str) -> Self {
        self.allowed_apis.retain(|a| a != api);
        self
    }

    pub fn is_api_allowed(&self, api: &str) -> bool {
        self.allowed_apis.contains(&api.to_string())
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Security policy for plugin execution
pub struct SecurityPolicy {
    pub allow_file_read: bool,
    pub allow_file_write: bool,
    pub allow_network: bool,
    pub allow_shell: bool,
    pub allowed_paths: Vec<String>,
}

impl SecurityPolicy {
    pub fn restrictive() -> Self {
        Self {
            allow_file_read: false,
            allow_file_write: false,
            allow_network: false,
            allow_shell: false,
            allowed_paths: Vec::new(),
        }
    }

    pub fn standard() -> Self {
        Self {
            allow_file_read: true,
            allow_file_write: false,
            allow_network: false,
            allow_shell: false,
            allowed_paths: Vec::new(),
        }
    }

    pub fn permissive() -> Self {
        Self {
            allow_file_read: true,
            allow_file_write: true,
            allow_network: true,
            allow_shell: false,
            allowed_paths: Vec::new(),
        }
    }
}
