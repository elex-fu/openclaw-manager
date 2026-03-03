#!/bin/bash
#
# OpenClaw Installation Script for macOS 10.15 Catalina
# Last updated: 2026-02-26
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="$HOME/.openclaw"
BIN_DIR="$INSTALL_DIR/bin"
VERSION="1.0.0"
MIN_RUST_VERSION="1.70.0"

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}OpenClaw Installer for macOS 10.15 Catalina${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

# Check if running on Catalina
if ! sw_vers -productVersion | grep -q "^10.15"; then
    echo -e "${YELLOW}Warning: This script is designed for macOS 10.15 Catalina${NC}"
    echo -e "${YELLOW}Your system: $(sw_vers -productVersion)${NC}"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check architecture (Catalina supports both Intel and first Apple Silicon)
ARCH=$(uname -m)
echo -e "${BLUE}Detected architecture: $ARCH${NC}"

if [ "$ARCH" = "arm64" ]; then
    echo -e "${YELLOW}Note: Apple Silicon detected on Catalina${NC}"
    echo -e "${YELLOW}Some features may have limited support${NC}"
fi

# Function to print progress
progress() {
    echo -e "${BLUE}[*]${NC} $1"
}

# Function to print success
success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

# Function to print error
error() {
    echo -e "${RED}[✗]${NC} $1"
}

# Function to print warning
warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# Check prerequisites
progress "Checking prerequisites..."

# Check Xcode Command Line Tools
if ! xcode-select -p &>/dev/null; then
    error "Xcode Command Line Tools not found"
    echo "Installing Xcode Command Line Tools..."
    xcode-select --install
    echo "Please complete the installation and run this script again"
    exit 1
fi
success "Xcode Command Line Tools installed"

# Check Homebrew (optional but recommended)
if ! command -v brew &>/dev/null; then
    warning "Homebrew not found. Installing..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi
success "Homebrew available"

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

# Download OpenClaw binary
progress "Downloading OpenClaw $VERSION..."

# Determine download URL based on architecture
if [ "$ARCH" = "arm64" ]; then
    DOWNLOAD_URL="https://github.com/openclai/openclaw/releases/download/v${VERSION}/openclaw-${VERSION}-aarch64-apple-darwin.tar.gz"
else
    DOWNLOAD_URL="https://github.com/openclai/openclaw/releases/download/v${VERSION}/openclaw-${VERSION}-x86_64-apple-darwin.tar.gz"
fi

echo "Download URL: $DOWNLOAD_URL"

# Download and extract
TEMP_DIR=$(mktemp -d)
curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_DIR/openclaw.tar.gz"
tar -xzf "$TEMP_DIR/openclaw.tar.gz" -C "$TEMP_DIR"

# Install binary
if [ -f "$TEMP_DIR/openclaw" ]; then
    cp "$TEMP_DIR/openclaw" "$BIN_DIR/"
    chmod +x "$BIN_DIR/openclaw"
fi

# Cleanup
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
  platform: "macos-10.15"
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
    description: "一个通用的 AI 助手"
    model_id: "default-gpt4"
    system_prompt: "You are a helpful assistant."
    skills: []
    enabled: true
settings:
  log_level: "info"
  auto_update: true
  theme: "system"
  language: "zh-CN"
  custom_vars: {}
EOF

success "Configuration created"

# Add to PATH
progress "Adding OpenClaw to PATH..."

SHELL_CONFIG=""
if [ -f "$HOME/.zshrc" ]; then
    SHELL_CONFIG="$HOME/.zshrc"
elif [ -f "$HOME/.bash_profile" ]; then
    SHELL_CONFIG="$HOME/.bash_profile"
elif [ -f "$HOME/.bashrc" ]; then
    SHELL_CONFIG="$HOME/.bashrc"
fi

if [ -n "$SHELL_CONFIG" ]; then
    if ! grep -q "\.openclaw/bin" "$SHELL_CONFIG"; then
        echo "" >> "$SHELL_CONFIG"
        echo "# OpenClaw" >> "$SHELL_CONFIG"
        echo 'export PATH="$HOME/.openclaw/bin:$PATH"' >> "$SHELL_CONFIG"
        success "Added to PATH in $SHELL_CONFIG"
    else
        success "Already in PATH"
    fi
else
    warning "Could not find shell config file"
fi

# Create LaunchAgent for auto-start (optional)
progress "Creating LaunchAgent..."

LAUNCHAGENT_DIR="$HOME/Library/LaunchAgents"
mkdir -p "$LAUNCHAGENT_DIR"

cat > "$LAUNCHAGENT_DIR/com.openclaw.daemon.plist" << EOF
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
    <key>StandardOutPath</key>
    <string>$INSTALL_DIR/logs/daemon.log</string>
    <key>StandardErrorPath</key>
    <string>$INSTALL_DIR/logs/daemon-error.log</string>
</dict>
</plist>
EOF

mkdir -p "$INSTALL_DIR/logs"

success "LaunchAgent created"

# Installation complete
echo ""
echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}Installation Complete!${NC}"
echo -e "${GREEN}================================${NC}"
echo ""
echo -e "OpenClaw $VERSION has been installed to:"
echo -e "  ${BLUE}$INSTALL_DIR${NC}"
echo ""
echo -e "To get started:"
echo -e "  1. Restart your terminal or run: ${YELLOW}source $SHELL_CONFIG${NC}"
echo -e "  2. Run: ${YELLOW}openclaw --version${NC}"
echo -e "  3. Start the service: ${YELLOW}openclaw serve${NC}"
echo -e "  4. Open dashboard: ${YELLOW}openclaw dashboard${NC}"
echo ""
echo -e "For help, run: ${YELLOW}openclaw --help${NC}"
echo ""

# Catalina-specific notes
echo -e "${YELLOW}macOS Catalina (10.15) Notes:${NC}"
echo -e "  - If you see 'cannot be opened because the developer cannot be verified'"
echo -e "    Go to System Preferences > Security & Privacy > General and click 'Allow Anyway'"
echo -e "  - For Apple Silicon Macs on Catalina, Rosetta 2 is recommended"
echo ""
