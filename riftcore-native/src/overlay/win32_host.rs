//! Path B Win32 + DirectComposition (`DxgiFromVisual`) overlay surface host.
//!
//! Creates a dedicated `WS_EX_NOREDIRECTIONBITMAP | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`
//! Win32 popup window, attaches a `wgpu::Surface` backed by DX12 `DxgiFromVisual` with
//! `PreMultiplied` alpha compositing (eliminating `LWA_COLORKEY` black-pixel punch-out), and renders
//! an isolated `egui::Context` (`zoom_factor = 1.0`) via `egui_wgpu::Renderer`.

#[cfg(windows)]
mod imp {
    use crate::overlay::host::{
        OverlayDiagnostics, OverlayHostBackend, OverlayRenderState, OverlaySurfaceHost,
    };
    use crate::overlay::hud_modes::cursor_hits_rects;
    use crate::platform::overlay_window::{
        WS_EX_APPWINDOW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP,
        WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
    };
    use crate::platform::window::{cursor_screen_position, is_window_above, window_ex_style};
    use eframe::egui::{self, Pos2, Rect, Vec2};
    use eframe::egui_wgpu::{
        self, Renderer as EguiWgpuRenderer, RendererOptions, ScreenDescriptor,
    };
    use eframe::wgpu;
    use std::cell::RefCell;
    use std::ffi::c_void;
    use std::num::NonZeroIsize;
    use std::rc::Rc;
    use std::time::{Duration, Instant};

    const GWL_EXSTYLE: i32 = -20;
    const GWLP_USERDATA: i32 = -21;

    const WS_POPUP: u32 = 0x8000_0000;

    const SW_HIDE: i32 = 0;
    const SW_SHOWNOACTIVATE: i32 = 4;

    const HWND_TOPMOST: isize = -1;
    const SWP_NOSIZE: u32 = 0x0001;
    const SWP_NOMOVE: u32 = 0x0002;
    const SWP_NOACTIVATE: u32 = 0x0010;
    const SWP_NOOWNERZORDER: u32 = 0x0200;

    const WM_DESTROY: u32 = 0x0002;
    const WM_SIZE: u32 = 0x0005;
    const WM_DISPLAYCHANGE: u32 = 0x007E;
    const WM_SETCURSOR: u32 = 0x0020;
    const WM_NCHITTEST: u32 = 0x0084;
    const WM_TIMER: u32 = 0x0113;
    const WM_MOUSEACTIVATE: u32 = 0x0021;
    const WM_STYLECHANGING: u32 = 0x007C;
    const WM_MOUSEMOVE: u32 = 0x0200;
    const WM_LBUTTONDOWN: u32 = 0x0201;
    const WM_LBUTTONUP: u32 = 0x0202;
    const WM_RBUTTONDOWN: u32 = 0x0204;
    const WM_RBUTTONUP: u32 = 0x0205;
    const WM_MOUSEWHEEL: u32 = 0x020A;
    const WM_CAPTURECHANGED: u32 = 0x0215;
    const WM_MOUSELEAVE: u32 = 0x02A3;
    const WM_DPICHANGED: u32 = 0x02E0;

    const HTTRANSPARENT: isize = -1;
    const HIT_TIMER_ID: usize = 0x5243;
    const HIT_TIMER_MS: u32 = 30;
    const LWA_ALPHA: u32 = 0x2;
    const MA_NOACTIVATE: isize = 3;
    const IDC_ARROW: usize = 32512;
    const TME_LEAVE: u32 = 0x0000_0002;

    #[repr(C)]
    struct StyleStruct {
        style_old: u32,
        style_new: u32,
    }

    #[repr(C)]
    struct TrackMouseEventStruct {
        cb_size: u32,
        dw_flags: u32,
        hwnd_track: *mut c_void,
        dw_hover_time: u32,
    }

    type WndProc = unsafe extern "system" fn(*mut c_void, u32, usize, isize) -> isize;

    #[repr(C)]
    struct WndClassExW {
        cb_size: u32,
        style: u32,
        lpfn_wnd_proc: WndProc,
        cb_cls_extra: i32,
        cb_wnd_extra: i32,
        h_instance: *mut c_void,
        h_icon: *mut c_void,
        h_cursor: *mut c_void,
        hbr_background: *mut c_void,
        lpsz_menu_name: *const u16,
        lpsz_class_name: *const u16,
        h_icon_sm: *mut c_void,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(module_name: *const u16) -> *mut c_void;
    }

