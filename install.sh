#!/bin/bash
# Leaf Package Manager Installation Script (Pre-compiled Binary)
# Usage: curl -sSL https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash

set -e

LEAF_DIR="$HOME/.local/leaf"
BIN_DIR="$HOME/.local/bin"
LEAF_VERSION="latest"
REPO="ktauchathuranga/leaf"

echo "🍃 Installing Leaf Package Manager..."

# Check if leaf is already installed and warn about nuke
if [ -f "$BIN_DIR/leaf" ]; then
    echo "⚠️  Leaf is already installed."
    echo "If you want to completely remove it first, run: leaf nuke --confirmed"
    echo "Continuing with installation/update..."
fi

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)
        case "$ARCH" in
            x86_64)
                PLATFORM="linux-x86_64"
                ;;
            aarch64|arm64)
                PLATFORM="linux-aarch64"
                ;;
            *)
                echo "❌ Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    Darwin*)
        case "$ARCH" in
            x86_64)
                PLATFORM="macos-x86_64"
                ;;
            arm64)
                PLATFORM="macos-aarch64"
                ;;
            *)
                echo "❌ Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    *)
        echo "❌ Unsupported operating system: $OS"
        exit 1
        ;;
esac

echo "📋 Detected platform: $PLATFORM"

# Create directories
mkdir -p "$LEAF_DIR" "$BIN_DIR"

# Get the latest release download URL
if [ "$LEAF_VERSION" = "latest" ]; then
    echo "🔍 Finding latest release..."
    DOWNLOAD_URL=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | \
        grep "browser_download_url.*leaf-$PLATFORM.tar.gz" | \
        cut -d '"' -f 4)
    
    if [ -z "$DOWNLOAD_URL" ]; then
        echo "❌ Could not find release for platform $PLATFORM"
        echo "Available releases:"
        curl -s "https://api.github.com/repos/$REPO/releases/latest" | \
            grep "browser_download_url.*tar.gz" | \
            cut -d '"' -f 4 | \
            sed 's/.*leaf-\(.*\)\.tar\.gz/  - \1/'
        exit 1
    fi
else
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LEAF_VERSION/leaf-$PLATFORM.tar.gz"
fi

echo "📥 Downloading leaf binary..."
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
echo "📦 Extracting binary..."
cd "$TEMP_DIR"
tar -xzf "leaf-$PLATFORM.tar.gz"
cp leaf "$BIN_DIR/leaf"
chmod +x "$BIN_DIR/leaf"

# Download package definitions
echo "📋 Downloading package definitions..."
curl -sSL "https://raw.githubusercontent.com/$REPO/main/packages.json" -o "$LEAF_DIR/packages.json"

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

# Create leaf config
cat > "$LEAF_DIR/config.json" << EOF
{
    "version": "$LEAF_VERSION",
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
    echo "🎉 Leaf Package Manager installed successfully!"
    echo "📍 Version: $VERSION_INFO"
else
    echo "⚠️  Installation completed but leaf command test failed"
fi

echo ""
echo "Usage:"
echo "  leaf install <package>        # Install a package"
echo "  leaf remove <package>         # Remove a package"
echo "  leaf list                     # List installed packages"
echo "  leaf search <term>            # Search available packages"
echo "  leaf update                   # Update package list"
echo "  leaf nuke --confirmed         # Remove everything (DESTRUCTIVE)"
echo ""
echo "To get started, restart your terminal or run:"
echo "  source $SHELL_RC"
echo ""
echo "Then try: leaf install nvim"
