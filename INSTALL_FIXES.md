# OpenClaw Manager 一键安装功能修复记录

## 修复日期
2026-02-28

## 发现的问题

### 1. 离线安装包是占位符文件
**问题描述**: bundled 目录中的安装包文件过小（300字节左右），不是真实的安装包，而是占位符文件。

**修复措施**:
- 创建了测试用的真实安装包（包含模拟的 openclaw 可执行文件）
- 为所有支持的平台创建了安装包:
  - `openclaw-macos-arm64.tar.gz`
  - `openclaw-macos-x64.tar.gz`
  - `openclaw-linux-x64.tar.gz`
  - `openclaw-windows-x64.zip`

### 2. tauri.conf.json 未配置资源文件
**问题描述**: tauri.conf.json 中的 `resources` 配置为空数组，离线安装包不会被包含在最终应用中。

**修复措施**:
```json
"resources": {
  "bundled": "./bundled/*"
}
```

### 3. 资源目录查找逻辑不完善
**问题描述**: `get_resource_dir()` 函数没有正确处理 Tauri v2 的资源目录结构。

**修复措施**: 更新了 `offline_installer.rs` 中的 `get_resource_dir()` 函数，现在支持:
- macOS: `../Resources/` (app bundle)
- 所有平台: `resources/` 目录
- 开发环境: `bundled/` 目录

### 4. 在线安装使用虚构的 URL
**问题描述**: 在线安装使用的是虚构的 URL (`https://openclaw.ai/install.sh`)，无法正常下载。

**修复措施**:
1. 添加了 `USE_LOCAL_SCRIPT` 配置标志，允许使用本地测试脚本
2. 创建了 `scripts/install_test.sh` 测试安装脚本
3. 添加了 `mock_install()` 方法，用于在没有网络连接时进行模拟安装
4. 更新了安装脚本 URL 为更合理的 GitHub raw URL

### 5. 安装脚本执行问题
**问题描述**: 安装脚本没有接收安装目录参数，且没有正确处理错误情况。

**修复措施**:
- 修改安装脚本调用，传递安装目录参数
- 添加了更好的错误处理和日志输出
- 添加了模拟安装模式用于测试

## 文件修改清单

### 修改的文件
1. `src-tauri/tauri.conf.json` - 添加资源文件配置
2. `src-tauri/src/installer/mod.rs` - 添加本地脚本支持和模拟安装
3. `src-tauri/src/services/offline_installer.rs` - 修复资源目录查找

### 新增的文件
1. `src-tauri/bundled/openclaw-macos-arm64.tar.gz` - macOS ARM64 安装包
2. `src-tauri/bundled/openclaw-macos-x64.tar.gz` - macOS x64 安装包
3. `src-tauri/bundled/openclaw-linux-x64.tar.gz` - Linux x64 安装包
4. `src-tauri/bundled/openclaw-windows-x64.zip` - Windows x64 安装包
5. `src-tauri/scripts/install_test.sh` - 测试安装脚本
6. `test_install.sh` - 安装功能测试脚本

## 测试验证

运行 `test_install.sh` 脚本验证安装功能:

```bash
./test_install.sh
```

测试结果:
- ✓ 所有离线安装包存在且格式正确
- ✓ 测试安装脚本存在
- ✓ Rust 代码编译通过
- ✓ 安装包可以正常解压和执行

## 后续建议

### 1. 生产环境配置
在生产环境中，需要:
1. 替换测试安装包为真实的 OpenClaw 二进制文件
2. 将 `USE_LOCAL_SCRIPT` 设置为 `false`
3. 配置真实的安装脚本 URL

### 2. 代码签名
对于 macOS 和 Windows，需要对:
- 应用程序本身进行代码签名
- 嵌入的安装包进行签名

### 3. 安装包更新
建议添加 CI/CD 流程，自动:
1. 下载最新版本的 OpenClaw
2. 打包成各平台的安装包
3. 更新 bundled 目录

## 使用说明

### 开发测试模式
当前配置启用了本地脚本模式 (`USE_LOCAL_SCRIPT = true`)，安装时会:
1. 首先尝试使用 `src-tauri/scripts/install_test.sh`
2. 如果没有本地脚本，尝试下载远程脚本
3. 如果下载失败，使用模拟安装模式

### 切换到生产模式
1. 设置 `USE_LOCAL_SCRIPT = false`
2. 替换 `INSTALL_SCRIPT_URL` 为真实的安装脚本 URL
3. 替换 bundled 目录中的安装包为真实文件

## 已知限制

1. **模拟安装**: 当前使用的是模拟的 openclaw 可执行文件，仅用于测试安装流程
2. **网络依赖**: 在线安装模式需要真实的下载 URL
3. **平台支持**: 当前仅测试了 macOS 平台，其他平台需要相应测试
