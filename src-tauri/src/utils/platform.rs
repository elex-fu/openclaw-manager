//! Platform-specific utilities for OpenClaw Manager
//!
//! This module provides cross-platform path resolution and platform detection
//! utilities for the OpenClaw installation and management system.

use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur when resolving platform-specific paths
#[derive(Error, Debug)]
pub enum PlatformError {
    /// Home directory could not be determined
    #[error("Home directory not found")]
    HomeDirNotFound,

    /// Local app data directory could not be determined (Windows-specific)
    #[error("Local app data directory not found")]
    LocalAppDataNotFound,

    /// The current platform is not supported
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    /// Failed to create directory
    #[error("Failed to create directory: {0}")]
    DirectoryCreationFailed(String),

    /// Path is not valid UTF-8
    #[error("Path contains invalid characters")]
    InvalidPath,
}

/// Type of installation directory
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallDirType {
    /// User-level installation (no admin required)
    User,
    /// System-level installation (requires admin)
    System,
}

/// Platform-specific installation path utilities
pub struct InstallPaths;

impl InstallPaths {
    /// Get the default installation directory for the current platform
    ///
    /// This returns a user-level directory that does not require administrator
    /// privileges to write to.
    ///
    /// # Platform-specific paths:
    /// - **macOS**: `~/Applications/OpenClaw`
    /// - **Windows**: `%LOCALAPPDATA%/Programs/OpenClaw`
    /// - **Linux**: `~/.local/bin/openclaw`
    ///
    /// # Returns
    /// - `Ok(PathBuf)` - The installation directory path
    /// - `Err(PlatformError)` - If the home directory cannot be determined
    ///
    /// # Examples
    /// ```rust
    /// use openclaw_manager::utils::platform::InstallPaths;
    ///
    /// match InstallPaths::default_install_dir() {
    ///     Ok(path) => println!("Install to: {:?}", path),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    pub fn default_install_dir() -> Result<PathBuf, PlatformError> {
        match std::env::consts::OS {
            "macos" => {
                let home = dirs::home_dir().ok_or(PlatformError::HomeDirNotFound)?;
                Ok(home.join("Applications").join("OpenClaw"))
            }
            "windows" => {
                let local_app_data =
                    dirs::data_local_dir().ok_or(PlatformError::LocalAppDataNotFound)?;
                Ok(local_app_data.join("Programs").join("OpenClaw"))
            }
            "linux" => {
                let home = dirs::home_dir().ok_or(PlatformError::HomeDirNotFound)?;
                Ok(home.join(".local").join("bin").join("openclaw"))
            }
            _ => Err(PlatformError::UnsupportedPlatform(
                std::env::consts::OS.to_string(),
            )),
        }
    }

    /// Get the system-level installation directory for the current platform
    ///
    /// This returns a system-wide directory that typically requires administrator
    /// privileges to write to.
    ///
    /// # Platform-specific paths:
    /// - **macOS**: `/Applications/OpenClaw`
    /// - **Windows**: `C:\Program Files\OpenClaw`
    /// - **Linux**: `/opt/openclaw`
    ///
    /// # Returns
    /// - `Ok(PathBuf)` - The system installation directory path
    /// - `Err(PlatformError)` - If the platform is not supported
    pub fn system_install_dir() -> Result<PathBuf, PlatformError> {
        match std::env::consts::OS {
            "macos" => Ok(PathBuf::from("/Applications/OpenClaw")),
            "windows" => Ok(PathBuf::from("C:\\Program Files\\OpenClaw")),
            "linux" => Ok(PathBuf::from("/opt/openclaw")),
            _ => Err(PlatformError::UnsupportedPlatform(
                std::env::consts::OS.to_string(),
            )),
        }
    }

    /// Get the appropriate installation directory based on the install type
    ///
    /// # Arguments
    /// * `install_type` - The type of installation (User or System)
    ///
    /// # Returns
    /// - `Ok(PathBuf)` - The installation directory path
    /// - `Err(PlatformError)` - If the directory cannot be determined
    pub fn get_install_dir(install_type: InstallDirType) -> Result<PathBuf, PlatformError> {
        match install_type {
            InstallDirType::User => Self::default_install_dir(),
            InstallDirType::System => Self::system_install_dir(),
        }
    }

    /// Get the configuration directory for OpenClaw
    ///
    /// # Platform-specific paths:
    /// - **macOS**: `~/Library/Application Support/OpenClaw`
    /// - **Windows**: `%APPDATA%/OpenClaw`
    /// - **Linux**: `~/.config/openclaw`
    pub fn config_dir() -> Result<PathBuf, PlatformError> {
        dirs::config_dir()
            .map(|d| d.join("OpenClaw"))
            .ok_or(PlatformError::HomeDirNotFound)
    }

    /// Get the data directory for OpenClaw
    ///
    /// # Platform-specific paths:
    /// - **macOS**: `~/Library/Application Support/OpenClaw`
    /// - **Windows**: `%LOCALAPPDATA%/OpenClaw`
    /// - **Linux**: `~/.local/share/openclaw`
    pub fn data_dir() -> Result<PathBuf, PlatformError> {
        dirs::data_dir()
            .map(|d| d.join("OpenClaw"))
            .ok_or(PlatformError::HomeDirNotFound)
    }

    /// Get the cache directory for OpenClaw
    ///
    /// # Platform-specific paths:
    /// - **macOS**: `~/Library/Caches/OpenClaw`
    /// - **Windows**: `%LOCALAPPDATA%/OpenClaw/Cache`
    /// - **Linux**: `~/.cache/openclaw`
    pub fn cache_dir() -> Result<PathBuf, PlatformError> {
        dirs::cache_dir()
            .map(|d| d.join("OpenClaw"))
            .ok_or(PlatformError::HomeDirNotFound)
    }

    /// Get the logs directory for OpenClaw
    ///
    /// # Platform-specific paths:
    /// - **macOS**: `~/Library/Logs/OpenClaw`
    /// - **Windows**: `%LOCALAPPDATA%/OpenClaw/Logs`
    /// - **Linux**: `~/.local/share/openclaw/logs`
    pub fn logs_dir() -> Result<PathBuf, PlatformError> {
        match std::env::consts::OS {
            "macos" => dirs::home_dir()
                .map(|h| h.join("Library").join("Logs").join("OpenClaw"))
                .ok_or(PlatformError::HomeDirNotFound),
            "windows" => dirs::data_local_dir()
                .map(|d| d.join("OpenClaw").join("Logs"))
                .ok_or(PlatformError::LocalAppDataNotFound),
            "linux" => dirs::data_dir()
                .map(|d| d.join("openclaw").join("logs"))
                .ok_or(PlatformError::HomeDirNotFound),
            _ => Err(PlatformError::UnsupportedPlatform(
                std::env::consts::OS.to_string(),
            )),
        }
    }

    /// Ensure that the installation directory exists, creating it if necessary
    ///
    /// # Arguments
    /// * `install_type` - The type of installation (User or System)
    ///
    /// # Returns
    /// - `Ok(PathBuf)` - The installation directory path (which now exists)
    /// - `Err(PlatformError)` - If the directory cannot be created
    pub fn ensure_install_dir(install_type: InstallDirType) -> Result<PathBuf, PlatformError> {
        let path = Self::get_install_dir(install_type)?;

        if !path.exists() {
            std::fs::create_dir_all(&path).map_err(|e| {
                PlatformError::DirectoryCreationFailed(format!(
                    "Failed to create {:?}: {}",
                    path, e
                ))
            })?;
        }

        Ok(path)
    }

    /// Get the executable name for the current platform
    ///
    /// Returns "openclaw" on Unix systems and "openclaw.exe" on Windows
    pub fn executable_name() -> &'static str {
        if std::env::consts::OS == "windows" {
            "openclaw.exe"
        } else {
            "openclaw"
        }
    }

    /// Get the library extension for the current platform
    ///
    /// Returns "dylib" on macOS, "dll" on Windows, and "so" on Linux
    pub fn library_extension() -> &'static str {
        match std::env::consts::OS {
            "macos" => "dylib",
            "windows" => "dll",
            _ => "so",
        }
    }

    /// Check if the current platform is supported
    pub fn is_supported() -> bool {
        matches!(
            std::env::consts::OS,
            "macos" | "windows" | "linux"
        )
    }

    /// Get the current platform identifier
    ///
    /// Returns a string like "macos-arm64", "windows-x64", "linux-x64", etc.
    pub fn current_platform() -> String {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        // Normalize architecture names
        let arch_normalized = match arch {
            "aarch64" => "arm64",
            "x86_64" => "x64",
            "x86" => "x86",
            _ => arch,
        };

        format!("{}-{}", os, arch_normalized)
    }
}

