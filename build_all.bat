@echo off
setlocal enabledelayedexpansion

echo ===================================================
echo     RiftCore Companion - 1-Click Master Builder
echo ===================================================
echo.

set "RUSTUP_HOME=F:\.rustup"
set "CARGO_HOME=F:\.cargo"
set "PATH=F:\w64devkit\bin;F:\.cargo\bin;%PATH%"
if not exist "F:\temp" mkdir "F:\temp"
set "TEMP=F:\temp"
set "TMP=F:\temp"

cd /d "F:\league app"

for /f "tokens=*" %%v in ('node -p "require('./package.json').version"') do set "APP_VERSION=%%v"
if "!APP_VERSION!"=="" set "APP_VERSION=1.1.2"

echo Target Version: v!APP_VERSION!
echo.

echo [1/5] Packaging Frontend and High-Performance Overlay Engine...
call node scripts/build_electron_exe.mjs
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Overlay packaging failed!
    exit /b %ERRORLEVEL%
)

echo.
echo [2/4] Generating Windows Installer (NSIS)...
if not exist "installer_output" mkdir "installer_output"
makensis /DPRODUCT_VERSION="!APP_VERSION!" "installer\installer.nsi"
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Installer generation failed!
    exit /b %ERRORLEVEL%
)

echo.
echo [3/4] Packaging Portable ZIP Archive...
powershell -Command "Compress-Archive -Path 'RiftCore_Rust_Portable\*' -DestinationPath 'installer_output\RiftCore-Portable-!APP_VERSION!.zip' -Force"

echo.
echo ===================================================
echo   BUILD COMPLETED SUCCESSFULLY!
echo ===================================================
echo   Installer: F:\league app\installer_output\RiftCore-Setup-!APP_VERSION!.exe
echo   Portable:  F:\league app\installer_output\RiftCore-Portable-!APP_VERSION!.zip
echo ===================================================
