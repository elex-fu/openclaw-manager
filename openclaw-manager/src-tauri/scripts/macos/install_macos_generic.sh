#!/bin/bash
#
# OpenClaw Generic Installation Script for macOS
# Fallback for unsupported or unknown macOS versions
# Last updated: 2026-02-26
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
INSTALL_DIR="$HOME/.openclaw"
BIN_DIR="$INSTALL_DIR/bin"
VERSION="1.0.0"

echo -e "${CYAN}================================${NC}"
echo -e "${CYAN}OpenClaw Generic Installer for macOS${NC}"
echo -e "${CYAN}================================${NC}"
echo ""

# Show detected system info
echo -e "${YELLOW}Detected System Information:${NC}"
echo "  macOS Version: $(sw_vers -productVersion)"
echo "  Kernel: $(uname -r)"
echo "  Architecture: $(uname -m)"
echo ""

warning() {
    echo ""
    echo -e "${YELLOW}================================${NC}"
    echo -e "${YELLOW}WARNING: Generic Installer${NC}"
    echo -e "${YELLOW}================================${NC}"
    echo ""
    echo "This is a generic installer for macOS."
    echo "Your specific macOS version may not be fully tested."
    echo ""
    echo "Supported versions:"
    echo "  - macOS 10.15 Catalina"
    echo "  - macOS 11 Big Sur"
    echo "  - macOS 12 Monterey"
    echo "  - macOS 13 Ventura"
    echo "  - macOS 14 Sonoma"
    echo "  - macOS 15 Sequoia"
    echo ""
    read -p "Continue with generic installation? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
}

progress() { echo -e "${BLUE}[*]${NC} $1"; }
success() { echo -e "${GREEN}[✓]${NC} $1"; }
error() { echo -e "${RED}[✗]${NC} $1"; }
warning

ARCH=$(uname -m)

# Check prerequisites
progress "Checking prerequisites..."

# Check Xcode Command Line Tools
if ! xcode-select -p &>/dev/null; then
    error "Xcode Command Line Tools not found"
    echo "Installing..."
    xcode-select --install
    exit 1
fi
success "Xcode Command Line Tools installed"

# Check Rust
if ! command -v rustc &>/dev/null; then
    warning "Rust not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

RUST_VERSION=$(rustc --version | cut -d' ' -f2)
progress "Rust version: $RUST_VERSION"
success "Rust installed"

# Create installation directory
progress "Creating installation directory..."
mkdir -p "$BIN_DIR"
success "Created $INSTALL_DIR"

# Download OpenClaw
progress "Downloading OpenClaw $VERSION..."

if [ "$ARCH" = "arm64" ]; then
    DOWNLOAD_URL="https://github.com/openclai/openclaw/releases/download/v${VERSION}/openclaw-${VERSION}-aarch64-apple-darwin.tar.gz"
else
    DOWNLOAD_URL="https://github.com/openclai/openclaw/releases/download/v${VERSION}/openclaw-${VERSION}-x86_64-apple-darwin.tar.gz"
fi

TEMP_DIR=$(mktemp -d)
curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_DIR/openclaw.tar.gz"
tar -xzf "$TEMP_DIR/openclaw.tar.gz" -C "$TEMP_DIR"

if [ -f "$TEMP_DIR/openclaw" ]; then
    cp "$TEMP_DIR/openclaw" "$BIN_DIR/"
    chmod +x "$BIN_DIR/openclaw"
fi

rm -rf "$TEMP_DIR"
success "OpenClaw binary installed"

# Create configuration
progress "Creating configuration..."
mkdir -p "$INSTALL_DIR/config"

cat > "$INSTALL_DIR/config.yaml" << EOF
version: "1.0.0"
name: "My OpenClaw"
installation:
  date: "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  version: "$VERSION"
  platform: "macos-generic"
  architecture: "$ARCH"
models:
  - id: "default-gpt4"
    name: "GPT-4"
    provider: "openai"
    model: "gpt-4"
    temperature: 0.7
    max_tokens: 4096
    enabled: true
agents:
  - id: "default-assistant"
    name: "默认助手"
    model_id: "default-gpt4"
    enabled: true
settings:
  log_level: "info"
  auto_update: true
  theme: "system"
  language: "zh-CN"
EOF

success "Configuration created"

# Add to PATH
progress "Adding OpenClaw to PATH..."
SHELL_CONFIG=""
if [ -f "$HOME/.zshrc" ]; then
    SHELL_CONFIG="$HOME/.zshrc"
elif [ -f "$HOME/.bash_profile" ]; then
    SHELL_CONFIG="$HOME/.bash_profile"
fi

if [ -n "$SHELL_CONFIG" ]; then
    if ! grep -q "\.openclaw/bin" "$SHELL_CONFIG"; then
        echo "" >> "$SHELL_CONFIG"
        echo "# OpenClaw" >> "$SHELL_CONFIG"
        echo 'export PATH="$HOME/.openclaw/bin:$PATH"' >> "$SHELL_CONFIG"
    fi
    success "Added to PATH"
fi

echo ""
echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}Installation Complete!${NC}"
echo -e "${GREEN}================================${NC}"
echo ""
echo -e "OpenClaw $VERSION installed to: ${BLUE}$INSTALL_DIR${NC}"
echo ""
echo -e "${YELLOW}Note: This was a generic installation.${NC}"
echo -e "${YELLOW}If you encounter issues, please report them${NC}"
echo -e "${YELLOW}with your macOS version: $(sw_vers -productVersion)${NC}"
echo ""
