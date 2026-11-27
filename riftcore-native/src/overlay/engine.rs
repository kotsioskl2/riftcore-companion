//! Core overlay lifecycle state machine, hit-test passthrough controller, and frame scheduler.

use crate::overlay::host::{OverlayDiagnostics, OverlayRenderState, OverlaySurfaceHost};
use crate::overlay::hud_modes::{
    clamp_logical_panel_pos, compute_overlay_rect_px, physical_cursor_to_logical,
    DisplayPresentationMode, HudPresentationMode, TargetWindowState,
};
use crate::platform::window::MonitorMetrics;
use eframe::egui::{self, Pos2, Rect, Vec2};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// High-level overlay lifecycle state driven by user toggles and target window status.
///
/// League's Fullscreen mode is shown like Borderless: Windows 10/11 compose Fullscreen games,
/// so the topmost no-activate HUD window stays visible over them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum OverlayLifecycleState {
    #[default]
    Disabled,
    WaitingForGameWindow,
    WaitingForForeground,
    ActiveVisible,
}

/// Overlay user configuration independent of live game telemetry.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlayConfig {
    pub mode: HudPresentationMode,
    pub opacity: f32,
    pub edit_layout: bool,
    pub force_passthrough: bool,
    pub mayhem_open: bool,
    pub positions: HashMap<String, [f32; 2]>,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            mode: HudPresentationMode::Integrated,
            opacity: 0.90,
            edit_layout: false,
            force_passthrough: false,
            mayhem_open: false,
            positions: HashMap::new(),
        }
    }
}

/// Commands applied to [`OverlayEngine`] from global hotkeys, UI buttons, or window trackers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OverlayCommand {
    SetEnabled { enabled: bool, automatic: bool },
    ToggleEnabled,
    TogglePassthrough,
    UpdateTarget(Option<TargetWindowState>),
}

/// Interactive region hit-tester with active-drag lock to prevent dropping fast drags.
#[derive(Debug, Default)]
pub struct PassthroughController {
    interactive_rects: Vec<Rect>,
    drag_locked: bool,
    last_applied: Option<bool>,
}

impl PassthroughController {
    pub fn set_interactive_rects(&mut self, rects: Vec<Rect>) {
        self.interactive_rects = rects;
    }

    pub fn interactive_rects(&self) -> &[Rect] {
        &self.interactive_rects
    }

    /// Computes whether mouse passthrough (`WS_EX_TRANSPARENT | WS_EX_LAYERED`) should be active.
    /// Locks passthrough OFF during an active pointer drag so fast drags never drop mid-flight.
    pub fn evaluate(
        &mut self,
        cursor_logical: Option<Pos2>,
        pointer_down: bool,
        force_passthrough: bool,
    ) -> bool {
        if force_passthrough {
            self.drag_locked = false;
            return true;
        }
        let inside = cursor_logical
            .map(|pt| self.interactive_rects.iter().any(|r| r.contains(pt)))
            .unwrap_or(false);

        if pointer_down && (inside || self.drag_locked) {
            self.drag_locked = true;
            return false;
        }
        if !pointer_down {
            self.drag_locked = false;
        }
        !inside
    }

    pub fn should_apply(&mut self, desired: bool) -> bool {
        if self.last_applied == Some(desired) {
            false
        } else {
            self.last_applied = Some(desired);
            true
        }
    }

    pub fn current(&self) -> bool {
        self.last_applied.unwrap_or(true)
    }
}

/// Determines the next repaint delay from lifecycle and interaction state (`1s` idle, `16ms` active).
#[derive(Debug, Default)]
pub struct RepaintScheduler {
    last_clock_sec: Option<u64>,
}

