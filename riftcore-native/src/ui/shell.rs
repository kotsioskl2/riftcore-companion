//! Modular application shell (Wave 3).
//!
//! Owns the settings store, background worker, page state, textures and the Path B
//! Win32/DirectComposition overlay, and drives the `ui::chrome` + `ui::pages` render tree.
//! The legacy monolith (`app::RiftCoreApp` in the binary) remains the default runtime and
//! the fallback; this shell is opt-in until it is verified on Windows.

use crate::config::settings::SettingsStore;
use crate::domain::champions;
use crate::domain::live_game::{LiveGameLatch, LiveGameState};
use crate::domain::protocol::{CoreCommand, CoreEvent, CoreNotice, CoreSnapshot, GameflowPhase};
use crate::overlay::engine::{OverlayCommand, OverlayConfig, OverlayEngine, TickInput};
use crate::overlay::host::{
    select_host_or_fallback, OverlayPanelSettings, OverlayRenderState, OverlaySurfaceHost,
};
use crate::overlay::hud_modes::{monitor_for_rect, TargetWindowState};
use crate::overlay::panels::render_overlay_ui;
use crate::overlay::win32_host::Win32OverlayHost;
use crate::platform::hotkeys::{start_hotkey_listener, HotkeyEvent, HotkeyHandle};
use crate::platform::tray::{TrayAction, TrayController};
use crate::platform::window::{cursor_screen_position, query_monitors, MonitorMetrics};
use crate::services::worker::{BackgroundWorker, WindowTrackerState};
use crate::ui::chrome;
use crate::ui::pages::{
    champ_select_local_champion, select_champion_setup_inner, AppUiState, PageContext,
    PendingUiEffects,
};
use crate::ui::textures::{logo_rgba, TextureManager};
use crate::ui::theme::{apply_native_theme, install_font_family, ThemePalette};
use eframe::egui;
use serde_json::Value;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Idle repaint cadence when nothing else asks for a sooner frame.
const IDLE_REPAINT: Duration = Duration::from_millis(250);
/// Monitor layout changes rarely; re-query at most this often.
const MONITOR_REFRESH: Duration = Duration::from_secs(2);

/// Which overlay surface the shell ended up with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OverlayHostStatus {
    PathB,
    /// Path B failed; the Path A stand-in keeps lifecycle state but does not draw on screen.
    Fallback(String),
}

pub struct RiftCoreShell {
    store: SettingsStore,
    snapshot: CoreSnapshot,
    palette: ThemePalette,
    textures: TextureManager,
    state: AppUiState,
    worker: BackgroundWorker,
    hotkey_rx: Receiver<HotkeyEvent>,
    hotkeys: HotkeyHandle,
    tray: TrayController,

    overlay_engine: OverlayEngine,
    overlay_config: OverlayConfig,
    overlay_render: OverlayRenderState,
    overlay_host: Box<dyn OverlaySurfaceHost>,
    overlay_status: OverlayHostStatus,
    window: WindowTrackerState,
    monitors: Vec<MonitorMetrics>,
    monitors_at: Option<Instant>,
    live_game_latch: LiveGameLatch,
}

