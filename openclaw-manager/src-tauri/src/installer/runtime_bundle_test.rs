//! RuntimeBundle 单元测试

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// 测试 RuntimeBundle 创建
    #[test]
    fn test_runtime_bundle_creation() {
        let bundle = RuntimeBundle::new();
        assert!(bundle.is_ok(), "应该能创建 RuntimeBundle");
    }

    /// 测试 RuntimePackage 创建
    #[test]
    fn test_runtime_package_node() {
        let package = RuntimePackage::node22();
        assert_eq!(package.required, true, "Node.js 应该是必需的");
        assert!(package.archive_name.contains("node"), "包名应该包含 node");
    }

    #[test]
    fn test_runtime_package_python() {
        let package = RuntimePackage::python310();
        assert_eq!(package.required, true, "Python 应该是必需的");
        assert!(package.archive_name.contains("python"), "包名应该包含 python");
    }

    /// 测试路径获取
    #[test]
    fn test_runtime_paths() {
        let temp_dir = TempDir::new().unwrap();
        let bundle = RuntimeBundle::with_install_dir(temp_dir.path().to_path_buf());

        let node_path = bundle.get_runtime_path(RuntimeType::Node);
        assert!(node_path.to_string_lossy().contains("node"), "路径应该包含 node");

        let python_path = bundle.get_runtime_path(RuntimeType::Python);
        assert!(python_path.to_string_lossy().contains("python"), "路径应该包含 python");
    }

    /// 测试环境变量设置
    #[tokio::test]
    async fn test_setup_environment() {
        let temp_dir = TempDir::new().unwrap();
        let bundle = RuntimeBundle::with_install_dir(temp_dir.path().to_path_buf());

        // 创建必要的目录结构
        bundle.ensure_install_dir().unwrap();

        // 创建假的 node 和 python 可执行文件
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let node_bin = bundle.get_runtime_path(RuntimeType::Node).join("bin");
            std::fs::create_dir_all(&node_bin).unwrap();
            let node_exe = node_bin.join("node");
            std::fs::write(&node_exe, "#!/bin/sh\necho 'v22.0.0'").unwrap();
            std::fs::set_permissions(&node_exe, std::fs::Permissions::from_mode(0o755)).unwrap();

            let python_bin = bundle.get_runtime_path(RuntimeType::Python).join("bin");
            std::fs::create_dir_all(&python_bin).unwrap();
            let python_exe = python_bin.join("python3");
            std::fs::write(&python_exe, "#!/bin/sh\necho '3.10.0'").unwrap();
            std::fs::set_permissions(&python_exe, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        // 测试环境变量设置
        let result = bundle.setup_environment().await;
        assert!(result.is_ok(), "应该能设置环境变量");
    }

    /// 测试资源路径查找（开发环境）
    #[test]
    fn test_find_resource_paths() {
        let bundle = RuntimeBundle::new().unwrap();
        let paths = bundle.find_resource_paths();

        // 开发环境应该至少找到一个路径
        assert!(!paths.is_empty(), "应该能找到资源路径");
    }

    /// 测试运行时检测（未安装）
    #[test]
    fn test_detect_installed_runtimes_none() {
        let temp_dir = TempDir::new().unwrap();
        let bundle = RuntimeBundle::with_install_dir(temp_dir.path().to_path_buf());

        let detected = bundle.detect_installed_runtimes();
        assert!(detected.node.is_none(), "未安装时 Node 应该为 None");
        assert!(detected.python.is_none(), "未安装时 Python 应该为 None");
    }
}
