#!/bin/bash
#
# OpenClaw Installation Script for macOS 12 Monterey
# Last updated: 2026-02-26
# Note: For macOS 12.7.6+, MacPorts is recommended over Homebrew
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
USE_MACPORTS=false

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}OpenClaw Installer for macOS 12 Monterey${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

# Check macOS version
MACOS_VERSION=$(sw_vers -productVersion)
echo -e "${BLUE}Detected macOS version: $MACOS_VERSION${NC}"

# For 12.7.6+, suggest MacPorts
if echo "$MACOS_VERSION" | grep -qE "^12\.(7|8|9)"; then
    echo ""
    echo -e "${CYAN}================================${NC}"
    echo -e "${CYAN}Important Note for macOS $MACOS_VERSION${NC}"
    echo -e "${CYAN}================================${NC}"
    echo ""
    echo -e "${YELLOW}Homebrew may have compatibility issues with macOS 12.7.6+${NC}"
    echo -e "${YELLOW}MacPorts is recommended for better stability${NC}"
    echo ""
    echo -e "To install MacPorts:"
    echo -e "  1. Download from: ${CYAN}https://www.macports.org/install.php${NC}"
    echo -e "  2. Install the package for macOS 12 Monterey"
    echo -e "  3. Run: ${CYAN}sudo port selfupdate${NC}"
    echo ""
    read -p "Do you want to use MacPorts instead of Homebrew? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        USE_MACPORTS=true
    fi
fi

ARCH=$(uname -m)
echo -e "${BLUE}Architecture: $ARCH${NC}"

progress() { echo -e "${BLUE}[*]${NC} $1"; }
success() { echo -e "${GREEN}[✓]${NC} $1"; }
error() { echo -e "${RED}[✗]${NC} $1"; }
warning() { echo -e "${YELLOW}[!]${NC} $1"; }

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

# Package manager setup
if [ "$USE_MACPORTS" = true ]; then
    progress "Checking MacPorts..."
    if ! command -v port &>/dev/null; then
        error "MacPorts not found"
        echo "Please install MacPorts first:"
        echo "  1. Download from https://www.macports.org/install.php"
        echo "  2. Install the package for macOS 12 Monterey"
        echo "  3. Run: sudo port selfupdate"
        exit 1
    fi

    success "MacPorts available"

    # Update MacPorts
    progress "Updating MacPorts..."
    sudo port selfupdate

    # Install dependencies via MacPorts
    progress "Installing dependencies via MacPorts..."

    if ! command -v curl &>/dev/null; then
        sudo port install curl
    fi

    if ! command -v git &>/dev/null; then
        sudo port install git
    fi

    # Install Rust via MacPorts if not present
    if ! command -v rustc &>/dev/null; then
        progress "Installing Rust via MacPorts..."
        sudo port install rust
    fi

    success "MacPorts dependencies installed"
else
    # Use Homebrew
    progress "Checking Homebrew..."
    if ! command -v brew &>/dev/null; then
        warning "Homebrew not found. Installing..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        if [ "$ARCH" = "arm64" ]; then
            echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> "$HOME/.zprofile"
            eval "$(/opt/homebrew/bin/brew shellenv)"
        fi
    fi
    success "Homebrew available"

    # Check for potential Homebrew issues on 12.7.6+
    if echo "$MACOS_VERSION" | grep -qE "^12\.(7|8|9)"; then
        warning "Homebrew may have issues on macOS $MACOS_VERSION"
        warning "If installation fails, consider using MacPorts instead"
    fi
fi

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
  platform: "macos-12.0"
  architecture: "$ARCH"
  package_manager: "$([ "$USE_MACPORTS" = true ] && echo "macports" || echo "homebrew")"
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

# Create LaunchAgent
progress "Creating LaunchAgent..."
mkdir -p "$HOME/Library/LaunchAgents"

cat > "$HOME/Library/LaunchAgents/com.openclaw.daemon.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.openclaw.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>$BIN_DIR/openclaw</string>
        <string>serve</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
EOF

success "LaunchAgent created"

echo ""
echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}Installation Complete!${NC}"
echo -e "${GREEN}================================${NC}"
echo ""
echo -e "OpenClaw $VERSION installed to: ${BLUE}$INSTALL_DIR${NC}"
if [ "$USE_MACPORTS" = true ]; then
    echo -e "Package manager: ${CYAN}MacPorts${NC}"
else
    echo -e "Package manager: ${CYAN}Homebrew${NC}"
fi
echo ""
echo -e "Next steps:"
echo -e "  1. Restart terminal or run: ${YELLOW}source $SHELL_CONFIG${NC}"
echo -e "  2. Verify: ${YELLOW}openclaw --version${NC}"
echo -e "  3. Start: ${YELLOW}openclaw serve${NC}"
echo ""
