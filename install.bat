@echo off
REM Git-Nexus Installation Wrapper for Windows
REM This script runs the PowerShell installation script

echo Installing git-nexus...
echo.

powershell -ExecutionPolicy Bypass -File "%~dp0install.ps1"

if %ERRORLEVEL% EQU 0 (
    echo.
    echo Installation completed successfully!
    echo.
    pause
) else (
    echo.
    echo Installation failed!
    echo.
    pause
    exit /b 1
)
