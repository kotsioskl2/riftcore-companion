//! Overlay surface host abstraction, diagnostics, shared render payload, and DX12 visual config.

use crate::config::settings::AppSettings;
use crate::domain::live_game::LiveGameState;
use crate::domain::protocol::LIVE_COACH_ENABLED;
use crate::overlay::eframe_fallback::EframeChildViewportFallback;
use crate::overlay::hud_modes::HudPresentationMode;
use eframe::egui::{self, Rect};
use eframe::egui_wgpu::{SurfaceConfig, WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew};
use eframe::wgpu;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Active rendering host implementation backing the transparent overlay surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayHostBackend {
    /// Dedicated Win32 HWND + DirectComposition (`DxgiFromVisual`) + isolated `egui_wgpu::Renderer`.
    PathBWin32DComp,
    /// Fallback `eframe` child-viewport / headless-context path if DirectComposition host init fails.
    PathAEframeFallback,
}

/// Runtime telemetry reported by the active overlay surface host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayDiagnostics {
    pub host_backend: OverlayHostBackend,
    pub adapter_backend: Option<wgpu::Backend>,
    pub adapter_name: String,
    pub supported_alpha_modes: Vec<wgpu::CompositeAlphaMode>,
    pub selected_alpha_mode: Option<wgpu::CompositeAlphaMode>,
    pub surface_format: Option<wgpu::TextureFormat>,
    pub hwnd: isize,
    pub ex_style: usize,
    pub frames_rendered: u64,
    pub resize_events: u64,
    pub passthrough_active: bool,
    pub rect_px: [i32; 4],
    pub last_surface_status: String,
    /// Times the HUD re-asserted its topmost z-order after the game window rose above it.
    pub topmost_reasserts: u64,
}

impl Default for OverlayDiagnostics {
    fn default() -> Self {
        Self {
            host_backend: OverlayHostBackend::PathBWin32DComp,
            adapter_backend: None,
            adapter_name: String::new(),
            supported_alpha_modes: Vec::new(),
            selected_alpha_mode: None,
            surface_format: None,
            hwnd: 0,
            ex_style: 0,
            frames_rendered: 0,
            resize_events: 0,
            passthrough_active: true,
            rect_px: [0, 0, 1, 1],
            last_surface_status: "uninitialized".to_string(),
            topmost_reasserts: 0,
        }
    }
}

/// Game clock that runs smoothly between live-data snapshots (about one per second).
///
/// It extrapolates from the latest snapshot, never runs backwards, catches up at most at double
/// speed instead of jumping, and stops after [`Self::MAX_LEAD_SECS`] without new data (paused
/// game or stalled feed). Only a jump beyond [`Self::SNAP_SECS`] (a new game, a reconnect)
/// moves it in one step.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameClock {
    sample: Option<ClockSample>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ClockSample {
    /// Game time the snapshot reported.
    game_secs: f64,
    /// When the snapshot arrived.
    at: Instant,
    /// Clock reading when the snapshot arrived; the clock moves on from here.
    from_secs: f64,
}

impl ClockSample {
    fn secs_at(&self, now: Instant) -> f64 {
        let dt = now.saturating_duration_since(self.at).as_secs_f64();
        let estimate = self.game_secs + dt.min(GameClock::MAX_LEAD_SECS);
        estimate.clamp(self.from_secs, self.from_secs + GameClock::MAX_RATE * dt)
    }
}

impl GameClock {
    /// How far past the last snapshot the clock may run before it waits for new data.
    /// Snapshots arrive about every 1.1 s (a 0.9 s poll on a 220 ms loop), so the lead covers
    /// that gap with margin; a paused game keeps sending the same time, and the clock then reads
    /// at most one second ahead.
    pub const MAX_LEAD_SECS: f64 = 1.5;
    /// Differences larger than this are a different game state, not jitter: snap to it.
    pub const SNAP_SECS: f64 = 3.0;
    /// Fastest catch-up rate (game seconds per real second).
    const MAX_RATE: f64 = 2.0;

