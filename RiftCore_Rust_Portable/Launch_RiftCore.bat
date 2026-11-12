@echo off
if exist "%~dp0RiftCore.exe" (
  start "" "%~dp0RiftCore.exe"
) else (
  start "" "%~dp0RiftCore_Rust_Portable\RiftCore.exe"
)