impl RiftCoreShell {
    /// Builds the shell. Fails (so `main` can fall back to the legacy app) only if the
    /// background worker thread cannot be spawned.
    pub fn new(creation: &eframe::CreationContext<'_>) -> Result<Self, String> {
        let egui_ctx = creation.egui_ctx.clone();
        let mut store = SettingsStore::load_default();
        store.settings.hud_preview_mode = false;
        let palette = apply_native_theme(&egui_ctx, &store.settings.theme);
        install_font_family(&egui_ctx, &store.settings.font_family);

        let repaint_ctx = egui_ctx.clone();
        let worker = BackgroundWorker::start(Arc::new(move || repaint_ctx.request_repaint()))
            .map_err(|err| format!("background worker failed to start: {err}"))?;
        let _ = worker.try_send_command(CoreCommand::SetAutoAccept(store.settings.auto_accept));

        let (hotkey_rx, hotkeys) = start_hotkey_listener();
        let tray = TrayController::new("RiftCore Companion", logo_rgba(32))
            .unwrap_or_else(|_| TrayController::unavailable());

        let mut state = AppUiState::default();
        state.pinned = store.settings.always_on_top;
        state.tray_available = tray.is_available();
        state.show_whats_new = store.settings.last_seen_version != env!("CARGO_PKG_VERSION");
        state.show_quick_tour = !state.show_whats_new && !store.settings.first_run_tour_seen;
        for warning in store.warnings.drain(..) {
            state.set_notice(CoreNotice::Warning(warning));
        }
        if store.settings.always_on_top {
            egui_ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                egui::viewport::WindowLevel::AlwaysOnTop,
            ));
        }

        let primary = match creation.wgpu_render_state.clone() {
            Some(render_state) => Win32OverlayHost::new(render_state),
            None => Err("wgpu render state unavailable (renderer is not wgpu)".to_string()),
        };
        let (overlay_host, host_error) = select_host_or_fallback(primary);
        let overlay_status = match host_error {
            None => OverlayHostStatus::PathB,
            Some(err) => {
                state.set_notice(CoreNotice::OverlayError(format!(
                    "In-game HUD unavailable (Path B init failed: {err}). Restart with RIFTCORE_UI=legacy to use the classic overlay."
                )));
                OverlayHostStatus::Fallback(err)
            }
        };

        let mut shell = Self {
            store,
            snapshot: CoreSnapshot::default(),
            palette,
            textures: TextureManager::new(),
            state,
            worker,
            hotkey_rx,
            hotkeys,
            tray,
            overlay_engine: OverlayEngine::default(),
            overlay_config: OverlayConfig::default(),
            overlay_render: OverlayRenderState::default(),
            overlay_host,
            overlay_status,
            window: WindowTrackerState::default(),
            monitors: Vec::new(),
            monitors_at: None,
            live_game_latch: LiveGameLatch::default(),
        };
        shell.sync_overlay_config();
        Ok(shell)
    }

    pub fn overlay_status(&self) -> &OverlayHostStatus {
        &self.overlay_status
    }

    fn save_settings(&mut self) {
        if let Err(err) = self.store.save() {
            self.state.set_notice(CoreNotice::Warning(format!(
                "Could not save settings: {err}"
            )));
        }
    }

    fn sync_overlay_config(&mut self) {
        let s = &self.store.settings;
        self.overlay_config.edit_layout = s.hud_edit_layout;
        self.overlay_config.positions.clone_from(&s.hud_positions);
        self.overlay_config.force_passthrough = self.state.overlay_mouse_passthrough;
    }

    fn set_overlay(&mut self, enabled: bool, automatic: bool) {
        self.overlay_engine.apply_command(
            OverlayCommand::SetEnabled { enabled, automatic },
            &mut self.overlay_config,
        );
        self.state.overlay_mode = self.overlay_engine.is_enabled();
        self.state.overlay_auto_entered = enabled && automatic;
        self.state.overlay_dismissed_by_user = !enabled && !automatic;
    }

    fn handle_tray_and_close(&mut self, ctx: &egui::Context) {
        for action in self.tray.poll_actions() {
            match action {
                TrayAction::ShowWindow => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                TrayAction::ToggleHud => {
                    let next = !self.overlay_engine.is_enabled();
                    self.set_overlay(next, false);
                }
                TrayAction::ExitApp => {
                    self.state.force_exit = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }

        let close_requested = ctx.input(|input| input.viewport().close_requested());
        if close_requested && !self.state.force_exit {
            if !self.store.settings.close_confirmation_seen {
                self.state.show_close_confirmation = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            } else if self.store.settings.close_to_tray {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                if self.state.tray_available {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }
            }
        }
    }

    fn drain_worker(&mut self, ctx: &egui::Context) {
        let mut commands = Vec::new();
        while let Some(snapshot) = self.worker.try_recv_snapshot() {
            if snapshot.riot_lookup.is_some() {
                self.state.profile.lookup_loading = false;
            }
            if let Some(notice) = snapshot.notice.clone() {
                if self.snapshot.notice.as_ref() != Some(&notice) {
                    self.state.set_notice(notice);
                }
            }
            self.snapshot = snapshot;
            self.state.parsed_live_game = self
                .snapshot
                .live_game
                .as_ref()
                .and_then(LiveGameState::from_allgamedata);

            if self.snapshot.phase == GameflowPhase::ChampSelect {
                let lcu_champion = self
                    .snapshot
                    .champ_select
                    .as_ref()
                    .and_then(champ_select_local_champion)
                    .unwrap_or(0);
                if lcu_champion > 0 && lcu_champion != self.state.champ_select.last_synced_champion
                {
                    if let Some(champion) = champions::champion_by_id(lcu_champion) {
                        select_champion_setup_inner(
                            &self.snapshot,
                            &self.store,
                            &mut self.state,
                            &mut commands,
                            champion,
                            None,
                            true,
                        );
                    }
                }
            } else {
                self.state.champ_select.last_synced_champion = 0;
            }
        }
        while let Some(event) = self.worker.try_recv_event() {
            if let CoreEvent::Notice(notice) = event {
                self.state.set_notice(notice);
            }
        }
        while let Some(window) = self.worker.try_recv_window() {
            self.window = window;
        }
        for command in commands {
            self.dispatch(command);
        }

        if self
            .snapshot
            .update_status
            .as_deref()
            .is_some_and(|status| status == "Installing update and restarting…")
            && !self.state.force_exit
        {
            self.state.force_exit = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn dispatch(&mut self, command: CoreCommand) {
        if let Err(err) = self.worker.try_send_command(command) {
            self.state.set_notice(CoreNotice::Warning(format!(
                "Background service busy, command dropped: {err}"
            )));
        }
    }

    fn handle_hotkeys(&mut self) {
        while let Ok(event) = self.hotkey_rx.try_recv() {
            if event.toggle_overlay {
                let next = !self.overlay_engine.is_enabled();
                self.set_overlay(next, false);
            }
            if event.toggle_passthrough && self.overlay_engine.is_enabled() {
                self.state.overlay_mouse_passthrough = !self.state.overlay_mouse_passthrough;
            }
        }
    }

    fn ready_check_alert(&mut self) {
        let ready = self.snapshot.ready_check.as_ref().is_some_and(|check| {
            check.get("state").and_then(Value::as_str) == Some("InProgress")
                && check.get("playerResponse").and_then(Value::as_str) == Some("None")
        });
        if ready && !self.state.ready_check_alerted && self.store.settings.sound_alerts {
            play_ready_alert();
        }
        self.state.ready_check_alerted = ready;
    }

    fn update_overlay_target(&mut self) {
        let stale = self
            .monitors_at
            .is_none_or(|at| at.elapsed() >= MONITOR_REFRESH);
        if stale {
            self.monitors = query_monitors();
            self.monitors_at = Some(Instant::now());
        }
        let target = self.window.window.map(|info| {
            let monitor = monitor_for_rect(info.bounds, &self.monitors);
            TargetWindowState::from_league_window(info, self.window.is_foreground, monitor, None)
        });
        self.state.overlay_target = self.window.window;
        self.state.overlay_target_is_foreground = self.window.is_foreground;
        self.overlay_engine.apply_command(
            OverlayCommand::UpdateTarget(target),
            &mut self.overlay_config,
        );

        // One failed live-client poll doesn't end the match (see `LiveGameLatch`).
        let live = self
            .live_game_latch
            .update(self.state.parsed_live_game.is_some(), Instant::now());
        self.overlay_engine
            .on_live_game_state(live, &mut self.overlay_config);
        self.state.overlay_mode = self.overlay_engine.is_enabled();
        self.state.overlay_waiting_for_window =
            self.state.overlay_mode && (self.window.window.is_none() || !self.window.is_foreground);
    }

    fn apply_effects(&mut self, ctx: &egui::Context, effects: PendingUiEffects) {
        if effects.apply_theme {
            self.palette = apply_native_theme(ctx, &self.store.settings.theme);
        }
        if effects.apply_font {
            install_font_family(ctx, &self.store.settings.font_family);
        }
        if effects.apply_always_on_top {
            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                if self.store.settings.always_on_top {
                    egui::viewport::WindowLevel::AlwaysOnTop
                } else {
                    egui::viewport::WindowLevel::Normal
                },
            ));
        }
        if let Some((enabled, automatic)) = effects.set_overlay {
            self.set_overlay(enabled, automatic);
        }
        if let Some(passthrough) = effects.set_passthrough {
            self.state.overlay_mouse_passthrough = passthrough;
        }
        if let Some(url) = effects.open_browser_url {
            ctx.open_url(egui::OpenUrl::new_tab(url));
        }
        if effects.save_settings {
            self.save_settings();
        }
    }

    /// Runs one overlay engine tick and returns the engine's requested repaint delay.
    fn tick_overlay(&mut self, ctx: &egui::Context) -> Option<Duration> {
        self.sync_overlay_config();
        self.overlay_render.settings =
            OverlayPanelSettings::from_app_settings(&self.store.settings);
        self.overlay_render.preview_mode = self.store.settings.hud_preview_mode;
        self.overlay_render.show_mayhem_cards = self.state.show_mayhem_cards;
        self.overlay_render
            .apply_live_game(self.state.parsed_live_game.clone());

        let monitor = self
            .overlay_engine
            .target()
            .and_then(|t| monitor_for_rect(t.client_rect_px, &self.monitors))
            .cloned();
        let pointer_down = self.overlay_host.pointer_down();
        let (_, next) = self.overlay_engine.tick(
            self.overlay_host.as_mut(),
            TickInput {
                monitor: monitor.as_ref(),
                cursor_px: cursor_screen_position(),
                pointer_down,
                config: &self.overlay_config,
                render_state: &mut self.overlay_render,
                force_interactive_redraw: false,
            },
            &mut render_overlay_ui,
        );

        let actions = std::mem::take(&mut self.overlay_render.pending_actions);
        let mut save = false;
        for (key, pos) in actions.updated_positions {
            self.store.settings.hud_positions.insert(key, pos);
            save = true;
        }
        if actions.request_close_overlay {
            self.set_overlay(false, false);
        }
        if let Some(set) = actions.export_item_set {
            self.dispatch(CoreCommand::ExportItemSet(set));
        }
        if let Some(text) = actions.copy_clipboard {
            ctx.copy_text(text);
        }
        if let Some(notice) = actions.notice {
            self.state.set_notice(CoreNotice::Info(notice));
        }
        self.state.show_mayhem_cards = self.overlay_render.show_mayhem_cards;
        if save {
            self.save_settings();
        }
        next
    }
}

impl eframe::App for RiftCoreShell {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.textures.begin_frame();
        self.handle_tray_and_close(&ctx);
        self.drain_worker(&ctx);
        self.handle_hotkeys();
        self.ready_check_alert();
        self.update_overlay_target();

        let mut commands = Vec::new();
        let mut effects = PendingUiEffects::default();
        {
            let palette = self.palette;
            let mut page_ctx = PageContext {
                snapshot: &self.snapshot,
                store: &mut self.store,
                palette: &palette,
                textures: &mut self.textures,
                state: &mut self.state,
                commands: &mut commands,
                effects: &mut effects,
            };
            chrome::render_titlebar(&mut page_ctx, ui);
            let modal_open = page_ctx.state.show_whats_new || page_ctx.state.show_quick_tour;
            ui.add_enabled_ui(!modal_open, |ui| {
                chrome::render_sidebar(&mut page_ctx, ui);
                chrome::render_main(&mut page_ctx, ui);
            });
            chrome::render_dialogs(&mut page_ctx, &ctx);
        }
        for command in commands {
            self.dispatch(command);
        }
        self.apply_effects(&ctx, effects);

        let overlay_next = self.tick_overlay(&ctx);
        let next = overlay_next.map_or(IDLE_REPAINT, |d| d.min(IDLE_REPAINT));
        ctx.request_repaint_after(next);
    }

    fn on_exit(&mut self) {
        self.overlay_host.close();
        self.hotkeys.shutdown();
        self.worker.shutdown();
        self.store.settings.hud_preview_mode = false;
        if let Err(err) = self.store.save() {
            eprintln!("RiftCore: failed to save settings on exit: {err}");
        }
    }
}