    /// Feeds the game time of a snapshot that arrived at `now`; repeats of the same snapshot
    /// are ignored. Returns `true` when the time moved back by more than [`Self::SNAP_SECS`]
    /// (a new game), so per-game state such as manual timers should be cleared.
    pub fn observe(&mut self, game_secs: f64, now: Instant) -> bool {
        let game_secs = if game_secs.is_finite() {
            game_secs.max(0.0)
        } else {
            0.0
        };
        let Some(sample) = self.sample else {
            self.sample = Some(ClockSample {
                game_secs,
                at: now,
                from_secs: game_secs,
            });
            return false;
        };
        if (game_secs - sample.game_secs).abs() < 1e-3 {
            return false;
        }
        let shown = sample.secs_at(now);
        let snap = (game_secs - shown).abs() > Self::SNAP_SECS;
        self.sample = Some(ClockSample {
            game_secs,
            at: now,
            from_secs: if snap { game_secs } else { shown },
        });
        snap && game_secs < shown
    }

    pub fn reset(&mut self) {
        self.sample = None;
    }

    /// Smoothed game time at `now`; `None` before the first snapshot.
    pub fn secs_at(&self, now: Instant) -> Option<f64> {
        self.sample.map(|sample| sample.secs_at(now))
    }

    /// Time until the mm:ss reading changes (assuming normal speed), for repaint scheduling.
    pub fn until_next_second(&self, now: Instant) -> Option<Duration> {
        let secs = self.secs_at(now)?;
        Some(Duration::from_secs_f64(
            (1.0 - secs.fract()).clamp(0.05, 1.0),
        ))
    }
}

/// Active tab inside the manual live-timers overlay drawer.
///
/// Riot compliance: enemy summoner spell cooldown tracking is intentionally absent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum OverlayTimerTab {
    #[default]
    Camps,
    Objectives,
}

/// Projected overlay configuration extracted from [`AppSettings`].
#[derive(Clone, Debug, PartialEq)]
pub struct OverlayPanelSettings {
    pub hud_mode: HudPresentationMode,
    pub opacity: f32,
    pub hud_gold_lead: bool,
    pub hud_cs_benchmark: bool,
    pub hud_objective_timers: bool,
    pub hud_ally_ult_timers: bool,
    pub hud_manual_camp_timers: bool,
    pub hud_skill_order: bool,
    pub hud_item_goal: bool,
    pub hud_trinket_reminder: bool,
    pub hud_mayhem_cards: bool,
    pub hud_jungle_pathing: bool,
    pub hud_map_objective_pins: bool,
    pub hud_map_route_pins: bool,
    pub hud_map_ward_pins: bool,
    pub jungle_start_buff: String,
    pub hud_aram_health_timers: bool,
    pub hud_minimap_scale: f32,
    pub hud_minimap_flip: bool,
    pub hud_minimap_zoom: f32,
    pub hud_minimap_nudge_x: f32,
    pub hud_minimap_nudge_y: f32,
    pub hud_edit_layout: bool,
    pub hud_positions: HashMap<String, [f32; 2]>,
}

impl Default for OverlayPanelSettings {
    fn default() -> Self {
        Self {
            hud_mode: HudPresentationMode::Integrated,
            opacity: 0.90,
            hud_gold_lead: true,
            hud_cs_benchmark: true,
            hud_objective_timers: true,
            hud_ally_ult_timers: true,
            hud_manual_camp_timers: true,
            hud_skill_order: true,
            hud_item_goal: true,
            hud_trinket_reminder: true,
            hud_mayhem_cards: true,
            hud_jungle_pathing: true,
            hud_map_objective_pins: true,
            hud_map_route_pins: true,
            hud_map_ward_pins: false,
            jungle_start_buff: "red".to_string(),
            hud_aram_health_timers: true,
            hud_minimap_scale: 2.16,
            hud_minimap_flip: false,
            hud_minimap_zoom: 1.0,
            hud_minimap_nudge_x: 0.0,
            hud_minimap_nudge_y: 0.0,
            hud_edit_layout: false,
            hud_positions: HashMap::new(),
        }
    }
}

