//! OpenClawInstaller 单元测试

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// 测试 OpenClawInstaller 创建
    #[test]
    fn test_installer_creation() {
        let installer = OpenClawInstaller::new();
        assert!(installer.is_ok(), "应该能创建安装器");

        let installer = installer.unwrap();
        let install_dir = installer.get_install_dir();
        assert!(install_dir.to_string_lossy().contains(".openclaw"), "安装目录应该包含 .openclaw");
    }

    /// 测试安装状态检查（未安装情况）
    #[tokio::test]
    async fn test_check_installation_not_installed() {
        // 使用临时目录作为安装目录
        let temp_dir = TempDir::new().unwrap();
        let installer = OpenClawInstaller::with_install_dir(temp_dir.path().to_path_buf());

        let status = installer.check_installation();
        assert!(status.is_ok(), "检查应该成功");

        match status.unwrap() {
            InstallStatus::NotInstalled => {
                // 预期结果
            }
            _ => panic!("未安装时应返回 NotInstalled"),
        }
    }

    /// 测试安装目录创建
    #[test]
    fn test_ensure_install_dir() {
        let temp_dir = TempDir::new().unwrap();
        let installer = OpenClawInstaller::with_install_dir(temp_dir.path().join("openclaw"));

        let result = installer.ensure_install_dir();
        assert!(result.is_ok(), "应该能创建安装目录");

        // 验证目录存在
        assert!(temp_dir.path().join("openclaw").exists(), "目录应该存在");
        assert!(temp_dir.path().join("openclaw").join("bin").exists(), "bin 目录应该存在");
    }
}
