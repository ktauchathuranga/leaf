#!/bin/bash
# Leaf Package Manager Installation Script (Linux only)
# Usage: curl -sSL https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash
# Usage: wget -qO- https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash
# Usage with version: curl -sSL https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash -s -- --version v1.2.3
# Usage with prerelease: curl -sSL https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash -s -- --prerelease

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print functions
info() { echo -e "${BLUE}[-] $1${NC}"; }
success() { echo -e "${GREEN}[✓] $1${NC}"; }
warning() { echo -e "${YELLOW}[!] $1${NC}"; }
error() { echo -e "${RED}[!] $1${NC}"; exit 1; }

# Check if running on Linux
if [[ "$(uname -s)" != "Linux" ]]; then
    error "Leaf package manager only supports Linux. Your OS: $(uname -s)"
fi

LEAF_DIR="$HOME/.local/leaf"
BIN_DIR="$HOME/.local/bin"
REPO="ktauchathuranga/leaf"
REQUESTED_VERSION=""
INSTALL_PRERELEASE=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --version)
            REQUESTED_VERSION="$2"
            shift 2
            ;;
        --prerelease)
            INSTALL_PRERELEASE=true
            shift
            ;;
        --help)
            echo "Leaf Package Manager Installation Script"
            echo ""
            echo "Usage:"
            echo "  curl -sSL https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash"
            echo "  curl -sSL https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash -s -- --version v1.2.3"
            echo "  curl -sSL https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash -s -- --prerelease"
            echo ""
            echo "Options:"
            echo "  --version VERSION  Install specific version (e.g., v1.2.3, v1.2.3-beta)"
            echo "  --prerelease      Install latest prerelease version (alpha, beta, rc)"
            echo "  --help            Show this help message"
            echo ""
            echo "By default, installs the latest stable release."
            exit 0
            ;;
        *)
            error "Unknown option: $1. Use --help for usage information."
            ;;
    esac
done

# Validate arguments
if [ -n "$REQUESTED_VERSION" ] && [ "$INSTALL_PRERELEASE" = true ]; then
    error "Cannot specify both --version and --prerelease"
fi

info "Installing Leaf Package Manager for Linux..."

# Clean up old binary from previous updates
if [ -f "$BIN_DIR/leaf.old" ]; then
    rm -f "$BIN_DIR/leaf.old"
fi

# Check if leaf is already installed and get its version
if [ -f "$BIN_DIR/leaf" ]; then
    CURRENT_VERSION=$("$BIN_DIR/leaf" --version | grep -oP '\d+\.\d+\.\d+(?:-(?:alpha|beta|rc)\d*)?')
    warning "Leaf is already installed (version $CURRENT_VERSION)."
    warning "If you want to completely remove it first, run: leaf nuke --confirmed"
    info "Continuing with installation/update..."
else
    CURRENT_VERSION=""
fi

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64)
        PLATFORM="linux-x86_64"
        ASSET_NAME="leaf-linux-x86_64.tar.gz"
        ;;
    aarch64|arm64)
        PLATFORM="linux-aarch64"
        ASSET_NAME="leaf-linux-aarch64.tar.gz"
        ;;
    *)
        error "Unsupported architecture: $ARCH. Leaf only supports x86_64 and aarch64 on Linux."
        ;;
esac

info "Detected platform: $PLATFORM"

# Create directories
mkdir -p "$LEAF_DIR" "$BIN_DIR"

# Function to fetch data
fetch_data() {
    local url="$1"
    if command -v curl >/dev/null 2>&1; then
        curl -sSL "$url"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "$url"
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
}

# Function to get release info
get_release_info() {
    local release_type="$1" # "latest" or specific version
    local api_url

    if [ "$release_type" = "latest" ]; then
        if [ "$INSTALL_PRERELEASE" = true ]; then
            api_url="https://api.github.com/repos/$REPO/releases"
        else
            api_url="https://api.github.com/repos/$REPO/releases/latest"
        fi
    else
        api_url="https://api.github.com/repos/$REPO/releases/tags/$release_type"
    fi

    fetch_data "$api_url"
}

# Get the release info
info "Finding release..."
if [ -n "$REQUESTED_VERSION" ]; then
    info "Requesting specific version: $REQUESTED_VERSION"
    RELEASE_INFO=$(get_release_info "$REQUESTED_VERSION")
    # Check if the release exists
    if echo "$RELEASE_INFO" | grep -q '"message": "Not Found"'; then
        error "Version $REQUESTED_VERSION not found. Please check the available releases at: https://github.com/$REPO/releases"
    fi
else
    RELEASE_INFO=$(get_release_info "latest")
fi

# For prereleases, find the latest one by parsing and sorting by published_at
if [ -n "$REQUESTED_VERSION" ]; then
    SELECTED_RELEASE_INFO="$RELEASE_INFO"
else
    if [ "$INSTALL_PRERELEASE" = true ]; then
        # Extract prerelease entries and sort by published_at
        SELECTED_RELEASE_INFO=$(echo "$RELEASE_INFO" | grep -B100 '"prerelease": true' | grep -A100 '"tag_name":' | sed 's/},{/}\n{/g' | sort -r -k4 -t'"' | grep -m1 '"tag_name":')
        if [ -z "$SELECTED_RELEASE_INFO" ]; then
            error "No prerelease versions found. Please check the available releases at: https://github.com/$REPO/releases"
        fi
        # Re-fetch the full release info for the selected tag
        LATEST_VERSION_TAG=$(echo "$SELECTED_RELEASE_INFO" | grep '"tag_name":' | cut -d '"' -f 4)
        if [ -n "$LATEST_VERSION_TAG" ]; then
            SELECTED_RELEASE_INFO=$(get_release_info "$LATEST_VERSION_TAG")
        else
            error "Failed to parse prerelease version. Please check the available releases at: https://github.com/$REPO/releases"
        fi
    else
        SELECTED_RELEASE_INFO="$RELEASE_INFO"
    fi