#[cfg(windows)]
fn play_ready_alert() {
    #[link(name = "user32")]
    extern "system" {
        fn MessageBeep(kind: u32) -> i32;
    }
    // SAFETY: MessageBeep takes a plain flag value and has no pointer arguments.
    unsafe {
        let _ = MessageBeep(0x0000_0030);
    }
}

#[cfg(not(windows))]
fn play_ready_alert() {}

/// Runtime UI selection. The legacy monolith stays the default until the modular
/// shell has been verified on Windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiRuntime {
    Legacy,
    Modular,
}

impl UiRuntime {
    /// Resolves from `--modular-ui` / `--legacy-ui` CLI flags, then `RIFTCORE_UI`
    /// (`modular` | `legacy`), defaulting to `Legacy`.
    pub fn resolve<I: IntoIterator<Item = String>>(args: I, env_value: Option<&str>) -> Self {
        let mut from_args = None;
        for arg in args {
            match arg.as_str() {
                "--modular-ui" => from_args = Some(Self::Modular),
                "--legacy-ui" => from_args = Some(Self::Legacy),
                _ => {}
            }
        }
        from_args.unwrap_or_else(|| match env_value.map(str::trim) {
            Some(v) if v.eq_ignore_ascii_case("modular") => Self::Modular,
            _ => Self::Legacy,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::UiRuntime;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn defaults_to_legacy() {
        assert_eq!(
            UiRuntime::resolve(args(&["riftcore.exe"]), None),
            UiRuntime::Legacy
        );
        assert_eq!(
            UiRuntime::resolve(args(&[]), Some("garbage")),
            UiRuntime::Legacy
        );
    }

    #[test]
    fn env_selects_modular() {
        assert_eq!(
            UiRuntime::resolve(args(&[]), Some(" Modular ")),
            UiRuntime::Modular
        );
    }

    #[test]
    fn cli_flag_overrides_env() {
        assert_eq!(
            UiRuntime::resolve(args(&["x", "--legacy-ui"]), Some("modular")),
            UiRuntime::Legacy
        );
        assert_eq!(
            UiRuntime::resolve(args(&["x", "--modular-ui"]), Some("legacy")),
            UiRuntime::Modular
        );
    }
}
