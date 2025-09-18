# 🍃 Leaf Package Manager

A fast, simple, and sudo-free package manager for Linux and Windows, written in Rust.

[![Release](https://img.shields.io/github/v/release/ktauchathuranga/leaf?sort=semver)](https://github.com/ktauchathuranga/leaf/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows-blue.svg)](https://github.com/ktauchathuranga/leaf)

## Features

- **Fast & Lightweight** - Written in Rust for optimal performance.
- **No Sudo Required** - Install packages in your user space, no admin privileges needed.
- **Simple Commands** - Easy-to-use and intuitive CLI.
- **Cross-Platform** - Natively supports Linux and Windows.
- **Efficient Caching** - Downloads are cached for faster reinstalls and offline use.
- **Package Search** - Easily find the tools you need.
- **Clean Uninstalls** - Completely removes packages without leaving behind junk.

## Quick Start

### Installation

#### Linux

Install Leaf with a single command in your terminal:

```bash
curl -sSL https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash
```

After installation, restart your terminal or run `source ~/.bashrc` (or `~/.zshrc`).

#### Windows

Install Leaf by running this command in a **PowerShell** terminal:

```powershell
irm https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.ps1 | iex
```

After installation, restart your PowerShell terminal for the `PATH` changes to take effect.

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
```

## Commands

| Command | Description | Example |
|---------|-------------|---------|
| `leaf install <package>` | Install a package | `leaf install nvim` |
| `leaf remove <package>` | Remove an installed package | `leaf remove nvim` |
| `leaf list` | List all installed packages | `leaf list` |
| `leaf search <term>` | Search for available packages | `leaf search rust` |
| `leaf update` | Update package definitions from the registry | `leaf update` |
| `leaf self-update` | Update Leaf to the latest version | `leaf self-update` |
| `leaf nuke --confirmed`| **DESTRUCTIVE**: Remove all packages and Leaf itself | `leaf nuke --confirmed` |
| `leaf --help` | Show help information | `leaf --help` |

## How It Works

1.  **User-Space Installation**: Packages are installed into your user directory, not system-wide.
    -   **Linux**: `~/.local/share/leaf/packages/`
    -   **Windows**: `%LOCALAPPDATA%\leaf\packages\`
2.  **Automatic PATH Management**: Executables are linked into a common `bin` directory that you add to your PATH once.
3.  **Cross-Platform Awareness**: The `packages.json` registry contains different URLs and instructions for each operating system.
4.  **Clean Removal**: `leaf remove` deletes the package directory and its executable link, keeping your system clean.

## Directory Structure

**On Linux:**
```
~/.local/
├── share/leaf/
│   ├── packages/         # Installed packages
│   ├── cache/            # Downloaded archives
│   ├── config.json       # Leaf configuration
│   └── packages.json     # Package definitions
└── bin/                  # Executable symlinks (in your PATH)
```

**On Windows:**
```
%LOCALAPPDATA%\
└── leaf\
    ├── packages/         # Installed packages
    ├── cache/            # Downloaded archives
    ├── config.json       # Leaf configuration
    └── packages.json     # Package definitions

# Note: Binaries are typically placed in a directory already in the user's PATH,
# or you will be instructed to add it.
```

## Contributing

We welcome contributions! The easiest way to contribute is by adding new packages.

### Adding New Packages

1.  Fork this repository.
2.  Edit `packages.json` to add or update a package. The new format requires a `platforms` object:
    ```json
    "package-name": {
      "description": "A cool tool.",
      "version": "1.2.3",
      "tags": ["cli", "tool"],
      "platforms": {
        "linux-x86_64": {
          "url": "https://.../download/v1.2.3/linux-amd64.tar.gz",
          "type": "archive",
          "executables": ["bin/executable"]
        },
        "windows-x86_64": {
          "url": "https://.../download/v1.2.3/windows-amd64.zip",
          "type": "archive",
          "executables": ["bin/executable.exe"]
        }
      }
    }
    ```
3.  Run `cargo test` to validate the URLs in your new entry.
4.  Submit a pull request!

### Development

1.  Clone the repository: `git clone https://github.com/ktauchathuranga/leaf.git`
2.  Build from source: `cargo build --release`
3.  Run tests: `cargo test`

## Requirements

- **Linux**: glibc 2.18+
- **Windows**: Windows 10+
- **Architectures**: x86_64

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
