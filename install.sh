#!/bin/bash
# Leaf Package Manager Installation Script (Linux only)
# Usage: curl -sSL https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash
# Usage: wget -qO- https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash
# Usage with version: curl -sSL https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash -s -- --version v1.2.3

set -e

# Check if running on Linux
if [[ "$(uname -s)" != "Linux" ]]; then
    echo "❌ Leaf package manager only supports Linux."
    echo "Your OS: $(uname -s)"
    exit 1
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
            echo "  --version VERSION    Install specific version (e.g., v1.2.3, v1.2.3-beta)"
            echo "  --prerelease        Install latest prerelease version"
            echo "  --help              Show this help message"
            echo ""
            echo "By default, installs the latest stable release."
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information."
            exit 1
            ;;
    esac
done

echo "🍃 Installing Leaf Package Manager for Linux..."

# Clean up old binary from previous updates
if [ -f "$BIN_DIR/leaf.old" ]; then
    rm -f "$BIN_DIR/leaf.old"
fi

# Check if leaf is already installed and get its version
if [ -f "$BIN_DIR/leaf" ]; then
    CURRENT_VERSION=$("$BIN_DIR/leaf" --version | awk '{print $NF}')
    echo "[!] Leaf is already installed (version $CURRENT_VERSION)."
    echo "If you want to completely remove it first, run: leaf nuke --confirmed"
    echo "Continuing with installation/update..."
else
    CURRENT_VERSION=""
fi

# Detect architecture
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64)
        PLATFORM="linux-x86_64"
        ;;
    aarch64|arm64)
        PLATFORM="linux-aarch64"
        ;;
    *)
        echo "❌ Unsupported architecture: $ARCH"
        echo "Leaf only supports x86_64 and aarch64 on Linux."
        exit 1
        ;;
esac

echo "[-] Detected platform: $PLATFORM"

# Create directories
mkdir -p "$LEAF_DIR" "$BIN_DIR"

# Function to get release info
get_release_info() {
    local release_type="$1"  # "latest" or specific version
    local api_url
    
    if [ "$release_type" = "latest" ]; then
        if [ "$INSTALL_PRERELEASE" = true ]; then
            # Get all releases and find the latest (including prereleases)
            api_url="https://api.github.com/repos/$REPO/releases"
        else
            # Get only the latest stable release
            api_url="https://api.github.com/repos/$REPO/releases/latest"
        fi
    else
        # Get specific release by tag
        api_url="https://api.github.com/repos/$REPO/releases/tags/$release_type"
    fi
    
    if command -v curl >/dev/null 2>&1; then
        local response=$(curl -s "$api_url")
    elif command -v wget >/dev/null 2>&1; then
        local response=$(wget -qO- "$api_url")
    else
        echo "❌ Neither curl nor wget found. Please install one of them."
        exit 1
    fi
    
    if [ "$release_type" = "latest" ] && [ "$INSTALL_PRERELEASE" = true ]; then
        # Parse array response and get the first (latest) release
        echo "$response" | sed 's/^\[//' | sed 's/\]$//' | sed 's/},{/}\n{/g' | head -1
    else
        echo "$response"
    fi
}

# Get the release info
echo "[-] Finding release..."

if [ -n "$REQUESTED_VERSION" ]; then
    echo "[-] Requesting specific version: $REQUESTED_VERSION"
    RELEASE_INFO=$(get_release_info "$REQUESTED_VERSION")
    
    # Check if the release exists
    if echo "$RELEASE_INFO" | grep -q '"message": "Not Found"'; then
        echo "❌ Version $REQUESTED_VERSION not found."
        echo "Please check the available releases at: https://github.com/$REPO/releases"
        exit 1
    fi
else
    RELEASE_INFO=$(get_release_info "latest")
fi