impl RepaintScheduler {
    /// Returns the optimal next repaint interval, avoiding 33ms spinning when stationary or hidden.
    pub fn next_repaint_after(
        &mut self,
        state: OverlayLifecycleState,
        interacting_or_hovering: bool,
        clock_sec: u64,
    ) -> Option<Duration> {
        match state {
            OverlayLifecycleState::Disabled
            | OverlayLifecycleState::WaitingForGameWindow
            | OverlayLifecycleState::WaitingForForeground => None,
            OverlayLifecycleState::ActiveVisible => {
                self.last_clock_sec = Some(clock_sec);
                if interacting_or_hovering {
                    Some(Duration::from_millis(16))
                } else {
                    Some(Duration::from_millis(1000))
                }
            }
        }
    }

    pub fn clock_changed(&mut self, clock_sec: u64) -> bool {
        if self.last_clock_sec != Some(clock_sec) {
            self.last_clock_sec = Some(clock_sec);
            true
        } else {
            false
        }
    }
}

/// Bundled per-tick inputs for [`OverlayEngine::tick`].
pub struct TickInput<'a> {
    pub monitor: Option<&'a MonitorMetrics>,
    pub cursor_px: Option<[i32; 2]>,
    pub pointer_down: bool,
    pub config: &'a OverlayConfig,
    pub render_state: &'a mut OverlayRenderState,
    pub force_interactive_redraw: bool,
}

/// Coordinates lifecycle transitions, physical rect placement, passthrough toggling,
/// and selective GPU frame rendering across Path B (`Win32OverlayHost`) and Path A fallback.
#[derive(Debug, Default)]
pub struct OverlayEngine {
    enabled: bool,
    auto_entered: bool,
    dismissed_by_user: bool,
    target: Option<TargetWindowState>,
    passthrough: PassthroughController,
    scheduler: RepaintScheduler,
    last_render_signature: Option<u64>,
    last_render_rect: Option<[i32; 4]>,
    last_diagnostics: Option<OverlayDiagnostics>,
}

impl OverlayEngine {
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn target(&self) -> Option<&TargetWindowState> {
        self.target.as_ref()
    }

    pub fn last_diagnostics(&self) -> Option<&OverlayDiagnostics> {
        self.last_diagnostics.as_ref()
    }

    pub fn lifecycle_state(&self) -> OverlayLifecycleState {
        if !self.enabled {
            return OverlayLifecycleState::Disabled;
        }
        let Some(target) = self.target.as_ref() else {
            return OverlayLifecycleState::WaitingForGameWindow;
        };
        // Alt-tab out of Fullscreen minimises League (or leaves it in the background): hide.
        if !target.is_foreground || target.presentation == DisplayPresentationMode::Minimized {
            return OverlayLifecycleState::WaitingForForeground;
        }
        OverlayLifecycleState::ActiveVisible
    }

    pub fn apply_command(&mut self, cmd: OverlayCommand, config: &mut OverlayConfig) {
        match cmd {
            OverlayCommand::SetEnabled { enabled, automatic } => {
                if self.enabled == enabled {
                    return;
                }
                if !automatic {
                    self.dismissed_by_user = !enabled;
                } else if !enabled {
                    self.dismissed_by_user = false;
                }
                self.enabled = enabled;
                self.auto_entered = automatic && enabled;
                if !enabled {
                    self.last_render_signature = None;
                }
            }
            OverlayCommand::ToggleEnabled => {
                let next = !self.enabled;
                self.apply_command(
                    OverlayCommand::SetEnabled {
                        enabled: next,
                        automatic: false,
                    },
                    config,
                );
            }
            OverlayCommand::TogglePassthrough => {
                if self.enabled {
                    config.force_passthrough = !config.force_passthrough;
                }
            }
            OverlayCommand::UpdateTarget(target) => {
                self.target = target;
            }
        }
    }

