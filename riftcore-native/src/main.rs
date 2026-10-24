#![windows_subsystem = "windows"]

use std::sync::{Arc, Mutex};
use std::process::Command;
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::{
    http::{header::CONTENT_TYPE, Request, Response},
    WebViewBuilder, WebViewBuilderExtWindows,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use base64::prelude::*;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../dist/"]
struct EmbeddedAssets;

#[cfg(target_os = "windows")]
fn ensure_webview2_loader() {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    let target_dll = exe_dir.join("WebView2Loader.dll");
    if !target_dll.exists() {
        const LOADER_BYTES: &[u8] = include_bytes!("../WebView2Loader.dll");
        let _ = std::fs::write(&target_dll, LOADER_BYTES);
    }
}

#[cfg(target_os = "windows")]
use tao::platform::windows::WindowExtWindows;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LcuCredentials {
    port: String,
    token: String,
    install_dir: Option<PathBuf>,
}

#[derive(Default)]
struct AppState {
    credentials: Option<LcuCredentials>,
    gameflow_phase: String,
    is_compact: bool,
    is_pinned: bool,
}

enum AppUserEvent {
    Exit,
    EvalScript(String),
    ToggleOverlay,
    ApplyTopmost(bool),
    ShowAndFocus,
}

fn detect_league_client() -> Option<LcuCredentials> {
    // 1. Direct lockfile check in standard and installed directories (Fastest, 0ms, zero CPU)
    let candidates = [
        "E:\\Riot Games\\League of Legends\\lockfile",
        "F:\\Riot Games\\League of Legends (PBE)\\lockfile",
        "C:\\Riot Games\\League of Legends\\lockfile",
        "D:\\Riot Games\\League of Legends\\lockfile",
    ];

    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            let parts: Vec<&str> = content.trim().split(':').collect();
            if parts.len() >= 5 {
                let port = parts[2].to_string();
                let token = parts[3].to_string();
                if !port.is_empty() && !token.is_empty() {
                    let parent_dir = Path::new(path).parent().map(|p| p.to_path_buf());
                    return Some(LcuCredentials { port, token, install_dir: parent_dir });
                }
            }
        }
    }

    // 2. Query process path of LeagueClientUx if installed elsewhere
    if let Ok(output) = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-Process LeagueClientUx -ErrorAction SilentlyContinue).Path",
        ])
        .output()
    {
        let exe_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !exe_path.is_empty() {
            if let Some(parent) = Path::new(&exe_path).parent() {
                let lockfile = parent.join("lockfile");
                if let Ok(content) = std::fs::read_to_string(&lockfile) {
                    let parts: Vec<&str> = content.trim().split(':').collect();
                    if parts.len() >= 5 {
                        let port = parts[2].to_string();
                        let token = parts[3].to_string();
                        if !port.is_empty() && !token.is_empty() {
                            return Some(LcuCredentials { port, token, install_dir: Some(parent.to_path_buf()) });
                        }
                    }
                }
            }
        }
    }

    None
}

fn is_riot_client_open() -> bool {
    let local_appdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let lockfile = format!("{}\\Riot Games\\Riot Client\\Config\\lockfile", local_appdata);
    Path::new(&lockfile).exists()
}