    #[link(name = "user32")]
    extern "system" {
        fn RegisterClassExW(wc: *const WndClassExW) -> u16;
        fn CreateWindowExW(
            ex_style: u32,
            class_name: *const u16,
            window_name: *const u16,
            style: u32,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            parent: *mut c_void,
            menu: *mut c_void,
            instance: *mut c_void,
            param: *mut c_void,
        ) -> *mut c_void;
        fn DestroyWindow(hwnd: *mut c_void) -> i32;
        fn DefWindowProcW(hwnd: *mut c_void, msg: u32, wparam: usize, lparam: isize) -> isize;
        fn ShowWindow(hwnd: *mut c_void, cmd_show: i32) -> i32;
        fn SetWindowPos(
            hwnd: *mut c_void,
            insert_after: *mut c_void,
            x: i32,
            y: i32,
            cx: i32,
            cy: i32,
            flags: u32,
        ) -> i32;
        fn GetWindowLongPtrW(hwnd: *mut c_void, index: i32) -> isize;
        fn SetWindowLongPtrW(hwnd: *mut c_void, index: i32, new_long: isize) -> isize;
        fn LoadCursorW(instance: *mut c_void, cursor_name: *const u16) -> *mut c_void;
        fn SetCursor(hcursor: *mut c_void) -> *mut c_void;
        fn TrackMouseEvent(tme: *mut TrackMouseEventStruct) -> i32;
        fn GetCapture() -> *mut c_void;
        fn SetCapture(hwnd: *mut c_void) -> *mut c_void;
        fn ReleaseCapture() -> i32;
        fn SetTimer(hwnd: *mut c_void, id: usize, elapse_ms: u32, func: *const c_void) -> usize;
        fn KillTimer(hwnd: *mut c_void, id: usize) -> i32;
        fn SetLayeredWindowAttributes(hwnd: *mut c_void, key: u32, alpha: u8, flags: u32) -> i32;
    }

    #[derive(Default)]
    struct OverlayInputShared {
        events: Vec<egui::Event>,
        pixels_per_point: f32,
        pointer_down: bool,
        tracking_leave: bool,
        pending_resize: Option<[u32; 2]>,
        /// Interactive panel rects (logical, window-local) from the last rendered frame.
        hit_rects: Vec<Rect>,
        origin_px: [i32; 2],
        force_passthrough: bool,
        /// Whether `WS_EX_TRANSPARENT` (click-through) is currently applied.
        passthrough: bool,
        repaint: Option<egui::Context>,
        /// A display mode or DPI change arrived (e.g. a Fullscreen game switched resolution),
        /// so cached monitor metrics are stale.
        display_changed: bool,
        /// Game window the HUD must stay above while it is on screen (0 = none); re-checked on
        /// every hit-test timer tick, so the HUD is back on top within ~30 ms of the game rising.
        game_hwnd: isize,
        topmost_reasserts: u64,
    }

