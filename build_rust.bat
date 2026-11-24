@echo off
setlocal
cd /d "%~dp0"

echo Building RiftCore as a native Rust desktop application...
set "RUST_TOOLCHAIN="
set "RUST_OUTPUT=riftcore-native\target\release\riftcore-native.exe"

rem Wave 0+: Use D:\RiftCore-CargoTarget to avoid filling F: drive (only ~11 MB free).
rem Override by setting CARGO_TARGET_DIR in the environment before calling this script.
if not defined CARGO_TARGET_DIR set "CARGO_TARGET_DIR=D:\RiftCore-CargoTarget"

where cl >nul 2>nul
if errorlevel 1 (
    set "RUST_TOOLCHAIN=+stable-x86_64-pc-windows-gnu"
    if exist "%USERPROFILE%\scoop\apps\mingw\current\bin" set "PATH=%USERPROFILE%\scoop\apps\mingw\current\bin;%PATH%"
    where x86_64-w64-mingw32-gcc >nul 2>nul
    if errorlevel 1 (
        echo [ERROR] A Windows C linker is required. Install Visual Studio Build Tools or MinGW-w64.
        exit /b 1
    )
    cargo %RUST_TOOLCHAIN% --version >nul 2>nul
    if errorlevel 1 (
        echo [ERROR] The Rust GNU toolchain is missing. Install it with: rustup toolchain install stable-x86_64-pc-windows-gnu
        exit /b 1
    )
    echo Using the MinGW-w64 Rust target.
)

cargo %RUST_TOOLCHAIN% build --manifest-path "riftcore-native\Cargo.toml" --release --target-dir "%CARGO_TARGET_DIR%"
if errorlevel 1 (
    echo [ERROR] Rust build failed.
    exit /b 1
)

rem RUST_OUTPUT now points to the target dir; adjust path accordingly.
set "RUST_OUTPUT=%CARGO_TARGET_DIR%\release\riftcore-native.exe"

if not exist "RiftCore_Rust_Portable" mkdir "RiftCore_Rust_Portable"
copy /Y "%RUST_OUTPUT%" "RiftCore_Rust_Portable\RiftCore.exe" > nul
if errorlevel 1 exit /b 1
copy /Y "public\icon.ico" "RiftCore_Rust_Portable\icon.ico" > nul
copy /Y "LICENSE" "RiftCore_Rust_Portable\LICENSE" > nul
copy /Y "Launch_RiftCore.bat" "RiftCore_Rust_Portable\Launch_RiftCore.bat" > nul

rem Remove leftovers from previous WebView and Electron packages.
if exist "RiftCore_Rust_Portable\WebView2Loader.dll" del /Q "RiftCore_Rust_Portable\WebView2Loader.dll"
if exist "RiftCore_Rust_Portable\resources\app.asar" del /Q "RiftCore_Rust_Portable\resources\app.asar"
if exist "RiftCore_Rust_Portable\RiftCore_Overlay.exe" del /Q "RiftCore_Rust_Portable\RiftCore_Overlay.exe"
for %%F in (chrome_*.pak resources.pak snapshot_blob.bin v8_context_snapshot.bin icudtl.dat d3dcompiler_47.dll libEGL.dll libGLESv2.dll ffmpeg.dll vk_swiftshader.dll vulkan-1.dll) do if exist "RiftCore_Rust_Portable\%%F" del /Q "RiftCore_Rust_Portable\%%F"
if exist "RiftCore_Rust_Portable\resources" rmdir /S /Q "RiftCore_Rust_Portable\resources"

echo Native executable staged at RiftCore_Rust_Portable\RiftCore.exe
exit /b 0
