//! Bridge from the classic app to the Path B in-game HUD (dedicated Win32 window rendered
//! with DirectX 12 + DirectComposition, true per-pixel transparency).
//!
//! The classic app keeps owning the HUD on/off state, hotkeys and auto-start; this bridge
//! feeds that state, the League window position and live match data into the shared overlay
//! engine and panels (`riftcore_native::overlay`). If the Path B window cannot be created the
//! app keeps using its original child-viewport overlay instead.

use eframe::egui;
use riftcore_native::config::settings::AppSettings as OverlaySettingsSource;
use riftcore_native::domain::live_game::LiveGameState;
use riftcore_native::overlay::engine::{
    OverlayCommand, OverlayConfig, OverlayEngine, OverlayLifecycleState, TickInput,
};
use riftcore_native::overlay::host::{
    OverlayFrameActions, OverlayPanelSettings, OverlayRenderState, OverlaySurfaceHost,
};
use riftcore_native::overlay::hud_modes::{
    monitor_for_rect, HudPresentationMode, TargetWindowState,
};
use riftcore_native::overlay::panels::render_overlay_ui;
use riftcore_native::overlay::win32_host::Win32OverlayHost;
use riftcore_native::platform::window::{
    cursor_screen_position, find_league_game_window, is_league_window_foreground,
    minimised_league_window_pid, query_monitors, GameCfgWatcher, LeagueWindowInfo, MonitorMetrics,
};
use serde_json::Value;
use std::hash::Hasher;
use std::time::{Duration, Instant};

const MONITOR_REFRESH: Duration = Duration::from_secs(2);
const TARGET_REFRESH: Duration = Duration::from_millis(250);
/// Hover polling cadence while visible when the HUD window cannot wake the app itself.
const POLL_WITHOUT_WAKE: Duration = Duration::from_millis(100);

/// Why the HUD is or is not on screen; [`PathBStatus::message`] is user-facing text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PathBStatus {
    #[default]
    Disabled,
    WaitingForGame,
    Minimised,
    Background,
    ExclusiveFullscreen,
    Visible,
}

impl PathBStatus {
    pub fn message(self) -> Option<&'static str> {
        match self {
            Self::Disabled | Self::Visible => None,
            Self::WaitingForGame => Some("Waiting for a League of Legends game window"),
            Self::Minimised => Some("League is minimised \u{2014} click back into the game to show the HUD"),
            Self::Background => Some("League is in the background \u{2014} click into the game to show the HUD"),
            Self::ExclusiveFullscreen => Some(
                "League is in Fullscreen \u{2014} switch to Borderless (Esc > Video > Window Mode) to see the HUD",
            ),
        }
    }
}

/// Hashes serialized bytes without allocating, to detect settings changes cheaply.
struct HashWriter(std::collections::hash_map::DefaultHasher);

impl std::io::Write for HashWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

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
    league_window: Option<(LeagueWindowInfo, bool)>,
    minimised: bool,
    game_cfg: GameCfgWatcher,
    window_mode: Option<u8>,
    settings_hash: Option<u64>,
    status: PathBStatus,
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
            league_window: None,
            minimised: false,
            game_cfg: GameCfgWatcher::default(),
            window_mode: None,
            settings_hash: None,
            status: PathBStatus::Disabled,
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
            let window = find_league_game_window();
            self.league_window = window.map(|info| (info, is_league_window_foreground(info)));
            // A skipped-but-present game window is minimised (fullscreen alt-tab gives 1x1).
            let pid = window
                .map(|info| info.process_id)
                .or_else(minimised_league_window_pid);
            self.minimised = window.is_none() && pid.is_some();
            self.window_mode = pid.and_then(|pid| self.game_cfg.poll(pid));
            let target = self.league_window.map(|(info, foreground)| {
                let monitor = monitor_for_rect(info.bounds, &self.monitors);
                TargetWindowState::from_league_window(info, foreground, monitor, self.window_mode)
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

        // Both settings structs share the same JSON shape, so convert through JSON, but only
        // when the settings changed; a failed conversion keeps the last good HUD settings.
        let mut hasher = HashWriter(Default::default());
        let hash = serde_json::to_writer(&mut hasher, settings)
            .ok()
            .map(|()| hasher.0.finish());
        if hash != self.settings_hash {
            self.settings_hash = hash;
            match serde_json::to_value(settings)
                .and_then(serde_json::from_value::<OverlaySettingsSource>)
            {
                Ok(source) => {
                    self.render.settings = OverlayPanelSettings::from_app_settings(&source);
                }
                Err(err) => eprintln!("[riftcore hud] settings conversion failed: {err}"),
            }
        }
        self.render.preview_mode = false;
        self.render.show_mayhem_cards = frame.show_mayhem_cards;

        let monitor = self
            .engine
            .target()
            .and_then(|target| monitor_for_rect(target.client_rect_px, &self.monitors))
            .cloned();
        let pointer_down = self.host.pointer_down();
        self.host.set_force_passthrough(frame.force_passthrough);
        let (state, next) = self.engine.tick(
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
        self.status = match state {
            OverlayLifecycleState::Disabled => PathBStatus::Disabled,
            OverlayLifecycleState::ActiveVisible => PathBStatus::Visible,
            OverlayLifecycleState::BlockedByExclusiveFullscreen => PathBStatus::ExclusiveFullscreen,
            OverlayLifecycleState::WaitingForForeground => PathBStatus::Background,
            OverlayLifecycleState::WaitingForGameWindow if self.minimised => {
                if self.window_mode == Some(0) {
                    PathBStatus::ExclusiveFullscreen
                } else {
                    PathBStatus::Minimised
                }
            }
            OverlayLifecycleState::WaitingForGameWindow => PathBStatus::WaitingForGame,
        };
        // Idle ~1 s, 16 ms while hovering/dragging; keep polling for the game while waiting.
        let next_repaint = match state {
            OverlayLifecycleState::Disabled => None,
            OverlayLifecycleState::ActiveVisible if !self.host.has_repaint_context() => {
                next.map(|delay| delay.min(POLL_WITHOUT_WAKE))
            }
            OverlayLifecycleState::ActiveVisible => next,
            _ => Some(TARGET_REFRESH),
        };

        PathBOutput {
            actions: std::mem::take(&mut self.render.pending_actions),
            show_mayhem_cards: self.render.show_mayhem_cards,
            next_repaint,
        }
    }

    /// Why the HUD is (not) showing; `status().message()` is the notice for the HUD page.
    pub fn status(&self) -> PathBStatus {
        self.status
    }

    /// League game window found by the last 250 ms search, with its foreground flag.
    pub fn league_window(&self) -> Option<(LeagueWindowInfo, bool)> {
        self.league_window
    }

    /// Lets the HUD window wake the app when the cursor reaches a panel or input arrives,
    /// so the app can repaint lazily (call once after creating the overlay).
    pub fn set_repaint_context(&mut self, ctx: &egui::Context) {
        self.host.set_repaint_context(ctx.clone());
    }

    pub fn close(&mut self) {
        self.host.close();
    }
}
