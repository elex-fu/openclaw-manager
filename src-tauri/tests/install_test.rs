//! 安装功能集成测试

/// 测试安装目录创建和基本功能
#[test]
fn test_install_directory_creation() {
    use std::fs;
    use std::path::PathBuf;

    let test_dir = PathBuf::from("/tmp/test_openclaw_basic");
    let _ = fs::remove_dir_all(&test_dir);

    // 测试目录创建
    let result = fs::create_dir_all(&test_dir);
    assert!(result.is_ok(), "应该能创建安装目录");

    // 测试 bin 目录创建
    let bin_dir = test_dir.join("bin");
    let result = fs::create_dir_all(&bin_dir);
    assert!(result.is_ok(), "应该能创建 bin 目录");

    // 测试文件写入
    let test_file = bin_dir.join("openclaw");
    let result = fs::write(&test_file, "#!/bin/bash\necho 'test'");
    assert!(result.is_ok(), "应该能写入文件");

    // 设置执行权限（Unix）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_file).unwrap().permissions();
        perms.set_mode(0o755);
        let result = fs::set_permissions(&test_file, perms);
        assert!(result.is_ok(), "应该能设置权限");
    }

    // 验证文件存在
    assert!(test_file.exists(), "文件应该存在");

    // 清理
    let _ = fs::remove_dir_all(&test_dir);
}

/// 测试安装包解压
#[test]
fn test_package_extraction() {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    let bundled_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("bundled");
    let test_dir = PathBuf::from("/tmp/test_openclaw_extraction");
    let _ = fs::remove_dir_all(&test_dir);

    // 查找 macOS ARM64 包
    let package = bundled_dir.join("openclaw-macos-arm64.tar.gz");

    if package.exists() {
        // 创建测试目录
        fs::create_dir_all(&test_dir).unwrap();

        // 解压包
        let output = Command::new("tar")
            .args(["-xzf", package.to_str().unwrap(), "-C", test_dir.to_str().unwrap()])
            .output()
            .expect("应该能执行 tar 命令");

        assert!(output.status.success(), "tar 解压应该成功");

        // 验证解压的文件
        let openclaw_bin = test_dir.join("bin").join("openclaw");
        assert!(openclaw_bin.exists(), "解压后 openclaw 应该存在");

        // 清理
        let _ = fs::remove_dir_all(&test_dir);
    } else {
        println!("跳过测试：安装包不存在");
    }
}

/// 测试配置文件创建和读取
#[test]
fn test_config_creation() {
    use std::fs;
    use std::path::PathBuf;

    let test_dir = PathBuf::from("/tmp/test_openclaw_config");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();

    // 创建测试配置
    let config = r#"
version: "1.0.0"
name: "Test OpenClaw"
models:
  - id: "test-model"
    name: "Test Model"
    provider: "test"
    api_key: null
    api_base: null
    model: "test-model"
    temperature: 0.7
    max_tokens: 2048
    enabled: true
default_model: "test-model"
agents: []
skills: []
settings:
  log_level: "info"
  auto_update: false
  theme: "light"
  language: "zh-CN"
  custom_vars: {}
"#;

    let config_path = test_dir.join("config.yaml");
    fs::write(&config_path, config).unwrap();

    // 读取并验证
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("version: \"1.0.0\""), "配置应该包含版本");
    assert!(content.contains("Test OpenClaw"), "配置应该包含名称");

    // 清理
    let _ = fs::remove_dir_all(&test_dir);
}
