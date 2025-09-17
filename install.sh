#!/bin/bash
# Leaf Package Manager Installation Script
# Usage: curl -sSL https://raw.githubusercontent.com/yourusername/leaf/main/install.sh | bash

set -e

LEAF_DIR="$HOME/.local/leaf"
BIN_DIR="$HOME/.local/bin"
LEAF_VERSION="v1.0.0"

echo "🍃 Installing Leaf Package Manager..."

# Create directories
mkdir -p "$LEAF_DIR" "$BIN_DIR"

# Download the main leaf script
echo "Downloading leaf binary..."
curl -sSL "https://raw.githubusercontent.com/yourusername/leaf/main/leaf" -o "$BIN_DIR/leaf"
chmod +x "$BIN_DIR/leaf"

# Download package definitions
curl -sSL "https://raw.githubusercontent.com/yourusername/leaf/main/packages.json" -o "$LEAF_DIR/packages.json"

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
    echo "Added $BIN_DIR to PATH in $SHELL_RC"
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

echo ""
echo "🎉 Leaf Package Manager installed successfully!"
echo ""
echo "Usage:"
echo "  leaf install <package>  # Install a package"
echo "  leaf remove <package>   # Remove a package"
echo "  leaf list              # List installed packages"
echo "  leaf search <term>     # Search available packages"
echo "  leaf update            # Update package list"
echo ""
echo "To get started, restart your terminal or run:"
echo "  source $SHELL_RC"
echo ""
echo "Then try: leaf install nvim"