    /// Toggles only `WS_EX_TRANSPARENT`; `WS_EX_LAYERED` stays on (with `LWA_ALPHA` 255) so the
    /// change needs no z-order/frame update and is safe from inside the window procedure.
    unsafe fn set_click_through(hwnd: *mut c_void, passthrough: bool) {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as usize;
        let next = (if passthrough {
            ex | WS_EX_TRANSPARENT
        } else {
            ex & !WS_EX_TRANSPARENT
        }) | WS_EX_LAYERED;
        if next != ex {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next as isize);
        }
    }

    /// Re-asserts topmost only when `game_hwnd` is above the HUD (a Fullscreen game is itself
    /// topmost and rises whenever it is activated). No move, no resize and `SWP_NOACTIVATE`, so
    /// the game keeps focus and its Fullscreen mode. Callers must not hold a `RefCell` borrow of
    /// the shared state: `SetWindowPos` sends messages back to the window procedure.
    unsafe fn raise_above_game(hwnd: *mut c_void, game_hwnd: isize) -> bool {
        if hwnd.is_null() || game_hwnd == 0 || !is_window_above(game_hwnd, hwnd as isize) {
            return false;
        }
        SetWindowPos(
            hwnd,
            HWND_TOPMOST as *mut c_void,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
        ) != 0
    }

    /// Hit-tests the live cursor against the last frame's panel rects and flips click-through
    /// immediately (no wait for the app's next tick). Returns `true` when over a panel.
    unsafe fn refresh_hit_test(
        hwnd: *mut c_void,
        shared: *const RefCell<OverlayInputShared>,
    ) -> bool {
        let Some(cursor) = cursor_screen_position() else {
            return true;
        };
        if shared.is_null() {
            return true;
        }
        let (inside, wake) = {
            let Ok(mut st) = (*shared).try_borrow_mut() else {
                return true;
            };
            let inside = !st.force_passthrough
                && (st.pointer_down
                    || cursor_hits_rects(&st.hit_rects, st.origin_px, st.pixels_per_point, cursor));
            if st.passthrough == !inside {
                return inside;
            }
            st.passthrough = !inside;
            (inside, st.repaint.clone())
        };
        set_click_through(hwnd, !inside);
        if let Some(ctx) = wake {
            ctx.request_repaint();
        }
        inside
    }

    /// Polls a wgpu error-scope future once; native wgpu resolves it immediately.
    fn poll_ready<F: std::future::Future>(fut: F) -> Option<F::Output> {
        let mut fut = std::pin::pin!(fut);
        let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
        match std::future::Future::poll(fut.as_mut(), &mut cx) {
            std::task::Poll::Ready(out) => Some(out),
            std::task::Poll::Pending => None,
        }
    }

    // SAFETY: Win32 guarantees `hwnd` is valid during synchronous message dispatch on the
    // UI thread. `GWLP_USERDATA` is non-null only between `Win32OverlayHost::new` and clearing
    // it to `0` prior to `DestroyWindow` / `Rc::from_raw` in `close()`.
    unsafe extern "system" fn overlay_wnd_proc(
        hwnd: *mut c_void,
        msg: u32,
        wparam: usize,
        lparam: isize,
    ) -> isize {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const RefCell<OverlayInputShared>;
        match msg {
            WM_MOUSEACTIVATE => return MA_NOACTIVATE,
            WM_STYLECHANGING => {
                if (wparam as i32) == GWL_EXSTYLE && lparam != 0 {
                    // SAFETY: Win32 passes a non-null aligned `STYLESTRUCT` pointer in `lparam`.
                    let ss = &mut *(lparam as *mut StyleStruct);
                    ss.style_new |= (WS_EX_NOACTIVATE
                        | WS_EX_TOOLWINDOW
                        | WS_EX_TOPMOST
                        | WS_EX_NOREDIRECTIONBITMAP
                        | WS_EX_LAYERED) as u32;
                    ss.style_new &= !(WS_EX_APPWINDOW as u32);
                }
                return 0;
            }
            WM_TIMER if wparam == HIT_TIMER_ID => {
                let _ = refresh_hit_test(hwnd, ptr);
                if !ptr.is_null() {
                    // SAFETY: non-null `GWLP_USERDATA` points to the live shared state; each
                    // borrow ends before `raise_above_game` sends messages back to this proc.
                    let shared = &*ptr;
                    let game = shared.try_borrow().map_or(0, |st| st.game_hwnd);
                    if raise_above_game(hwnd, game) {
                        if let Ok(mut st) = shared.try_borrow_mut() {
                            st.topmost_reasserts += 1;
                        }
                    }
                }
                return 0;
            }
            WM_NCHITTEST if !refresh_hit_test(hwnd, ptr) => return HTTRANSPARENT,
            WM_SETCURSOR => {
                // Arrow only over RiftCore panels; elsewhere keep the game's own cursor.
                if refresh_hit_test(hwnd, ptr) {
                    let _ = SetCursor(LoadCursorW(std::ptr::null_mut(), IDC_ARROW as *const u16));
                }
                return 1;
            }
            _ => {}
        }

        let mut wake = None;
        if !ptr.is_null() {
            // SAFETY: Non-null `GWLP_USERDATA` points to the live `Rc<RefCell<OverlayInputShared>>`.
            let shared = &*ptr;
            if let Ok(mut st) = shared.try_borrow_mut() {
                let ppp = st.pixels_per_point.max(0.5);
                let lparam_pos = |lp: isize| -> Pos2 {
                    let x = (lp as i32 & 0xFFFF) as i16 as f32;
                    let y = ((lp as i32 >> 16) & 0xFFFF) as i16 as f32;
                    Pos2::new(x / ppp, y / ppp)
                };
                match msg {
                    WM_MOUSEMOVE => {
                        if !st.tracking_leave {
                            let mut tme = TrackMouseEventStruct {
                                cb_size: std::mem::size_of::<TrackMouseEventStruct>() as u32,
                                dw_flags: TME_LEAVE,
                                hwnd_track: hwnd,
                                dw_hover_time: 0,
                            };
                            if TrackMouseEvent(&mut tme) != 0 {
                                st.tracking_leave = true;
                            }
                        }
                        st.events
                            .push(egui::Event::PointerMoved(lparam_pos(lparam)));
                    }
                    WM_MOUSELEAVE => {
                        st.tracking_leave = false;
                        st.events.push(egui::Event::PointerGone);
                    }
                    WM_LBUTTONDOWN => {
                        st.pointer_down = true;
                        let _ = SetCapture(hwnd);
                        st.events.push(egui::Event::PointerButton {
                            pos: lparam_pos(lparam),
                            button: egui::PointerButton::Primary,
                            pressed: true,
                            modifiers: egui::Modifiers::NONE,
                        });
                    }
                    WM_LBUTTONUP => {
                        st.pointer_down = false;
                        if GetCapture() == hwnd {
                            let _ = ReleaseCapture();
                        }
                        st.events.push(egui::Event::PointerButton {
                            pos: lparam_pos(lparam),
                            button: egui::PointerButton::Primary,
                            pressed: false,
                            modifiers: egui::Modifiers::NONE,
                        });
                    }
                    WM_CAPTURECHANGED => {
                        st.pointer_down = false;
                    }
                    WM_RBUTTONDOWN => {
                        st.events.push(egui::Event::PointerButton {
                            pos: lparam_pos(lparam),
                            button: egui::PointerButton::Secondary,
                            pressed: true,
                            modifiers: egui::Modifiers::NONE,
                        });
                    }
                    WM_RBUTTONUP => {
                        st.events.push(egui::Event::PointerButton {
                            pos: lparam_pos(lparam),
                            button: egui::PointerButton::Secondary,
                            pressed: false,
                            modifiers: egui::Modifiers::NONE,
                        });
                    }
                    WM_MOUSEWHEEL => {
                        let delta_y = ((wparam >> 16) as u16 as i16) as f32 / 120.0;
                        st.events.push(egui::Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Line,
                            delta: Vec2::new(0.0, delta_y * 2.0),
                            phase: egui::TouchPhase::Move,
                            source: egui::MouseWheelSource::Wheel,
                            modifiers: egui::Modifiers::NONE,
                        });
                    }
                    WM_SIZE => {
                        let w = (lparam as u32) & 0xFFFF;
                        let h = ((lparam as u32) >> 16) & 0xFFFF;
                        if w > 0 && h > 0 {
                            st.pending_resize = Some([w, h]);
                        }
                    }
                    // The HUD scale comes from the engine (monitor DPI capped to the game size)
                    // on its next tick; only flag that monitor metrics must be re-read.
                    WM_DPICHANGED | WM_DISPLAYCHANGE => {
                        st.display_changed = true;
                    }
                    WM_DESTROY => {
                        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    }
                    _ => {}
                }
                wake = st.repaint.clone().filter(|_| !st.events.is_empty());
            }
        }
        if let Some(ctx) = wake {
            ctx.request_repaint();
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    /// Dedicated DirectComposition (`DxgiFromVisual`) transparent Win32 overlay surface host.
    pub struct Win32OverlayHost {
        hwnd: *mut c_void,
        render_state: egui_wgpu::RenderState,
        surface: Option<wgpu::Surface<'static>>,
        alpha_mode: wgpu::CompositeAlphaMode,
        supported_alpha_modes: Vec<wgpu::CompositeAlphaMode>,
        egui_ctx: egui::Context,
        renderer: EguiWgpuRenderer,
        pending_textures_delta: egui::TexturesDelta,
        shared_input: Rc<RefCell<OverlayInputShared>>,
        shared_input_raw: *const RefCell<OverlayInputShared>,
        /// Logically shown; the window itself appears once `show_pending` is cleared.
        visible: bool,
        /// Shown but not yet on screen: the first frame at the new size/DPI is presented
        /// before `ShowWindow`, so a stale or empty buffer never flashes up.
        show_pending: bool,
        rect_px: [i32; 4],
        configured_size: [u32; 2],
        reconfigure_pending: bool,
        last_issue_log: Option<Instant>,
        start_time: Instant,
        frames_rendered: u64,
        resize_events: u64,
        last_surface_status: String,
    }

    impl Win32OverlayHost {
        /// Creates the off-focus Win32 DirectComposition host window and attaches a `wgpu::Surface`.
        pub fn new(render_state: egui_wgpu::RenderState) -> Result<Self, String> {
            let class_name: Vec<u16> = "RiftCoreOverlayWindowClass\0".encode_utf16().collect();
            let window_title: Vec<u16> = "RiftCore In-Game HUD\0".encode_utf16().collect();

            let shared_input = Rc::new(RefCell::new(OverlayInputShared {
                pixels_per_point: 1.0,
                passthrough: true,
                ..Default::default()
            }));
            let shared_input_raw = Rc::into_raw(Rc::clone(&shared_input));

            // SAFETY: `class_name` and `window_title` are NUL-terminated UTF-16 slices live for
            // this call. Failure reclaims `shared_input_raw` immediately via `Rc::from_raw`;
            // success transfers that raw alias to `GWLP_USERDATA` until `close()`.
            let hwnd = unsafe {
                let hinstance = GetModuleHandleW(std::ptr::null());
                let wc = WndClassExW {
                    cb_size: std::mem::size_of::<WndClassExW>() as u32,
                    style: 0,
                    lpfn_wnd_proc: overlay_wnd_proc,
                    cb_cls_extra: 0,
                    cb_wnd_extra: 0,
                    h_instance: hinstance,
                    h_icon: std::ptr::null_mut(),
                    // No class cursor: WM_SETCURSOR sets the arrow only over panels.
                    h_cursor: std::ptr::null_mut(),
                    hbr_background: std::ptr::null_mut(),
                    lpsz_menu_name: std::ptr::null(),
                    lpsz_class_name: class_name.as_ptr(),
                    h_icon_sm: std::ptr::null_mut(),
                };
                let _ = RegisterClassExW(&wc);

                let ex_style = (WS_EX_NOREDIRECTIONBITMAP
                    | WS_EX_TOPMOST
                    | WS_EX_TOOLWINDOW
                    | WS_EX_NOACTIVATE
                    | WS_EX_TRANSPARENT
                    | WS_EX_LAYERED) as u32;
                let hwnd = CreateWindowExW(
                    ex_style,
                    class_name.as_ptr(),
                    window_title.as_ptr(),
                    WS_POPUP,
                    0,
                    0,
                    640,
                    320,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    hinstance,
                    std::ptr::null_mut(),
                );
                if hwnd.is_null() {
                    let _ = Rc::from_raw(shared_input_raw);
                    return Err("CreateWindowExW failed for Win32OverlayHost".to_string());
                }
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, shared_input_raw as isize);
                // A layered window must have its attributes set; opaque `LWA_ALPHA` leaves the
                // DComp content untouched, and `WS_EX_LAYERED` + `WS_EX_TRANSPARENT` is what
                // makes the window click-through across processes.
                let _ = SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA);
                hwnd
            };

            let (surface, alpha_mode, supported_alpha_modes) =
                match Self::create_surface_for_hwnd(&render_state, hwnd, [640, 320]) {
                    Ok(ok) => ok,
                    Err(err) => {
                        // SAFETY: Surface creation failed before `Self` was constructed; reclaim
                        // `GWLP_USERDATA`, destroy `hwnd`, and balance `Rc::into_raw` cleanly.
                        unsafe {
                            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                            DestroyWindow(hwnd);
                            let _ = Rc::from_raw(shared_input_raw);
                        }
                        return Err(err);
                    }
                };

            let egui_ctx = egui::Context::default();
            // Keep overlay zoom isolated at 1.0 regardless of main companion DPI zoom.
            egui_ctx.set_zoom_factor(1.0);
            let renderer = EguiWgpuRenderer::new(
                &render_state.device,
                render_state.target_format,
                RendererOptions::default(),
            );

            Ok(Self {
                hwnd,
                render_state,
                surface: Some(surface),
                alpha_mode,
                supported_alpha_modes,
                egui_ctx,
                renderer,
                pending_textures_delta: egui::TexturesDelta::default(),
                shared_input,
                shared_input_raw,
                visible: false,
                show_pending: false,
                rect_px: [0, 0, 640, 320],
                configured_size: [640, 320],
                reconfigure_pending: false,
                last_issue_log: None,
                start_time: Instant::now(),
                frames_rendered: 0,
                resize_events: 0,
                last_surface_status: "initialized".to_string(),
            })
        }

        /// Returns the underlying Win32 `HWND` (`0` after [`OverlaySurfaceHost::close`]).
        pub fn hwnd(&self) -> isize {
            self.hwnd as isize
        }

        fn create_surface_for_hwnd(
            render_state: &egui_wgpu::RenderState,
            hwnd: *mut c_void,
            size_px: [u32; 2],
        ) -> Result<
            (
                wgpu::Surface<'static>,
                wgpu::CompositeAlphaMode,
                Vec<wgpu::CompositeAlphaMode>,
            ),
            String,
        > {
            let hwnd_nz =
                NonZeroIsize::new(hwnd as isize).ok_or("null HWND in create_surface_for_hwnd")?;
            let mut win_handle = wgpu::rwh::Win32WindowHandle::new(hwnd_nz);
            // SAFETY: `GetModuleHandleW(null)` returns the current executable module handle.
            let hinst = unsafe { GetModuleHandleW(std::ptr::null()) as isize };
            win_handle.hinstance = NonZeroIsize::new(hinst);
            let disp_handle = wgpu::rwh::WindowsDisplayHandle::new();

            // SAFETY: `hwnd_nz` is a live Win32 window created in `Win32OverlayHost::new`, and
            // `close()` guarantees `self.surface` is dropped before `DestroyWindow`.
            let surface = unsafe {
                render_state
                    .instance
                    .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                        raw_display_handle: Some(wgpu::rwh::RawDisplayHandle::Windows(disp_handle)),
                        raw_window_handle: wgpu::rwh::RawWindowHandle::Win32(win_handle),
                    })
            }
            .map_err(|e| format!("create_surface_unsafe failed: {e}"))?;

            let caps = surface.get_capabilities(&render_state.adapter);
            let alpha_mode = if caps
                .alpha_modes
                .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
            {
                wgpu::CompositeAlphaMode::PreMultiplied
            } else if caps
                .alpha_modes
                .contains(&wgpu::CompositeAlphaMode::PostMultiplied)
            {
                wgpu::CompositeAlphaMode::PostMultiplied
            } else {
                wgpu::CompositeAlphaMode::Auto
            };

            Self::configure_surface(
                &surface,
                render_state,
                alpha_mode,
                size_px[0].max(1),
                size_px[1].max(1),
            )?;
            Ok((surface, alpha_mode, caps.alpha_modes))
        }

        fn configure_surface(
            surface: &wgpu::Surface<'static>,
            render_state: &egui_wgpu::RenderState,
            alpha_mode: wgpu::CompositeAlphaMode,
            width: u32,
            height: u32,
        ) -> Result<(), String> {
            let mut cfg = surface
                .get_default_config(&render_state.adapter, width.max(1), height.max(1))
                .ok_or_else(|| "Surface unsupported by DX12 adapter".to_string())?;
            cfg.format = render_state.target_format;
            cfg.present_mode = wgpu::PresentMode::AutoVsync;
            cfg.alpha_mode = alpha_mode;
            cfg.view_formats = vec![render_state.target_format];
            cfg.desired_maximum_frame_latency = 1;
            let scope = render_state
                .device
                .push_error_scope(wgpu::ErrorFilter::Validation);
            surface.configure(&render_state.device, &cfg);
            match poll_ready(scope.pop()).flatten() {
                Some(err) => Err(format!("surface configure failed: {err}")),
                None => Ok(()),
            }
        }

        /// Click-through forced by the user (ALT+SHIFT+~): panels never become interactive.
        pub fn set_force_passthrough(&mut self, force: bool) {
            self.shared_input.borrow_mut().force_passthrough = force;
        }

        /// Context woken when the cursor reaches a panel or HUD input arrives.
        pub fn set_repaint_context(&mut self, ctx: egui::Context) {
            self.shared_input.borrow_mut().repaint = Some(ctx);
        }

        pub fn has_repaint_context(&self) -> bool {
            self.shared_input.borrow().repaint.is_some()
        }

        /// True once after the HUD window saw a display mode or DPI change (cached monitor
        /// metrics are stale; re-query them before placing the HUD).
        pub fn take_display_changed(&mut self) -> bool {
            std::mem::take(&mut self.shared_input.borrow_mut().display_changed)
        }

        /// Puts the (already sized and rendered) window on screen without activating it,
        /// starts hit-testing, and raises it to the top of the topmost band.
        fn show_now(&mut self) {
            self.show_pending = false;
            if self.hwnd.is_null() {
                return;
            }
            // SAFETY: `self.hwnd` is verified non-null and owned by this thread;
            // `SW_SHOWNOACTIVATE` never activates the window or takes focus from the game.
            unsafe {
                ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
                // The hit-test timer flips click-through as the cursor enters a panel,
                // independent of how often the app ticks.
                let _ = SetTimer(self.hwnd, HIT_TIMER_ID, HIT_TIMER_MS, std::ptr::null());
            }
            let passthrough = self.shared_input.borrow().passthrough;
            self.apply_ex_style(passthrough);
        }

        /// Records a non-success surface status and logs it at most every 10 s.
        fn note_surface_issue(&mut self, status: String) {
            if self
                .last_issue_log
                .is_none_or(|at| at.elapsed() >= Duration::from_secs(10))
            {
                eprintln!("[riftcore hud] overlay surface: {status}");
                self.last_issue_log = Some(Instant::now());
            }
            self.last_surface_status = status;
        }

        fn apply_ex_style(&self, passthrough: bool) {
            if self.hwnd.is_null() {
                return;
            }
            if let Ok(mut st) = self.shared_input.try_borrow_mut() {
                st.passthrough = passthrough;
                if passthrough {
                    st.pointer_down = false;
                }
            }
            // SAFETY: `self.hwnd` is verified non-null and owned by this UI thread; style updates
            // preserve no-activate/toolwindow flags and release active capture when going passthrough.
            unsafe {
                if passthrough && GetCapture() == self.hwnd {
                    let _ = ReleaseCapture();
                }
                set_click_through(self.hwnd, passthrough);
                SetWindowPos(
                    self.hwnd,
                    HWND_TOPMOST as *mut c_void,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }
        }
    }

    impl OverlaySurfaceHost for Win32OverlayHost {
        fn backend_kind(&self) -> OverlayHostBackend {
            OverlayHostBackend::PathBWin32DComp
        }

        fn pointer_down(&self) -> bool {
            self.shared_input.borrow().pointer_down
        }

        fn sync_window(
            &mut self,
            visible: bool,
            rect_px: [i32; 4],
            pixels_per_point: f32,
            passthrough: bool,
        ) {
            if self.hwnd.is_null() {
                self.visible = false;
                return;
            }
            // A show still pending from the last tick had no frame rendered in between;
            // it is put up at the end of this call rather than kept hidden.
            let stale_show = self.show_pending;
            let size_px = [
                (rect_px[2] - rect_px[0]).max(1) as u32,
                (rect_px[3] - rect_px[1]).max(1) as u32,
            ];
            {
                let mut st = self.shared_input.borrow_mut();
                st.pixels_per_point = if pixels_per_point.is_finite() {
                    pixels_per_point.max(1.0)
                } else {
                    1.0
                };
                st.origin_px = [rect_px[0], rect_px[1]];
            }

            if self.rect_px != rect_px {
                self.rect_px = rect_px;
                // SAFETY: `self.hwnd` is verified non-null and owned by this thread.
                unsafe {
                    SetWindowPos(
                        self.hwnd,
                        HWND_TOPMOST as *mut c_void,
                        rect_px[0],
                        rect_px[1],
                        size_px[0] as i32,
                        size_px[1] as i32,
                        SWP_NOACTIVATE,
                    );
                }
            }

            if self.configured_size != size_px {
                if let Some(surface) = &self.surface {
                    if Self::configure_surface(
                        surface,
                        &self.render_state,
                        self.alpha_mode,
                        size_px[0],
                        size_px[1],
                    )
                    .is_ok()
                    {
                        self.configured_size = size_px;
                        self.resize_events += 1;
                    }
                }
            }

            if self.shared_input.borrow().passthrough != passthrough {
                self.apply_ex_style(passthrough);
            }

            if self.visible != visible {
                self.visible = visible;
                if visible {
                    // Shown by `render_if_needed` once the first frame at this size is presented.
                    self.show_pending = true;
                } else {
                    self.show_pending = false;
                    {
                        let mut st = self.shared_input.borrow_mut();
                        st.pointer_down = false;
                        st.game_hwnd = 0;
                    }
                    // SAFETY: `self.hwnd` is verified non-null; clear mouse capture before
                    // hiding. `SW_HIDE` never activates another window of ours.
                    unsafe {
                        if GetCapture() == self.hwnd {
                            let _ = ReleaseCapture();
                        }
                        ShowWindow(self.hwnd, SW_HIDE);
                        let _ = KillTimer(self.hwnd, HIT_TIMER_ID);
                    }
                }
            }
            if stale_show && self.show_pending {
                self.show_now();
            }
        }

        fn keep_above(&mut self, game_hwnd: isize) -> bool {
            let on_screen = !self.hwnd.is_null() && self.visible && !self.show_pending;
            // The HUD's 30 ms hit-test timer keeps re-checking this window between app ticks.
            self.shared_input.borrow_mut().game_hwnd = if on_screen { game_hwnd } else { 0 };
            if !on_screen {
                return false;
            }
            // SAFETY: `self.hwnd` is verified non-null and owned by this thread, and no borrow
            // of the shared state is held across the call.
            let raised = unsafe { raise_above_game(self.hwnd, game_hwnd) };
            if raised {
                self.shared_input.borrow_mut().topmost_reasserts += 1;
            }
            raised
        }

        fn render_if_needed(
            &mut self,
            force_redraw: bool,
            state: &mut OverlayRenderState,
            render_ui: &mut dyn FnMut(&mut egui::Ui, &mut OverlayRenderState, &mut Vec<Rect>),
        ) -> Vec<Rect> {
            if self.hwnd.is_null() || !self.visible || !force_redraw {
                return Vec::new();
            }

            let pending_resize = self.shared_input.borrow_mut().pending_resize.take();
            if let Some(sz) = pending_resize {
                if sz != self.configured_size {
                    if let Some(surface) = &self.surface {
                        if Self::configure_surface(
                            surface,
                            &self.render_state,
                            self.alpha_mode,
                            sz[0],
                            sz[1],
                        )
                        .is_ok()
                        {
                            self.configured_size = sz;
                            self.resize_events += 1;
                        }
                    }
                }
            }

            let (events, ppp) = {
                let mut st = self.shared_input.borrow_mut();
                (std::mem::take(&mut st.events), st.pixels_per_point.max(1.0))
            };

            let [w_px, h_px] = self.configured_size;
            let screen_size_pts = Vec2::new(w_px as f32 / ppp, h_px as f32 / ppp);
            let mut raw_input = egui::RawInput {
                viewport_id: egui::ViewportId::ROOT,
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, screen_size_pts)),
                time: Some(self.start_time.elapsed().as_secs_f64()),
                events,
                focused: false,
                ..Default::default()
            };
            if let Some(vp) = raw_input.viewports.get_mut(&egui::ViewportId::ROOT) {
                vp.native_pixels_per_point = Some(ppp);
            }

            state.host_diag = self.diagnostics();
            let mut rects = Vec::new();
            let full_output = self.egui_ctx.run_ui(raw_input, |ui| {
                render_ui(ui, state, &mut rects);
            });
            self.shared_input.borrow_mut().hit_rects.clone_from(&rects);
            self.pending_textures_delta
                .append(full_output.textures_delta);

            let clipped_primitives = self
                .egui_ctx
                .tessellate(full_output.shapes, full_output.pixels_per_point);

            let Some(surface) = self.surface.as_ref() else {
                return rects;
            };
            // Capture validation errors from this frame instead of the shared device's
            // default (panicking) uncaptured-error handler.
            let scope = self
                .render_state
                .device
                .push_error_scope(wgpu::ErrorFilter::Validation);
            if std::mem::take(&mut self.reconfigure_pending) {
                let _ = Self::configure_surface(
                    surface,
                    &self.render_state,
                    self.alpha_mode,
                    w_px,
                    h_px,
                );
            }

            let mut encoder =
                self.render_state
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("riftcore_overlay_encoder"),
                    });

            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [w_px.max(1), h_px.max(1)],
                pixels_per_point: full_output.pixels_per_point,
            };

            for (id, deltas) in self.pending_textures_delta.set.drain() {
                for delta in deltas {
                    self.renderer.update_texture(
                        &self.render_state.device,
                        &self.render_state.queue,
                        id,
                        &delta,
                    );
                }
            }
            let user_cmd_bufs = self.renderer.update_buffers(
                &self.render_state.device,
                &self.render_state.queue,
                &mut encoder,
                &clipped_primitives,
                &screen_descriptor,
            );

            // Outdated/Lost: reconfigure or recreate, then retry once in the same frame.
            let mut output_frame = None;
            for retry in [false, true] {
                let Some(surface) = self.surface.as_ref() else {
                    break;
                };
                match surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(frame) => {
                        self.last_surface_status = "Success".to_string();
                        output_frame = Some(frame);
                        break;
                    }
                    wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                        // Configuring while a texture is acquired is invalid: do it next frame.
                        self.reconfigure_pending = true;
                        self.last_surface_status = "Suboptimal".to_string();
                        output_frame = Some(frame);
                        break;
                    }
                    wgpu::CurrentSurfaceTexture::Outdated if !retry => {
                        let configured = Self::configure_surface(
                            surface,
                            &self.render_state,
                            self.alpha_mode,
                            w_px,
                            h_px,
                        );
                        if let Err(err) = configured {
                            self.note_surface_issue(format!("Outdated: {err}"));
                        }
                    }
                    wgpu::CurrentSurfaceTexture::Lost if !retry => {
                        self.surface = None;
                        match Self::create_surface_for_hwnd(
                            &self.render_state,
                            self.hwnd,
                            [w_px, h_px],
                        ) {
                            Ok((new_surf, new_alpha, new_modes)) => {
                                self.surface = Some(new_surf);
                                self.alpha_mode = new_alpha;
                                self.supported_alpha_modes = new_modes;
                            }
                            Err(err) => self.note_surface_issue(format!("Lost: {err}")),
                        }
                    }
                    other => {
                        self.note_surface_issue(format!("{other:?}"));
                        break;
                    }
                }
            }

            if let Some(frame) = output_frame {
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                {
                    let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("riftcore_overlay_pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    self.renderer.render(
                        &mut render_pass.forget_lifetime(),
                        &clipped_primitives,
                        &screen_descriptor,
                    );
                }
                let encoded = encoder.finish();
                self.render_state
                    .queue
                    .submit(user_cmd_bufs.into_iter().chain([encoded]));
                for id in self.pending_textures_delta.free.drain() {
                    self.renderer.free_texture(&id);
                }
                self.render_state.queue.present(frame);
                self.frames_rendered += 1;
            }
            if let Some(err) = poll_ready(scope.pop()).flatten() {
                self.note_surface_issue(format!("validation error: {err}"));
            }
            if self.show_pending {
                self.show_now();
            }

            rects
        }

        fn diagnostics(&self) -> OverlayDiagnostics {
            let info = self.render_state.adapter.get_info();
            OverlayDiagnostics {
                host_backend: OverlayHostBackend::PathBWin32DComp,
                adapter_backend: Some(info.backend),
                adapter_name: info.name,
                supported_alpha_modes: self.supported_alpha_modes.clone(),
                selected_alpha_mode: Some(self.alpha_mode),
                surface_format: Some(self.render_state.target_format),
                hwnd: self.hwnd as isize,
                ex_style: window_ex_style(self.hwnd as isize),
                frames_rendered: self.frames_rendered,
                resize_events: self.resize_events,
                passthrough_active: self.shared_input.borrow().passthrough,
                rect_px: self.rect_px,
                last_surface_status: self.last_surface_status.clone(),
                topmost_reasserts: self.shared_input.borrow().topmost_reasserts,
            }
        }

        fn close(&mut self) {
            self.visible = false;
            self.show_pending = false;
            self.pending_textures_delta.clear();
            // Drop wgpu surface BEFORE destroying the HWND.
            self.surface.take();
            if !self.hwnd.is_null() {
                if let Ok(mut st) = self.shared_input.try_borrow_mut() {
                    st.pointer_down = false;
                }
                // SAFETY: `self.surface` has already been dropped; `GWLP_USERDATA` is cleared
                // to 0 before `DestroyWindow` so `overlay_wnd_proc` cannot read `shared_input`
                // while or after it is freed.
                unsafe {
                    if GetCapture() == self.hwnd {
                        let _ = ReleaseCapture();
                    }
                    ShowWindow(self.hwnd, SW_HIDE);
                    SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
                    DestroyWindow(self.hwnd);
                }
                self.hwnd = std::ptr::null_mut();
            }
            if !self.shared_input_raw.is_null() {
                // SAFETY: `self.hwnd` is destroyed and `GWLP_USERDATA` zeroed, so this is the
                // sole remaining raw pointer from `Rc::into_raw` in `Win32OverlayHost::new`.
                unsafe {
                    let _ = Rc::from_raw(self.shared_input_raw);
                }
                self.shared_input_raw = std::ptr::null();
            }
        }
    }

    impl Drop for Win32OverlayHost {
        fn drop(&mut self) {
            self.close();
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use crate::overlay::host::{
        OverlayDiagnostics, OverlayHostBackend, OverlayRenderState, OverlaySurfaceHost,
    };
    use eframe::egui::{self, Rect};
    use eframe::egui_wgpu;

    /// Non-Windows stub returning `Err` so [`crate::overlay::host::select_host_or_fallback`] falls back cleanly.
    pub struct Win32OverlayHost;

    impl Win32OverlayHost {
        pub fn new(_render_state: egui_wgpu::RenderState) -> Result<Self, String> {
            Err("Win32OverlayHost requires Windows".to_string())
        }

        pub fn hwnd(&self) -> isize {
            0
        }

        pub fn set_force_passthrough(&mut self, _force: bool) {}

        pub fn set_repaint_context(&mut self, _ctx: egui::Context) {}

        pub fn has_repaint_context(&self) -> bool {
            false
        }

        pub fn take_display_changed(&mut self) -> bool {
            false
        }
    }

    impl OverlaySurfaceHost for Win32OverlayHost {
        fn backend_kind(&self) -> OverlayHostBackend {
            OverlayHostBackend::PathBWin32DComp
        }

        fn sync_window(
            &mut self,
            _visible: bool,
            _rect_px: [i32; 4],
            _pixels_per_point: f32,
            _passthrough: bool,
        ) {
        }

        fn render_if_needed(
            &mut self,
            _force_redraw: bool,
            _state: &mut OverlayRenderState,
            _render_ui: &mut dyn FnMut(&mut egui::Ui, &mut OverlayRenderState, &mut Vec<Rect>),
        ) -> Vec<Rect> {
            Vec::new()
        }

        fn diagnostics(&self) -> OverlayDiagnostics {
            OverlayDiagnostics::default()
        }

        fn close(&mut self) {}
    }
}

pub use imp::Win32OverlayHost;
