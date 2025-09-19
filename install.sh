#!/usr/bin/env bash
set -euo pipefail

# Substrate MCP Server Installer
#
# Usage:
#   curl -sSfL https://raw.githubusercontent.com/Moonsong-Labs/substrate-mcp/main/install.sh | bash

REPO_URL="https://github.com/Moonsong-Labs/substrate-mcp"
BIN_NAME="substrate-mcp"
INSTALL_DIR="$HOME/.substrate-mcp/bin"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

detect_platform() {
    local arch=""
    local os=""
    
    # Detect OS
    case "$(uname -s)" in
        Darwin)
            os="macos"
            ;;
        Linux)
            os="linux"
            ;;
        *)
            error "Unsupported operating system: $(uname -s)"
            error "Please use 'cargo install --git $REPO_URL' instead"
            exit 1
            ;;
    esac
    
    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            arch="aarch64"
            ;;
        *)
            error "Unsupported architecture: $(uname -m)"
            error "Please use 'cargo install --git $REPO_URL' instead"
            exit 1
            ;;
    esac
    
    echo "${os}-${arch}"
}

# Function to get the latest release version
get_latest_version() {
    local version
    version=$(curl -s "https://api.github.com/repos/Moonsong-Labs/substrate-mcp/releases/latest" | \
              grep '"tag_name":' | \
              sed -E 's/.*"([^"]+)".*/\1/')
    
    if [ -z "$version" ]; then
        error "Failed to get latest version from GitHub API"
        error "Please use 'cargo install --git $REPO_URL' instead"
        exit 1
    fi
    
    echo "$version"
}


# Function to download and extract binary
download_and_extract() {
    local platform="$1"
    local version="$2"
    local archive_name="${BIN_NAME}-${platform}.tar.gz"
    local download_url="${REPO_URL}/releases/download/${version}/${archive_name}"
    local temp_dir
    temp_dir=$(mktemp -d)
    
    info "Downloading ${BIN_NAME} ${version} for ${platform}..."
    
    if ! curl -L --progress-bar "$download_url" -o "$temp_dir/$archive_name"; then
        error "Failed to download binary from $download_url"
        error "Please check if the release exists or use 'cargo install --git $REPO_URL' instead"
        exit 1
    fi
    
    
    info "Extracting binary..."
    if ! tar -xzf "$temp_dir/$archive_name" -C "$temp_dir"; then
        error "Failed to extract archive"
        exit 1
    fi
    
    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"
    
    # Find the actual binary name in the extracted files
    local binary_path
    binary_path=$(find "$temp_dir" -name "${BIN_NAME}*" -type f | head -1)
    
    if [ -z "$binary_path" ]; then
        error "Could not find binary in extracted archive"
        exit 1
    fi
    
    # Move binary to install directory with the correct name
    if ! mv "$binary_path" "$INSTALL_DIR/$BIN_NAME"; then
        error "Failed to move binary to $INSTALL_DIR"
        exit 1
    fi
    
    # Make binary executable
    chmod +x "$INSTALL_DIR/$BIN_NAME"
    
    # Cleanup
    rm -rf "$temp_dir"
    
    success "Binary installed to $INSTALL_DIR/$BIN_NAME"
}

# Function to setup PATH
setup_path() {
    local shell_profile=""
    local shell_name
    shell_name=$(basename "$SHELL")
    
    # Check if already in PATH
    if echo "$PATH" | grep -q "$INSTALL_DIR"; then
        info "$INSTALL_DIR is already in your PATH"
        return 0
    fi
    
    # Determine shell profile file
    case "$shell_name" in
        bash)
            if [ -f "$HOME/.bash_profile" ]; then
                shell_profile="$HOME/.bash_profile"
            elif [ -f "$HOME/.bashrc" ]; then
                shell_profile="$HOME/.bashrc"
            else
                shell_profile="$HOME/.bash_profile"
            fi
            ;;
        zsh)
            shell_profile="$HOME/.zshrc"
            ;;
        fish)
            # Fish has a different syntax
            if command -v fish >/dev/null 2>&1; then
                info "Adding $INSTALL_DIR to fish PATH..."
                fish -c "fish_add_path $INSTALL_DIR" 2>/dev/null || true
                success "Added $INSTALL_DIR to fish PATH"
                return 0
            fi
            ;;
        *)
            warn "Unknown shell: $shell_name"
            warn "Please manually add $INSTALL_DIR to your PATH"
            return 1
            ;;
    esac
    
    # Add to PATH for bash/zsh
    if [ -n "$shell_profile" ]; then
        info "Adding $INSTALL_DIR to PATH in $shell_profile..."
        echo "" >> "$shell_profile"
        echo "# substrate-mcp" >> "$shell_profile"
        echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$shell_profile"
        success "Added $INSTALL_DIR to PATH in $shell_profile"
        info "Please restart your shell or run: source $shell_profile"
    fi
}


# Main installation function
main() {
    info "Starting substrate-mcp installation..."
    
    # Check if curl is available
    if ! command -v curl >/dev/null 2>&1; then
        error "curl is required but not installed"
        error "Please install curl and try again"
        exit 1
    fi
    
    # Check if tar is available
    if ! command -v tar >/dev/null 2>&1; then
        error "tar is required but not installed"
        error "Please install tar and try again"
        exit 1
    fi
    
    local platform version
    
    # Detect platform
    platform=$(detect_platform)
    info "Detected platform: $platform"
    
    # Get latest version
    info "Fetching latest release information..."
    version=$(get_latest_version)
    info "Latest version: $version"
    
    # Download and extract
    download_and_extract "$platform" "$version"
    
    setup_path
    
    echo ""
    success "substrate-mcp installation complete!"
}

# Run main function
main "$@"
