use std::ffi::c_void;
use std::num::NonZeroIsize;

pub const WS_EX_TOPMOST: usize = 0x0000_0008;
pub const WS_EX_TRANSPARENT: usize = 0x0000_0020;
pub const WS_EX_TOOLWINDOW: usize = 0x0000_0080;
pub const WS_EX_APPWINDOW: usize = 0x0004_0000;
pub const WS_EX_LAYERED: usize = 0x0008_0000;
pub const WS_EX_NOREDIRECTIONBITMAP: usize = 0x0020_0000;
pub const WS_EX_NOACTIVATE: usize = 0x0800_0000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OverlayRectPx {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl OverlayRectPx {
    pub fn from_array(rect: [i32; 4]) -> Self {
        Self {
            left: rect[0],
            top: rect[1],
            right: rect[2],
            bottom: rect[3],
        }
    }

    pub fn width(self) -> i32 {
        (self.right - self.left).max(1)
    }

    pub fn height(self) -> i32 {
        (self.bottom - self.top).max(1)
    }

    pub fn to_array(self) -> [i32; 4] {
        [self.left, self.top, self.right, self.bottom]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayWindowConfig {
    pub class_name: String,
    pub title: String,
    pub passthrough: bool,
}

impl Default for OverlayWindowConfig {
    fn default() -> Self {
        Self {
            class_name: "RiftCoreOverlayWindowClass".to_string(),
            title: "RiftCore In-Game HUD".to_string(),
            passthrough: true,
        }
    }
}

/// Safe RAII handle wrapper over a Win32 overlay `HWND`.
/// Wave 2 (`overlay::win32_host`) extends creation / DirectComposition visual binding.
#[derive(Debug)]
pub struct OverlayWindowHandle {
    hwnd: NonZeroIsize,
    owns_window: bool,
}

impl OverlayWindowHandle {
    /// Wraps an already-created `HWND` without taking ownership of `DestroyWindow`.
    pub fn borrow_existing(hwnd: isize) -> Option<Self> {
        NonZeroIsize::new(hwnd).map(|hwnd| Self {
            hwnd,
            owns_window: false,
        })
    }

    /// Wraps a newly created `HWND` owned by this handle (destroyed on `Drop`).
    pub fn from_owned(hwnd: isize) -> Option<Self> {
        NonZeroIsize::new(hwnd).map(|hwnd| Self {
            hwnd,
            owns_window: true,
        })
    }

    pub fn hwnd(&self) -> isize {
        self.hwnd.get()
    }

    pub fn raw_ptr(&self) -> *mut c_void {
        self.hwnd.get() as *mut c_void
    }

    pub fn ex_style(&self) -> usize {
        crate::platform::window::window_ex_style(self.hwnd.get())
    }

    #[cfg(windows)]
    pub fn set_passthrough(&mut self, passthrough: bool) -> Result<(), String> {
        #[link(name = "kernel32")]
        extern "system" {
            fn SetLastError(err: u32);
            fn GetLastError() -> u32;
        }
        #[link(name = "user32")]
        extern "system" {
            fn GetWindowLongPtrW(hwnd: *mut c_void, index: i32) -> isize;
            fn SetWindowLongPtrW(hwnd: *mut c_void, index: i32, new_long: isize) -> isize;
            fn SetWindowPos(
                hwnd: *mut c_void,
                insert_after: *mut c_void,
                x: i32,
                y: i32,
                cx: i32,
                cy: i32,
                flags: u32,
            ) -> i32;
        }

        const GWL_EXSTYLE: i32 = -20;
        const SWP_NOSIZE: u32 = 0x0001;
        const SWP_NOMOVE: u32 = 0x0002;
        const SWP_NOZORDER: u32 = 0x0004;
        const SWP_NOACTIVATE: u32 = 0x0010;
        const SWP_FRAMECHANGED: u32 = 0x0020;

        let hwnd_ptr = self.raw_ptr();
        // SAFETY: `self.hwnd` is non-zero; we clear and check `GetLastError` around
        // `GetWindowLongPtrW` and `SetWindowLongPtrW` as required by Win32 documentation.
        unsafe {
            SetLastError(0);
            let current = GetWindowLongPtrW(hwnd_ptr, GWL_EXSTYLE);
            let err = GetLastError();
            if current == 0 && err != 0 {
                return Err(format!(
                    "GetWindowLongPtrW failed with Win32 error {err:#x}"
                ));
            }
            let mut next = (current as usize) | WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE;
            next &= !WS_EX_APPWINDOW;
            if passthrough {
                next |= WS_EX_TRANSPARENT;
            } else {
                next &= !WS_EX_TRANSPARENT;
            }
            SetLastError(0);
            let prev = SetWindowLongPtrW(hwnd_ptr, GWL_EXSTYLE, next as isize);
            let set_err = GetLastError();
            if prev == 0 && set_err != 0 {
                return Err(format!(
                    "SetWindowLongPtrW failed with Win32 error {set_err:#x}"
                ));
            }
            let _ = SetWindowPos(
                hwnd_ptr,
                std::ptr::null_mut(),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );
        }
        Ok(())
    }

    #[cfg(not(windows))]
    pub fn set_passthrough(&mut self, _passthrough: bool) -> Result<(), String> {
        Err("Win32 overlay window is only supported on Windows".to_string())
    }

    #[cfg(windows)]
    pub fn set_bounds_and_visibility(
        &mut self,
        rect: OverlayRectPx,
        visible: bool,
    ) -> Result<(), String> {
        #[link(name = "user32")]
        extern "system" {
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
        }

        const SW_HIDE: i32 = 0;
        const SW_SHOWNOACTIVATE: i32 = 4;
        const HWND_TOPMOST: isize = -1;
        const SWP_NOACTIVATE: u32 = 0x0010;

        let hwnd_ptr = self.raw_ptr();
        // SAFETY: `hwnd_ptr` is non-zero; `SetWindowPos` and `ShowWindow` keep focus on the game.
        unsafe {
            if SetWindowPos(
                hwnd_ptr,
                HWND_TOPMOST as *mut c_void,
                rect.left,
                rect.top,
                rect.width(),
                rect.height(),
                SWP_NOACTIVATE,
            ) == 0
            {
                return Err("SetWindowPos failed while positioning overlay window".to_string());
            }
            let _ = ShowWindow(hwnd_ptr, if visible { SW_SHOWNOACTIVATE } else { SW_HIDE });
        }
        Ok(())
    }

    #[cfg(not(windows))]
    pub fn set_bounds_and_visibility(
        &mut self,
        _rect: OverlayRectPx,
        _visible: bool,
    ) -> Result<(), String> {
        Err("Win32 overlay window is only supported on Windows".to_string())
    }

    #[cfg(windows)]
    pub fn maintain_topmost_noactivate(&self) -> Result<(), String> {
        #[link(name = "user32")]
        extern "system" {
            fn SetWindowPos(
                hwnd: *mut c_void,
                insert_after: *mut c_void,
                x: i32,
                y: i32,
                cx: i32,
                cy: i32,
                flags: u32,
            ) -> i32;
        }
        const HWND_TOPMOST: isize = -1;
        const SWP_NOSIZE: u32 = 0x0001;
        const SWP_NOMOVE: u32 = 0x0002;
        const SWP_NOACTIVATE: u32 = 0x0010;
        // SAFETY: `self.hwnd` is non-zero and `SetWindowPos` applies `HWND_TOPMOST | SWP_NOACTIVATE`.
        let ok = unsafe {
            SetWindowPos(
                self.raw_ptr(),
                HWND_TOPMOST as *mut c_void,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            )
        };
        if ok == 0 {
            Err("SetWindowPos failed to maintain overlay topmost z-order".to_string())
        } else {
            Ok(())
        }
    }

    #[cfg(not(windows))]
    pub fn maintain_topmost_noactivate(&self) -> Result<(), String> {
        Err("Win32 overlay window is only supported on Windows".to_string())
    }
}

impl Drop for OverlayWindowHandle {
    fn drop(&mut self) {
        #[cfg(windows)]
        if self.owns_window {
            #[link(name = "user32")]
            extern "system" {
                fn DestroyWindow(hwnd: *mut c_void) -> i32;
            }
            // SAFETY: `self.owns_window` ensures exclusive ownership of `self.hwnd`.
            unsafe {
                let _ = DestroyWindow(self.raw_ptr());
            }
        }
    }
}

/// Safe fallback maintenance helper for Path A (`eframe` child viewport) looked up by title,
/// verifying same-process ownership before modifying extended styles.
#[cfg(windows)]
pub fn maintain_native_overlay_by_title(title: &str) {
    #[link(name = "user32")]
    extern "system" {
        fn FindWindowW(class_name: *const u16, window_name: *const u16) -> *mut c_void;
        fn GetWindowThreadProcessId(hwnd: *mut c_void, process_id: *mut u32) -> u32;
    }

    let wide_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: `wide_title` is NUL-terminated; `FindWindowW` returns null or an HWND whose
    // owning PID we verify matches `std::process::id()` before mutating extended styles.
    let hwnd = unsafe {
        let hwnd = FindWindowW(std::ptr::null(), wide_title.as_ptr());
        if hwnd.is_null() {
            return;
        }
        let mut owner = 0u32;
        let _ = GetWindowThreadProcessId(hwnd, &mut owner);
        if owner != std::process::id() {
            return;
        }
        hwnd as isize
    };

    if let Some(mut handle) = OverlayWindowHandle::borrow_existing(hwnd) {
        let passthrough = (handle.ex_style() & WS_EX_TRANSPARENT) != 0;
        let _ = handle.set_passthrough(passthrough);
        let _ = handle.maintain_topmost_noactivate();
    }
}

#[cfg(not(windows))]
pub fn maintain_native_overlay_by_title(_title: &str) {}