    /// Auto-shows the overlay when a live match begins unless the user explicitly dismissed it
    /// during the current match, and auto-hides it on match exit if it was auto-entered.
    pub fn on_live_game_state(&mut self, live: bool, config: &mut OverlayConfig) {
        if live && !self.enabled && !self.dismissed_by_user {
            self.apply_command(
                OverlayCommand::SetEnabled {
                    enabled: true,
                    automatic: true,
                },
                config,
            );
        } else if !live && self.enabled && self.auto_entered {
            self.apply_command(
                OverlayCommand::SetEnabled {
                    enabled: false,
                    automatic: true,
                },
                config,
            );
        }
        if !live {
            self.dismissed_by_user = false;
        }
    }

    /// Synchronizes window bounds/visibility/passthrough and renders a new GPU frame only when
    /// interacting, dirty, resized, or telemetry/clock changed.
    pub fn tick(
        &mut self,
        host: &mut dyn OverlaySurfaceHost,
        input: TickInput<'_>,
        render_ui: &mut dyn FnMut(&mut egui::Ui, &mut OverlayRenderState, &mut Vec<Rect>),
    ) -> (OverlayLifecycleState, Option<Duration>) {
        let TickInput {
            monitor,
            cursor_px,
            pointer_down,
            config,
            render_state,
            force_interactive_redraw,
        } = input;

        render_state.clear_pending_actions();
        let now = Instant::now();
        render_state.refresh_clock(now);
        let state = self.lifecycle_state();
        let Some(target) = self
            .target
            .as_ref()
            .filter(|_| state == OverlayLifecycleState::ActiveVisible)
        else {
            host.sync_window(false, [0, 0, 1, 1], 1.0, true);
            let _ = self.passthrough.should_apply(true);
            render_state.passthrough_active = true;
            // Whatever was on screen is stale: the first visible frame is always re-rendered
            // at the size and scale current at that moment (alt-tab back, resolution change).
            self.last_render_signature = None;
            self.last_render_rect = None;
            self.passthrough.set_interactive_rects(Vec::new());
            self.last_diagnostics = Some(host.diagnostics());
            return (
                state,
                self.scheduler
                    .next_repaint_after(state, false, render_state.clock_secs),
            );
        };

        let rect_px = compute_overlay_rect_px(target, monitor, config.mode, config.mayhem_open);
        let ppp = target.hud_pixels_per_point();
        let cursor_logical = cursor_px.map(|c| physical_cursor_to_logical(c, rect_px, ppp));
        let desired_passthrough =
            self.passthrough
                .evaluate(cursor_logical, pointer_down, config.force_passthrough);
        let passthrough_changed = self.passthrough.should_apply(desired_passthrough);

        render_state.force_passthrough = config.force_passthrough;
        render_state.passthrough_active = desired_passthrough;
        host.sync_window(true, rect_px, ppp, desired_passthrough);
        // A Fullscreen League window is topmost too and rises above the HUD when activated.
        host.keep_above(target.hwnd);

        let signature = render_state.render_signature();
        let dirty = force_interactive_redraw
            || !desired_passthrough
            || passthrough_changed
            || self.last_render_signature != Some(signature)
            || self.last_render_rect != Some(rect_px);

        if dirty {
            let rects = host.render_if_needed(true, render_state, render_ui);
            self.passthrough.set_interactive_rects(rects);
            self.last_render_signature = Some(signature);
            self.last_render_rect = Some(rect_px);
        }

        self.last_diagnostics = Some(host.diagnostics());
        let interacting_or_hovering = !desired_passthrough || force_interactive_redraw;
        let next = self
            .scheduler
            .next_repaint_after(state, interacting_or_hovering, render_state.clock_secs)
            .map(|delay| {
                // Wake for the next game-clock second so timers tick on time, not up to 1 s late.
                render_state
                    .game_clock
                    .until_next_second(now)
                    .map_or(delay, |tick| delay.min(tick))
            });
        (state, next)
    }
}

/// Re-exports geometry helpers cleanly for engine consumers.
pub fn clamp_panel_in_area(pos: [f32; 2], area: Rect, panel_size: Vec2) -> Pos2 {
    clamp_logical_panel_pos(pos, area, panel_size)
}
