@echo off
set "RUSTUP_HOME=F:\.rustup"
set "CARGO_HOME=F:\.cargo"
set "PATH=F:\w64devkit\bin;F:\.cargo\bin;%PATH%"

cd /d "F:\league app\riftcore-native"
echo Compiling RiftCore Native with Rust toolchain on F: ...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo Cargo build failed!
    exit /b %ERRORLEVEL%
)

echo Build succeeded!
cd /d "F:\league app"
echo Copying updated dist and WebView2Loader.dll...
xcopy /E /I /Y "F:\league app\dist" "F:\league app\riftcore-native\target\release\dist"
copy /Y "F:\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\webview2-com-sys-0.33.0\x64\WebView2Loader.dll" "F:\league app\riftcore-native\target\release\"

echo Done!