impl OverlayPanelSettings {
    /// Projects overlay-relevant toggles and coordinates from persisted [`AppSettings`].
    pub fn from_app_settings(s: &AppSettings) -> Self {
        Self {
            hud_mode: HudPresentationMode::parse_lossy(&s.hud_mode),
            opacity: 0.90,
            hud_gold_lead: s.hud_gold_lead,
            hud_cs_benchmark: s.hud_cs_benchmark,
            hud_objective_timers: s.hud_objective_timers,
            hud_ally_ult_timers: s.hud_ally_ult_timers,
            hud_manual_camp_timers: s.hud_manual_camp_timers,
            hud_skill_order: s.hud_skill_order,
            hud_item_goal: s.hud_item_goal,
            hud_trinket_reminder: s.hud_trinket_reminder,
            hud_mayhem_cards: s.hud_mayhem_cards,
            hud_jungle_pathing: s.hud_jungle_pathing,
            hud_map_objective_pins: s.hud_map_objective_pins,
            hud_map_route_pins: s.hud_map_route_pins,
            hud_map_ward_pins: s.hud_map_ward_pins,
            jungle_start_buff: s.jungle_start_buff.clone(),
            hud_aram_health_timers: s.hud_aram_health_timers,
            hud_minimap_scale: s.hud_minimap_scale,
            hud_minimap_flip: s.hud_minimap_flip,
            hud_minimap_zoom: s.hud_minimap_zoom,
            hud_minimap_nudge_x: s.hud_minimap_nudge_x,
            hud_minimap_nudge_y: s.hud_minimap_nudge_y,
            hud_edit_layout: s.hud_edit_layout,
            hud_positions: s.hud_positions.clone(),
        }
    }
}

/// Actions emitted by interactive overlay panels during a rendered frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OverlayFrameActions {
    pub updated_positions: Vec<(String, [f32; 2])>,
    pub request_close_overlay: bool,
    pub export_item_set: Option<Value>,
    pub copy_clipboard: Option<String>,
    pub notice: Option<String>,
}

/// How long the HUD keeps the last live snapshot when a poll returns nothing (busy game).
pub const LIVE_DATA_GRACE: Duration = Duration::from_secs(3);

/// Complete typed state rendered inside the isolated overlay `egui::Context`.
#[derive(Clone, Debug)]
pub struct OverlayRenderState {
    pub live_game: Option<LiveGameState>,
    /// Whole game seconds shown by the HUD (from [`Self::game_clock`], smooth between snapshots).
    pub clock_secs: u64,
    pub game_clock: GameClock,
    /// When the last live snapshot arrived.
    pub live_seen_at: Option<Instant>,
    pub preview_mode: bool,
    pub settings: OverlayPanelSettings,
    pub force_passthrough: bool,
    pub passthrough_active: bool,
    pub show_mayhem_cards: bool,
    pub show_build_panel: bool,
    pub show_timers_panel: bool,
    pub mayhem_mode: String,
    pub mayhem_tier: String,
    pub mayhem_search: String,
    pub timer_tab: OverlayTimerTab,
    pub aram_relic_seconds: String,
    /// Manual countdowns (camps, ally ults, ARAM relics): key -> game second they end at, so
    /// they follow the game clock (and pause with it).
    pub manual_timers: HashMap<String, f64>,
    /// Creep score snapshot taken as game time crosses 10:00 (`CS@10`).
    pub cs_at_10: Option<u32>,
    /// Active-game coaching flag; locked behind `crate::domain::protocol::LIVE_COACH_ENABLED`.
    pub live_coach_enabled: bool,
    pub host_diag: OverlayDiagnostics,
    pub pending_actions: OverlayFrameActions,
}

impl Default for OverlayRenderState {
    fn default() -> Self {
        Self {
            live_game: None,
            clock_secs: 0,
            game_clock: GameClock::default(),
            live_seen_at: None,
            preview_mode: false,
            settings: OverlayPanelSettings::default(),
            force_passthrough: false,
            passthrough_active: true,
            show_mayhem_cards: false,
            show_build_panel: false,
            show_timers_panel: false,
            mayhem_mode: "Both".to_string(),
            mayhem_tier: "ALL".to_string(),
            mayhem_search: String::new(),
            timer_tab: OverlayTimerTab::Camps,
            aram_relic_seconds: "90".to_string(),
            manual_timers: HashMap::new(),
            cs_at_10: None,
            live_coach_enabled: LIVE_COACH_ENABLED,
            host_diag: OverlayDiagnostics::default(),
            pending_actions: OverlayFrameActions::default(),
        }
    }
}

