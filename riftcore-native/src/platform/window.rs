use crate::platform::process::utf16_z_to_string;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowBounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl WindowBounds {
    pub fn width(self) -> i32 {
        (self.right - self.left).max(0)
    }

    pub fn height(self) -> i32 {
        (self.bottom - self.top).max(0)
    }

    pub fn to_array(self) -> [i32; 4] {
        [self.left, self.top, self.right, self.bottom]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeagueWindowInfo {
    pub hwnd: isize,
    pub process_id: u32,
    pub bounds: [i32; 4],
    pub dpi: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonitorMetrics {
    pub rect_px: [i32; 4],
    pub work_px: [i32; 4],
    pub dpi: u32,
}

pub fn dpi_scale(dpi: u32) -> f32 {
    (dpi.max(96) as f32 / 96.0).max(1.0)
}

#[cfg(windows)]
pub fn is_valid_window(hwnd: isize) -> bool {
    use std::ffi::c_void;

    if hwnd == 0 {
        return false;
    }

    #[link(name = "user32")]
    extern "system" {
        fn IsWindow(hwnd: *mut c_void) -> i32;
    }

    // SAFETY: `IsWindow` safely validates any arbitrary non-zero `HWND` handle.
    unsafe { IsWindow(hwnd as *mut c_void) != 0 }
}

#[cfg(not(windows))]
pub fn is_valid_window(_hwnd: isize) -> bool {
    false
}

#[cfg(windows)]
pub fn window_dpi(hwnd: isize) -> u32 {
    use std::ffi::c_void;

    if hwnd == 0 {
        return 96;
    }

    #[link(name = "user32")]
    extern "system" {
        fn GetDpiForWindow(hwnd: *mut c_void) -> u32;
    }

    // SAFETY: `hwnd` is non-zero; `GetDpiForWindow` returns 0 on invalid HWND, clamped to >= 96.
    unsafe { GetDpiForWindow(hwnd as *mut c_void).max(96) }
}

#[cfg(not(windows))]
pub fn window_dpi(_hwnd: isize) -> u32 {
    96
}

#[cfg(windows)]
pub fn foreground_window_hwnd() -> isize {
    use std::ffi::c_void;

    #[link(name = "user32")]
    extern "system" {
        fn GetForegroundWindow() -> *mut c_void;
    }

    // SAFETY: `GetForegroundWindow` takes no pointer inputs and returns null or a valid HWND.
    unsafe { GetForegroundWindow() as isize }
}

#[cfg(not(windows))]
pub fn foreground_window_hwnd() -> isize {
    0
}

#[cfg(windows)]
pub fn cursor_screen_position() -> Option<[i32; 2]> {
    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[link(name = "user32")]
    extern "system" {
        fn GetCursorPos(point: *mut Point) -> i32;
    }

    let mut point = Point { x: 0, y: 0 };
    // SAFETY: `point` is a valid writable stack `Point` struct.
    (unsafe { GetCursorPos(&mut point) } != 0).then_some([point.x, point.y])
}

#[cfg(not(windows))]
pub fn cursor_screen_position() -> Option<[i32; 2]> {
    None
}

#[cfg(windows)]
pub fn window_from_screen_position(px: [i32; 2]) -> isize {
    use std::ffi::c_void;

    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[link(name = "user32")]
    extern "system" {
        fn WindowFromPoint(point: Point) -> *mut c_void;
    }

    // SAFETY: `WindowFromPoint` accepts `Point` by value and returns null or an HWND.
    unsafe { WindowFromPoint(Point { x: px[0], y: px[1] }) as isize }
}

#[cfg(not(windows))]
pub fn window_from_screen_position(_px: [i32; 2]) -> isize {
    0
}

#[cfg(windows)]
pub fn window_ex_style(hwnd: isize) -> usize {
    use std::ffi::c_void;

    if hwnd == 0 {
        return 0;
    }

    #[link(name = "user32")]
    extern "system" {
        fn GetWindowLongPtrW(hwnd: *mut c_void, index: i32) -> isize;
    }

    const GWL_EXSTYLE: i32 = -20;
    // SAFETY: `hwnd` is non-zero; `GetWindowLongPtrW` handles stale HWNDs safely by returning 0.
    unsafe { GetWindowLongPtrW(hwnd as *mut c_void, GWL_EXSTYLE) as usize }
}

#[cfg(not(windows))]
pub fn window_ex_style(_hwnd: isize) -> usize {
    0
}

/// Enumerates physical monitors and per-monitor DPI metrics.
#[cfg(windows)]
pub fn query_monitors() -> Vec<MonitorMetrics> {
    use std::ffi::c_void;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct WinRect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    struct MonitorInfo {
        cb_size: u32,
        rc_monitor: WinRect,
        rc_work: WinRect,
        dw_flags: u32,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn LoadLibraryA(name: *const u8) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, proc_name: *const u8) -> *mut c_void;
        fn FreeLibrary(lib_module: *mut c_void) -> i32;
    }

    #[link(name = "user32")]
    extern "system" {
        fn EnumDisplayMonitors(
            hdc: *mut c_void,
            clip: *const WinRect,
            enum_proc: unsafe extern "system" fn(
                *mut c_void,
                *mut c_void,
                *mut WinRect,
                isize,
            ) -> i32,
            data: isize,
        ) -> i32;
        fn GetMonitorInfoW(monitor: *mut c_void, info: *mut MonitorInfo) -> i32;
    }

    type GetDpiForMonitorFn =
        unsafe extern "system" fn(*mut c_void, u32, *mut u32, *mut u32) -> i32;

    // SAFETY: Invoked synchronously by `EnumDisplayMonitors`, which guarantees `hmon` is valid
    // and passes `data` pointing to the stack-pinned `(Vec<MonitorMetrics>, Option<GetDpiForMonitorFn>)`.
    unsafe extern "system" fn collect_monitor(
        hmon: *mut c_void,
        _hdc: *mut c_void,
        _rect: *mut WinRect,
        data: isize,
    ) -> i32 {
        let (out, get_dpi_fn) =
            &mut *(data as *mut (Vec<MonitorMetrics>, Option<GetDpiForMonitorFn>));
        let mut info = MonitorInfo {
            cb_size: std::mem::size_of::<MonitorInfo>() as u32,
            rc_monitor: WinRect {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            rc_work: WinRect {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            dw_flags: 0,
        };
        if GetMonitorInfoW(hmon, &mut info) != 0 {
            let mut dpi_x = 96u32;
            let mut dpi_y = 96u32;
            if let Some(func) = *get_dpi_fn {
                let _ = func(hmon, 0, &mut dpi_x, &mut dpi_y);
            }
            out.push(MonitorMetrics {
                rect_px: [
                    info.rc_monitor.left,
                    info.rc_monitor.top,
                    info.rc_monitor.right,
                    info.rc_monitor.bottom,
                ],
                work_px: [
                    info.rc_work.left,
                    info.rc_work.top,
                    info.rc_work.right,
                    info.rc_work.bottom,
                ],
                dpi: dpi_x.max(96),
            });
        }
        1
    }

    // SAFETY: `LoadLibraryA`/`GetProcAddress` use static NUL-terminated C strings.
    // `EnumDisplayMonitors` runs synchronously on the current thread before `payload` goes
    // out of scope or `FreeLibrary(shcore)` is called.
    unsafe {
        let shcore = LoadLibraryA(c"shcore.dll".as_ptr().cast::<u8>());
        let get_dpi_fn: Option<GetDpiForMonitorFn> = if !shcore.is_null() {
            let proc = GetProcAddress(shcore, c"GetDpiForMonitor".as_ptr().cast::<u8>());
            if proc.is_null() {
                None
            } else {
                Some(std::mem::transmute::<*mut c_void, GetDpiForMonitorFn>(proc))
            }
        } else {
            None
        };
        let mut payload = (Vec::<MonitorMetrics>::new(), get_dpi_fn);
        let _ = EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            collect_monitor,
            &mut payload as *mut _ as isize,
        );
        if !shcore.is_null() {
            let _ = FreeLibrary(shcore);
        }
        payload.0
    }
}

#[cfg(not(windows))]
pub fn query_monitors() -> Vec<MonitorMetrics> {
    Vec::new()
}

/// Discovers the active League of Legends game window with hardened process/title checks
/// to prevent arbitrary windows (e.g. browser tabs) from spoofing the overlay target.
#[cfg(windows)]
pub fn find_league_game_window() -> Option<LeagueWindowInfo> {
    use std::ffi::c_void;

    #[repr(C)]
    struct WinRect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[link(name = "user32")]
    extern "system" {
        fn EnumWindows(
            callback: unsafe extern "system" fn(*mut c_void, isize) -> i32,
            parameter: isize,
        ) -> i32;
    }

    struct SearchResult {
        window: Option<LeagueWindowInfo>,
        area: i64,
    }

    // SAFETY: Invoked synchronously by `EnumWindows`, passing valid `hwnd` handles and
    // `parameter` pointing to the live stack `SearchResult`.
    unsafe extern "system" fn find_game_window(hwnd: *mut c_void, parameter: isize) -> i32 {
        #[link(name = "user32")]
        extern "system" {
            fn GetWindowThreadProcessId(hwnd: *mut c_void, process_id: *mut u32) -> u32;
            fn GetClientRect(hwnd: *mut c_void, rect: *mut WinRect) -> i32;
            fn ClientToScreen(hwnd: *mut c_void, point: *mut Point) -> i32;
            fn GetDpiForWindow(hwnd: *mut c_void) -> u32;
            fn IsWindowVisible(hwnd: *mut c_void) -> i32;
            fn IsIconic(hwnd: *mut c_void) -> i32;
            fn GetClassNameW(hwnd: *mut c_void, class_name: *mut u16, max_count: i32) -> i32;
            fn GetWindowTextW(hwnd: *mut c_void, text: *mut u16, max_count: i32) -> i32;
        }
        #[link(name = "kernel32")]
        extern "system" {
            fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut c_void;
            fn QueryFullProcessImageNameW(
                process: *mut c_void,
                flags: u32,
                path: *mut u16,
                size: *mut u32,
            ) -> i32;
            fn CloseHandle(handle: *mut c_void) -> i32;
        }

        const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

        if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
            return 1;
        }

        let mut class_buf = [0u16; 128];
        let class_len = GetClassNameW(hwnd, class_buf.as_mut_ptr(), class_buf.len() as i32);
        let class_name = if class_len > 0 {
            utf16_z_to_string(&class_buf[..(class_len as usize).min(class_buf.len())])
        } else {
            String::new()
        };

        let mut title_buf = [0u16; 128];
        let title_len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), title_buf.len() as i32);
        let title = if title_len > 0 {
            utf16_z_to_string(&title_buf[..(title_len as usize).min(title_buf.len())])
        } else {
            String::new()
        };

        let looks_like_riot_class = class_name.eq_ignore_ascii_case("RiotWindowClass");
        let looks_like_league_title = title.eq_ignore_ascii_case("League of Legends (TM) Client");
        if !looks_like_riot_class && !looks_like_league_title {
            return 1;
        }

        let mut pid = 0u32;
        let _ = GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return 1;
        }

        // Verify executable path whenever accessible; only allow fallback when Vanguard blocks OpenProcess.
        let mut verified_league = false;
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if !process.is_null() {
            let mut buffer = [0u16; 1024];
            let mut size = buffer.len() as u32;
            if QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut size) != 0 {
                if let Some(slice) = buffer.get(..size as usize) {
                    let image_path = String::from_utf16_lossy(slice);
                    verified_league = Path::new(&image_path)
                        .file_name()
                        .is_some_and(|name| name.eq_ignore_ascii_case("League of Legends.exe"));
                }
            }
            let _ = CloseHandle(process);
            if !verified_league {
                return 1;
            }
        } else if !(looks_like_riot_class && looks_like_league_title) {
            return 1;
        }

        let mut client = WinRect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if GetClientRect(hwnd, &mut client) == 0 {
            return 1;
        }
        let width = client.right - client.left;
        let height = client.bottom - client.top;
        if width < 600 || height < 400 {
            return 1;
        }
        let mut origin = Point {
            x: client.left,
            y: client.top,
        };
        if ClientToScreen(hwnd, &mut origin) == 0 {
            return 1;
        }

        let area = i64::from(width) * i64::from(height);
        let result = &mut *(parameter as *mut SearchResult);
        if area > result.area {
            result.area = area;
            result.window = Some(LeagueWindowInfo {
                hwnd: hwnd as isize,
                process_id: pid,
                bounds: [origin.x, origin.y, origin.x + width, origin.y + height],
                dpi: GetDpiForWindow(hwnd).max(96),
            });
        }
        1
    }

    let mut result = SearchResult {
        window: None,
        area: 0,
    };
    // SAFETY: `EnumWindows` completes synchronously before `result` drops.
    unsafe {
        let _ = EnumWindows(find_game_window, &mut result as *mut SearchResult as isize);
    }
    result.window
}