#[cfg(target_os = "windows")]
#[link(name = "shell32")]
#[link(name = "user32")]
extern "system" {
    fn CreateMutexW(lpMutexAttributes: *mut std::ffi::c_void, bInitialOwner: i32, lpName: *const u16) -> *mut std::ffi::c_void;
    fn GetLastError() -> u32;
    fn GetModuleFileNameW(hModule: *mut std::ffi::c_void, lpFilename: *mut u16, nSize: u32) -> u32;
    fn ExtractIconExW(
        lpszFile: *const u16,
        nIconIndex: i32,
        phiconLarge: *mut *mut std::ffi::c_void,
        phiconSmall: *mut *mut std::ffi::c_void,
        nIcons: u32,
    ) -> u32;
    fn LoadIconW(hInstance: *mut std::ffi::c_void, lpIconName: *const u16) -> *mut std::ffi::c_void;
    fn SendMessageW(hWnd: *mut std::ffi::c_void, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn SetClassLongPtrW(hWnd: *mut std::ffi::c_void, nIndex: i32, dwNewLong: isize) -> isize;
    fn SetCurrentProcessExplicitAppUserModelID(AppID: *const u16) -> i32;
    fn GetWindowLongW(hWnd: *mut std::ffi::c_void, nIndex: i32) -> i32;
    fn SetWindowLongW(hWnd: *mut std::ffi::c_void, nIndex: i32, dwNewLong: i32) -> i32;
    fn SetWindowPos(
        hWnd: *mut std::ffi::c_void,
        hWndInsertAfter: *mut std::ffi::c_void,
        X: i32,
        Y: i32,
        cx: i32,
        cy: i32,
        uFlags: u32,
    ) -> i32;
    #[allow(dead_code)]
    fn RegisterHotKey(hWnd: *mut std::ffi::c_void, id: i32, fsModifiers: u32, vk: u32) -> i32;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: *mut std::ffi::c_void, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    fn PostQuitMessage(nExitCode: i32);
    fn Shell_NotifyIconW(dwMessage: u32, lpData: *mut NOTIFYICONDATAW) -> i32;
    fn RegisterClassExW(lpWndClass: *const WNDCLASSEXW) -> u16;
    fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        X: i32,
        Y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: *mut std::ffi::c_void,
        hMenu: *mut std::ffi::c_void,
        hInstance: *mut std::ffi::c_void,
        lpParam: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void;
    fn DefWindowProcW(hWnd: *mut std::ffi::c_void, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn CreatePopupMenu() -> *mut std::ffi::c_void;
    fn AppendMenuW(hMenu: *mut std::ffi::c_void, uFlags: u32, uIDNewItem: usize, lpNewItem: *const u16) -> i32;
    fn TrackPopupMenu(hMenu: *mut std::ffi::c_void, uFlags: u32, x: i32, y: i32, nReserved: i32, hWnd: *mut std::ffi::c_void, prcRect: *const std::ffi::c_void) -> i32;
    fn DestroyMenu(hMenu: *mut std::ffi::c_void) -> i32;
    fn GetCursorPos(lpPoint: *mut POINT) -> i32;
    fn SetForegroundWindow(hWnd: *mut std::ffi::c_void) -> i32;
    fn GetForegroundWindow() -> *mut std::ffi::c_void;
    fn BringWindowToTop(hWnd: *mut std::ffi::c_void) -> i32;
    fn SwitchToThisWindow(hWnd: *mut std::ffi::c_void, fUnknown: i32);
    fn GetAsyncKeyState(vKey: i32) -> i16;
    fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> *mut std::ffi::c_void;
    fn GetSystemMetrics(nIndex: i32) -> i32;
    fn GetWindow(hWnd: *mut std::ffi::c_void, uCmd: u32) -> *mut std::ffi::c_void;
    fn DestroyWindow(hWnd: *mut std::ffi::c_void) -> i32;
    fn ShowWindow(hWnd: *mut std::ffi::c_void, nCmdShow: i32) -> i32;
}

#[cfg(target_os = "windows")]
#[link(name = "dwmapi")]
extern "system" {
    fn DwmSetWindowAttribute(
        hwnd: *mut std::ffi::c_void,
        dwAttribute: u32,
        pvAttribute: *const std::ffi::c_void,
        cbAttribute: u32,
    ) -> i32;
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct MSG {
    hwnd: *mut std::ffi::c_void,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    pt: POINT,
}

#[allow(non_snake_case)]
#[cfg(target_os = "windows")]
#[repr(C)]
struct WNDCLASSEXW {
    cbSize: u32,
    style: u32,
    lpfnWndProc: Option<unsafe extern "system" fn(*mut std::ffi::c_void, u32, usize, isize) -> isize>,
    cbClsExtra: i32,
    cbWndExtra: i32,
    hInstance: *mut std::ffi::c_void,
    hIcon: *mut std::ffi::c_void,
    hCursor: *mut std::ffi::c_void,
    hbrBackground: *mut std::ffi::c_void,
    lpszMenuName: *const u16,
    lpszClassName: *const u16,
    hIconSm: *mut std::ffi::c_void,
}

#[allow(non_snake_case)]
#[cfg(target_os = "windows")]
#[repr(C)]
struct NOTIFYICONDATAW {
    cbSize: u32,
    hWnd: *mut std::ffi::c_void,
    uID: u32,
    uFlags: u32,
    uCallbackMessage: u32,
    hIcon: *mut std::ffi::c_void,
    szTip: [u16; 128],
    dwState: u32,
    dwStateMask: u32,
    szInfo: [u16; 256],
    uTimeoutOrVersion: u32,
    szInfoTitle: [u16; 64],
    dwInfoFlags: u32,
    guidItem: [u8; 16],
    hBalloonIcon: *mut std::ffi::c_void,
}

const WM_SETICON: u32 = 0x0080;
const ICON_SMALL: usize = 0;
const ICON_BIG: usize = 1;
const GCLP_HICON: i32 = -14;
const GCLP_HICONSM: i32 = -34;

const GWL_EXSTYLE: i32 = -20;
const WS_EX_TOPMOST: i32 = 0x00000008;
const WS_EX_TOOLWINDOW: i32 = 0x00000080;
const WS_EX_APPWINDOW: i32 = 0x00040000;
const WS_EX_NOACTIVATE: i32 = 0x08000000;
const WS_EX_TRANSPARENT: i32 = 0x00000020;

const HWND_TOPMOST: isize = -1;
const HWND_NOTOPMOST: isize = -2;

const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOZORDER: u32 = 0x0004;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_FRAMECHANGED: u32 = 0x0020;
const SWP_SHOWWINDOW: u32 = 0x0040;
const SW_HIDE: i32 = 0;
const SW_SHOWNA: i32 = 8;

#[allow(dead_code)]
const WM_HOTKEY: u32 = 0x0312;
#[allow(dead_code)]
const MOD_CONTROL: u32 = 0x0002;
#[allow(dead_code)]
const MOD_ALT: u32 = 0x0001;
#[allow(dead_code)]
const MOD_NOREPEAT: u32 = 0x4000;
#[allow(dead_code)]
const VK_SPACE: u32 = 0x20;
#[allow(dead_code)]
const VK_OEM_3: u32 = 0xC0;

const ERROR_ALREADY_EXISTS: u32 = 183;

const WM_TRAYICON: u32 = 0x8000 + 101;
const TRAY_ID: u32 = 1001;
const IDM_OPEN: usize = 2001;
const IDM_TOGGLE_HUD: usize = 2002;
const IDM_EXIT: usize = 2003;

static TRAY_PROXY: std::sync::OnceLock<tao::event_loop::EventLoopProxy<AppUserEvent>> = std::sync::OnceLock::new();
static TRAY_HWND: std::sync::atomic::AtomicPtr<std::ffi::c_void> = std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

#[cfg(target_os = "windows")]
unsafe extern "system" fn tray_wnd_proc(
    hwnd: *mut std::ffi::c_void,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    match msg {
        WM_TRAYICON => {
            let event = lparam as u32;
            const WM_LBUTTONUP: u32 = 0x0202;
            const WM_LBUTTONDBLCLK: u32 = 0x0203;
            const WM_RBUTTONUP: u32 = 0x0205;

            if event == WM_LBUTTONUP || event == WM_LBUTTONDBLCLK {
                if let Some(proxy) = TRAY_PROXY.get() {
                    let _ = proxy.send_event(AppUserEvent::ShowAndFocus);
                }
            } else if event == WM_RBUTTONUP {
                let hmenu = CreatePopupMenu();
                if !hmenu.is_null() {
                    let open_str: Vec<u16> = "Open RiftCore\0".encode_utf16().collect();
                    let toggle_str: Vec<u16> = "Toggle Compact HUD\0".encode_utf16().collect();
                    let exit_str: Vec<u16> = "Exit Completely\0".encode_utf16().collect();

                    AppendMenuW(hmenu, 0, IDM_OPEN, open_str.as_ptr());
                    AppendMenuW(hmenu, 0, IDM_TOGGLE_HUD, toggle_str.as_ptr());
                    AppendMenuW(hmenu, 0x00000800 /* MF_SEPARATOR */, 0, std::ptr::null());
                    AppendMenuW(hmenu, 0, IDM_EXIT, exit_str.as_ptr());

                    let mut pt: POINT = std::mem::zeroed();
                    GetCursorPos(&mut pt);
                    SetForegroundWindow(hwnd);

                    const TPM_RIGHTBUTTON: u32 = 0x0002;
                    const TPM_BOTTOMALIGN: u32 = 0x0020;
                    TrackPopupMenu(
                        hmenu,
                        TPM_RIGHTBUTTON | TPM_BOTTOMALIGN,
                        pt.x,
                        pt.y,
                        0,
                        hwnd,
                        std::ptr::null(),
                    );
                    DestroyMenu(hmenu);
                }
            }
            0
        }
        0x0111 /* WM_COMMAND */ => {
            let id = wparam & 0xFFFF;
            if let Some(proxy) = TRAY_PROXY.get() {
                match id {
                    IDM_OPEN => {
                        let _ = proxy.send_event(AppUserEvent::ShowAndFocus);
                    }
                    IDM_TOGGLE_HUD => {
                        let _ = proxy.send_event(AppUserEvent::ToggleOverlay);
                    }
                    IDM_EXIT => {
                        let _ = proxy.send_event(AppUserEvent::Exit);
                    }
                    _ => {}
                }
            }
            0
        }
        0x0002 /* WM_DESTROY */ => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[cfg(target_os = "windows")]
fn start_system_tray(
    proxy: tao::event_loop::EventLoopProxy<AppUserEvent>,
    h_icon: *mut std::ffi::c_void,
) {
    let _ = TRAY_PROXY.set(proxy);
    let icon_usize = h_icon as usize;
    std::thread::spawn(move || unsafe {
        let mut h_icon = icon_usize as *mut std::ffi::c_void;
        if h_icon.is_null() {
            h_icon = LoadIconW(std::ptr::null_mut(), 32512 as *const u16);
        }

        let class_name: Vec<u16> = "RiftCoreTrayClass\0".encode_utf16().collect();
        let mut wc: WNDCLASSEXW = std::mem::zeroed();
        wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
        wc.lpfnWndProc = Some(tray_wnd_proc);
        wc.lpszClassName = class_name.as_ptr();
        wc.hIcon = h_icon;
        wc.hIconSm = h_icon;

        RegisterClassExW(&wc);

        let window_title: Vec<u16> = "RiftCoreTrayWindow\0".encode_utf16().collect();
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            window_title.as_ptr(),
            0,
            0, 0, 0, 0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        if hwnd.is_null() {
            return;
        }

        TRAY_HWND.store(hwnd, std::sync::atomic::Ordering::SeqCst);

        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = TRAY_ID;
        nid.uFlags = 0x00000001 /* NIF_MESSAGE */ | 0x00000002 /* NIF_ICON */ | 0x00000004 /* NIF_TIP */;
        nid.uCallbackMessage = WM_TRAYICON;
        nid.hIcon = h_icon;

        let tip = "RiftCore - Minimalist League Companion\0";
        for (i, c) in tip.encode_utf16().enumerate() {
            if i < nid.szTip.len() {
                nid.szTip[i] = c;
            }
        }

        Shell_NotifyIconW(0 /* NIM_ADD */, &mut nid);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Shell_NotifyIconW(2 /* NIM_DELETE */, &mut nid);
    });
}

fn find_league_game_cfg() -> Option<PathBuf> {
    let candidate_paths = [
        "E:\\Riot Games\\League of Legends\\Config\\game.cfg",
        "C:\\Riot Games\\League of Legends\\Config\\game.cfg",
        "D:\\Riot Games\\League of Legends\\Config\\game.cfg",
        "F:\\Riot Games\\League of Legends\\Config\\game.cfg",
        "E:\\Riot Games\\League of Legends (PBE)\\Config\\game.cfg",
        "C:\\Riot Games\\League of Legends (PBE)\\Config\\game.cfg",
    ];
    for p in &candidate_paths {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

fn ensure_league_fullscreen_cfg() {
    if let Some(cfg_path) = find_league_game_cfg() {
        if let Ok(content) = std::fs::read_to_string(&cfg_path) {
            if content.contains("WindowMode=2") || content.contains("WindowMode=1") {
                let mut new_lines = Vec::new();
                for line in content.lines() {
                    if line.trim().starts_with("WindowMode=") {
                        new_lines.push("WindowMode=0".to_string());
                    } else {
                        new_lines.push(line.to_string());
                    }
                }
                let _ = std::fs::write(&cfg_path, new_lines.join("\r\n"));
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn maintain_overlay_topmost(overlay_hwnd: *mut std::ffi::c_void) {
    unsafe {
        let class_name: Vec<u16> = "RiotWindowClass\0".encode_utf16().collect();
        let league_hwnd = FindWindowW(class_name.as_ptr(), std::ptr::null());
        if !league_hwnd.is_null() {
            let fg = GetForegroundWindow();
            let is_league_active = (fg == league_hwnd) || (fg == overlay_hwnd);

            if is_league_active {
                // League of Legends is the active foreground window:
                // Ensure overlay is shown and stays layered directly over League
                ShowWindow(overlay_hwnd, SW_SHOWNA);
                let ex = GetWindowLongW(overlay_hwnd, GWL_EXSTYLE);
                if (ex & WS_EX_TOPMOST) == 0 {
                    let new_ex = (ex | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW) & !WS_EX_APPWINDOW;
                    SetWindowLongW(overlay_hwnd, GWL_EXSTYLE, new_ex);
                }
                // Only update Z-order if the overlay has actually been covered by another window
                let prev_wnd = GetWindow(overlay_hwnd, 3 /* GW_HWNDPREV */);
                if !prev_wnd.is_null() {
                    SetWindowPos(
                        overlay_hwnd,
                        HWND_TOPMOST as *mut std::ffi::c_void,
                        0, 0, 0, 0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                    );
                }
            } else {
                // User has Alt-Tabbed out of League (Desktop, File Explorer, Chrome, Discord):
                // Hide overlay immediately so it never traps or covers Windows desktop!
                ShowWindow(overlay_hwnd, SW_HIDE);
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn apply_fullscreen_topmost(hwnd: *mut std::ffi::c_void, enable: bool) {
    unsafe {
        let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
        if enable {
            // In-Game Overlay Mode:
            // 1. WS_EX_TOPMOST: Stays layered on top of Fullscreen League
            // 2. WS_EX_NOACTIVATE: Never steals mouse/keyboard focus from League
            // 3. WS_EX_TOOLWINDOW: Hides from Windows Taskbar and Alt-Tab
            let new_ex = (ex | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW) & !WS_EX_APPWINDOW;
            SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex);
            SetWindowPos(
                hwnd,
                HWND_TOPMOST as *mut std::ffi::c_void,
                0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        } else {
            // Full Desktop Client Mode: Standard window with taskbar presence
            let new_ex = (ex | WS_EX_APPWINDOW) & !(WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW);
            SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex);
            SetWindowPos(
                hwnd,
                HWND_NOTOPMOST as *mut std::ffi::c_void,
                0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
            );
        }
    }
}

fn is_already_running() -> bool {
    #[cfg(target_os = "windows")]
    unsafe {
        let name: Vec<u16> = "Local\\RiftCoreCompanion_SingleInstance\0".encode_utf16().collect();
        let handle = CreateMutexW(std::ptr::null_mut(), 1, name.as_ptr());
        if !handle.is_null() && GetLastError() == ERROR_ALREADY_EXISTS {
            return true;
        }
    }
    false
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Register explicit AppUserModelID so Windows Taskbar binds to RiftCore branding rather than generic host
    #[cfg(target_os = "windows")]
    unsafe {
        let app_id: Vec<u16> = "RiftCore.Companion.App.1.0\0".encode_utf16().collect();
        let _ = SetCurrentProcessExplicitAppUserModelID(app_id.as_ptr());
    }

    #[cfg(target_os = "windows")]
    ensure_webview2_loader();

    // Ensure League of Legends remains in true Full Screen mode (never forced to borderless)
    ensure_league_fullscreen_cfg();

    // Optimize WebView2 memory: limit to 1 renderer process, disable unused Edge telemetry/features, enable low-memory mode
    std::env::set_var(
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
        "--renderer-process-limit=1 --enable-low-end-device-mode --disable-features=msWebOOUI,msPdfOOUI,Translate,msEdgeSidebarV2,msHub,msEdgeRewards --disable-gpu-shader-disk-cache --disable-background-networking --disable-component-update --disk-cache-size=10485760 --media-cache-size=10485760"
    );

    if is_already_running() {
        // Enforce single window instance
        return Ok(());
    }

    let state = Arc::new(Mutex::new(AppState {
        credentials: detect_league_client(),
        gameflow_phase: "None".to_string(),
        is_compact: false,
        is_pinned: false,
    }));

    let event_loop = EventLoopBuilder::<AppUserEvent>::with_user_event().build();

    let window = WindowBuilder::new()
        .with_title("RiftCore - Minimalist League Companion")
        .with_inner_size(LogicalSize::new(1240.0, 820.0))
        .with_min_inner_size(LogicalSize::new(240.0, 180.0))
        .with_transparent(true)
        .with_decorations(false)
        .build(&event_loop)?;

    let window = Arc::new(window);

    let mut tray_icon_handle: *mut std::ffi::c_void = std::ptr::null_mut();
    // Apply native taskbar icon from own embedded PE binary (large & small)
    #[cfg(target_os = "windows")]
    unsafe {
        let hwnd = window.hwnd() as *mut std::ffi::c_void;

        let mut exe_path = [0u16; 1024];
        let len = GetModuleFileNameW(std::ptr::null_mut(), exe_path.as_mut_ptr(), 1024);
        if len > 0 {
            let mut h_large: *mut std::ffi::c_void = std::ptr::null_mut();
            let mut h_small: *mut std::ffi::c_void = std::ptr::null_mut();
            let count = ExtractIconExW(exe_path.as_ptr(), 0, &mut h_large, &mut h_small, 1);
            if count > 0 {
                if !h_large.is_null() {
                    SendMessageW(hwnd, WM_SETICON, ICON_BIG, h_large as isize);
                    SetClassLongPtrW(hwnd, GCLP_HICON, h_large as isize);
                }
                if !h_small.is_null() {
                    SendMessageW(hwnd, WM_SETICON, ICON_SMALL, h_small as isize);
                    SetClassLongPtrW(hwnd, GCLP_HICONSM, h_small as isize);
                    tray_icon_handle = h_small;
                } else {
                    tray_icon_handle = h_large;
                }
            }
        }

        // Apply Windows 11 DWM theme: Dark mode and blend border into #05080e background
        const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;
        const DWMWA_BORDER_COLOR: u32 = 34;
        let dark_mode: i32 = 1;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark_mode as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<i32>() as u32,
        );
        // COLORREF format 0x00BBGGRR: #05080e -> R=0x05, G=0x08, B=0x0e -> 0x000E0805
        let border_color: u32 = 0x000E0805;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &border_color as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        );
    }
    let proxy = event_loop.create_proxy();

    #[cfg(target_os = "windows")]
    start_system_tray(proxy.clone(), tray_icon_handle);

    // Global in-game Hotkey Thread: GetAsyncKeyState hardware polling
    // Bypasses DirectInput/RawInput locks in Fullscreen games!
    #[cfg(target_os = "windows")]
    {
        let hotkey_proxy = proxy.clone();
        std::thread::spawn(move || {
            let mut was_pressed = false;

            loop {
                std::thread::sleep(std::time::Duration::from_millis(20));

                let ctrl = unsafe { (GetAsyncKeyState(0x11 /* VK_CONTROL */) as u16 & 0x8000) != 0 };
                let alt = unsafe { (GetAsyncKeyState(0x12 /* VK_MENU / Alt */) as u16 & 0x8000) != 0 };
                let space = unsafe { (GetAsyncKeyState(0x20 /* VK_SPACE */) as u16 & 0x8000) != 0 };
                let tilde = unsafe { (GetAsyncKeyState(0xC0 /* VK_OEM_3 / ~ */) as u16 & 0x8000) != 0 };
                let x_key = unsafe { (GetAsyncKeyState(0x58 /* X */) as u16 & 0x8000) != 0 };
                let f10 = unsafe { (GetAsyncKeyState(0x79 /* VK_F10 */) as u16 & 0x8000) != 0 };

                let is_combo_down = (ctrl && space) || (alt && tilde) || (alt && x_key) || (ctrl && tilde) || f10;

                if is_combo_down && !was_pressed {
                    let _ = hotkey_proxy.send_event(AppUserEvent::ToggleOverlay);
                }
                was_pressed = is_combo_down;
            }
        });
    }

    let http_client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    // Locate dist directory reliably
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let possible_paths = [
        exe_dir.join("dist"),
        exe_dir.join("../dist"),
        exe_dir.join("../../dist"),
        PathBuf::from("F:/league app/dist"),
    ];

    let dist_dir = possible_paths
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_else(|| PathBuf::from("F:/league app/dist"));

    let state_clone = Arc::clone(&state);
    let window_clone = Arc::clone(&window);
    let http_clone = http_client.clone();
    let proxy_ipc = proxy.clone();

    let dist_dir_protocol = dist_dir.clone();

    let builder = WebViewBuilder::new()
        .with_transparent(true);

    #[cfg(target_os = "windows")]
    let builder = builder
        .with_browser_accelerator_keys(false)
        .with_devtools(false);

    let builder = builder
        .with_initialization_script(r#"
            window.__riftEnvironment = 'wry';
            console.log('[RiftCore] Running inside Native Rust runtime on Drive F:');
        "#)
        .with_custom_protocol("rift".into(), move |_webview_id, request: Request<Vec<u8>>| {
            let path = request.uri().path();
            let decoded = path.replace("%20", " ");
            let clean_path = if decoded == "/" || decoded.is_empty() {
                "index.html"
            } else {
                decoded.trim_start_matches(|c| c == '/' || c == '\\')
            };

            // 1. First priority: Load directly from embedded assets in RAM (Zero loose files required!)
            if let Some(file) = EmbeddedAssets::get(clean_path) {
                let mime = mime_guess::from_path(clean_path)
                    .first_or_octet_stream()
                    .as_ref()
                    .to_string();

                return Response::builder()
                    .header(CONTENT_TYPE, mime)
                    .header("Access-Control-Allow-Origin", "*")
                    .body(Cow::Owned(file.data.into_owned()))
                    .unwrap();
            }

            // 2. Fallback: Check adjacent disk dist folder (allows hot dev tweaks if folder exists)
            let file_path = dist_dir_protocol.join(clean_path);
            if let Ok(content) = std::fs::read(&file_path) {
                let mime = mime_guess::from_path(clean_path)
                    .first_or_octet_stream()
                    .as_ref()
                    .to_string();

                return Response::builder()
                    .header(CONTENT_TYPE, mime)
                    .header("Access-Control-Allow-Origin", "*")
                    .body(Cow::Owned(content))
                    .unwrap();
            }

            // 3. Fallback to embedded index.html for SPA client-side routing
            if let Some(index_file) = EmbeddedAssets::get("index.html") {
                return Response::builder()
                    .header(CONTENT_TYPE, "text/html; charset=utf-8")
                    .header("Access-Control-Allow-Origin", "*")
                    .body(Cow::Owned(index_file.data.into_owned()))
                    .unwrap();
            }

            // 4. 404 Not Found
            Response::builder()
                .status(404)
                .body(Cow::Owned(Vec::new()))
                .unwrap()
        })
        .with_ipc_handler(move |req: wry::http::Request<String>| {
            let msg = req.body();
            if let Ok(json) = serde_json::from_str::<Value>(msg) {
                let action = json.get("action").and_then(|v| v.as_str()).unwrap_or("");
                let req_id = json.get("id").and_then(|v| v.as_i64()).unwrap_or(0);

                match action {
                    "drag-window" => {
                        let _ = window_clone.drag_window();
                    }
                    "resize-window" => {
                        let w = json.get("width").and_then(|v| v.as_i64()).unwrap_or(340) as i32;
                        let h = json.get("height").and_then(|v| v.as_i64()).unwrap_or(270) as i32;
                        #[cfg(target_os = "windows")]
                        unsafe {
                            let hwnd = window_clone.hwnd() as *mut std::ffi::c_void;
                            SetWindowPos(
                                hwnd,
                                std::ptr::null_mut(),
                                0,
                                0,
                                w,
                                h,
                                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                            );
                        }
                        let reply_script = format!("window.__riftIpcReply({}, null, {{ width: {}, height: {} }});", req_id, w, h);
                        let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(reply_script));
                    }
                    "snap-overlay" => {
                        let position = json.get("position").and_then(|v| v.as_str()).unwrap_or("top-right");
                        let w = json.get("width").and_then(|v| v.as_i64()).unwrap_or(340) as i32;
                        let h = json.get("height").and_then(|v| v.as_i64()).unwrap_or(270) as i32;
                        #[cfg(target_os = "windows")]
                        unsafe {
                            let sw = GetSystemMetrics(0 /* SM_CXSCREEN */);
                            let (x, y) = match position {
                                "top-center" => (((sw - w) / 2).max(0), 12),
                                "top-left" => (24, 60),
                                _ => ((sw - w - 24).max(0), 60),
                            };
                            let hwnd = window_clone.hwnd() as *mut std::ffi::c_void;
                            SetWindowPos(
                                hwnd,
                                HWND_TOPMOST as *mut std::ffi::c_void,
                                x,
                                y,
                                w,
                                h,
                                SWP_NOACTIVATE | SWP_SHOWWINDOW,
                            );
                        }
                        let reply_script = format!("window.__riftIpcReply({}, null, {{ snapped: true }});", req_id);
                        let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(reply_script));
                    }
                    "set-hud-mode" => {
                        let mode = json.get("mode").and_then(|v| v.as_str()).unwrap_or("card");
                        #[cfg(target_os = "windows")]
                        unsafe {
                            let hwnd = window_clone.hwnd() as *mut std::ffi::c_void;
                            let sw = GetSystemMetrics(0 /* SM_CXSCREEN */);
                            let sh = GetSystemMetrics(1 /* SM_CYSCREEN */);
                            let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);

                            if mode == "integrated" {
                                // Fullscreen transparent click-through layer covering the game
                                let new_ex = (ex | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT) & !WS_EX_APPWINDOW;
                                SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex);
                                SetWindowPos(
                                    hwnd,
                                    HWND_TOPMOST as *mut std::ffi::c_void,
                                    0, 0, sw, sh,
                                    SWP_NOACTIVATE | SWP_SHOWWINDOW,
                                );
                            } else if mode == "card" {
                                let new_ex = (ex | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW) & !(WS_EX_APPWINDOW | WS_EX_TRANSPARENT);
                                SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex);
                                let w = 340;
                                let h = 270;
                                let x = (sw - w - 24).max(0);
                                let y = 60;
                                SetWindowPos(
                                    hwnd,
                                    HWND_TOPMOST as *mut std::ffi::c_void,
                                    x, y, w, h,
                                    SWP_NOACTIVATE | SWP_SHOWWINDOW,
                                );
                            } else if mode == "bar" {
                                let new_ex = (ex | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW) & !(WS_EX_APPWINDOW | WS_EX_TRANSPARENT);
                                SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex);
                                let w = 620;
                                let h = 42;
                                let x = ((sw - w) / 2).max(0);
                                let y = 12;
                                SetWindowPos(
                                    hwnd,
                                    HWND_TOPMOST as *mut std::ffi::c_void,
                                    x, y, w, h,
                                    SWP_NOACTIVATE | SWP_SHOWWINDOW,
                                );
                            }
                        }
                        let reply_script = format!("window.__riftIpcReply({}, null, {{ mode: '{}' }});", req_id, mode);
                        let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(reply_script));
                    }
                    "set-click-through" => {
                        let enable = json.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
                        #[cfg(target_os = "windows")]
                        unsafe {
                            let hwnd = window_clone.hwnd() as *mut std::ffi::c_void;
                            let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
                            let new_ex = if enable {
                                ex | WS_EX_TRANSPARENT
                            } else {
                                ex & !WS_EX_TRANSPARENT
                            };
                            SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex);
                        }
                        let reply_script = format!("window.__riftIpcReply({}, null, {{ clickThrough: {} }});", req_id, enable);
                        let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(reply_script));
                    }
                    "window-control" => {
                        let sub_action = json.get("subAction").and_then(|v| v.as_str()).unwrap_or("");
                        match sub_action {
                            "minimize" => {
                                window_clone.set_minimized(true);
                            }
                            "maximize" => {
                                let is_max = window_clone.is_maximized();
                                window_clone.set_maximized(!is_max);
                            }
                            "close" | "minimize-to-tray" => {
                                window_clone.set_visible(false);
                            }
                            "exit-app" => {
                                let _ = proxy_ipc.send_event(AppUserEvent::Exit);
                            }
                            "toggle-pin" => {
                                let mut st = state_clone.lock().unwrap();
                                st.is_pinned = !st.is_pinned;
                                #[cfg(target_os = "windows")]
                                {
                                    let hwnd = window_clone.hwnd() as *mut std::ffi::c_void;
                                    apply_fullscreen_topmost(hwnd, st.is_pinned);
                                }
                                let is_pinned = st.is_pinned;
                                let reply_script = format!("window.__riftIpcReply({}, null, {{ isPinned: {} }});", req_id, is_pinned);
                                let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(reply_script));
                            }
                            "set-pin-on" => {
                                let mut st = state_clone.lock().unwrap();
                                st.is_pinned = true;
                                #[cfg(target_os = "windows")]
                                {
                                    let hwnd = window_clone.hwnd() as *mut std::ffi::c_void;
                                    apply_fullscreen_topmost(hwnd, true);
                                }
                                let reply_script = format!("window.__riftIpcReply({}, null, {{ isPinned: true }});", req_id);
                                let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(reply_script));
                            }
                            "set-pin-off" => {
                                let mut st = state_clone.lock().unwrap();
                                st.is_pinned = false;
                                #[cfg(target_os = "windows")]
                                {
                                    let hwnd = window_clone.hwnd() as *mut std::ffi::c_void;
                                    apply_fullscreen_topmost(hwnd, false);
                                }
                                let reply_script = format!("window.__riftIpcReply({}, null, {{ isPinned: false }});", req_id);
                                let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(reply_script));
                            }
                            "toggle-compact" => {
                                let mut st = state_clone.lock().unwrap();
                                #[cfg(target_os = "windows")]
                                let hwnd = window_clone.hwnd() as *mut std::ffi::c_void;

                                #[cfg(target_os = "windows")]
                                let is_in_game = unsafe {
                                    let class_name: Vec<u16> = "RiotWindowClass\0".encode_utf16().collect();
                                    let league_hwnd = FindWindowW(class_name.as_ptr(), std::ptr::null());
                                    !league_hwnd.is_null() || st.gameflow_phase == "InProgress"
                                };
                                #[cfg(not(target_os = "windows"))]
                                let is_in_game = st.gameflow_phase == "InProgress";

                                if !st.is_compact && !is_in_game {
                                    let reply_script = format!(
                                        "window.__riftIpcReply({}, null, {{ isCompactMode: false, isPinned: false, notInGame: true }});",
                                        req_id
                                    );
                                    let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(reply_script));
                                    return;
                                }

                                st.is_compact = !st.is_compact;
                                if st.is_compact {
                                    st.is_pinned = true;
                                    #[cfg(target_os = "windows")]
                                    unsafe {
                                        maintain_overlay_topmost(hwnd);
                                        let sw = GetSystemMetrics(0 /* SM_CXSCREEN */);
                                        let sh = GetSystemMetrics(1 /* SM_CYSCREEN */);
                                        let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
                                        let new_ex = (ex | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT) & !WS_EX_APPWINDOW;
                                        SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex);
                                        SetWindowPos(
                                            hwnd,
                                            HWND_TOPMOST as *mut std::ffi::c_void,
                                            0,
                                            0,
                                            sw,
                                            sh,
                                            SWP_NOACTIVATE | SWP_SHOWWINDOW,
                                        );
                                    }
                                } else {
                                    st.is_pinned = false;
                                    #[cfg(target_os = "windows")]
                                    unsafe {
                                        let sw = GetSystemMetrics(0 /* SM_CXSCREEN */);
                                        let sh = GetSystemMetrics(1 /* SM_CYSCREEN */);
                                        let app_w = 1240;
                                        let app_h = 820;
                                        let app_x = ((sw - app_w) / 2).max(0);
                                        let app_y = ((sh - app_h) / 2).max(0);
                                        let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
                                        let normal_ex = (ex | WS_EX_APPWINDOW) & !(WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST);
                                        SetWindowLongW(hwnd, GWL_EXSTYLE, normal_ex);
                                        SetWindowPos(
                                            hwnd,
                                            HWND_NOTOPMOST as *mut std::ffi::c_void,
                                            app_x,
                                            app_y,
                                            app_w,
                                            app_h,
                                            SWP_SHOWWINDOW | SWP_FRAMECHANGED,
                                        );
                                        apply_fullscreen_topmost(hwnd, false);
                                        SetForegroundWindow(hwnd);
                                    }
                                }
                                let is_compact = st.is_compact;
                                let is_pinned = st.is_pinned;
                                let reply_script = format!(
                                    "window.__riftIpcReply({}, null, {{ isCompactMode: {}, isPinned: {} }});",
                                    req_id, is_compact, is_pinned
                                );
                                let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(reply_script));
                            }
                            _ => {}
                        }
                    }
                    "set-always-on-top" => {
                        let on_top = json.get("alwaysOnTop").and_then(|v| v.as_bool()).unwrap_or(true);
                        let mut st = state_clone.lock().unwrap();
                        st.is_pinned = on_top;
                        #[cfg(target_os = "windows")]
                        {
                            let hwnd = window_clone.hwnd() as *mut std::ffi::c_void;
                            apply_fullscreen_topmost(hwnd, on_top);
                        }
                        let reply_script = format!("window.__riftIpcReply({}, null, {{ isPinned: {} }});", req_id, on_top);
                        let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(reply_script));
                    }
                    "set-compact-hud" => {
                        let compact = json.get("compact").and_then(|v| v.as_bool()).unwrap_or(true);
                        let mut st = state_clone.lock().unwrap();
                        st.is_compact = compact;
                        #[cfg(target_os = "windows")]
                        let hwnd = window_clone.hwnd() as *mut std::ffi::c_void;
                        if compact {
                            st.is_pinned = true;
                            #[cfg(target_os = "windows")]
                            unsafe {
                                maintain_overlay_topmost(hwnd);
                                let sw = GetSystemMetrics(0 /* SM_CXSCREEN */);
                                let sh = GetSystemMetrics(1 /* SM_CYSCREEN */);
                                let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
                                let new_ex = (ex | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT) & !WS_EX_APPWINDOW;
                                SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex);
                                SetWindowPos(
                                    hwnd,
                                    HWND_TOPMOST as *mut std::ffi::c_void,
                                    0,
                                    0,
                                    sw,
                                    sh,
                                    SWP_NOACTIVATE | SWP_SHOWWINDOW,
                                );
                            }
                        } else {
                            st.is_pinned = false;
                            #[cfg(target_os = "windows")]
                            unsafe {
                                let sw = GetSystemMetrics(0 /* SM_CXSCREEN */);
                                let sh = GetSystemMetrics(1 /* SM_CYSCREEN */);
                                let app_w = 1240;
                                let app_h = 820;
                                let app_x = ((sw - app_w) / 2).max(0);
                                let app_y = ((sh - app_h) / 2).max(0);
                                let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
                                let normal_ex = (ex | WS_EX_APPWINDOW) & !(WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST);
                                SetWindowLongW(hwnd, GWL_EXSTYLE, normal_ex);
                                SetWindowPos(
                                    hwnd,
                                    HWND_NOTOPMOST as *mut std::ffi::c_void,
                                    app_x,
                                    app_y,
                                    app_w,
                                    app_h,
                                    SWP_SHOWWINDOW | SWP_FRAMECHANGED,
                                );
                                apply_fullscreen_topmost(hwnd, false);
                                SetForegroundWindow(hwnd);
                            }
                        }
                        let is_compact = st.is_compact;
                        let is_pinned = st.is_pinned;
                        let reply_script = format!(
                            "window.__riftIpcReply({}, null, {{ isCompactMode: {}, isPinned: {} }});",
                            req_id, is_compact, is_pinned
                        );
                        let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(reply_script));
                    }
                    "focus-window" => {
                        window_clone.set_focus();
                        let reply_script = format!("window.__riftIpcReply({}, null, {{ success: true }});", req_id);
                        let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(reply_script));
                    }
                    "check-client-status" => {
                        let st = state_clone.lock().unwrap();
                        let connected = st.credentials.is_some();
                        let phase = st.gameflow_phase.clone();
                        let riot_ready = is_riot_client_open();
                        let reply_script = format!(
                            "window.__riftIpcReply({}, null, {{ connected: {}, gameflowPhase: '{}', riotClient: {} }});",
                            req_id, connected, phase, riot_ready
                        );
                        let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(reply_script));
                    }
                    "launch-league" => {
                        let league_paths = [
                            "E:\\Riot Games\\League of Legends\\LeagueClient.exe",
                            "F:\\Riot Games\\League of Legends (PBE)\\LeagueClient.exe",
                            "C:\\Riot Games\\League of Legends\\LeagueClient.exe",
                            "D:\\Riot Games\\League of Legends\\LeagueClient.exe",
                        ];
                        for path in &league_paths {
                            if Path::new(path).exists() {
                                let _ = Command::new("cmd")
                                    .args(["/c", "start", "", path])
                                    .spawn();
                                break;
                            }
                        }
                        let script = format!("window.__riftIpcReply({}, null, {{ success: true }});", req_id);
                        let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(script));
                    }
                    "lcu-request" => {
                        let endpoint = json.get("endpoint").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let method = json.get("method").and_then(|v| v.as_str()).unwrap_or("GET").to_string();
                        let body = json.get("body").cloned();
                        let creds = state_clone.lock().unwrap().credentials.clone();
                        let client = http_clone.clone();
                        let proxy_worker = proxy_ipc.clone();

                        tokio::spawn(async move {
                            if let Some(c) = creds {
                                let url = format!("https://127.0.0.1:{}{}", c.port, endpoint);
                                let auth = format!("Basic {}", BASE64_STANDARD.encode(format!("riot:{}", c.token)));

                                let req_builder = match method.to_uppercase().as_str() {
                                    "POST" => client.post(&url),
                                    "PUT" => client.put(&url),
                                    "PATCH" => client.patch(&url),
                                    "DELETE" => client.delete(&url),
                                    _ => client.get(&url),
                                };

                                let mut req_builder = req_builder
                                    .header("Authorization", auth)
                                    .header("Content-Type", "application/json");

                                if let Some(b) = body {
                                    req_builder = req_builder.json(&b);
                                }

                                match req_builder.send().await {
                                    Ok(res) => {
                                        let text = res.text().await.unwrap_or_else(|_| "{}".to_string());
                                        let script = format!("window.__riftIpcReply({}, null, {});", req_id, text);
                                        let _ = proxy_worker.send_event(AppUserEvent::EvalScript(script));
                                    }
                                    Err(e) => {
                                        let err_msg = format!("{:?}", e).replace('"', "\\\"");
                                        let script = format!("window.__riftIpcReply({}, \"{}\", null);", req_id, err_msg);
                                        let _ = proxy_worker.send_event(AppUserEvent::EvalScript(script));
                                    }
                                }
                            } else {
                                let script = format!("window.__riftIpcReply({}, \"Client Not Connected\", null);", req_id);
                                let _ = proxy_worker.send_event(AppUserEvent::EvalScript(script));
                            }
                        });
                    }
                    "riot-web-api" => {
                        let url = json.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let key = json.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let client = http_clone.clone();
                        let proxy_worker = proxy_ipc.clone();

                        tokio::spawn(async move {
                            if !url.is_empty() {
                                let mut req = client.get(&url);
                                if !key.is_empty() {
                                    req = req.header("X-Riot-Token", &key);
                                }
                                match req.send().await {
                                    Ok(res) => {
                                        let status = res.status().as_u16();
                                        let body_text = res.text().await.unwrap_or_else(|_| "{}".to_string());
                                        let script = format!(
                                            "window.__riftIpcReply({}, null, {{ status: {}, data: {} }});",
                                            req_id, status, body_text
                                        );
                                        let _ = proxy_worker.send_event(AppUserEvent::EvalScript(script));
                                    }
                                    Err(e) => {
                                        let script = format!(
                                            "window.__riftIpcReply({}, null, {{ status: 500, data: {{ error: \"{}\" }} }});",
                                            req_id, e
                                        );
                                        let _ = proxy_worker.send_event(AppUserEvent::EvalScript(script));
                                    }
                                }
                            } else {
                                let script = format!("window.__riftIpcReply({}, \"Empty URL\", null);", req_id);
                                let _ = proxy_worker.send_event(AppUserEvent::EvalScript(script));
                            }
                        });
                    }
                    "accept-match" => {
                        let creds = state_clone.lock().unwrap().credentials.clone();
                        let client = http_clone.clone();
                        let proxy_worker = proxy_ipc.clone();

                        tokio::spawn(async move {
                            if let Some(c) = creds {
                                let url = format!("https://127.0.0.1:{}/lol-matchmaking/v1/ready-check/accept", c.port);
                                let auth = format!("Basic {}", BASE64_STANDARD.encode(format!("riot:{}", c.token)));
                                let _ = client
                                    .post(&url)
                                    .header("Authorization", auth)
                                    .header("Content-Type", "application/json")
                                    .body("{}")
                                    .send()
                                    .await;
                            }
                            let script = format!("window.__riftIpcReply({}, null, {{ success: true }});", req_id);
                            let _ = proxy_worker.send_event(AppUserEvent::EvalScript(script));
                        });
                    }
                    "import-runes" => {
                        let page_data = json.get("pageData").cloned();
                        let creds = state_clone.lock().unwrap().credentials.clone();
                        let client = http_clone.clone();
                        let proxy_worker = proxy_ipc.clone();

                        tokio::spawn(async move {
                            if let Some(c) = creds {
                                let auth = format!("Basic {}", BASE64_STANDARD.encode(format!("riot:{}", c.token)));
                                let pages_url = format!("https://127.0.0.1:{}/lol-perks/v1/pages", c.port);
                                let mut target_pid = None;
                                let mut fallback_del_pid = None;
                                let mut total_pages = 0;

                                if let Ok(res) = client.get(&pages_url).header("Authorization", &auth).send().await {
                                    if let Ok(pages) = res.json::<Vec<serde_json::Value>>().await {
                                        total_pages = pages.len();
                                        for p in &pages {
                                            let is_del = p.get("isDeletable").and_then(|v| v.as_bool()).unwrap_or(false);
                                            let is_edit = p.get("isEditable").and_then(|v| v.as_bool()).unwrap_or(false);
                                            let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                            if let Some(pid) = p.get("id").and_then(|v| v.as_i64()) {
                                                if (is_del || is_edit) && (name.contains("Rift") || name.contains("RiftCore")) {
                                                    target_pid = Some(pid);
                                                    break;
                                                }
                                                if is_del {
                                                    fallback_del_pid = Some(pid);
                                                }
                                            }
                                        }
                                    }
                                }

                                if let Some(mut pd) = page_data {
                                    if let Some(obj) = pd.as_object_mut() {
                                        let raw_name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("RiftCore Build");
                                        let clean_name: String = raw_name.chars().filter(|ch| ch.is_ascii_alphanumeric() || *ch == ' ').collect();
                                        let clean_name = if clean_name.trim().is_empty() { "RiftCore Build".to_string() } else { clean_name.trim().to_string() };
                                        obj.insert("name".to_string(), serde_json::Value::String(clean_name));
                                        obj.insert("current".to_string(), serde_json::Value::Bool(true));
                                        obj.remove("isActive");
                                    }

                                    let mut applied = false;
                                    let mut final_pid = target_pid;

                                    // 1. If an existing RiftCore page is found, update it directly via PUT
                                    if let Some(pid) = target_pid {
                                        let put_url = format!("https://127.0.0.1:{}/lol-perks/v1/pages/{}", c.port, pid);
                                        if let Some(obj) = pd.as_object_mut() {
                                            obj.insert("id".to_string(), serde_json::Value::Number(serde_json::Number::from(pid)));
                                        }
                                        if let Ok(put_res) = client.put(&put_url).header("Authorization", &auth).json(&pd).send().await {
                                            if put_res.status().is_success() {
                                                applied = true;
                                            }
                                        }
                                    }

                                    // 2. If no RiftCore page or PUT failed
                                    if !applied {
                                        if let Some(obj) = pd.as_object_mut() {
                                            obj.remove("id");
                                        }
                                        let create_url = format!("https://127.0.0.1:{}/lol-perks/v1/pages", c.port);

                                        // If page limit reached (5 or more pages), delete fallback deletable page first
                                        if total_pages >= 5 {
                                            if let Some(del_id) = fallback_del_pid {
                                                let p_del = format!("https://127.0.0.1:{}/lol-perks/v1/pages/{}", c.port, del_id);
                                                let _ = client.delete(&p_del).header("Authorization", &auth).send().await;
                                            }
                                        }

                                        let post_res = client.post(&create_url).header("Authorization", &auth).json(&pd).send().await;
                                        if let Ok(res) = post_res {
                                            if res.status().is_success() {
                                                applied = true;
                                                if let Ok(created_page) = res.json::<serde_json::Value>().await {
                                                    if let Some(nid) = created_page.get("id").and_then(|v| v.as_i64()) {
                                                        final_pid = Some(nid);
                                                    }
                                                }
                                            } else if let Some(del_id) = fallback_del_pid {
                                                // Retry with delete
                                                let p_del = format!("https://127.0.0.1:{}/lol-perks/v1/pages/{}", c.port, del_id);
                                                let _ = client.delete(&p_del).header("Authorization", &auth).send().await;
                                                let retry = client.post(&create_url).header("Authorization", &auth).json(&pd).send().await;
                                                if let Ok(r) = retry {
                                                    if r.status().is_success() {
                                                        applied = true;
                                                        if let Ok(created_page) = r.json::<serde_json::Value>().await {
                                                            if let Some(nid) = created_page.get("id").and_then(|v| v.as_i64()) {
                                                                final_pid = Some(nid);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // 3. Ensure the page is marked current in the LCU
                                    if applied {
                                        if let Some(pid) = final_pid {
                                            let set_current_url = format!("https://127.0.0.1:{}/lol-perks/v1/currentpage", c.port);
                                            let _ = client.put(&set_current_url).header("Authorization", &auth).json(&serde_json::Value::Number(serde_json::Number::from(pid))).send().await;
                                        }
                                    }
                                }
                            }
                            let script = format!("window.__riftIpcReply({}, null, {{ success: true }});", req_id);
                            let _ = proxy_worker.send_event(AppUserEvent::EvalScript(script));
                        });
                    }
                    "import-spells" => {
                        let s1 = json.get("spell1Id").and_then(|v| v.as_i64()).unwrap_or(4);
                        let s2 = json.get("spell2Id").and_then(|v| v.as_i64()).unwrap_or(12);
                        let creds = state_clone.lock().unwrap().credentials.clone();
                        let client = http_clone.clone();
                        let proxy_worker = proxy_ipc.clone();

                        tokio::spawn(async move {
                            if let Some(c) = creds {
                                let auth = format!("Basic {}", BASE64_STANDARD.encode(format!("riot:{}", c.token)));
                                let spell_url = format!("https://127.0.0.1:{}/lol-champ-select/v1/session/my-selection", c.port);
                                let body = serde_json::json!({
                                    "spell1Id": s1,
                                    "spell2Id": s2
                                });
                                let _ = client.patch(&spell_url).header("Authorization", &auth).json(&body).send().await;
                            }
                            let script = format!("window.__riftIpcReply({}, null, {{ success: true }});", req_id);
                            let _ = proxy_worker.send_event(AppUserEvent::EvalScript(script));
                        });
                    }
                    "export-item-set" => {
                        let item_set = json.get("itemSet").cloned();
                        let champion_key_opt = json.get("championKey").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let creds = state_clone.lock().unwrap().credentials.clone();
                        let client = http_clone.clone();
                        let proxy_worker = proxy_ipc.clone();

                        tokio::spawn(async move {
                            if let (Some(c), Some(mut new_set)) = (creds, item_set) {
                                let auth = format!("Basic {}", BASE64_STANDARD.encode(format!("riot:{}", c.token)));

                                // Clean up non-ASCII emojis from title and blocks for League engine compatibility
                                if let Some(obj) = new_set.as_object_mut() {
                                    if let Some(title_val) = obj.get("title").and_then(|v| v.as_str()) {
                                        let clean_title: String = title_val.chars().filter(|ch| ch.is_ascii()).collect();
                                        let clean_title = clean_title.trim().to_string();
                                        obj.insert("title".to_string(), serde_json::Value::String(clean_title));
                                    }
                                    obj.insert("associatedMaps".to_string(), serde_json::Value::Array(Vec::new()));
                                    obj.insert("map".to_string(), serde_json::Value::String("any".to_string()));
                                    obj.insert("mode".to_string(), serde_json::Value::String("any".to_string()));
                                }

                                // 1. Direct Disk Persistence to League Config directory
                                let mut candidate_dirs: Vec<PathBuf> = Vec::new();
                                if let Some(ref idir) = c.install_dir {
                                    candidate_dirs.push(idir.clone());
                                }
                                candidate_dirs.push(PathBuf::from(r"E:\Riot Games\League of Legends"));
                                candidate_dirs.push(PathBuf::from(r"C:\Riot Games\League of Legends"));
                                candidate_dirs.push(PathBuf::from(r"D:\Riot Games\League of Legends"));
                                candidate_dirs.push(PathBuf::from(r"F:\Riot Games\League of Legends (PBE)"));

                                for dir in &candidate_dirs {
                                    if dir.exists() {
                                        let config_dir = dir.join("Config");
                                        if config_dir.exists() {
                                            if let Some(ref ckey) = champion_key_opt {
                                                let champ_rec_dir = config_dir.join("Champions").join(ckey).join("Recommended");
                                                let _ = std::fs::create_dir_all(&champ_rec_dir);
                                                let champ_file = champ_rec_dir.join("RiftCore.json");
                                                let _ = std::fs::write(&champ_file, serde_json::to_string_pretty(&new_set).unwrap_or_default());
                                            }
                                            let global_rec_dir = config_dir.join("Global").join("Recommended");
                                            let _ = std::fs::create_dir_all(&global_rec_dir);
                                            let global_file = if let Some(ref ckey) = champion_key_opt {
                                                global_rec_dir.join(format!("RiftCore_{}.json", ckey))
                                            } else {
                                                global_rec_dir.join("RiftCore.json")
                                            };
                                            let _ = std::fs::write(&global_file, serde_json::to_string_pretty(&new_set).unwrap_or_default());
                                            break;
                                        }
                                    }
                                }

                                // 2. LCU API Synchronization
                                let summ_url = format!("https://127.0.0.1:{}/lol-summoner/v1/current-summoner", c.port);
                                if let Ok(s_res) = client.get(&summ_url).header("Authorization", &auth).send().await {
                                    if let Ok(summ_json) = s_res.json::<serde_json::Value>().await {
                                        let account_id = summ_json.get("accountId").and_then(|v| v.as_i64());
                                        let summoner_id = summ_json.get("summonerId").and_then(|v| v.as_i64());
                                        let target_id = account_id.or(summoner_id);

                                        if let Some(id) = target_id {
                                            let sets_url = format!("https://127.0.0.1:{}/lol-item-sets/v1/item-sets/{}/sets", c.port, id);
                                            let mut sets_obj = match client.get(&sets_url).header("Authorization", &auth).send().await {
                                                Ok(r) => r.json::<serde_json::Value>().await.unwrap_or_else(|_| serde_json::json!({ "itemSets": [] })),
                                                Err(_) => serde_json::json!({ "itemSets": [] }),
                                            };

                                            if let Some(arr) = sets_obj.get_mut("itemSets").and_then(|v| v.as_array_mut()) {
                                                let target_champ = new_set.get("associatedChampions")
                                                    .and_then(|ac| ac.as_array())
                                                    .and_then(|arr| arr.first())
                                                    .and_then(|v| v.as_i64());

                                                arr.retain(|s| {
                                                    let is_rift = s.get("title").and_then(|t| t.as_str()).map(|t| t.contains("RiftCore")).unwrap_or(false);
                                                    let match_champ = s.get("associatedChampions")
                                                        .and_then(|ac| ac.as_array())
                                                        .and_then(|a| a.first())
                                                        .and_then(|v| v.as_i64()) == target_champ;
                                                    !(is_rift && match_champ)
                                                });
                                                arr.push(new_set.clone());
                                            }

                                            // Put back to LCU with account_id and fallback to summoner_id
                                            let _ = client.put(&sets_url).header("Authorization", &auth).json(&sets_obj).send().await;
                                            if let (Some(sid), true) = (summoner_id, account_id.is_some()) {
                                                let s_url = format!("https://127.0.0.1:{}/lol-item-sets/v1/item-sets/{}/sets", c.port, sid);
                                                let _ = client.put(&s_url).header("Authorization", &auth).json(&sets_obj).send().await;
                                            }
                                        }
                                    }
                                }
                            }
                            let script = format!("window.__riftIpcReply({}, null, {{ success: true }});", req_id);
                            let _ = proxy_worker.send_event(AppUserEvent::EvalScript(script));
                        });
                    }
                    "get-live-client-data" => {
                        let client = http_clone.clone();
                        let proxy_worker = proxy_ipc.clone();

                        tokio::spawn(async move {
                            let live_url = "https://127.0.0.1:2999/liveclientdata/allgamedata";
                            match client.get(live_url).send().await {
                                Ok(res) => {
                                    let text = res.text().await.unwrap_or_default();
                                    let compact = match serde_json::from_str::<serde_json::Value>(&text) {
                                        Ok(val) => serde_json::to_string(&val).unwrap_or_else(|_| "null".to_string()),
                                        Err(_) => "null".to_string(),
                                    };
                                    let script = format!("window.__riftIpcReply({}, null, {});", req_id, compact);
                                    let _ = proxy_worker.send_event(AppUserEvent::EvalScript(script));
                                }
                                Err(_) => {
                                    let script = format!("window.__riftIpcReply({}, null, null);", req_id);
                                    let _ = proxy_worker.send_event(AppUserEvent::EvalScript(script));
                                }
                            }
                        });
                    }
                    "get-league-window-mode" => {
                        let mut mode = 0;
                        let mut found = false;
                        let mut cfg_path_str = String::new();
                        if let Some(cfg_path) = find_league_game_cfg() {
                            cfg_path_str = cfg_path.to_string_lossy().to_string();
                            if let Ok(content) = std::fs::read_to_string(&cfg_path) {
                                found = true;
                                for line in content.lines() {
                                    let trimmed = line.trim();
                                    if trimmed.starts_with("WindowMode=") {
                                        if let Some(val_str) = trimmed.strip_prefix("WindowMode=") {
                                            mode = val_str.parse::<i32>().unwrap_or(0);
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                        let script = format!(
                            "window.__riftIpcReply({}, null, {{ found: {}, windowMode: {}, path: \"{}\" }});",
                            req_id, found, mode, cfg_path_str.replace('\\', "\\\\")
                        );
                        let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(script));
                    }
                    "set-league-fullscreen" | "set-league-borderless" => {
                        let mut success = false;
                        if let Some(cfg_path) = find_league_game_cfg() {
                            if let Ok(content) = std::fs::read_to_string(&cfg_path) {
                                let mut new_lines = Vec::new();
                                let mut replaced = false;
                                for line in content.lines() {
                                    if line.trim().starts_with("WindowMode=") {
                                        new_lines.push("WindowMode=0".to_string());
                                        replaced = true;
                                    } else {
                                        new_lines.push(line.to_string());
                                    }
                                }
                                if !replaced {
                                    new_lines.push("WindowMode=0".to_string());
                                }
                                if std::fs::write(&cfg_path, new_lines.join("\r\n")).is_ok() {
                                    success = true;
                                }
                            }
                        }
                        let script = format!(
                            "window.__riftIpcReply({}, null, {{ success: {} }});",
                            req_id, success
                        );
                        let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(script));
                    }
                    "check-app-update" => {
                        let http_upd = http_clone.clone();
                        let proxy_upd = proxy_ipc.clone();
                        tokio::spawn(async move {
                            let current_version = env!("CARGO_PKG_VERSION");
                            match http_upd
                                .get("https://api.github.com/repos/kotsioskl2/riftcore-companion/releases/latest")
                                .header("User-Agent", "RiftCore-Updater")
                                .header("Accept", "application/vnd.github+json")
                                .send()
                                .await
                            {
                                Ok(res) if res.status().is_success() => {
                                    let text = res.text().await.unwrap_or_default();
                                    if let Ok(release) = serde_json::from_str::<serde_json::Value>(&text) {
                                        let tag = release.get("tag_name")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .trim_start_matches('v');
                                        let body = release.get("body")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .replace('\\', "\\\\")
                                            .replace('"', "\\\"")
                                            .replace('\n', "\\n")
                                            .replace('\r', "");
                                        let has_update = tag != current_version && !tag.is_empty();

                                        // Find the .exe installer asset download URL
                                        let mut download_url = String::new();
                                        let mut asset_size: u64 = 0;
                                        if let Some(assets) = release.get("assets").and_then(|v| v.as_array()) {
                                            for asset in assets {
                                                let name = asset.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                                if name.ends_with(".exe") && name.contains("Setup") {
                                                    download_url = asset.get("browser_download_url")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("")
                                                        .to_string();
                                                    asset_size = asset.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                                                    break;
                                                }
                                            }
                                        }

                                        let script = format!(
                                            "window.__riftIpcReply({}, null, {{ hasUpdate: {}, currentVersion: \"{}\", latestVersion: \"{}\", downloadUrl: \"{}\", assetSize: {}, releaseNotes: \"{}\" }});",
                                            req_id, has_update, current_version, tag, download_url, asset_size, body
                                        );
                                        let _ = proxy_upd.send_event(AppUserEvent::EvalScript(script));
                                    } else {
                                        let script = format!("window.__riftIpcReply({}, null, {{ hasUpdate: false, currentVersion: \"{}\", latestVersion: \"{}\", downloadUrl: \"\", assetSize: 0, releaseNotes: \"\" }});", req_id, current_version, current_version);
                                        let _ = proxy_upd.send_event(AppUserEvent::EvalScript(script));
                                    }
                                }
                                _ => {
                                    let current_version = env!("CARGO_PKG_VERSION");
                                    let script = format!("window.__riftIpcReply({}, null, {{ hasUpdate: false, currentVersion: \"{}\", latestVersion: \"{}\", downloadUrl: \"\", assetSize: 0, releaseNotes: \"\" }});", req_id, current_version, current_version);
                                    let _ = proxy_upd.send_event(AppUserEvent::EvalScript(script));
                                }
                            }
                        });
                    }
                    "download-app-update" => {
                        let url = json.get("downloadUrl").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let http_dl = http_clone.clone();
                        let proxy_dl = proxy_ipc.clone();
                        tokio::spawn(async move {
                            if url.is_empty() {
                                let script = format!("window.__riftIpcReply({}, \"No download URL provided\", null);", req_id);
                                let _ = proxy_dl.send_event(AppUserEvent::EvalScript(script));
                                return;
                            }

                            // Notify frontend: download started
                            let script = format!(
                                "window.dispatchEvent(new CustomEvent('rift-update-progress', {{ detail: {{ stage: 'downloading', percent: 0 }} }}));"
                            );
                            let _ = proxy_dl.send_event(AppUserEvent::EvalScript(script));

                            match http_dl.get(&url)
                                .header("User-Agent", "RiftCore-Updater")
                                .send()
                                .await
                            {
                                Ok(res) if res.status().is_success() => {
                                    let total_size = res.content_length().unwrap_or(0);
                                    let bytes = match res.bytes().await {
                                        Ok(b) => b,
                                        Err(e) => {
                                            let script = format!("window.__riftIpcReply({}, \"{}\", null);", req_id, e);
                                            let _ = proxy_dl.send_event(AppUserEvent::EvalScript(script));
                                            return;
                                        }
                                    };

                                    // Save to temp
                                    let temp_dir = std::env::temp_dir();
                                    let installer_path = temp_dir.join("RiftCore-Setup-latest.exe");
                                    if let Err(e) = std::fs::write(&installer_path, &bytes) {
                                        let script = format!("window.__riftIpcReply({}, \"Failed to save installer: {}\", null);", req_id, e);
                                        let _ = proxy_dl.send_event(AppUserEvent::EvalScript(script));
                                        return;
                                    }

                                    // Notify frontend: download complete, launching installer
                                    let script = format!(
                                        "window.dispatchEvent(new CustomEvent('rift-update-progress', {{ detail: {{ stage: 'installing', percent: 100 }} }}));"
                                    );
                                    let _ = proxy_dl.send_event(AppUserEvent::EvalScript(script));

                                    // Launch the installer and exit
                                    let _ = Command::new(&installer_path).spawn();

                                    // Give it a moment then exit
                                    std::thread::sleep(std::time::Duration::from_millis(500));
                                    std::process::exit(0);
                                }
                                Ok(res) => {
                                    let status = res.status();
                                    let script = format!("window.__riftIpcReply({}, \"Download failed with HTTP {}\", null);", req_id, status);
                                    let _ = proxy_dl.send_event(AppUserEvent::EvalScript(script));
                                }
                                Err(e) => {
                                    let script = format!("window.__riftIpcReply({}, \"{}\", null);", req_id, e);
                                    let _ = proxy_dl.send_event(AppUserEvent::EvalScript(script));
                                }
                            }
                        });
                    }
                    _ => {}
                }
            }
        });

    let webview = builder.with_url("http://rift.localhost").build(&window)?;

    // Background poller thread
    let bg_state = Arc::clone(&state);
    let bg_http = http_client.clone();
    let bg_proxy = proxy.clone();
    let window_bg = Arc::clone(&window);

    tokio::spawn(async move {
        let mut last_phase = String::new();
        let mut last_connected = false;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

            let current_creds = {
                let mut st = bg_state.lock().unwrap();
                st.credentials = detect_league_client();
                st.credentials.clone()
            };

            let is_connected = current_creds.is_some();
            if is_connected != last_connected {
                last_connected = is_connected;
                let riot_ready = is_riot_client_open();
                let script = format!(
                    "window.dispatchEvent(new CustomEvent('rift-lcu-status', {{ detail: {{ connected: {}, riotClient: {} }} }}));",
                    is_connected, riot_ready
                );
                let _ = bg_proxy.send_event(AppUserEvent::EvalScript(script));
            }

            if let Some(ref creds) = current_creds {
                let auth = format!("Basic {}", BASE64_STANDARD.encode(format!("riot:{}", creds.token)));

                // 1. Check Gameflow Phase
                let phase_url = format!("https://127.0.0.1:{}/lol-gameflow/v1/gameflow-phase", creds.port);
                if let Ok(res) = bg_http.get(&phase_url).header("Authorization", &auth).send().await {
                    if let Ok(phase) = res.json::<String>().await {
                        if phase != last_phase {
                            last_phase = phase.clone();
                            let mut st = bg_state.lock().unwrap();
                            st.gameflow_phase = phase.clone();

                            let script = format!(
                                "window.dispatchEvent(new CustomEvent('rift-phase', {{ detail: '{}' }}));",
                                phase
                            );
                            let _ = bg_proxy.send_event(AppUserEvent::EvalScript(script));
                        }
                    }
                }

                // 2. If ReadyCheck phase, check ready-check state
                if last_phase == "ReadyCheck" {
                    let rc_url = format!("https://127.0.0.1:{}/lol-matchmaking/v1/ready-check", creds.port);
                    if let Ok(res) = bg_http.get(&rc_url).header("Authorization", &auth).send().await {
                        if let Ok(rc_json) = res.text().await {
                            let script = format!(
                                "window.dispatchEvent(new CustomEvent('rift-ready-check', {{ detail: {} }}));",
                                rc_json
                            );
                            let _ = bg_proxy.send_event(AppUserEvent::EvalScript(script));
                        }
                    }
                }

                // 3. If ChampSelect phase, query champ select session
                if last_phase == "ChampSelect" {
                    let cs_url = format!("https://127.0.0.1:{}/lol-champ-select/v1/session", creds.port);
                    if let Ok(res) = bg_http.get(&cs_url).header("Authorization", &auth).send().await {
                        if res.status().is_success() {
                            if let Ok(cs_json) = res.text().await {
                                if !cs_json.contains("errorCode") {
                                    let script = format!(
                                        "window.dispatchEvent(new CustomEvent('rift-champ-select', {{ detail: {} }}));",
                                        cs_json
                                    );
                                    let _ = bg_proxy.send_event(AppUserEvent::EvalScript(script));
                                }
                            }
                        }
                    }
                }

                // 4. Live Client Data on port 2999 (Direct League of Legends in-game telemetry)
                // Polled with a 600ms timeout for all game modes (Summoner's Rift, ARAM, Arena, URF, etc.)
                let live_url = "https://127.0.0.1:2999/liveclientdata/allgamedata";
                if let Ok(res) = bg_http.get(live_url).timeout(std::time::Duration::from_millis(600)).send().await {
                    if let Ok(live_json) = res.text().await {
                        if live_json.contains("gameData") {
                            if last_phase != "InProgress" {
                                last_phase = "InProgress".to_string();
                                let mut st = bg_state.lock().unwrap();
                                st.gameflow_phase = "InProgress".to_string();
                                st.is_pinned = true;
                                st.is_compact = true;
                                #[cfg(target_os = "windows")]
                                unsafe {
                                    let hwnd = window_bg.hwnd() as *mut std::ffi::c_void;
                                    maintain_overlay_topmost(hwnd);
                                    let sw = GetSystemMetrics(0 /* SM_CXSCREEN */);
                                    let sh = GetSystemMetrics(1 /* SM_CYSCREEN */);
                                    let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
                                    let new_ex = (ex | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT) & !WS_EX_APPWINDOW;
                                    SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex);
                                    SetWindowPos(
                                        hwnd,
                                        HWND_TOPMOST as *mut std::ffi::c_void,
                                        0,
                                        0,
                                        sw,
                                        sh,
                                        SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED,
                                    );
                                }
                                let script = "window.dispatchEvent(new CustomEvent('rift-compact-hud', { detail: true })); window.dispatchEvent(new CustomEvent('rift-phase', { detail: 'InProgress' }));";
                                let _ = bg_proxy.send_event(AppUserEvent::EvalScript(script.to_string()));
                            } else {
                                #[cfg(target_os = "windows")]
                                {
                                    let is_compact = {
                                        let st = bg_state.lock().unwrap();
                                        st.is_compact
                                    };
                                    if is_compact {
                                        maintain_overlay_topmost(window_bg.hwnd() as *mut std::ffi::c_void);
                                    }
                                }
                            }

                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&live_json) {
                                if let Ok(compact_json) = serde_json::to_string(&val) {
                                    let script = format!(
                                        "window.dispatchEvent(new CustomEvent('rift-live-data', {{ detail: {} }}));",
                                        compact_json
                                    );
                                    let _ = bg_proxy.send_event(AppUserEvent::EvalScript(script));
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(AppUserEvent::Exit) => {
                #[cfg(target_os = "windows")]
                unsafe {
                    let tray_hwnd = TRAY_HWND.load(std::sync::atomic::Ordering::SeqCst);
                    if !tray_hwnd.is_null() {
                        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
                        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                        nid.hWnd = tray_hwnd;
                        nid.uID = TRAY_ID;
                        Shell_NotifyIconW(2 /* NIM_DELETE */, &mut nid);
                        DestroyWindow(tray_hwnd);
                    }
                }
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(AppUserEvent::ShowAndFocus) => {
                window.set_visible(true);
                window.set_minimized(false);
                window.set_focus();
                #[cfg(target_os = "windows")]
                {
                    let hwnd = window.hwnd() as *mut std::ffi::c_void;
                    unsafe {
                        SetForegroundWindow(hwnd);
                        BringWindowToTop(hwnd);
                    }
                }
            }
            Event::UserEvent(AppUserEvent::EvalScript(script)) => {
                let _ = webview.evaluate_script(&script);
            }
            Event::UserEvent(AppUserEvent::ToggleOverlay) => {
                let mut st = state.lock().unwrap();
                #[cfg(target_os = "windows")]
                let hwnd = window.hwnd() as *mut std::ffi::c_void;

                if st.is_compact {
                    // Restore to full desktop companion app
                    st.is_compact = false;
                    st.is_pinned = false;
                    window.set_visible(true);
                    window.set_minimized(false);

                    #[cfg(target_os = "windows")]
                    unsafe {
                        let sw = GetSystemMetrics(0 /* SM_CXSCREEN */);
                        let sh = GetSystemMetrics(1 /* SM_CYSCREEN */);
                        let app_w = 1240;
                        let app_h = 820;
                        let app_x = ((sw - app_w) / 2).max(0);
                        let app_y = ((sh - app_h) / 2).max(0);
                        let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
                        let normal_ex = (ex | WS_EX_APPWINDOW) & !(WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST);
                        SetWindowLongW(hwnd, GWL_EXSTYLE, normal_ex);
                        SetWindowPos(
                            hwnd,
                            HWND_NOTOPMOST as *mut std::ffi::c_void,
                            app_x,
                            app_y,
                            app_w,
                            app_h,
                            SWP_SHOWWINDOW | SWP_FRAMECHANGED,
                        );
                        apply_fullscreen_topmost(hwnd, false);
                        SetForegroundWindow(hwnd);
                    }
                    let _ = webview.evaluate_script("window.dispatchEvent(new CustomEvent('rift-compact-hud', { detail: false }));");
                    let _ = webview.evaluate_script("window.dispatchEvent(new CustomEvent('rift-overlay-toggled', { detail: { visible: false } }));");
                } else {
                    #[cfg(target_os = "windows")]
                    let is_in_game = unsafe {
                        let class_name: Vec<u16> = "RiotWindowClass\0".encode_utf16().collect();
                        let league_hwnd = FindWindowW(class_name.as_ptr(), std::ptr::null());
                        !league_hwnd.is_null() || st.gameflow_phase == "InProgress"
                    };
                    #[cfg(not(target_os = "windows"))]
                    let is_in_game = st.gameflow_phase == "InProgress";

                    if !is_in_game {
                        window.set_visible(true);
                        window.set_minimized(false);
                        #[cfg(target_os = "windows")]
                        unsafe {
                            SetForegroundWindow(hwnd);
                        }
                    } else {
                        // Switch to fullscreen in-game overlay
                        window.set_visible(true);
                        window.set_minimized(false);
                        st.is_compact = true;
                        st.is_pinned = true;

                    #[cfg(target_os = "windows")]
                    unsafe {
                        maintain_overlay_topmost(hwnd);
                        let sw = GetSystemMetrics(0 /* SM_CXSCREEN */);
                        let sh = GetSystemMetrics(1 /* SM_CYSCREEN */);
                        let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
                        let new_ex = (ex | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT) & !WS_EX_APPWINDOW;
                        SetWindowLongW(hwnd, GWL_EXSTYLE, new_ex);
                        SetWindowPos(
                            hwnd,
                            HWND_TOPMOST as *mut std::ffi::c_void,
                            0,
                            0,
                            sw,
                            sh,
                            SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED,
                        );
                    }
                    let _ = webview.evaluate_script("window.dispatchEvent(new CustomEvent('rift-compact-hud', { detail: true }));");
                    let _ = webview.evaluate_script("window.dispatchEvent(new CustomEvent('rift-overlay-toggled', { detail: { visible: true } }));");
                    }
                }
            }
            Event::UserEvent(AppUserEvent::ApplyTopmost(on_top)) => {
                #[cfg(target_os = "windows")]
                {
                    let hwnd = window.hwnd() as *mut std::ffi::c_void;
                    apply_fullscreen_topmost(hwnd, on_top);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                window.set_visible(false);
            }
            _ => (),
        }
    });
}
