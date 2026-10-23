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

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LcuCredentials {
    port: String,
    token: String,
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
                    return Some(LcuCredentials { port, token });
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
                if let Ok(content) = std::fs::read_to_string(lockfile) {
                    let parts: Vec<&str> = content.trim().split(':').collect();
                    if parts.len() >= 5 {
                        let port = parts[2].to_string();
                        let token = parts[3].to_string();
                        if !port.is_empty() && !token.is_empty() {
                            return Some(LcuCredentials { port, token });
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
extern "system" {
    fn CreateMutexW(lpMutexAttributes: *mut std::ffi::c_void, bInitialOwner: i32, lpName: *const u16) -> *mut std::ffi::c_void;
    fn GetLastError() -> u32;
}

const ERROR_ALREADY_EXISTS: u32 = 183;

fn is_already_running() -> bool {
    #[cfg(target_os = "windows")]
    unsafe {
        let name: Vec<u16> = "Global\\RiftCoreCompanion_SingleInstance\0".encode_utf16().collect();
        let handle = CreateMutexW(std::ptr::null_mut(), 1, name.as_ptr());
        if handle.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
            return true;
        }
    }
    false
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        .with_min_inner_size(LogicalSize::new(420.0, 600.0))
        .with_decorations(false)
        .build(&event_loop)?;

    let window = Arc::new(window);
    let proxy = event_loop.create_proxy();

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

    let builder = WebViewBuilder::new();

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

            let file_path = dist_dir_protocol.join(clean_path);

            match std::fs::read(&file_path) {
                Ok(content) => {
                    let mime = if clean_path.ends_with(".html") {
                        "text/html; charset=utf-8"
                    } else if clean_path.ends_with(".js") {
                        "application/javascript; charset=utf-8"
                    } else if clean_path.ends_with(".css") {
                        "text/css; charset=utf-8"
                    } else if clean_path.ends_with(".svg") {
                        "image/svg+xml"
                    } else if clean_path.ends_with(".png") {
                        "image/png"
                    } else if clean_path.ends_with(".json") {
                        "application/json"
                    } else if clean_path.ends_with(".ico") {
                        "image/x-icon"
                    } else if clean_path.ends_with(".woff2") {
                        "font/woff2"
                    } else if clean_path.ends_with(".woff") {
                        "font/woff"
                    } else {
                        "application/octet-stream"
                    };

                    Response::builder()
                        .header(CONTENT_TYPE, mime)
                        .header("Access-Control-Allow-Origin", "*")
                        .body(Cow::Owned(content))
                        .unwrap()
                }
                Err(e) => {
                    if clean_path != "favicon.ico" {
                        eprintln!("Failed reading asset {:?}: {}", file_path, e);
                    }
                    Response::builder()
                        .header(CONTENT_TYPE, "image/x-icon")
                        .status(404)
                        .body(Cow::Owned(Vec::new()))
                        .unwrap()
                }
            }
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
                            "close" => {
                                let _ = proxy_ipc.send_event(AppUserEvent::Exit);
                            }
                            "toggle-pin" => {
                                let mut st = state_clone.lock().unwrap();
                                st.is_pinned = !st.is_pinned;
                                window_clone.set_always_on_top(st.is_pinned);
                                if st.is_pinned {
                                    window_clone.set_focus();
                                }
                                let is_pinned = st.is_pinned;
                                let reply_script = format!("window.__riftIpcReply({}, null, {{ isPinned: {} }});", req_id, is_pinned);
                                let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(reply_script));
                            }
                            "set-pin-on" => {
                                let mut st = state_clone.lock().unwrap();
                                st.is_pinned = true;
                                window_clone.set_always_on_top(true);
                                window_clone.set_focus();
                                let reply_script = format!("window.__riftIpcReply({}, null, {{ isPinned: true }});", req_id);
                                let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(reply_script));
                            }
                            "set-pin-off" => {
                                let mut st = state_clone.lock().unwrap();
                                st.is_pinned = false;
                                window_clone.set_always_on_top(false);
                                let reply_script = format!("window.__riftIpcReply({}, null, {{ isPinned: false }});", req_id);
                                let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(reply_script));
                            }
                            "toggle-compact" => {
                                let mut st = state_clone.lock().unwrap();
                                st.is_compact = !st.is_compact;
                                if st.is_compact {
                                    window_clone.set_inner_size(LogicalSize::new(360.0, 720.0));
                                    window_clone.set_always_on_top(true);
                                    st.is_pinned = true;
                                    window_clone.set_focus();
                                } else {
                                    window_clone.set_inner_size(LogicalSize::new(1240.0, 820.0));
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
                        window_clone.set_always_on_top(on_top);
                        if on_top {
                            window_clone.set_focus();
                        }
                        let reply_script = format!("window.__riftIpcReply({}, null, {{ isPinned: {} }});", req_id, on_top);
                        let _ = proxy_ipc.send_event(AppUserEvent::EvalScript(reply_script));
                    }
                    "set-compact-hud" => {
                        let compact = json.get("compact").and_then(|v| v.as_bool()).unwrap_or(true);
                        let mut st = state_clone.lock().unwrap();
                        st.is_compact = compact;
                        if compact {
                            window_clone.set_inner_size(LogicalSize::new(360.0, 720.0));
                            window_clone.set_always_on_top(true);
                            st.is_pinned = true;
                            window_clone.set_focus();
                        } else {
                            window_clone.set_inner_size(LogicalSize::new(1240.0, 820.0));
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
                                // Delete current page if one exists
                                let del_url = format!("https://127.0.0.1:{}/lol-perks/v1/currentpage", c.port);
                                let _ = client.delete(&del_url).header("Authorization", &auth).send().await;

                                // Create new page
                                if let Some(pd) = page_data {
                                    let create_url = format!("https://127.0.0.1:{}/lol-perks/v1/pages", c.port);
                                    let _ = client.post(&create_url).header("Authorization", &auth).json(&pd).send().await;
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
                    "get-live-client-data" => {
                        let client = http_clone.clone();
                        let proxy_worker = proxy_ipc.clone();

                        tokio::spawn(async move {
                            let live_url = "https://127.0.0.1:2999/liveclientdata/allgamedata";
                            match client.get(live_url).send().await {
                                Ok(res) => {
                                    let text = res.text().await.unwrap_or_else(|_| "null".to_string());
                                    let script = format!("window.__riftIpcReply({}, null, {});", req_id, text);
                                    let _ = proxy_worker.send_event(AppUserEvent::EvalScript(script));
                                }
                                Err(_) => {
                                    let script = format!("window.__riftIpcReply({}, null, null);", req_id);
                                    let _ = proxy_worker.send_event(AppUserEvent::EvalScript(script));
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
                        if let Ok(cs_json) = res.text().await {
                            let script = format!(
                                "window.dispatchEvent(new CustomEvent('rift-champ-select', {{ detail: {} }}));",
                                cs_json
                            );
                            let _ = bg_proxy.send_event(AppUserEvent::EvalScript(script));
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
                                let script = "window.dispatchEvent(new CustomEvent('rift-phase', { detail: 'InProgress' }));";
                                let _ = bg_proxy.send_event(AppUserEvent::EvalScript(script.to_string()));
                            }

                            let script = format!(
                                "window.dispatchEvent(new CustomEvent('rift-live-data', {{ detail: {} }}));",
                                live_json
                            );
                            let _ = bg_proxy.send_event(AppUserEvent::EvalScript(script));
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
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(AppUserEvent::EvalScript(script)) => {
                let _ = webview.evaluate_script(&script);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });
}
