//! HUD presentation modes and mixed-DPI physical/logical coordinate geometry.
//!
//! All overlay window rectangles are computed in Win32 virtual-screen physical pixels
//! (`[left, top, right, bottom]`) using the target monitor's DPI, avoiding `egui` logical
//! point / `zoom_factor` drift across mixed-DPI multi-monitor setups.

use crate::platform::window::{dpi_scale, LeagueWindowInfo, MonitorMetrics};
use eframe::egui::{Pos2, Rect, Vec2};

/// Visual layout style of the in-game companion overlay.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum HudPresentationMode {
    #[default]
    Integrated,
    Bar,
    Card,
    Pill,
}

impl HudPresentationMode {
    /// Parses persisted `hud_mode` strings cleanly (`"bar"`, `"pill"`, `"card"`, `"integrated"`).
    pub fn parse_lossy(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "bar" => Self::Bar,
            "pill" => Self::Pill,
            "card" => Self::Card,
            _ => Self::Integrated,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Integrated => "integrated",
            Self::Bar => "bar",
            Self::Card => "card",
            Self::Pill => "pill",
        }
    }

    pub const fn display_label(self) -> &'static str {
        match self {
            Self::Integrated => "Integrated",
            Self::Bar => "Bar",
            Self::Card => "Card",
            Self::Pill => "Pill",
        }
    }
}

/// Detected OS presentation state of the target League game window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DisplayPresentationMode {
    Borderless,
    Windowed,
    ExclusiveFullscreen,
    Minimized,
    #[default]
    Unknown,
}

/// Snapshot of the target League game window in physical screen coordinates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetWindowState {
    pub hwnd: isize,
    pub client_rect_px: [i32; 4],
    pub monitor_dpi: u32,
    pub is_foreground: bool,
    pub presentation: DisplayPresentationMode,
}

impl TargetWindowState {
    /// Builds a `TargetWindowState` from a `LeagueWindowInfo` discovered off the UI thread.
    ///
    /// Exclusive fullscreen cannot be distinguished reliably from borderless by monitor bounds
    /// alone; `game_cfg_window_mode` (`Some(0)`) marks exclusive fullscreen explicitly when known.
    pub fn from_league_window(
        info: LeagueWindowInfo,
        is_foreground: bool,
        monitor: Option<&MonitorMetrics>,
        game_cfg_window_mode: Option<u8>,
    ) -> Self {
        let presentation = infer_presentation_mode(&info, monitor, game_cfg_window_mode);
        Self {
            hwnd: info.hwnd,
            client_rect_px: info.bounds,
            monitor_dpi: info.dpi.max(96),
            is_foreground,
            presentation,
        }
    }
}

/// Infers `DisplayPresentationMode` conservatively so borderless windows are never misclassified
/// as exclusive fullscreen solely for spanning a monitor.
pub fn infer_presentation_mode(
    info: &LeagueWindowInfo,
    monitor: Option<&MonitorMetrics>,
    game_cfg_window_mode: Option<u8>,
) -> DisplayPresentationMode {
    let w = info.bounds[2] - info.bounds[0];
    let h = info.bounds[3] - info.bounds[1];
    if w <= 0 || h <= 0 {
        return DisplayPresentationMode::Minimized;
    }
    if game_cfg_window_mode == Some(0) {
        return DisplayPresentationMode::ExclusiveFullscreen;
    }
    if let Some(m) = monitor {
        if info.bounds == m.rect_px {
            return DisplayPresentationMode::Borderless;
        }
    }
    DisplayPresentationMode::Windowed
}

/// Selects the monitor containing the center (or intersecting largest area) of `rect_px`.
pub fn monitor_for_rect(rect_px: [i32; 4], monitors: &[MonitorMetrics]) -> Option<&MonitorMetrics> {
    if monitors.is_empty() {
        return None;
    }
    let cx = rect_px[0].saturating_add(rect_px[2]) / 2;
    let cy = rect_px[1].saturating_add(rect_px[3]) / 2;
    if let Some(m) = monitors.iter().find(|m| {
        cx >= m.rect_px[0] && cx < m.rect_px[2] && cy >= m.rect_px[1] && cy < m.rect_px[3]
    }) {
        return Some(m);
    }
    monitors.iter().max_by_key(|m| {
        let ix = (rect_px[2].min(m.rect_px[2]) - rect_px[0].max(m.rect_px[0])).max(0) as i64;
        let iy = (rect_px[3].min(m.rect_px[3]) - rect_px[1].max(m.rect_px[1])).max(0) as i64;
        ix * iy
    })
}

