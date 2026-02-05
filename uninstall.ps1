# Git-Nexus Uninstallation Script for Windows
# Run with: powershell -ExecutionPolicy Bypass -File uninstall.ps1

$ErrorActionPreference = "Stop"

$BINARY_NAME = "git-nexus.exe"
$INSTALL_DIR = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { "$env:USERPROFILE\.local\bin" }
$BINARY_PATH = Join-Path $INSTALL_DIR $BINARY_NAME

Write-Host "Uninstalling git-nexus..." -ForegroundColor Cyan
Write-Host ""

if (Test-Path $BINARY_PATH) {
    Write-Host "Found: $BINARY_PATH" -ForegroundColor Green
    Remove-Item $BINARY_PATH -Force
    Write-Host "Successfully removed $BINARY_NAME" -ForegroundColor Green
} else {
    Write-Host "WARNING: Binary not found at: $BINARY_PATH" -ForegroundColor Yellow
    
    # Check other common locations
    $otherLocations = @(
        "C:\Program Files\git-nexus\$BINARY_NAME",
        "$env:LOCALAPPDATA\Programs\git-nexus\$BINARY_NAME",
        "$env:ProgramFiles\git-nexus\$BINARY_NAME"
    )
    
    $found = $false
    foreach ($location in $otherLocations) {
        if (Test-Path $location) {
            Write-Host "Found at: $location" -ForegroundColor Green
            
            try {
                Remove-Item $location -Force
                Write-Host "Successfully removed $BINARY_NAME" -ForegroundColor Green
                $found = $true
                break
            } catch {
                Write-Host "ERROR: No write permission. Try running as Administrator:" -ForegroundColor Red
                Write-Host "  Remove-Item '$location' -Force" -ForegroundColor White
                $found = $true
                break
            }
        }
    }
    
    if (-not $found) {
        Write-Host "ERROR: Could not find $BINARY_NAME in common locations" -ForegroundColor Red
        exit 1
    }
}

Write-Host ""
Write-Host "git-nexus has been uninstalled" -ForegroundColor Green
Write-Host ""