impl OverlayRenderState {
    /// Returns `true` only when `LIVE_COACH_ENABLED` is enabled in protocol AND toggled on.
    pub fn effective_live_coaching_enabled(&self) -> bool {
        LIVE_COACH_ENABLED && self.live_coach_enabled
    }

    /// Refreshes `live_game` and the game clock from the latest parsed Riot Live Client telemetry.
    pub fn apply_live_game(&mut self, live: Option<LiveGameState>) {
        self.apply_live_game_at(live, Instant::now());
    }

    /// [`Self::apply_live_game`] for a snapshot that arrived at `now`. Repeating the same
    /// snapshot is harmless; a missing one keeps the last state for [`LIVE_DATA_GRACE`].
    pub fn apply_live_game_at(&mut self, live: Option<LiveGameState>, now: Instant) {
        match live {
            Some(live) => {
                if self.game_clock.observe(live.game_time_secs, now) {
                    self.reset_match_state();
                }
                self.live_seen_at = Some(now);
                self.live_game = Some(live);
                self.refresh_clock(now);
            }
            None if self.live_game.is_some()
                && self
                    .live_seen_at
                    .is_some_and(|at| now.saturating_duration_since(at) < LIVE_DATA_GRACE) => {}
            None => {
                self.live_game = None;
                self.live_seen_at = None;
                self.game_clock.reset();
                self.clock_secs = 0;
                self.reset_match_state();
            }
        }
    }

    /// Advances `clock_secs` to the smoothed game time at `now` (no-op without live data).
    pub fn refresh_clock(&mut self, now: Instant) {
        if let Some(secs) = self.game_clock.secs_at(now) {
            self.clock_secs = secs as u64;
        }
    }

    /// Drops per-match state (manual timers, CS@10) when a game ends or a new one starts.
    fn reset_match_state(&mut self) {
        self.manual_timers.clear();
        self.cs_at_10 = None;
    }

    /// Clears transient per-frame action outputs before rendering the next frame.
    pub fn clear_pending_actions(&mut self) {
        self.pending_actions = OverlayFrameActions::default();
    }

