# Leaf Package Manager Installation Script for Windows
# Usage: irm https://raw.githubusercontent.com/ktauchathuranga/leaf/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$LeafDir = "$env:LOCALAPPDATA\leaf"
$BinDir = "$env:LOCALAPPDATA\Microsoft\WindowsApps" # Often in user's PATH by default
$Repo = "ktauchathuranga/leaf"
$LeafVersion = "latest"

Write-Host "🍃 Installing Leaf Package Manager for Windows..."

# Check if leaf is already installed
if (Get-Command leaf -ErrorAction SilentlyContinue) {
    Write-Host "[!] Leaf is already installed."
    Write-Host "If you want to completely remove it first, run: leaf nuke --confirmed"
    Write-Host "Continuing with installation/update..."
}

# Detect platform
$Arch = switch ($env:PROCESSOR_ARCHITECTURE) {
    "AMD64" { "x86_64" }
    "ARM64" { "aarch64" }
    default {
        Write-Error "❌ Unsupported architecture: $env:PROCESSOR_ARCHITECTURE"
        exit 1
    }
}
$Platform = "windows-$Arch"
Write-Host "[-] Detected platform: $Platform"

# Create directories
New-Item -Path $LeafDir, $BinDir -ItemType Directory -Force | Out-Null

# Get the latest release download URL
if ($LeafVersion -eq "latest") {
    Write-Host "[-] Finding latest release..."
    try {
        $ApiUrl = "https://api.github.com/repos/$Repo/releases/latest"
        $ReleaseInfo = Invoke-RestMethod -Uri $ApiUrl
        $DownloadUrl = $ReleaseInfo.assets | Where-Object { $_.name -like "leaf-$Platform.zip" } | Select-Object -ExpandProperty browser_download_url -First 1
        
        if (-not $DownloadUrl) {
            Write-Error "[!] Could not find release for platform $Platform"
            $AvailableAssets = $ReleaseInfo.assets | Where-Object { $_.name -like "*.zip" } | ForEach-Object { "  - " + ($_.name -replace 'leaf-(.*)\.zip', '$1') }
            Write-Host "Available releases:"
            Write-Host $AvailableAssets
            exit 1
        }
    } catch {
        Write-Error "❌ Failed to fetch release info from GitHub API. $_"
        exit 1
    }
} else {
    $DownloadUrl = "https://github.com/$Repo/releases/download/$LeafVersion/leaf-$Platform.zip"
}

Write-Host "[-] Downloading leaf binary..."
$TempDir = New-Item -ItemType Directory -Path (Join-Path $env:TEMP ([System.Guid]::NewGuid().ToString()))
$TempFile = Join-Path $TempDir "leaf-$Platform.zip"

try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempFile -UseBasicParsing
} catch {
    Write-Error "❌ Download failed. Please check the URL and your connection. $_"
    exit 1
}

# Extract and install
Write-Host "[-] Extracting binary..."
Expand-Archive -Path $TempFile -DestinationPath $TempDir -Force
Copy-Item -Path (Join-Path $TempDir "leaf.exe") -Destination (Join-Path $BinDir "leaf.exe") -Force

# Download package definitions
Write-Host "[-] Downloading package definitions..."
$PackagesUrl = "https://raw.githubusercontent.com/$Repo/main/packages.json"
Invoke-WebRequest -Uri $PackagesUrl -OutFile (Join-Path $LeafDir "packages.json") -UseBasicParsing

# Add BinDir to user's PATH if not present
$UserPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
if (-not ($UserPath -split ';' -contains $BinDir)) {
    $NewPath = ($UserPath, $BinDir) -join ';'
    [System.Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
    Write-Host "✅ Added $BinDir to your PATH. Please restart your terminal."
}

# Create leaf config
$Config = @{
    version = $LeafVersion
    install_dir = $LeafDir
    bin_dir = $BinDir
    packages_dir = "$LeafDir\packages"
    cache_dir = "$LeafDir\cache"
}
$Config | ConvertTo-Json | Set-Content -Path (Join-Path $LeafDir "config.json")

New-Item -Path "$LeafDir\packages", "$LeafDir\cache" -ItemType Directory -Force | Out-Null

# Cleanup
Remove-Item -Path $TempDir -Recurse -Force

# Test installation
try {
    $VersionInfo = (leaf --version | Out-String).Trim()
    Write-Host ""
    Write-Host "[-] Leaf Package Manager installed successfully!"
    Write-Host "[-] Version: $VersionInfo"
} catch {
    Write-Warning "[!] Installation completed but 'leaf' command test failed. Try restarting your terminal."
}

Write-Host ""
Write-Host "Usage:"
Write-Host "  leaf install <package>        # Install a package"
Write-Host "  leaf remove <package>         # Remove a package"
Write-Host "  leaf list                     # List installed packages"
Write-Host "  leaf search <term>            # Search available packages"
Write-Host "  leaf update                   # Update package list"
Write-Host "  leaf nuke --confirmed         # Remove everything (DESTRUCTIVE)"
Write-Host ""
Write-Host "To get started, please restart your terminal or run 'refreshenv'."
Write-Host "Then try: leaf install go"