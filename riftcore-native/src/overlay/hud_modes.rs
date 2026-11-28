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
    /// The DPI is the monitor's (what Windows renders the HUD window at), falling back to the
    /// game window's own DPI when no monitor is known.
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
            monitor_dpi: monitor.map_or(info.dpi, |m| m.dpi).max(96),
            is_foreground,
            presentation,
        }
    }

    /// Pixels per point the HUD renders at over this game window; see [`hud_pixels_per_point`].
    pub fn hud_pixels_per_point(&self) -> f32 {
        let [left, top, right, bottom] = self.client_rect_px;
        hud_pixels_per_point(
            self.monitor_dpi,
            [right.saturating_sub(left), bottom.saturating_sub(top)],
        )
    }
}

/// Smallest HUD canvas, in points, that the panel layout is designed for (16:9 at 720p).
pub const MIN_HUD_CANVAS_PTS: [f32; 2] = [1280.0, 720.0];

/// Pixels per point for the HUD over a `client_px` game: the monitor's Windows scaling, lowered
/// (never below 1.0) when it would leave less than [`MIN_HUD_CANVAS_PTS`]. League's own HUD
/// scales with the game resolution, not with Windows scaling, so e.g. 1920x1080 at 200 % renders
/// the HUD at 150 % instead of letting its panels cover the game's ability bar and minimap.
pub fn hud_pixels_per_point(monitor_dpi: u32, client_px: [i32; 2]) -> f32 {
    let fit_w = client_px[0].max(1) as f32 / MIN_HUD_CANVAS_PTS[0];
    let fit_h = client_px[1].max(1) as f32 / MIN_HUD_CANVAS_PTS[1];
    dpi_scale(monitor_dpi).min(fit_w).min(fit_h).max(1.0)
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
///
/// The HUD window covers the game's client area in every HUD style: the style only changes
/// the main stats panel, and the other overlays (minimap pins, timers, benchmark, drawers)
/// need the whole game to draw on.
pub fn compute_overlay_rect_px(
    target: &TargetWindowState,
    monitor: Option<&MonitorMetrics>,
) -> [i32; 4] {
    let [t_left, t_top, t_right, t_bottom] = target.client_rect_px;
    let game_w = t_right.saturating_sub(t_left).max(1);
    let game_h = t_bottom.saturating_sub(t_top).max(1);
    // Keep 1 physical pixel inside a full-screen client rect so Windows DWM does not treat the
    // topmost DirectComposition window as exclusive full-screen occlusion.
    let (off_x, off_y, width_px, height_px) = (0, 0, (game_w - 1).max(1), (game_h - 1).max(1));

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

/// True when screen-pixel `cursor_px` is inside any of `rects` (logical points, local to a
/// window whose top-left is `origin_px`). Used by the HUD window's per-message hit test.
pub fn cursor_hits_rects(
    rects: &[Rect],
    origin_px: [i32; 2],
    pixels_per_point: f32,
    cursor_px: [i32; 2],
) -> bool {
    let local = physical_cursor_to_logical(
        cursor_px,
        [origin_px[0], origin_px[1], 0, 0],
        pixels_per_point,
    );
    rects.iter().any(|rect| rect.contains(local))
}

/// League's own bottom-centre ability/item bar on a HUD covering the game (`screen`, in points),
/// which RiftCore panels keep clear of. League scales its HUD with the game height, so the
/// zone is 780x190 at 1080 and proportional elsewhere, whatever the Windows scaling.
pub fn league_ability_bar_zone(screen: Rect) -> Rect {
    let unit = screen.height() / 1080.0;
    let size = Vec2::new((780.0 * unit).min(screen.width()), 190.0 * unit);
    Rect::from_min_size(
        Pos2::new(screen.center().x - size.x / 2.0, screen.bottom() - size.y),
        size,
    )
}

/// League's chat box: bottom-left, or bottom-right when the minimap is flipped to the left
/// (both can't share the corner). 480 wide at 1080, the lower 30 % of the height.
pub fn league_chat_zone(screen: Rect, minimap_flipped: bool) -> Rect {
    let size = Vec2::new(
        (480.0 * screen.height() / 1080.0).min(screen.width()),
        screen.height() * 0.3,
    );
    let left = if minimap_flipped {
        screen.right() - size.x
    } else {
        screen.left()
    };
    Rect::from_min_size(Pos2::new(left, screen.bottom() - size.y), size)
}

/// Default centre-top point of the integrated toolbar: top-centre, under RiftCore's scoreboard.
pub fn default_toolbar_pos(area: Rect) -> Pos2 {
    Pos2::new(area.center().x, area.top() + 96.0)
}

/// Default top-left of the ARAM relic panel: left edge, vertically centred (above chat).
pub fn default_relic_pos(area: Rect) -> Pos2 {
    Pos2::new(area.left() + 4.0, area.center().y - 60.0)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_test_uses_window_origin_and_dpi() {
        let rects = [Rect::from_min_size(
            Pos2::new(10.0, 10.0),
            Vec2::new(100.0, 40.0),
        )];
        assert!(cursor_hits_rects(
            &rects,
            [-1920, 100],
            1.5,
            [-1920 + 30, 100 + 30]
        ));
        assert!(!cursor_hits_rects(
            &rects,
            [-1920, 100],
            1.5,
            [-1920 + 200, 100 + 30]
        ));
        assert!(!cursor_hits_rects(&[], [0, 0], 1.0, [0, 0]));
    }

    #[test]
    fn default_panels_avoid_ability_bar_and_chat() {
        for (w, h) in [
            (1280.0, 720.0),
            (1920.0, 1080.0),
            (2560.0, 1440.0),
            (3840.0, 2160.0),
        ] {
            let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(w, h));
            let area = screen.shrink(14.0);
            let toolbar_top = default_toolbar_pos(area);
            let toolbar = Rect::from_min_size(
                Pos2::new(toolbar_top.x - 320.0, toolbar_top.y),
                Vec2::new(640.0, 44.0),
            );
            let relic = Rect::from_min_size(default_relic_pos(area), Vec2::new(200.0, 140.0));
            for panel in [toolbar, relic] {
                assert!(
                    !panel.intersects(league_ability_bar_zone(screen)),
                    "{w}x{h}"
                );
                assert!(
                    !panel.intersects(league_chat_zone(screen, false)),
                    "{w}x{h}"
                );
                assert!(area.contains_rect(panel), "{w}x{h}");
            }
        }
    }

    #[test]
    fn hud_scale_follows_windows_scaling_but_keeps_a_720p_canvas() {
        // (client px, monitor dpi) -> pixels per point
        for (client, dpi, expected) in [
            ([1920, 1080], 96, 1.0),
            ([1920, 1080], 144, 1.5),
            ([1920, 1080], 192, 1.5),
            ([1280, 720], 120, 1.0),
            ([2560, 1440], 144, 1.5),
            ([2560, 1440], 192, 2.0),
            ([3840, 2160], 192, 2.0),
            ([1024, 600], 192, 1.0),
        ] {
            let ppp = hud_pixels_per_point(dpi, client);
            assert!((ppp - expected).abs() < 1e-4, "{client:?}@{dpi}: {ppp}");
            let canvas = [client[0] as f32 / ppp, client[1] as f32 / ppp];
            assert!(
                ppp == 1.0 || (canvas[0] >= 1279.9 && canvas[1] >= 719.9),
                "{client:?}@{dpi}"
            );
        }
        // The HUD follows the monitor's DPI, not the game window's own (possibly unaware) DPI.
        let info = LeagueWindowInfo {
            hwnd: 7,
            process_id: 1,
            bounds: [0, 0, 2560, 1440],
            dpi: 96,
        };
        let monitor = MonitorMetrics {
            rect_px: [0, 0, 2560, 1440],
            work_px: [0, 0, 2560, 1400],
            dpi: 144,
        };
        let target = TargetWindowState::from_league_window(info, true, Some(&monitor), Some(0));
        assert_eq!(target.monitor_dpi, 144);
        assert_eq!(
            target.presentation,
            DisplayPresentationMode::ExclusiveFullscreen
        );
        assert!((target.hud_pixels_per_point() - 1.5).abs() < 1e-4);
    }

    #[test]
    fn hud_window_covers_the_game_one_pixel_inside() {
        for (client, dpi) in [([1280, 720], 96), ([1920, 1080], 144), ([3840, 2160], 192)] {
            let target = TargetWindowState {
                hwnd: 1,
                client_rect_px: [0, 0, client[0], client[1]],
                monitor_dpi: dpi,
                is_foreground: true,
                presentation: DisplayPresentationMode::ExclusiveFullscreen,
            };
            let full = compute_overlay_rect_px(&target, None);
            assert_eq!(full, [0, 0, client[0] - 1, client[1] - 1]);
        }
    }
}
