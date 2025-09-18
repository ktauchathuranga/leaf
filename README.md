# 🍃 Leaf Package Manager

A fast, simple, and sudo-free package manager for Linux and macOS, written in Rust.

[![Release](https://img.shields.io/github/v/release/ktauchathuranga/leaf?sort=semver)](https://github.com/ktauchathuranga/leaf/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS-blue.svg)](https://github.com/ktauchathuranga/leaf)

## Features

- **Fast & Lightweight** - Written in Rust for optimal performance
- **No sudo required** - Install packages in your user space
- **Simple commands** - Easy-to-use CLI interface
- **Cross-platform** - Supports Linux (x86_64) and macOS (Intel)
- **Smart package management** - Automatic dependency handling and symlink creation
- **Efficient caching** - Downloads are cached for faster reinstalls
- **Package search** - Find packages with fuzzy search
- **Package registry** - Curated list of popular development tools

## Quick Start

### Installation

Install Leaf with a single command:

```bash
curl -sSL https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash
```

After installation, restart your terminal or run:
```bash
source ~/.bashrc  # or ~/.zshrc
```

### Basic Usage

```bash
# Install a package
leaf install nvim

# List installed packages
leaf list

# Search for packages
leaf search editor

# Remove a package
leaf remove nvim

# Update package list
leaf update
```

## Commands

| Command | Description | Example |
|---------|-------------|---------|
| `leaf install <package>` | Install a package | `leaf install ripgrep` |
| `leaf remove <package>` | Remove an installed package | `leaf remove ripgrep` |
| `leaf list` | List all installed packages | `leaf list` |
| `leaf search <term>` | Search available packages | `leaf search rust` |
| `leaf update` | Update package definitions | `leaf update` |
| `leaf --help` | Show help information | `leaf --help` |
| `leaf --version` | Show version information | `leaf --version` |
| `leaf self-update` | Update the leaf package manger | `leaf self-update` |

## Available Packages

Leaf includes a curated selection of popular development tools and utilities:

### Editors & IDEs
- `nvim` - Neovim text editor
- `helix` - Modern modal text editor
- `code` - Visual Studio Code

*Use `leaf search .` to see all available packages.*

## How It Works

1. **User-space installation**: All packages are installed in `~/.local/leaf/packages/`
2. **Automatic PATH management**: Executables are symlinked to `~/.local/bin/`
3. **Smart extraction**: Supports `.tar.gz`, `.tar.xz`, and `.zip` archives
4. **Metadata tracking**: Each installation is tracked with version and file information
5. **Clean removal**: Removes all files and symlinks when uninstalling

## Directory Structure

```
~/.local/leaf/
├── packages/           # Installed packages
│   └── nvim/          # Package directory
├── cache/             # Downloaded archives
├── config.json        # Leaf configuration
└── packages.json      # Package definitions

~/.local/bin/          # Executable symlinks (added to PATH)
```

## Configuration

Leaf stores its configuration in `~/.local/leaf/config.json`:

```json
{
  "version": "1.0.0",
  "install_dir": "/home/user/.local/leaf",
  "bin_dir": "/home/user/.local/bin",
  "packages_dir": "/home/user/.local/leaf/packages",
  "cache_dir": "/home/user/.local/leaf/cache"
}
```

## Contributing

We welcome contributions! Here's how you can help:

### Adding New Packages

1. Fork this repository
2. Edit `packages.json` to add your package:
   ```json
   {
     "package-name": {
       "description": "Package description",
       "url": "https://github.com/user/repo/releases/download/v1.0.0/binary.tar.gz",
       "version": "1.0.0",
       "type": "archive",
       "executables": "bin/executable-name",
       "tags": ["category", "tool"]
     }
   }
   ```
3. Submit a pull request

### Development

1. Clone the repository:
   ```bash
   git clone https://github.com/ktauchathuranga/leaf.git
   cd leaf
   ```

2. Build from source:
   ```bash
   cargo build --release
   ```

3. Run tests:
    ```bash
    cargo test
    ```
    this will check each package in the `packages.json` file and validate them.

## Requirements

- **Linux**: glibc 2.17+ (most distributions from 2012+)
- **macOS**: macOS 10.12+ (Sierra)
- **Architecture**: x86_64

## Troubleshooting

### Command not found after installation

Make sure `~/.local/bin` is in your PATH:
```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Permission denied when running installed packages

Some packages may need executable permissions:
```bash
chmod +x ~/.local/bin/package-name
```

### Package not found

Update the package list:
```bash
leaf update
```

## Comparison

| Feature | Leaf | Homebrew | Snap | AppImage |
|---------|------|----------|------|----------|
| Sudo required | No | No | Yes | No |
| User-space install | Yes | Yes | No | Yes |
| Fast execution | Yes | Yes | No | Yes |
| Simple CLI | Yes | Yes | Yes | No |
| Cross-platform | Yes | Yes | No | Yes |

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/ktauchathuranga/leaf/issues)
- **Discussions**: [GitHub Discussions](https://github.com/ktauchathuranga/leaf/discussions)
---

<p align="center">
  <strong>Shhhh!</strong>
</p>