/// Computes exact Win32 physical virtual-screen coordinates `[left, top, right, bottom]`
/// for the overlay window across mixed-DPI multi-monitor coordinates.
pub fn compute_overlay_rect_px(
    target: &TargetWindowState,
    monitor: Option<&MonitorMetrics>,
    mode: HudPresentationMode,
    mayhem_open: bool,
) -> [i32; 4] {
    let [t_left, t_top, t_right, t_bottom] = target.client_rect_px;
    let game_w = (t_right - t_left).max(1);
    let game_h = (t_bottom - t_top).max(1);
    let scale = dpi_scale(target.monitor_dpi);

    let (off_x, off_y, width_px, height_px) = if mayhem_open && mode != HudPresentationMode::Integrated
    {
        let w = (430.0 * scale).round() as i32;
        let h = (560.0 * scale).round() as i32;
        (
            (game_w - w).max(0),
            (52.0 * scale).round() as i32,
            w.min(game_w),
            h.min(game_h),
        )
    } else {
        match mode {
            HudPresentationMode::Bar => {
                let w = (620.0 * scale).round() as i32;
                let h = (54.0 * scale).round() as i32;
                let clamped_w = w.min(game_w);
                (
                    ((game_w - clamped_w) / 2).max(0),
                    (12.0 * scale).round() as i32,
                    clamped_w,
                    h.min(game_h),
                )
            }
            HudPresentationMode::Pill => {
                let w = (360.0 * scale).round() as i32;
                let h = (48.0 * scale).round() as i32;
                let clamped_w = w.min(game_w);
                (
                    (game_w - clamped_w - (14.0 * scale).round() as i32).max(0),
                    (58.0 * scale).round() as i32,
                    clamped_w,
                    h.min(game_h),
                )
            }
            HudPresentationMode::Card => {
                let w = (360.0 * scale).round() as i32;
                let h = (330.0 * scale).round() as i32;
                let clamped_w = w.min(game_w);
                (
                    (game_w - clamped_w - (14.0 * scale).round() as i32).max(0),
                    (58.0 * scale).round() as i32,
                    clamped_w,
                    h.min(game_h),
                )
            }
            HudPresentationMode::Integrated => {
                // Keep 1 physical pixel inside a full-screen client rect so Windows DWM does not
                // treat the topmost DirectComposition window as exclusive full-screen occlusion.
                (0, 0, (game_w - 1).max(1), (game_h - 1).max(1))
            }
        }
    };

    let mut left = t_left + off_x;
    let mut top = t_top + off_y;
    let mut right = left + width_px.max(1);
    let mut bottom = top + height_px.max(1);

    if let Some(m) = monitor {
        let [m_left, m_top, m_right, m_bottom] = m.rect_px;
        let m_w = (m_right - m_left).max(1);
        let m_h = (m_bottom - m_top).max(1);
        let cur_w = (right - left).min(m_w);
        let cur_h = (bottom - top).min(m_h);
        let max_left = (m_right - cur_w).max(m_left);
        let max_top = (m_bottom - cur_h).max(m_top);
        left = left.clamp(m_left, max_left);
        top = top.clamp(m_top, max_top);
        right = left + cur_w;
        bottom = top + cur_h;
    }

    [left, top, right, bottom]
}

/// Converts physical Win32 virtual-screen cursor coordinates to overlay-local logical points.
pub fn physical_cursor_to_logical(
    cursor_px: [i32; 2],
    overlay_rect_px: [i32; 4],
    pixels_per_point: f32,
) -> Pos2 {
    let ppp = pixels_per_point.max(0.5);
    Pos2::new(
        (cursor_px[0] - overlay_rect_px[0]) as f32 / ppp,
        (cursor_px[1] - overlay_rect_px[1]) as f32 / ppp,
    )
}

/// Clamps a saved logical panel position inside `area` so resolution or monitor switches
/// never strand draggable panels off-screen.
pub fn clamp_logical_panel_pos(pos: [f32; 2], area: Rect, panel_size: Vec2) -> Pos2 {
    let max_x = (area.right() - panel_size.x).max(area.left());
    let max_y = (area.bottom() - panel_size.y).max(area.top());
    Pos2::new(
        pos[0].clamp(area.left(), max_x),
        pos[1].clamp(area.top(), max_y),
    )
}
