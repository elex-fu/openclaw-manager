use serde::Serialize;
use thiserror::Error;

/// 应用错误类型
#[derive(Error, Debug)]
pub enum AppError {
    /// 安装错误
    #[error("Installation failed: {0}")]
    Install(#[from] InstallError),

    /// 配置错误
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// 网络错误
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    /// 安全存储错误
    #[error("Secure storage error: {0}")]
    SecureStorage(#[from] SecureStorageError),

    /// 进程错误
    #[error("Process error: {0}")]
    Process(#[from] ProcessError),

    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 序列化错误
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// 未知错误
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// 安装错误
#[derive(Error, Debug, Clone)]
pub enum InstallError {
    #[error("Unsupported platform")]
    UnsupportedPlatform,
    
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    
    #[error("Checksum verification failed")]
    ChecksumFailed,
    
    #[error("Extraction failed: {0}")]
    ExtractionFailed(String),
    
    #[error("Installation directory not writable: {0}")]
    DirectoryNotWritable(String),
    
    #[error("Version {0} not found")]
    VersionNotFound(String),
    
    #[error("Installation script failed: {0}")]
    ScriptFailed(String),
    
    #[error("Package not found: {0}")]
    PackageNotFound(String),
}

/// 配置错误
#[derive(Error, Debug, Clone)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),
    
    #[error("Configuration version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: u32 },
    
    #[error("Configuration is locked by another process")]
    Locked,
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Failed to import configuration: {0}")]
    ImportFailed(String),
    
    #[error("Failed to export configuration: {0}")]
    ExportFailed(String),
}

/// 网络错误
#[derive(Error, Debug, Clone)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Timeout after {0} seconds")]
    Timeout(u64),
    
    #[error("DNS resolution failed: {0}")]
    DnsFailed(String),
    
    #[error("TLS handshake failed: {0}")]
    TlsFailed(String),
    
    #[error("HTTP error {code}: {message}")]
    HttpError { code: u16, message: String },
    
    #[error("All mirrors failed")]
    AllMirrorsFailed,
}

/// 安全存储错误
#[derive(Error, Debug, Clone)]
pub enum SecureStorageError {
    #[error("Keyring error: {0}")]
    Keyring(String),
    
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    #[error("Access denied to secure storage")]
    AccessDenied,
}

/// 进程错误
#[derive(Error, Debug, Clone)]
pub enum ProcessError {
    #[error("Service '{0}' already running")]
    AlreadyRunning(String),
    
    #[error("Service '{0}' not found")]
    NotFound(String),
    
    #[error("Failed to start service: {0}")]
    StartFailed(String),
    
    #[error("Failed to stop service: {0}")]
    StopFailed(String),
    
    #[error("Port {0} is already in use")]
    PortInUse(u16),
    
    #[error("Process crashed with exit code: {0}")]
    Crashed(i32),
    
    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),
    
    #[error("Service timeout after {0} seconds")]
    Timeout(u64),
}

/// 错误严重程度
#[derive(Debug, Clone, Serialize)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// 用户友好的错误信息
#[derive(Debug, Clone, Serialize)]
pub struct UserErrorMessage {
    pub title: String,
    pub description: String,
    pub action: Option<String>,
    pub severity: ErrorSeverity,
    pub retryable: bool,
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl AppError {
    /// 转换为前端友好的错误信息
    pub fn to_user_message(&self) -> UserErrorMessage {
        match self {
            AppError::Install(e) => match e {
                InstallError::UnsupportedPlatform => UserErrorMessage {
                    title: "不支持的系统".to_string(),
                    description: "当前操作系统不支持 OpenClaw 安装".to_string(),
                    action: Some("请使用 macOS、Linux 或 Windows 系统".to_string()),
                    severity: ErrorSeverity::Error,
                    retryable: false,
                },
                InstallError::DownloadFailed(_) => UserErrorMessage {
                    title: "下载失败".to_string(),
                    description: e.to_string(),
                    action: Some("检查网络连接或尝试切换镜像源".to_string()),
                    severity: ErrorSeverity::Warning,
                    retryable: true,
                },
                InstallError::ChecksumFailed => UserErrorMessage {
                    title: "文件校验失败".to_string(),
                    description: "下载的文件可能已损坏".to_string(),
                    action: Some("请重新尝试下载或更换镜像源".to_string()),
                    severity: ErrorSeverity::Error,
                    retryable: true,
                },
                _ => UserErrorMessage {
                    title: "安装失败".to_string(),
                    description: e.to_string(),
                    action: Some("检查网络连接或尝试离线安装".to_string()),
                    severity: ErrorSeverity::Error,
                    retryable: true,
                },
            },
            AppError::Config(e) => match e {
                ConfigError::VersionMismatch { .. } => UserErrorMessage {
                    title: "配置版本不兼容".to_string(),
                    description: e.to_string(),
                    action: Some("请重新配置或更新应用".to_string()),
                    severity: ErrorSeverity::Warning,
                    retryable: false,
                },
                ConfigError::InvalidFormat(_) => UserErrorMessage {
                    title: "配置文件格式错误".to_string(),
                    description: e.to_string(),
                    action: Some("检查配置文件语法".to_string()),
                    severity: ErrorSeverity::Warning,
                    retryable: false,
                },
                _ => UserErrorMessage {
                    title: "配置错误".to_string(),
                    description: e.to_string(),
                    action: Some("检查配置文件格式".to_string()),
                    severity: ErrorSeverity::Warning,
                    retryable: false,
                },
            },
            AppError::Network(e) => match e {
                NetworkError::Timeout(_) => UserErrorMessage {
                    title: "连接超时".to_string(),
                    description: "服务器响应时间过长".to_string(),
                    action: Some("请稍后重试或检查网络连接".to_string()),
                    severity: ErrorSeverity::Warning,
                    retryable: true,
                },
                NetworkError::DnsFailed(_) => UserErrorMessage {
                    title: "DNS 解析失败".to_string(),
                    description: "无法解析服务器地址".to_string(),
                    action: Some("检查网络设置或 DNS 配置".to_string()),
                    severity: ErrorSeverity::Warning,
                    retryable: true,
                },
                _ => UserErrorMessage {
                    title: "网络连接失败".to_string(),
                    description: "无法连接到远程服务器".to_string(),
                    action: Some("检查网络设置或切换镜像源".to_string()),
                    severity: ErrorSeverity::Warning,
                    retryable: true,
                },
            },
            AppError::SecureStorage(e) => match e {
                SecureStorageError::AccessDenied => UserErrorMessage {
                    title: "密钥访问被拒绝".to_string(),
                    description: "无法访问系统密钥链".to_string(),
                    action: Some("请检查系统权限设置".to_string()),
                    severity: ErrorSeverity::Error,
                    retryable: false,
                },
                _ => UserErrorMessage {
                    title: "安全存储错误".to_string(),
                    description: e.to_string(),
                    action: Some("请检查系统密钥链访问权限".to_string()),
                    severity: ErrorSeverity::Error,
                    retryable: false,
                },
            },
            AppError::Process(e) => match e {
                ProcessError::AlreadyRunning(_) => UserErrorMessage {
                    title: "服务已在运行".to_string(),
                    description: e.to_string(),
                    action: Some("请等待当前操作完成或重启应用".to_string()),
                    severity: ErrorSeverity::Info,
                    retryable: false,
                },
                ProcessError::PortInUse(port) => UserErrorMessage {
                    title: "端口被占用".to_string(),
                    description: format!("端口 {} 已被其他程序占用", port),
                    action: Some("请关闭占用该端口的程序或修改配置".to_string()),
                    severity: ErrorSeverity::Warning,
                    retryable: true,
                },
                _ => UserErrorMessage {
                    title: "服务操作失败".to_string(),
                    description: e.to_string(),
                    action: Some("请稍后重试".to_string()),
                    severity: ErrorSeverity::Error,
                    retryable: true,
                },
            },
            AppError::Io(e) => UserErrorMessage {
                title: "文件操作失败".to_string(),
                description: e.to_string(),
                action: Some("检查文件权限或磁盘空间".to_string()),
                severity: ErrorSeverity::Error,
                retryable: true,
            },
            AppError::Serialization(e) => UserErrorMessage {
                title: "数据解析失败".to_string(),
                description: e.clone(),
                action: Some("检查数据格式".to_string()),
                severity: ErrorSeverity::Warning,
                retryable: false,
            },
            AppError::Unknown(e) => UserErrorMessage {
                title: "操作失败".to_string(),
                description: e.clone(),
                action: Some("请稍后重试，如果问题持续请反馈".to_string()),
                severity: ErrorSeverity::Error,
                retryable: true,
            },
        }
    }

    /// 是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(self.to_user_message().retryable, true)
    }
}

/// 统一的 API 响应结构
#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<UserErrorMessage>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(err: AppError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(err.to_user_message()),
        }
    }

    pub fn from_result(result: Result<T, AppError>) -> Self {
        match result {
            Ok(data) => Self::success(data),
            Err(err) => Self::error(err),
        }
    }
}

impl<T> From<Result<T, AppError>> for ApiResponse<T> {
    fn from(result: Result<T, AppError>) -> Self {
        Self::from_result(result)
    }
}
