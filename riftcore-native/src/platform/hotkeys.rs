use std::sync::mpsc::{self, Receiver};
use std::thread::JoinHandle;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HotkeyEvent {
    pub toggle_overlay: bool,
    pub toggle_passthrough: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HotkeyRegistrationStatus {
    pub toggle_alt_tilde: bool,
    pub passthrough_alt_shift_tilde: bool,
    pub toggle_ctrl_space: bool,
}

/// Lifecycle guard for the global hotkey listener thread.
/// Posting `WM_QUIT` on `shutdown()` or `Drop` guarantees clean `UnregisterHotKey`
/// deregistration and prevents leaked background threads or stale OS hotkeys.
pub struct HotkeyHandle {
    #[cfg(windows)]
    thread_id: u32,
    join_handle: Option<JoinHandle<()>>,
    pub registration: HotkeyRegistrationStatus,
}

impl HotkeyHandle {
    /// Signals the Win32 message loop with `WM_QUIT` and joins the listener thread.
    pub fn shutdown(&mut self) {
        #[cfg(windows)]
        if self.thread_id != 0 {
            #[link(name = "user32")]
            extern "system" {
                fn PostThreadMessageW(
                    id_thread: u32,
                    msg: u32,
                    wparam: usize,
                    lparam: isize,
                ) -> i32;
            }
            const WM_QUIT: u32 = 0x0012;
            // SAFETY: `self.thread_id` was retrieved from `GetCurrentThreadId()` after the
            // listener thread forced its Win32 message queue creation with `PeekMessageW`.
            unsafe {
                let _ = PostThreadMessageW(self.thread_id, WM_QUIT, 0, 0);
            }
            self.thread_id = 0;
        }

        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for HotkeyHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Starts the dedicated Win32 global hotkey message loop with `MOD_NOREPEAT` enabled
/// on all bindings (`Alt+\``, `Alt+Shift+\``, and `Ctrl+Space`).
pub fn start_hotkey_listener() -> (Receiver<HotkeyEvent>, HotkeyHandle) {
    let (event_tx, event_rx) = mpsc::channel();

    #[cfg(windows)]
    {
        let (init_tx, init_rx) = mpsc::sync_channel::<(u32, HotkeyRegistrationStatus)>(1);
        let join_handle = std::thread::Builder::new()
            .name("riftcore-hotkeys".to_string())
            .spawn(move || {
                use std::ffi::c_void;

                #[repr(C)]
                struct Point {
                    x: i32,
                    y: i32,
                }

                #[repr(C)]
                struct Message {
                    hwnd: *mut c_void,
                    message: u32,
                    wparam: usize,
                    lparam: isize,
                    time: u32,
                    point: Point,
                    private: u32,
                }

                #[link(name = "kernel32")]
                extern "system" {
                    fn GetCurrentThreadId() -> u32;
                }

                #[link(name = "user32")]
                extern "system" {
                    fn PeekMessageW(
                        msg: *mut Message,
                        hwnd: *mut c_void,
                        msg_filter_min: u32,
                        msg_filter_max: u32,
                        remove_msg: u32,
                    ) -> i32;
                    fn RegisterHotKey(
                        hwnd: *mut c_void,
                        id: i32,
                        modifiers: u32,
                        vk: u32,
                    ) -> i32;
                    fn UnregisterHotKey(hwnd: *mut c_void, id: i32) -> i32;
                    fn GetMessageW(
                        msg: *mut Message,
                        hwnd: *mut c_void,
                        msg_filter_min: u32,
                        msg_filter_max: u32,
                    ) -> i32;
                }

                const MOD_ALT: u32 = 0x0001;
                const MOD_CONTROL: u32 = 0x0002;
                const MOD_SHIFT: u32 = 0x0004;
                const MOD_NOREPEAT: u32 = 0x4000;
                const VK_OEM_3: u32 = 0x00C0;
                const VK_SPACE: u32 = 0x0020;
                const WM_HOTKEY: u32 = 0x0312;
                const PM_NOREMOVE: u32 = 0x0000;

                const ID_TOGGLE_ALT_TILDE: i32 = 1;
                const ID_PASSTHROUGH_ALT_SHIFT_TILDE: i32 = 2;
                const ID_TOGGLE_CTRL_SPACE: i32 = 3;

                let mut message = Message {
                    hwnd: std::ptr::null_mut(),
                    message: 0,
                    wparam: 0,
                    lparam: 0,
                    time: 0,
                    point: Point { x: 0, y: 0 },
                    private: 0,
                };

                // SAFETY: Calling `PeekMessageW` on the current thread forces creation of its
                // Win32 message queue before `GetCurrentThreadId` is published for `PostThreadMessageW`.
                // `RegisterHotKey`/`UnregisterHotKey`/`GetMessageW` operate strictly on thread-local
                // hotkeys (`hwnd = null`) and stack-allocated `Message`.
                unsafe {
                    let _ = PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_NOREMOVE);

                    let status = HotkeyRegistrationStatus {
                        toggle_alt_tilde: RegisterHotKey(
                            std::ptr::null_mut(),
                            ID_TOGGLE_ALT_TILDE,
                            MOD_ALT | MOD_NOREPEAT,
                            VK_OEM_3,
                        ) != 0,
                        passthrough_alt_shift_tilde: RegisterHotKey(
                            std::ptr::null_mut(),
                            ID_PASSTHROUGH_ALT_SHIFT_TILDE,
                            MOD_ALT | MOD_SHIFT | MOD_NOREPEAT,
                            VK_OEM_3,
                        ) != 0,
                        toggle_ctrl_space: RegisterHotKey(
                            std::ptr::null_mut(),
                            ID_TOGGLE_CTRL_SPACE,
                            MOD_CONTROL | MOD_NOREPEAT,
                            VK_SPACE,
                        ) != 0,
                    };

                    let thread_id = GetCurrentThreadId();
                    let _ = init_tx.send((thread_id, status));

                    loop {
                        let res = GetMessageW(&mut message, std::ptr::null_mut(), 0, 0);
                        if res <= 0 {
                            break;
                        }
                        if message.message == WM_HOTKEY {
                            let event = match message.wparam as i32 {
                                ID_TOGGLE_ALT_TILDE | ID_TOGGLE_CTRL_SPACE => HotkeyEvent {
                                    toggle_overlay: true,
                                    toggle_passthrough: false,
                                },
                                ID_PASSTHROUGH_ALT_SHIFT_TILDE => HotkeyEvent {
                                    toggle_overlay: false,
                                    toggle_passthrough: true,
                                },
                                _ => continue,
                            };
                            if event_tx.send(event).is_err() {
                                break;
                            }
                        }
                    }

                    if status.toggle_alt_tilde {
                        let _ = UnregisterHotKey(std::ptr::null_mut(), ID_TOGGLE_ALT_TILDE);
                    }
                    if status.passthrough_alt_shift_tilde {
                        let _ =
                            UnregisterHotKey(std::ptr::null_mut(), ID_PASSTHROUGH_ALT_SHIFT_TILDE);
                    }
                    if status.toggle_ctrl_space {
                        let _ = UnregisterHotKey(std::ptr::null_mut(), ID_TOGGLE_CTRL_SPACE);
                    }
                }
            })
            .ok();

        let (thread_id, registration) = if join_handle.is_some() {
            init_rx.recv().unwrap_or((0, HotkeyRegistrationStatus::default()))
        } else {
            (0, HotkeyRegistrationStatus::default())
        };

        (
            event_rx,
            HotkeyHandle {
                thread_id,
                join_handle,
                registration,
            },
        )
    }

    #[cfg(not(windows))]
    {
        let _ = event_tx;
        (
            event_rx,
            HotkeyHandle {
                join_handle: None,
                registration: HotkeyRegistrationStatus::default(),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn hotkey_listener_shuts_down_cleanly_on_drop() {
        let (_rx, mut handle) = start_hotkey_listener();
        handle.shutdown();
    }
}
