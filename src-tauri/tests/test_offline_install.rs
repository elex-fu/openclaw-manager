// 离线安装测试脚本
// 用于测试 OfflineInstaller 的各个功能

use std::path::PathBuf;

fn main() {
    println!("==================================");
    println!("离线安装功能测试");
    println!("==================================\n");

    // 测试 1: 检测平台
    println!("测试 1: 检测当前平台...");
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    println!("  操作系统: {}", os);
    println!("  架构: {}\n", arch);

    // 测试 2: 检查资源目录
    println!("测试 2: 检查资源目录...");
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    println!("  可执行文件目录: {:?}", exe_dir);

    // 可能的资源目录
    let possible_dirs = vec![
        exe_dir.join("../Resources"),
        exe_dir.join("resources"),
        exe_dir.join("bundled"),
        exe_dir.to_path_buf(),
    ];

    for dir in &possible_dirs {
        let exists = dir.exists();
        println!("  {:?} - {}", dir, if exists { "存在" } else { "不存在" });
    }
    println!();

    // 测试 3: 检查安装包
    println!("测试 3: 检查离线安装包...");
    let current_dir = std::env::current_dir().unwrap();
    let bundled_dir = current_dir.join("src-tauri").join("bundled");
    println!("  bundled 目录: {:?}", bundled_dir);

    let expected_packages = vec![
        "openclaw-macos-arm64.tar.gz",
        "openclaw-macos-x64.tar.gz",
        "openclaw-linux-x64.tar.gz",
        "openclaw-windows-x64.zip",
    ];

    for package in &expected_packages {
        let path = bundled_dir.join(package);
        if path.exists() {
            let metadata = std::fs::metadata(&path).unwrap();
            println!("  ✓ {} ({} 字节)", package, metadata.len());
        } else {
            println!("  ✗ {} 不存在", package);
        }
    }
    println!();

    // 测试 4: 验证安装包内容
    println!("测试 4: 验证安装包内容...");
    let test_package = bundled_dir.join("openclaw-macos-arm64.tar.gz");
    if test_package.exists() {
        println!("  检查 {:?}...", test_package);

        // 使用 tar 命令列出内容
        let output = std::process::Command::new("tar")
            .args(["-tzf", test_package.to_str().unwrap()])
            .output();

        match output {
            Ok(result) if result.status.success() => {
                let content = String::from_utf8_lossy(&result.stdout);
                println!("  包内文件:");
                for line in content.lines().take(10) {
                    println!("    - {}", line);
                }
            }
            _ => println!("  无法读取包内容"),
        }
    }
    println!();

    // 测试 5: 模拟安装流程
    println!("测试 5: 模拟安装流程...");
    let test_install_dir = std::path::Path::new("/tmp/test_openclaw_install");

    // 清理之前的测试目录
    let _ = std::fs::remove_dir_all(test_install_dir);

    // 创建安装目录
    match std::fs::create_dir_all(test_install_dir) {
        Ok(_) => println!("  ✓ 创建安装目录: {:?}", test_install_dir),
        Err(e) => println!("  ✗ 创建安装目录失败: {}", e),
    }

    // 重新获取 bundled_dir
    let bundled_dir = current_dir.join("src-tauri").join("bundled");
    // 解压安装包
    let package = bundled_dir.join("openclaw-macos-arm64.tar.gz");
    if package.exists() {
        let output = std::process::Command::new("tar")
            .args(["-xzf", package.to_str().unwrap(), "-C", test_install_dir.to_str().unwrap()])
            .output();

        match output {
            Ok(result) if result.status.success() => {
                println!("  ✓ 解压安装包成功");

                // 检查解压后的文件
                let bin_dir = test_install_dir.join("bin");
                if bin_dir.exists() {
                    println!("  ✓ bin 目录存在");

                    let openclaw_bin = bin_dir.join("openclaw");
                    if openclaw_bin.exists() {
                        println!("  ✓ openclaw 可执行文件存在");

                        // 设置执行权限
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let mut perms = std::fs::metadata(&openclaw_bin).unwrap().permissions();
                            perms.set_mode(perms.mode() | 0o111);
                            std::fs::set_permissions(&openclaw_bin, perms).unwrap();
                            println!("  ✓ 设置执行权限成功");

                            // 测试执行
                            let version_output = std::process::Command::new(&openclaw_bin)
                                .arg("--version")
                                .output();

                            match version_output {
                                Ok(result) if result.status.success() => {
                                    let version = String::from_utf8_lossy(&result.stdout);
                                    println!("  ✓ 可执行文件运行成功");
                                    println!("  版本信息: {}", version.trim());
                                }
                                Ok(result) => {
                                    let stderr = String::from_utf8_lossy(&result.stderr);
                                    println!("  ⚠ 可执行文件返回错误: {}", stderr);
                                }
                                Err(e) => {
                                    println!("  ⚠ 无法运行可执行文件: {}", e);
                                }
                            }
                        }
                    } else {
                        println!("  ✗ openclaw 可执行文件不存在");
                    }
                } else {
                    println!("  ✗ bin 目录不存在");
                }
            }
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                println!("  ✗ 解压失败: {}", stderr);
            }
            Err(e) => println!("  ✗ 解压命令执行失败: {}", e),
        }
    }

    // 清理测试目录
    let _ = std::fs::remove_dir_all(test_install_dir);
    println!();

    println!("==================================");
    println!("测试完成");
    println!("==================================");
}
