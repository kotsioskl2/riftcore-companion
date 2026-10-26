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
echo [2/5] Compiling Native Rust Companion (Cargo Release)...
cd /d "F:\league app\riftcore-native"
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Cargo build failed!
    exit /b %ERRORLEVEL%
)

echo.
echo [3/5] Assembling Portable Release Package...
cd /d "F:\league app"
if not exist "RiftCore_Rust_Portable" mkdir "RiftCore_Rust_Portable"
if exist "RiftCore_Rust_Portable\dist" rd /s /q "RiftCore_Rust_Portable\dist"
copy /Y "riftcore-native\target\release\riftcore-native.exe" "RiftCore_Rust_Portable\RiftCore.exe" >nul
copy /Y "riftcore-native\target\release\WebView2Loader.dll" "RiftCore_Rust_Portable\WebView2Loader.dll" >nul
copy /Y "riftcore-native\icon.ico" "RiftCore_Rust_Portable\icon.ico" >nul

echo.
echo [4/5] Generating Windows Installer (NSIS)...
if not exist "installer_output" mkdir "installer_output"
makensis /DPRODUCT_VERSION="!APP_VERSION!" "installer\installer.nsi"
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Installer generation failed!
    exit /b %ERRORLEVEL%
)

echo.
echo [5/5] Packaging Portable ZIP Archive...
powershell -Command "Compress-Archive -Path 'RiftCore_Rust_Portable\*' -DestinationPath 'installer_output\RiftCore-Portable-!APP_VERSION!.zip' -Force"

echo.
echo ===================================================
echo   BUILD COMPLETED SUCCESSFULLY!
echo ===================================================
echo   Installer: F:\league app\installer_output\RiftCore-Setup-!APP_VERSION!.exe
echo   Portable:  F:\league app\installer_output\RiftCore-Portable-!APP_VERSION!.zip
echo ===================================================
