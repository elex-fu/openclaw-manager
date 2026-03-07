# OpenClaw Manager 完整打包发布方案

## 概述

本文档描述从 GitHub 仓库到多平台安装包发布的完整流程，支持 6 个平台组合：
- macOS: Intel (x64) + Apple Silicon (ARM64)
- Windows: Intel/AMD (x64) + ARM64 (Snapdragon)
- Linux: x64 + ARM64

---

## 一、GitHub 仓库设置

### 1.1 必需的文件结构

```
openclaw-manager/
├── .github/
│   └── workflows/
│       ├── build-release.yml      # 主构建工作流
│       ├── build-nightly.yml      # 夜间构建
│       └── pr-check.yml           # PR 检查
├── src-tauri/
│   ├── bundled/                   # 离线安装包目录
│   ├── src/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── scripts/
│   ├── build-offline-package.js   # 下载离线包
│   └── build-release.sh           # 本地构建脚本
├── docs/
│   ├── BUILD_RELEASE_STRATEGY.md  # 本文档
│   └── LINUX-DEPENDENCIES.md
└── package.json
```

### 1.2 必需的 Secrets 配置

在 GitHub 仓库 Settings -> Secrets and variables -> Actions 中配置：

| Secret 名称 | 用途 | 获取方式 |
|------------|------|----------|
| `APPLE_CERTIFICATE` | macOS 代码签名 | Base64 编码的 .p12 证书 |
| `APPLE_CERTIFICATE_PASSWORD` | 证书密码 | 创建证书时设置 |
| `APPLE_SIGNING_IDENTITY` | 签名身份 | 证书中的 Common Name |
| `APPLE_ID` | 苹果开发者账号 | developer@example.com |
| `APPLE_PASSWORD` | 应用专用密码 | Apple ID 管理页面生成 |
| `APPLE_TEAM_ID` | 开发者团队 ID | Apple Developer Portal |
| `WINDOWS_CERTIFICATE` | Windows 代码签名 | Base64 编码的 .pfx 证书 |
| `WINDOWS_CERTIFICATE_PASSWORD` | 证书密码 | 创建证书时设置 |

---

## 二、CI/CD 工作流程设计

### 2.1 主构建工作流 (build-release.yml)

