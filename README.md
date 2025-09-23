# 🍃 Leaf Package Manager

A fast, simple, and sudo-free package manager for Linux, written in Rust.

[![Release](https://img.shields.io/github/v/release/ktauchathuranga/leaf?sort=semver)](https://github.com/ktauchathuranga/leaf/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Linux-blue.svg)](https://github.com/ktauchathuranga/leaf)

## Features

- **Fast & Lightweight** - Written in Rust for optimal performance.
- **No Sudo Required** - Install packages in your user space, no admin privileges needed.
- **Multi-Architecture** - Supports both `x86_64` and `aarch64` (ARM64) Linux systems.
- **Simple Commands** - Easy-to-use and intuitive CLI.
- **Efficient Caching** - Downloads are cached for faster reinstalls and offline use.
- **Clean Uninstalls** - Completely removes packages without leaving behind junk.
- **Version Control** - Install or update to specific versions or prereleases.

## Quick Start

### Installation

Install Leaf with a single command in your terminal. By default, the latest stable release is installed:

Using `curl`:
```bash
curl -sSL https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash
```

Using `wget`:
```bash
wget -qO- https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash
```

To install a specific version or prerelease:
```bash
# Install a specific stable version
curl -sSL https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash -s -- --version v1.2.3

# Install a specific prerelease version
curl -sSL https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash -s -- --version v1.2.3-beta

# Install the latest prerelease (alpha, beta, or rc)
curl -sSL https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash -s -- --prerelease
```

After installation, restart your terminal or run `source ~/.bashrc` (or `~/.zshrc`).

### Basic Usage

```bash
# Install a package
leaf install go

# List installed packages
leaf list

# Search for packages
leaf search editor

# Remove a package
leaf remove go

# Update the list of available packages
leaf update

# Update Leaf to the latest stable version
leaf self-update

# Update Leaf to a specific version
leaf self-update --version v1.2.3

# Update Leaf to the latest prerelease
leaf self-update --prerelease
```

## Commands

| Command | Description | Example |
|---------|-------------|---------|
| `leaf install <package>` | Install a package | `leaf install nvim` |
| `leaf remove <package>` | Remove an installed package | `leaf remove nvim` |
| `leaf list` | List all installed packages | `leaf list` |
| `leaf search <term>` | Search for available packages | `leaf search rust` |
| `leaf update` | Update package definitions from the registry | `leaf update` |
| `leaf self-update [--version <version>] [--prerelease]` | Update Leaf to the latest stable version, a specific version, or the latest prerelease | `leaf self-update`<br>`leaf self-update --version v1.2.3`<br>`leaf self-update --prerelease` |
| `leaf nuke --confirmed`| **DESTRUCTIVE**: Remove all packages and Leaf itself | `leaf nuke --confirmed` |
| `leaf --help` | Show help information | `leaf --help` |

## How It Works

1. **User-Space Installation**: Packages are installed into your user directory (`~/.local/leaf/packages/`), not system-wide.
2. **Automatic PATH Management**: Executables are linked into a common `bin` directory that you add to your PATH once.
3. **Clean Removal**: `leaf remove` deletes the package directory and its executable link, keeping your system clean.
4. **Version Control**: Use `--version` or `--prerelease` with `self-update` or the install script to control which version of Leaf is installed.

## Directory Structure

```
~/.local/
├── bin/                  # Executable symlinks (in your PATH)
└── leaf/
    ├── packages/         # Installed packages
    ├── cache/            # Downloaded archives
    ├── config.json       # Leaf configuration
    └── packages.json     # Package definitions
```

## Contributing

We welcome contributions! The easiest way to contribute is by adding new packages.

### Adding New Packages

1. Fork this repository.
2. Edit `packages.json` to add or update a package. The format requires a `platforms` object with entries for each supported architecture.
    ```json
    "package-name": {
      "description": "A cool tool.",
      "version": "1.2.3",
      "tags": ["cli", "tool"],
      "platforms": {
        "linux-x86_64": {
          "url": "https://.../download/v1.2.3/linux-x86_64.tar.gz",
          "type": "archive",
          "executables": [
            {
              "path": "path/inside/archive/to/executable",
              "name": "desired-command-name"
            }
          ]
        },
        "linux-aarch64": {
          "url": "https://.../download/v1.2.3/linux-arm64.tar.gz",
          "type": "archive",
          "executables": [
            {
              "path": "path/inside/archive/to/executable",
              "name": "desired-command-name"
            }
          ]
        }
      }
    }
    ```
3. Run `cargo test` to validate the URLs in your new entry.
4. Submit a pull request!

### Development

1. Clone the repository: `git clone https://github.com/ktauchathuranga/leaf.git`
2. Build from source: `cargo build --release`
3. Run tests: `cargo test`

## Requirements

- **Linux**: glibc 2.18+
- **Architectures**: `x86_64` (Intel/AMD 64-bit), `aarch64` (ARM 64-bit)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
