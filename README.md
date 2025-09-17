# 🍃 Leaf Package Manager

A simple, sudo-free package manager for Ubuntu/Linux systems. Install packages to your home directory without root access.

## 🚀 Installation

Install Leaf with a single command:

```bash
curl -sSL https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.sh | bash
```

Then restart your terminal or run:
```bash
source ~/.bashrc
```

## 📖 Usage

```bash
# Install packages
leaf install nvim
leaf install nodejs
leaf install go

# Search for packages
leaf search editor
leaf search javascript

# List installed packages
leaf list

# Remove packages
leaf remove nvim

# Update package list
leaf update
```

## ✨ Features

- **No sudo required** - All packages install to `~/.local/`
- **Simple installation** - One-line curl command
- **Fast and lightweight** - Written in Python, minimal dependencies
- **User-friendly** - Colored output and progress bars
- **Package caching** - Downloads are cached for reinstallation

## 📦 Available Packages

- **nvim** - Modern terminal-based text editor
- **nodejs** - JavaScript runtime (includes npm)
- **go** - The Go programming language
- **fd** - Fast alternative to find
- **bat** - Cat clone with syntax highlighting
- And more...

## 🔧 How it Works

Leaf downloads pre-compiled binaries and installs them to:
- **Packages**: `~/.local/leaf/packages/`
- **Binaries**: `~/.local/bin/` (added to PATH)
- **Cache**: `~/.local/leaf/cache/`

## 🤝 Contributing

Add new packages by editing `packages.json`:

```json
{
  "package-name": {
    "description": "Package description",
    "url": "https://example.com/package.tar.gz",
    "version": "1.0.0",
    "type": "archive",
    "executables": ["path/to/binary"],
    "tags": ["tag1", "tag2"]
  }
}
```

## 📄 License

MIT License
