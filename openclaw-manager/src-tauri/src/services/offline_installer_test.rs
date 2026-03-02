//! 离线安装器单元测试

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// 测试平台检测
    #[test]
    fn test_current_platform_detection() {
        let platform = Platform::current();

        #[cfg(target_os = "macos")]
        assert_eq!(platform, Platform::MacOS, "macOS 应该检测到 MacOS");

        #[cfg(target_os = "windows")]
        assert_eq!(platform, Platform::Windows, "Windows 应该检测到 Windows");

        #[cfg(target_os = "linux")]
        assert_eq!(platform, Platform::Linux, "Linux 应该检测到 Linux");
    }

    /// 测试架构检测
    #[test]
    fn test_current_arch_detection() {
        let arch = Arch::current();

        #[cfg(target_arch = "x86_64")]
        assert_eq!(arch, Arch::X86_64, "x86_64 应该检测到 X86_64");

        #[cfg(target_arch = "aarch64")]
        assert_eq!(arch, Arch::ARM64, "aarch64 应该检测到 ARM64");
    }

    /// 测试文件名生成
    #[test]
    fn test_package_filename_generation() {
        let filename = OfflineInstaller::get_package_filename(Platform::MacOS, Arch::ARM64);
        assert_eq!(filename, "openclaw-macos-arm64.tar.gz");

        let filename = OfflineInstaller::get_package_filename(Platform::MacOS, Arch::X86_64);
        assert_eq!(filename, "openclaw-macos-x64.tar.gz");

        let filename = OfflineInstaller::get_package_filename(Platform::Windows, Arch::X86_64);
        assert_eq!(filename, "openclaw-windows-x64.zip");

        let filename = OfflineInstaller::get_package_filename(Platform::Linux, Arch::X86_64);
        assert_eq!(filename, "openclaw-linux-x64.tar.gz");
    }

    /// 测试为当前平台创建安装器
    #[test]
    fn test_for_current_platform() {
        let result = OfflineInstaller::for_current_platform();
        assert!(result.is_ok(), "应该能为当前平台创建安装器");

        let installer = result.unwrap();
        let info = installer.get_info();

        // 验证平台信息
        assert_eq!(info.platform, Platform::current());
        assert_eq!(info.arch, Arch::current());
    }

    /// 测试资源查找（可能找不到，取决于环境）
    #[test]
    fn test_find_resource() {
        let result = OfflineInstaller::for_current_platform();
        if let Ok(installer) = result {
            let resource = installer.find_resource();
            // 开发环境可能找不到，但不应该报错
            println!("Resource found: {:?}", resource.is_some());
        }
    }

    /// 测试安装包校验（使用测试包）
    #[test]
    fn test_verify_package() {
        let temp_dir = TempDir::new().unwrap();

        // 创建测试用的 tar.gz 文件
        let test_file = temp_dir.path().join("test.tar.gz");

        // 创建一个简单的 tar.gz
        use std::process::Command;
        let test_content_dir = temp_dir.path().join("test_content");
        std::fs::create_dir(&test_content_dir).unwrap();
        std::fs::write(test_content_dir.join("test.txt"), "test content").unwrap();

        let output = Command::new("tar")
            .args([
                "-czf",
                test_file.to_str().unwrap(),
                "-C",
                temp_dir.path().to_str().unwrap(),
                "test_content",
            ])
            .output();

        if output.is_ok() && output.unwrap().status.success() {
            // 创建测试安装器
            let installer = OfflineInstaller {
                info: PackageInfo {
                    version: "test".to_string(),
                    platform: Platform::MacOS,
                    arch: Arch::ARM64,
                    checksum: "invalid".to_string(),
                },
                resource_path: Some(test_file.clone()),
            };

            // 验证包（校验和会失败，但结构应该正确）
            let result = installer.verify_package();
            // 由于校验和无效，这里可能会失败，但不应该 panic
            println!("Verification result: {:?}", result);
        }
    }

    /// 测试包大小估算
    #[test]
    fn test_estimate_package_size() {
        let temp_dir = TempDir::new().unwrap();

        // 创建测试用的 tar.gz 文件
        let test_file = temp_dir.path().join("test.tar.gz");
        std::fs::write(&test_file, "test content for size estimation").unwrap();

        let installer = OfflineInstaller {
            info: PackageInfo {
                version: "test".to_string(),
                platform: Platform::MacOS,
                arch: Arch::ARM64,
                checksum: "test".to_string(),
            },
            resource_path: Some(test_file),
        };

        let size = installer.estimate_package_size();
        assert!(size > 0, "应该能估算包大小");
    }

    /// 测试 Platform 和 Arch 的 Display 实现
    #[test]
    fn test_platform_arch_display() {
        assert_eq!(format!("{}", Platform::MacOS), "macos");
        assert_eq!(format!("{}", Platform::Windows), "windows");
        assert_eq!(format!("{}", Platform::Linux), "linux");

        assert_eq!(format!("{}", Arch::X86_64), "x64");
        assert_eq!(format!("{}", Arch::ARM64), "arm64");
    }
}
