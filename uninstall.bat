@echo off
REM Git-Nexus Uninstallation Wrapper for Windows

echo Uninstalling git-nexus...
echo.

powershell -ExecutionPolicy Bypass -File "%~dp0uninstall.ps1"

if %ERRORLEVEL% EQU 0 (
    echo.
    echo Uninstallation completed successfully!
    echo.
    pause
) else (
    echo.
    echo Uninstallation failed!
    echo.
    pause
    exit /b 1
)