fi

# Parse release information
DOWNLOAD_URL=$(echo "$SELECTED_RELEASE_INFO" | grep "browser_download_url.*$ASSET_NAME" | head -1 | cut -d '"' -f 4)
LATEST_VERSION_TAG=$(echo "$SELECTED_RELEASE_INFO" | grep '"tag_name":' | head -1 | cut -d '"' -f 4)
LATEST_VERSION=${LATEST_VERSION_TAG#v} # Strip 'v' prefix
IS_PRERELEASE=$(echo "$SELECTED_RELEASE_INFO" | grep '"prerelease":' | head -1 | cut -d ':' -f 2 | tr -d ' ,' | tr -d '"')

if [ -z "$DOWNLOAD_URL" ]; then
    error "Could not find release asset $ASSET_NAME for platform $PLATFORM in release $LATEST_VERSION_TAG. Please check the available releases at: https://github.com/$REPO/releases"
fi

# Show release information
if [ "$IS_PRERELEASE" = "true" ]; then
    warning "Found prerelease version: $LATEST_VERSION"
    warning "⚠️ WARNING: This is a prerelease version that may contain bugs"
else
    info "Found stable version: $LATEST_VERSION"
fi

# Compare versions
if [ -n "$CURRENT_VERSION" ] && [ "$CURRENT_VERSION" = "$LATEST_VERSION" ]; then
    success "You are already on version $LATEST_VERSION."
    exit 0
fi

info "Target version: $LATEST_VERSION. Current version: $CURRENT_VERSION."

# Download with progress bar
info "Downloading leaf binary..."
TEMP_DIR=$(mktemp -d)
TEMP_FILE="$TEMP_DIR/$ASSET_NAME"
if command -v curl >/dev/null 2>&1; then
    curl -L --progress-bar "$DOWNLOAD_URL" -o "$TEMP_FILE"
elif command -v wget >/dev/null 2>&1; then
    wget --progress=bar:force:noscroll "$DOWNLOAD_URL" -O "$TEMP_FILE"
else
    error "Neither curl nor wget found. Please install one of them."
fi

# Extract and install
info "Extracting binary..."
cd "$TEMP_DIR"
tar -xzf "$ASSET_NAME"
NEW_LEAF_BIN="$TEMP_DIR/leaf"
CURRENT_LEAF_BIN="$BIN_DIR/leaf"
OLD_LEAF_BIN="$BIN_DIR/leaf.old"

if [ -f "$CURRENT_LEAF_BIN" ]; then
    info "Replacing current binary..."
    mv "$CURRENT_LEAF_BIN" "$OLD_LEAF_BIN"
fi

mv "$NEW_LEAF_BIN" "$CURRENT_LEAF_BIN"
chmod +x "$CURRENT_LEAF_BIN"

# Download package definitions
info "Downloading package definitions..."
fetch_data "https://raw.githubusercontent.com/$REPO/main/packages.json" > "$LEAF_DIR/packages.json"

# Add to PATH if not already there
SHELL_RC=""
if [ -n "$BASH_VERSION" ]; then
    SHELL_RC="$HOME/.bashrc"
elif [ -n "$ZSH_VERSION" ]; then
    SHELL_RC="$HOME/.zshrc"
else
    SHELL_RC="$HOME/.profile"
fi
if [ -f "$SHELL_RC" ] && ! grep -q "$BIN_DIR" "$SHELL_RC"; then
    echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> "$SHELL_RC"
    success "Added $BIN_DIR to PATH in $SHELL_RC"
fi

# Create/update leaf config
cat > "$LEAF_DIR/config.json" << EOF
{
    "version": "$LATEST_VERSION",
    "install_dir": "$LEAF_DIR",
    "bin_dir": "$BIN_DIR",
    "packages_dir": "$LEAF_DIR/packages",
    "cache_dir": "$LEAF_DIR/cache"
}
EOF
mkdir -p "$LEAF_DIR/packages" "$LEAF_DIR/cache"

# Cleanup
rm -rf "$TEMP_DIR"

# Test installation
if "$BIN_DIR/leaf" --version >/dev/null 2>&1; then
    VERSION_INFO=$("$BIN_DIR/leaf" --version)
    success "Leaf Package Manager installed/updated successfully!"
    success "Version: $VERSION_INFO"
    if [ "$IS_PRERELEASE" = "true" ]; then
        warning "⚠️ You are running a prerelease version"
    fi
else
    error "Installation completed but leaf command test failed"
fi

echo ""
echo "Usage:"
echo "  leaf install <package>        # Install a package"
echo "  leaf remove <package>         # Remove a package"
echo "  leaf list                     # List installed packages"
echo "  leaf search <term>            # Search available packages"
echo "  leaf update                   # Update package list"
echo "  leaf self-update              # Update leaf to latest stable version"
echo "  leaf self-update --version v1.2.3  # Update to specific version"
echo "  leaf self-update --prerelease # Update to latest prerelease"
echo "  leaf nuke --confirmed         # Remove everything (DESTRUCTIVE)"
echo ""
echo "To get started, restart your terminal or run:"
echo "  source $SHELL_RC"
echo ""
echo "Then try: leaf install nvim"
