#!/bin/bash
# AOF Installation Script
# Usage: curl -sSL https://aof.sh/install.sh | bash
# or: curl -sSL https://aof.sh/install.sh | bash -s -- --version v0.1.0

set -e

# Configuration
REPO="agenticopsorg/aof"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
GITHUB_RELEASE_API="https://api.github.com/repos/$REPO/releases"
VERSION="${1:-latest}"
VERBOSE="${VERBOSE:-0}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log() {
    echo -e "${BLUE}ℹ${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1" >&2
    exit 1
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

debug() {
    if [ "$VERBOSE" = "1" ]; then
        echo -e "${BLUE}[DEBUG]${NC} $1"
    fi
}

# Detect OS and architecture
detect_platform() {
    local os=$(uname -s)
    local arch=$(uname -m)

    case "$os" in
        Darwin)
            case "$arch" in
                x86_64) echo "macos-x86_64" ;;
                arm64) echo "macos-aarch64" ;;
                *) error "Unsupported architecture: $arch" ;;
            esac
            ;;
        Linux)
            case "$arch" in
                x86_64) echo "linux-x86_64" ;;
                aarch64) echo "linux-aarch64" ;;
                armv7l) echo "linux-armv7" ;;
                *) error "Unsupported architecture: $arch" ;;
            esac
            ;;
        MINGW* | MSYS* | CYGWIN*)
            case "$arch" in
                x86_64) echo "windows-x86_64" ;;
                i686) echo "windows-i686" ;;
                *) error "Unsupported architecture: $arch" ;;
            esac
            ;;
        *)
            error "Unsupported OS: $os"
            ;;
    esac
}

# Get the latest release version
get_latest_version() {
    local latest_url="$GITHUB_RELEASE_API/latest"
    debug "Fetching latest version from: $latest_url"

    if ! command -v jq &> /dev/null; then
        # Fallback to grep if jq is not available
        curl -sSL "$latest_url" | grep -o '"tag_name":"[^"]*' | cut -d'"' -f4 | head -1
    else
        curl -sSL "$latest_url" | jq -r '.tag_name'
    fi
}

# Get download URL for a specific version
get_download_url() {
    local version="$1"
    local platform="$2"

    # Build filename (without version, matching GitHub release assets)
    local binary_name="aofctl-${platform}"

    # GitHub releases URL pattern
    local base_url="https://github.com/$REPO/releases/download"
    local tag="$version"

    echo "${base_url}/${tag}/${binary_name}"
}

# Download binary
download_binary() {
    local url="$1"
    local output="$2"

    log "Downloading from: $url"

    if ! curl -sSL --progress-bar "$url" -o "$output"; then
        error "Failed to download binary from $url"
    fi

    debug "Binary downloaded to: $output"
}

# Verify checksum if available
verify_checksum() {
    local binary="$1"
    local checksum_url="${2}.sha256"

    log "Verifying checksum..."

    # Try to download checksum
    if ! curl -sSL "$checksum_url" -o "${binary}.sha256" 2>/dev/null; then
        warn "Checksum file not found, skipping verification"
        return 0
    fi

    # Verify checksum
    if command -v sha256sum &> /dev/null; then
        if ! sha256sum -c "${binary}.sha256" >/dev/null 2>&1; then
            error "Checksum verification failed!"
        fi
    elif command -v shasum &> /dev/null; then
        if ! shasum -a 256 -c "${binary}.sha256" >/dev/null 2>&1; then
            error "Checksum verification failed!"
        fi
    else
        warn "sha256sum/shasum not found, skipping verification"
    fi

    rm -f "${binary}.sha256"
    success "Checksum verified"
}

# Install binary
install_binary() {
    local binary="$1"
    local install_path="$2"
    local install_dir=$(dirname "$install_path")

    # Create install directory if it doesn't exist
    if ! mkdir -p "$install_dir" 2>/dev/null; then
        # Try with sudo if regular mkdir fails
        if ! sudo mkdir -p "$install_dir"; then
            error "Failed to create installation directory: $install_dir"
        fi
    fi

    # Copy binary (try without sudo first)
    if ! cp "$binary" "$install_path" 2>/dev/null; then
        # Try with sudo if regular cp fails
        if ! sudo cp "$binary" "$install_path"; then
            error "Failed to install binary to $install_path"
        fi
        # If we used sudo, also need sudo for chmod
        sudo chmod +x "$install_path"
    else
        # Make executable
        chmod +x "$install_path"
    fi

    success "Installed aofctl to $install_path"
}

# Check if install directory is in PATH
check_path() {
    local install_path="$1"

    # Get the directory containing the binary
    local bin_dir=$(dirname "$install_path")

    if [[ ":$PATH:" == *":$bin_dir:"* ]]; then
        return 0
    else
        return 1
    fi
}

# Main installation flow
main() {
    echo -e "${BLUE}"
    echo "╔════════════════════════════════════════════════╗"
    echo "║  AOF (Agentic Ops Framework) Installer        ║"
    echo "╚════════════════════════════════════════════════╝"
    echo -e "${NC}"

    # Detect platform
    log "Detecting platform..."
    platform=$(detect_platform)
    success "Detected: $platform"

    # Get version
    if [ "$VERSION" = "latest" ]; then
        log "Fetching latest version..."
        VERSION=$(get_latest_version)
        if [ -z "$VERSION" ]; then
            error "Failed to get latest version"
        fi
    fi
    success "Version: $VERSION"

    # Get download URL
    log "Preparing download..."
    download_url=$(get_download_url "$VERSION" "$platform")
    debug "Download URL: $download_url"

    # Create temp directory
    temp_dir=$(mktemp -d)
    trap "rm -rf $temp_dir" EXIT

    binary_path="$temp_dir/aofctl"

    # Download binary
    download_binary "$download_url" "$binary_path"
    success "Download complete"

    # Verify checksum
    verify_checksum "$binary_path" "$download_url"

    # Install binary
    install_dir_path="$INSTALL_DIR/aofctl"
    install_binary "$binary_path" "$install_dir_path"

    # Check PATH
    echo ""
    if check_path "$install_dir_path"; then
        success "aofctl is in your PATH"
    else
        warn "aofctl is installed but not in your PATH"
        echo ""
        echo "Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo ""
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
        echo ""
    fi

    # Verify installation
    echo ""
    log "Verifying installation..."

    # Check if binary exists and is executable
    if [[ -x "$install_dir_path" ]]; then
        local installed_version=$("$install_dir_path" --version 2>/dev/null || echo "unknown")
        success "Installation successful!"
        echo ""
        echo "  Binary: $install_dir_path"
        echo "  Version: $installed_version"
        echo ""
        echo "Get started:"
        echo "  aofctl run agent --help"
        echo "  aofctl get agents"
        echo ""
    else
        error "Installation verification failed - binary not found or not executable at $install_dir_path"
    fi
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=1
            shift
            ;;
        --help)
            echo "AOF Installation Script"
            echo ""
            echo "Usage: curl -sSL https://aof.sh/install.sh | bash [options]"
            echo ""
            echo "Options:"
            echo "  --version VERSION      Install specific version (default: latest)"
            echo "  --install-dir DIR      Installation directory (default: /usr/local/bin)"
            echo "  --verbose              Enable verbose output"
            echo "  --help                 Show this help message"
            echo ""
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            ;;
    esac
done

# Run installation
main