```yaml
name: Build and Release

on:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+*'  # v0.1.0, v0.1.0-beta.1
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to build (e.g., v0.1.0)'
        required: true
        type: string
      draft:
        description: 'Create as draft release'
        required: false
        default: true
        type: boolean

env:
  CARGO_INCREMENTAL: 0
  RUST_BACKTRACE: short

jobs:
  # ========== 构建矩阵 ==========
  build:
    name: Build ${{ matrix.platform }} (${{ matrix.arch }})
    runs-on: ${{ matrix.os }}

    strategy:
      fail-fast: false
      matrix:
        include:
          # macOS x64 (Intel)
          - os: macos-13
            platform: macos
            arch: x86_64
            target: x86_64-apple-darwin
            bundle-target: x86_64-apple-darwin

          # macOS ARM64 (Apple Silicon)
          - os: macos-latest
            platform: macos
            arch: aarch64
            target: aarch64-apple-darwin
            bundle-target: aarch64-apple-darwin

          # Windows x64
          - os: windows-latest
            platform: windows
            arch: x86_64
            target: x86_64-pc-windows-msvc
            bundle-target: x86_64-pc-windows-msvc

          # Windows ARM64
          - os: windows-latest
            platform: windows
            arch: aarch64
            target: aarch64-pc-windows-msvc
            bundle-target: aarch64-pc-windows-msvc

          # Linux x64
          - os: ubuntu-22.04
            platform: linux
            arch: x86_64
            target: x86_64-unknown-linux-gnu
            bundle-target: x86_64-unknown-linux-gnu

          # Linux ARM64
          - os: ubuntu-22.04
            platform: linux
            arch: aarch64
            target: aarch64-unknown-linux-gnu
            bundle-target: aarch64-unknown-linux-gnu

    steps:
      # ---------- 检出代码 ----------
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      # ---------- 获取版本信息 ----------
      - name: Get version
        id: version
        shell: bash
        run: |
          if [ "${{ github.event_name }}" = "workflow_dispatch" ]; then
            VERSION="${{ github.event.inputs.version }}"
          else
            VERSION="${GITHUB_REF#refs/tags/}"
          fi
          echo "version=$VERSION" >> $GITHUB_OUTPUT
          echo "Version: $VERSION"

      # ---------- 下载离线安装包 ----------
      - name: Download offline packages
        shell: bash
        run: |
          mkdir -p src-tauri/bundled
          PLATFORM="${{ matrix.platform }}"
          ARCH="${{ matrix.arch }}"

          if [ "$PLATFORM" = "macos" ]; then
            EXT="tar.gz"
          elif [ "$PLATFORM" = "windows" ]; then
            EXT="zip"
          else
            EXT="tar.gz"
          fi

          FILENAME="openclaw-${{ steps.version.outputs.version }}-${PLATFORM}-${ARCH}.${EXT}"
          URL="https://github.com/openclai/openclaw/releases/download/${{ steps.version.outputs.version }}/${FILENAME}"

          echo "Downloading: $URL"
          curl -L -o "src-tauri/bundled/${FILENAME}" "$URL" || {
            echo "Warning: Could not download $FILENAME, build will skip embedding"
            rm -f "src-tauri/bundled/${FILENAME}"
          }

      # ---------- 设置 Node.js ----------
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      # ---------- 安装依赖 ----------
      - name: Install dependencies
        run: npm ci

      # ---------- 平台特定设置 ----------

      # macOS 设置
      - name: Setup macOS signing
        if: matrix.platform == 'macos'
        env:
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
        run: |
          echo "$APPLE_CERTIFICATE" | base64 --decode > certificate.p12
          security create-keychain -p "${{ secrets.KEYCHAIN_PASSWORD }}" build.keychain
          security default-keychain -s build.keychain
          security unlock-keychain -p "${{ secrets.KEYCHAIN_PASSWORD }}" build.keychain
          security import certificate.p12 -k build.keychain -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
          security set-keychain-settings -t 3600 -u build.keychain

      # Windows 设置
      - name: Setup Windows signing
        if: matrix.platform == 'windows'
        shell: pwsh
        run: |
          $certBytes = [Convert]::FromBase64String("${{ secrets.WINDOWS_CERTIFICATE }}")
          [IO.File]::WriteAllBytes("certificate.pfx", $certBytes)

      # Linux 设置
      - name: Install Linux dependencies
        if: matrix.platform == 'linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libgtk-3-dev \
            libwebkit2gtk-4.0-dev \
            libappindicator3-dev \
            librsvg2-dev \
            patchelf \
            libssl-dev \
            pkg-config

      # ---------- 安装 Rust ----------
      - name: Setup Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}

      # ---------- 构建应用 ----------
      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        with:
          args: --target ${{ matrix.target }}
          target: ${{ matrix.target }}

      # ---------- 上传产物 ----------
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.platform }}-${{ matrix.arch }}
          path: |
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.dmg
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.app
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.msi
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.exe
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.deb
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.rpm
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.AppImage
          if-no-files-found: warn

  # ========== 创建 Release ==========
  release:
    name: Create Release
    needs: build
    runs-on: ubuntu-latest
    permissions:
      contents: write

    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts
          pattern: '*'
          merge-multiple: false

      - name: Display artifacts
        run: |
          echo "Artifacts downloaded:"
          find artifacts -type f -name "*.dmg" -o -name "*.msi" -o -name "*.exe" -o -name "*.deb" -o -name "*.rpm" -o -name "*.AppImage"

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          tag_name: ${{ github.ref_name }}
          name: OpenClaw Manager ${{ github.ref_name }}
          body: |
            ## 安装包下载

            ### macOS
            - `OpenClaw Manager_*_x64.dmg` - Intel Mac
            - `OpenClaw Manager_*_aarch64.dmg` - Apple Silicon (M1/M2/M3)

            ### Windows
            - `OpenClaw Manager_*_x64_en-US.msi` - 64-bit Windows
            - `OpenClaw Manager_*_aarch64_en-US.msi` - ARM64 Windows

            ### Linux
            - `openclaw-manager_*.deb` - Debian/Ubuntu
            - `openclaw-manager-*.rpm` - Fedora/RHEL
            - `openclaw-manager_*.AppImage` - 通用 Linux 格式

            ## 系统要求
            - macOS 10.15+ (Intel) / macOS 11+ (Apple Silicon)
            - Windows 10 1809+ / Windows 11
            - Ubuntu 20.04+ / Debian 11+ / Fedora 35+

            ## 验证
            安装包已进行代码签名，下载后可在系统设置中验证。
          files: |
            artifacts/**/*.dmg
            artifacts/**/*.msi
            artifacts/**/*.exe
            artifacts/**/*.deb
            artifacts/**/*.rpm
            artifacts/**/*.AppImage
          draft: ${{ github.event.inputs.draft || true }}
          prerelease: ${{ contains(github.ref_name, 'beta') || contains(github.ref_name, 'alpha') || contains(github.ref_name, 'rc') }}
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  # ========== 更新更新服务器 ==========
  update-server:
    name: Update Server
    needs: release
    runs-on: ubuntu-latest
    if: github.event_name == 'push' && !contains(github.ref_name, 'beta') && !contains(github.ref_name, 'alpha')

    steps:
      - name: Checkout updater repo
        uses: actions/checkout@v4
        with:
          repository: openclai/openclaw-updater
          token: ${{ secrets.UPDATER_TOKEN }}

      - name: Update version manifest
        run: |
          cat > manifest.json << EOF
          {
            "version": "${{ github.ref_name }}",
            "notes": "New release available",
            "pub_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
            "platforms": {
              "darwin-x86_64": {
                "signature": "",
                "url": "https://github.com/openclai/openclaw-manager/releases/download/${{ github.ref_name }}/OpenClaw.Manager_${{ github.ref_name }}_x64.dmg"
              },
              "darwin-aarch64": {
                "signature": "",
                "url": "https://github.com/openclai/openclaw-manager/releases/download/${{ github.ref_name }}/OpenClaw.Manager_${{ github.ref_name }}_aarch64.dmg"
              },
              "windows-x86_64": {
                "signature": "",
                "url": "https://github.com/openclai/openclaw-manager/releases/download/${{ github.ref_name }}/OpenClaw.Manager_${{ github.ref_name }}_x64_en-US.msi"
              }
            }
          }
          EOF

      - name: Commit manifest
        run: |
          git config user.name "GitHub Actions"
          git config user.email "actions@github.com"
          git add manifest.json
          git commit -m "Update manifest for ${{ github.ref_name }}"
          git push