    /// Computes a deterministic summary hash so `OverlayEngine::tick` re-renders immediately
    /// when telemetry or panel state updates without spinning at 30 FPS while stationary.
    pub fn render_signature(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.clock_secs.hash(&mut h);
        self.preview_mode.hash(&mut h);
        self.settings.hud_mode.hash(&mut h);
        ((self.settings.opacity * 1000.0).round() as u32).hash(&mut h);
        self.settings.hud_edit_layout.hash(&mut h);
        self.settings.hud_gold_lead.hash(&mut h);
        self.settings.hud_cs_benchmark.hash(&mut h);
        self.settings.hud_objective_timers.hash(&mut h);
        self.settings.hud_ally_ult_timers.hash(&mut h);
        self.settings.hud_manual_camp_timers.hash(&mut h);
        self.settings.hud_skill_order.hash(&mut h);
        self.settings.hud_item_goal.hash(&mut h);
        self.settings.hud_trinket_reminder.hash(&mut h);
        self.settings.hud_mayhem_cards.hash(&mut h);
        self.settings.hud_jungle_pathing.hash(&mut h);
        self.settings.hud_map_objective_pins.hash(&mut h);
        self.settings.hud_map_route_pins.hash(&mut h);
        self.settings.hud_map_ward_pins.hash(&mut h);
        self.settings.jungle_start_buff.hash(&mut h);
        self.settings.hud_aram_health_timers.hash(&mut h);
        self.settings.hud_minimap_flip.hash(&mut h);
        ((self.settings.hud_minimap_scale * 100.0).round() as i32).hash(&mut h);
        ((self.settings.hud_minimap_zoom * 100.0).round() as i32).hash(&mut h);
        self.passthrough_active.hash(&mut h);
        self.show_mayhem_cards.hash(&mut h);
        self.show_build_panel.hash(&mut h);
        self.show_timers_panel.hash(&mut h);
        self.mayhem_mode.hash(&mut h);
        self.mayhem_tier.hash(&mut h);
        self.mayhem_search.hash(&mut h);
        self.timer_tab.hash(&mut h);
        self.aram_relic_seconds.hash(&mut h);
        self.manual_timers.len().hash(&mut h);
        self.effective_live_coaching_enabled().hash(&mut h);
        if let Some(live) = self.live_game.as_ref() {
            live.game_mode.hash(&mut h);
            live.map_number.hash(&mut h);
            live.active_player.champion_name.hash(&mut h);
            live.active_player.level.hash(&mut h);
            live.active_player.scores.kills.hash(&mut h);
            live.active_player.scores.deaths.hash(&mut h);
            live.active_player.scores.assists.hash(&mut h);
            live.active_player.scores.creep_score.hash(&mut h);
            (live.active_player.current_gold as i64).hash(&mut h);
            live.active_player.items.len().hash(&mut h);
            for item in &live.active_player.items {
                item.item_id.hash(&mut h);
                item.count.hash(&mut h);
                item.slot.hash(&mut h);
            }
            live.active_player.abilities.q.ability_level.hash(&mut h);
            live.active_player.abilities.w.ability_level.hash(&mut h);
            live.active_player.abilities.e.ability_level.hash(&mut h);
            live.active_player.abilities.r.ability_level.hash(&mut h);
            live.events.len().hash(&mut h);
        }
        h.finish()
    }
}

/// Callback signature invoked inside the overlay surface host's isolated `egui::Context`.
pub type OverlayRenderCallback =
    Box<dyn FnMut(&mut egui::Ui, &mut OverlayRenderState, &mut Vec<Rect>) + Send>;

/// Hardware/window host interface separating Path B (`Win32OverlayHost`) from Path A fallback.
pub trait OverlaySurfaceHost {
    fn backend_kind(&self) -> OverlayHostBackend;

    /// Shows/hides the HUD window at `rect_px` (physical virtual-screen pixels), rendering at
    /// `pixels_per_point`, with click-through on or off. Never activates the window.
    fn sync_window(
        &mut self,
        visible: bool,
        rect_px: [i32; 4],
        pixels_per_point: f32,
        passthrough: bool,
    );

    /// Re-raises the visible HUD above `game_hwnd` without activating it when the game window
    /// has moved above the HUD in z-order (a Fullscreen game is topmost and rises whenever it
    /// is activated). Does nothing while the HUD is already above. Returns `true` if it raised.
    fn keep_above(&mut self, _game_hwnd: isize) -> bool {
        false
    }

    fn render_if_needed(
        &mut self,
        force_redraw: bool,
        state: &mut OverlayRenderState,
        render_ui: &mut dyn FnMut(&mut egui::Ui, &mut OverlayRenderState, &mut Vec<Rect>),
    ) -> Vec<Rect>;

    fn pointer_down(&self) -> bool {
        false
    }

    fn diagnostics(&self) -> OverlayDiagnostics;

    fn close(&mut self);
}

/// Selects the primary overlay surface host, falling back cleanly to Path A if initialization fails.
pub fn select_host_or_fallback<H: OverlaySurfaceHost + 'static>(
    primary: Result<H, String>,
) -> (Box<dyn OverlaySurfaceHost>, Option<String>) {
    match primary {
        Ok(host) => (Box::new(host), None),
        Err(err) => (Box::new(EframeChildViewportFallback::default()), Some(err)),
    }
}

