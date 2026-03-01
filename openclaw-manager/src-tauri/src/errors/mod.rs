//! 错误处理模块
//!
//! 提供统一的错误类型和错误处理机制

pub mod app_error;

pub use app_error::{
    AppError, ApiResponse, ConfigError, ErrorSeverity, InstallError, NetworkError,
    ProcessError, SecureStorageError, UserErrorMessage,
};
