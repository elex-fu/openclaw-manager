# Linux 依赖说明

本文档说明 OpenClaw Manager 在 Linux 平台上的依赖要求，特别是安全存储功能的相关依赖。

## 目录

- [概述](#概述)
- [运行模式](#运行模式)
- [完整功能模式](#完整功能模式)
- [最小化模式](#最小化模式)
- [WSL 注意事项](#wsl-注意事项)
- [故障排除](#故障排除)

## 概述

OpenClaw Manager 在 Linux 上使用安全存储功能时，优先使用系统密钥链（keyring）来存储 API 密钥等敏感信息。当系统密钥链不可用时，会自动降级到加密文件存储模式。

## 运行模式

### 完整功能模式

在完整功能模式下，应用使用系统密钥链（D-Bus Secret Service 或 KWallet）存储 API 密钥。

#### 依赖要求

| 依赖 | 包名（Debian/Ubuntu） | 包名（Fedora/RHEL） | 包名（Arch） | 说明 |
|------|----------------------|---------------------|-------------|------|
| D-Bus | `dbus` | `dbus` | `dbus` | 进程间通信 |
| libsecret | `libsecret-1-0` | `libsecret` | `libsecret` | Secret Service API |
| GNOME Keyring | `gnome-keyring` | `gnome-keyring` | `gnome-keyring` | 密钥环守护进程（可选） |
| KWallet | `kwalletmanager` | `kwalletmanager5` | `kwallet` | KDE 密钥管理（可选） |

#### 安装命令

**Debian/Ubuntu:**
```bash
sudo apt update
sudo apt install dbus libsecret-1-0 gnome-keyring
```

**Fedora/RHEL:**
```bash
sudo dnf install dbus libsecret gnome-keyring
```

**Arch Linux:**
```bash
sudo pacman -S dbus libsecret gnome-keyring
```

#### 验证密钥链可用性

```bash
# 检查 D-Bus 是否运行
echo $DBUS_SESSION_BUS_ADDRESS

# 检查 secret service 是否可用
secret-tool store --label="test" testkey testvalue
secret-tool lookup testkey testvalue
secret-tool clear testkey testvalue
```

### 最小化模式

当系统密钥链不可用时，OpenClaw Manager 会自动降级到最小化模式，使用 AES-256-GCM 加密文件存储 API 密钥。

#### 依赖要求

最小化模式**不需要**额外的系统依赖。只需要：

- 可写的用户配置目录（`~/.config/openclaw-manager/`）
- 能够读取机器 ID（`/etc/machine-id` 或 `/var/lib/dbus/machine-id`）

#### 加密文件存储详情

- **加密算法**: AES-256-GCM
- **密钥派生**: 基于机器 ID + 固定盐值
- **存储位置**: `~/.config/openclaw-manager/{provider}.enc`
- **文件权限**: `0o600`（仅所有者可读写）
- **Nonce**: 每次加密随机生成 12 字节

#### 安全注意事项

1. **机器绑定**: 加密文件与当前机器绑定，无法直接复制到其他机器使用
2. **权限保护**: 文件权限设置为仅所有者可读写
3. **无内存保护**: 相比密钥链，加密文件在内存中可能存在更长时间

## WSL 注意事项

在 Windows Subsystem for Linux (WSL) 中运行 OpenClaw Manager 时，需要注意以下事项：

### WSL 1

- 没有独立的内核，D-Bus 服务可能无法正常运行
- 推荐使用**最小化模式**（加密文件存储）
- 需要确保 `~/.config` 目录在 WSL 文件系统中（不要在 `/mnt/c/` 等 Windows 挂载点）

### WSL 2

- 有完整的 Linux 内核，可以运行 D-Bus
- 需要手动启动 D-Bus 和密钥环服务：

```bash
# 启动 D-Bus
sudo service dbus start

# 启动 GNOME Keyring（如果使用）
eval $(gnome-keyring-daemon --start)
export SSH_AUTH_SOCK
```

### WSL 通用建议

1. **配置文件位置**: 建议将配置放在 WSL 文件系统内以获得更好的性能：
   ```bash
   # 好的做法
   ~/.config/openclaw-manager/

   # 避免
   /mnt/c/Users/xxx/.config/openclaw-manager/
   ```

2. **机器 ID**: WSL 实例可能没有 `/etc/machine-id`，应用会自动使用 `username@hostname` 作为替代

3. **权限**: WSL 的权限模型与原生 Linux 略有不同，但应用会自动处理

## 故障排除

### 问题：无法保存 API 密钥

**症状**: 保存 API 密钥时提示 "Secure storage error"

**解决步骤**:

1. 检查配置目录权限：
   ```bash
   ls -la ~/.config/openclaw-manager/
   ```

2. 检查机器 ID：
   ```bash
   cat /etc/machine-id
   # 或
   cat /var/lib/dbus/machine-id
   ```

3. 检查磁盘空间：
   ```bash
   df -h ~/.config/
   ```

### 问题：密钥链测试通过但无法存储

**症状**: 测试可以保存，但实际保存 API 密钥失败

**可能原因**:
- 某些密钥链实现有存储配额限制
- 密钥链被锁定（需要输入密码解锁）

**解决**:
```bash
# 检查密钥链状态
gnome-keyring-daemon --unlock

# 或使用 secret-tool 测试
secret-tool store --label="OpenClaw Test" openclaw test
```

### 问题：加密文件无法读取

**症状**: 之前保存的 API 密钥无法读取

**可能原因**:
1. 机器 ID 改变（系统重装、克隆虚拟机）
2. 文件权限被修改
3. 文件损坏

**检查**:
```bash
# 检查文件存在和权限
ls -la ~/.config/openclaw-manager/*.enc

# 检查机器 ID
hostnamectl | grep "Machine ID"
```

**注意**: 如果机器 ID 改变，之前保存的加密文件将无法解密，需要重新输入 API 密钥。

### 问题：WSL 中无法启动密钥链

**症状**: WSL 中提示 keyring 不可用

**解决**:
```bash
# 安装必要的包
sudo apt update
sudo apt install dbus-x11 gnome-keyring

# 设置环境变量
export DISPLAY=:0

# 启动密钥链守护进程
eval $(gnome-keyring-daemon --start --components=secrets)
export GNOME_KEYRING_CONTROL
export SSH_AUTH_SOCK
```

或者，接受使用加密文件存储模式（自动降级）。

## 相关文档

- [用户指南](USER_GUIDE.md)
- [故障排除](TROUBLESHOOTING.md)
- [MVP 路线图](MVP-ROADMAP-v2.md)
