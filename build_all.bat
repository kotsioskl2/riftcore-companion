@echo off
setlocal

echo ================================================
echo        RiftCore Companion - Native Build
echo ================================================
echo.

cd /d "%~dp0"
set "APP_VERSION=2.0.8"

echo [1/4] Building the native Rust application...
call build_rust.bat
if errorlevel 1 exit /b 1

echo.
echo [2/4] Building the Windows installer...
if not exist "installer_output" mkdir "installer_output"
makensis /DPRODUCT_VERSION="%APP_VERSION%" "installer\installer.nsi"
if errorlevel 1 (
    echo [ERROR] NSIS installer build failed.
    exit /b 1
)

echo.
echo [3/4] Creating the portable ZIP...
powershell -NoProfile -Command "Compress-Archive -Path '%~dp0RiftCore_Rust_Portable\*' -DestinationPath '%~dp0installer_output\RiftCore-Portable-%APP_VERSION%.zip' -Force"
if errorlevel 1 (
    echo [ERROR] Portable ZIP build failed.
    exit /b 1
)

echo.
echo [4/4] Build complete.
echo Installer: installer_output\RiftCore-Setup-%APP_VERSION%.exe
echo Portable:  installer_output\RiftCore-Portable-%APP_VERSION%.zip
exit /b 0
