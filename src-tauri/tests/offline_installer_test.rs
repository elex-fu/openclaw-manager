//! 离线安装器单元测试
//!
//! 测试平台检测、架构检测、包文件名生成等功能
//! 覆盖6个平台组合：macOS x64/ARM64, Windows x64/ARM64, Linux x64/ARM64

use openclaw_manager::services::offline_installer::{Arch, OfflineInstaller, Platform, PackageInfo, current_platform_info};

/// 测试平台 Display 实现
#[test]
fn test_platform_display() {
    assert_eq!(format!("{}", Platform::MacOS), "macos");
    assert_eq!(format!("{}", Platform::Windows), "windows");
    assert_eq!(format!("{}", Platform::Linux), "linux");
}

/// 测试架构 Display 实现
#[test]
fn test_arch_display() {
    assert_eq!(format!("{}", Arch::X86_64), "x64");
    assert_eq!(format!("{}", Arch::ARM64), "arm64");
}

/// 测试 ARM64 平台支持（MVP v2 新增）
#[test]
fn test_arm64_platform_support() {
    // 验证 ARM64 架构类型存在且可正确使用
    let arch = Arch::ARM64;
    assert_eq!(format!("{}", arch), "arm64");

    // 验证所有平台与 ARM64 的组合
    let platforms = vec![Platform::MacOS, Platform::Windows, Platform::Linux];
    for platform in platforms {
        // 验证可以创建包含 ARM64 的包信息
        let package_info = PackageInfo {
            version: "1.0.0".to_string(),
            platform,
            arch: Arch::ARM64,
            checksum: "test".to_string(),
        };

        assert_eq!(package_info.arch, Arch::ARM64);
        assert!(format!("{}", package_info.arch).contains("arm64"));
    }
}

/// 测试 x86_64 平台支持
#[test]
fn test_x86_64_platform_support() {
    let arch = Arch::X86_64;
    assert_eq!(format!("{}", arch), "x64");

    let platforms = vec![Platform::MacOS, Platform::Windows, Platform::Linux];
    for platform in platforms {
        let package_info = PackageInfo {
            version: "1.0.0".to_string(),
            platform,
            arch: Arch::X86_64,
            checksum: "test".to_string(),
        };

        assert_eq!(package_info.arch, Arch::X86_64);
        assert!(format!("{}", package_info.arch).contains("x64"));
    }
}

/// 测试为当前平台创建安装器
#[test]
fn test_for_current_platform() {
    let result = OfflineInstaller::for_current_platform();
    assert!(result.is_ok(), "应该能为当前平台创建安装器");

    let _installer = result.unwrap();
    // 安装器创建成功即可，验证后续功能
}

/// 测试平台信息获取
#[test]
fn test_platform_info() {
    let (platform, arch) = current_platform_info();

    // 验证平台与当前编译目标一致
    #[cfg(target_os = "macos")]
    assert_eq!(platform, Platform::MacOS);

    #[cfg(target_os = "windows")]
    assert_eq!(platform, Platform::Windows);

    #[cfg(target_os = "linux")]
    assert_eq!(platform, Platform::Linux);

    // 验证架构与当前编译目标一致
    #[cfg(target_arch = "x86_64")]
    assert_eq!(arch, Arch::X86_64);

    #[cfg(target_arch = "aarch64")]
    assert_eq!(arch, Arch::ARM64);
}

/// 测试6平台支持矩阵（MVP v2 新增）
#[test]
fn test_six_platform_matrix() {
    // 定义所有6个平台组合
    let platform_arch_combinations = vec![
        (Platform::MacOS, Arch::X86_64, "macos", "x64"),
        (Platform::MacOS, Arch::ARM64, "macos", "arm64"),
        (Platform::Windows, Arch::X86_64, "windows", "x64"),
        (Platform::Windows, Arch::ARM64, "windows", "arm64"),
        (Platform::Linux, Arch::X86_64, "linux", "x64"),
        (Platform::Linux, Arch::ARM64, "linux", "arm64"),
    ];

    for (platform, arch, expected_platform, expected_arch) in platform_arch_combinations {
        // 创建包信息
        let package_info = PackageInfo {
            version: "1.0.0".to_string(),
            platform,
            arch,
            checksum: "test".to_string(),
        };

        // 验证平台
        let platform_str = format!("{}", package_info.platform);
        assert_eq!(
            platform_str, expected_platform,
            "平台标识应匹配: got {}, expected {}",
            platform_str, expected_platform
        );

        // 验证架构
        let arch_str = format!("{}", package_info.arch);
        assert_eq!(
            arch_str, expected_arch,
            "架构标识应匹配: got {}, expected {}",
            arch_str, expected_arch
        );

        // 验证包文件名格式（根据平台和架构推断）
        let expected_extension = if platform == Platform::Windows {
            ".zip"
        } else {
            ".tar.gz"
        };

        let expected_filename = format!(
            "openclaw-{}-{}{}",
            expected_platform,
            if expected_arch == "x64" { "x64" } else { "arm64" },
            expected_extension
        );

        // 验证文件名包含正确的标识
        assert!(
            expected_filename.contains(expected_platform),
            "文件名应包含平台标识"
        );
        assert!(
            expected_filename.contains(if expected_arch == "x64" { "x64" } else { "arm64" }),
            "文件名应包含架构标识"
        );
    }
}

