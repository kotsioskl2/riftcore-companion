@echo off
setlocal enabledelayedexpansion

echo ===================================================
echo     RiftCore Companion - 1-Click Master Builder
echo ===================================================
echo.

set "RUSTUP_HOME=F:\.rustup"
set "CARGO_HOME=F:\.cargo"
set "PATH=F:\w64devkit\bin;F:\.cargo\bin;%PATH%"

cd /d "F:\league app"

echo [1/4] Building Frontend (Vite + React 19)...
call npm run build:renderer
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Frontend build failed!
    exit /b %ERRORLEVEL%
)

echo.
echo [2/4] Compiling Native Rust Backend (Cargo Release)...
cd /d "F:\league app\riftcore-native"
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Cargo build failed!
    exit /b %ERRORLEVEL%
)

echo.
echo [3/4] Assembling Portable Release Package...
cd /d "F:\league app"
if not exist "RiftCore_Rust_Portable" mkdir "RiftCore_Rust_Portable"
if exist "RiftCore_Rust_Portable\dist" rd /s /q "RiftCore_Rust_Portable\dist"
copy /Y "riftcore-native\target\release\riftcore-native.exe" "RiftCore_Rust_Portable\RiftCore.exe" >nul
copy /Y "riftcore-native\target\release\WebView2Loader.dll" "RiftCore_Rust_Portable\WebView2Loader.dll" >nul
copy /Y "riftcore-native\icon.ico" "RiftCore_Rust_Portable\icon.ico" >nul

echo.
echo [4/5] Generating Windows Installer (NSIS)...
if not exist "installer_output" mkdir "installer_output"
makensis "installer\installer.nsi"
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Installer generation failed!
    exit /b %ERRORLEVEL%
)

echo.
echo [5/5] Packaging Portable ZIP Archive...
powershell -Command "Compress-Archive -Path 'RiftCore_Rust_Portable\*' -DestinationPath 'installer_output\RiftCore-Portable-1.1.0.zip' -Force"

echo.
echo ===================================================
echo   BUILD COMPLETED SUCCESSFULLY!
echo ===================================================
echo   Installer: F:\league app\installer_output\RiftCore-Setup-1.1.0.exe
echo   Portable:  F:\league app\RiftCore_Rust_Portable\RiftCore.exe
echo ===================================================
