#!/bin/bash
# 离线安装包构建脚本

VERSION="${1:-latest}"
PLATFORMS=("macos-arm64" "macos-x64" "windows-x64" "windows-arm64" "linux-x64" "linux-arm64")
OUTPUT_DIR="src-tauri/bundled"

mkdir -p $OUTPUT_DIR

for platform in "${PLATFORMS[@]}"; do
    echo "Building offline package for $platform..."

    case $platform in
        "macos-arm64")
            curl -L "https://github.com/openclaw/openclaw/releases/download/$VERSION/openclaw-$VERSION-macos-arm64.tar.gz" \
                -o "$OUTPUT_DIR/openclaw-macos-arm64.tar.gz" || echo "Download failed for $platform"
            ;;
        "macos-x64")
            curl -L "https://github.com/openclaw/openclaw/releases/download/$VERSION/openclaw-$VERSION-macos-x64.tar.gz" \
                -o "$OUTPUT_DIR/openclaw-macos-x64.tar.gz" || echo "Download failed for $platform"
            ;;
        "windows-x64")
            curl -L "https://github.com/openclaw/openclaw/releases/download/$VERSION/openclaw-$VERSION-windows-x64.zip" \
                -o "$OUTPUT_DIR/openclaw-windows-x64.zip" || echo "Download failed for $platform"
            ;;
        "windows-arm64")
            curl -L "https://github.com/openclaw/openclaw/releases/download/$VERSION/openclaw-$VERSION-windows-arm64.zip" \
                -o "$OUTPUT_DIR/openclaw-windows-arm64.zip" || echo "Download failed for $platform"
            ;;
        "linux-x64")
            curl -L "https://github.com/openclaw/openclaw/releases/download/$VERSION/openclaw-$VERSION-linux-x64.tar.gz" \
                -o "$OUTPUT_DIR/openclaw-linux-x64.tar.gz" || echo "Download failed for $platform"
            ;;
        "linux-arm64")
            curl -L "https://github.com/openclaw/openclaw/releases/download/$VERSION/openclaw-$VERSION-linux-arm64.tar.gz" \
                -o "$OUTPUT_DIR/openclaw-linux-arm64.tar.gz" || echo "Download failed for $platform"
            ;;
    esac
done

echo "Offline packages built in $OUTPUT_DIR/"
ls -la $OUTPUT_DIR/