#[cfg(not(windows))]
pub fn find_league_game_window() -> Option<LeagueWindowInfo> {
    None
}

/// Checks if the discovered League game window is currently the foreground window.
#[cfg(windows)]
pub fn is_league_window_foreground(window: LeagueWindowInfo) -> bool {
    use std::ffi::c_void;

    #[link(name = "user32")]
    extern "system" {
        fn GetForegroundWindow() -> *mut c_void;
        fn GetWindowThreadProcessId(hwnd: *mut c_void, process_id: *mut u32) -> u32;
        fn GetClassNameW(hwnd: *mut c_void, class_name: *mut u16, max_count: i32) -> i32;
        fn GetWindowTextW(hwnd: *mut c_void, text: *mut u16, max_count: i32) -> i32;
    }

    // SAFETY: `GetForegroundWindow` returns null or a valid HWND; stack buffers bound string lengths.
    unsafe {
        let foreground = GetForegroundWindow();
        if foreground.is_null() {
            return false;
        }
        if (foreground as isize) == window.hwnd {
            return true;
        }
        let mut foreground_pid = 0u32;
        let _ = GetWindowThreadProcessId(foreground, &mut foreground_pid);
        if window.process_id != 0 && foreground_pid == window.process_id {
            return true;
        }
        let mut class_buf = [0u16; 64];
        let class_len = GetClassNameW(foreground, class_buf.as_mut_ptr(), class_buf.len() as i32);
        let class_match = class_len > 0
            && utf16_z_to_string(&class_buf[..(class_len as usize).min(class_buf.len())])
                .eq_ignore_ascii_case("RiotWindowClass");
        if !class_match {
            return false;
        }
        let mut title_buf = [0u16; 128];
        let title_len = GetWindowTextW(foreground, title_buf.as_mut_ptr(), title_buf.len() as i32);
        title_len > 0
            && utf16_z_to_string(&title_buf[..(title_len as usize).min(title_buf.len())])
                .eq_ignore_ascii_case("League of Legends (TM) Client")
    }
}

#[cfg(not(windows))]
pub fn is_league_window_foreground(_window: LeagueWindowInfo) -> bool {
    false
}
