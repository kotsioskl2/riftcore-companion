//! Native external DirectComposition overlay engine and Riot-compliant HUD modules.

pub mod eframe_fallback;
pub mod engine;
pub mod host;
pub mod hud_modes;
pub mod minimap;
pub mod panels;
pub mod win32_host;

pub use eframe_fallback::{EframeChildViewportFallback, PathAEframeFallbackStub};
pub use engine::{
    clamp_panel_in_area, OverlayCommand, OverlayConfig, OverlayEngine, OverlayLifecycleState,
    PassthroughController, RepaintScheduler, TickInput,
};
pub use host::{
    build_dx12_visual_wgpu_options, select_host_or_fallback, OverlayDiagnostics,
    OverlayFrameActions, OverlayHostBackend, OverlayPanelSettings, OverlayRenderCallback,
    OverlayRenderState, OverlaySurfaceHost, OverlayTimerTab,
};
pub use hud_modes::{
    clamp_logical_panel_pos, compute_overlay_rect_px, infer_presentation_mode, monitor_for_rect,
    physical_cursor_to_logical, DisplayPresentationMode, HudPresentationMode, TargetWindowState,
};
pub use minimap::{
    build_minimap_guide_model, jungle_route_status, render_minimap_guide_pins, MinimapGuideModel,
};
pub use panels::{
    filter_mayhem_cards, missing_trinket_reminder, render_overlay_ui, resolve_personal_build,
    OVERLAY_ENEMY_SPELL_TIMERS_REMOVED,
};
pub use win32_host::Win32OverlayHost;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::live_game::{LiveGameState, ObjectiveKind};
    use crate::domain::protocol::LIVE_COACH_ENABLED;
    use crate::platform::window::MonitorMetrics;
    use eframe::egui::{Pos2, Rect, Vec2};
    use serde_json::json;
    use std::time::Duration;

    #[test]
    fn mixed_dpi_physical_rect_preserves_negative_and_secondary_origins() {
        let left_monitor = MonitorMetrics {
            rect_px: [-2560, 0, 0, 1440],
            work_px: [-2560, 0, 0, 1400],
            dpi: 144,
        };
        let target = TargetWindowState {
            hwnd: 42,
            client_rect_px: [-2560, 0, 0, 1440],
            monitor_dpi: 144,
            is_foreground: true,
            presentation: DisplayPresentationMode::Borderless,
        };

        // Every HUD style covers the game, one pixel inside it.
        let rect = compute_overlay_rect_px(&target, Some(&left_monitor));
        assert_eq!(rect, [-2560, 0, -1, 1439]);
    }

    #[test]
    fn degenerate_monitor_metrics_never_panic_on_clamp() {
        let degenerate = MonitorMetrics {
            rect_px: [100, 100, 100, 100],
            work_px: [100, 100, 100, 100],
            dpi: 96,
        };
        let target = TargetWindowState {
            hwnd: 99,
            client_rect_px: [0, 0, 1920, 1080],
            monitor_dpi: 96,
            is_foreground: true,
            presentation: DisplayPresentationMode::Borderless,
        };
        let rect = compute_overlay_rect_px(&target, Some(&degenerate));
        assert!(rect[2] > rect[0]);
        assert!(rect[3] > rect[1]);
    }

    #[test]
    fn cursor_physical_to_logical_scales_and_clamps_panel_pos() {
        let overlay_rect = [1920, 100, 3840, 1180];
        let pos_150 = physical_cursor_to_logical([1920 + 300, 100 + 150], overlay_rect, 1.5);
        assert!((pos_150.x - 200.0).abs() < 0.01);
        assert!((pos_150.y - 100.0).abs() < 0.01);

        let area = Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 720.0));
        let clamped = clamp_logical_panel_pos([2400.0, 950.0], area, Vec2::new(300.0, 150.0));
        assert_eq!(clamped, Pos2::new(980.0, 570.0));
    }

    #[test]
    fn passthrough_drag_lock_prevents_dropping_fast_drag() {
        let mut ctrl = PassthroughController::default();
        ctrl.set_interactive_rects(vec![Rect::from_min_size(
            Pos2::new(10.0, 10.0),
            Vec2::new(100.0, 40.0),
        )]);

        assert!(ctrl.evaluate(Some(Pos2::new(200.0, 200.0)), false, false));
        assert!(!ctrl.evaluate(Some(Pos2::new(50.0, 25.0)), true, false));
        // Cursor moves fast outside rect while pointer held down -> stays interactive
        assert!(!ctrl.evaluate(Some(Pos2::new(400.0, 400.0)), true, false));
        // Pointer released outside rect -> restores passthrough
        assert!(ctrl.evaluate(Some(Pos2::new(400.0, 400.0)), false, false));
        // Force passthrough overrides even inside rect
        assert!(ctrl.evaluate(Some(Pos2::new(50.0, 25.0)), true, true));
    }

    #[test]
    fn overlay_engine_fsm_fallback_and_stationary_pacing() {
        let (mut host, err) =
            select_host_or_fallback::<Win32OverlayHost>(Err("dx12 init failed".to_string()));
        assert_eq!(host.backend_kind(), OverlayHostBackend::PathAEframeFallback);
        assert_eq!(err.as_deref(), Some("dx12 init failed"));

        let mut engine = OverlayEngine::default();
        let mut cfg = OverlayConfig::default();

        engine.on_live_game_state(true, &mut cfg);
        assert_eq!(
            engine.lifecycle_state(),
            OverlayLifecycleState::WaitingForGameWindow
        );

        // League's Fullscreen mode shows the HUD just like Borderless.
        engine.apply_command(
            OverlayCommand::UpdateTarget(Some(TargetWindowState {
                hwnd: 1,
                client_rect_px: [0, 0, 1920, 1080],
                monitor_dpi: 96,
                is_foreground: true,
                presentation: DisplayPresentationMode::ExclusiveFullscreen,
            })),
            &mut cfg,
        );
        assert_eq!(
            engine.lifecycle_state(),
            OverlayLifecycleState::ActiveVisible
        );

        engine.apply_command(
            OverlayCommand::UpdateTarget(Some(TargetWindowState {
                hwnd: 1,
                client_rect_px: [0, 0, 1920, 1080],
                monitor_dpi: 96,
                is_foreground: false,
                presentation: DisplayPresentationMode::Borderless,
            })),
            &mut cfg,
        );
        assert_eq!(
            engine.lifecycle_state(),
            OverlayLifecycleState::WaitingForForeground
        );

        engine.apply_command(
            OverlayCommand::UpdateTarget(Some(TargetWindowState {
                hwnd: 1,
                client_rect_px: [0, 0, 1920, 1080],
                monitor_dpi: 96,
                is_foreground: true,
                presentation: DisplayPresentationMode::Borderless,
            })),
            &mut cfg,
        );
        assert_eq!(
            engine.lifecycle_state(),
            OverlayLifecycleState::ActiveVisible
        );

        let mut state = OverlayRenderState {
            clock_secs: 42,
            ..Default::default()
        };
        let (lifecycle, next) = engine.tick(
            host.as_mut(),
            TickInput {
                monitor: None,
                cursor_px: None,
                pointer_down: false,
                config: &cfg,
                render_state: &mut state,
                force_interactive_redraw: false,
            },
            &mut |ui, s, r| render_overlay_ui(ui, s, r),
        );
        assert_eq!(lifecycle, OverlayLifecycleState::ActiveVisible);
        assert_eq!(next, Some(Duration::from_millis(1000)));
        assert_eq!(host.diagnostics().frames_rendered, 1);

        // Repeated tick with unchanged stationary state skips redundant re-render
        let _ = engine.tick(
            host.as_mut(),
            TickInput {
                monitor: None,
                cursor_px: None,
                pointer_down: false,
                config: &cfg,
                render_state: &mut state,
                force_interactive_redraw: false,
            },
            &mut |ui, s, r| render_overlay_ui(ui, s, r),
        );
        assert_eq!(host.diagnostics().frames_rendered, 1);

        // Manual user dismissal stays remembered while match continues, then resets when match ends
        engine.apply_command(
            OverlayCommand::SetEnabled {
                enabled: false,
                automatic: false,
            },
            &mut cfg,
        );
        engine.on_live_game_state(true, &mut cfg);
        assert!(!engine.is_enabled());
        engine.on_live_game_state(false, &mut cfg);
        engine.on_live_game_state(true, &mut cfg);
        assert!(engine.is_enabled());
    }

    /// Records what the engine asks of the HUD window; `game_above` simulates a Fullscreen
    /// League window that rose above the HUD (it is topmost too and rises on activation).
    #[derive(Default)]
    struct RecordingHost {
        syncs: Vec<(bool, [i32; 4], f32)>,
        frames: u64,
        game_above: bool,
        raises: u64,
    }

    impl OverlaySurfaceHost for RecordingHost {
        fn backend_kind(&self) -> OverlayHostBackend {
            OverlayHostBackend::PathAEframeFallback
        }

        fn sync_window(&mut self, visible: bool, rect_px: [i32; 4], ppp: f32, _pass: bool) {
            self.syncs.push((visible, rect_px, ppp));
        }

        fn keep_above(&mut self, game_hwnd: isize) -> bool {
            assert_eq!(game_hwnd, 7, "keeps above the League window");
            let raise = std::mem::take(&mut self.game_above);
            self.raises += u64::from(raise);
            raise
        }

        fn render_if_needed(
            &mut self,
            force_redraw: bool,
            _state: &mut OverlayRenderState,
            _render_ui: &mut dyn FnMut(
                &mut eframe::egui::Ui,
                &mut OverlayRenderState,
                &mut Vec<Rect>,
            ),
        ) -> Vec<Rect> {
            self.frames += u64::from(force_redraw);
            Vec::new()
        }

        fn diagnostics(&self) -> OverlayDiagnostics {
            OverlayDiagnostics {
                frames_rendered: self.frames,
                topmost_reasserts: self.raises,
                ..OverlayDiagnostics::default()
            }
        }

        fn close(&mut self) {}
    }

    #[test]
    fn fullscreen_hud_shows_stays_on_top_hides_on_alt_tab_and_returns_at_new_size() {
        let fullscreen = |size: [i32; 2], dpi: u32, foreground: bool| TargetWindowState {
            hwnd: 7,
            client_rect_px: [0, 0, size[0], size[1]],
            monitor_dpi: dpi,
            is_foreground: foreground,
            presentation: DisplayPresentationMode::ExclusiveFullscreen,
        };
        let mut engine = OverlayEngine::default();
        let mut cfg = OverlayConfig::default();
        let mut host = RecordingHost::default();
        let mut state = OverlayRenderState::default();
        let mut tick = |engine: &mut OverlayEngine, host: &mut RecordingHost| {
            let cfg = OverlayConfig::default();
            engine
                .tick(
                    host,
                    TickInput {
                        monitor: None,
                        cursor_px: None,
                        pointer_down: false,
                        config: &cfg,
                        render_state: &mut state,
                        force_interactive_redraw: false,
                    },
                    &mut |_, _, _| {},
                )
                .0
        };
        engine.on_live_game_state(true, &mut cfg);

        // 1. Fullscreen 2560x1440 at 150 %: visible, full size minus the 1 px inset.
        let target = Some(fullscreen([2560, 1440], 144, true));
        engine.apply_command(OverlayCommand::UpdateTarget(target), &mut cfg);
        assert_eq!(
            tick(&mut engine, &mut host),
            OverlayLifecycleState::ActiveVisible
        );
        assert_eq!(host.syncs.last(), Some(&(true, [0, 0, 2559, 1439], 1.5)));
        assert_eq!(host.frames, 1);

        // 2. League was activated and rose above the HUD: re-raised once, and not again
        //    while the HUD stays on top (no z-order calls when nothing changed).
        host.game_above = true;
        tick(&mut engine, &mut host);
        tick(&mut engine, &mut host);
        assert_eq!(host.raises, 1);
        assert_eq!(host.frames, 1, "stationary HUD is not re-rendered");

        // 3. Alt-tab: League minimised (window gone) or in the background -> hidden.
        engine.apply_command(OverlayCommand::UpdateTarget(None), &mut cfg);
        assert_eq!(
            tick(&mut engine, &mut host),
            OverlayLifecycleState::WaitingForGameWindow
        );
        assert!(!host.syncs.last().expect("synced").0);
        let target = Some(fullscreen([2560, 1440], 144, false));
        engine.apply_command(OverlayCommand::UpdateTarget(target), &mut cfg);
        assert_eq!(
            tick(&mut engine, &mut host),
            OverlayLifecycleState::WaitingForForeground
        );
        assert!(!host.syncs.last().expect("synced").0);
        assert_eq!(host.raises, 1, "a hidden HUD never touches the z-order");

        // 4. Back in game at 1920x1080 (fresh monitor DPI 200 %): the first visible frame is
        //    re-rendered at the new size, with the HUD scale capped to a 1280x720 canvas.
        let target = Some(fullscreen([1920, 1080], 192, true));
        engine.apply_command(OverlayCommand::UpdateTarget(target), &mut cfg);
        assert_eq!(
            tick(&mut engine, &mut host),
            OverlayLifecycleState::ActiveVisible
        );
        assert_eq!(host.syncs.last(), Some(&(true, [0, 0, 1919, 1079], 1.5)));
        assert_eq!(host.frames, 2);

        // 5. Even with an unchanged size and state, coming back always renders a fresh frame.
        engine.apply_command(OverlayCommand::UpdateTarget(None), &mut cfg);
        tick(&mut engine, &mut host);
        let target = Some(fullscreen([1920, 1080], 192, true));
        engine.apply_command(OverlayCommand::UpdateTarget(target), &mut cfg);
        tick(&mut engine, &mut host);
        assert_eq!(host.frames, 3);
    }

    #[test]
    fn compliance_gates_sr_objectives_trinkets_and_mayhem_ranks() {
        const { assert!(!LIVE_COACH_ENABLED) };
        const { assert!(OVERLAY_ENEMY_SPELL_TIMERS_REMOVED) };

        // Even if runtime toggle were flipped, protocol gate locks active coaching off
        let state = OverlayRenderState {
            live_coach_enabled: true,
            ..OverlayRenderState::default()
        };
        assert!(!state.effective_live_coaching_enabled());

        // Timer drawer exposes only Camps & Objectives (zero enemy spell cooldown tab)
        assert_ne!(
            std::mem::discriminant(&OverlayTimerTab::Camps),
            std::mem::discriminant(&OverlayTimerTab::Objectives)
        );

        let sr_payload = json!({
            "gameData": { "gameTime": 1260.0, "gameMode": "CLASSIC", "mapNumber": 11 },
            "activePlayer": {
                "summonerName": "PlayerOne",
                "championName": "Ahri",
                "scores": { "kills": 3, "deaths": 1, "assists": 4, "creepScore": 140 },
                "currentGold": 1500.0,
                "items": [{ "itemID": 6655, "slot": 0, "count": 1, "displayName": "Luden's Companion" }]
            },
            "allPlayers": [
                {
                    "summonerName": "PlayerOne",
                    "championName": "Ahri",
                    "team": "ORDER",
                    "position": "MIDDLE",
                    "items": [{ "itemID": 6655, "slot": 0, "count": 1, "displayName": "Luden's Companion" }]
                }
            ],
            "events": { "Events": [] }
        });
        let sr = LiveGameState::from_allgamedata(&sr_payload).expect("valid SR payload");
        assert!(missing_trinket_reminder(Some(&sr)));
        let (champ, build) = resolve_personal_build(&sr).expect("build profile for Ahri");
        assert_eq!(champ.name, "Ahri");
        assert!(!build.core_items.is_empty());

        let area = Rect::from_min_size(Pos2::ZERO, Vec2::new(1920.0, 1080.0));
        let sr_minimap =
            build_minimap_guide_model(area, &sr, &OverlayPanelSettings::default(), 1260)
                .expect("SR minimap model");
        let baron_pins = sr_minimap
            .objective_pins
            .iter()
            .filter(|p| p.kind == ObjectiveKind::Baron)
            .count();
        assert_eq!(baron_pins, 1);

        let aram_payload = json!({
            "gameData": { "gameTime": 600.0, "gameMode": "ARAM", "mapNumber": 12 },
            "activePlayer": {
                "summonerName": "PlayerOne",
                "championName": "Ahri",
                "items": [{ "itemID": 3348, "slot": 6, "count": 1 }]
            },
            "allPlayers": [],
            "events": { "Events": [] }
        });
        let aram = LiveGameState::from_allgamedata(&aram_payload).expect("valid ARAM payload");
        assert!(!missing_trinket_reminder(Some(&aram)));
        assert!(
            build_minimap_guide_model(area, &aram, &OverlayPanelSettings::default(), 600,)
                .is_none()
        );

        let mayhem = filter_mayhem_cards("Both", "Prismatic", "");
        assert!(!mayhem.is_empty());
        let serialized = serde_json::to_value(mayhem[0]).expect("serialize card");
        assert!(!serialized.as_object().unwrap().contains_key("rank"));
    }
}
