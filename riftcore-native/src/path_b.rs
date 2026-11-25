//! Bridge from the classic app to the Path B in-game HUD (dedicated Win32 window rendered
//! with DirectX 12 + DirectComposition, true per-pixel transparency).
//!
//! The classic app keeps owning the HUD on/off state, hotkeys and auto-start; this bridge
//! feeds that state, the League window position and live match data into the shared overlay
//! engine and panels (`riftcore_native::overlay`). If the Path B window cannot be created the
//! app keeps using its original child-viewport overlay instead.

use riftcore_native::config::settings::AppSettings as OverlaySettingsSource;
use riftcore_native::domain::live_game::LiveGameState;
use riftcore_native::overlay::engine::{OverlayCommand, OverlayConfig, OverlayEngine, TickInput};
use riftcore_native::overlay::host::{
    OverlayFrameActions, OverlayPanelSettings, OverlayRenderState, OverlaySurfaceHost,
};
use riftcore_native::overlay::hud_modes::{
    monitor_for_rect, HudPresentationMode, TargetWindowState,
};
use riftcore_native::overlay::panels::render_overlay_ui;
use riftcore_native::overlay::win32_host::Win32OverlayHost;
use riftcore_native::platform::window::{
    cursor_screen_position, find_league_game_window, is_league_window_foreground, query_monitors,
    MonitorMetrics,
};
use serde_json::Value;
use std::time::{Duration, Instant};

const MONITOR_REFRESH: Duration = Duration::from_secs(2);
const TARGET_REFRESH: Duration = Duration::from_millis(250);

/// What the classic app tells the HUD each frame.
pub struct PathBFrame<'a> {
    pub enabled: bool,
    pub automatic: bool,
    pub settings: &'a crate::backend::AppSettings,
    pub force_passthrough: bool,
    pub show_mayhem_cards: bool,
}

/// What the HUD hands back after a frame.
pub struct PathBOutput {
    pub actions: OverlayFrameActions,
    pub show_mayhem_cards: bool,
    pub next_repaint: Option<Duration>,
}

pub struct PathBOverlay {
    host: Win32OverlayHost,
    engine: OverlayEngine,
    config: OverlayConfig,
    render: OverlayRenderState,
    monitors: Vec<MonitorMetrics>,
    monitors_at: Option<Instant>,
    target_at: Option<Instant>,
}

impl PathBOverlay {
    /// Creates the Path B HUD window. Fails when the renderer is not DirectX 12 with
    /// DirectComposition support, or when `RIFTCORE_OVERLAY=classic` asks for the old HUD.
    pub fn new(render_state: Option<eframe::egui_wgpu::RenderState>) -> Result<Self, String> {
        if std::env::var("RIFTCORE_OVERLAY")
            .is_ok_and(|value| value.eq_ignore_ascii_case("classic"))
        {
            return Err("classic overlay requested by RIFTCORE_OVERLAY".to_string());
        }
        let render_state = render_state.ok_or_else(|| "renderer is not wgpu".to_string())?;
        let host = Win32OverlayHost::new(render_state)?;
        Ok(Self {
            host,
            engine: OverlayEngine::default(),
            config: OverlayConfig::default(),
            render: OverlayRenderState::default(),
            monitors: Vec::new(),
            monitors_at: None,
            target_at: None,
        })
    }

    /// Call when a new client snapshot arrives (about once a second during a match).
    pub fn set_live_game(&mut self, raw: Option<&Value>) {
        self.render
            .apply_live_game(raw.and_then(LiveGameState::from_allgamedata));
    }

    pub fn tick(&mut self, frame: PathBFrame<'_>) -> PathBOutput {
        if self
            .monitors_at
            .is_none_or(|at| at.elapsed() >= MONITOR_REFRESH)
        {
            self.monitors = query_monitors();
            self.monitors_at = Some(Instant::now());
        }
        if self
            .target_at
            .is_none_or(|at| at.elapsed() >= TARGET_REFRESH)
        {
            let target = find_league_game_window().map(|info| {
                let monitor = monitor_for_rect(info.bounds, &self.monitors);
                TargetWindowState::from_league_window(
                    info,
                    is_league_window_foreground(info),
                    monitor,
                    None,
                )
            });
            self.engine
                .apply_command(OverlayCommand::UpdateTarget(target), &mut self.config);
            self.target_at = Some(Instant::now());
        }
        self.engine.apply_command(
            OverlayCommand::SetEnabled {
                enabled: frame.enabled,
                automatic: frame.automatic,
            },
            &mut self.config,
        );

        let settings = frame.settings;
        self.config.mode = HudPresentationMode::parse_lossy(&settings.hud_mode);
        self.config.edit_layout = settings.hud_edit_layout;
        self.config.positions.clone_from(&settings.hud_positions);
        self.config.force_passthrough = frame.force_passthrough;
        self.config.mayhem_open = frame.show_mayhem_cards;

        // Both settings structs share the same JSON shape, so convert through JSON.
        let source: OverlaySettingsSource = serde_json::to_value(settings)
            .ok()
            .and_then(|value| serde_json::from_value(value).ok())
            .unwrap_or_default();
        self.render.settings = OverlayPanelSettings::from_app_settings(&source);
        self.render.preview_mode = false;
        self.render.show_mayhem_cards = frame.show_mayhem_cards;

        let monitor = self
            .engine
            .target()
            .and_then(|target| monitor_for_rect(target.client_rect_px, &self.monitors))
            .cloned();
        let pointer_down = self.host.pointer_down();
        let (_, next_repaint) = self.engine.tick(
            &mut self.host,
            TickInput {
                monitor: monitor.as_ref(),
                cursor_px: cursor_screen_position(),
                pointer_down,
                config: &self.config,
                render_state: &mut self.render,
                force_interactive_redraw: false,
            },
            &mut render_overlay_ui,
        );

        PathBOutput {
            actions: std::mem::take(&mut self.render.pending_actions),
            show_mayhem_cards: self.render.show_mayhem_cards,
            next_repaint,
        }
    }

    pub fn close(&mut self) {
        self.host.close();
    }
}