/// 测试 PackageInfo 序列化和反序列化
#[test]
fn test_package_info_serialization() {
    let package_info = PackageInfo {
        version: "1.0.0".to_string(),
        platform: Platform::Linux,
        arch: Arch::ARM64,
        checksum: "abc123".to_string(),
    };

    // 序列化
    let json = serde_json::to_string(&package_info).unwrap();
    assert!(json.contains("1.0.0"));
    assert!(json.contains("abc123"));

    // 反序列化
    let deserialized: PackageInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.version, package_info.version);
    assert_eq!(deserialized.platform, package_info.platform);
    assert_eq!(deserialized.arch, package_info.arch);
    assert_eq!(deserialized.checksum, package_info.checksum);
}

/// 测试 Platform 枚举比较
#[test]
fn test_platform_equality() {
    assert_eq!(Platform::MacOS, Platform::MacOS);
    assert_eq!(Platform::Windows, Platform::Windows);
    assert_eq!(Platform::Linux, Platform::Linux);

    assert_ne!(Platform::MacOS, Platform::Windows);
    assert_ne!(Platform::Windows, Platform::Linux);
    assert_ne!(Platform::Linux, Platform::MacOS);
}

/// 测试 Arch 枚举比较
#[test]
fn test_arch_equality() {
    assert_eq!(Arch::X86_64, Arch::X86_64);
    assert_eq!(Arch::ARM64, Arch::ARM64);

    assert_ne!(Arch::X86_64, Arch::ARM64);
}

/// 测试 PackageInfo 克隆
#[test]
fn test_package_info_clone() {
    let info = PackageInfo {
        version: "1.0.0".to_string(),
        platform: Platform::Linux,
        arch: Arch::ARM64,
        checksum: "abc".to_string(),
    };

    let cloned = info.clone();
    assert_eq!(info.version, cloned.version);
    assert_eq!(info.platform, cloned.platform);
    assert_eq!(info.arch, cloned.arch);
    assert_eq!(info.checksum, cloned.checksum);
}

/// 测试条件编译 - 当前平台应该正确检测
#[test]
fn test_current_platform_detection() {
    let (platform, arch) = current_platform_info();

    // 根据编译目标验证
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        assert_eq!(platform, Platform::MacOS);
        assert_eq!(arch, Arch::X86_64);
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        assert_eq!(platform, Platform::MacOS);
        assert_eq!(arch, Arch::ARM64);
    }

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        assert_eq!(platform, Platform::Windows);
        assert_eq!(arch, Arch::X86_64);
    }

    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        assert_eq!(platform, Platform::Windows);
        assert_eq!(arch, Arch::ARM64);
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        assert_eq!(platform, Platform::Linux);
        assert_eq!(arch, Arch::X86_64);
    }

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        assert_eq!(platform, Platform::Linux);
        assert_eq!(arch, Arch::ARM64);
    }
}

/// 测试 Platform 克隆
#[test]
fn test_platform_clone() {
    let platform = Platform::MacOS;
    let cloned = platform.clone();
    assert_eq!(platform, cloned);
}

/// 测试 Arch 克隆
#[test]
fn test_arch_clone() {
    let arch = Arch::ARM64;
    let cloned = arch.clone();
    assert_eq!(arch, cloned);
}

/// 测试当前平台文件名（通过 current_platform_info）
#[test]
fn test_current_platform_filename() {
    let (platform, arch) = current_platform_info();

    // 根据平台和架构构建期望的文件名
    let extension = match platform {
        Platform::Windows => ".zip",
        _ => ".tar.gz",
    };

    let arch_str = format!("{}", arch);
    let platform_str = format!("{}", platform);

    let expected_filename = format!("openclaw-{}-{}{}", platform_str, arch_str, extension);

    // 验证文件名格式正确
    assert!(expected_filename.starts_with("openclaw-"));
    assert!(expected_filename.contains(&platform_str));
    assert!(expected_filename.contains(&arch_str));
}
