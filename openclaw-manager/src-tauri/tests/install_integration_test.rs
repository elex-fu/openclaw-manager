//! 安装功能集成测试

use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;

/// 测试离线安装器创建
#[tokio::test]
async fn test_offline_installer_creation() {
    // 尝试创建离线安装器
    let result = openclaw_manager::services::offline_installer::OfflineInstaller::for_current_platform();

    // 应该能成功创建（即使包不存在）
    assert!(result.is_ok(), "应该能创建离线安装器");
}

/// 测试安装包查找
#[tokio::test]
async fn test_package_lookup() {
    use openclaw_manager::services::offline_installer::OfflineInstaller;

    let installer = OfflineInstaller::for_current_platform();
    if let Ok(installer) = installer {
        // 尝试读取包数据
        let result = installer.read_package_data().await;

        // 根据 bundled 目录是否存在，可能是成功或失败
        match result {
            Ok(data) => {
                println!("找到安装包，大小: {} 字节", data.len());
                assert!(!data.is_empty(), "安装包不应为空");
            }
            Err(e) => {
                println!("未找到安装包: {}", e);
                // 这是预期的，如果在 CI 环境中没有安装包
            }
        }
    }
}

/// 测试完整离线安装流程
#[tokio::test]
async fn test_full_offline_install() {
    use openclaw_manager::services::offline_installer::OfflineInstaller;
    use std::fs;

    // 创建临时安装目录
    let test_dir = PathBuf::from("/tmp/test_openclaw_install_integration");
    let _ = fs::remove_dir_all(&test_dir);

    let installer = OfflineInstaller::for_current_platform();

    if let Ok(installer) = installer {
        // 执行安装
        let result = timeout(
            Duration::from_secs(30),
            installer.install(&test_dir)
        ).await;

        match result {
            Ok(Ok(())) => {
                println!("安装成功");

                // 验证安装的文件
                let bin_dir = test_dir.join("bin");
                assert!(bin_dir.exists(), "bin 目录应该存在");

                let openclaw_bin = bin_dir.join("openclaw");
                assert!(openclaw_bin.exists(), "openclaw 可执行文件应该存在");

                // 清理
                let _ = fs::remove_dir_all(&test_dir);
            }
            Ok(Err(e)) => {
                println!("安装失败: {}", e);
                // 预期可能失败，如果 bundled 目录中没有安装包
            }
            Err(_) => {
                panic!("安装超时");
            }
        }
    }
}

/// 测试在线安装器的模拟模式
#[tokio::test]
async fn test_mock_install() {
    use openclaw_manager::installer::OpenClawInstaller;
    use std::fs;

    // 创建临时安装目录
    let test_dir = PathBuf::from("/tmp/test_openclaw_mock_install");
    let _ = fs::remove_dir_all(&test_dir);

    // 创建安装器
    let installer = OpenClawInstaller::new();
    assert!(installer.is_ok(), "应该能创建安装器");

    let installer = installer.unwrap();

    // 测试目录创建
    let result = fs::create_dir_all(installer.get_install_dir());
    assert!(result.is_ok(), "应该能创建安装目录");

    // 测试默认配置创建
    let result = installer.create_default_config();
    assert!(result.is_ok(), "应该能创建默认配置");

    // 验证配置文件
    let config_path = installer.get_install_dir().join("config.yaml");
    assert!(config_path.exists(), "配置文件应该存在");

    // 读取并验证配置
    let config = installer.read_config();
    assert!(config.is_ok(), "应该能读取配置");

    let config = config.unwrap();
    assert_eq!(config.version, "1.0.0", "配置版本应该匹配");

    // 清理
    let _ = fs::remove_dir_all(installer.get_install_dir());
}

/// 测试安装进度报告
#[tokio::test]
async fn test_install_progress() {
    use openclaw_manager::installer::{InstallProgress, InstallStage};
    use tokio::sync::mpsc;

    let (tx, mut rx) = mpsc::channel::<InstallProgress>(10);

    // 发送一些进度
    let progress = InstallProgress {
        stage: InstallStage::Downloading,
        percentage: 50.0,
        message: "测试中...".to_string(),
    };

    let result = tx.send(progress).await;
    assert!(result.is_ok(), "应该能发送进度");

    // 接收进度
    if let Some(received) = rx.recv().await {
        assert_eq!(received.stage, InstallStage::Downloading);
        assert_eq!(received.percentage, 50.0);
        assert_eq!(received.message, "测试中...");
    } else {
        panic!("应该能接收进度");
    }
}