# Parse release information
DOWNLOAD_URL=$(echo "$RELEASE_INFO" | grep "browser_download_url.*leaf-$PLATFORM.tar.gz" | head -1 | cut -d '"' -f 4)
LATEST_VERSION_TAG=$(echo "$RELEASE_INFO" | grep '"tag_name":' | head -1 | cut -d '"' -f 4)
LATEST_VERSION=${LATEST_VERSION_TAG#v} # Strip 'v' prefix
IS_PRERELEASE=$(echo "$RELEASE_INFO" | grep '"prerelease":' | head -1 | cut -d ':' -f 2 | tr -d ' ,' | tr -d '"')

if [ -z "$DOWNLOAD_URL" ]; then
    echo "[!] Could not find release for platform $PLATFORM"
    echo "Available releases:"
    if command -v curl >/dev/null 2>&1; then
        if [ -n "$REQUESTED_VERSION" ]; then
            curl -s "https://api.github.com/repos/$REPO/releases/tags/$REQUESTED_VERSION" | \
                grep "browser_download_url.*tar.gz" | \
                cut -d '"' -f 4 | \
                sed 's/.*leaf-\(.*\)\.tar\.gz/  - \1/'
        else
            curl -s "https://api.github.com/repos/$REPO/releases/latest" | \
                grep "browser_download_url.*tar.gz" | \
                cut -d '"' -f 4 | \
                sed 's/.*leaf-\(.*\)\.tar\.gz/  - \1/'
        fi
    else
        if [ -n "$REQUESTED_VERSION" ]; then
            wget -qO- "https://api.github.com/repos/$REPO/releases/tags/$REQUESTED_VERSION" | \
                grep "browser_download_url.*tar.gz" | \
                cut -d '"' -f 4 | \
                sed 's/.*leaf-\(.*\)\.tar\.gz/  - \1/'
        else
            wget -qO- "https://api.github.com/repos/$REPO/releases/latest" | \
                grep "browser_download_url.*tar.gz" | \
                cut -d '"' -f 4 | \
                sed 's/.*leaf-\(.*\)\.tar\.gz/  - \1/'
        fi
    fi
    exit 1
fi

# Show release information
if [ "$IS_PRERELEASE" = "true" ]; then
    echo "[-] Found prerelease version: $LATEST_VERSION"
    echo "[-] ⚠️  WARNING: This is a prerelease version that may contain bugs"
else
    echo "[-] Found stable version: $LATEST_VERSION"
fi

# Compare versions
if [ "$CURRENT_VERSION" = "$LATEST_VERSION" ]; then
    echo "✅ You are already on version $LATEST_VERSION."
    exit 0
fi

echo "[-] Target version: $LATEST_VERSION. Current version: $CURRENT_VERSION."
echo "[-] Downloading leaf binary..."
TEMP_DIR=$(mktemp -d)
TEMP_FILE="$TEMP_DIR/leaf-$PLATFORM.tar.gz"

# Download with progress bar
if command -v curl >/dev/null 2>&1; then
    curl -L --progress-bar "$DOWNLOAD_URL" -o "$TEMP_FILE"
elif command -v wget >/dev/null 2>&1; then
    wget --progress=bar:force:noscroll "$DOWNLOAD_URL" -O "$TEMP_FILE"
else
    echo "❌ Neither curl nor wget found. Please install one of them."
    exit 1
fi

# Extract and install
echo "[-] Extracting binary..."
cd "$TEMP_DIR"
tar -xzf "leaf-$PLATFORM.tar.gz"

NEW_LEAF_BIN="$TEMP_DIR/leaf"
CURRENT_LEAF_BIN="$BIN_DIR/leaf"
OLD_LEAF_BIN="$BIN_DIR/leaf.old"

if [ -f "$CURRENT_LEAF_BIN" ]; then
    echo "[-] Replacing current binary..."
    # Rename current binary. Allowed even if it's running.
    mv "$CURRENT_LEAF_BIN" "$OLD_LEAF_BIN"
fi

# Move new binary into place
mv "$NEW_LEAF_BIN" "$CURRENT_LEAF_BIN"
chmod +x "$CURRENT_LEAF_BIN"

# Download package definitions
echo "[-] Downloading package definitions..."
if command -v curl >/dev/null 2>&1; then
    curl -sSL "https://raw.githubusercontent.com/$REPO/main/packages.json" -o "$LEAF_DIR/packages.json"
elif command -v wget >/dev/null 2>&1; then
    wget -qO "$LEAF_DIR/packages.json" "https://raw.githubusercontent.com/$REPO/main/packages.json"
fi

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
    echo "✅ Added $BIN_DIR to PATH in $SHELL_RC"
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
    echo ""
    echo "[-] Leaf Package Manager installed/updated successfully!"
    echo "[-] Version: $VERSION_INFO"
    if [ "$IS_PRERELEASE" = "true" ]; then
        echo "[-] ⚠️  You are running a prerelease version"
    fi
else
    echo "[!] Installation completed but leaf command test failed"
fi

echo ""
echo "Usage:"
echo "  leaf install <package>        # Install a package"
echo "  leaf remove <package>         # Remove a package"
echo "  leaf list                     # List installed packages"
echo "  leaf search <term>            # Search available packages"
echo "  leaf update                   # Update package list"
echo "  leaf self-update              # Update leaf itself"
echo "  leaf self-update --version v1.2.3  # Update to specific version"
echo "  leaf nuke --confirmed         # Remove everything (DESTRUCTIVE)"
echo ""
echo "To get started, restart your terminal or run:"
echo "  source $SHELL_RC"
echo ""
echo "Then try: leaf install nvim"
