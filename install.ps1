# Git-Nexus Installation Script for Windows
# Run with: powershell -ExecutionPolicy Bypass -File install.ps1

$ErrorActionPreference = "Stop"

$BINARY_NAME = "git-nexus.exe"
$INSTALL_DIR = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { "$env:USERPROFILE\.local\bin" }

Write-Host "🚀 Installing git-nexus for Windows..." -ForegroundColor Cyan
Write-Host ""

# Check if cargo is installed
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Error: Cargo is not installed." -ForegroundColor Red
    Write-Host "   Please install Rust from https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

# Build the project
Write-Host "📦 Building git-nexus in release mode..." -ForegroundColor Green
cargo build --release

if (-not (Test-Path "target\release\$BINARY_NAME")) {
    Write-Host "❌ Error: Build failed. Binary not found at target\release\$BINARY_NAME" -ForegroundColor Red
    exit 1
}

# Create install directory if it doesn't exist
if (-not (Test-Path $INSTALL_DIR)) {
    Write-Host "📁 Creating directory: $INSTALL_DIR" -ForegroundColor Yellow
    New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null
}

# Copy binary to install directory
Write-Host "📥 Installing $BINARY_NAME to $INSTALL_DIR..." -ForegroundColor Green
Copy-Item "target\release\$BINARY_NAME" -Destination $INSTALL_DIR -Force

Write-Host ""
Write-Host "✅ Installation complete!" -ForegroundColor Green
Write-Host ""
Write-Host "📍 Binary installed to: $INSTALL_DIR\$BINARY_NAME" -ForegroundColor Cyan
Write-Host ""

# Check if install directory is in PATH
$pathDirs = $env:PATH -split ';'
if ($pathDirs -notcontains $INSTALL_DIR) {
    Write-Host "⚠️  Warning: $INSTALL_DIR is not in your PATH" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "   Add it to your PATH by running (as Administrator):" -ForegroundColor Yellow
    Write-Host "   [Environment]::SetEnvironmentVariable('Path', `$env:Path + ';$INSTALL_DIR', 'User')" -ForegroundColor White
    Write-Host ""
    Write-Host "   Or manually add it via:" -ForegroundColor Yellow
    Write-Host "   1. Open 'Environment Variables' in System Properties" -ForegroundColor White
    Write-Host "   2. Edit 'Path' in User variables" -ForegroundColor White
    Write-Host "   3. Add: $INSTALL_DIR" -ForegroundColor White
    Write-Host ""
    Write-Host "   Then restart your terminal" -ForegroundColor Yellow
} else {
    Write-Host "🎉 You can now run: git-nexus" -ForegroundColor Green
    Write-Host ""
    Write-Host "   Try it out:" -ForegroundColor Cyan
    Write-Host "   git-nexus --help" -ForegroundColor White
    Write-Host "   git-nexus" -ForegroundColor White
}

Write-Host ""