/// Platform information utilities
pub struct PlatformInfo;

impl PlatformInfo {
    /// Get the current operating system name
    pub fn os() -> &'static str {
        std::env::consts::OS
    }

    /// Get the current architecture
    pub fn arch() -> &'static str {
        std::env::consts::ARCH
    }

    /// Get the family of the current OS (e.g., "unix", "windows")
    pub fn family() -> &'static str {
        std::env::consts::FAMILY
    }

    /// Check if running on Windows
    pub fn is_windows() -> bool {
        std::env::consts::OS == "windows"
    }

    /// Check if running on macOS
    pub fn is_macos() -> bool {
        std::env::consts::OS == "macos"
    }

    /// Check if running on Linux
    pub fn is_linux() -> bool {
        std::env::consts::OS == "linux"
    }

    /// Check if running on a Unix-like system
    pub fn is_unix() -> bool {
        std::env::consts::FAMILY == "unix"
    }

    /// Check if the current architecture is ARM64
    pub fn is_arm64() -> bool {
        matches!(std::env::consts::ARCH, "aarch64" | "arm64")
    }

    /// Check if the current architecture is x86_64
    pub fn is_x64() -> bool {
        std::env::consts::ARCH == "x86_64"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_install_dir() {
        let result = InstallPaths::default_install_dir();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("OpenClaw"));
    }

    #[test]
    fn test_system_install_dir() {
        let result = InstallPaths::system_install_dir();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("OpenClaw"));
    }

    #[test]
    fn test_config_dir() {
        let result = InstallPaths::config_dir();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("OpenClaw"));
    }

    #[test]
    fn test_data_dir() {
        let result = InstallPaths::data_dir();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("OpenClaw"));
    }

    #[test]
    fn test_cache_dir() {
        let result = InstallPaths::cache_dir();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("OpenClaw"));
    }

    #[test]
    fn test_logs_dir() {
        let result = InstallPaths::logs_dir();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("OpenClaw")
            || path.to_string_lossy().contains("openclaw"));
    }

    #[test]
    fn test_executable_name() {
        let name = InstallPaths::executable_name();
        if PlatformInfo::is_windows() {
            assert_eq!(name, "openclaw.exe");
        } else {
            assert_eq!(name, "openclaw");
        }
    }

    #[test]
    fn test_library_extension() {
        let ext = InstallPaths::library_extension();
        if PlatformInfo::is_macos() {
            assert_eq!(ext, "dylib");
        } else if PlatformInfo::is_windows() {
            assert_eq!(ext, "dll");
        } else {
            assert_eq!(ext, "so");
        }
    }

    #[test]
    fn test_is_supported() {
        assert!(InstallPaths::is_supported());
    }

    #[test]
    fn test_current_platform() {
        let platform = InstallPaths::current_platform();
        assert!(!platform.is_empty());
        assert!(platform.contains('-'));
    }

    #[test]
    fn test_platform_info() {
        assert_eq!(PlatformInfo::os(), std::env::consts::OS);
        assert_eq!(PlatformInfo::arch(), std::env::consts::ARCH);
        assert_eq!(PlatformInfo::family(), std::env::consts::FAMILY);
    }

    #[test]
    fn test_install_dir_type() {
        assert_eq!(InstallDirType::User, InstallDirType::User);
        assert_eq!(InstallDirType::System, InstallDirType::System);
        assert_ne!(InstallDirType::User, InstallDirType::System);
    }
}
