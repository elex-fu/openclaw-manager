# 嵌入式 Runtime 目录

此目录用于存放 OpenClaw 运行所需的嵌入式环境，实现"一键部署"零依赖启动。

## 目录结构

```
runtimes/
├── node-v22.x.x-darwin-x64.tar.gz      # Node.js 22 (macOS x64)
├── node-v22.x.x-darwin-arm64.tar.gz    # Node.js 22 (macOS ARM64)
├── node-v22.x.x-linux-x64.tar.xz       # Node.js 22 (Linux x64)
├── node-v22.x.x-win-x64.zip            # Node.js 22 (Windows x64)
├── python-3.10.x-macos11-x64.tar.gz    # Python 3.10 (macOS x64)
├── python-3.10.x-macos11-arm64.tar.gz  # Python 3.10 (macOS ARM64)
├── python-3.10.x-linux-x64.tar.gz      # Python 3.10 (Linux x64)
├── python-3.10.x-embed-win-amd64.zip   # Python 3.10 (Windows x64)
└── README.md
```

## 获取方式

### Node.js 22

从 Node.js 官方下载预编译二进制文件：

```bash
# macOS ARM64 (Apple Silicon)
curl -O https://nodejs.org/dist/v22.0.0/node-v22.0.0-darwin-arm64.tar.gz

# macOS x64 (Intel)
curl -O https://nodejs.org/dist/v22.0.0/node-v22.0.0-darwin-x64.tar.gz

# Linux x64
curl -O https://nodejs.org/dist/v22.0.0/node-v22.0.0-linux-x64.tar.xz

# Windows x64
curl -O https://nodejs.org/dist/v22.0.0/node-v22.0.0-win-x64.zip
```

### Python 3.10

#### macOS

使用 `python-build-standalone` 项目：

```bash
# 下载独立构建版本
curl -L -o python-3.10.14-macos11-x64.tar.gz \
  https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-3.10.14+20240107-x86_64-apple-darwin-install_only.tar.gz

curl -L -o python-3.10.14-macos11-arm64.tar.gz \
  https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-3.10.14+20240107-aarch64-apple-darwin-install_only.tar.gz
```

#### Linux

```bash
curl -L -o python-3.10.14-linux-x64.tar.gz \
  https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-3.10.14+20240107-x86_64-unknown-linux-gnu-install_only.tar.gz
```

#### Windows

```bash
curl -L -o python-3.10.14-embed-win-amd64.zip \
  https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-3.10.14+20240107-x86_64-pc-windows-msvc-install_only.tar.gz
```

## 命名规范

Runtime 包文件名必须符合以下格式：

- **Node.js**: `node-v{VERSION}-{PLATFORM}-{ARCH}.tar.gz` 或 `.tar.xz` 或 `.zip`
  - PLATFORM: `darwin`, `linux`, `win32`
  - ARCH: `x64`, `arm64`

- **Python**: `python-{VERSION}-{PLATFORM}{VERSION}-{ARCH}.tar.gz` 或 `.zip`
  - PLATFORM: `macos11`, `linux`, `embed-win`
  - ARCH: `x64`, `arm64`, `amd64`

## 文件大小参考

| 组件 | 压缩后 | 解压后 |
|------|--------|--------|
| Node.js 22 | ~30MB | ~100MB |
| Python 3.10 | ~25MB | ~80MB |
| **总计** | **~55MB** | **~180MB** |

## 开发环境

在开发环境中，如果没有放置真实的 runtime 包，安装器会自动回退到使用系统已安装的版本（如果可用）。

生产环境打包时，请确保将 runtime 包放入此目录。

## CI/CD 自动化

可以在 CI 流程中自动下载这些包：

```yaml
# .github/workflows/build.yml 示例
- name: Download Runtimes
  run: |
    mkdir -p src-tauri/bundled/runtimes
    cd src-tauri/bundled/runtimes

    # 根据平台下载对应 runtime
    if [[ "$RUNNER_OS" == "macOS" ]]; then
      curl -O https://nodejs.org/dist/v22.0.0/node-v22.0.0-darwin-x64.tar.gz
      curl -L -o python-3.10.14-macos11-x64.tar.gz \
        https://github.com/indygreg/python-build-standalone/...
    fi
```