/// Builds the DX12 `DxgiFromVisual` wgpu configuration required for Path B DirectComposition
/// transparency alongside an opaque root `eframe` window.
pub fn build_dx12_visual_wgpu_options() -> WgpuConfiguration {
    let mut create_new = WgpuSetupCreateNew::without_display_handle();
    create_new.instance_descriptor.backends = wgpu::Backends::DX12;
    create_new
        .instance_descriptor
        .backend_options
        .dx12
        .presentation_system = wgpu::Dx12SwapchainKind::DxgiFromVisual;

    WgpuConfiguration {
        surface: SurfaceConfig::LOW_LATENCY,
        wgpu_setup: WgpuSetup::CreateNew(create_new),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_clock_runs_smoothly_between_snapshots_without_jumps() {
        let t0 = Instant::now();
        let at = |secs: f64| t0 + Duration::from_secs_f64(secs);
        // Snapshots as the live client delivers them: ~1 s apart, repeated ~4x per second by
        // the app, one slightly behind, one ahead, then a stall (paused game).
        let feed = [
            (0.0, 600.0),
            (0.25, 600.0),
            (0.75, 600.0),
            (1.0, 600.9),
            (2.0, 603.0),
        ];
        let mut clock = GameClock::default();
        assert_eq!(clock.secs_at(t0), None);
        let mut shown: Option<f64> = None;
        let mut next_feed = 0;
        for step in 0..900 {
            let now = step as f64 * 0.01;
            while next_feed < feed.len() && feed[next_feed].0 <= now {
                assert!(!clock.observe(feed[next_feed].1, at(feed[next_feed].0)));
                next_feed += 1;
            }
            let secs = clock.secs_at(at(now)).expect("clock running");
            if let Some(prev) = shown {
                let delta = secs - prev;
                // Never backwards, never faster than 2x: no visible jump or skipped second.
                assert!(
                    (-1e-6..=0.02 + 1e-6).contains(&delta),
                    "{now}: {prev} -> {secs}"
                );
            }
            if step == 50 {
                // Runs on between snapshots instead of stalling at the last one.
                assert!((secs - 600.5).abs() < 1e-6, "{secs}");
            }
            shown = Some(secs);
        }
        // Without new data the clock stops 1.5 s past the last snapshot instead of running away.
        assert!((clock.secs_at(at(30.0)).expect("running") - 604.5).abs() < 1e-6);
        assert!(clock.until_next_second(at(30.0)).expect("running") <= Duration::from_secs(1));
        // A new game (time far back) snaps and asks for per-game state to be reset.
        assert!(clock.observe(30.0, at(31.0)));
        assert_eq!(clock.secs_at(at(31.0)), Some(30.0));
        // A reconnect far ahead snaps too, but is not a new game.
        assert!(!clock.observe(100.0, at(32.0)));
        assert_eq!(clock.secs_at(at(32.0)), Some(100.0));
    }

    #[test]
    fn hud_keeps_live_state_through_a_dropped_poll_and_resets_per_game_timers() {
        let t0 = Instant::now();
        let at = |secs: u64| t0 + Duration::from_secs(secs);
        let live = |game_time_secs: f64| LiveGameState {
            game_time_secs,
            ..LiveGameState::default()
        };
        let mut state = OverlayRenderState::default();
        state.apply_live_game_at(Some(live(1200.0)), t0);
        assert_eq!(state.clock_secs, 1200);
        state
            .manual_timers
            .insert("jungle-camp-gromp".into(), 1300.0);
        // One failed poll: no 00:00 flash, the clock keeps running on the last snapshot.
        state.apply_live_game_at(None, at(1));
        assert!(state.live_game.is_some());
        state.refresh_clock(at(1) + Duration::from_millis(500));
        assert_eq!(state.clock_secs, 1201);
        // Live data gone for longer than the grace period: the match is over.
        state.apply_live_game_at(None, at(5));
        assert!(state.live_game.is_none());
        assert_eq!(state.clock_secs, 0);
        assert!(state.manual_timers.is_empty());
        // A new game straight after another (time jumps back) drops the old timers too.
        state.apply_live_game_at(Some(live(1500.0)), at(6));
        state.manual_timers.insert("ally-ult-A1".into(), 1600.0);
        state.cs_at_10 = Some(80);
        state.apply_live_game_at(Some(live(20.0)), at(7));
        assert!(state.manual_timers.is_empty());
        assert_eq!(state.cs_at_10, None);
        assert_eq!(state.clock_secs, 20);
    }
}
