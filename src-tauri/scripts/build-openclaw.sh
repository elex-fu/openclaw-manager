#!/bin/bash

# OpenClaw Build Script
# Builds OpenClaw and creates the distribution archive for bundling

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
OPENCLAW_SOURCE="${PROJECT_ROOT}/target/release/bundled/**/*/openclaw"
BUILD_DIR="${PROJECT_ROOT}/build"
BUNDLED_DIR="${PROJECT_ROOT}/src-tauri/bundled"

echo "======================================"
echo "OpenClaw Build Script"
echo "======================================"

# Detect platform
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

if [ "$PLATFORM" == "darwin" ]; then
    PLATFORM="macos"
    if [ "$ARCH" == "arm64" ]; then
        ARCH="aarch64"
    elif [ "$ARCH" == "x86_64" ]; then
        ARCH="x64"
    fi
elif [ "$PLATFORM" == "linux" ]; then
    PLATFORM="linux"
    if [ "$ARCH" == "x86_64" ]; then
        ARCH="x64"
    fi
fi

PACKAGE_NAME="openclaw-${PLATFORM}-${ARCH}"
PACKAGE_FILE="${PACKAGE_NAME}.tar.gz"

echo "Platform: $PLATFORM"
echo "Architecture: $ARCH"
echo "Package: $PACKAGE_FILE"

# Check if OpenClaw source exists
if [ ! -d "$OPENCLAW_SOURCE" ]; then
    echo "Error: OpenClaw source not found at $OPENCLAW_SOURCE"
    exit 1
fi

echo ""
echo "Found OpenClaw at: $OPENCLAW_SOURCE"

# Create directories
mkdir -p "$BUILD_DIR"
mkdir -p "$BUNDLED_DIR"

# Create temporary directory for packaging
TEMP_DIR=$(mktemp -d)
PACKAGE_DIR="${TEMP_DIR}/${PACKAGE_NAME}"
mkdir -p "$PACKAGE_DIR"

echo ""
echo "Packaging OpenClaw..."

# Copy OpenClaw files
cp -r "${OPENCLAW_SOURCE}/." "$PACKAGE_DIR/"

# Create archive
cd "$TEMP_DIR"
tar -czf "${BUILD_DIR}/${PACKAGE_FILE}" "$PACKAGE_NAME"
cd "$PROJECT_ROOT"

# Clean up
rm -rf "$TEMP_DIR"

echo ""
echo "Archive created: ${BUILD_DIR}/${PACKAGE_FILE}"
echo "Size: $(du -h "${BUILD_DIR}/${PACKAGE_FILE}" | cut -f1)"

# Copy to bundled directory
cp "${BUILD_DIR}/${PACKAGE_FILE}" "${BUNDLED_DIR}/"
echo "Copied to: ${BUNDLED_DIR}/${PACKAGE_FILE}"

# Copy to app bundles
APP_BUNDLE="${PROJECT_ROOT}/OpenClaw-Manager.app"
if [ -d "$APP_BUNDLE" ]; then
    RESOURCES_DIR="${APP_BUNDLE}/Contents/Resources"
    mkdir -p "$RESOURCES_DIR"
    cp "${BUILD_DIR}/${PACKAGE_FILE}" "$RESOURCES_DIR/"
    echo "Copied to app bundle"
fi

TARGET_BUNDLE="${PROJECT_ROOT}/src-tauri/target/release/bundle/macos/OpenClaw Manager.app"
if [ -d "$TARGET_BUNDLE" ]; then
    RESOURCES_DIR="${TARGET_BUNDLE}/Contents/Resources"
    mkdir -p "$RESOURCES_DIR"
    cp "${BUILD_DIR}/${PACKAGE_FILE}" "$RESOURCES_DIR/"
    echo "Copied to target bundle"
fi

echo ""
echo "======================================"
echo "Build complete!"
echo "======================================"
