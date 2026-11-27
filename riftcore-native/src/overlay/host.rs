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
use std::time::Instant;

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
        }
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

/// Complete typed state rendered inside the isolated overlay `egui::Context`.
#[derive(Clone, Debug)]
pub struct OverlayRenderState {
    pub live_game: Option<LiveGameState>,
    pub clock_secs: u64,
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
    pub manual_timers: HashMap<String, Instant>,
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

    /// Refreshes `live_game` and `clock_secs` from the latest parsed Riot Live Client telemetry.
    pub fn apply_live_game(&mut self, live: Option<LiveGameState>) {
        self.clock_secs = live
            .as_ref()
            .map(|g| g.game_time_secs.max(0.0) as u64)
            .unwrap_or(0);
        self.live_game = live;
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

    fn sync_window(
        &mut self,
        visible: bool,
        rect_px: [i32; 4],
        monitor_dpi: u32,
        passthrough: bool,
    );

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
