use crate::backend::{self, AppSettings, CoreCommand, CoreSnapshot};
use eframe::egui::{
    self, Align, Color32, FontId, Frame, Layout, RichText, ScrollArea, Sense, Stroke, Vec2,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

const BG: Color32 = Color32::from_rgb(11, 13, 18);
const SURFACE: Color32 = Color32::from_rgb(16, 19, 26);
const CARD: Color32 = Color32::from_rgb(21, 25, 34);
const RAISED: Color32 = Color32::from_rgb(30, 35, 47);
const INSET: Color32 = Color32::from_rgb(15, 18, 24);
const CYAN: Color32 = Color32::from_rgb(86, 194, 255);
const CYAN_DIM: Color32 = Color32::from_rgb(24, 52, 74);
const ACCENT_SOFT: Color32 = Color32::from_rgb(25, 44, 62);
const BUTTON_HOVER: Color32 = Color32::from_rgb(40, 46, 61);
const RED: Color32 = Color32::from_rgb(251, 113, 133);
const GOLD: Color32 = Color32::from_rgb(251, 191, 84);
const GREEN: Color32 = Color32::from_rgb(52, 211, 153);
const TEXT: Color32 = Color32::from_rgb(232, 235, 241);
const SECONDARY: Color32 = Color32::from_rgb(163, 171, 187);
const MUTED: Color32 = Color32::from_rgb(110, 118, 136);
const BORDER: Color32 = Color32::from_rgb(31, 36, 48);
const OVERLAY_WINDOW_TITLE: &str = "RiftCore In-Game HUD";

fn overlay_viewport_id() -> egui::ViewportId {
    egui::ViewportId::from_hash_of(OVERLAY_WINDOW_TITLE)
}

static RUNE_CATALOG: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../assets/data/runesReforged.json")).unwrap_or(Value::Null)
});
static MAYHEM_CARDS: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../assets/data/mayhem_cards.json")).unwrap_or(Value::Null)
});

#[cfg(windows)]
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIcon, TrayIconBuilder,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Champion {
    id: u32,
    name: String,
    key: String,
    title: String,
    #[serde(default)]
    roles: Vec<String>,
}

#[derive(Clone, Debug)]
struct ChampionSetup {
    role: String,
    primary_tree: u32,
    secondary_tree: u32,
    perks: [i64; 9],
    secondary_rows: [usize; 2],
    spell_one: i64,
    spell_two: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Page {
    ChampSelect,
    Draft,
    Lobby,
    Hud,
    Tft,
    Modes,
    TierList,
    ChampionPool,
    Profile,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SettingsSection {
    Gameplay,
    Overlay,
    RiotApi,
    App,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HudSettingsTab {
    Timers,
    Stats,
    Guides,
    Layout,
}

#[derive(Clone, Copy, Debug)]
struct HotkeyEvent {
    toggle_overlay: bool,
    toggle_passthrough: bool,
}

pub struct RiftCoreApp {
    snapshot_rx: Receiver<CoreSnapshot>,
    commands: Sender<CoreCommand>,
    hotkey_rx: Receiver<HotkeyEvent>,
    snapshot: CoreSnapshot,
    settings: AppSettings,
    settings_path: PathBuf,
    champions: Vec<Champion>,
    champion_setups: HashMap<u32, ChampionSetup>,
    champion_textures: HashMap<String, egui::TextureHandle>,
    tft_textures: HashMap<String, egui::TextureHandle>,
    rune_textures: HashMap<i64, egui::TextureHandle>,
    spell_textures: HashMap<i64, egui::TextureHandle>,
    manual_spell_timers: HashMap<String, Instant>,
    logo_texture: egui::TextureHandle,
    selected_champion: u32,
    build_role: String,
    champ_select_detail_tab: usize,
    riot_id_search: String,
    profile_lookup_loading: bool,
    selected_analysis_match: Option<Value>,
    champion_search: String,
    champion_role_filter: String,
    last_synced_champion: u32,
    /// Champion whose locked-in pick was already auto-imported this champ select.
    last_auto_imported_champion: u32,
    pool_search: String,
    pool_role: String,
    pool_assign_slot: Option<usize>,
    tier_role: String,
    tier_search: String,
    tier_view_cards: bool,
    tier_sort: String,
    draft_allies: [u32; 5],
    draft_enemies: [u32; 5],
    tft_unit_input: String,
    tft_selected_cell: Option<usize>,
    tft_tab: String,
    tft_search: String,
    selected_tft_comp: String,
    mode_search: String,
    aram_relic_seconds: String,
    tft_component_one: String,
    tft_component_two: String,
    tft_data_category: String,
    selected_game_mode: usize,
    mayhem_search: String,
    mayhem_tier: String,
    mayhem_mode: String,
    active_page: Page,
    settings_section: SettingsSection,
    hud_settings_tab: HudSettingsTab,
    overlay_mode: bool,
    overlay_auto_entered: bool,
    overlay_waiting_for_window: bool,
    overlay_dismissed_by_user: bool,
    live_game_latch: riftcore_native::domain::live_game::LiveGameLatch,
    overlay_mouse_passthrough: bool,
    overlay_input_passthrough: bool,
    overlay_interactive_regions: Vec<egui::Rect>,
    overlay_viewport_open: bool,
    overlay_target: Option<backend::LeagueWindowInfo>,
    overlay_target_is_foreground: bool,
    overlay_last_maintain: Instant,
    ready_check_alerted: bool,
    pinned: bool,
    show_quick_tour: bool,
    show_whats_new: bool,
    show_mayhem_cards: bool,
    show_overlay_build: bool,
    show_overlay_timers: bool,
    overlay_timer_tab: String,
    show_close_confirmation: bool,
    force_exit: bool,
    /// The main window is hidden in the system tray (the app keeps running).
    hidden_in_tray: bool,
    update_banner_dismissed: bool,
    #[cfg(windows)]
    tray_icon: Option<TrayIcon>,
    /// New in-game HUD (Path B). `None` means the original child-viewport HUD is used.
    path_b: Option<crate::path_b::PathBOverlay>,
    path_b_status: crate::path_b::PathBStatus,
    primary_tree: u32,
    secondary_tree: u32,
    selected_perks: [i64; 9],
    secondary_rune_rows: [usize; 2],
    spell_one: i64,
    spell_two: i64,
    notice_until: Option<Instant>,
    notice: String,
    last_bounds_check: Instant,
}

impl RiftCoreApp {
    pub fn new(creation: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.dark_mode = true;
        visuals.panel_fill = BG;
        visuals.window_fill = SURFACE;
        visuals.override_text_color = Some(TEXT);
        visuals.widgets.noninteractive.bg_fill = CARD;
        visuals.widgets.inactive.bg_fill = CARD;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, SECONDARY);
        visuals.widgets.hovered.bg_fill = RAISED;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, CYAN);
        visuals.widgets.active.bg_fill = CYAN_DIM;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, CYAN);
        visuals.selection.bg_fill = CYAN_DIM;
        visuals.selection.stroke = Stroke::new(1.0, CYAN);
        creation.egui_ctx.set_visuals(visuals);
        creation.egui_ctx.style_mut_of(egui::Theme::Dark, |style| {
            style.spacing.item_spacing = Vec2::new(8.0, 9.0);
            style.spacing.button_padding = Vec2::new(11.0, 7.0);
            style.spacing.interact_size.y = 32.0;
            style
                .text_styles
                .insert(egui::TextStyle::Body, FontId::proportional(14.0));
            style
                .text_styles
                .insert(egui::TextStyle::Button, FontId::proportional(12.5));
            style
                .text_styles
                .insert(egui::TextStyle::Small, FontId::proportional(11.5));
            style
                .text_styles
                .insert(egui::TextStyle::Monospace, FontId::monospace(12.5));
            style
                .text_styles
                .insert(egui::TextStyle::Heading, FontId::proportional(22.0));
            style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(12);
            style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(12);
            style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(12);
            style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(12);
            style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
            style.visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
            style.visuals.extreme_bg_color = INSET;
        });

        let settings_path = settings_file();
        let mut settings = load_settings(&settings_path);
        // Desktop preview is a one-session action. Older versions persisted it
        // and could reopen a full-screen HUD over the desktop on the next launch.
        settings.hud_preview_mode = false;
        apply_native_theme(&creation.egui_ctx, &settings.theme);
        install_font_family(&creation.egui_ctx, &settings.font_family);
        creation.egui_ctx.set_zoom_factor(1.06);
        settings.tft_board.resize(28, String::new());
        settings.tft_board.truncate(28);
        let pinned = settings.always_on_top;
        if settings.always_on_top {
            creation
                .egui_ctx
                .send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    egui::viewport::WindowLevel::AlwaysOnTop,
                ));
        }
        let (snapshot_rx, commands) = backend::start_core(creation.egui_ctx.clone());
        let _ = commands.send(CoreCommand::SetAutoAccept(settings.auto_accept));
        let _ = commands.send(CoreCommand::SetAutoBorderless(settings.hud_auto_borderless));
        let _ = commands.send(CoreCommand::CheckUpdates);
        let (hotkey_rx, _) = start_hotkey_listener(creation.egui_ctx.clone());
        let show_whats_new = settings.last_seen_version != env!("CARGO_PKG_VERSION");
        let show_quick_tour = !show_whats_new && !settings.first_run_tour_seen;
        #[cfg(windows)]
        let tray_icon = build_tray_icon();
        let path_b = match crate::path_b::PathBOverlay::new(creation.wgpu_render_state.clone()) {
            Ok(mut overlay) => {
                overlay.set_repaint_context(&creation.egui_ctx);
                Some(overlay)
            }
            Err(err) => {
                eprintln!("RiftCore: new in-game HUD unavailable ({err}); using classic HUD");
                None
            }
        };
        let champions: Vec<Champion> =
            serde_json::from_str(include_str!("../../all_champions.json")).unwrap_or_default();
        let champion_textures = crate::assets::CHAMPION_PORTRAITS
            .iter()
            .filter_map(|(key, bytes)| {
                load_png_texture(
                    &creation.egui_ctx,
                    format!("champion-{key}"),
                    bytes,
                    Some(64),
                )
                .map(|texture| ((*key).to_string(), texture))
            })
            .collect();
        let tft_textures = crate::assets::TFT_ICONS
            .iter()
            .filter_map(|(key, bytes)| {
                load_png_texture(&creation.egui_ctx, format!("tft-{key}"), bytes, Some(48))
                    .map(|texture| ((*key).to_string(), texture))
            })
            .collect();
        let rune_textures = crate::assets::RUNE_ICONS
            .iter()
            .filter_map(|(id, bytes)| {
                load_png_texture(&creation.egui_ctx, format!("rune-{id}"), bytes, Some(48))
                    .map(|texture| (*id, texture))
            })
            .collect();
        let spell_textures = crate::assets::SPELL_ICONS
            .iter()
            .filter_map(|(id, bytes)| {
                load_png_texture(&creation.egui_ctx, format!("spell-{id}"), bytes, Some(48))
                    .map(|texture| (*id, texture))
            })
            .collect();
        let logo_texture = load_png_texture(
            &creation.egui_ctx,
            "riftcore-logo".to_string(),
            crate::assets::RIFTCORE_LOGO,
            Some(64),
        )
        .expect("embedded RiftCore logo must be a valid PNG");
        let mut app = Self {
            snapshot_rx,
            commands,
            hotkey_rx,
            snapshot: CoreSnapshot::default(),
            settings,
            settings_path,
            champions,
            champion_setups: HashMap::new(),
            champion_textures,
            tft_textures,
            rune_textures,
            spell_textures,
            manual_spell_timers: HashMap::new(),
            logo_texture,
            selected_champion: 103,
            build_role: "MID".to_string(),
            champ_select_detail_tab: 1,
            riot_id_search: String::new(),
            profile_lookup_loading: false,
            selected_analysis_match: None,
            champion_search: String::new(),
            champion_role_filter: "ALL".to_string(),
            last_synced_champion: 0,
            last_auto_imported_champion: 0,
            pool_search: String::new(),
            pool_role: "MID".to_string(),
            pool_assign_slot: None,
            tier_role: "ALL".to_string(),
            tier_search: String::new(),
            tier_view_cards: false,
            tier_sort: "NAME".to_string(),
            draft_allies: [266, 64, 0, 222, 412],
            draft_enemies: [122, 254, 238, 51, 53],
            tft_unit_input: String::new(),
            tft_selected_cell: None,
            tft_tab: "COMPS".to_string(),
            tft_search: String::new(),
            selected_tft_comp: String::new(),
            mode_search: String::new(),
            aram_relic_seconds: String::new(),
            tft_component_one: "B.F. Sword".to_string(),
            tft_component_two: "Sparring Gloves".to_string(),
            tft_data_category: "UNITS".to_string(),
            selected_game_mode: 0,
            mayhem_search: String::new(),
            mayhem_tier: "ALL".to_string(),
            mayhem_mode: "Both".to_string(),
            active_page: Page::ChampSelect,
            settings_section: SettingsSection::Gameplay,
            hud_settings_tab: HudSettingsTab::Timers,
            overlay_mode: false,
            overlay_auto_entered: false,
            overlay_waiting_for_window: false,
            overlay_dismissed_by_user: false,
            live_game_latch: Default::default(),
            overlay_mouse_passthrough: false,
            overlay_input_passthrough: true,
            overlay_interactive_regions: Vec::new(),
            overlay_viewport_open: false,
            overlay_target: None,
            overlay_target_is_foreground: false,
            overlay_last_maintain: Instant::now() - Duration::from_secs(2),
            ready_check_alerted: false,
            pinned,
            show_quick_tour,
            show_whats_new,
            show_mayhem_cards: false,
            show_overlay_build: false,
            show_overlay_timers: false,
            overlay_timer_tab: "OBJECTIVES".to_string(),
            show_close_confirmation: false,
            force_exit: false,
            hidden_in_tray: false,
            update_banner_dismissed: false,
            #[cfg(windows)]
            tray_icon,
            path_b,
            path_b_status: Default::default(),
            primary_tree: 8100,
            secondary_tree: 8000,
            selected_perks: [8112, 8126, 8140, 8135, 9111, 8014, 5008, 5008, 5001],
            secondary_rune_rows: [2, 3],
            spell_one: 4,
            spell_two: 12,
            notice_until: None,
            notice: String::new(),
            last_bounds_check: Instant::now() - Duration::from_secs(5),
        };
        if show_whats_new {
            app.settings.last_seen_version = env!("CARGO_PKG_VERSION").to_string();
            app.save_settings();
        }
        app
    }

    fn set_notice(&mut self, text: impl Into<String>) {
        self.notice = text.into();
        self.notice_until = Some(Instant::now() + Duration::from_secs(5));
    }

    fn save_settings(&self) {
        if let Some(parent) = self.settings_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_vec_pretty(&self.settings) {
            let _ = std::fs::write(&self.settings_path, content);
        }
    }

    fn set_overlay_mode(&mut self, ctx: &egui::Context, enabled: bool, automatic: bool) {
        if self.overlay_mode == enabled {
            return;
        }
        if !automatic {
            self.overlay_dismissed_by_user = !enabled;
        } else if !enabled {
            self.overlay_dismissed_by_user = false;
        }
        self.overlay_mode = enabled;
        self.overlay_auto_entered = automatic && enabled;
        if enabled {
            self.overlay_target = backend::league_game_window();
            self.overlay_target_is_foreground = self
                .overlay_target
                .map(backend::league_window_is_foreground)
                .unwrap_or(false);
            self.overlay_waiting_for_window =
                self.overlay_target.is_none() || !self.overlay_target_is_foreground;
            if self.overlay_target.is_none() {
                self.set_notice("Start a League match or Practice Tool to enable the native HUD.");
            } else if !self.overlay_target_is_foreground {
                self.set_notice("HUD will appear when the League game is in the foreground.");
            }
            ctx.request_repaint();
        } else {
            self.overlay_auto_entered = false;
            self.overlay_waiting_for_window = false;
            if self.overlay_viewport_open {
                ctx.send_viewport_cmd_to(overlay_viewport_id(), egui::ViewportCommand::Close);
                self.overlay_viewport_open = false;
            }
        }
    }

    fn render_overlay_viewport(&mut self, ctx: &egui::Context) {
        if !self.overlay_mode {
            if self.overlay_viewport_open {
                ctx.send_viewport_cmd_to(overlay_viewport_id(), egui::ViewportCommand::Close);
                self.overlay_viewport_open = false;
            }
            return;
        }

        let target = match self.overlay_target {
            Some(target) => target,
            None => {
                if self.overlay_viewport_open {
                    ctx.send_viewport_cmd_to(
                        overlay_viewport_id(),
                        egui::ViewportCommand::Visible(false),
                    );
                }
                return;
            }
        };

        // Keep the native viewport alive between focus changes; only hide it
        // while another window is in front of the game (or keep visible in preview mode).
        let visible = self.overlay_target_is_foreground;
        if !visible && !self.overlay_viewport_open {
            return;
        }

        let scale = (target.dpi as f32 / 96.0).max(1.0);
        let [left, top, right, bottom] = target.bounds;
        let game_width = (right - left).max(1) as f32;
        let game_height = (bottom - top).max(1) as f32;
        let (offset_x, offset_y, width_px, height_px) =
            if self.show_mayhem_cards && self.settings.hud_mode != "integrated" {
                (
                    (game_width - 430.0).max(0.0),
                    52.0,
                    430.0f32.min(game_width),
                    560.0f32.min(game_height),
                )
            } else {
                match self.settings.hud_mode.as_str() {
                    "bar" => (
                        (game_width - 620.0).max(0.0) * 0.5,
                        10.0,
                        620.0f32.min(game_width),
                        54.0,
                    ),
                    "pill" => (
                        (game_width - 300.0).max(0.0),
                        58.0,
                        300.0f32.min(game_width),
                        50.0,
                    ),
                    "card" => (
                        (game_width - 360.0).max(0.0),
                        58.0,
                        360.0f32.min(game_width),
                        350.0f32.min(game_height),
                    ),
                    // Stay one physical pixel inside an exact borderless-game
                    // surface. Some Windows driver paths treat a same-sized
                    // topmost window as full-screen and stop alpha composition.
                    _ => (
                        0.0,
                        0.0,
                        (game_width - 1.0).max(1.0),
                        (game_height - 1.0).max(1.0),
                    ),
                }
            };
        let width = width_px / scale;
        let height = height_px / scale;
        let overlay_origin = [left as f32 + offset_x, top as f32 + offset_y];
        let cursor_inside_control = backend::cursor_screen_position()
            .map(|cursor| {
                let local = egui::pos2(
                    (cursor[0] as f32 - overlay_origin[0]) / scale,
                    (cursor[1] as f32 - overlay_origin[1]) / scale,
                );
                self.overlay_interactive_regions
                    .iter()
                    .any(|region| region.contains(local))
            })
            .unwrap_or(false);
        let mouse_passthrough = self.overlay_mouse_passthrough || !cursor_inside_control;
        let update_mouse_passthrough =
            !self.overlay_viewport_open || self.overlay_input_passthrough != mouse_passthrough;
        let builder = egui::ViewportBuilder::default()
            .with_title(OVERLAY_WINDOW_TITLE)
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(false)
            .with_taskbar(false)
            .with_active(false)
            .with_always_on_top()
            .with_position(egui::pos2(
                (left as f32 + offset_x) / scale,
                (top as f32 + offset_y) / scale,
            ))
            .with_inner_size(Vec2::new(width, height))
            .with_visible(visible);

        let close_requested =
            ctx.show_viewport_immediate(overlay_viewport_id(), builder, |overlay_ui, _class| {
                let overlay_ctx = overlay_ui.ctx().clone();
                let close_requested = overlay_ctx.input(|input| input.viewport().close_requested());
                if update_mouse_passthrough {
                    overlay_ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(
                        mouse_passthrough,
                    ));
                }
                if !close_requested && visible {
                    self.render_overlay(overlay_ui);
                }
                close_requested
            });

        if close_requested {
            self.overlay_mode = false;
            self.overlay_auto_entered = false;
            self.overlay_dismissed_by_user = true;
            self.overlay_waiting_for_window = false;
            self.set_notice("HUD overlay closed");
            self.overlay_viewport_open = false;
        } else {
            self.overlay_viewport_open = true;
            self.overlay_input_passthrough = mouse_passthrough;
            if visible && self.overlay_last_maintain.elapsed() >= Duration::from_millis(750) {
                backend::maintain_native_overlay();
                self.overlay_last_maintain = Instant::now();
            }
        }
    }

    fn render_titlebar(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        egui::Panel::top("titlebar")
            .exact_size(38.0)
            .frame(
                Frame::new()
                    .fill(SURFACE)
                    .stroke(Stroke::new(1.0, BORDER))
                    .inner_margin(egui::Margin::symmetric(12, 0)),
            )
            .show(ui, |ui| {
                let header_size = Vec2::new(ui.available_width(), ui.available_height());
                let drag_response = ui.interact(
                    ui.max_rect().shrink(1.0),
                    egui::Id::unique("riftcore-titlebar-drag-area"),
                    Sense::click_and_drag(),
                );
                if drag_response.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                } else if drag_response.double_clicked() {
                    let maximized = ctx.input(|input| input.viewport().maximized.unwrap_or(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
                }
                ui.allocate_ui_with_layout(
                    header_size,
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        let logo = egui::Image::new((self.logo_texture.id(), Vec2::splat(24.0)))
                            .corner_radius(egui::CornerRadius::same(6))
                            .sense(Sense::click_and_drag());
                        if ui.add(logo).drag_started() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }
                        ui.add_space(7.0);
                        let brand = ui.add(
                            egui::Label::new(
                                RichText::new("RIFTCORE").size(12.0).strong().color(TEXT),
                            )
                            .sense(Sense::click_and_drag()),
                        );
                        if brand.drag_started() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }
                        ui.label(RichText::new("COMPANION").size(8.0).strong().color(MUTED));
                        ui.add_space(8.0);
                        titlebar_badge(ui, "NATIVE", CYAN);
                        ui.add_space(8.0);
                        let (dot, status) = if self.snapshot.connected {
                            (
                                GREEN,
                                format!("In Game  ·  {}", phase_name(&self.snapshot.phase)),
                            )
                        } else if self.snapshot.riot_client_open {
                            (GOLD, "Riot Ready".to_string())
                        } else {
                            (RED, "Offline".to_string())
                        };
                        titlebar_status(ui, &status, dot);
                        if !self.snapshot.connected && self.snapshot.riot_client_open {
                            ui.add_space(6.0);
                            if titlebar_action_button(ui, "LAUNCH", false, CYAN).clicked() {
                                let _ = self.commands.send(CoreCommand::LaunchLeague);
                            }
                        }
                        ui.add_space((ui.available_width() - 172.0).max(6.0));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if window_button(ui, "×", RED).clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            let maximized =
                                ctx.input(|input| input.viewport().maximized.unwrap_or(false));
                            if window_button(ui, if maximized { "❐" } else { "□" }, SECONDARY)
                                .clicked()
                            {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
                            }
                            if window_button(ui, "—", SECONDARY).clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            }
                            let overlay_label = if self.overlay_mode {
                                if self.overlay_waiting_for_window {
                                    "WAITING"
                                } else {
                                    "HUD ON"
                                }
                            } else {
                                "HUD"
                            };
                            if titlebar_action_button(ui, overlay_label, self.overlay_mode, CYAN)
                                .clicked()
                            {
                                self.set_overlay_mode(&ctx, !self.overlay_mode, false);
                            }
                            if titlebar_action_button(
                                ui,
                                if self.pinned { "PINNED" } else { "PIN" },
                                self.pinned,
                                GOLD,
                            )
                            .clicked()
                            {
                                self.pinned = !self.pinned;
                                self.settings.always_on_top = self.pinned;
                                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                                    if self.pinned {
                                        egui::viewport::WindowLevel::AlwaysOnTop
                                    } else {
                                        egui::viewport::WindowLevel::Normal
                                    },
                                ));
                                self.save_settings();
                            }
                        });
                    },
                );
            });
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("navigation")
            .exact_size(224.0)
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(SURFACE)
                    .inner_margin(egui::Margin::same(14))
                    .stroke(Stroke::new(1.0, BORDER)),
            )
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .id_salt("nav-scroll-v2")
                    .show(ui, |ui| {
                        nav_section(ui, "MATCH & LIVE");
                        self.nav_button(ui, Page::ChampSelect, "🎯", "Champ Select", "LIVE", CYAN);
                        self.nav_button(ui, Page::Draft, "🧠", "Draft Coach", "SIM", RED);
                        self.nav_button(ui, Page::Lobby, "👥", "Lobby Scout", "SMART", GREEN);
                        self.nav_button(ui, Page::Hud, "🎮", "In-Game HUD", "", CYAN);
                        nav_section(ui, "GAME MODES & META");
                        self.nav_button(ui, Page::Tft, "♟️", "TFT Companion", "TFT", GOLD);
                        self.nav_button(ui, Page::Modes, "⚔️", "Arena & Modes", "MODES", GOLD);
                        self.nav_button(ui, Page::TierList, "🏆", "Tier List & Builds", "", CYAN);
                        self.nav_button(ui, Page::ChampionPool, "📚", "Champion Pool", "", CYAN);
                        self.nav_button(ui, Page::Profile, "📈", "Profile & Analysis", "", CYAN);
                        nav_section(ui, "CONFIGURATION");
                        self.nav_button(ui, Page::Settings, "⚙️", "Preferences", "", CYAN);
                        ui.add_space(4.0);
                        if action_button(ui, "QUICK TOUR", false, GOLD).clicked() {
                            self.show_quick_tour = true;
                        }
                        if action_button(ui, "WHAT'S NEW", false, CYAN).clicked() {
                            self.show_whats_new = true;
                        }
                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("QUICK AUTOMATION")
                                .size(10.5)
                                .strong()
                                .color(MUTED),
                        );
                        if checkbox_row(ui, "Auto Accept Queue", &mut self.settings.auto_accept) {
                            let _ = self
                                .commands
                                .send(CoreCommand::SetAutoAccept(self.settings.auto_accept));
                            self.save_settings();
                        }
                        if checkbox_row(
                            ui,
                            "Auto Import Runes",
                            &mut self.settings.auto_import_runes,
                        ) {
                            self.save_settings();
                        }
                        if checkbox_row(ui, "Audio Alerts", &mut self.settings.sound_alerts) {
                            self.save_settings();
                        }
                        ui.add_space(6.0);
                        if ui
                            .add_sized(
                                [ui.available_width(), 36.0],
                                egui::Button::new(
                                    RichText::new("☕  Fuel on Ko-fi")
                                        .size(12.5)
                                        .strong()
                                        .color(TEXT),
                                )
                                .fill(RAISED)
                                .stroke(Stroke::new(1.0, CYAN_DIM)),
                            )
                            .clicked()
                        {
                            ui.ctx()
                                .open_url(egui::OpenUrl::new_tab("https://ko-fi.com/X6I629C814"));
                        }
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new(format!("VERSION  {}", env!("CARGO_PKG_VERSION")))
                                .size(10.5)
                                .color(MUTED),
                        );
                    });
            });
    }

    fn nav_button(
        &mut self,
        ui: &mut egui::Ui,
        page: Page,
        icon: &str,
        label: &str,
        badge: &str,
        accent: Color32,
    ) {
        let selected = self.active_page == page;
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 42.0), Sense::click());
        let hover =
            ui.ctx()
                .animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.15);
        let picked = ui
            .ctx()
            .animate_bool_with_time(response.id.with("selected"), selected, 0.22);
        let fill = lerp_color(
            lerp_color(Color32::TRANSPARENT, RAISED, hover),
            ACCENT_SOFT,
            picked,
        );
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(12), fill);
        if picked > 0.01 {
            let inset = 7.0 + (rect.height() * 0.5 - 7.0) * (1.0 - picked);
            ui.painter().rect_filled(
                egui::Rect::from_min_max(
                    rect.min + Vec2::new(1.0, inset),
                    rect.min + Vec2::new(4.0, rect.height() - inset),
                ),
                egui::CornerRadius::same(2),
                accent.gamma_multiply(picked),
            );
        }
        let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
        let foreground = if response.hovered() || selected {
            TEXT
        } else {
            SECONDARY
        };
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 20.0, rect.center().y),
            Vec2::new(27.0, 27.0),
        );
        ui.painter().rect_filled(
            icon_rect,
            egui::CornerRadius::same(6),
            Color32::from_rgba_unmultiplied(
                accent.r(),
                accent.g(),
                accent.b(),
                if selected { 42 } else { 18 },
            ),
        );
        ui.painter().rect_stroke(
            icon_rect,
            egui::CornerRadius::same(6),
            Stroke::new(
                0.8,
                Color32::from_rgba_unmultiplied(
                    accent.r(),
                    accent.g(),
                    accent.b(),
                    if selected { 135 } else { 48 },
                ),
            ),
            egui::StrokeKind::Inside,
        );
        ui.painter().text(
            egui::pos2(icon_rect.center().x, icon_rect.center().y),
            egui::Align2::CENTER_CENTER,
            icon,
            FontId::proportional(18.0),
            if selected { accent } else { SECONDARY },
        );
        ui.painter().text(
            egui::pos2(rect.left() + 38.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            FontId::proportional(13.0),
            foreground,
        );
        if !badge.is_empty() {
            let width = badge.chars().count() as f32 * 6.5 + 12.0;
            let badge_rect = egui::Rect::from_center_size(
                egui::pos2(rect.right() - width * 0.5 - 8.0, rect.center().y),
                Vec2::new(width, 20.0),
            );
            ui.painter().rect_filled(
                badge_rect,
                egui::CornerRadius::same(6),
                Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 22),
            );
            ui.painter().rect_stroke(
                badge_rect,
                egui::CornerRadius::same(6),
                Stroke::new(
                    1.0,
                    Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 80),
                ),
                egui::StrokeKind::Inside,
            );
            ui.painter().text(
                badge_rect.center(),
                egui::Align2::CENTER_CENTER,
                badge,
                FontId::proportional(9.0),
                accent,
            );
        }
        if response.clicked() {
            self.active_page = page;
            if page == Page::Hud && self.snapshot.live_game.is_some() {
                self.set_notice("HUD view selected");
            }
        }
    }

    fn render_main(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(Frame::new().fill(BG).inner_margin(egui::Margin::same(20)))
            .show(ui, |ui| {
                paint_command_grid(ui);
                if let Some(until) = self.notice_until {
                    if Instant::now() < until {
                        Frame::new()
                            .fill(ACCENT_SOFT)
                            .stroke(Stroke::new(1.0, CYAN_DIM))
                            .corner_radius(egui::CornerRadius::same(10))
                            .inner_margin(egui::Margin::same(12))
                            .show(ui, |ui| {
                                ui.label(RichText::new(&self.notice).size(13.0).color(CYAN));
                            });
                        ui.add_space(10.0);
                    } else {
                        self.notice_until = None;
                    }
                }
                if !self.update_banner_dismissed
                    && self
                        .snapshot
                        .update_status
                        .as_deref()
                        .is_some_and(|status| status.contains("is available"))
                {
                    let version = self
                        .snapshot
                        .latest_release
                        .as_ref()
                        .and_then(|release| release.get("tag_name"))
                        .and_then(Value::as_str)
                        .unwrap_or("New version")
                        .to_string();
                    let release_url = self
                        .snapshot
                        .latest_release
                        .as_ref()
                        .and_then(|release| release.get("html_url"))
                        .and_then(Value::as_str)
                        .map(str::to_owned);
                    let has_auto_update = self
                        .snapshot
                        .latest_release
                        .as_ref()
                        .and_then(|release| release.get("assets"))
                        .and_then(Value::as_array)
                        .is_some_and(|assets| {
                            assets.iter().any(|asset| {
                                asset.get("name").and_then(Value::as_str) == Some("RiftCore.exe")
                            })
                        });
                    Frame::new()
                        .fill(RAISED)
                        .stroke(Stroke::new(1.0, GOLD))
                        .corner_radius(egui::CornerRadius::same(12))
                        .inner_margin(egui::Margin::symmetric(13, 9))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("UPDATE READY").size(9.5).strong().color(GOLD),
                                );
                                ui.label(
                                    RichText::new(format!("RiftCore {version} is available"))
                                        .size(11.0)
                                        .color(TEXT),
                                );
                                let update_busy = self
                                    .snapshot
                                    .update_status
                                    .as_deref()
                                    .is_some_and(|status| status.starts_with("Downloading update"));
                                if action_button(
                                    ui,
                                    if update_busy {
                                        "DOWNLOADING…"
                                    } else if has_auto_update {
                                        "UPDATE & RESTART"
                                    } else {
                                        "DOWNLOAD SETUP"
                                    },
                                    update_busy,
                                    CYAN,
                                )
                                .clicked()
                                    && !update_busy
                                {
                                    if has_auto_update {
                                        let _ = self.commands.send(CoreCommand::InstallUpdate);
                                    } else if let Some(url) = release_url.as_deref() {
                                        ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                                    }
                                }
                                if action_button(ui, "×", false, SECONDARY).clicked() {
                                    self.update_banner_dismissed = true;
                                }
                            });
                        });
                    ui.add_space(10.0);
                }
                let waiting_for_accept = self
                    .snapshot
                    .ready_check
                    .as_ref()
                    .map(|check| {
                        check.get("state").and_then(Value::as_str) == Some("InProgress")
                            && check.get("playerResponse").and_then(Value::as_str) == Some("None")
                    })
                    .unwrap_or(false);
                if waiting_for_accept {
                    Frame::new()
                        .fill(Color32::from_rgb(11, 34, 38))
                        .stroke(Stroke::new(1.0, GREEN))
                        .inner_margin(egui::Margin::same(10))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new("MATCH FOUND")
                                            .size(14.0)
                                            .strong()
                                            .color(TEXT),
                                    );
                                    ui.label(
                                        RichText::new(if self.settings.auto_accept {
                                            "Auto-accept is enabled"
                                        } else {
                                            "Ready check is waiting for you"
                                        })
                                        .size(10.5)
                                        .color(SECONDARY),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if action_button(ui, "ACCEPT MATCH", false, GREEN).clicked() {
                                        let _ = self.commands.send(CoreCommand::AcceptMatch);
                                        self.set_notice("Accepting the match…");
                                    }
                                });
                            });
                        });
                    ui.add_space(10.0);
                }
                page_transition(
                    ui,
                    (
                        self.active_page,
                        &self.tft_tab,
                        self.selected_game_mode,
                        self.champ_select_detail_tab,
                        self.show_mayhem_cards,
                    ),
                );
                if self.active_page == Page::ChampSelect {
                    self.render_champ_select(ui);
                } else {
                    ScrollArea::vertical()
                        // Keep each page's scroll position separate. The previous shared ID
                        // could restore the HUD page's bottom offset on Champ Select.
                        .id_salt(("main-scroll-v3", self.active_page))
                        .auto_shrink([false, false])
                        .scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                        )
                        .show(ui, |ui| match self.active_page {
                            Page::ChampSelect => {}
                            Page::Draft => self.render_draft(ui),
                            Page::Lobby => self.render_lobby(ui),
                            Page::Hud => self.render_hud_page(ui),
                            Page::Tft => self.render_tft(ui),
                            Page::Modes => self.render_modes(ui),
                            Page::TierList => self.render_tier_list(ui),
                            Page::ChampionPool => self.render_champion_pool(ui),
                            Page::Profile => self.render_profile(ui),
                            Page::Settings => self.render_settings(ui),
                        });
                }
            });
    }

    fn render_page_heading(
        &self,
        ui: &mut egui::Ui,
        eyebrow: &str,
        title: &str,
        description: &str,
    ) {
        let (link_label, link_color) = if self.snapshot.connected {
            ("LIVE LINK", GREEN)
        } else if self.snapshot.riot_client_open {
            ("CLIENT READY", GOLD)
        } else {
            ("CLIENT OFFLINE", MUTED)
        };
        Frame::new()
            .inner_margin(egui::Margin::symmetric(6, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(eyebrow.to_uppercase())
                                .size(10.5)
                                .strong()
                                .color(CYAN),
                        );
                        ui.add_space(2.0);
                        ui.label(RichText::new(title).size(31.0).strong().color(TEXT));
                        ui.label(RichText::new(description).size(13.0).color(SECONDARY));
                    });
                    ui.add_space((ui.available_width() - 194.0).max(0.0));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.vertical(|ui| {
                            badge(ui, link_label, link_color);
                            ui.add_space(5.0);
                            ui.label(
                                RichText::new(phase_name(&self.snapshot.phase))
                                    .size(11.0)
                                    .color(MUTED),
                            );
                        });
                    });
                });
            });
        ui.add_space(14.0);
    }

    fn select_champion_setup(&mut self, champion_id: u32, allow_auto_import: bool) {
        let Some(champion) = self
            .champions
            .iter()
            .find(|champion| champion.id == champion_id)
            .cloned()
        else {
            return;
        };
        // In champ select, the assigned position picks the setup; otherwise the profile role.
        let assigned_role = (self.snapshot.phase == "ChampSelect")
            .then_some(self.snapshot.champ_select.as_ref())
            .flatten()
            .and_then(champ_select_assigned_role);
        let saved = self
            .champion_setups
            .get(&champion_id)
            .filter(|setup| assigned_role.is_none_or(|role| setup.role.eq_ignore_ascii_case(role)))
            .cloned();
        self.selected_champion = champion_id;
        self.last_synced_champion = champion_id;
        if let Some(setup) = &saved {
            self.build_role = setup.role.clone();
            self.primary_tree = setup.primary_tree;
            self.secondary_tree = setup.secondary_tree;
            self.selected_perks = setup.perks;
            self.secondary_rune_rows = setup.secondary_rows;
            self.spell_one = setup.spell_one;
            self.spell_two = setup.spell_two;
        } else {
            let build = crate::builds::recommend(
                &champion.key,
                &champion.name,
                &champion.roles,
                assigned_role,
            );
            self.build_role = build.role.clone();
            self.primary_tree = build.primary_tree;
            self.secondary_tree = build.secondary_tree;
            self.selected_perks = std::array::from_fn(|index| {
                build.rune_ids.get(index).copied().unwrap_or(5008) as i64
            });
            self.secondary_rune_rows = secondary_rows_for_perks(
                self.secondary_tree,
                self.selected_perks[4],
                self.selected_perks[5],
            );
            (self.spell_one, self.spell_two) = preferred_spell_order(
                build.spell_one,
                build.spell_two,
                &self.settings.flash_slot_preference,
            );
        }

        if !allow_auto_import || self.snapshot.phase != "ChampSelect" {
            return;
        }
        let recommendation = crate::builds::recommend(
            &champion.key,
            &champion.name,
            &champion.roles,
            Some(&self.build_role),
        );
        if self.settings.auto_import_spells {
            let _ = self.commands.send(CoreCommand::ImportSpells {
                spell1: self.spell_one,
                spell2: self.spell_two,
            });
        }
        if self.settings.auto_import_runes {
            let page = rune_page(
                &champion.name,
                self.primary_tree,
                self.secondary_tree,
                &self.selected_perks,
            );
            let _ = self.commands.send(CoreCommand::ImportRunes(page));
        }
        if self.settings.auto_import_item_sets {
            let set = champion_item_set(&champion, &recommendation);
            let _ = self.commands.send(CoreCommand::ExportItemSet(set));
        }
    }

    fn render_champ_select(&mut self, ui: &mut egui::Ui) {
        // The rune tab needs the vertical space, so it skips the big page header.
        if self.champ_select_detail_tab != 1 {
            self.render_page_heading(
                ui,
                "PRE-GAME SYSTEM",
                "Champion Select",
                "Choose your champion, inspect the recommended build, or tune every rune.",
            );
        }
        let search = self.champion_search.trim().to_lowercase();
        let role_filter = self.champion_role_filter.clone();
        let matches = self
            .champions
            .iter()
            .filter(|champion| {
                let matches_search = search.is_empty()
                    || champion.name.to_lowercase().contains(&search)
                    || champion.title.to_lowercase().contains(&search)
                    || champion
                        .roles
                        .iter()
                        .any(|role| role.to_lowercase().contains(&search));
                let matches_role = role_filter == "ALL"
                    || champion
                        .roles
                        .iter()
                        .any(|role| role_matches(role, &role_filter));
                matches_search && matches_role
            })
            .cloned()
            .collect::<Vec<_>>();

        if self.champ_select_detail_tab != 1 {
            panel(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("⌕").size(18.0).color(CYAN));
                    let search_width = (ui.available_width() - 280.0).max(180.0);
                    ui.add_sized(
                        [search_width, 34.0],
                        egui::TextEdit::singleline(&mut self.champion_search).hint_text(format!(
                            "Find a champion · {} available",
                            self.champions.len()
                        )),
                    );
                    let mut auto_sync = self.settings.auto_import_runes
                        && self.settings.auto_import_spells
                        && self.settings.auto_import_item_sets;
                    if ui.checkbox(&mut auto_sync, "AUTO-SYNC").changed() {
                        self.settings.auto_import_runes = auto_sync;
                        self.settings.auto_import_spells = auto_sync;
                        self.settings.auto_import_item_sets = auto_sync;
                        self.save_settings();
                    }
                    if action_button(ui, "REFRESH CLIENT", false, CYAN).clicked() {
                        let _ = self.commands.send(CoreCommand::Refresh);
                        self.set_notice("Refreshing League Client connection");
                    }
                });
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    for role in ["ALL", "TOP", "JUNGLE", "MID", "ADC", "SUPPORT"] {
                        if filter_chip(ui, role, self.champion_role_filter == role, CYAN).clicked()
                        {
                            self.champion_role_filter = role.to_string();
                        }
                    }
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!("{} of {}", matches.len(), self.champions.len()))
                            .size(10.0)
                            .color(MUTED),
                    );
                });
            });
            ui.add_space(8.0);
        }
        let pane_height = (ui.available_height() - 4.0).clamp(280.0, 900.0);
        let runes_mode = self.champ_select_detail_tab == 1;
        let total_width = ui.available_width();
        let left_width = if runes_mode {
            (total_width * 0.34).clamp(220.0, 300.0)
        } else {
            ((total_width - 8.0) / 2.0).max(420.0)
        };
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(left_width, pane_height),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.set_width(left_width);
                    panel(ui, |ui| {
                        ui.set_min_height((pane_height - 32.0).max(240.0));
                        if self.champ_select_detail_tab == 1 {
                            ui.add_sized(
                                [ui.available_width(), 28.0],
                                egui::TextEdit::singleline(&mut self.champion_search)
                                    .hint_text("Find a champion…"),
                            );
                            ui.add_space(6.0);
                        } else {
                            panel_title(
                                ui,
                                "CHOOSE CHAMPION",
                                &format!("{} champions match", matches.len()),
                            );
                        }
                        let avail_width = ui.available_width();
                        let min_cols = if runes_mode && avail_width < 240.0 {
                            2.0
                        } else {
                            3.0
                        };
                        let num_cols =
                            ((avail_width - 16.0) / 106.0).floor().max(min_cols) as usize;
                        let card_w = ((avail_width - 16.0 - ((num_cols - 1) as f32 * 8.0))
                            / num_cols as f32)
                            .max(80.0);
                        ScrollArea::vertical()
                            .id_salt("champion-roster-scroll-v3")
                            .auto_shrink([false, false])
                            .scroll_bar_visibility(
                                egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                            )
                            .max_height((pane_height - 92.0).max(180.0))
                            .show(ui, |ui| {
                                egui::Grid::new("champion-grid")
                                    .num_columns(num_cols)
                                    .spacing([8.0, 8.0])
                                    .show(ui, |ui| {
                                        for (index, champ) in matches.iter().enumerate() {
                                            let selected = self.selected_champion == champ.id;
                                            let (rect, response) = ui.allocate_exact_size(
                                                Vec2::new(card_w, 112.0),
                                                Sense::click(),
                                            );
                                            let animation_id =
                                                ui.make_persistent_id(("champion-card", champ.id));
                                            let hover_t = ui.ctx().animate_bool_with_time(
                                                animation_id.with("hover"),
                                                response.hovered(),
                                                0.14,
                                            );
                                            let selected_t = ui.ctx().animate_bool_with_time(
                                                animation_id.with("selected"),
                                                selected,
                                                0.18,
                                            );
                                            let emphasis = selected_t.max(hover_t * 0.52);
                                            let border = lerp_color(BORDER, CYAN, emphasis);
                                            ui.painter().rect_filled(
                                                rect,
                                                egui::CornerRadius::same(10),
                                                lerp_color(CARD, ACCENT_SOFT, emphasis),
                                            );
                                            ui.painter().rect_stroke(
                                                rect,
                                                egui::CornerRadius::same(10),
                                                Stroke::new(1.0 + selected_t, border),
                                                egui::StrokeKind::Inside,
                                            );
                                            if let Some(texture) =
                                                self.champion_textures.get(&champ.key)
                                            {
                                                let portrait_size =
                                                    68.0 + 4.0 * hover_t + selected_t;
                                                let image_rect = egui::Rect::from_center_size(
                                                    egui::pos2(rect.center().x, rect.top() + 40.0),
                                                    Vec2::splat(portrait_size),
                                                );
                                                egui::Image::new((texture.id(), image_rect.size()))
                                                    .corner_radius(egui::CornerRadius::same(9))
                                                    .paint_at(ui, image_rect);
                                            } else {
                                                ui.painter().text(
                                                    egui::pos2(rect.center().x, rect.top() + 40.0),
                                                    egui::Align2::CENTER_CENTER,
                                                    initials(&champ.name),
                                                    FontId::proportional(21.0),
                                                    CYAN,
                                                );
                                            }
                                            ui.painter().rect_filled(
                                                egui::Rect::from_min_max(
                                                    egui::pos2(
                                                        rect.left() + 1.0,
                                                        rect.bottom() - 28.0,
                                                    ),
                                                    egui::pos2(
                                                        rect.right() - 1.0,
                                                        rect.bottom() - 1.0,
                                                    ),
                                                ),
                                                egui::CornerRadius::same(9),
                                                Color32::from_black_alpha(168),
                                            );
                                            if selected_t > 0.01 {
                                                ui.painter().rect_filled(
                                                    egui::Rect::from_min_max(
                                                        egui::pos2(
                                                            rect.left() + 2.0,
                                                            rect.top() + 12.0,
                                                        ),
                                                        egui::pos2(
                                                            rect.left() + 4.0,
                                                            rect.bottom() - 12.0,
                                                        ),
                                                    ),
                                                    egui::CornerRadius::same(2),
                                                    Color32::from_rgba_unmultiplied(
                                                        CYAN.r(),
                                                        CYAN.g(),
                                                        CYAN.b(),
                                                        (selected_t * 235.0) as u8,
                                                    ),
                                                );
                                            }
                                            ui.painter().text(
                                                egui::pos2(rect.center().x, rect.bottom() - 13.0),
                                                egui::Align2::CENTER_CENTER,
                                                &champ.name,
                                                FontId::proportional(10.5),
                                                lerp_color(TEXT, CYAN, selected_t),
                                            );
                                            if response.clicked() {
                                                self.select_champion_setup(champ.id, false);
                                            }
                                            if index % num_cols == num_cols - 1 {
                                                ui.end_row();
                                            }
                                        }
                                    });
                                if matches.is_empty() {
                                    Frame::new()
                                        .fill(CARD)
                                        .corner_radius(egui::CornerRadius::same(10))
                                        .inner_margin(egui::Margin::same(18))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new("No champions match those filters.")
                                                    .color(SECONDARY),
                                            );
                                        });
                                }
                            });
                    });
                },
            );
            ui.add_space(8.0);
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), pane_height),
                Layout::top_down(Align::Min),
                |ui| {
                    panel(ui, |ui| {
                        ui.set_min_height((pane_height - 32.0).max(240.0));
                        ScrollArea::vertical()
                            .id_salt("champion-details-scroll-v3")
                            .auto_shrink([false, false])
                            .scroll_bar_visibility(
                                egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                            )
                            .max_height((pane_height - 48.0).max(220.0))
                            .show(ui, |ui| {
                                let champ = self
                                    .champions
                                    .iter()
                                    .find(|champion| champion.id == self.selected_champion)
                                    .cloned();
                                if let Some(champ) = champ {
                                    let mut recommendation = crate::builds::recommend(
                                        &champ.key,
                                        &champ.name,
                                        &champ.roles,
                                        Some(&self.build_role),
                                    );
                                    if self.champ_select_detail_tab == 1 {
                                        ui.horizontal(|ui| {
                                            avatar(
                                                ui,
                                                &champ.name,
                                                34.0,
                                                CYAN,
                                                self.champion_textures.get(&champ.key),
                                            );
                                            ui.add_space(6.0);
                                            ui.label(
                                                RichText::new(champ.name.to_uppercase())
                                                    .size(16.0)
                                                    .strong()
                                                    .color(TEXT),
                                            );
                                            for role in &champ.roles {
                                                badge(ui, role, Color32::from_rgb(100, 140, 180));
                                            }
                                            ui.label(
                                                RichText::new(format!(
                                                    "Skill Max: {}",
                                                    recommendation.skill_priority
                                                ))
                                                .size(10.5)
                                                .strong()
                                                .color(GOLD),
                                            );
                                        });
                                    } else {
                                        // Champion Hero Card
                                        Frame::new()
                                            .fill(INSET)
                                            .stroke(Stroke::new(1.0, BORDER))
                                            .corner_radius(egui::CornerRadius::same(12))
                                            .inner_margin(egui::Margin::same(12))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    avatar(
                                                        ui,
                                                        &champ.name,
                                                        58.0,
                                                        CYAN,
                                                        self.champion_textures.get(&champ.key),
                                                    );
                                                    ui.add_space(8.0);
                                                    ui.vertical(|ui| {
                                                        ui.horizontal(|ui| {
                                                            ui.label(
                                                                RichText::new(
                                                                    champ.name.to_uppercase(),
                                                                )
                                                                .size(20.0)
                                                                .strong()
                                                                .color(TEXT),
                                                            );
                                                            ui.add_space(6.0);
                                                            badge(
                                                                ui,
                                                                &recommendation
                                                                    .archetype
                                                                    .replace('_', " "),
                                                                CYAN,
                                                            );
                                                        });
                                                        ui.label(
                                                            RichText::new(&champ.title)
                                                                .size(11.5)
                                                                .color(SECONDARY),
                                                        );
                                                        ui.add_space(3.0);
                                                        ui.horizontal(|ui| {
                                                            for role in &champ.roles {
                                                                badge(
                                                                    ui,
                                                                    role,
                                                                    Color32::from_rgb(
                                                                        100, 140, 180,
                                                                    ),
                                                                );
                                                            }
                                                            ui.add_space(6.0);
                                                            ui.label(
                                                                RichText::new(format!(
                                                                    "Skill Max: {}",
                                                                    recommendation.skill_priority
                                                                ))
                                                                .size(10.5)
                                                                .strong()
                                                                .color(GOLD),
                                                            );
                                                        });
                                                    });
                                                });
                                            });
                                    }
                                    ui.add_space(8.0);
                                    ui.horizontal_wrapped(|ui| {
                                        for (tab, label) in [
                                            (0usize, "BUILD"),
                                            (1, "RUNES & SPELLS"),
                                            (2, "SESSION"),
                                        ] {
                                            if action_button(
                                                ui,
                                                label,
                                                self.champ_select_detail_tab == tab,
                                                CYAN,
                                            )
                                            .clicked()
                                            {
                                                self.champ_select_detail_tab = tab;
                                            }
                                        }
                                    });
                                    ui.add_space(6.0);
                                    if self.champ_select_detail_tab == 0 {
                                        panel(ui, |ui| {
                                            panel_title(
                                        ui,
                                        "CHAMPION BUILD",
                                        "Role-based starting path, skill order and shop import",
                                    );
                                            ui.horizontal_wrapped(|ui| {
                                                ui.label(
                                                    RichText::new("ROLE")
                                                        .size(9.5)
                                                        .strong()
                                                        .color(MUTED),
                                                );
                                                egui::ComboBox::from_id_salt("champion-build-role")
                                                    .selected_text(&self.build_role)
                                                    .show_ui(ui, |ui| {
                                                        for role in [
                                                            "TOP", "JUNGLE", "MID", "ADC",
                                                            "SUPPORT",
                                                        ] {
                                                            if champ.roles.iter().any(|entry| {
                                                                role_matches(entry, role)
                                                            }) {
                                                                ui.selectable_value(
                                                                    &mut self.build_role,
                                                                    role.to_string(),
                                                                    role,
                                                                );
                                                            }
                                                        }
                                                    });
                                                badge(
                                                    ui,
                                                    &recommendation.archetype.replace('_', " "),
                                                    CYAN,
                                                );
                                            });
                                            recommendation = crate::builds::recommend(
                                                &champ.key,
                                                &champ.name,
                                                &champ.roles,
                                                Some(&self.build_role),
                                            );
                                            ui.add_space(4.0);
                                            ui.horizontal_wrapped(|ui| {
                                                ui.label(
                                                    RichText::new("SKILL PRIORITY")
                                                        .size(9.0)
                                                        .strong()
                                                        .color(MUTED),
                                                );
                                                ui.label(
                                                    RichText::new(&recommendation.skill_priority)
                                                        .size(11.5)
                                                        .strong()
                                                        .color(GOLD),
                                                );
                                            });
                                            ui.horizontal_wrapped(|ui| {
                                                ui.label(
                                                    RichText::new("LEVEL ORDER")
                                                        .size(9.0)
                                                        .strong()
                                                        .color(MUTED),
                                                );
                                                ui.label(
                                                    RichText::new(
                                                        recommendation.skill_order.join("  "),
                                                    )
                                                    .size(10.0)
                                                    .color(TEXT),
                                                );
                                            });
                                            ui.add_space(5.0);
                                            ui.add(
                                                egui::Label::new(
                                                    RichText::new(format!(
                                                        "START  {}    BOOTS  {}",
                                                        recommendation
                                                            .starting_items
                                                            .iter()
                                                            .map(|id| crate::builds::item_name(*id))
                                                            .collect::<Vec<_>>()
                                                            .join(" + "),
                                                        crate::builds::item_name(
                                                            recommendation.boots
                                                        )
                                                    ))
                                                    .size(10.5)
                                                    .color(SECONDARY),
                                                )
                                                .wrap(),
                                            );
                                            ui.add(
                                                egui::Label::new(
                                                    RichText::new(format!(
                                                        "CORE  {}",
                                                        recommendation
                                                            .core_items
                                                            .iter()
                                                            .map(|id| crate::builds::item_name(*id))
                                                            .collect::<Vec<_>>()
                                                            .join("  →  ")
                                                    ))
                                                    .size(10.5)
                                                    .color(TEXT),
                                                )
                                                .wrap(),
                                            );
                                            ui.add(
                                                egui::Label::new(
                                                    RichText::new(format!(
                                                        "SITUATIONAL  {}",
                                                        recommendation
                                                            .situational_items
                                                            .iter()
                                                            .map(|id| crate::builds::item_name(*id))
                                                            .collect::<Vec<_>>()
                                                            .join("  ·  ")
                                                    ))
                                                    .size(10.0)
                                                    .color(SECONDARY),
                                                )
                                                .wrap(),
                                            );
                                            ui.add_space(4.0);
                                            ui.add(
                                                egui::Label::new(
                                                    RichText::new(&recommendation.tip)
                                                        .size(10.5)
                                                        .color(MUTED),
                                                )
                                                .wrap(),
                                            );
                                            ui.add_space(8.0);
                                            ui.horizontal_wrapped(|ui| {
                                                if action_button(
                                                    ui,
                                                    "APPLY BUILD RUNES",
                                                    false,
                                                    CYAN,
                                                )
                                                .clicked()
                                                {
                                                    let page = crate::builds::rune_page(
                                                        &champ.name,
                                                        &recommendation,
                                                    );
                                                    let _ = self
                                                        .commands
                                                        .send(CoreCommand::ImportRunes(page));
                                                    self.primary_tree = recommendation.primary_tree;
                                                    self.secondary_tree =
                                                        recommendation.secondary_tree;
                                                    self.selected_perks =
                                                        std::array::from_fn(|index| {
                                                            recommendation
                                                                .rune_ids
                                                                .get(index)
                                                                .copied()
                                                                .unwrap_or(5008)
                                                                as i64
                                                        });
                                                    self.secondary_rune_rows =
                                                        secondary_rows_for_perks(
                                                            self.secondary_tree,
                                                            self.selected_perks[4],
                                                            self.selected_perks[5],
                                                        );
                                                    self.set_notice(
                                                "Sending recommended rune page to League Client…",
                                            );
                                                }
                                                if action_button(
                                                    ui,
                                                    "APPLY BUILD SPELLS",
                                                    false,
                                                    RED,
                                                )
                                                .clicked()
                                                {
                                                    let (spell_one, spell_two) =
                                                        preferred_spell_order(
                                                            recommendation.spell_one,
                                                            recommendation.spell_two,
                                                            &self.settings.flash_slot_preference,
                                                        );
                                                    self.spell_one = spell_one;
                                                    self.spell_two = spell_two;
                                                    let _ = self.commands.send(
                                                        CoreCommand::ImportSpells {
                                                            spell1: spell_one,
                                                            spell2: spell_two,
                                                        },
                                                    );
                                                    self.set_notice(
                                        "Sending recommended summoner spells to League Client…",
                                    );
                                                }
                                                if action_button(ui, "EXPORT ITEM SET", false, GOLD)
                                                    .clicked()
                                                {
                                                    let _ = self.commands.send(
                                                        CoreCommand::ExportItemSet(
                                                            champion_item_set(
                                                                &champ,
                                                                &recommendation,
                                                            ),
                                                        ),
                                                    );
                                                    self.set_notice(
                                                "Adding champion build to the in-game shop…",
                                            );
                                                }
                                            });
                                        });
                                    } else if self.champ_select_detail_tab == 1 {
                                        self.render_rune_editor(ui, &champ, &recommendation);
                                    } else {
                                        self.render_team_panel(ui);
                                    }
                                }
                            });
                    });
                },
            );
        });

        if self.selected_champion > 0 {
            self.champion_setups.insert(
                self.selected_champion,
                ChampionSetup {
                    role: self.build_role.clone(),
                    primary_tree: self.primary_tree,
                    secondary_tree: self.secondary_tree,
                    perks: self.selected_perks,
                    secondary_rows: self.secondary_rune_rows,
                    spell_one: self.spell_one,
                    spell_two: self.spell_two,
                },
            );
        }
    }

    fn render_rune_editor(
        &mut self,
        ui: &mut egui::Ui,
        champion: &Champion,
        recommendation: &crate::builds::BuildRecommendation,
    ) {
        panel(ui, |ui| {
            let meta_perks = std::array::from_fn(|index| {
                recommendation.rune_ids.get(index).copied().unwrap_or(5008) as i64
            });
            let presets = [
                (
                    "META",
                    format!("{} build", self.build_role.to_lowercase()),
                    recommendation.primary_tree,
                    recommendation.secondary_tree,
                    meta_perks,
                    CYAN,
                ),
                (
                    "BURST",
                    "Damage first".to_string(),
                    8100,
                    8200,
                    [
                        rune_id_preferred(8100, 0, &["Electrocute"]),
                        rune_id_preferred(8100, 1, &["Sudden Impact"]),
                        rune_id_preferred(8100, 2, &["Eyeball Collection"]),
                        rune_id_preferred(8100, 3, &["Treasure Hunter", "Ultimate Hunter"]),
                        rune_id_preferred(8200, 1, &["Manaflow Band"]),
                        rune_id_preferred(8200, 2, &["Transcendence"]),
                        5008,
                        5008,
                        5001,
                    ],
                    RED,
                ),
                (
                    "SUSTAIN",
                    "Safer trades".to_string(),
                    8400,
                    8000,
                    [
                        rune_id_preferred(8400, 0, &["Grasp of the Undying"]),
                        rune_id_preferred(8400, 1, &["Demolish"]),
                        rune_id_preferred(8400, 2, &["Bone Plating"]),
                        rune_id_preferred(8400, 3, &["Overgrowth"]),
                        rune_id_preferred(8000, 1, &["Triumph"]),
                        rune_id_preferred(8000, 2, &["Legend: Haste", "Legend: Alacrity"]),
                        5008,
                        5008,
                        5001,
                    ],
                    GREEN,
                ),
            ];
            let mut selected_preset = None;
            ui.horizontal(|ui| {
                ui.label(RichText::new("PRESET").size(8.5).strong().color(MUTED));
                for (name, detail, primary, secondary, perks, accent) in presets {
                    let response = filter_chip(ui, name, false, accent);
                    if response.clicked() {
                        selected_preset = Some((primary, secondary, perks));
                    }
                    response.on_hover_text(detail);
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let cta_size = Vec2::new(210.0, 36.0);
                    let (cta_rect, cta_resp) = ui.allocate_exact_size(cta_size, Sense::click());
                    let cta_hover = ui.ctx().animate_bool(cta_resp.id, cta_resp.hovered());
                    let cta_fill = lerp_color(ACCENT_SOFT, CYAN_DIM, cta_hover);
                    ui.painter()
                        .rect_filled(cta_rect, egui::CornerRadius::same(8), cta_fill);
                    ui.painter().rect_stroke(
                        cta_rect,
                        egui::CornerRadius::same(8),
                        Stroke::new(1.2 + cta_hover * 0.8, CYAN),
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        cta_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "⚡ APPLY RUNES + SPELLS",
                        FontId::proportional(12.0),
                        Color32::WHITE,
                    );
                    if cta_resp.clicked() {
                        let page = rune_page(
                            &champion.name,
                            self.primary_tree,
                            self.secondary_tree,
                            &self.selected_perks,
                        );
                        let _ = self.commands.send(CoreCommand::ImportRunes(page));
                        let _ = self.commands.send(CoreCommand::ImportSpells {
                            spell1: self.spell_one,
                            spell2: self.spell_two,
                        });
                        self.set_notice("Sending rune page and summoner spells to League Client…");
                    }
                });
            });
            if let Some((primary, secondary, perks)) = selected_preset {
                self.primary_tree = primary;
                self.secondary_tree = secondary;
                self.selected_perks = perks;
                self.secondary_rune_rows = secondary_rows_for_perks(secondary, perks[4], perks[5]);
            }
            normalize_rune_setup(
                self.primary_tree,
                self.secondary_tree,
                &mut self.secondary_rune_rows,
                &mut self.selected_perks,
            );

            ui.add_space(10.0);
            ui.columns(3, |columns| {
                let primary_accent = rune_tree_color(self.primary_tree);
                let secondary_accent = rune_tree_color(self.secondary_tree);

                // ── Primary card: tree picker + keystone and three rows on a rune path ──
                rune_card(
                    &mut columns[0],
                    primary_accent,
                    self.rune_textures.get(&(self.primary_tree as i64)),
                    |ui| {
                        rune_section_heading(
                            ui,
                            "PRIMARY STYLE",
                            tree_name(self.primary_tree),
                            primary_accent,
                        );
                        if let Some(tree) =
                            rune_tree_picker(ui, self.primary_tree, None, &self.rune_textures)
                        {
                            self.primary_tree = tree;
                            if self.secondary_tree == tree {
                                self.secondary_tree = RUNE_TREES
                                    .iter()
                                    .find(|(id, _)| *id != tree)
                                    .map(|(id, _)| *id)
                                    .unwrap_or(8400);
                            }
                            normalize_rune_setup(
                                self.primary_tree,
                                self.secondary_tree,
                                &mut self.secondary_rune_rows,
                                &mut self.selected_perks,
                            );
                        }
                        ui.add_space(8.0);
                        let rail_slot = ui.painter().add(egui::Shape::Noop);
                        let rail_x = ui.max_rect().left() + 4.0;
                        let mut rail_top = None;
                        let mut nodes = Vec::new();
                        for row in 0..=3 {
                            let slot = row;
                            if row > 0 {
                                ui.add_space(4.0);
                            }
                            rail_top.get_or_insert(ui.cursor().top() + 6.0);
                            let title = if row == 0 {
                                "KEYSTONE".to_string()
                            } else {
                                format!("ROW {row}")
                            };
                            rune_section_heading(
                                ui,
                                &title,
                                &selected_rune_name(
                                    self.primary_tree,
                                    row,
                                    self.selected_perks[slot],
                                ),
                                primary_accent,
                            );
                            let options = rune_options(self.primary_tree, row);
                            let result = rune_icon_row(
                                ui,
                                &options,
                                self.selected_perks[slot],
                                &self.rune_textures,
                                primary_accent,
                                if row == 0 { 42.0 } else { 34.0 },
                                10.0,
                            );
                            if let Some(id) = result.clicked {
                                self.selected_perks[slot] = id;
                            }
                            nodes.extend(result.selected_center);
                        }
                        paint_rune_rail(
                            ui,
                            rail_slot,
                            rail_x,
                            rail_top.unwrap_or_default(),
                            &nodes,
                            primary_accent,
                        );
                    },
                );

                // ── Secondary card: every row visible, click any rune (two rows max) ──
                rune_card(
                    &mut columns[1],
                    secondary_accent,
                    self.rune_textures.get(&(self.secondary_tree as i64)),
                    |ui| {
                        rune_section_heading(
                            ui,
                            "SECONDARY STYLE",
                            tree_name(self.secondary_tree),
                            secondary_accent,
                        );
                        if let Some(tree) = rune_tree_picker(
                            ui,
                            self.secondary_tree,
                            Some(self.primary_tree),
                            &self.rune_textures,
                        ) {
                            self.secondary_tree = tree;
                        }
                        normalize_rune_setup(
                            self.primary_tree,
                            self.secondary_tree,
                            &mut self.secondary_rune_rows,
                            &mut self.selected_perks,
                        );
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            ui.add_space(14.0);
                            ui.label(
                                RichText::new("Pick 2 runes from different rows")
                                    .size(8.5)
                                    .color(MUTED),
                            );
                        });
                        ui.add_space(6.0);
                        let rail_slot = ui.painter().add(egui::Shape::Noop);
                        let rail_x = ui.max_rect().left() + 4.0;
                        let mut rail_top = None;
                        let mut nodes = Vec::new();
                        for row in 1..=3 {
                            if row > 1 {
                                ui.add_space(4.0);
                            }
                            rail_top.get_or_insert(ui.cursor().top() + 6.0);
                            let chosen_slot = self
                                .secondary_rune_rows
                                .iter()
                                .position(|chosen| *chosen == row)
                                .map(|choice| 4 + choice);
                            let selected = chosen_slot
                                .map(|slot| self.selected_perks[slot])
                                .unwrap_or(-1);
                            let detail = chosen_slot
                                .map(|slot| {
                                    selected_rune_name(
                                        self.secondary_tree,
                                        row,
                                        self.selected_perks[slot],
                                    )
                                })
                                .unwrap_or_else(|| "—".to_string());
                            rune_section_heading(
                                ui,
                                &format!("ROW {row}"),
                                &detail,
                                if chosen_slot.is_some() {
                                    secondary_accent
                                } else {
                                    MUTED
                                },
                            );
                            let options = rune_options(self.secondary_tree, row);
                            let result = rune_icon_row(
                                ui,
                                &options,
                                selected,
                                &self.rune_textures,
                                secondary_accent,
                                34.0,
                                10.0,
                            );
                            if let Some(id) = result.clicked {
                                match chosen_slot {
                                    Some(slot) => self.selected_perks[slot] = id,
                                    None => {
                                        // New row replaces the older of the two picks.
                                        self.secondary_rune_rows =
                                            [self.secondary_rune_rows[1], row];
                                        self.selected_perks[4] = self.selected_perks[5];
                                        self.selected_perks[5] = id;
                                    }
                                }
                                normalize_rune_setup(
                                    self.primary_tree,
                                    self.secondary_tree,
                                    &mut self.secondary_rune_rows,
                                    &mut self.selected_perks,
                                );
                            }
                            nodes.extend(result.selected_center);
                        }
                        paint_rune_rail(
                            ui,
                            rail_slot,
                            rail_x,
                            rail_top.unwrap_or_default(),
                            &nodes,
                            secondary_accent,
                        );
                    },
                );

                // ── Shards + summoner spells ──
                rune_card(&mut columns[2], CYAN, None, |ui| {
                    rune_section_heading(ui, "STAT SHARDS", "Offense · Flex · Defense", CYAN);
                    for (row, label) in [(0usize, "OFFENSE"), (1, "FLEX"), (2, "DEFENSE")] {
                        let options = stat_shard_options(row)
                            .iter()
                            .map(|(id, name)| (*id, (*name).to_string()))
                            .collect::<Vec<_>>();
                        let slot = 6 + row;
                        ui.add_space(3.0);
                        ui.horizontal(|ui| {
                            ui.add_space(14.0);
                            ui.label(RichText::new(label).size(8.0).strong().color(MUTED));
                            ui.label(
                                RichText::new(selected_rune_name_from_options(
                                    &options,
                                    self.selected_perks[slot],
                                ))
                                .size(9.5)
                                .color(lerp_color(CYAN, TEXT, 0.45)),
                            );
                        });
                        let result = rune_icon_row(
                            ui,
                            &options,
                            self.selected_perks[slot],
                            &self.rune_textures,
                            CYAN,
                            30.0,
                            10.0,
                        );
                        if let Some(id) = result.clicked {
                            self.selected_perks[slot] = id;
                        }
                    }
                    ui.add_space(10.0);
                    rune_section_heading(
                        ui,
                        "SUMMONER SPELLS",
                        &format!(
                            "{} + {}",
                            spell_name(self.spell_one),
                            spell_name(self.spell_two)
                        ),
                        GOLD,
                    );
                    ui.add_space(2.0);
                    summoner_spell_dropdowns(
                        ui,
                        &mut self.spell_one,
                        &mut self.spell_two,
                        &self.spell_textures,
                    );
                });
            });
        });
    }
    fn render_team_panel(&self, ui: &mut egui::Ui) {
        panel(ui, |ui| {
            panel_title(
                ui,
                "CHAMP SELECT SESSION",
                "Live picks from your current lobby",
            );
            if let Some(session) = self.snapshot.champ_select.as_ref() {
                ui.columns(2, |cols| {
                    for (column, key, heading, accent) in [
                        (0, "myTeam", "YOUR TEAM", CYAN),
                        (1, "theirTeam", "OPPOSING TEAM", RED),
                    ] {
                        cols[column]
                            .label(RichText::new(heading).size(10.0).strong().color(accent));
                        if let Some(members) = session.get(key).and_then(Value::as_array) {
                            for member in members.iter().take(5) {
                                let id = member
                                    .get("championId")
                                    .and_then(Value::as_u64)
                                    .unwrap_or(0) as u32;
                                let champ = self.champions.iter().find(|entry| entry.id == id);
                                let name = if id == 0 {
                                    "Choosing…".to_string()
                                } else {
                                    champ
                                        .map(|entry| entry.name.clone())
                                        .unwrap_or_else(|| format!("Champion {id}"))
                                };
                                let role = member
                                    .get("assignedPosition")
                                    .and_then(Value::as_str)
                                    .unwrap_or("UNASSIGNED");
                                cols[column].horizontal(|ui| {
                                    avatar(
                                        ui,
                                        &name,
                                        28.0,
                                        accent,
                                        champ.and_then(|entry| {
                                            self.champion_textures.get(&entry.key)
                                        }),
                                    );
                                    ui.label(RichText::new(name).size(11.5).strong().color(TEXT));
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(RichText::new(role).size(9.5).color(MUTED));
                                    });
                                });
                            }
                        }
                    }
                });
            } else {
                Frame::new()
                    .fill(INSET)
                    .stroke(Stroke::new(1.0, BORDER))
                    .corner_radius(egui::CornerRadius::same(10))
                    .inner_margin(egui::Margin::same(16))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            avatar(ui, "👥", 44.0, CYAN, None);
                            ui.add_space(8.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(if self.snapshot.connected {
                                        "LOBBY SCOUT STANDBY"
                                    } else {
                                        "LEAGUE CLIENT DISCONNECTED"
                                    })
                                    .size(13.0)
                                    .strong()
                                    .color(TEXT),
                                );
                                ui.label(
                                    RichText::new(if self.snapshot.connected {
                                        "Enter Champion Select or queue up to automatically detect lobby participants, roles, and opponent picks."
                                    } else {
                                        "Launch League of Legends to connect live session intelligence, runes auto-import, and lobby scouting."
                                    })
                                    .size(11.0)
                                    .color(SECONDARY),
                                );
                            });
                        });
                    });
            }
        });
    }

    fn render_draft(&mut self, ui: &mut egui::Ui) {
        self.render_page_heading(
            ui,
            "TEAM COMPOSITION",
            "Draft Coach",
            "Build a 5v5 draft or sync picks from the live League Client session",
        );
        ui.horizontal(|ui| {
            if action_button(
                ui,
                "SYNC LIVE DRAFT",
                self.snapshot.champ_select.is_some(),
                CYAN,
            )
            .clicked()
            {
                if let Some(session) = self.snapshot.champ_select.as_ref() {
                    self.draft_allies = session_picks(session, "myTeam");
                    self.draft_enemies = session_picks(session, "theirTeam");
                    self.set_notice("Draft board synced from the League Client");
                } else {
                    self.set_notice("Open champion select to sync the live draft");
                }
            }
            if action_button(ui, "CLEAR BOARD", false, RED).clicked() {
                self.draft_allies = [0; 5];
                self.draft_enemies = [0; 5];
            }
            ui.label(
                RichText::new("Local simulation only; picks are not sent to League Client.")
                    .size(10.0)
                    .color(MUTED),
            );
        });

        if self.snapshot.champ_select.is_some() {
            ui.add_space(10.0);
            self.render_team_panel(ui);
        }
        ui.add_space(10.0);
        let champions = self.champions.clone();
        let textures = self.champion_textures.clone();
        ui.columns(2, |columns| {
            render_draft_team(
                &mut columns[0],
                "ALLY DRAFT",
                "Your team picks",
                &mut self.draft_allies,
                &champions,
                &textures,
                CYAN,
            );
            render_draft_team(
                &mut columns[1],
                "ENEMY DRAFT",
                "Opponent picks",
                &mut self.draft_enemies,
                &champions,
                &textures,
                RED,
            );
        });
        let ally_filled = self.draft_allies.iter().filter(|id| **id != 0).count();
        let enemy_filled = self.draft_enemies.iter().filter(|id| **id != 0).count();
        ui.add_space(12.0);
        panel(ui, |ui| {
            panel_title(
                ui,
                "COMPOSITION READOUT",
                "Heuristic coverage check from the selected champions; no match outcome is predicted",
            );
            ui.columns(2, |columns| {
                for (column, label, ids, accent, filled) in [
                    (0, "ALLY", &self.draft_allies, CYAN, ally_filled),
                    (1, "ENEMY", &self.draft_enemies, RED, enemy_filled),
                ] {
                    let readout = draft_team_analysis(ids, &self.champions);
                    columns[column].vertical(|ui| {
                        Frame::new()
                            .fill(INSET)
                            .stroke(Stroke::new(1.0, BORDER))
                            .corner_radius(egui::CornerRadius::same(10))
                            .inner_margin(egui::Margin::same(12))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("{label} COMPOSITION"))
                                            .size(13.0)
                                            .strong()
                                            .color(accent),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        badge(
                                            ui,
                                            &format!("{filled}/5 PICKS"),
                                            if filled == 5 { GREEN } else { GOLD },
                                        );
                                    });
                                });
                                ui.add_space(8.0);

                                // Damage split meter
                                let total_dmg = (readout.physical + readout.magic).max(1);
                                let phys_pct = (readout.physical as f32 / total_dmg as f32) * 100.0;
                                let magic_pct = (readout.magic as f32 / total_dmg as f32) * 100.0;
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("DAMAGE PROFILE")
                                            .size(9.5)
                                            .strong()
                                            .color(MUTED),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(
                                            RichText::new(format!(
                                                "Physical {:.0}%  ·  Magic {:.0}%",
                                                phys_pct, magic_pct
                                            ))
                                            .size(9.5)
                                            .color(SECONDARY),
                                        );
                                    });
                                });
                                custom_progress_bar(
                                    ui,
                                    phys_pct / 100.0,
                                    Color32::from_rgb(235, 110, 45),
                                    6.0,
                                );
                                ui.add_space(8.0);

                                // Attribute meter bars
                                for (attr_name, val, max_val, bar_color) in [
                                    (
                                        "Frontline & Durability",
                                        readout.frontline,
                                        5,
                                        Color32::from_rgb(52, 152, 219),
                                    ),
                                    (
                                        "Crowd Control & Utility",
                                        readout.utility,
                                        5,
                                        Color32::from_rgb(155, 89, 182),
                                    ),
                                    ("Magic Threat", readout.magic, 5, CYAN),
                                    (
                                        "Physical Threat",
                                        readout.physical,
                                        5,
                                        Color32::from_rgb(230, 126, 34),
                                    ),
                                ] {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(attr_name).size(10.5).color(TEXT));
                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                ui.label(
                                                    RichText::new(format!("{val} / {max_val}"))
                                                        .size(10.0)
                                                        .color(SECONDARY),
                                                );
                                            },
                                        );
                                    });
                                    custom_progress_bar(
                                        ui,
                                        val as f32 / max_val as f32,
                                        bar_color,
                                        4.0,
                                    );
                                    ui.add_space(3.0);
                                }

                                ui.add_space(6.0);
                                // Strategic advice card
                                let is_balanced = readout.advice.starts_with("Composition");
                                Frame::new()
                                    .fill(if is_balanced {
                                        Color32::from_rgba_unmultiplied(46, 125, 50, 40)
                                    } else {
                                        Color32::from_rgba_unmultiplied(198, 120, 20, 40)
                                    })
                                    .stroke(Stroke::new(
                                        1.0,
                                        if is_balanced { GREEN } else { GOLD },
                                    ))
                                    .corner_radius(egui::CornerRadius::same(6))
                                    .inner_margin(egui::Margin::symmetric(10, 8))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(if is_balanced {
                                                    "🛡️"
                                                } else {
                                                    "⚠️"
                                                })
                                                .size(14.0),
                                            );
                                            ui.label(
                                                RichText::new(&readout.advice)
                                                    .size(11.0)
                                                    .color(TEXT),
                                            );
                                        });
                                    });
                            });
                    });
                }
            });
        });
    }

    fn render_lobby(&mut self, ui: &mut egui::Ui) {
        self.render_page_heading(
            ui,
            "LOBBY INTELLIGENCE",
            "Lobby Scout",
            "Review live team picks and look up public rank, mastery and match history by Riot ID",
        );
        self.render_team_panel(ui);
        ui.add_space(12.0);
        self.render_profile_summary(ui);
        ui.add_space(12.0);
        self.render_riot_search(ui);
        self.render_riot_lookup(ui);
    }

    fn render_hud_page(&mut self, ui: &mut egui::Ui) {
        self.render_page_heading(
            ui,
            "LIVE COMPANION",
            "In-Game HUD",
            "Native transparent overlay for live match information",
        );
        self.render_hud_display_card(ui);
        if self.snapshot.live_game.is_some() {
            ui.horizontal(|ui| {
                let label = if self.overlay_mode {
                    "HIDE NATIVE OVERLAY"
                } else {
                    "SHOW NATIVE OVERLAY"
                };
                if action_button(ui, label, self.overlay_mode, CYAN).clicked() {
                    self.set_overlay_mode(ui.ctx(), !self.overlay_mode, false);
                }
                if action_button(ui, "REFRESH LIVE DATA", false, SECONDARY).clicked() {
                    let _ = self.commands.send(CoreCommand::Refresh);
                }
            });
            self.render_live_summary(ui);
            if self.settings.hud_aram_health_timers
                && self
                    .snapshot
                    .live_game
                    .as_ref()
                    .and_then(|live| live.get("gameData"))
                    .and_then(|data| data.get("gameMode"))
                    .and_then(Value::as_str)
                    .is_some_and(|mode| mode.eq_ignore_ascii_case("ARAM"))
            {
                self.render_aram_relic_timers(ui);
            }
        } else {
            ui.columns(2, |columns| {
                panel(&mut columns[0], |ui| {
                    ui.horizontal(|ui| {
                        avatar(ui, "HUD", 42.0, CYAN, None);
                        ui.vertical(|ui| {
                            ui.label(RichText::new("NATIVE IN-GAME OVERLAY").size(14.0).strong().color(TEXT));
                            let (status, color) = if self.overlay_mode && self.overlay_waiting_for_window {
                                ("WAITING FOR LEAGUE", SECONDARY)
                            } else if self.overlay_mode {
                                ("OVERLAY ENABLED", CYAN)
                            } else {
                                ("STANDBY", SECONDARY)
                            };
                            badge(ui, status, color);
                        });
                    });
                    ui.add_space(5.0);
                    ui.label(RichText::new("Works over League's Fullscreen, Borderless and Windowed modes. If it ever stays hidden over Fullscreen on your PC, Borderless is the fallback.").size(10.5).color(SECONDARY));
                    ui.add_space(8.0);
                    let overlay_btn = if self.overlay_mode { "DISABLE IN-GAME HUD" } else { "ENABLE IN-GAME HUD" };
                    if action_button(ui, overlay_btn, self.overlay_mode, CYAN).clicked() {
                        self.set_overlay_mode(ui.ctx(), !self.overlay_mode, false);
                    }
                });
                panel(&mut columns[1], |ui| {
                    panel_title(ui, "HUD LAYOUT MOCKUP", "Example placement; does not cover your desktop");
                    render_hud_mockup(ui, &self.settings.hud_mode);
                });
            });
        }
        ui.add_space(8.0);
        self.render_hud_settings(ui);
    }

    /// Says why the HUD is hidden (e.g. League minimised) and, in Fullscreen, offers Borderless
    /// as an optional fallback; the HUD itself draws over Fullscreen.
    fn render_hud_display_card(&mut self, ui: &mut egui::Ui) {
        let status = self.path_b.as_ref().filter(|_| self.overlay_mode);
        let status = status.and_then(|path_b| path_b.status().message());
        let fullscreen = self.snapshot.league_window_mode == Some(0);
        panel(ui, |ui| {
            if let Some(message) = status {
                ui.label(RichText::new(message).size(12.5).strong().color(GOLD));
                ui.add_space(4.0);
            }
            if fullscreen {
                ui.label(
                    RichText::new(
                        "League is set to Fullscreen; the HUD draws over it. If it stays hidden on your PC, Borderless always works.",
                    )
                    .size(10.5)
                    .color(SECONDARY),
                );
                if action_button(ui, "SWITCH LEAGUE TO BORDERLESS", false, CYAN).clicked() {
                    let _ = self.commands.send(CoreCommand::SetLeagueWindowMode(2));
                }
                ui.add_space(4.0);
            }
            if settings_row(
                ui,
                "Always use Borderless",
                &mut self.settings.hud_auto_borderless,
                "Optional: switch League from Fullscreen to Borderless (set while the game is closed)",
            ) {
                let enabled = self.settings.hud_auto_borderless;
                let _ = self.commands.send(CoreCommand::SetAutoBorderless(enabled));
                self.save_settings();
            }
        });
        ui.add_space(8.0);
    }

    fn render_live_summary(&self, ui: &mut egui::Ui) {
        if let Some(live) = self.snapshot.live_game.as_ref() {
            let data = live.get("gameData").unwrap_or(&Value::Null);
            let active = live.get("activePlayer").unwrap_or(&Value::Null);
            let me = active_player_entry(live).unwrap_or(&Value::Null);
            let time = data.get("gameTime").and_then(Value::as_f64).unwrap_or(0.0) as u64;
            let scores = me.get("scores").unwrap_or(&Value::Null);
            ui.horizontal(|ui| {
                metric(
                    ui,
                    "GAME CLOCK",
                    &format!("{:02}:{:02}", time / 60, time % 60),
                    CYAN,
                );
                metric(
                    ui,
                    "K / D / A",
                    &format!(
                        "{} / {} / {}",
                        num(scores, "kills"),
                        num(scores, "deaths"),
                        num(scores, "assists")
                    ),
                    TEXT,
                );
                metric(ui, "CS", &num(scores, "creepScore"), GOLD);
                metric(ui, "CURRENT GOLD", &num(active, "currentGold"), GREEN);
            });
            ui.add_space(10.0);
            self.render_roster(ui, live);
        }
    }

    fn render_aram_relic_timers(&mut self, ui: &mut egui::Ui) {
        panel(ui, |ui| {
            panel_title(
                ui,
                "ARAM HEALTH RELIC TIMERS",
                "Enter the displayed respawn time for this match, then start the manual countdown",
            );
            ui.horizontal(|ui| {
                ui.label(RichText::new("Respawn seconds").size(10.0).color(SECONDARY));
                ui.add_sized(
                    [90.0, 26.0],
                    egui::TextEdit::singleline(&mut self.aram_relic_seconds).hint_text("seconds"),
                );
            });
            ui.horizontal(|ui| {
                for side in ["LEFT", "RIGHT"] {
                    let key = format!("aram-relic-{side}");
                    let remaining = self
                        .manual_spell_timers
                        .get(&key)
                        .map(|ready_at| {
                            ready_at.saturating_duration_since(Instant::now()).as_secs()
                        })
                        .unwrap_or(0);
                    if remaining > 0 {
                        ui.label(
                            RichText::new(format!(
                                "{side} {:02}:{:02}",
                                remaining / 60,
                                remaining % 60
                            ))
                            .size(10.0)
                            .monospace()
                            .color(GREEN),
                        );
                        if action_button(ui, "RESET", false, SECONDARY).clicked() {
                            self.manual_spell_timers.remove(&key);
                        }
                    } else {
                        let seconds = self
                            .aram_relic_seconds
                            .parse::<u64>()
                            .ok()
                            .filter(|seconds| *seconds > 0);
                        if action_button(ui, &format!("START {side}"), seconds.is_none(), GOLD)
                            .clicked()
                        {
                            if let Some(seconds) = seconds {
                                self.manual_spell_timers
                                    .insert(key, Instant::now() + Duration::from_secs(seconds));
                            }
                        }
                    }
                }
            });
        });
    }

    fn render_ally_ultimate_timers(&mut self, ui: &mut egui::Ui, live: &Value) {
        let allies = allied_teammates(live);
        if allies.is_empty() {
            return;
        }
        Frame::new()
            .fill(Color32::from_rgba_unmultiplied(5, 10, 19, 232))
            .stroke(Stroke::new(1.0, BORDER))
            .corner_radius(egui::CornerRadius::same(10))
            .inner_margin(egui::Margin::same(9))
            .show(ui, |ui| {
                ui.set_min_width(185.0);
                panel_title(
                    ui,
                    "ALLY R TIMERS",
                    "Click when an ally casts R · 100s manual reference",
                );
                for (role, champion, timer_key) in allies {
                    let remaining = self
                        .manual_spell_timers
                        .get(&timer_key)
                        .map(|ready_at| {
                            ready_at.saturating_duration_since(Instant::now()).as_secs()
                        })
                        .unwrap_or(0);
                    let label = if remaining > 0 {
                        format!("{:02}:{:02} · RESET", remaining / 60, remaining % 60)
                    } else {
                        "MARK R USED".to_string()
                    };
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [68.0, 24.0],
                            egui::Label::new(
                                RichText::new(format!("{role} · {champion}"))
                                    .size(9.0)
                                    .color(TEXT),
                            ),
                        );
                        let color = if remaining > 0 { RED } else { GREEN };
                        if ui
                            .add_sized(
                                [103.0, 24.0],
                                egui::Button::new(
                                    RichText::new(label).size(8.5).strong().color(color),
                                ),
                            )
                            .clicked()
                        {
                            if remaining > 0 {
                                self.manual_spell_timers.remove(&timer_key);
                            } else {
                                self.manual_spell_timers
                                    .insert(timer_key, Instant::now() + Duration::from_secs(100));
                            }
                        }
                    });
                }
            });
    }

    fn render_roster(&self, ui: &mut egui::Ui, live: &Value) {
        panel(ui, |ui| {
            panel_title(
                ui,
                "LIVE SCOREBOARD",
                live.get("gameData")
                    .and_then(|data| data.get("gameMode"))
                    .and_then(Value::as_str)
                    .unwrap_or("LIVE MATCH"),
            );
            if let Some(players) = live.get("allPlayers").and_then(Value::as_array) {
                egui::Grid::new("live-roster")
                    .num_columns(2)
                    .spacing([18.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("SUMMONER").size(9.5).strong().color(MUTED));
                        ui.label(
                            RichText::new("CHAMPION      K / D / A       CS")
                                .size(9.5)
                                .strong()
                                .color(MUTED),
                        );
                        ui.end_row();
                        for (team, label, color) in scoreboard_teams(live) {
                            ui.label(RichText::new(label).size(10.0).strong().color(color));
                            ui.label("");
                            ui.end_row();
                            for player in players.iter().filter(|player| {
                                player.get("team").and_then(Value::as_str) == Some(team)
                            }) {
                                ui.label(
                                    RichText::new(
                                        player
                                            .get("summonerName")
                                            .and_then(Value::as_str)
                                            .unwrap_or("Player"),
                                    )
                                    .size(11.0)
                                    .color(TEXT),
                                );
                                let scores = player.get("scores").unwrap_or(&Value::Null);
                                let stats = format!(
                                    "{}   {} / {} / {}      {} CS",
                                    player
                                        .get("championName")
                                        .and_then(Value::as_str)
                                        .unwrap_or("—"),
                                    num(scores, "kills"),
                                    num(scores, "deaths"),
                                    num(scores, "assists"),
                                    num(scores, "creepScore")
                                );
                                ui.label(RichText::new(stats).size(10.5).color(SECONDARY));
                                ui.end_row();
                            }
                        }
                    });
            }
        });
    }

    fn render_hud_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for (tab, label) in [
                (HudSettingsTab::Timers, "TIMERS"),
                (HudSettingsTab::Stats, "STATS"),
                (HudSettingsTab::Guides, "GUIDES"),
                (HudSettingsTab::Layout, "LAYOUT"),
            ] {
                if action_button(ui, label, self.hud_settings_tab == tab, CYAN).clicked() {
                    self.hud_settings_tab = tab;
                }
            }
            ui.label(
                RichText::new("HUD MODULES")
                    .size(10.0)
                    .strong()
                    .color(MUTED),
            );
        });
        ui.add_space(4.0);

        let mut changed = false;
        match self.hud_settings_tab {
            HudSettingsTab::Timers => ui.columns(2, |columns| {
                panel(&mut columns[0], |ui| {
                    panel_title(ui, "COMBAT TIMERS", "Objectives and champion cooldowns");
                    changed |= toggle_card_row(
                        ui,
                        "Objective Timers",
                        "Dragon, Baron, and Rift Herald",
                        &mut self.settings.hud_objective_timers,
                    );
                    changed |= toggle_card_row(
                        ui,
                        "Allied Ultimate Timers",
                        "Manual click timers for allied R",
                        &mut self.settings.hud_ally_ult_timers,
                    );
                });
                panel(&mut columns[1], |ui| {
                    panel_title(ui, "RESPAWNS", "Camp and mode-specific timers");
                    changed |= toggle_card_row(
                        ui,
                        "Jungle Camp Respawns",
                        "Manual camp respawn dock",
                        &mut self.settings.hud_manual_camp_timers,
                    );
                    changed |= toggle_card_row(
                        ui,
                        "ARAM Health Relic Timers",
                        "Manual health relic countdowns",
                        &mut self.settings.hud_aram_health_timers,
                    );
                });
            }),
            HudSettingsTab::Stats => ui.columns(2, |columns| {
                panel(&mut columns[0], |ui| {
                    panel_title(ui, "STATS & ECONOMY", "Live match progress");
                    changed |= toggle_card_row(
                        ui,
                        "Current Gold Tracker",
                        "Display active gold",
                        &mut self.settings.hud_gold_lead,
                    );
                    changed |= toggle_card_row(
                        ui,
                        "CS Per Minute Pace",
                        "Show your CS per minute",
                        &mut self.settings.hud_cs_benchmark,
                    );
                });
                panel(&mut columns[1], |ui| {
                    panel_title(ui, "PLAYER GUIDES", "Build reminders");
                    changed |= toggle_card_row(
                        ui,
                        "Ability Rank Guide",
                        "Show your current ability ranks",
                        &mut self.settings.hud_skill_order,
                    );
                    changed |= toggle_card_row(
                        ui,
                        "Item Build Goals",
                        "Show your current inventory",
                        &mut self.settings.hud_item_goal,
                    );
                    changed |= toggle_card_row(
                        ui,
                        "Trinket Ward Reminder",
                        "Reminder when no trinket is equipped",
                        &mut self.settings.hud_trinket_reminder,
                    );
                });
            }),
            HudSettingsTab::Guides => ui.columns(2, |columns| {
                panel(&mut columns[0], |ui| {
                    panel_title(ui, "ROUTE GUIDES", "Mode and jungle references");
                    changed |= toggle_card_row(
                        ui,
                        "Mayhem & Arena Cards Guide",
                        "Card selector with rarity and descriptions",
                        &mut self.settings.hud_mayhem_cards,
                    );
                    changed |= toggle_card_row(
                        ui,
                        "Jungle Opening Pathing",
                        "Optimal first-clear path",
                        &mut self.settings.hud_jungle_pathing,
                    );
                    if self.settings.hud_jungle_pathing {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Start route:").size(10.5).color(SECONDARY));
                            for (side, label, accent) in
                                [("red", "RED", RED), ("blue", "BLUE", CYAN)]
                            {
                                if action_button(
                                    ui,
                                    label,
                                    self.settings.jungle_start_buff == side,
                                    accent,
                                )
                                .clicked()
                                {
                                    self.settings.jungle_start_buff = side.to_string();
                                    changed = true;
                                }
                            }
                        });
                    }
                });
                panel(&mut columns[1], |ui| {
                    panel_title(ui, "MINIMAP PINS", "Visual reference markers");
                    changed |= toggle_card_row(
                        ui,
                        "Objective Countdown Pins",
                        "Markers on objective pits",
                        &mut self.settings.hud_map_objective_pins,
                    );
                    changed |= toggle_card_row(
                        ui,
                        "Jungle Route Step Pins",
                        "Numbered jungle clear steps",
                        &mut self.settings.hud_map_route_pins,
                    );
                    changed |= toggle_card_row(
                        ui,
                        "Static Ward Guide Pins",
                        "Strategic ward locations",
                        &mut self.settings.hud_map_ward_pins,
                    );
                });
            }),
            HudSettingsTab::Layout => ui.columns(2, |columns| {
                panel(&mut columns[0], |ui| {
                    panel_title(ui, "HUD PRESENTATION", "Choose the overlay arrangement");
                    ui.horizontal_wrapped(|ui| {
                        for (mode, label) in [
                            ("integrated", "INTEGRATED"),
                            ("bar", "TOP BAR"),
                            ("card", "TACTICAL CARD"),
                            ("pill", "MINIMAP PILL"),
                        ] {
                            if action_button(ui, label, self.settings.hud_mode == mode, CYAN)
                                .clicked()
                            {
                                self.settings.hud_mode = mode.to_string();
                                changed = true;
                            }
                        }
                    });
                    if checkbox_row(
                        ui,
                        "Edit overlay layout",
                        &mut self.settings.hud_edit_layout,
                    ) {
                        if self.settings.hud_edit_layout {
                            self.overlay_mouse_passthrough = false;
                            if self.overlay_viewport_open {
                                ui.ctx().send_viewport_cmd_to(
                                    overlay_viewport_id(),
                                    egui::ViewportCommand::MousePassthrough(false),
                                );
                            }
                        }
                        changed = true;
                    }
                });
                panel(&mut columns[1], |ui| {
                    panel_title(ui, "MINIMAP TIMER DOCK", "Adjust position and size");
                    changed |= ui
                        .horizontal(|ui| {
                            ui.label(RichText::new("Map scale").size(10.5).color(SECONDARY));
                            ui.add(
                                egui::Slider::new(&mut self.settings.hud_minimap_scale, 1.0..=3.0)
                                    .show_value(true),
                            )
                            .changed()
                        })
                        .inner;
                    changed |= ui
                        .horizontal(|ui| {
                            ui.label(RichText::new("Dock size").size(10.5).color(SECONDARY));
                            ui.add(
                                egui::Slider::new(&mut self.settings.hud_minimap_zoom, 0.75..=1.35)
                                    .show_value(true),
                            )
                            .changed()
                        })
                        .inner;
                    changed |= checkbox_row(
                        ui,
                        "Minimap is flipped to the left",
                        &mut self.settings.hud_minimap_flip,
                    );
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Offset").size(10.0).color(SECONDARY));
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut self.settings.hud_minimap_nudge_x)
                                    .range(-250.0..=250.0)
                                    .speed(2.0)
                                    .suffix(" x"),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut self.settings.hud_minimap_nudge_y)
                                    .range(-250.0..=250.0)
                                    .speed(2.0)
                                    .suffix(" y"),
                            )
                            .changed();
                        if action_button(ui, "RESET", false, SECONDARY).clicked() {
                            self.settings.hud_minimap_nudge_x = 0.0;
                            self.settings.hud_minimap_nudge_y = 0.0;
                            self.settings.hud_minimap_zoom = 1.0;
                            changed = true;
                        }
                    });
                });
            }),
        }
        if changed {
            self.save_settings();
        }
    }

    fn render_modes(&mut self, ui: &mut egui::Ui) {
        self.render_page_heading(
            ui,
            "GAME MODE CENTER",
            "Game Modes",
            "Mode-specific setup, rune imports and practical strategy notes",
        );
        ui.horizontal_wrapped(|ui| {
            for (index, label) in ["RIFT", "ARAM", "ARENA", "URF", "ONE FOR ALL", "ARAM MAYHEM"]
                .iter()
                .enumerate()
            {
                if action_button(
                    ui,
                    label,
                    self.selected_game_mode == index,
                    if index == 1 {
                        GOLD
                    } else if index == 4 {
                        RED
                    } else {
                        CYAN
                    },
                )
                .clicked()
                {
                    self.selected_game_mode = index;
                }
            }
        });
        if action_button(ui, "OPEN MAYHEM & ARENA CARD GUIDE", false, GOLD).clicked() {
            self.show_mayhem_cards = true;
        }
        ui.add_space(12.0);
        let mode_index = self.selected_game_mode.min(5);
        let (mode_name, subtitle, details, expected_mode, accent) = match mode_index {
            1 => ("ARAM", "HOWLING ABYSS", "Choose mode runes and spells, then open a champion's full build. Balance modifiers change by patch and are not guessed here.", "ARAM", GOLD),
            2 => ("ARENA", "ARENA 2V2", "Build a complementary duo around engage, durability and damage. Arena rounds and shop state are not published by the Live Client API.", "CHERRY", GOLD),
            3 => ("URF", "ULTRA RAPID FIRE", "Prioritize reliable crowd control, mobility and repeatable damage. The guide avoids unsupported win-rate rankings.", "URF", RED),
            4 => ("ONE FOR ALL", "ONE FOR ALL", "Coordinate the team vote around one champion. Consider wave clear, engage, range and how the same matchup plays into the enemy team.", "ONEFORALL", Color32::from_rgb(192, 132, 252)),
            5 => ("ARAM MAYHEM", "ARAM MAYHEM", "Browse the curated Mayhem and Arena augment catalog by tier, role fit, and tactical effect.", "ARAM", GOLD),
            _ => ("SUMMONER'S RIFT", "RANKED & NORMAL", "Role-based build, runes, summoner spells and live objective information.", "CLASSIC", CYAN),
        };
        let detected = self
            .snapshot
            .live_game
            .as_ref()
            .and_then(|live| live.get("gameData"))
            .and_then(|data| data.get("gameMode"))
            .and_then(Value::as_str)
            .unwrap_or("—");
        panel(ui, |ui| {
            panel_title(ui, subtitle, details);
            ui.horizontal(|ui| {
                metric(
                    ui,
                    "CLIENT",
                    if self.snapshot.connected {
                        "CONNECTED"
                    } else {
                        "OFFLINE"
                    },
                    if self.snapshot.connected {
                        GREEN
                    } else {
                        MUTED
                    },
                );
                metric(
                    ui,
                    "LIVE MODE",
                    detected,
                    if detected.eq_ignore_ascii_case(expected_mode) {
                        GREEN
                    } else {
                        GOLD
                    },
                );
            });
        });

        let champion = self
            .champions
            .iter()
            .find(|champion| champion.id == self.selected_champion)
            .cloned();
        if let Some(champion) = champion {
            let build =
                crate::builds::recommend(&champion.key, &champion.name, &champion.roles, None);
            let (spell_one, spell_two) = match mode_index {
                1 => (4, 32),
                2 => (4, 14),
                3 => (4, 14),
                4 => (4, 6),
                _ => (build.spell_one, build.spell_two),
            };
            panel(ui, |ui| {
                panel_title(
                    ui,
                    &format!("{mode_name} SETUP · {}", champion.name),
                    "Champion build profile and explicit import actions",
                );
                ui.horizontal(|ui| {
                    avatar(
                        ui,
                        &champion.name,
                        56.0,
                        accent,
                        self.champion_textures.get(&champion.key),
                    );
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} · {}",
                                build.role,
                                build.archetype.replace('_', " ")
                            ))
                            .size(12.0)
                            .strong()
                            .color(TEXT),
                        );
                        ui.label(
                            RichText::new(format!("Skill order: {}", build.skill_priority))
                                .size(10.0)
                                .color(CYAN),
                        );
                        ui.label(RichText::new(&build.tip).size(10.0).color(SECONDARY));
                    });
                    if action_button(ui, "OPEN FULL BUILD", false, CYAN).clicked() {
                        self.selected_champion = champion.id;
                        self.build_role = build.role.clone();
                        self.active_page = Page::ChampSelect;
                    }
                });
                ui.add_space(7.0);
                ui.horizontal(|ui| {
                    if action_button(ui, "APPLY MODE RUNES + SPELLS", false, accent).clicked() {
                        let (primary, secondary, perks) = match mode_index {
                            1 => (
                                8100,
                                8000,
                                vec![8128, 8143, 8140, 8135, 8009, 9105, 5008, 5008, 5001],
                            ),
                            2 => (
                                8000,
                                8400,
                                vec![8010, 9111, 9104, 8299, 8473, 8451, 5008, 5008, 5001],
                            ),
                            3 => (
                                8100,
                                8200,
                                vec![8128, 8139, 8140, 8135, 8226, 8210, 5008, 5008, 5001],
                            ),
                            _ => (
                                build.primary_tree,
                                build.secondary_tree,
                                build.rune_ids.clone(),
                            ),
                        };
                        let page = if mode_index == 0 {
                            crate::builds::rune_page(&champion.name, &build)
                        } else {
                            json!({
                                "name": format!("RiftCore {mode_name} {}", champion.name),
                                "primaryStyleId": primary,
                                "subStyleId": secondary,
                                "selectedPerkIds": perks,
                                "current": true
                            })
                        };
                        let _ = self.commands.send(CoreCommand::ImportRunes(page));
                        let _ = self.commands.send(CoreCommand::ImportSpells {
                            spell1: spell_one,
                            spell2: spell_two,
                        });
                        self.set_notice(format!("Applying {mode_name} runes and spells…"));
                    }
                    if action_button(ui, "OPEN LIVE HUD", false, SECONDARY).clicked() {
                        self.active_page = Page::Hud;
                    }
                    ui.label(
                        RichText::new(format!(
                            "Suggested spells: {} + {}",
                            spell_name(spell_one),
                            spell_name(spell_two)
                        ))
                        .size(10.0)
                        .color(GOLD),
                    );
                });
            });
        }

        if mode_index == 2 {
            ui.add_space(10.0);
            panel(ui, |ui| {
                panel_title(
                    ui,
                    "DUO SYNERGY IDEAS",
                    "Pair strategy prompts · no win-rate ranking",
                );
                for (pair, strategy) in [
                    ("Taric + Master Yi", "Protect a melee carry through an all-in and coordinate invulnerability windows."),
                    ("Ivern + Rengar", "Use brush creation and brush-based leaps to create unexpected engage angles."),
                    ("Alistar + Poppy", "Look for displacement and ring pressure when the map closes."),
                    ("Kayle + Braum", "Use peel and protection to buy time for a scaling damage carry."),
                    ("Singed + Cassiopeia", "Layer poison, slows and sustained spell damage in close fights."),
                ] {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(pair).size(10.5).strong().color(GOLD));
                        ui.label(RichText::new(strategy).size(10.0).color(SECONDARY));
                    });
                }
            });
        } else if mode_index == 1 {
            ui.add_space(10.0);
            panel(ui, |ui| {
                panel_title(
                    ui,
                    "ARAM TEAM PLAN",
                    "Use the current patch and in-game balance values",
                );
                ui.label(RichText::new("Snowball can provide an engage option; balance your team between range, reliable engage, wave clear and sustain. Static champion damage modifiers are intentionally omitted because they change across patches.").size(10.5).color(SECONDARY));
            });
            ui.add_space(8.0);
            self.render_aram_relic_timers(ui);
        } else if mode_index == 3 {
            ui.add_space(10.0);
            panel(ui, |ui| {
                panel_title(
                    ui,
                    "URF BUILD PRINCIPLES",
                    "General picks guidance, without invented meta rates",
                );
                ui.label(RichText::new("Prioritize skills that remain useful on short cooldowns, repeatable crowd control, mobility or defensive tools. Check item and rune interactions in the current game client before committing to a build.").size(10.5).color(SECONDARY));
            });
        } else if mode_index == 4 {
            ui.add_space(10.0);
            panel(ui, |ui| {
                panel_title(
                    ui,
                    "ONE FOR ALL TEAM CHECKLIST",
                    "Use the champion picker below to compare options",
                );
                ui.label(RichText::new("Look for strong wave clear, a clear way to start fights, and enough range or durability to handle the opponent's team. Confirm shared-ability interactions in the current game mode; they can change with patches.").size(10.5).color(SECONDARY));
            });
        } else {
            ui.add_space(10.0);
            self.render_hud_settings(ui);
        }

        ui.add_space(10.0);
        panel(ui, |ui| {
            ui.horizontal(|ui| {
                panel_title(
                    ui,
                    "CHAMPION PICKER",
                    "Select a champion for this mode's build and rune setup",
                );
                ui.add_sized(
                    [220.0, 27.0],
                    egui::TextEdit::singleline(&mut self.mode_search)
                        .hint_text("Search champions…"),
                );
            });
            let query = self.mode_search.to_lowercase();
            let visible: Vec<Champion> = self
                .champions
                .iter()
                .filter(|champion| {
                    query.is_empty() || champion.name.to_lowercase().contains(&query)
                })
                .take(24)
                .cloned()
                .collect();
            ui.columns(6, |columns| {
                for (index, champion) in visible.iter().enumerate() {
                    columns[index % 6].vertical_centered(|ui| {
                        if let Some(texture) = self.champion_textures.get(&champion.key) {
                            ui.add(egui::Image::new((texture.id(), Vec2::splat(42.0))));
                        }
                        if action_button(
                            ui,
                            &champion.name,
                            self.selected_champion == champion.id,
                            CYAN,
                        )
                        .clicked()
                        {
                            self.selected_champion = champion.id;
                        }
                    });
                }
            });
        });
    }

    fn render_tft(&mut self, ui: &mut egui::Ui) {
        self.render_page_heading(
            ui,
            "TEAMFIGHT TACTICS",
            "TFT Companion",
            &format!(
                "Set {} data, composition guides, item recipes, augments and saved board",
                crate::tft_data::current_set()
            ),
        );
        ui.horizontal_wrapped(|ui| {
            for tab in ["COMPS", "ITEMS", "ODDS", "AUGMENTS", "BOARD", "SET DATA"] {
                if action_button(
                    ui,
                    tab,
                    self.tft_tab == tab,
                    if tab == "BOARD" { GOLD } else { CYAN },
                )
                .clicked()
                {
                    self.tft_tab = tab.to_string();
                }
            }
        });
        ui.add_space(8.0);
        match self.tft_tab.as_str() {
            "ITEMS" => self.render_tft_items(ui),
            "ODDS" => self.render_tft_odds(ui),
            "AUGMENTS" => self.render_tft_augments(ui),
            "BOARD" => self.render_tft_board(ui),
            "SET DATA" => self.render_tft_set_data(ui),
            _ => self.render_tft_comps(ui),
        }
    }

    fn render_tft_comps(&mut self, ui: &mut egui::Ui) {
        let data = crate::tft_data::current_comps();
        let comps = data
            .get("comps")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let patch = data.get("patch").and_then(Value::as_str).unwrap_or("—");
        let source = data
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let str_list = |comp: &Value, key: &str| -> Vec<String> {
            comp.get(key)
                .and_then(Value::as_array)
                .map(|list| {
                    list.iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default()
        };
        if !comps.iter().any(|comp| {
            comp.get("id").and_then(Value::as_str) == Some(self.selected_tft_comp.as_str())
        }) {
            self.selected_tft_comp = comps
                .first()
                .and_then(|comp| comp.get("id"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
        }
        let query = self.tft_search.trim().to_lowercase();
        let visible = comps
            .iter()
            .filter(|comp| {
                if query.is_empty() {
                    return true;
                }
                let mut haystack = comp
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_lowercase();
                for key in ["units", "traits"] {
                    haystack.push(' ');
                    haystack.push_str(&str_list(comp, key).join(" ").to_lowercase());
                }
                haystack.contains(&query)
            })
            .cloned()
            .collect::<Vec<_>>();
        let selected = comps
            .iter()
            .find(|comp| {
                comp.get("id").and_then(Value::as_str) == Some(self.selected_tft_comp.as_str())
            })
            .cloned();

        ui.columns(2, |columns| {
            panel(&mut columns[0], |ui| {
                panel_title(
                    ui,
                    &format!("SET {} META COMPS", crate::tft_data::current_set()),
                    &format!("Top ranked comps · patch {patch}"),
                );
                ui.add_sized(
                    [ui.available_width(), 28.0],
                    egui::TextEdit::singleline(&mut self.tft_search)
                        .hint_text("Search comps, champions or traits…"),
                );
                ui.add_space(8.0);
                ScrollArea::vertical()
                    .id_salt("tft-comp-list")
                    .max_height(560.0)
                    .show(ui, |ui| {
                        for comp in &visible {
                            let id = comp.get("id").and_then(Value::as_str).unwrap_or_default();
                            let name = comp
                                .get("name")
                                .and_then(Value::as_str)
                                .unwrap_or("Composition");
                            let tier = comp.get("tier").and_then(Value::as_str).unwrap_or("—");
                            let playstyle = comp
                                .get("playstyle")
                                .and_then(Value::as_str)
                                .unwrap_or_default();
                            let avg = comp.get("avg_place").and_then(Value::as_f64).unwrap_or(0.0);
                            let is_selected = self.selected_tft_comp == id;
                            let response = Frame::new()
                                .fill(if is_selected { ACCENT_SOFT } else { CARD })
                                .stroke(Stroke::new(1.0, if is_selected { CYAN } else { BORDER }))
                                .corner_radius(egui::CornerRadius::same(10))
                                .inner_margin(egui::Margin::same(10))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        tier_badge(ui, tier);
                                        ui.label(
                                            RichText::new(name).size(12.5).strong().color(TEXT),
                                        );
                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                ui.label(
                                                    RichText::new(format!("avg {avg:.2}"))
                                                        .size(10.0)
                                                        .color(SECONDARY),
                                                );
                                            },
                                        );
                                    });
                                    ui.label(RichText::new(playstyle).size(9.5).color(GOLD));
                                    ui.add_space(4.0);
                                    ui.horizontal_wrapped(|ui| {
                                        ui.spacing_mut().item_spacing.x = 4.0;
                                        for unit in str_list(comp, "units") {
                                            self.tft_unit_icon(ui, &unit, 30.0);
                                        }
                                    });
                                })
                                .response
                                .interact(Sense::click());
                            if response.clicked() {
                                self.selected_tft_comp = id.to_string();
                            }
                            ui.add_space(6.0);
                        }
                        if visible.is_empty() {
                            ui.label(
                                RichText::new("No comps match that search.")
                                    .size(11.0)
                                    .color(SECONDARY),
                            );
                        }
                    });
            });
            panel(&mut columns[1], |ui| {
                let Some(comp) = selected.as_ref() else {
                    panel_title(ui, "NO COMPOSITION SELECTED", "Choose a comp from the list");
                    return;
                };
                let detail_width = ui.available_width();
                ScrollArea::vertical()
                    .id_salt("tft-comp-detail")
                    .max_height(620.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.set_max_width(detail_width - 12.0);
                        let name = comp
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or("Composition");
                        let tier = comp.get("tier").and_then(Value::as_str).unwrap_or("—");
                        ui.horizontal(|ui| {
                            tier_badge(ui, tier);
                            ui.label(RichText::new(name).size(18.0).strong().color(TEXT));
                        });
                        ui.add_space(6.0);
                        ui.horizontal_wrapped(|ui| {
                            let stat =
                                |ui: &mut egui::Ui, label: &str, value: String, color: Color32| {
                                    Frame::new()
                                        .fill(CARD)
                                        .stroke(Stroke::new(1.0, BORDER))
                                        .corner_radius(egui::CornerRadius::same(8))
                                        .inner_margin(egui::Margin::symmetric(10, 6))
                                        .show(ui, |ui| {
                                            ui.vertical(|ui| {
                                                ui.label(
                                                    RichText::new(label)
                                                        .size(8.5)
                                                        .strong()
                                                        .color(MUTED),
                                                );
                                                ui.label(
                                                    RichText::new(value)
                                                        .size(13.0)
                                                        .strong()
                                                        .color(color),
                                                );
                                            });
                                        });
                                };
                            let num =
                                |key: &str| comp.get(key).and_then(Value::as_f64).unwrap_or(0.0);
                            let text = |key: &str| {
                                comp.get(key)
                                    .and_then(Value::as_str)
                                    .unwrap_or("—")
                                    .to_string()
                            };
                            stat(ui, "AVG PLACE", format!("{:.2}", num("avg_place")), TEXT);
                            stat(ui, "TOP 4", format!("{:.1}%", num("top4")), GREEN);
                            stat(ui, "WIN", format!("{:.1}%", num("win")), GOLD);
                            stat(ui, "PLAYSTYLE", text("playstyle"), CYAN);
                            stat(ui, "DIFFICULTY", text("difficulty"), SECONDARY);
                        });

                        ui.add_space(12.0);
                        ui.label(RichText::new("MAIN CARRY").size(9.0).strong().color(MUTED));
                        let carry = comp
                            .get("carry")
                            .and_then(Value::as_str)
                            .unwrap_or("—")
                            .to_string();
                        let carry_items = str_list(comp, "carry_items");
                        ui.horizontal(|ui| {
                            self.tft_unit_icon(ui, &carry, 58.0);
                            ui.vertical(|ui| {
                                ui.label(RichText::new(&carry).size(14.0).strong().color(GOLD));
                                ui.horizontal(|ui| {
                                    for item in &carry_items {
                                        let texture =
                                            crate::tft_data::item_by_name(item).and_then(|entry| {
                                                self.tft_textures.get(&entry.image_key)
                                            });
                                        let response = match texture {
                                            Some(texture) => ui.add(
                                                egui::Image::new((texture.id(), Vec2::splat(30.0)))
                                                    .corner_radius(egui::CornerRadius::same(5)),
                                            ),
                                            None => {
                                                ui.label(RichText::new(item).size(10.0).color(TEXT))
                                            }
                                        };
                                        response.on_hover_text(item);
                                    }
                                });
                                ui.label(
                                    RichText::new(carry_items.join(" · "))
                                        .size(9.5)
                                        .color(SECONDARY),
                                );
                            });
                        });

                        ui.add_space(12.0);
                        ui.label(RichText::new("FINAL BOARD").size(9.0).strong().color(MUTED));
                        ui.add_space(4.0);
                        let frontline = str_list(comp, "frontline");
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
                            for unit in str_list(comp, "units") {
                                ui.allocate_ui(Vec2::new(60.0, 86.0), |ui| {
                                    ui.vertical_centered(|ui| {
                                        self.tft_unit_icon(ui, &unit, 48.0);
                                        ui.label(RichText::new(&unit).size(9.0).color(TEXT));
                                        if frontline.contains(&unit) {
                                            ui.label(
                                                RichText::new("FRONT")
                                                    .size(7.5)
                                                    .strong()
                                                    .color(MUTED),
                                            );
                                        }
                                    });
                                });
                            }
                        });

                        ui.add_space(10.0);
                        ui.label(RichText::new("TRAITS").size(9.0).strong().color(MUTED));
                        let traits = str_list(comp, "traits");
                        let row_limit = detail_width - 40.0;
                        let mut rows: Vec<Vec<String>> = vec![Vec::new()];
                        let mut row_width = 0.0;
                        for name in traits {
                            let chip_width = ui
                                .painter()
                                .layout_no_wrap(
                                    name.clone(),
                                    egui::FontId::proportional(10.0),
                                    CYAN,
                                )
                                .size()
                                .x
                                + 24.0;
                            if row_width + chip_width > row_limit
                                && !rows.last().unwrap().is_empty()
                            {
                                rows.push(Vec::new());
                                row_width = 0.0;
                            }
                            row_width += chip_width;
                            rows.last_mut().unwrap().push(name);
                        }
                        for row in rows {
                            ui.horizontal(|ui| {
                                for name in row {
                                    trait_chip(ui, &name);
                                }
                            });
                        }

                        ui.add_space(12.0);
                        if action_button(ui, "LOAD UNITS ON BOARD", false, CYAN).clicked() {
                            self.settings.tft_board.fill(String::new());
                            // Row 0 is the front row of the 4x7 board and row 3 the back row.
                            let (mut front, mut back) = (0usize, 0usize);
                            for unit in str_list(comp, "units") {
                                let cell = if frontline.contains(&unit) {
                                    front += 1;
                                    front
                                } else {
                                    back += 1;
                                    3 * 7 + back - 1
                                };
                                if cell < self.settings.tft_board.len() {
                                    self.settings.tft_board[cell] = unit;
                                }
                            }
                            self.save_settings();
                            self.tft_tab = "BOARD".to_string();
                        }
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!(
                                "Source: {source} Items and stats change every patch."
                            ))
                            .size(9.0)
                            .color(MUTED),
                        );
                    });
            });
        });
    }

    fn tft_item_icon(
        &self,
        ui: &mut egui::Ui,
        name: &str,
        size: f32,
        selected: bool,
    ) -> egui::Response {
        let texture = crate::tft_data::item_by_name(name)
            .and_then(|entry| self.tft_textures.get(&entry.image_key));
        let stroke = if selected {
            Stroke::new(2.0, GOLD)
        } else {
            Stroke::new(1.0, BORDER)
        };
        let response = Frame::new()
            .stroke(stroke)
            .fill(if selected {
                Color32::from_rgba_unmultiplied(GOLD.r(), GOLD.g(), GOLD.b(), 30)
            } else {
                Color32::TRANSPARENT
            })
            .corner_radius(egui::CornerRadius::same(6))
            .inner_margin(egui::Margin::same(2))
            .show(ui, |ui| match texture {
                Some(texture) => {
                    ui.add(
                        egui::Image::new((texture.id(), Vec2::splat(size)))
                            .corner_radius(egui::CornerRadius::same(4)),
                    );
                }
                None => {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
                    ui.painter()
                        .rect_filled(rect, egui::CornerRadius::same(4), CARD);
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        initials(name),
                        egui::FontId::proportional(size * 0.32),
                        TEXT,
                    );
                }
            })
            .response
            .interact(Sense::click());
        response.on_hover_text(name)
    }

    fn tft_unit_icon(&self, ui: &mut egui::Ui, name: &str, size: f32) {
        let entry = crate::tft_data::champion_by_name(name);
        let cost = entry.and_then(|entry| entry.cost).unwrap_or(0);
        let border = tft_cost_color(cost);
        let texture = entry.and_then(|entry| self.tft_textures.get(&entry.image_key));
        let response = Frame::new()
            .stroke(Stroke::new(2.0, border))
            .corner_radius(egui::CornerRadius::same(6))
            .inner_margin(egui::Margin::same(1))
            .show(ui, |ui| match texture {
                Some(texture) => {
                    ui.add(
                        egui::Image::new((texture.id(), Vec2::splat(size)))
                            .corner_radius(egui::CornerRadius::same(5)),
                    );
                }
                None => {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
                    ui.painter()
                        .rect_filled(rect, egui::CornerRadius::same(5), CARD);
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        initials(name),
                        egui::FontId::proportional(size * 0.32),
                        TEXT,
                    );
                }
            })
            .response;
        response.on_hover_text(format!("{name} · {cost}g"));
    }

    fn render_tft_items(&mut self, ui: &mut egui::Ui) {
        let data = crate::tft_data::current_items();
        let str_of = |value: &Value| value.as_str().unwrap_or_default().to_string();
        let components: Vec<String> = data
            .get("components")
            .and_then(Value::as_array)
            .map(|list| list.iter().map(str_of).collect())
            .unwrap_or_default();
        let recipes: Vec<(String, String, String, String)> = data
            .get("recipes")
            .and_then(Value::as_array)
            .map(|list| {
                list.iter()
                    .filter_map(|recipe| {
                        let parts = recipe.get("components")?.as_array()?;
                        Some((
                            str_of(recipe.get("name")?),
                            str_of(parts.first()?),
                            str_of(parts.get(1)?),
                            recipe.get("effect").map(str_of).unwrap_or_default(),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let source = data
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or_default();
        for selected in [&mut self.tft_component_one, &mut self.tft_component_two] {
            if !components.contains(selected) {
                *selected = components.first().cloned().unwrap_or_default();
            }
        }
        let recipe_for = |a: &str, b: &str| {
            recipes
                .iter()
                .find(|(_, one, two, _)| (one == a && two == b) || (one == b && two == a))
                .cloned()
        };
        ui.columns(2, |columns| {
            panel(&mut columns[0], |ui| {
                panel_title(
                    ui,
                    &format!("SET {} CRAFTING MIXER", crate::tft_data::current_set()),
                    "Pick two components to see what they build",
                );
                for (field, slot) in [("COMPONENT A", 0), ("COMPONENT B", 1)] {
                    ui.label(RichText::new(field).size(8.5).strong().color(MUTED));
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(5.0, 5.0);
                        for component in &components {
                            let current = if slot == 0 {
                                &self.tft_component_one
                            } else {
                                &self.tft_component_two
                            };
                            let is_selected = current == component;
                            if self
                                .tft_item_icon(ui, component, 34.0, is_selected)
                                .clicked()
                            {
                                if slot == 0 {
                                    self.tft_component_one = component.clone();
                                } else {
                                    self.tft_component_two = component.clone();
                                }
                            }
                        }
                    });
                    ui.add_space(6.0);
                }

                ui.add_space(4.0);
                let (first, second) = (
                    self.tft_component_one.clone(),
                    self.tft_component_two.clone(),
                );
                Frame::new()
                    .fill(CARD)
                    .stroke(Stroke::new(
                        1.0,
                        Color32::from_rgba_unmultiplied(GOLD.r(), GOLD.g(), GOLD.b(), 120),
                    ))
                    .corner_radius(egui::CornerRadius::same(10))
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        match recipe_for(&first, &second) {
                            Some((name, _, _, effect)) => {
                                ui.horizontal(|ui| {
                                    self.tft_item_icon(ui, &name, 54.0, false);
                                    ui.vertical(|ui| {
                                        ui.label(
                                            RichText::new(&name).size(16.0).strong().color(GOLD),
                                        );
                                        ui.label(
                                            RichText::new(format!("{first} + {second}"))
                                                .size(9.5)
                                                .color(SECONDARY),
                                        );
                                        ui.add_space(3.0);
                                        ui.label(RichText::new(&effect).size(10.5).color(TEXT));
                                    });
                                });
                            }
                            None => {
                                ui.label(
                                    RichText::new(
                                        "These two components don't combine into an item.",
                                    )
                                    .size(10.5)
                                    .color(SECONDARY),
                                );
                            }
                        }
                    });

                ui.add_space(10.0);
                ui.label(
                    RichText::new(format!("EVERYTHING {} BUILDS", first.to_uppercase()))
                        .size(8.5)
                        .strong()
                        .color(MUTED),
                );
                ui.add_space(4.0);
                let builds: Vec<_> = components
                    .iter()
                    .filter_map(|other| {
                        recipe_for(&first, other).map(|recipe| (other.clone(), recipe))
                    })
                    .collect();
                egui::Grid::new("tft-builds-from-a")
                    .num_columns(2)
                    .spacing(Vec2::new(14.0, 6.0))
                    .show(ui, |ui| {
                        for (index, (other, (name, _, _, _))) in builds.iter().enumerate() {
                            let response = ui
                                .horizontal(|ui| {
                                    self.tft_item_icon(ui, other, 22.0, false);
                                    ui.label(RichText::new("→").size(11.0).color(MUTED));
                                    self.tft_item_icon(ui, name, 26.0, false);
                                    ui.label(
                                        RichText::new(name).size(10.0).color(if *other == second {
                                            GOLD
                                        } else {
                                            TEXT
                                        }),
                                    );
                                })
                                .response
                                .interact(Sense::click());
                            if response.clicked() {
                                self.tft_component_two = other.clone();
                            }
                            if index % 2 == 1 {
                                ui.end_row();
                            }
                        }
                    });
                ui.add_space(8.0);
                ui.label(RichText::new(source).size(9.0).color(MUTED));
            });
            panel(&mut columns[1], |ui| {
                panel_title(
                    ui,
                    &format!("SET {} ITEM DATA", crate::tft_data::current_set()),
                    "Current Data Dragon item names",
                );
                ui.add_sized(
                    [ui.available_width(), 27.0],
                    egui::TextEdit::singleline(&mut self.tft_search)
                        .hint_text("Search current set items…"),
                );
                let query = self.tft_search.to_lowercase();
                ScrollArea::vertical().max_height(470.0).show(ui, |ui| {
                    for item in crate::tft_data::items()
                        .iter()
                        .filter(|item| item.name.to_lowercase().contains(&query))
                    {
                        ui.horizontal(|ui| {
                            texture_icon(ui, &self.tft_textures, &item.image_key, 30.0);
                            ui.label(RichText::new(&item.name).size(10.5).strong().color(TEXT));
                        });
                        ui.separator();
                    }
                });
            });
        });
    }

    fn render_tft_odds(&self, ui: &mut egui::Ui) {
        let data = crate::tft_data::current_odds();
        let levels = data
            .get("levels")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let numbers = |key: &str| -> Vec<u64> {
            data.get(key)
                .and_then(Value::as_array)
                .map(|list| list.iter().filter_map(Value::as_u64).collect())
                .unwrap_or_default()
        };
        let pool = numbers("pool_copies");
        let per_cost = numbers("champions_per_cost");
        let patch = data.get("patch").and_then(Value::as_str).unwrap_or("—");
        let source = data
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let colors = [1u32, 2, 3, 4, 5].map(tft_cost_color);

        panel(ui, |ui| {
            panel_title(
                ui,
                &format!("SET {} SHOP ODDS", crate::tft_data::current_set()),
                &format!("Chance per shop slot by level · patch {patch}"),
            );
            ui.horizontal(|ui| {
                for (index, color) in colors.iter().enumerate() {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                    ui.painter()
                        .rect_filled(rect, egui::CornerRadius::same(2), *color);
                    ui.label(
                        RichText::new(format!("{}-cost", index + 1))
                            .size(9.5)
                            .color(SECONDARY),
                    );
                    ui.add_space(8.0);
                }
            });
            ui.add_space(8.0);
            egui::Grid::new("tft-odds-table")
                .num_columns(8)
                .spacing([14.0, 7.0])
                .min_col_width(0.0)
                .show(ui, |ui| {
                    for header in ["LEVEL", "", "1", "2", "3", "4", "5", "XP TO NEXT"] {
                        ui.label(RichText::new(header).size(9.0).strong().color(MUTED));
                    }
                    ui.end_row();
                    for row in &levels {
                        let level = row.get("level").and_then(Value::as_u64).unwrap_or(0);
                        let odds: Vec<u64> = row
                            .get("odds")
                            .and_then(Value::as_array)
                            .map(|list| list.iter().filter_map(Value::as_u64).collect())
                            .unwrap_or_default();
                        ui.label(
                            RichText::new(format!("Lv {level}"))
                                .size(12.0)
                                .strong()
                                .color(TEXT),
                        );
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::new(260.0, 16.0), Sense::hover());
                        ui.painter()
                            .rect_filled(rect, egui::CornerRadius::same(4), CARD);
                        let mut x = rect.left();
                        for (index, value) in odds.iter().enumerate() {
                            let width = rect.width() * (*value as f32 / 100.0);
                            if width > 0.0 {
                                let segment = egui::Rect::from_min_size(
                                    egui::pos2(x, rect.top()),
                                    Vec2::new(width, rect.height()),
                                );
                                ui.painter().rect_filled(
                                    segment,
                                    egui::CornerRadius::ZERO,
                                    colors[index],
                                );
                                x += width;
                            }
                        }
                        for (index, value) in odds.iter().enumerate() {
                            let color = if *value == 0 { MUTED } else { colors[index] };
                            ui.label(
                                RichText::new(format!("{value}%"))
                                    .size(11.0)
                                    .strong()
                                    .color(color),
                            );
                        }
                        let xp = row
                            .get("xp_to_next")
                            .and_then(Value::as_u64)
                            .map(|xp| format!("{xp} XP"))
                            .unwrap_or_else(|| "max".to_string());
                        ui.label(RichText::new(xp).size(10.5).color(SECONDARY));
                        ui.end_row();
                    }
                });

            if pool.len() == 5 {
                ui.add_space(14.0);
                ui.label(
                    RichText::new("CHAMPION POOL")
                        .size(9.0)
                        .strong()
                        .color(MUTED),
                );
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    for index in 0..5 {
                        let champions = per_cost.get(index).copied().unwrap_or(0);
                        Frame::new()
                            .fill(CARD)
                            .stroke(Stroke::new(1.0, colors[index]))
                            .corner_radius(egui::CornerRadius::same(8))
                            .inner_margin(egui::Margin::symmetric(10, 6))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new(format!("{}-COST", index + 1))
                                            .size(8.5)
                                            .strong()
                                            .color(colors[index]),
                                    );
                                    ui.label(
                                        RichText::new(format!("{} copies each", pool[index]))
                                            .size(11.5)
                                            .strong()
                                            .color(TEXT),
                                    );
                                    if champions > 0 {
                                        ui.label(
                                            RichText::new(format!(
                                                "{champions} champions · {} total",
                                                champions * pool[index]
                                            ))
                                            .size(9.0)
                                            .color(SECONDARY),
                                        );
                                    }
                                });
                            });
                    }
                });
            }
            ui.add_space(10.0);
            ui.label(RichText::new(source).size(9.0).color(MUTED));
        });
    }

    fn render_tft_augments(&mut self, ui: &mut egui::Ui) {
        panel(ui, |ui| {
            panel_title(
                ui,
                &format!("SET {} AUGMENTS", crate::tft_data::current_set()),
                "Current Data Dragon augment names and descriptions",
            );
            ui.add_space(6.0);
            ui.add_sized(
                [ui.available_width(), 27.0],
                egui::TextEdit::singleline(&mut self.tft_search)
                    .hint_text("Search augment name or description…"),
            );
            let query = self.tft_search.to_lowercase();
            ScrollArea::vertical().max_height(520.0).show(ui, |ui| {
                for augment in crate::tft_data::augments().iter().filter(|augment| {
                    augment.name.to_lowercase().contains(&query)
                        || augment.description.to_lowercase().contains(&query)
                }) {
                    ui.horizontal(|ui| {
                        texture_icon(ui, &self.tft_textures, &augment.image_key, 34.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&augment.name).size(11.0).strong().color(TEXT));
                            ui.label(
                                RichText::new(clean_tft_text(&augment.description))
                                    .size(9.5)
                                    .color(SECONDARY),
                            );
                        });
                    });
                    ui.separator();
                }
            });
        });
    }

    fn render_tft_set_data(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for category in ["UNITS", "TRAITS", "ITEMS"] {
                if action_button(ui, category, self.tft_data_category == category, CYAN).clicked() {
                    self.tft_data_category = category.to_string();
                }
            }
            ui.add_sized(
                [260.0, 27.0],
                egui::TextEdit::singleline(&mut self.tft_search)
                    .hint_text("Search current set data…"),
            );
        });
        let query = self.tft_search.trim().to_lowercase();
        let category = self.tft_data_category.clone();
        panel(ui, |ui| {
            let (title, source) = match category.as_str() {
                "TRAITS" => ("TRAITS", crate::tft_data::traits()),
                "ITEMS" => ("ITEMS", crate::tft_data::items()),
                _ => ("UNITS", crate::tft_data::champions()),
            };
            let mut entries: Vec<&crate::tft_data::TftEntry> = source
                .iter()
                .filter(|entry| query.is_empty() || entry.name.to_lowercase().contains(&query))
                .collect();
            entries.sort_by(|a, b| {
                a.cost
                    .unwrap_or(0)
                    .cmp(&b.cost.unwrap_or(0))
                    .then(a.name.cmp(&b.name))
            });
            panel_title(
                ui,
                &format!("SET {} {title}", crate::tft_data::current_set()),
                &format!("{} of {} entries", entries.len(), source.len()),
            );
            let placing = category == "UNITS" && self.tft_selected_cell.is_some();
            // Tiles are 96px wide + 12px margin + 2px stroke, with an 8px gap.
            let per_row = ((ui.available_width() - 16.0) / 118.0).floor().max(1.0) as usize;
            ScrollArea::vertical()
                .id_salt("tft-set-data-scroll")
                .max_height(540.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
                    for row in entries.chunks(per_row) {
                        ui.horizontal(|ui| {
                            for entry in row {
                                let tile = Frame::new()
                                    .fill(CARD)
                                    .stroke(Stroke::new(1.0, BORDER))
                                    .corner_radius(egui::CornerRadius::same(8))
                                    .inner_margin(egui::Margin::same(6))
                                    .show(ui, |ui| {
                                        ui.set_width(96.0);
                                        ui.vertical_centered(|ui| {
                                            if category == "UNITS" {
                                                self.tft_unit_icon(ui, &entry.name, 46.0);
                                            } else {
                                                texture_icon(
                                                    ui,
                                                    &self.tft_textures,
                                                    &entry.image_key,
                                                    46.0,
                                                );
                                            }
                                            ui.add(
                                                egui::Label::new(
                                                    RichText::new(&entry.name)
                                                        .size(10.0)
                                                        .strong()
                                                        .color(TEXT),
                                                )
                                                .truncate(),
                                            );
                                            if let (Some(cost), "UNITS") =
                                                (entry.cost, category.as_str())
                                            {
                                                ui.label(
                                                    RichText::new(format!("{cost}g"))
                                                        .size(9.0)
                                                        .color(tft_cost_color(cost)),
                                                );
                                            }
                                        });
                                    })
                                    .response
                                    .interact(Sense::click())
                                    .on_hover_text(&entry.name);
                                if placing && tile.clicked() {
                                    self.tft_unit_input = entry.name.clone();
                                    if let Some(cell) = self
                                        .tft_selected_cell
                                        .and_then(|index| self.settings.tft_board.get_mut(index))
                                    {
                                        *cell = entry.name.clone();
                                    }
                                    self.save_settings();
                                    self.tft_tab = "BOARD".to_string();
                                }
                            }
                        });
                    }
                });
        });
        if self.tft_selected_cell.is_none() && self.tft_data_category == "UNITS" {
            ui.label(
                RichText::new(
                    "Select a board hex on the Board tab, then click a unit here to place it.",
                )
                .size(10.0)
                .color(MUTED),
            );
        }
    }

    fn render_tft_board(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            panel(&mut columns[0], |ui| {
                panel_title(ui, "COMPOSITION BOARD", "Select a hex, then enter a unit name");
                for row in 0..4 {
                    ui.horizontal(|ui| {
                        ui.add_space(if row % 2 == 1 { 20.0 } else { 0.0 });
                        for col in 0..7 {
                            let index = row * 7 + col;
                            let name = self.settings.tft_board.get(index).map(String::as_str).unwrap_or("");
                            let label = if name.is_empty() { format!("{}·{}", row + 1, col + 1) } else { name.chars().take(8).collect() };
                            let selected = self.tft_selected_cell == Some(index);
                            let button = egui::Button::new(RichText::new(label).size(9.5).strong().color(if selected { CYAN } else if name.is_empty() { MUTED } else { TEXT }))
                                .fill(if selected { ACCENT_SOFT } else { CARD })
                                .stroke(Stroke::new(1.0, if selected { CYAN } else { BORDER }));
                            if ui.add_sized([54.0, 42.0], button).clicked() {
                                self.tft_selected_cell = Some(index);
                                self.tft_unit_input = name.to_string();
                            }
                        }
                    });
                    ui.add_space(5.0);
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("UNIT").size(10.0).strong().color(MUTED));
                    ui.add_sized([180.0, 27.0], egui::TextEdit::singleline(&mut self.tft_unit_input).hint_text("Enter a unit name"));
                    if action_button(ui, "PLACE", false, CYAN).clicked() {
                        if let Some(index) = self.tft_selected_cell {
                            if let Some(cell) = self.settings.tft_board.get_mut(index) {
                                *cell = self.tft_unit_input.trim().to_string();
                                self.save_settings();
                            }
                        } else {
                            self.set_notice("Select a board hex first");
                        }
                    }
                });
                ui.horizontal(|ui| {
                    if action_button(ui, "REMOVE UNIT", false, RED).clicked() {
                        if let Some(index) = self.tft_selected_cell {
                            if let Some(cell) = self.settings.tft_board.get_mut(index) {
                                cell.clear();
                                self.tft_unit_input.clear();
                                self.save_settings();
                            }
                        }
                    }
                    if action_button(ui, "CLEAR BOARD", false, RED).clicked() {
                        self.settings.tft_board.fill(String::new());
                        self.tft_unit_input.clear();
                        self.tft_selected_cell = None;
                        self.save_settings();
                    }
                    if action_button(ui, "BROWSE CURRENT SET UNITS", false, CYAN).clicked() {
                        self.tft_data_category = "UNITS".to_string();
                        self.tft_tab = "SET DATA".to_string();
                    }
                });
            });
            panel(&mut columns[1], |ui| {
                panel_title(ui, "BOARD SUMMARY", "Saved locally in RiftCore preferences");
                let filled = self.settings.tft_board.iter().filter(|unit| !unit.trim().is_empty()).count();
                metric(ui, "UNITS PLACED", &filled.to_string(), CYAN);
                ui.add_space(9.0);
                for row in (0..4).rev() {
                    let names = (0..7).filter_map(|col| {
                        let value = self.settings.tft_board.get(row * 7 + col)?;
                        (!value.trim().is_empty()).then_some(value.as_str())
                    }).collect::<Vec<_>>();
                    if !names.is_empty() {
                        ui.label(RichText::new(format!("ROW {}  {}", row + 1, names.join("  ·  "))).size(10.5).color(SECONDARY));
                    }
                }
                if filled == 0 {
                    ui.label(RichText::new("Your board is empty. Select a hex and add a unit name.").size(11.0).color(SECONDARY));
                }
                ui.add_space(12.0);
                ui.label(RichText::new("This board saves locally. Load a Set 18 comp from the Comps tab, or browse every unit in Set Data.").size(10.0).color(MUTED));
            });
        });
    }

    fn render_tier_list(&mut self, ui: &mut egui::Ui) {
        self.render_page_heading(
            ui,
            "PATCH BUILD GUIDE",
            "Tier List & Builds",
            "Current champion roster, role-specific build paths, rune styles and skill orders",
        );
        ui.horizontal_wrapped(|ui| {
            for role in ["ALL", "TOP", "JUNGLE", "MID", "ADC", "SUPPORT"] {
                if action_button(ui, role, self.tier_role == role, CYAN).clicked() {
                    self.tier_role = role.to_string();
                }
            }
            ui.add_sized(
                [240.0, 28.0],
                egui::TextEdit::singleline(&mut self.tier_search).hint_text("Search champions…"),
            );
            if action_button(ui, "TABLE", !self.tier_view_cards, CYAN).clicked() {
                self.tier_view_cards = false;
            }
            if action_button(ui, "CARDS", self.tier_view_cards, CYAN).clicked() {
                self.tier_view_cards = true;
            }
            egui::ComboBox::from_id_salt("tier-sort")
                .selected_text(format!("SORT: {}", self.tier_sort))
                .show_ui(ui, |ui| {
                    for sort in ["NAME", "ROLE", "ARCHETYPE"] {
                        ui.selectable_value(&mut self.tier_sort, sort.to_string(), sort);
                    }
                });
        });
        ui.label(RichText::new("Build paths are role-based guidance. Riot Data Dragon does not publish live win-rate or pick-rate statistics, so this view does not invent meta percentages.").size(10.5).color(MUTED));
        ui.add_space(10.0);
        let search = self.tier_search.to_lowercase();
        let role = self.tier_role.clone();
        let sort = self.tier_sort.clone();
        let mut champions: Vec<Champion> = self
            .champions
            .iter()
            .filter(|champion| {
                let role_matches = role == "ALL"
                    || champion
                        .roles
                        .iter()
                        .any(|entry| role_matches(entry, &role));
                let search_matches = search.is_empty()
                    || champion.name.to_lowercase().contains(&search)
                    || champion.title.to_lowercase().contains(&search);
                role_matches && search_matches
            })
            .cloned()
            .collect();
        champions.sort_by(|left, right| {
            let left_build = crate::builds::recommend(&left.key, &left.name, &left.roles, None);
            let right_build = crate::builds::recommend(&right.key, &right.name, &right.roles, None);
            match sort.as_str() {
                "ROLE" => left_build
                    .role
                    .cmp(&right_build.role)
                    .then(left.name.cmp(&right.name)),
                "ARCHETYPE" => left_build
                    .archetype
                    .cmp(&right_build.archetype)
                    .then(left.name.cmp(&right.name)),
                _ => left.name.cmp(&right.name),
            }
        });
        panel(ui, |ui| {
            panel_title(
                ui,
                if self.tier_view_cards {
                    "BUILD CARDS"
                } else {
                    "BUILD LEADERBOARD"
                },
                &format!(
                    "{} champions match these filters · guidance only, no fabricated rates",
                    champions.len()
                ),
            );
            ScrollArea::vertical().max_height(540.0).show(ui, |ui| {
                if !self.tier_view_cards {
                    egui::Grid::new("tier-table")
                        .num_columns(5)
                        .spacing([12.0, 7.0])
                        .show(ui, |ui| {
                            for heading in ["CHAMPION", "ROLE", "BUILD STYLE", "SKILL ORDER", ""] {
                                ui.label(RichText::new(heading).size(9.0).strong().color(MUTED));
                            }
                            ui.end_row();
                            for champion in &champions {
                                let build = crate::builds::recommend(
                                    &champion.key,
                                    &champion.name,
                                    &champion.roles,
                                    None,
                                );
                                ui.horizontal(|ui| {
                                    avatar(
                                        ui,
                                        &champion.name,
                                        28.0,
                                        CYAN,
                                        self.champion_textures.get(&champion.key),
                                    );
                                    ui.label(
                                        RichText::new(&champion.name)
                                            .size(11.0)
                                            .strong()
                                            .color(TEXT),
                                    );
                                });
                                ui.label(RichText::new(&build.role).size(10.0).color(CYAN));
                                ui.label(
                                    RichText::new(build.archetype.replace('_', " "))
                                        .size(9.5)
                                        .color(SECONDARY),
                                );
                                ui.label(
                                    RichText::new(&build.skill_priority)
                                        .size(10.0)
                                        .strong()
                                        .color(GOLD),
                                );
                                if action_button(ui, "OPEN BUILD", false, CYAN).clicked() {
                                    self.selected_champion = champion.id;
                                    self.build_role = build.role;
                                    self.active_page = Page::ChampSelect;
                                }
                                ui.end_row();
                            }
                        });
                } else {
                    egui::Grid::new("tier-cards")
                        .num_columns(3)
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            for (index, champion) in champions.iter().enumerate() {
                                let build = crate::builds::recommend(
                                    &champion.key,
                                    &champion.name,
                                    &champion.roles,
                                    None,
                                );
                                let response = ui.allocate_ui_with_layout(
                                    Vec2::new(176.0, 76.0),
                                    Layout::left_to_right(Align::Center),
                                    |ui| {
                                        avatar(
                                            ui,
                                            &champion.name,
                                            42.0,
                                            CYAN,
                                            self.champion_textures.get(&champion.key),
                                        );
                                        ui.vertical(|ui| {
                                            ui.label(
                                                RichText::new(&champion.name)
                                                    .size(11.5)
                                                    .strong()
                                                    .color(TEXT),
                                            );
                                            ui.label(
                                                RichText::new(format!(
                                                    "{} · {}",
                                                    build.role,
                                                    build.archetype.replace('_', " ")
                                                ))
                                                .size(9.0)
                                                .color(CYAN),
                                            );
                                            ui.label(
                                                RichText::new(format!(
                                                    "Skill: {}",
                                                    build.skill_priority
                                                ))
                                                .size(9.5)
                                                .color(GOLD),
                                            );
                                            if action_button(ui, "OPEN SETUP", false, CYAN)
                                                .clicked()
                                            {
                                                self.selected_champion = champion.id;
                                                self.build_role = build.role.clone();
                                                self.active_page = Page::ChampSelect;
                                            }
                                        });
                                    },
                                );
                                let _ = response;
                                if index % 4 == 3 {
                                    ui.end_row();
                                }
                            }
                        });
                }
            });
        });
    }

    fn render_champion_pool(&mut self, ui: &mut egui::Ui) {
        self.render_page_heading(
            ui,
            "PERSONAL PICKS",
            "Champion Pool",
            "Build a three-pick plan for each role: blind, counter and pocket picks",
        );
        ui.horizontal_wrapped(|ui| {
            for role in ["TOP", "JUNGLE", "MID", "ADC", "SUPPORT"] {
                if action_button(ui, role, self.pool_role == role, CYAN).clicked() {
                    self.pool_role = role.to_string();
                    self.pool_assign_slot = None;
                }
            }
        });
        let mut slots = self
            .settings
            .champion_pool
            .get(&self.pool_role)
            .cloned()
            .unwrap_or_default();
        slots.resize(3, 0);
        slots.truncate(3);
        ui.add_space(8.0);
        panel(ui, |ui| {
            panel_title(
                ui,
                &format!("{} ROLE PLAN", self.pool_role),
                "Choose a blind pick, a counter pick and a pocket pick",
            );
            ui.columns(3, |columns| {
                for (index, (label, accent)) in [
                    ("BLIND PICK", CYAN),
                    ("COUNTER PICK", GOLD),
                    ("POCKET PICK", RED),
                ]
                .into_iter()
                .enumerate()
                {
                    let champion_id = slots[index];
                    let champion = self
                        .champions
                        .iter()
                        .find(|champion| champion.id == champion_id)
                        .cloned();
                    columns[index].vertical_centered(|ui| {
                        ui.label(RichText::new(label).size(9.0).strong().color(accent));
                        if let Some(champion) = champion.as_ref() {
                            avatar(
                                ui,
                                &champion.name,
                                50.0,
                                accent,
                                self.champion_textures.get(&champion.key),
                            );
                            ui.label(
                                RichText::new(&champion.name)
                                    .size(11.0)
                                    .strong()
                                    .color(TEXT),
                            );
                            if action_button(ui, "OPEN BUILD", false, CYAN).clicked() {
                                self.selected_champion = champion.id;
                                self.build_role = self.pool_role.clone();
                                self.active_page = Page::ChampSelect;
                            }
                        } else {
                            ui.add_space(35.0);
                            ui.label(RichText::new("Empty slot").size(10.0).color(MUTED));
                            ui.add_space(35.0);
                        }
                        if action_button(
                            ui,
                            if self.pool_assign_slot == Some(index) {
                                "SELECTING…"
                            } else {
                                "CHOOSE"
                            },
                            self.pool_assign_slot == Some(index),
                            accent,
                        )
                        .clicked()
                        {
                            self.pool_assign_slot = Some(index);
                        }
                        if champion_id != 0
                            && action_button(ui, "CLEAR", false, SECONDARY).clicked()
                        {
                            slots[index] = 0;
                            self.settings
                                .champion_pool
                                .insert(self.pool_role.clone(), slots.clone());
                            self.save_settings();
                        }
                    });
                }
            });
        });
        ui.add_space(9.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("CHAMPION SEARCH")
                    .size(10.0)
                    .strong()
                    .color(MUTED),
            );
            ui.add_sized(
                [260.0, 28.0],
                egui::TextEdit::singleline(&mut self.pool_search)
                    .hint_text("Type a champion name…"),
            );
            if let Some(slot) = self.pool_assign_slot {
                ui.label(
                    RichText::new(format!(
                        "Select a champion for the {} slot",
                        ["blind", "counter", "pocket"][slot]
                    ))
                    .size(10.0)
                    .color(GOLD),
                );
            } else {
                ui.label(
                    RichText::new(format!(
                        "{} favorites",
                        self.settings.favorite_champions.len()
                    ))
                    .size(10.0)
                    .color(GOLD),
                );
            }
        });
        let search = self.pool_search.to_lowercase();
        let champions: Vec<Champion> = self
            .champions
            .iter()
            .filter(|champion| search.is_empty() || champion.name.to_lowercase().contains(&search))
            .cloned()
            .collect();
        ui.columns(4, |columns| {
            for (index, champion) in champions.iter().enumerate() {
                let favorite = self.settings.favorite_champions.contains(&champion.id);
                let column = index % 4;
                columns[column].horizontal(|ui| {
                    if ui
                        .add_sized(
                            [28.0, 29.0],
                            egui::Button::new(if favorite { "★" } else { "☆" })
                                .fill(if favorite {
                                    Color32::from_rgb(38, 31, 25)
                                } else {
                                    CARD
                                })
                                .stroke(Stroke::new(1.0, if favorite { GOLD } else { BORDER })),
                        )
                        .clicked()
                    {
                        if favorite {
                            self.settings
                                .favorite_champions
                                .retain(|id| *id != champion.id);
                        } else {
                            self.settings.favorite_champions.push(champion.id);
                        }
                        self.save_settings();
                    }
                    if let Some(slot) = self.pool_assign_slot {
                        if ui
                            .add_sized(
                                [118.0, 29.0],
                                egui::Button::new(champion.name.as_str())
                                    .fill(CARD)
                                    .stroke(Stroke::new(1.0, BORDER)),
                            )
                            .clicked()
                        {
                            let mut role_slots = self
                                .settings
                                .champion_pool
                                .get(&self.pool_role)
                                .cloned()
                                .unwrap_or_else(|| vec![0; 3]);
                            role_slots.resize(3, 0);
                            role_slots[slot] = champion.id;
                            self.settings
                                .champion_pool
                                .insert(self.pool_role.clone(), role_slots);
                            self.pool_assign_slot = None;
                            self.save_settings();
                        }
                    } else {
                        let width = (ui.available_width() - 30.0).max(70.0);
                        if ui
                            .add_sized(
                                [width, 29.0],
                                egui::Button::new(champion.name.as_str())
                                    .fill(CARD)
                                    .stroke(Stroke::new(1.0, BORDER)),
                            )
                            .clicked()
                        {
                            self.selected_champion = champion.id;
                            self.active_page = Page::ChampSelect;
                        }
                    }
                });
            }
        });
    }

    fn render_profile(&mut self, ui: &mut egui::Ui) {
        self.render_page_heading(
            ui,
            "SUMMONER INSIGHTS",
            "Profile & Analysis",
            "Local match history, post-game breakdowns and Riot ID scouting",
        );
        self.render_profile_summary(ui);
        self.render_local_match_history(ui);
        self.render_riot_search(ui);
        self.render_riot_lookup(ui);
        self.render_match_analysis(ui);
    }

    fn render_profile_summary(&mut self, ui: &mut egui::Ui) {
        if let Some(summoner) = self.snapshot.summoner.as_ref() {
            panel(ui, |ui| {
                ui.horizontal(|ui| {
                    avatar(ui, "R", 58.0, CYAN, None);
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(
                                    summoner
                                        .get("gameName")
                                        .and_then(Value::as_str)
                                        .unwrap_or("Summoner"),
                                )
                                .size(22.0)
                                .strong()
                                .color(TEXT),
                            );
                            ui.add_space(6.0);
                            badge(
                                ui,
                                &format!("Level {}", num(summoner, "summonerLevel")),
                                CYAN,
                            );
                        });
                        ui.label(
                            RichText::new(format!(
                                "#{}",
                                summoner
                                    .get("tagLine")
                                    .and_then(Value::as_str)
                                    .unwrap_or("—")
                            ))
                            .size(12.0)
                            .color(SECONDARY),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if action_button(ui, "REFRESH PROFILE", false, CYAN).clicked() {
                            let _ = self.commands.send(CoreCommand::Refresh);
                        }
                    });
                });
            });
            ui.add_space(12.0);
            panel(ui, |ui| {
                panel_title(
                    ui,
                    "RANKED QUEUES",
                    "Current season rank and ladder progress",
                );
                if let Some(queues) = self
                    .snapshot
                    .ranked
                    .as_ref()
                    .and_then(|ranked| ranked.get("queues"))
                    .and_then(Value::as_array)
                {
                    ui.columns(2, |cols| {
                        for (idx, queue) in queues.iter().enumerate() {
                            let col = &mut cols[idx % 2];
                            let queue_name = queue
                                .get("queueType")
                                .and_then(Value::as_str)
                                .unwrap_or("Ranked");
                            let friendly_name = match queue_name {
                                "RANKED_SOLO_5x5" => "RANKED SOLO / DUO",
                                "RANKED_FLEX_SR" => "RANKED FLEX 5v5",
                                "RANKED_TFT" => "TFT RANKED",
                                "RANKED_TFT_DOUBLE_UP" => "TFT DOUBLE UP",
                                other => other,
                            };
                            let tier = queue
                                .get("tier")
                                .and_then(Value::as_str)
                                .unwrap_or("UNRANKED");
                            let division = queue
                                .get("division")
                                .and_then(Value::as_str)
                                .unwrap_or_default();
                            let lp = num(queue, "leaguePoints");
                            let wins = num(queue, "wins").parse::<u32>().unwrap_or(0);
                            let losses = num(queue, "losses").parse::<u32>().unwrap_or(0);
                            let total = wins + losses;
                            let wr = if total > 0 {
                                (wins as f32 / total as f32) * 100.0
                            } else {
                                0.0
                            };
                            let color = tier_color(tier);

                            Frame::new()
                                .fill(INSET)
                                .stroke(Stroke::new(1.0, BORDER))
                                .corner_radius(egui::CornerRadius::same(10))
                                .inner_margin(egui::Margin::same(12))
                                .show(col, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(friendly_name)
                                                .size(12.0)
                                                .strong()
                                                .color(TEXT),
                                        );
                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                badge(ui, &format!("{tier} {division}"), color);
                                            },
                                        );
                                    });
                                    ui.add_space(6.0);
                                    let lp_val = lp.parse::<f32>().unwrap_or(0.0);
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(format!("{lp} LP"))
                                                .size(16.0)
                                                .strong()
                                                .color(color),
                                        );
                                        ui.label(RichText::new("/ 100 LP").size(10.0).color(MUTED));
                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                if total > 0 {
                                                    badge(
                                                        ui,
                                                        &format!("{wr:.1}% WR"),
                                                        if wr >= 55.0 {
                                                            GREEN
                                                        } else if wr >= 48.0 {
                                                            CYAN
                                                        } else {
                                                            SECONDARY
                                                        },
                                                    );
                                                }
                                            },
                                        );
                                    });
                                    custom_progress_bar(ui, lp_val / 100.0, color, 6.0);
                                    ui.add_space(6.0);
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(format!("{wins}W  {losses}L"))
                                                .size(11.0)
                                                .color(SECONDARY),
                                        );
                                        if total > 0 {
                                            ui.with_layout(
                                                Layout::right_to_left(Align::Center),
                                                |ui| {
                                                    ui.label(
                                                        RichText::new(format!(
                                                            "{total} Total Matches"
                                                        ))
                                                        .size(9.5)
                                                        .color(MUTED),
                                                    );
                                                },
                                            );
                                        }
                                    });
                                });
                            col.add_space(8.0);
                        }
                    });
                } else {
                    ui.label(
                        RichText::new("Ranked data will appear when the League Client is online.")
                            .size(11.5)
                            .color(SECONDARY),
                    );
                }
            });
        } else {
            panel(ui, |ui| {
                ui.label(
                    RichText::new(if self.snapshot.connected {
                        "Loading summoner profile…"
                    } else {
                        "Connect to the League Client to load your profile."
                    })
                    .size(12.0)
                    .color(SECONDARY),
                );
            });
        }
    }

    fn render_local_match_history(&mut self, ui: &mut egui::Ui) {
        let puuid = self
            .snapshot
            .summoner
            .as_ref()
            .and_then(|summoner| summoner.get("puuid"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        let games = self
            .snapshot
            .match_history
            .as_ref()
            .and_then(|history| history.get("games"))
            .and_then(|games| games.get("games"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        panel(ui, |ui| {
            panel_title(
                ui,
                "RECENT LOCAL MATCHES",
                "Loaded directly from your League Client history",
            );
            if games.is_empty() {
                ui.label(
                    RichText::new(if self.snapshot.connected {
                        "No recent match history is available yet."
                    } else {
                        "Connect the League Client to load recent matches."
                    })
                    .size(10.5)
                    .color(SECONDARY),
                );
            }
            for game in &games {
                let Some(summary) = normalize_local_match(game, puuid, &self.champions) else {
                    continue;
                };
                ui.horizontal(|ui| {
                    let win = summary.get("win").and_then(Value::as_bool).unwrap_or(false);
                    ui.label(
                        RichText::new(if win { "WIN" } else { "LOSS" })
                            .size(9.5)
                            .strong()
                            .color(if win { GREEN } else { RED }),
                    );
                    ui.label(
                        RichText::new(
                            summary
                                .get("championName")
                                .and_then(Value::as_str)
                                .unwrap_or("Champion"),
                        )
                        .size(10.5)
                        .strong()
                        .color(TEXT),
                    );
                    ui.label(
                        RichText::new(format!(
                            "{} / {} / {}",
                            summary.get("kills").and_then(Value::as_u64).unwrap_or(0),
                            summary.get("deaths").and_then(Value::as_u64).unwrap_or(0),
                            summary.get("assists").and_then(Value::as_u64).unwrap_or(0)
                        ))
                        .size(10.0)
                        .color(SECONDARY),
                    );
                    ui.label(
                        RichText::new(format!(
                            "{} CS  ·  {:.1}/m",
                            summary.get("cs").and_then(Value::as_u64).unwrap_or(0),
                            summary
                                .get("csPerMin")
                                .and_then(Value::as_f64)
                                .unwrap_or(0.0)
                        ))
                        .size(9.5)
                        .color(GOLD),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if action_button(ui, "ANALYZE", false, CYAN).clicked() {
                            self.selected_analysis_match = Some(summary.clone());
                        }
                    });
                });
                ui.separator();
            }
        });
    }

    fn render_riot_search(&mut self, ui: &mut egui::Ui) {
        panel(ui, |ui| {
            panel_title(
                ui,
                "RIOT ID SCOUT",
                "Ranked entries, champion mastery and recent match history",
            );
            ui.horizontal_wrapped(|ui| {
                ui.add_sized(
                    [280.0, 30.0],
                    egui::TextEdit::singleline(&mut self.riot_id_search).hint_text("GameName#TAG"),
                );
                if action_button(
                    ui,
                    if self.profile_lookup_loading {
                        "SEARCHING…"
                    } else {
                        "SCOUT ACCOUNT"
                    },
                    self.profile_lookup_loading,
                    CYAN,
                )
                .clicked()
                    && !self.profile_lookup_loading
                {
                    self.profile_lookup_loading = true;
                    let _ = self.commands.send(CoreCommand::LookupRiotId {
                        riot_id: self.riot_id_search.trim().to_string(),
                        region: self.settings.riot_region.clone(),
                        api_key: self.settings.riot_api_key.clone(),
                    });
                }
                if self.settings.riot_api_key.trim().is_empty() {
                    ui.label(
                        RichText::new("Add an API key in Preferences to use Riot ID scouting.")
                            .size(9.5)
                            .color(GOLD),
                    );
                }
            });
        });
    }

    fn render_riot_lookup(&mut self, ui: &mut egui::Ui) {
        let Some(lookup) = self.snapshot.riot_lookup.clone() else {
            return;
        };
        if let Some(error) = lookup.get("error").and_then(Value::as_str) {
            panel(ui, |ui| {
                panel_title(ui, "SCOUT LOOKUP", "Riot API response");
                ui.label(RichText::new(error).size(11.0).color(RED));
            });
            return;
        }
        let account = lookup.get("account").unwrap_or(&Value::Null);
        let summoner = lookup.get("summoner").unwrap_or(&Value::Null);
        let display_name = format!(
            "{}#{}",
            account
                .get("gameName")
                .and_then(Value::as_str)
                .unwrap_or("Player"),
            account
                .get("tagLine")
                .and_then(Value::as_str)
                .unwrap_or("—")
        );
        panel(ui, |ui| {
            panel_title(
                ui,
                &display_name,
                &format!(
                    "{}  ·  Level {}",
                    lookup
                        .get("region")
                        .and_then(Value::as_str)
                        .unwrap_or("—")
                        .to_uppercase(),
                    num(summoner, "summonerLevel")
                ),
            );
            ui.label(
                RichText::new("RANKED QUEUES")
                    .size(9.0)
                    .strong()
                    .color(MUTED),
            );
            let ranked = lookup.get("ranked").and_then(Value::as_array);
            if let Some(entries) = ranked {
                if entries.is_empty() {
                    ui.label(
                        RichText::new("Unranked or no ranked entries found.")
                            .size(10.0)
                            .color(SECONDARY),
                    );
                }
                for entry in entries {
                    let queue = entry
                        .get("queueType")
                        .and_then(Value::as_str)
                        .unwrap_or("Ranked")
                        .replace('_', " ");
                    let tier = entry
                        .get("tier")
                        .and_then(Value::as_str)
                        .unwrap_or("UNRANKED");
                    let division = entry
                        .get("rank")
                        .and_then(Value::as_str)
                        .unwrap_or_default();
                    let wins = entry.get("wins").and_then(Value::as_u64).unwrap_or(0);
                    let losses = entry.get("losses").and_then(Value::as_u64).unwrap_or(0);
                    let rate = if wins + losses == 0 {
                        0.0
                    } else {
                        wins as f64 * 100.0 / (wins + losses) as f64
                    };
                    ui.label(RichText::new(format!("{queue}  ·  {tier} {division}  ·  {} LP  ·  {wins}W / {losses}L  ·  {rate:.1}%", entry.get("leaguePoints").and_then(Value::as_u64).unwrap_or(0))).size(10.0).color(GOLD));
                }
            }
            ui.add_space(6.0);
            ui.label(
                RichText::new("TOP CHAMPION MASTERY")
                    .size(9.0)
                    .strong()
                    .color(MUTED),
            );
            if let Some(masteries) = lookup.get("masteries").and_then(Value::as_array) {
                ui.horizontal_wrapped(|ui| {
                    for mastery in masteries.iter().take(5) {
                        let id = mastery
                            .get("championId")
                            .and_then(Value::as_u64)
                            .unwrap_or(0) as u32;
                        let name = self
                            .champions
                            .iter()
                            .find(|champion| champion.id == id)
                            .map(|champion| champion.name.as_str())
                            .unwrap_or("Champion");
                        ui.label(
                            RichText::new(format!(
                                "{name}  ·  M{}  ·  {} pts",
                                mastery
                                    .get("championLevel")
                                    .and_then(Value::as_u64)
                                    .unwrap_or(0),
                                mastery
                                    .get("championPoints")
                                    .and_then(Value::as_u64)
                                    .unwrap_or(0)
                            ))
                            .size(9.5)
                            .color(CYAN),
                        );
                    }
                });
            }
            ui.add_space(8.0);
            ui.label(
                RichText::new("RECENT MATCHES")
                    .size(9.0)
                    .strong()
                    .color(MUTED),
            );
            if let Some(matches) = lookup.get("matches").and_then(Value::as_array) {
                let puuid = account
                    .get("puuid")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                for game in matches {
                    let Some(summary) = normalize_riot_match(game, puuid) else {
                        continue;
                    };
                    ui.horizontal(|ui| {
                        let win = summary.get("win").and_then(Value::as_bool).unwrap_or(false);
                        ui.label(
                            RichText::new(if win { "WIN" } else { "LOSS" })
                                .size(9.0)
                                .strong()
                                .color(if win { GREEN } else { RED }),
                        );
                        ui.label(
                            RichText::new(
                                summary
                                    .get("championName")
                                    .and_then(Value::as_str)
                                    .unwrap_or("Champion"),
                            )
                            .size(10.0)
                            .strong()
                            .color(TEXT),
                        );
                        ui.label(
                            RichText::new(format!(
                                "{} / {} / {}",
                                summary.get("kills").and_then(Value::as_u64).unwrap_or(0),
                                summary.get("deaths").and_then(Value::as_u64).unwrap_or(0),
                                summary.get("assists").and_then(Value::as_u64).unwrap_or(0)
                            ))
                            .size(9.5)
                            .color(SECONDARY),
                        );
                        ui.label(
                            RichText::new(format!(
                                "{} CS  ·  {:.1}/m",
                                summary.get("cs").and_then(Value::as_u64).unwrap_or(0),
                                summary
                                    .get("csPerMin")
                                    .and_then(Value::as_f64)
                                    .unwrap_or(0.0)
                            ))
                            .size(9.0)
                            .color(GOLD),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if action_button(ui, "ANALYZE", false, CYAN).clicked() {
                                self.selected_analysis_match = Some(summary.clone());
                            }
                        });
                    });
                }
            }
        });
    }

    fn render_match_analysis(&mut self, ui: &mut egui::Ui) {
        let Some(summary) = self.selected_analysis_match.clone() else {
            return;
        };
        panel(ui, |ui| {
            ui.horizontal(|ui| {
                panel_title(
                    ui,
                    "POST-GAME ANALYSIS",
                    summary
                        .get("gameMode")
                        .and_then(Value::as_str)
                        .unwrap_or("Recent match"),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if action_button(ui, "CLOSE", false, SECONDARY).clicked() {
                        self.selected_analysis_match = None;
                    }
                });
            });
            let duration = summary.get("duration").and_then(Value::as_u64).unwrap_or(0);
            let kills = summary.get("kills").and_then(Value::as_u64).unwrap_or(0);
            let deaths = summary.get("deaths").and_then(Value::as_u64).unwrap_or(0);
            let assists = summary.get("assists").and_then(Value::as_u64).unwrap_or(0);
            ui.horizontal_wrapped(|ui| {
                metric(
                    ui,
                    "RESULT",
                    if summary.get("win").and_then(Value::as_bool).unwrap_or(false) {
                        "VICTORY"
                    } else {
                        "DEFEAT"
                    },
                    if summary.get("win").and_then(Value::as_bool).unwrap_or(false) {
                        GREEN
                    } else {
                        RED
                    },
                );
                metric(
                    ui,
                    "CHAMPION",
                    summary
                        .get("championName")
                        .and_then(Value::as_str)
                        .unwrap_or("—"),
                    CYAN,
                );
                metric(
                    ui,
                    "K / D / A",
                    &format!("{kills} / {deaths} / {assists}"),
                    TEXT,
                );
                metric(
                    ui,
                    "CS / MIN",
                    &format!(
                        "{:.1}",
                        summary
                            .get("csPerMin")
                            .and_then(Value::as_f64)
                            .unwrap_or(0.0)
                    ),
                    GOLD,
                );
                metric(
                    ui,
                    "GOLD",
                    &format!(
                        "{}",
                        summary.get("gold").and_then(Value::as_u64).unwrap_or(0)
                    ),
                    GREEN,
                );
                metric(
                    ui,
                    "DAMAGE",
                    &format!(
                        "{}",
                        summary.get("damage").and_then(Value::as_u64).unwrap_or(0)
                    ),
                    RED,
                );
                metric(
                    ui,
                    "VISION",
                    &format!(
                        "{}",
                        summary.get("vision").and_then(Value::as_u64).unwrap_or(0)
                    ),
                    CYAN,
                );
                metric(
                    ui,
                    "DURATION",
                    &format!("{}:{:02}", duration / 60, duration % 60),
                    SECONDARY,
                );
            });
            if let Some(items) = summary.get("items").and_then(Value::as_array) {
                ui.add_space(5.0);
                let item_names = items
                    .iter()
                    .filter_map(Value::as_u64)
                    .filter(|id| *id > 0)
                    .map(|id| crate::builds::item_name(id as u32))
                    .collect::<Vec<_>>()
                    .join("  ·  ");
                ui.label(
                    RichText::new(format!("ITEMS  {item_names}"))
                        .size(9.5)
                        .color(SECONDARY),
                );
            }
        });
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) {
        self.render_page_heading(
            ui,
            "RIFTCORE CONTROL",
            "Preferences",
            "Choose a category to manage gameplay, the HUD, Riot API, and app behavior",
        );

        ui.horizontal_wrapped(|ui| {
            for (section, label, accent) in [
                (SettingsSection::Gameplay, "🎮  GAMEPLAY", CYAN),
                (SettingsSection::Overlay, "🛡️  OVERLAY", GOLD),
                (SettingsSection::RiotApi, "🔑  RIOT API", GREEN),
                (SettingsSection::App, "⚙️  APP", SECONDARY),
            ] {
                if action_button(ui, label, self.settings_section == section, accent).clicked() {
                    self.settings_section = section;
                }
            }
        });
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(10.0);

        match self.settings_section {
            SettingsSection::Gameplay => {
                ui.columns(2, |columns| {
                    self.render_automation_settings(&mut columns[0]);
                    self.render_league_client_settings(&mut columns[1]);
                });
            }
            SettingsSection::Overlay => {
                self.render_desktop_overlay_settings(ui);
                self.render_hud_settings(ui);
            }
            SettingsSection::RiotApi => self.render_riot_api_settings(ui),
            SettingsSection::App => {
                ui.columns(2, |columns| {
                    self.render_app_experience_settings(&mut columns[0]);
                    self.render_update_settings(&mut columns[1]);
                });
            }
        }

        ui.add_space(12.0);
        panel(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("PREFERENCES").size(11.0).strong().color(TEXT));
                    ui.label(
                        RichText::new("Saved locally. Save after changing your Riot API key.")
                            .size(10.5)
                            .color(MUTED),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if action_button(ui, "SAVE PREFERENCES", false, CYAN).clicked() {
                        let _ = self
                            .commands
                            .send(CoreCommand::SetAutoAccept(self.settings.auto_accept));
                        self.save_settings();
                        self.set_notice("Preferences saved");
                    }
                });
            });
        });
    }

    fn render_automation_settings(&mut self, ui: &mut egui::Ui) {
        panel(ui, |ui| {
            panel_title(ui, "GAMEPLAY AUTOMATION", "League client actions");
            let mut changed = false;
            changed |= settings_row(
                ui,
                "Auto accept queue",
                &mut self.settings.auto_accept,
                "Accept the ready check automatically",
            );
            changed |= settings_row(
                ui,
                "Auto import runes",
                &mut self.settings.auto_import_runes,
                "Apply the selected rune page in champion select",
            );
            changed |= settings_row(
                ui,
                "Auto import summoner spells",
                &mut self.settings.auto_import_spells,
                "Apply the selected spells in champion select",
            );
            changed |= settings_row(
                ui,
                "Auto export item sets",
                &mut self.settings.auto_import_item_sets,
                "Add the selected build to the League shop",
            );
            changed |= settings_row(
                ui,
                "Audio alerts",
                &mut self.settings.sound_alerts,
                "Play a cue when a ready check appears",
            );
            if changed {
                let _ = self
                    .commands
                    .send(CoreCommand::SetAutoAccept(self.settings.auto_accept));
                self.save_settings();
            }
        });
    }

    fn render_league_client_settings(&mut self, ui: &mut egui::Ui) {
        panel(ui, |ui| {
            panel_title(ui, "LEAGUE CLIENT", "Launch and display controls");
            ui.horizontal_wrapped(|ui| {
                if action_button(ui, "LAUNCH LEAGUE", false, CYAN).clicked() {
                    let _ = self.commands.send(CoreCommand::LaunchLeague);
                }
                if action_button(ui, "BORDERLESS WINDOW", false, CYAN).clicked() {
                    let _ = self.commands.send(CoreCommand::SetLeagueWindowMode(2));
                }
                if action_button(ui, "FULLSCREEN", false, SECONDARY).clicked() {
                    let _ = self.commands.send(CoreCommand::SetLeagueWindowMode(0));
                }
            });
            ui.add_space(6.0);
            ui.label(
                RichText::new(
                    "Display changes back up game.cfg before writing. Restart League to apply them.",
                )
                .size(10.0)
                .color(MUTED),
            );
        });
    }

    fn render_desktop_overlay_settings(&mut self, ui: &mut egui::Ui) {
        panel(ui, |ui| {
            panel_title(ui, "WINDOW & OVERLAY", "Desktop behavior and HUD input");
            if settings_row(
                ui,
                "Always on top",
                &mut self.settings.always_on_top,
                "Keep the companion above other windows",
            ) {
                self.pinned = self.settings.always_on_top;
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                        if self.settings.always_on_top {
                            egui::viewport::WindowLevel::AlwaysOnTop
                        } else {
                            egui::viewport::WindowLevel::Normal
                        },
                    ));
                self.save_settings();
            }
            if settings_row(
                ui,
                "Force click-through",
                &mut self.overlay_mouse_passthrough,
                "Pass every click to League; HUD controls become unavailable",
            ) && self.overlay_viewport_open
            {
                ui.ctx().send_viewport_cmd_to(
                    overlay_viewport_id(),
                    egui::ViewportCommand::MousePassthrough(self.overlay_mouse_passthrough),
                );
            }
            ui.label(
                RichText::new("Alt + ~ toggles the HUD. Alt + Shift + ~ toggles click-through.")
                    .size(10.0)
                    .color(MUTED),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Flash key").size(11.5).color(SECONDARY));
                let changed = egui::ComboBox::from_id_salt("flash-slot")
                    .selected_text(if self.settings.flash_slot_preference == "D" {
                        "D"
                    } else {
                        "F"
                    })
                    .show_ui(ui, |ui| {
                        let mut changed = false;
                        changed |= ui
                            .selectable_value(
                                &mut self.settings.flash_slot_preference,
                                "F".to_string(),
                                "F",
                            )
                            .changed();
                        changed |= ui
                            .selectable_value(
                                &mut self.settings.flash_slot_preference,
                                "D".to_string(),
                                "D",
                            )
                            .changed();
                        changed
                    })
                    .inner
                    .unwrap_or(false);
                if changed {
                    self.save_settings();
                }
            });
        });
    }

    fn render_riot_api_settings(&mut self, ui: &mut egui::Ui) {
        panel(ui, |ui| {
            panel_title(ui, "RIOT API", "Player scouting and match history");
            ui.label(
                RichText::new("PLATFORM REGION")
                    .size(10.0)
                    .strong()
                    .color(MUTED),
            );
            let region_changed = egui::ComboBox::from_id_salt("riot-region")
                .selected_text(self.settings.riot_region.to_ascii_uppercase())
                .width(220.0)
                .show_ui(ui, |ui| {
                    let mut changed = false;
                    for region in [
                        "eun1", "euw1", "na1", "kr", "br1", "la1", "la2", "oc1", "tr1", "ru", "jp1",
                    ] {
                        changed |= ui
                            .selectable_value(
                                &mut self.settings.riot_region,
                                region.to_string(),
                                region.to_ascii_uppercase(),
                            )
                            .changed();
                    }
                    changed
                })
                .inner
                .unwrap_or(false);
            if region_changed {
                self.save_settings();
            }

            ui.add_space(8.0);
            ui.label(RichText::new("API KEY").size(10.0).strong().color(MUTED));
            ui.add_sized(
                [ui.available_width(), 34.0],
                egui::TextEdit::singleline(&mut self.settings.riot_api_key)
                    .password(true)
                    .hint_text("RGAPI-…"),
            );
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if action_button(ui, "TEST API KEY", false, CYAN).clicked() {
                    let _ = self.commands.send(CoreCommand::ValidateRiotApiKey {
                        region: self.settings.riot_region.clone(),
                        api_key: self.settings.riot_api_key.clone(),
                    });
                }
                if let Some(status) = self.snapshot.riot_api_status.as_deref() {
                    ui.label(RichText::new(status).size(10.0).color(
                        if status.contains("verified") {
                            GREEN
                        } else {
                            GOLD
                        },
                    ));
                }
            });
            ui.add_space(6.0);
            ui.label(
                RichText::new(
                    "The key is stored in your local RiftCore settings and sent only to Riot API endpoints.",
                )
                .size(10.0)
                .color(MUTED),
            );
        });
    }

    fn render_update_settings(&mut self, ui: &mut egui::Ui) {
        panel(ui, |ui| {
            panel_title(
                ui,
                "APP UPDATES",
                &format!("Installed version {}", env!("CARGO_PKG_VERSION")),
            );
            ui.horizontal_wrapped(|ui| {
                if action_button(ui, "CHECK FOR UPDATES", false, CYAN).clicked() {
                    let _ = self.commands.send(CoreCommand::CheckUpdates);
                }
                if let Some(status) = self.snapshot.update_status.as_deref() {
                    let color = if status.contains("available") {
                        GOLD
                    } else if status.starts_with("Could not") || status.contains("error") {
                        RED
                    } else {
                        GREEN
                    };
                    ui.label(RichText::new(status).size(10.0).color(color));
                }
            });
            if let Some(url) = self
                .snapshot
                .latest_release
                .as_ref()
                .and_then(|release| release.get("html_url"))
                .and_then(Value::as_str)
            {
                if action_button(ui, "VIEW RELEASE PAGE", false, SECONDARY).clicked() {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                }
            }

            let update_available = self
                .snapshot
                .update_status
                .as_deref()
                .is_some_and(|status| status.contains("is available"));
            if update_available {
                let release = self.snapshot.latest_release.as_ref();
                let has_auto_update = release
                    .and_then(|release| release.get("assets"))
                    .and_then(Value::as_array)
                    .is_some_and(|assets| {
                        assets.iter().any(|asset| {
                            asset.get("name").and_then(Value::as_str) == Some("RiftCore.exe")
                        })
                    });
                let release_url = release
                    .and_then(|release| release.get("html_url"))
                    .and_then(Value::as_str);
                let update_busy = self
                    .snapshot
                    .update_status
                    .as_deref()
                    .is_some_and(|status| status.starts_with("Downloading update"));
                if action_button(
                    ui,
                    if update_busy {
                        "DOWNLOADING UPDATE…"
                    } else if has_auto_update {
                        "UPDATE & RESTART"
                    } else {
                        "DOWNLOAD SETUP"
                    },
                    update_busy,
                    CYAN,
                )
                .clicked()
                    && !update_busy
                {
                    if has_auto_update {
                        let _ = self.commands.send(CoreCommand::InstallUpdate);
                    } else if let Some(url) = release_url {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                    }
                }
            }
        });
    }

    fn render_app_experience_settings(&mut self, ui: &mut egui::Ui) {
        panel(ui, |ui| {
            panel_title(ui, "APPEARANCE & BEHAVIOR", "Tune the app itself");
            let theme_context = ui.ctx().clone();
            let mut appearance_changed = false;

            ui.label(
                RichText::new("INTERFACE THEME")
                    .size(10.0)
                    .strong()
                    .color(MUTED),
            );
            appearance_changed |= egui::ComboBox::from_id_salt("native-theme")
                .selected_text(match self.settings.theme.as_str() {
                    "ember" => "Ember",
                    "void" => "Void",
                    _ => "Hextech",
                })
                .width(220.0)
                .show_ui(ui, |ui| {
                    let mut changed = false;
                    changed |= ui
                        .selectable_value(
                            &mut self.settings.theme,
                            "hextech".to_string(),
                            "Hextech",
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(&mut self.settings.theme, "ember".to_string(), "Ember")
                        .changed();
                    changed |= ui
                        .selectable_value(&mut self.settings.theme, "void".to_string(), "Void")
                        .changed();
                    changed
                })
                .inner
                .unwrap_or(false);

            ui.add_space(8.0);
            ui.label(RichText::new("TYPOGRAPHY").size(10.0).strong().color(MUTED));
            appearance_changed |= egui::ComboBox::from_id_salt("native-font")
                .selected_text(match self.settings.font_family.as_str() {
                    "outfit" => "Outfit",
                    "inter" => "Inter",
                    _ => "Plus Jakarta Sans",
                })
                .width(220.0)
                .show_ui(ui, |ui| {
                    let mut changed = false;
                    changed |= ui
                        .selectable_value(
                            &mut self.settings.font_family,
                            "plus-jakarta".to_string(),
                            "Plus Jakarta Sans",
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            &mut self.settings.font_family,
                            "outfit".to_string(),
                            "Outfit",
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            &mut self.settings.font_family,
                            "inter".to_string(),
                            "Inter",
                        )
                        .changed();
                    changed
                })
                .inner
                .unwrap_or(false);

            if appearance_changed {
                apply_native_theme(&theme_context, &self.settings.theme);
                install_font_family(&theme_context, &self.settings.font_family);
                self.save_settings();
            }

            ui.add_space(6.0);
            ui.label(
                RichText::new(
                    "Fonts are bundled locally, so the app keeps its appearance offline.",
                )
                .size(10.0)
                .color(MUTED),
            );
            ui.add_space(8.0);
            if settings_row(
                ui,
                "Close to system tray",
                &mut self.settings.close_to_tray,
                "Keep RiftCore running in the tray when you close its window",
            ) {
                self.save_settings();
            }
            ui.horizontal_wrapped(|ui| {
                if action_button(ui, "QUICK TOUR", false, GOLD).clicked() {
                    self.show_quick_tour = true;
                }
                if action_button(ui, "WHAT'S NEW", false, CYAN).clicked() {
                    self.show_whats_new = true;
                }
            });
        });
    }
    fn render_dialogs(&mut self, ctx: &egui::Context) {
        if self.show_whats_new || self.show_quick_tour {
            let viewport = ctx.content_rect();
            let content = egui::Rect::from_min_max(
                egui::pos2(
                    viewport.left(),
                    (viewport.top() + 38.0).min(viewport.bottom()),
                ),
                viewport.max,
            );
            ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::unique("riftcore-update-backdrop"),
            ))
            .rect_filled(content, 0.0, Color32::from_black_alpha(95));
        }
        if self.show_whats_new {
            let mut dismissed = false;
            egui::Window::new("What's new")
                .id(egui::Id::unique("riftcore-whats-new"))
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .movable(false)
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .frame(Frame::new().fill(SURFACE).stroke(Stroke::new(1.0, BORDER)).corner_radius(egui::CornerRadius::same(16)).inner_margin(egui::Margin::same(22)))
                .show(ctx, |ui| {
                    ui.set_min_width(520.0);
                    ui.label(RichText::new(format!("RIFTCORE {}", env!("CARGO_PKG_VERSION"))).size(10.0).strong().color(CYAN));
                    ui.heading(RichText::new("What's new").size(25.0).strong().color(TEXT));
                    ui.label(RichText::new("The native app is catching up with the original companion features.").size(12.0).color(SECONDARY));
                    ui.add_space(12.0);
                    for (title, detail) in [
                        ("Your look", "Choose Hextech, Ember, or Void and switch between bundled Plus Jakarta Sans, Outfit, and Inter fonts."),
                        ("HUD layouts", "Use the integrated overlay, compact top bar, tactical card, or minimap pill. Drag modules in edit mode."),
                        ("Minimap guide", "Show public objective countdowns, your numbered opening route, and optional static ward suggestions on the minimap."),
                        ("Ally timers", "Manually mark allied ultimate casts and reset their countdowns from the live HUD."),
                        ("In-game builds", "Open your active champion's core build and sync the item set to the League shop."),
                        ("Modes", "Browse the restored ARAM Mayhem and Arena card catalog by mode and tier."),
                        ("Desktop controls", "Switch League between Fullscreen and Borderless (the HUD works in both) and keep RiftCore in the system tray."),
                    ] {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("◆").size(12.0).color(CYAN));
                            ui.vertical(|ui| {
                                ui.label(RichText::new(title).size(12.0).strong().color(TEXT));
                                ui.label(RichText::new(detail).size(12.0).color(SECONDARY));
                            });
                        });
                        ui.add_space(8.0);
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if action_button(ui, "CONTINUE", false, CYAN).clicked() {
                            dismissed = true;
                        }
                    });
                });
            if dismissed {
                self.show_whats_new = false;
                if !self.settings.first_run_tour_seen {
                    self.show_quick_tour = true;
                }
            }
        }

        if self.show_quick_tour {
            let mut close = false;
            let mut open_champ_select = false;
            egui::Window::new("Quick tour")
                .id(egui::Id::unique("riftcore-quick-tour"))
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .movable(false)
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .frame(Frame::new().fill(SURFACE).stroke(Stroke::new(1.0, BORDER)).corner_radius(egui::CornerRadius::same(16)).inner_margin(egui::Margin::same(22)))
                .show(ctx, |ui| {
                    ui.set_min_width(570.0);
                    ui.set_max_width(680.0);
                    ui.label(RichText::new("QUICK TOUR · 2 MINUTES").size(10.0).strong().color(GOLD));
                    ui.heading(RichText::new("Your companion, at a glance").size(24.0).strong().color(TEXT));
                    ui.add_space(10.0);
                    for (step, title, detail) in [
                        ("01", "Prepare your pick", "Champ Select builds, runes, summoner spells, shop item sets, and client automation."),
                        ("02", "Plan the draft", "Draft Coach, Lobby Scout, champion pool, and tier list help with role and matchup prep."),
                        ("03", "Use the live HUD", "Switch between four overlay layouts, configure your objective dock, and start manual ARAM relic timers."),
                        ("04", "Explore the rest", "Game modes and Mayhem cards, TFT board and comps, profile lookup, match history, and post-game analysis."),
                    ] {
                        ui.horizontal(|ui| {
                            badge(ui, step, CYAN);
                            ui.vertical(|ui| {
                                ui.label(RichText::new(title).size(12.0).strong().color(TEXT));
                                ui.label(RichText::new(detail).size(12.0).color(SECONDARY));
                            });
                        });
                        ui.add_space(9.0);
                    }
                    Frame::new()
                        .fill(CARD)
                        .stroke(Stroke::new(1.0, BORDER))
                        .corner_radius(egui::CornerRadius::same(12))
                        .inner_margin(egui::Margin::same(12))
                        .show(ui, |ui| {
                            ui.label(RichText::new("SHORTCUTS").size(12.0).strong().color(CYAN));
                            ui.label(RichText::new("Works while RiftCore is running").size(11.5).color(SECONDARY));
                            ui.add_space(4.0);
                            ui.label(RichText::new("Alt + ` or Alt + Shift + Space  Toggle the HUD    ·    Alt + Shift + `  Toggle click-through").size(11.5).monospace().color(CYAN));
                        });
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if action_button(ui, "OPEN CHAMP SELECT", false, CYAN).clicked() {
                            open_champ_select = true;
                            close = true;
                        }
                        if action_button(ui, "DONE", false, SECONDARY).clicked() {
                            close = true;
                        }
                    });
                });
            if close {
                self.show_quick_tour = false;
                self.settings.first_run_tour_seen = true;
                self.save_settings();
                if open_champ_select {
                    self.active_page = Page::ChampSelect;
                }
            }
        }

        if self.show_mayhem_cards {
            let cards = MAYHEM_CARDS.as_array().cloned().unwrap_or_default();
            let mut close = false;
            egui::Modal::new(egui::Id::unique("riftcore-mayhem-guide"))
                .frame(Frame::new().fill(SURFACE).stroke(Stroke::new(1.0, BORDER)).corner_radius(egui::CornerRadius::same(16)).inner_margin(egui::Margin::same(18)))
                .show(ctx, |ui| {
                    ui.set_min_width(790.0);
                    ui.set_max_width(940.0);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("MODE REFERENCE").size(10.0).strong().color(GOLD));
                            ui.heading(RichText::new("Mayhem & Arena cards").size(22.0).strong().color(TEXT));
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if action_button(ui, "CLOSE", false, SECONDARY).clicked() {
                                close = true;
                            }
                        });
                    });
                    ui.add_space(8.0);
                    ui.horizontal_wrapped(|ui| {
                        for mode in ["Both", "ARAM", "Arena"] {
                            if action_button(ui, &mode.to_ascii_uppercase(), self.mayhem_mode == mode, GOLD).clicked() {
                                self.mayhem_mode = mode.to_string();
                            }
                        }
                        ui.add_space(8.0);
                        for tier in ["ALL", "Prismatic", "Gold", "Silver"] {
                            if action_button(ui, &tier.to_ascii_uppercase(), self.mayhem_tier == tier, CYAN).clicked() {
                                self.mayhem_tier = tier.to_string();
                            }
                        }
                    });
                    ui.add_space(6.0);
                    ui.add(egui::TextEdit::singleline(&mut self.mayhem_search).hint_text("Search a card, champion, role, or effect…").desired_width(ui.available_width()));
                    ui.add_space(8.0);
                    let query = self.mayhem_search.trim().to_lowercase();
                    let filtered = cards.iter().filter(|card| {
                        let mode = card.get("mode").and_then(Value::as_str).unwrap_or("Both");
                        let tier = card.get("tier").and_then(Value::as_str).unwrap_or("");
                        let matches_mode = self.mayhem_mode == "Both" || mode == "Both" || mode == self.mayhem_mode;
                        let matches_tier = self.mayhem_tier == "ALL" || tier == self.mayhem_tier;
                        let searchable = ["name", "archetype", "description", "tip", "tag"]
                            .iter()
                            .filter_map(|key| card.get(*key).and_then(Value::as_str))
                            .collect::<Vec<_>>().join(" ").to_lowercase();
                        matches_mode && matches_tier && (query.is_empty() || searchable.contains(&query))
                    }).collect::<Vec<_>>();
                    ui.label(RichText::new(format!("{} cards", filtered.len())).size(10.0).color(MUTED));
                    ScrollArea::vertical().id_salt("mayhem-card-scroll").max_height(510.0).show(ui, |ui| {
                        let no_results = filtered.is_empty();
                        for card in &filtered {
                            let name = card.get("name").and_then(Value::as_str).unwrap_or("Card");
                            let tier = card.get("tier").and_then(Value::as_str).unwrap_or("—");
                            let tag = card.get("tag").and_then(Value::as_str).unwrap_or("CARD");
                            let archetype = card.get("archetype").and_then(Value::as_str).unwrap_or("");
                            let description = card.get("description").and_then(Value::as_str).unwrap_or("");
                            let tip = card.get("tip").and_then(Value::as_str).unwrap_or("");
                            let tint = match tier { "Prismatic" => Color32::from_rgb(192, 132, 252), "Gold" => GOLD, _ => SECONDARY };
                            Frame::new().fill(CARD).stroke(Stroke::new(1.0, BORDER)).corner_radius(egui::CornerRadius::same(10)).inner_margin(egui::Margin::same(10)).show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(name).size(12.0).strong().color(TEXT));
                                            badge(ui, tier, tint);
                                            badge(ui, tag, CYAN);
                                        });
                                        ui.label(RichText::new(archetype).size(9.5).color(SECONDARY));
                                        ui.label(RichText::new(description).size(10.0).color(TEXT));
                                        ui.label(RichText::new(tip).size(9.5).color(GOLD));
                                    });
                                });
                            });
                            ui.add_space(6.0);
                        }
                        if no_results {
                            ui.label(RichText::new("No cards match those filters.").size(11.0).color(SECONDARY));
                        }
                    });
                    ui.label(RichText::new("Curated card descriptions can change between patches; check the in-game card text before choosing.").size(9.0).color(MUTED));
                });
            if close {
                self.show_mayhem_cards = false;
            }
        }

        if self.show_close_confirmation {
            let mut cancel = false;
            let mut hide = false;
            let mut exit = false;
            egui::Modal::new(egui::Id::unique("riftcore-close-confirmation"))
                .frame(Frame::new().fill(SURFACE).stroke(Stroke::new(1.0, BORDER)).corner_radius(egui::CornerRadius::same(16)).inner_margin(egui::Margin::same(20)))
                .show(ctx, |ui| {
                    ui.set_min_width(420.0);
                    ui.heading(RichText::new("Keep RiftCore nearby?").size(21.0).strong().color(TEXT));
                    ui.label(RichText::new("Minimizing to the tray keeps League monitoring and the HUD available in the background.").size(11.0).color(SECONDARY));
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if action_button(ui, if self.tray_icon_available() { "MINIMIZE TO TRAY" } else { "MINIMIZE TO TASKBAR" }, false, CYAN).clicked() {
                            hide = true;
                        }
                        if action_button(ui, "EXIT", false, RED).clicked() {
                            exit = true;
                        }
                        if action_button(ui, "CANCEL", false, SECONDARY).clicked() {
                            cancel = true;
                        }
                    });
                });
            if hide {
                self.show_close_confirmation = false;
                self.settings.close_confirmation_seen = true;
                self.settings.close_to_tray = self.tray_icon_available();
                self.save_settings();
                if self.tray_icon_available() {
                    self.hidden_in_tray = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }
            } else if exit {
                self.show_close_confirmation = false;
                self.settings.close_confirmation_seen = true;
                self.settings.close_to_tray = false;
                self.save_settings();
                self.force_exit = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else if cancel {
                self.show_close_confirmation = false;
            }
        }
    }

    /// Drives the new in-game HUD and applies what its panels asked for.
    fn tick_path_b(&mut self, ctx: &egui::Context) {
        let Some(path_b) = self.path_b.as_mut() else {
            return;
        };
        let output = path_b.tick(crate::path_b::PathBFrame {
            enabled: self.overlay_mode,
            automatic: self.overlay_auto_entered,
            settings: &self.settings,
            force_passthrough: self.overlay_mouse_passthrough,
            show_mayhem_cards: self.show_mayhem_cards,
        });
        let status = path_b.status();
        self.show_mayhem_cards = output.show_mayhem_cards;
        if status != self.path_b_status {
            self.path_b_status = status;
            // "Waiting for a game" is already announced when the HUD is switched on.
            if let Some(message) = status.message().filter(|_| {
                self.overlay_mode && status != crate::path_b::PathBStatus::WaitingForGame
            }) {
                self.set_notice(message);
            }
        }
        if let Some(next) = output.next_repaint {
            ctx.request_repaint_after(next);
        }
        let actions = output.actions;
        let mut save = false;
        for (key, pos) in actions.updated_positions {
            self.settings.hud_positions.insert(key, pos);
            save = true;
        }
        if save {
            self.save_settings();
        }
        if let Some(set) = actions.export_item_set {
            let _ = self.commands.send(CoreCommand::ExportItemSet(set));
        }
        if let Some(text) = actions.copy_clipboard {
            ctx.copy_text(text);
        }
        if let Some(notice) = actions.notice {
            self.set_notice(notice);
        }
        if actions.request_close_overlay {
            self.set_overlay_mode(ctx, false, false);
        }
    }

    fn show_from_tray(&mut self, ctx: &egui::Context) {
        self.hidden_in_tray = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    fn tray_icon_available(&self) -> bool {
        #[cfg(windows)]
        {
            self.tray_icon.is_some()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    fn render_mayhem_overlay(&mut self, ctx: &egui::Context, area: egui::Rect) {
        let cards = MAYHEM_CARDS.as_array().cloned().unwrap_or_default();
        let position = egui::pos2(
            (area.right() - 412.0).max(area.left() + 8.0),
            area.top() + 8.0,
        );
        let response = egui::Area::new(egui::Id::unique("riftcore-overlay-mayhem-cards"))
            .order(egui::Order::Foreground)
            .fixed_pos(position)
            .show(ctx, |ui| {
                Frame::new()
                    .fill(Color32::from_rgba_unmultiplied(9, 14, 25, 248))
                    .stroke(Stroke::new(1.0, GOLD))
                    .corner_radius(egui::CornerRadius::same(14))
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_width(390.0);
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new("MAYHEM & ARENA")
                                        .size(9.0)
                                        .strong()
                                        .color(GOLD),
                                );
                                ui.label(
                                    RichText::new("Card guide").size(16.0).strong().color(TEXT),
                                );
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if action_button(ui, "CLOSE", false, SECONDARY).clicked() {
                                    self.show_mayhem_cards = false;
                                }
                            });
                        });
                        ui.horizontal_wrapped(|ui| {
                            for mode in ["Both", "ARAM", "Arena"] {
                                if action_button(
                                    ui,
                                    &mode.to_ascii_uppercase(),
                                    self.mayhem_mode == mode,
                                    GOLD,
                                )
                                .clicked()
                                {
                                    self.mayhem_mode = mode.to_string();
                                }
                            }
                        });
                        ui.add(
                            egui::TextEdit::singleline(&mut self.mayhem_search)
                                .hint_text("Search cards or champion fit…")
                                .desired_width(ui.available_width()),
                        );
                        let query = self.mayhem_search.trim().to_lowercase();
                        let filtered = cards
                            .iter()
                            .filter(|card| {
                                let mode =
                                    card.get("mode").and_then(Value::as_str).unwrap_or("Both");
                                let tier = card.get("tier").and_then(Value::as_str).unwrap_or("");
                                let matches_mode = self.mayhem_mode == "Both"
                                    || mode == "Both"
                                    || mode == self.mayhem_mode;
                                let matches_tier =
                                    self.mayhem_tier == "ALL" || tier == self.mayhem_tier;
                                let searchable = ["name", "archetype", "description", "tip", "tag"]
                                    .iter()
                                    .filter_map(|key| card.get(*key).and_then(Value::as_str))
                                    .collect::<Vec<_>>()
                                    .join(" ")
                                    .to_lowercase();
                                matches_mode
                                    && matches_tier
                                    && (query.is_empty() || searchable.contains(&query))
                            })
                            .collect::<Vec<_>>();
                        ui.horizontal_wrapped(|ui| {
                            for tier in ["ALL", "Prismatic", "Gold", "Silver"] {
                                if action_button(
                                    ui,
                                    &tier.to_ascii_uppercase(),
                                    self.mayhem_tier == tier,
                                    CYAN,
                                )
                                .clicked()
                                {
                                    self.mayhem_tier = tier.to_string();
                                }
                            }
                        });
                        ScrollArea::vertical()
                            .id_salt("overlay-mayhem-scroll")
                            .max_height(450.0)
                            .show(ui, |ui| {
                                if filtered.is_empty() {
                                    ui.label(
                                        RichText::new("No cards match those filters.")
                                            .size(10.0)
                                            .color(SECONDARY),
                                    );
                                }
                                for card in filtered {
                                    ui.add_space(4.0);
                                    Frame::new()
                                        .fill(CARD)
                                        .stroke(Stroke::new(1.0, BORDER))
                                        .corner_radius(egui::CornerRadius::same(8))
                                        .inner_margin(egui::Margin::same(8))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    RichText::new(
                                                        card.get("name")
                                                            .and_then(Value::as_str)
                                                            .unwrap_or("Card"),
                                                    )
                                                    .size(11.0)
                                                    .strong()
                                                    .color(TEXT),
                                                );
                                                ui.label(
                                                    RichText::new(
                                                        card.get("tier")
                                                            .and_then(Value::as_str)
                                                            .unwrap_or(""),
                                                    )
                                                    .size(9.0)
                                                    .color(GOLD),
                                                );
                                            });
                                            ui.label(
                                                RichText::new(
                                                    card.get("archetype")
                                                        .and_then(Value::as_str)
                                                        .unwrap_or(""),
                                                )
                                                .size(9.0)
                                                .color(SECONDARY),
                                            );
                                            ui.label(
                                                RichText::new(
                                                    card.get("tip")
                                                        .and_then(Value::as_str)
                                                        .unwrap_or(""),
                                                )
                                                .size(9.5)
                                                .color(TEXT),
                                            );
                                        });
                                }
                            });
                    });
            });
        self.overlay_interactive_regions
            .push(response.response.rect);
    }

    fn render_overlay_build(&mut self, ctx: &egui::Context, area: egui::Rect) {
        let live = self.snapshot.live_game.clone();
        let active = live
            .as_ref()
            .and_then(|data| data.get("activePlayer"))
            .unwrap_or(&Value::Null);
        let active_name = active
            .get("championName")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let Some(champion) = self
            .champions
            .iter()
            .find(|champion| {
                champion.name.eq_ignore_ascii_case(active_name)
                    || champion.key.eq_ignore_ascii_case(active_name)
            })
            .cloned()
        else {
            return;
        };
        let build = crate::builds::recommend(&champion.key, &champion.name, &champion.roles, None);
        let default_pos = egui::pos2(
            (area.right() - 318.0).max(area.left() + 8.0),
            area.top() + 260.0,
        );
        let position = self
            .settings
            .hud_positions
            .get("build")
            .map(|position| egui::pos2(position[0], position[1]))
            .unwrap_or(default_pos);
        let movable = self.settings.hud_edit_layout;
        let mut export = false;
        let mut close = false;
        let response = egui::Area::new(egui::Id::unique("riftcore-overlay-build"))
            .current_pos(position)
            .movable(movable)
            .show(ctx, |ui| {
                Frame::new()
                    .fill(Color32::from_rgba_unmultiplied(7, 12, 22, 246))
                    .stroke(Stroke::new(1.0, CYAN))
                    .corner_radius(egui::CornerRadius::same(12))
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_width(292.0);
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new("LIVE BUILD").size(9.0).strong().color(CYAN),
                                );
                                ui.label(
                                    RichText::new(format!("{} · {}", champion.name, build.role))
                                        .size(15.0)
                                        .strong()
                                        .color(TEXT),
                                );
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                close = action_button(ui, "×", false, RED).clicked();
                            });
                        });
                        let current_gold = num(active, "currentGold");
                        let target_item = build
                            .core_items
                            .get(1)
                            .or_else(|| build.core_items.first())
                            .copied();
                        if let Some(item) = target_item {
                            ui.label(
                                RichText::new(format!(
                                    "NEXT CORE TARGET  ·  {}  ·  {current_gold}g held",
                                    crate::builds::item_name(item)
                                ))
                                .size(10.0)
                                .strong()
                                .color(GREEN),
                            );
                        }
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(format!(
                                "STARTING  {}",
                                build
                                    .starting_items
                                    .iter()
                                    .map(|id| crate::builds::item_name(*id))
                                    .collect::<Vec<_>>()
                                    .join("  ·  ")
                            ))
                            .size(9.5)
                            .color(SECONDARY),
                        );
                        ScrollArea::vertical()
                            .id_salt("overlay-live-build-items")
                            .max_height(168.0)
                            .show(ui, |ui| {
                                ui.add_space(6.0);
                                ui.label(
                                    RichText::new("CORE BUILD").size(9.0).strong().color(GOLD),
                                );
                                for (index, item) in build.core_items.iter().enumerate() {
                                    ui.label(
                                        RichText::new(format!(
                                            "{}. {}",
                                            index + 1,
                                            crate::builds::item_name(*item)
                                        ))
                                        .size(10.0)
                                        .color(TEXT),
                                    );
                                }
                                ui.add_space(5.0);
                                ui.label(
                                    RichText::new("SITUATIONAL").size(9.0).strong().color(GOLD),
                                );
                                for item in &build.situational_items {
                                    ui.label(
                                        RichText::new(crate::builds::item_name(*item))
                                            .size(9.5)
                                            .color(SECONDARY),
                                    );
                                }
                            });
                        ui.label(
                            RichText::new(format!("SKILL ORDER  {}", build.skill_priority))
                                .size(9.5)
                                .color(CYAN),
                        );
                        if action_button(ui, "SYNC BUILD TO LEAGUE SHOP", false, CYAN).clicked() {
                            export = true;
                        }
                        ui.label(RichText::new(&build.tip).size(9.0).color(MUTED));
                    });
            });
        self.overlay_interactive_regions
            .push(response.response.rect);
        if response.response.drag_stopped() {
            self.settings.hud_positions.insert(
                "build".to_string(),
                [response.response.rect.min.x, response.response.rect.min.y],
            );
            self.save_settings();
        }
        if close {
            self.show_overlay_build = false;
        }
        if export {
            let set = champion_item_set(&champion, &build);
            let _ = self.commands.send(CoreCommand::ExportItemSet(set));
            self.set_notice("Sending the recommended build to the League shop…");
        }
    }

    fn render_overlay_timers(
        &mut self,
        ctx: &egui::Context,
        area: egui::Rect,
        live: Option<&Value>,
    ) {
        let default_pos = egui::pos2(
            (area.left() + 300.0)
                .min(area.right() - 410.0)
                .max(area.left() + 8.0),
            area.top() + 112.0,
        );
        let position = self
            .settings
            .hud_positions
            .get("timers")
            .map(|position| egui::pos2(position[0], position[1]))
            .unwrap_or(default_pos);
        let mut copy_callout = false;
        let mut close = false;
        let movable = self.settings.hud_edit_layout;
        let response = egui::Area::new(egui::Id::unique("riftcore-overlay-timers"))
            .current_pos(position)
            .movable(movable)
            .show(ctx, |ui| {
                Frame::new()
                    .fill(Color32::from_rgba_unmultiplied(7, 12, 22, 246))
                    .stroke(Stroke::new(1.0, GOLD))
                    .corner_radius(egui::CornerRadius::same(12))
                    .inner_margin(egui::Margin::same(11))
                    .show(ui, |ui| {
                        ui.set_width(390.0);
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new("LIVE TIMERS").size(9.0).strong().color(GOLD),
                                );
                                ui.label(
                                    RichText::new("Manual tracking · click again to reset")
                                        .size(9.0)
                                        .color(MUTED),
                                );
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                close = action_button(ui, "×", false, RED).clicked();
                                if action_button(ui, "COPY CALL-OUT", false, CYAN).clicked() {
                                    copy_callout = true;
                                }
                            });
                        });
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            for tab in ["OBJECTIVES", "CAMPS"] {
                                let enabled = match tab {
                                    "CAMPS" => self.settings.hud_manual_camp_timers,
                                    _ => self.settings.hud_objective_timers,
                                };
                                if enabled
                                    && filter_chip(ui, tab, self.overlay_timer_tab == tab, GOLD)
                                        .clicked()
                                {
                                    self.overlay_timer_tab = tab.to_string();
                                }
                            }
                        });
                        ui.separator();
                        let max_height = (area.height() - 365.0).clamp(180.0, 420.0);
                        ScrollArea::vertical()
                            .id_salt("riftcore-overlay-timer-list")
                            .max_height(max_height)
                            .show(ui, |ui| match self.overlay_timer_tab.as_str() {
                                "CAMPS" if self.settings.hud_manual_camp_timers => {
                                    render_manual_camp_timers(ui, &mut self.manual_spell_timers);
                                }
                                _ if self.settings.hud_objective_timers => {
                                    render_objective_timer_cards(ui, live);
                                }
                                _ => {
                                    ui.label(
                                        RichText::new(
                                            "Enable a timer section in Settings → Overlay.",
                                        )
                                        .size(10.0)
                                        .color(SECONDARY),
                                    );
                                }
                            });
                    });
            });
        self.overlay_interactive_regions
            .push(response.response.rect);
        if response.response.drag_stopped() {
            self.settings.hud_positions.insert(
                "timers".to_string(),
                [response.response.rect.min.x, response.response.rect.min.y],
            );
            self.save_settings();
        }
        if close {
            self.show_overlay_timers = false;
        }
        if copy_callout {
            let mut callouts = Vec::new();
            if let Some(live) = live {
                for (label, timer) in objective_timers(live) {
                    callouts.push(format!("{} {}", label.to_lowercase(), timer));
                }
            }
            let message = if callouts.is_empty() {
                "[RiftCore] No active timers".to_string()
            } else {
                format!("[RiftCore] {}", callouts.join(" | "))
            };
            ctx.copy_text(message);
            self.set_notice("Timer call-out copied to clipboard");
        }
    }

    fn render_overlay(&mut self, ui: &mut egui::Ui) {
        self.overlay_interactive_regions.clear();
        let ctx = ui.ctx().clone();
        let live_snapshot = self.snapshot.live_game.clone();
        let frame_fill = Color32::TRANSPARENT;
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(frame_fill)
                    .inner_margin(egui::Margin::same(18)),
            )
            .show(ui, |ui| {
                let area = ui.available_rect_before_wrap();
                let live = live_snapshot.as_ref();
                let clock = live
                    .and_then(|data| data.get("gameData"))
                    .and_then(|data| data.get("gameTime"))
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0) as u64;
                let active = live
                    .and_then(|data| data.get("activePlayer"))
                    .unwrap_or(&Value::Null);
                // Scores and items live in the player's `allPlayers` entry.
                let me = live.and_then(active_player_entry).unwrap_or(&Value::Null);
                let scores = me.get("scores").unwrap_or(&Value::Null);
                let kda = format!(
                    "{} / {} / {}",
                    num(scores, "kills"),
                    num(scores, "deaths"),
                    num(scores, "assists")
                );
                let creep_score = scores
                    .get("creepScore")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0);
                let cs_per_minute = creep_score / (clock as f64 / 60.0).max(1.0);
                let game_mode = live
                    .and_then(|data| data.get("gameData"))
                    .and_then(|data| data.get("gameMode"))
                    .and_then(Value::as_str)
                    .unwrap_or("LIVE");

                match self.settings.hud_mode.as_str() {
                    "bar" => {
                        let bar_response = Frame::new()
                            .fill(Color32::from_rgba_unmultiplied(7, 12, 22, 238))
                            .stroke(Stroke::new(1.0, CYAN))
                            .corner_radius(egui::CornerRadius::same(12))
                            .inner_margin(egui::Margin::symmetric(12, 7))
                            .show(ui, |ui| {
                                ui.horizontal_centered(|ui| {
                                    ui.label(
                                        RichText::new("RIFTCORE").size(10.0).strong().color(CYAN),
                                    );
                                    ui.separator();
                                    ui.label(
                                        RichText::new(format!(
                                            "{:02}:{:02}",
                                            clock / 60,
                                            clock % 60
                                        ))
                                        .size(16.0)
                                        .monospace()
                                        .strong()
                                        .color(TEXT),
                                    );
                                    ui.label(RichText::new(kda).size(12.0).strong().color(TEXT));
                                    ui.label(
                                        RichText::new(format!("{} CS", num(scores, "creepScore")))
                                            .size(10.5)
                                            .color(GOLD),
                                    );
                                    if self.settings.hud_cs_benchmark {
                                        ui.label(
                                            RichText::new(format!("{cs_per_minute:.1}/m"))
                                                .size(10.0)
                                                .color(GOLD),
                                        );
                                    }
                                    if self.settings.hud_gold_lead {
                                        ui.label(
                                            RichText::new(format!(
                                                "{} G",
                                                num(active, "currentGold")
                                            ))
                                            .size(10.0)
                                            .color(GREEN),
                                        );
                                    }
                                    if self.settings.hud_mayhem_cards
                                        && action_button(ui, "CARDS", self.show_mayhem_cards, GOLD)
                                            .clicked()
                                    {
                                        self.show_mayhem_cards = !self.show_mayhem_cards;
                                    }
                                    if action_button(ui, "×", false, RED).clicked() {
                                        self.set_overlay_mode(&ctx, false, false);
                                    }
                                });
                            });
                        self.overlay_interactive_regions
                            .push(bar_response.response.rect);
                        if self.show_mayhem_cards {
                            self.render_mayhem_overlay(&ctx, area);
                        }
                        return;
                    }
                    "pill" => {
                        let pill_response = Frame::new()
                            .fill(Color32::from_rgba_unmultiplied(7, 12, 22, 238))
                            .stroke(Stroke::new(1.0, GOLD))
                            .corner_radius(egui::CornerRadius::same(20))
                            .inner_margin(egui::Margin::symmetric(13, 7))
                            .show(ui, |ui| {
                                ui.horizontal_centered(|ui| {
                                    ui.label(
                                        RichText::new(format!(
                                            "{:02}:{:02}",
                                            clock / 60,
                                            clock % 60
                                        ))
                                        .size(14.0)
                                        .monospace()
                                        .strong()
                                        .color(TEXT),
                                    );
                                    ui.label(RichText::new(kda).size(11.0).strong().color(TEXT));
                                    ui.label(
                                        RichText::new(format!("{} CS", num(scores, "creepScore")))
                                            .size(10.0)
                                            .color(GOLD),
                                    );
                                    if self.settings.hud_mayhem_cards
                                        && action_button(ui, "CARDS", self.show_mayhem_cards, GOLD)
                                            .clicked()
                                    {
                                        self.show_mayhem_cards = !self.show_mayhem_cards;
                                    }
                                    if action_button(ui, "×", false, RED).clicked() {
                                        self.set_overlay_mode(&ctx, false, false);
                                    }
                                });
                            });
                        self.overlay_interactive_regions
                            .push(pill_response.response.rect);
                        if self.show_mayhem_cards {
                            self.render_mayhem_overlay(&ctx, area);
                        }
                        return;
                    }
                    "card" => {
                        let card_response = Frame::new()
                            .fill(Color32::from_rgba_unmultiplied(7, 12, 22, 240))
                            .stroke(Stroke::new(1.0, CYAN))
                            .corner_radius(egui::CornerRadius::same(16))
                            .inner_margin(egui::Margin::same(14))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(
                                            RichText::new("LIVE TACTICAL CARD")
                                                .size(9.5)
                                                .strong()
                                                .color(CYAN),
                                        );
                                        ui.label(
                                            RichText::new(format!(
                                                "{:02}:{:02}  ·  {game_mode}",
                                                clock / 60,
                                                clock % 60
                                            ))
                                            .size(15.0)
                                            .strong()
                                            .color(TEXT),
                                        );
                                    });
                                    if action_button(ui, "×", false, RED).clicked() {
                                        self.set_overlay_mode(&ctx, false, false);
                                    }
                                });
                                ui.separator();
                                ui.horizontal(|ui| {
                                    metric(ui, "K / D / A", &kda, TEXT);
                                    metric(ui, "CS", &num(scores, "creepScore"), GOLD);
                                    metric(ui, "CS / MIN", &format!("{cs_per_minute:.1}"), GOLD);
                                });
                                if self.settings.hud_gold_lead {
                                    ui.label(
                                        RichText::new(format!(
                                            "CURRENT GOLD   {}",
                                            num(active, "currentGold")
                                        ))
                                        .size(11.0)
                                        .color(GREEN),
                                    );
                                }
                                if self.settings.hud_objective_timers {
                                    for (label, timer) in
                                        objective_timers(live.unwrap_or(&Value::Null))
                                    {
                                        ui.label(
                                            RichText::new(format!("{label:<9}  {timer}"))
                                                .size(10.5)
                                                .monospace()
                                                .color(TEXT),
                                        );
                                    }
                                }
                                ui.horizontal(|ui| {
                                    if self.settings.hud_mayhem_cards
                                        && action_button(
                                            ui,
                                            "MAYHEM & ARENA CARDS",
                                            self.show_mayhem_cards,
                                            GOLD,
                                        )
                                        .clicked()
                                    {
                                        self.show_mayhem_cards = !self.show_mayhem_cards;
                                    }
                                });
                            });
                        self.overlay_interactive_regions
                            .push(card_response.response.rect);
                        if self.show_mayhem_cards {
                            self.render_mayhem_overlay(&ctx, area);
                        }
                        return;
                    }
                    _ => {}
                }

                render_minimap_guide_pins(&ctx, area, live, &self.settings, clock);

                let center = area.center().x;
                let top_pos = self
                    .settings
                    .hud_positions
                    .get("scoreboard")
                    .map(|position| egui::pos2(position[0], position[1]))
                    .unwrap_or_else(|| egui::pos2(center - 285.0, 20.0));
                let top_response = egui::Area::new(ui.make_persistent_id("riftcore-hud-top"))
                    .current_pos(top_pos)
                    .movable(self.settings.hud_edit_layout)
                    .show(&ctx, |ui| {
                        Frame::new()
                            .fill(Color32::from_rgba_unmultiplied(5, 8, 17, 228))
                            .stroke(Stroke::new(1.0, CYAN))
                            .inner_margin(egui::Margin::same(10))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("◈ RIFTCORE").size(11.0).strong().color(CYAN),
                                    );
                                    ui.separator();
                                    ui.label(
                                        RichText::new(format!(
                                            "{:02}:{:02}",
                                            clock / 60,
                                            clock % 60
                                        ))
                                        .monospace()
                                        .size(18.0)
                                        .strong()
                                        .color(TEXT),
                                    );
                                    ui.separator();
                                    ui.label(RichText::new(kda).size(13.0).strong().color(TEXT));
                                    ui.separator();
                                    ui.label(
                                        RichText::new(format!("{} CS", num(scores, "creepScore")))
                                            .size(11.5)
                                            .color(GOLD),
                                    );
                                    if self.settings.hud_cs_benchmark {
                                        ui.separator();
                                        ui.label(
                                            RichText::new(format!("{cs_per_minute:.1}/m"))
                                                .size(11.0)
                                                .color(GOLD),
                                        );
                                    }
                                    if self.settings.hud_gold_lead {
                                        ui.separator();
                                        ui.label(
                                            RichText::new(format!(
                                                "{} G",
                                                num(active, "currentGold")
                                            ))
                                            .size(11.0)
                                            .color(GREEN),
                                        );
                                    }
                                });
                            });
                        ui.label(RichText::new(game_mode).size(9.0).strong().color(MUTED));
                    });
                if top_response.response.drag_stopped() {
                    self.settings.hud_positions.insert(
                        "scoreboard".to_string(),
                        [
                            top_response.response.rect.min.x,
                            top_response.response.rect.min.y,
                        ],
                    );
                    self.save_settings();
                }
                if self.settings.hud_edit_layout {
                    self.overlay_interactive_regions
                        .push(top_response.response.rect);
                }

                let dock_default = if self.settings.hud_minimap_flip {
                    egui::pos2(
                        area.left() + 16.0 + self.settings.hud_minimap_nudge_x,
                        area.bottom() - 150.0 + self.settings.hud_minimap_nudge_y,
                    )
                } else {
                    egui::pos2(
                        area.right() - 175.0 + self.settings.hud_minimap_nudge_x,
                        area.bottom() - 150.0 + self.settings.hud_minimap_nudge_y,
                    )
                };
                let dock_pos = self
                    .settings
                    .hud_positions
                    .get("objectives")
                    .map(|position| egui::pos2(position[0], position[1]))
                    .unwrap_or(dock_default);
                let dock_response = egui::Area::new(ui.make_persistent_id("riftcore-hud-left"))
                    .current_pos(dock_pos)
                    .movable(self.settings.hud_edit_layout)
                    .show(&ctx, |ui| {
                        Frame::new()
                            .fill(Color32::from_rgba_unmultiplied(5, 8, 17, 212))
                            .stroke(Stroke::new(1.0, BORDER))
                            .inner_margin(egui::Margin::same(10))
                            .show(ui, |ui| {
                                ui.set_min_width(150.0 * self.settings.hud_minimap_zoom);
                                ui.label(
                                    RichText::new("OBJECTIVES").size(10.0).strong().color(CYAN),
                                );
                                if self.settings.hud_objective_timers {
                                    if let Some(live) = live {
                                        for (label, timer) in objective_timers(live) {
                                            ui.label(
                                                RichText::new(format!("{label:<8} {timer}"))
                                                    .size(
                                                        (11.0 * self.settings.hud_minimap_zoom)
                                                            .clamp(9.0, 15.0),
                                                    )
                                                    .monospace()
                                                    .color(TEXT),
                                            );
                                        }
                                    }
                                }
                            });
                    });
                if dock_response.response.drag_stopped() {
                    self.settings.hud_positions.insert(
                        "objectives".to_string(),
                        [
                            dock_response.response.rect.min.x,
                            dock_response.response.rect.min.y,
                        ],
                    );
                    self.save_settings();
                }
                if self.settings.hud_edit_layout {
                    self.overlay_interactive_regions
                        .push(dock_response.response.rect);
                }

                if self.settings.hud_aram_health_timers && game_mode.eq_ignore_ascii_case("ARAM") {
                    let response =
                        egui::Area::new(ui.make_persistent_id("riftcore-hud-aram-relics"))
                            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(20.0, -110.0))
                            .show(&ctx, |ui| self.render_aram_relic_timers(ui));
                    self.overlay_interactive_regions
                        .push(response.response.rect);
                }

                if self.settings.hud_ally_ult_timers {
                    if let Some(live) = live {
                        let response =
                            egui::Area::new(ui.make_persistent_id("riftcore-hud-ally-ults"))
                                .fixed_pos(egui::pos2(area.left() + 18.0, area.top() + 116.0))
                                .show(&ctx, |ui| self.render_ally_ultimate_timers(ui, live));
                        self.overlay_interactive_regions
                            .push(response.response.rect);
                    }
                }

                let player_pos = self
                    .settings
                    .hud_positions
                    .get("player")
                    .map(|position| egui::pos2(position[0], position[1]))
                    .unwrap_or_else(|| egui::pos2((area.right() - 220.0).max(20.0), 98.0));
                let player_response = egui::Area::new(ui.make_persistent_id("riftcore-hud-player"))
                    .current_pos(player_pos)
                    .movable(self.settings.hud_edit_layout)
                    .show(&ctx, |ui| {
                        Frame::new()
                            .fill(Color32::from_rgba_unmultiplied(5, 8, 17, 212))
                            .stroke(Stroke::new(1.0, BORDER))
                            .inner_margin(egui::Margin::same(10))
                            .show(ui, |ui| {
                                ui.set_min_width(180.0);
                                if self.settings.hud_skill_order {
                                    ui.label(
                                        RichText::new("ABILITY RANKS")
                                            .size(10.0)
                                            .strong()
                                            .color(CYAN),
                                    );
                                    let abilities = active.get("abilities").unwrap_or(&Value::Null);
                                    ui.horizontal(|ui| {
                                        for slot in ["Q", "W", "E", "R"] {
                                            let rank = abilities
                                                .get(slot)
                                                .and_then(|ability| ability.get("abilityLevel"))
                                                .and_then(Value::as_u64)
                                                .unwrap_or(0);
                                            ui.label(
                                                RichText::new(format!("{slot} {rank}"))
                                                    .size(10.5)
                                                    .monospace()
                                                    .color(TEXT),
                                            );
                                        }
                                    });
                                }
                                if self.settings.hud_item_goal {
                                    ui.add_space(5.0);
                                    ui.label(
                                        RichText::new("INVENTORY").size(10.0).strong().color(GOLD),
                                    );
                                    if let Some(items) = me.get("items").and_then(Value::as_array) {
                                        let mut shown = 0;
                                        for item in items
                                            .iter()
                                            .filter(|item| {
                                                item.get("itemID")
                                                    .and_then(Value::as_u64)
                                                    .unwrap_or(0)
                                                    > 0
                                            })
                                            .take(7)
                                        {
                                            let name = item
                                                .get("displayName")
                                                .and_then(Value::as_str)
                                                .unwrap_or("Item");
                                            let count = item
                                                .get("count")
                                                .and_then(Value::as_u64)
                                                .unwrap_or(1);
                                            ui.label(
                                                RichText::new(if count > 1 {
                                                    format!("{name} ×{count}")
                                                } else {
                                                    name.to_string()
                                                })
                                                .size(10.0)
                                                .color(SECONDARY),
                                            );
                                            shown += 1;
                                        }
                                        if shown == 0 {
                                            ui.label(
                                                RichText::new("No items yet")
                                                    .size(10.0)
                                                    .color(MUTED),
                                            );
                                        }
                                    }
                                }
                                if self.settings.hud_trinket_reminder {
                                    let has_trinket = me
                                        .get("items")
                                        .and_then(Value::as_array)
                                        .map(|items| {
                                            items.iter().any(|item| {
                                                item.get("slot").and_then(Value::as_i64) == Some(6)
                                                    && item
                                                        .get("itemID")
                                                        .and_then(Value::as_u64)
                                                        .unwrap_or(0)
                                                        > 0
                                            })
                                        })
                                        .unwrap_or(false);
                                    if !has_trinket {
                                        ui.add_space(5.0);
                                        ui.label(
                                            RichText::new("TRINKET AVAILABLE")
                                                .size(10.0)
                                                .strong()
                                                .color(GOLD),
                                        );
                                    }
                                }
                            });
                    });
                if player_response.response.drag_stopped() {
                    self.settings.hud_positions.insert(
                        "player".to_string(),
                        [
                            player_response.response.rect.min.x,
                            player_response.response.rect.min.y,
                        ],
                    );
                    self.save_settings();
                }
                if self.settings.hud_edit_layout {
                    self.overlay_interactive_regions
                        .push(player_response.response.rect);
                }

                let toolbar_response =
                    egui::Area::new(ui.make_persistent_id("riftcore-hud-bottom"))
                        .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -18.0))
                        .show(&ctx, |ui| {
                            Frame::new()
                                .fill(Color32::from_rgba_unmultiplied(5, 8, 17, 218))
                                .stroke(Stroke::new(1.0, BORDER))
                                .inner_margin(egui::Margin::same(8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new("ALT + ~  HUD")
                                                .size(10.0)
                                                .strong()
                                                .color(CYAN),
                                        );
                                        ui.label(
                                            RichText::new("ALT + SHIFT + ~  INPUT")
                                                .size(10.0)
                                                .color(SECONDARY),
                                        );
                                        if !self.overlay_mouse_passthrough {
                                            ui.label(
                                                RichText::new("SMART INPUT")
                                                    .size(9.0)
                                                    .strong()
                                                    .color(GOLD),
                                            );
                                        }
                                        if self.settings.hud_mayhem_cards
                                            && action_button(
                                                ui,
                                                "MAYHEM CARDS",
                                                self.show_mayhem_cards,
                                                GOLD,
                                            )
                                            .clicked()
                                        {
                                            self.show_mayhem_cards = !self.show_mayhem_cards;
                                            if self.show_mayhem_cards {
                                                self.show_overlay_build = false;
                                            }
                                        }
                                        if action_button(ui, "BUILD", self.show_overlay_build, CYAN)
                                            .clicked()
                                        {
                                            self.show_overlay_build = !self.show_overlay_build;
                                            if self.show_overlay_build {
                                                self.show_mayhem_cards = false;
                                                self.show_overlay_timers = false;
                                            }
                                        }
                                        if action_button(
                                            ui,
                                            "TIMERS",
                                            self.show_overlay_timers,
                                            GOLD,
                                        )
                                        .clicked()
                                        {
                                            self.show_overlay_timers = !self.show_overlay_timers;
                                            if self.show_overlay_timers {
                                                self.show_overlay_build = false;
                                                self.show_mayhem_cards = false;
                                            }
                                        }
                                        if self.settings.hud_jungle_pathing
                                            && !game_mode.eq_ignore_ascii_case("ARAM")
                                        {
                                            if let Some((step, camp, remaining, active)) =
                                                jungle_route_status(
                                                    clock,
                                                    &self.settings.jungle_start_buff,
                                                )
                                            {
                                                ui.label(
                                                    RichText::new(format!(
                                                        "JUNGLE #{step} · {camp} {}",
                                                        format_hud_clock(remaining)
                                                    ))
                                                    .size(9.5)
                                                    .strong()
                                                    .color(if active { GREEN } else { SECONDARY }),
                                                );
                                            } else {
                                                ui.label(
                                                    RichText::new("JUNGLE ROUTE · COMPLETE")
                                                        .size(9.0)
                                                        .color(MUTED),
                                                );
                                            }
                                            if action_button(
                                                ui,
                                                "RED",
                                                self.settings.jungle_start_buff == "red",
                                                RED,
                                            )
                                            .clicked()
                                            {
                                                self.settings.jungle_start_buff = "red".to_string();
                                                self.save_settings();
                                            }
                                            if action_button(
                                                ui,
                                                "BLUE",
                                                self.settings.jungle_start_buff == "blue",
                                                CYAN,
                                            )
                                            .clicked()
                                            {
                                                self.settings.jungle_start_buff =
                                                    "blue".to_string();
                                                self.save_settings();
                                            }
                                        }
                                        if action_button(
                                            ui,
                                            if self.settings.hud_edit_layout {
                                                "DONE EDITING"
                                            } else {
                                                "EDIT LAYOUT"
                                            },
                                            self.settings.hud_edit_layout,
                                            CYAN,
                                        )
                                        .clicked()
                                        {
                                            self.settings.hud_edit_layout =
                                                !self.settings.hud_edit_layout;
                                            self.overlay_mouse_passthrough = false;
                                            ctx.send_viewport_cmd(
                                                egui::ViewportCommand::MousePassthrough(false),
                                            );
                                            self.save_settings();
                                        }
                                        if action_button(
                                            ui,
                                            if self.overlay_mouse_passthrough {
                                                "SMART INPUT"
                                            } else {
                                                "GAME INPUT"
                                            },
                                            false,
                                            GOLD,
                                        )
                                        .clicked()
                                        {
                                            self.overlay_mouse_passthrough =
                                                !self.overlay_mouse_passthrough;
                                            ctx.send_viewport_cmd(
                                                egui::ViewportCommand::MousePassthrough(
                                                    self.overlay_mouse_passthrough,
                                                ),
                                            );
                                        }
                                        if action_button(ui, "HIDE HUD", false, RED).clicked() {
                                            self.set_overlay_mode(&ctx, false, false);
                                        }
                                    });
                                });
                        });
                self.overlay_interactive_regions
                    .push(toolbar_response.response.rect);
                if self.show_overlay_build {
                    self.render_overlay_build(&ctx, area);
                }
                if self.show_mayhem_cards {
                    self.render_mayhem_overlay(&ctx, area);
                }
                if self.show_overlay_timers {
                    self.render_overlay_timers(&ctx, area, live);
                }
            });
    }
}

impl eframe::App for RiftCoreApp {
    fn on_exit(&mut self) {
        if let Some(path_b) = self.path_b.as_mut() {
            path_b.close();
        }
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // eframe skips `ui` for a hidden window, but RiftCore does its background work there
        // (tray menu, League client updates, hotkeys, HUD auto-start). Keep it running while
        // the window sits in the tray, is minimised, or is covered (e.g. by the game).
        let hidden = ctx.input(|input| input.viewport().visible() == Some(false));
        if self.hidden_in_tray || hidden {
            ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaintWhileHidden);
            ctx.request_repaint_after(Duration::from_millis(250));
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // The child HUD viewport is a per-pixel transparent window. An opaque GL
        // clear turns the unused part of that viewport into a black rectangle.
        [0.0, 0.0, 0.0, 0.0]
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        #[cfg(windows)]
        {
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                match event.id.as_ref() {
                    "riftcore-show" => self.show_from_tray(&ctx),
                    "riftcore-hud" => self.set_overlay_mode(&ctx, !self.overlay_mode, false),
                    "riftcore-exit" => {
                        self.force_exit = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    _ => {}
                }
            }
            while let Ok(event) = tray_icon::TrayIconEvent::receiver().try_recv() {
                // Hovering the icon also produces events; only a left click reopens.
                let clicked = matches!(
                    event,
                    tray_icon::TrayIconEvent::Click {
                        button: tray_icon::MouseButton::Left,
                        button_state: tray_icon::MouseButtonState::Up,
                        ..
                    } | tray_icon::TrayIconEvent::DoubleClick {
                        button: tray_icon::MouseButton::Left,
                        ..
                    }
                );
                if clicked {
                    self.show_from_tray(&ctx);
                }
            }
        }
        let close_requested = ctx.input(|input| input.viewport().close_requested());
        if close_requested && !self.force_exit {
            if !self.settings.close_confirmation_seen {
                self.show_close_confirmation = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            } else if self.settings.close_to_tray {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                if self.tray_icon_available() {
                    self.hidden_in_tray = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }
            }
        }
        while let Ok(snapshot) = self.snapshot_rx.try_recv() {
            if snapshot.riot_lookup.is_some() {
                self.profile_lookup_loading = false;
            }
            if let Some(notice) = snapshot.notice.clone() {
                if self.snapshot.notice != Some(notice.clone()) {
                    self.set_notice(notice);
                }
            }
            if let Some(path_b) = self.path_b.as_mut() {
                path_b.set_live_game(snapshot.live_game.as_ref());
            }
            self.snapshot = snapshot;
            if self.snapshot.phase == "ChampSelect" {
                let session = self.snapshot.champ_select.as_ref();
                let shown = session.and_then(champ_select_local_champion).unwrap_or(0);
                let locked = session.and_then(champ_select_locked_champion).unwrap_or(0);
                // Hovering only previews the setup; the League client is written once,
                // when the pick is locked in.
                if locked > 0 && locked != self.last_auto_imported_champion {
                    self.last_auto_imported_champion = locked;
                    self.select_champion_setup(locked, true);
                } else if shown > 0 && shown != self.last_synced_champion {
                    self.select_champion_setup(shown, false);
                }
            } else {
                self.last_auto_imported_champion = 0;
                self.last_synced_champion = 0;
            }
        }
        if self
            .snapshot
            .update_status
            .as_deref()
            .is_some_and(|status| status == "Installing update and restarting…")
            && !self.force_exit
        {
            self.force_exit = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        while let Ok(event) = self.hotkey_rx.try_recv() {
            if event.toggle_overlay {
                self.set_overlay_mode(&ctx, !self.overlay_mode, false);
            }
            if event.toggle_passthrough && self.overlay_mode {
                self.overlay_mouse_passthrough = !self.overlay_mouse_passthrough;
                if self.overlay_viewport_open {
                    ctx.send_viewport_cmd_to(
                        overlay_viewport_id(),
                        egui::ViewportCommand::MousePassthrough(self.overlay_mouse_passthrough),
                    );
                }
            }
        }

        let ready = self
            .snapshot
            .ready_check
            .as_ref()
            .map(|check| {
                check.get("state").and_then(Value::as_str) == Some("InProgress")
                    && check.get("playerResponse").and_then(Value::as_str) == Some("None")
            })
            .unwrap_or(false);
        if ready && !self.ready_check_alerted && self.settings.sound_alerts {
            play_ready_alert();
        }
        self.ready_check_alerted = ready;

        // One failed live-client poll doesn't end the match (see `LiveGameLatch`).
        let has_live_data = self
            .snapshot
            .live_game
            .as_ref()
            .is_some_and(|data| data.get("gameData").is_some());
        let live = self.live_game_latch.update(has_live_data, Instant::now());
        if live && !self.overlay_mode && !self.overlay_dismissed_by_user {
            self.set_overlay_mode(&ctx, true, true);
        } else if !live && self.overlay_mode && self.overlay_auto_entered {
            self.set_overlay_mode(&ctx, false, true);
        }
        if !live {
            self.overlay_dismissed_by_user = false;
        }
        if let Some(path_b) = self.path_b.as_ref() {
            // The new HUD already searches for the game window every 250 ms.
            let found = path_b.league_window();
            self.overlay_target = found.map(|(window, _)| backend::LeagueWindowInfo {
                hwnd: window.hwnd,
                process_id: window.process_id,
                bounds: window.bounds,
                dpi: window.dpi,
            });
            self.overlay_target_is_foreground = found.is_some_and(|(_, foreground)| foreground);
            self.overlay_waiting_for_window = self.overlay_mode
                && (self.overlay_target.is_none() || !self.overlay_target_is_foreground);
        } else if self.last_bounds_check.elapsed() > Duration::from_millis(250) {
            self.overlay_target = backend::league_game_window();
            self.overlay_target_is_foreground = self
                .overlay_target
                .map(backend::league_window_is_foreground)
                .unwrap_or(false);
            self.overlay_waiting_for_window = self.overlay_mode
                && (self.overlay_target.is_none() || !self.overlay_target_is_foreground);
            self.last_bounds_check = Instant::now();
        }

        self.render_titlebar(ui);
        ui.add_enabled_ui(!self.show_whats_new && !self.show_quick_tour, |ui| {
            self.render_sidebar(ui);
            self.render_main(ui);
        });
        self.render_dialogs(&ctx);
        if self.path_b.is_some() {
            self.tick_path_b(&ctx);
        } else {
            self.render_overlay_viewport(&ctx);
        }
        // The new HUD schedules its own repaints in `tick_path_b`; only the classic
        // fallback HUD needs a fast fixed cadence.
        let fast = self.overlay_mode && self.path_b.is_none();
        ctx.request_repaint_after(Duration::from_millis(if fast { 33 } else { 250 }));
    }
}

fn paint_command_grid(_ui: &egui::Ui) {}

fn panel(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    let inner_width = (ui.available_width() - 32.0).max(0.0);
    Frame::new()
        .fill(CARD)
        .corner_radius(egui::CornerRadius::same(18))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_min_width(inner_width);
            add_contents(ui);
        });
}

fn panel_title(ui: &mut egui::Ui, title: &str, description: &str) {
    ui.horizontal(|ui| {
        let (accent, _) = ui.allocate_exact_size(Vec2::new(3.0, 30.0), Sense::hover());
        ui.painter()
            .rect_filled(accent, egui::CornerRadius::same(2), CYAN);
        ui.add_space(4.0);
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(15.0).strong().color(TEXT));
            ui.label(RichText::new(description).size(12.5).color(SECONDARY));
        });
    });
    ui.add_space(12.0);
}

fn nav_section(ui: &mut egui::Ui, label: &str) {
    ui.add_space(13.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label.to_uppercase())
                .size(9.5)
                .strong()
                .color(MUTED),
        );
        let (line, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
        ui.painter().line_segment(
            [line.left_center(), line.right_center()],
            Stroke::new(1.0, BORDER),
        );
    });
    ui.add_space(6.0);
}

fn checkbox_row(ui: &mut egui::Ui, label: &str, value: &mut bool) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(12.5).color(SECONDARY));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            changed = ui.checkbox(value, "").changed();
        });
    });
    changed
}

fn tier_color(tier: &str) -> Color32 {
    match tier.to_uppercase().as_str() {
        "IRON" => Color32::from_rgb(117, 107, 107),
        "BRONZE" => Color32::from_rgb(165, 113, 78),
        "SILVER" => Color32::from_rgb(168, 187, 199),
        "GOLD" => Color32::from_rgb(234, 186, 70),
        "PLATINUM" => Color32::from_rgb(46, 178, 142),
        "EMERALD" => Color32::from_rgb(26, 188, 114),
        "DIAMOND" => Color32::from_rgb(112, 169, 255),
        "MASTER" => Color32::from_rgb(187, 98, 255),
        "GRANDMASTER" => Color32::from_rgb(243, 67, 67),
        "CHALLENGER" => Color32::from_rgb(244, 201, 74),
        _ => Color32::from_rgb(130, 145, 166),
    }
}

fn custom_progress_bar(ui: &mut egui::Ui, fraction: f32, color: Color32, height: f32) {
    let width = ui.available_width().max(40.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let r = (height * 0.5).round() as u8;
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(r), RAISED);
    let fill_width = (rect.width() * fraction.clamp(0.0, 1.0)).max(0.0);
    if fill_width > 0.0 {
        let fill_rect = egui::Rect::from_min_size(rect.min, Vec2::new(fill_width, height));
        ui.painter()
            .rect_filled(fill_rect, egui::CornerRadius::same(r), color);
    }
}

fn toggle_switch(ui: &mut egui::Ui, value: &mut bool) -> egui::Response {
    let desired_size = Vec2::new(36.0, 20.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }
    let how_on = ui.ctx().animate_bool(response.id, *value);
    let bg_color = lerp_color(RAISED, CYAN, how_on);
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(10), bg_color);
    let circle_x = egui::lerp((rect.left() + 10.0)..=(rect.right() - 10.0), how_on);
    let center = egui::pos2(circle_x, rect.center().y);
    ui.painter().circle_filled(center, 7.0, Color32::WHITE);
    response
}

fn toggle_card_row(ui: &mut egui::Ui, title: &str, subtitle: &str, value: &mut bool) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(12.0).strong().color(TEXT));
            if !subtitle.is_empty() {
                ui.label(RichText::new(subtitle).size(10.0).color(SECONDARY));
            }
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if toggle_switch(ui, value).changed() {
                changed = true;
            }
        });
    });
    ui.add_space(4.0);
    changed
}

fn render_hud_mockup(ui: &mut egui::Ui, hud_mode: &str) {
    let width = ui.available_width().clamp(280.0, 680.0);
    let height = (width * 0.38).clamp(110.0, 240.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, egui::CornerRadius::same(10), INSET);
    painter.rect_stroke(
        rect,
        egui::CornerRadius::same(10),
        Stroke::new(1.0, BORDER),
        egui::StrokeKind::Inside,
    );

    let top_center = egui::pos2(rect.center().x, rect.top() + 18.0);
    let top_bar_rect = egui::Rect::from_center_size(top_center, Vec2::new(width * 0.54, 22.0));
    painter.rect_filled(top_bar_rect, egui::CornerRadius::same(6), INSET);
    painter.rect_stroke(
        top_bar_rect,
        egui::CornerRadius::same(6),
        Stroke::new(1.0, CYAN),
        egui::StrokeKind::Inside,
    );
    painter.text(
        top_bar_rect.center(),
        egui::Align2::CENTER_CENTER,
        "◈ 14:22  ·  3 / 1 / 5  ·  128 CS (9.0/m)  ·  1,450 G",
        FontId::proportional(9.5),
        Color32::WHITE,
    );

    let map_size = height * 0.58;
    let map_rect = egui::Rect::from_min_size(
        egui::pos2(
            rect.right() - map_size - 14.0,
            rect.bottom() - map_size - 12.0,
        ),
        Vec2::splat(map_size),
    );
    painter.rect_filled(map_rect, egui::CornerRadius::same(6), INSET);
    painter.rect_stroke(
        map_rect,
        egui::CornerRadius::same(6),
        Stroke::new(1.0, BORDER),
        egui::StrokeKind::Inside,
    );
    painter.text(
        egui::pos2(map_rect.left() + 6.0, map_rect.top() + 6.0),
        egui::Align2::LEFT_TOP,
        "MINIMAP",
        FontId::proportional(8.5),
        MUTED,
    );

    painter.circle_filled(map_rect.center(), 4.0, CYAN);
    painter.circle_filled(
        egui::pos2(map_rect.left() + 22.0, map_rect.bottom() - 22.0),
        3.5,
        GREEN,
    );
    painter.circle_filled(
        egui::pos2(map_rect.right() - 22.0, map_rect.top() + 22.0),
        3.5,
        RED,
    );

    if hud_mode == "card" || hud_mode == "integrated" {
        let card_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 16.0, rect.top() + 20.0),
            Vec2::new(130.0, height * 0.70),
        );
        painter.rect_filled(card_rect, egui::CornerRadius::same(8), INSET);
        painter.rect_stroke(
            card_rect,
            egui::CornerRadius::same(8),
            Stroke::new(1.0, GOLD),
            egui::StrokeKind::Inside,
        );
        painter.text(
            egui::pos2(card_rect.left() + 8.0, card_rect.top() + 8.0),
            egui::Align2::LEFT_TOP,
            "TACTICAL TIMERS",
            FontId::proportional(8.5),
            GOLD,
        );
        painter.text(
            egui::pos2(card_rect.left() + 8.0, card_rect.top() + 24.0),
            egui::Align2::LEFT_TOP,
            "Dragon:  01:45",
            FontId::proportional(8.5),
            TEXT,
        );
        painter.text(
            egui::pos2(card_rect.left() + 8.0, card_rect.top() + 38.0),
            egui::Align2::LEFT_TOP,
            "Baron:   04:10",
            FontId::proportional(8.5),
            SECONDARY,
        );
        painter.text(
            egui::pos2(card_rect.left() + 8.0, card_rect.top() + 52.0),
            egui::Align2::LEFT_TOP,
            "Ultimates: Ready",
            FontId::proportional(8.5),
            GREEN,
        );
    }
}

fn settings_row(ui: &mut egui::Ui, label: &str, value: &mut bool, description: &str) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(label).size(12.5).strong().color(TEXT));
            ui.label(RichText::new(description).size(10.5).color(MUTED));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            changed = ui.checkbox(value, "").changed();
        });
    });
    ui.separator();
    changed
}

fn trait_chip(ui: &mut egui::Ui, name: &str) {
    Frame::new()
        .fill(Color32::from_rgba_unmultiplied(
            CYAN.r(),
            CYAN.g(),
            CYAN.b(),
            22,
        ))
        .stroke(Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(CYAN.r(), CYAN.g(), CYAN.b(), 110),
        ))
        .corner_radius(egui::CornerRadius::same(5))
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.add(egui::Label::new(RichText::new(name).size(10.0).color(CYAN)).extend());
        });
}

/// Strips Riot's tooltip markup (`<br>`, `<rules>…</rules>`, other tags) from Data Dragon text.
fn clean_tft_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        let after = &rest[start..];
        if after.starts_with("<rules>") {
            rest = after
                .find("</rules>")
                .map(|end| &after[end + "</rules>".len()..])
                .unwrap_or("");
        } else if let Some(end) = after.find('>') {
            out.push(' ');
            rest = &after[end + 1..];
        } else {
            rest = "";
        }
    }
    out.push_str(rest);
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn tft_cost_color(cost: u32) -> Color32 {
    match cost {
        1 => Color32::from_rgb(150, 160, 175),
        2 => Color32::from_rgb(60, 200, 120),
        3 => Color32::from_rgb(60, 140, 240),
        4 => Color32::from_rgb(190, 90, 240),
        5 => Color32::from_rgb(245, 190, 60),
        _ => BORDER,
    }
}

fn tier_badge(ui: &mut egui::Ui, tier: &str) {
    let color = match tier {
        "S" => Color32::from_rgb(255, 110, 110),
        "A" => Color32::from_rgb(245, 190, 60),
        "B" => Color32::from_rgb(60, 200, 120),
        _ => SECONDARY,
    };
    Frame::new()
        .fill(Color32::from_rgba_unmultiplied(
            color.r(),
            color.g(),
            color.b(),
            40,
        ))
        .stroke(Stroke::new(1.0, color))
        .corner_radius(egui::CornerRadius::same(5))
        .inner_margin(egui::Margin::symmetric(7, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(tier).size(11.0).strong().color(color));
        });
}

fn badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    Frame::new()
        .fill(Color32::from_rgba_unmultiplied(
            color.r(),
            color.g(),
            color.b(),
            30,
        ))
        .stroke(Stroke::new(
            0.8,
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 90),
        ))
        .corner_radius(egui::CornerRadius::same(5))
        .inner_margin(egui::Margin::symmetric(7, 3))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .size(9.0)
                    .strong()
                    .monospace()
                    .color(color),
            );
        });
}

fn filter_chip(ui: &mut egui::Ui, text: &str, selected: bool, accent: Color32) -> egui::Response {
    let width = (text.chars().count() as f32 * 6.2 + 22.0).max(44.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 28.0), Sense::click());
    let hovered =
        ui.ctx()
            .animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.12);
    let active = ui
        .ctx()
        .animate_bool_with_time(response.id.with("selected"), selected, 0.16);
    let emphasis = active.max(hovered * 0.5);
    ui.painter().rect_filled(
        rect,
        egui::CornerRadius::same(5),
        lerp_color(CARD, ACCENT_SOFT, emphasis),
    );
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(5),
        Stroke::new(0.8 + active * 0.7, lerp_color(BORDER, accent, emphasis)),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        FontId::proportional(9.5),
        lerp_color(SECONDARY, accent, active),
    );
    response
}

/// Fades and slides the rest of `ui` in whenever `key` changes (page or sub-tab switch).
fn page_transition(ui: &mut egui::Ui, key: impl std::hash::Hash) {
    const DURATION: f64 = 0.24;
    let key = {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    };
    let now = ui.input(|input| input.time);
    let started = ui.ctx().data_mut(|data| {
        let entry = data
            .get_temp_mut_or_insert_with(egui::Id::unique("riftcore-page-transition"), || {
                (key, now - DURATION)
            });
        if entry.0 != key {
            *entry = (key, now);
        }
        entry.1
    });
    let progress = ((now - started) / DURATION).clamp(0.0, 1.0) as f32;
    let eased = 1.0 - (1.0 - progress).powi(3);
    if progress < 1.0 {
        ui.ctx().request_repaint();
    }
    ui.multiply_opacity(eased);
    ui.add_space(12.0 * (1.0 - eased));
}

fn lerp_color(start: Color32, end: Color32, amount: f32) -> Color32 {
    let amount = amount.clamp(0.0, 1.0);
    let channel =
        |from: u8, to: u8| (from as f32 + (to as f32 - from as f32) * amount).round() as u8;
    Color32::from_rgba_unmultiplied(
        channel(start.r(), end.r()),
        channel(start.g(), end.g()),
        channel(start.b(), end.b()),
        channel(start.a(), end.a()),
    )
}

fn titlebar_badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(44.0, 18.0), Sense::hover());
    let fill = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 18);
    let stroke = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 110);
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(6), fill);
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(6),
        Stroke::new(0.8, stroke),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        FontId::proportional(7.5),
        color,
    );
}

fn titlebar_status(ui: &mut egui::Ui, text: &str, color: Color32) {
    let width = 23.0 + text.chars().count() as f32 * 5.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 20.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(7), CARD);
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(7),
        Stroke::new(1.0, BORDER),
        egui::StrokeKind::Inside,
    );
    ui.painter()
        .circle_filled(egui::pos2(rect.left() + 8.0, rect.center().y), 3.0, color);
    ui.painter().text(
        egui::pos2(rect.left() + 15.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        text,
        FontId::proportional(8.5),
        SECONDARY,
    );
}

fn titlebar_action_button(
    ui: &mut egui::Ui,
    text: &str,
    active: bool,
    accent: Color32,
) -> egui::Response {
    let width = (text.chars().count() as f32 * 5.7 + 14.0).max(36.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 24.0), Sense::click());
    let fill = if active {
        ACCENT_SOFT
    } else if response.hovered() {
        RAISED
    } else {
        CARD
    };
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(7), fill);
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(7),
        Stroke::new(
            if active { 0.9 } else { 0.6 },
            if active { accent } else { BORDER },
        ),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        FontId::proportional(8.5),
        if active { accent } else { TEXT },
    );
    response
}

fn metric(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    Frame::new()
        .fill(SURFACE)
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_min_width(120.0);
            ui.label(RichText::new(label).size(10.0).strong().color(MUTED));
            ui.label(RichText::new(value).size(16.0).strong().color(color));
        });
}

fn action_button(ui: &mut egui::Ui, text: &str, active: bool, accent: Color32) -> egui::Response {
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), FontId::proportional(12.0), TEXT);
    let padding = ui.spacing().button_padding;
    let size = Vec2::new(
        galley.size().x + padding.x * 2.0,
        (galley.size().y + padding.y * 2.0).max(ui.spacing().interact_size.y),
    );
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let ctx = ui.ctx();
        let hover = ctx.animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.14);
        let on = ctx.animate_bool_with_time(response.id.with("active"), active, 0.2);
        let press = ctx.animate_bool_with_time(
            response.id.with("press"),
            response.is_pointer_button_down_on(),
            0.08,
        );
        let fill = lerp_color(
            lerp_color(RAISED, BUTTON_HOVER, hover),
            lerp_color(ACCENT_SOFT, CYAN_DIM, hover),
            on,
        );
        let shape = rect.shrink(press * 1.5);
        let radius = egui::CornerRadius::same((shape.height() * 0.5).min(255.0) as u8);
        ui.painter().rect_filled(shape, radius, fill);
        if on > 0.01 {
            ui.painter().rect_stroke(
                shape,
                radius,
                Stroke::new(1.0, accent.gamma_multiply(0.55 * on)),
                egui::StrokeKind::Inside,
            );
        }
        let text_pos = shape.center() - galley.size() * 0.5;
        ui.painter()
            .galley(text_pos, galley, lerp_color(TEXT, accent, on));
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn window_button(ui: &mut egui::Ui, text: &str, color: Color32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(5),
            if color == RED {
                Color32::from_rgb(82, 31, 43)
            } else {
                CARD
            },
        );
    }
    let glyph_color = if color == RED && !response.hovered() {
        MUTED
    } else {
        color
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        FontId::proportional(11.0),
        glyph_color,
    );
    response
}

fn avatar(
    ui: &mut egui::Ui,
    label: &str,
    size: f32,
    color: Color32,
    portrait: Option<&egui::TextureHandle>,
) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    if let Some(texture) = portrait {
        egui::Image::new((texture.id(), rect.size()))
            .corner_radius(egui::CornerRadius::same((size * 0.48) as u8))
            .paint_at(ui, rect);
    } else {
        ui.painter()
            .circle_filled(rect.center(), size * 0.48, RAISED);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            initials(label),
            FontId::proportional(size * 0.30),
            color,
        );
    }
    ui.painter()
        .circle_stroke(rect.center(), size * 0.48, Stroke::new(1.4, color));
}

fn texture_icon(
    ui: &mut egui::Ui,
    textures: &HashMap<String, egui::TextureHandle>,
    key: &str,
    size: f32,
) {
    if let Some(texture) = textures.get(key) {
        ui.add(egui::Image::new((texture.id(), Vec2::splat(size))));
    }
}

fn load_png_texture(
    ctx: &egui::Context,
    name: String,
    bytes: &[u8],
    size: Option<u32>,
) -> Option<egui::TextureHandle> {
    let image = image::load_from_memory(bytes).ok()?;
    let image = match size {
        Some(size) => image.resize_exact(size, size, image::imageops::FilterType::Lanczos3),
        None => image,
    }
    .into_rgba8();
    let dimensions = [image.width() as usize, image.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(dimensions, image.as_raw());
    Some(ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR))
}

fn initials(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

fn phase_name(phase: &str) -> String {
    match phase {
        "None" | "" => "Client Idle".to_string(),
        "ReadyCheck" => "Queue Pop".to_string(),
        "ChampSelect" => "Champ Select".to_string(),
        "InProgress" | "GameStart" => "Live Match".to_string(),
        "WaitingForStats" | "EndOfGame" => "Post Game".to_string(),
        "Matchmaking" => "In Queue".to_string(),
        "Lobby" => "Lobby".to_string(),
        other => other.to_string(),
    }
}

fn num(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_f64)
        .map(|value| {
            if value.fract() == 0.0 {
                format!("{value:.0}")
            } else {
                format!("{value:.1}")
            }
        })
        .unwrap_or_else(|| "0".to_string())
}

fn normalize_local_match(game: &Value, puuid: &str, champions: &[Champion]) -> Option<Value> {
    let identities = game
        .get("participantIdentities")
        .and_then(Value::as_array)?;
    let identity = identities
        .iter()
        .find(|identity| {
            identity
                .get("player")
                .and_then(|player| player.get("puuid"))
                .and_then(Value::as_str)
                .is_some_and(|id| !puuid.is_empty() && id == puuid)
        })
        .or_else(|| identities.first())?;
    let participant_id = identity
        .get("participantId")
        .and_then(Value::as_u64)
        .unwrap_or(1);
    let participant = game
        .get("participants")
        .and_then(Value::as_array)?
        .iter()
        .find(|participant| {
            participant.get("participantId").and_then(Value::as_u64) == Some(participant_id)
        })?;
    let stats = participant.get("stats").unwrap_or(&Value::Null);
    let duration = game
        .get("gameDuration")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let duration = if duration > 86_400 {
        duration / 1000
    } else {
        duration
    };
    let cs = stats
        .get("totalMinionsKilled")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        + stats
            .get("neutralMinionsKilled")
            .and_then(Value::as_u64)
            .unwrap_or(0);
    let champion_id = participant
        .get("championId")
        .and_then(Value::as_u64)
        .unwrap_or(0) as u32;
    let champion_name = champions
        .iter()
        .find(|champion| champion.id == champion_id)
        .map(|champion| champion.name.as_str())
        .unwrap_or("Champion");
    let items = (0..7)
        .map(|index| {
            stats
                .get(format!("item{index}"))
                .and_then(Value::as_u64)
                .unwrap_or(0)
        })
        .collect::<Vec<_>>();
    Some(serde_json::json!({
        "championName": champion_name,
        "win": stats.get("win").and_then(Value::as_bool).unwrap_or(false),
        "kills": stats.get("kills").and_then(Value::as_u64).unwrap_or(0),
        "deaths": stats.get("deaths").and_then(Value::as_u64).unwrap_or(0),
        "assists": stats.get("assists").and_then(Value::as_u64).unwrap_or(0),
        "cs": cs,
        "csPerMin": cs as f64 / (duration as f64 / 60.0).max(1.0),
        "duration": duration,
        "gold": stats.get("goldEarned").and_then(Value::as_u64).unwrap_or(0),
        "damage": stats.get("totalDamageDealtToChampions").and_then(Value::as_u64).unwrap_or(0),
        "vision": stats.get("visionScore").and_then(Value::as_u64).unwrap_or(0),
        "gameMode": friendly_mode(game.get("gameMode").and_then(Value::as_str).unwrap_or("League"), game.get("queueId").and_then(Value::as_u64).unwrap_or(0)),
        "queueId": game.get("queueId").and_then(Value::as_u64).unwrap_or(0),
        "items": items
    }))
}

fn normalize_riot_match(game: &Value, puuid: &str) -> Option<Value> {
    let info = game.get("info")?;
    let participants = info.get("participants").and_then(Value::as_array)?;
    let participant = participants
        .iter()
        .find(|participant| {
            participant
                .get("puuid")
                .and_then(Value::as_str)
                .is_some_and(|id| id == puuid)
        })
        .or_else(|| participants.first())?;
    let duration = info
        .get("gameDuration")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let cs = participant
        .get("totalMinionsKilled")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        + participant
            .get("neutralMinionsKilled")
            .and_then(Value::as_u64)
            .unwrap_or(0);
    let items = (0..7)
        .map(|index| {
            participant
                .get(format!("item{index}"))
                .and_then(Value::as_u64)
                .unwrap_or(0)
        })
        .collect::<Vec<_>>();
    Some(serde_json::json!({
        "championName": participant.get("championName").and_then(Value::as_str).unwrap_or("Champion"),
        "win": participant.get("win").and_then(Value::as_bool).unwrap_or(false),
        "kills": participant.get("kills").and_then(Value::as_u64).unwrap_or(0),
        "deaths": participant.get("deaths").and_then(Value::as_u64).unwrap_or(0),
        "assists": participant.get("assists").and_then(Value::as_u64).unwrap_or(0),
        "cs": cs,
        "csPerMin": cs as f64 / (duration as f64 / 60.0).max(1.0),
        "duration": duration,
        "gold": participant.get("goldEarned").and_then(Value::as_u64).unwrap_or(0),
        "damage": participant.get("totalDamageDealtToChampions").and_then(Value::as_u64).unwrap_or(0),
        "vision": participant.get("visionScore").and_then(Value::as_u64).unwrap_or(0),
        "gameMode": friendly_mode(info.get("gameMode").and_then(Value::as_str).unwrap_or("League"), info.get("queueId").and_then(Value::as_u64).unwrap_or(0)),
        "queueId": info.get("queueId").and_then(Value::as_u64).unwrap_or(0),
        "items": items
    }))
}

fn friendly_mode(game_mode: &str, queue_id: u64) -> String {
    match queue_id {
        420 => "Ranked Solo/Duo".to_string(),
        440 => "Ranked Flex".to_string(),
        450 => "ARAM".to_string(),
        400 => "Normal Draft".to_string(),
        430 => "Normal Blind".to_string(),
        1700 => "Arena".to_string(),
        _ => game_mode.replace('_', " "),
    }
}

struct DraftReadout {
    frontline: usize,
    magic: usize,
    physical: usize,
    utility: usize,
    advice: String,
}

fn draft_team_analysis(ids: &[u32; 5], champions: &[Champion]) -> DraftReadout {
    let mut result = DraftReadout {
        frontline: 0,
        magic: 0,
        physical: 0,
        utility: 0,
        advice: String::new(),
    };
    let mut filled = 0;
    for id in ids.iter().copied().filter(|id| *id != 0) {
        let Some(champion) = champions.iter().find(|champion| champion.id == id) else {
            continue;
        };
        filled += 1;
        let build = crate::builds::recommend(&champion.key, &champion.name, &champion.roles, None);
        let archetype = build.archetype.as_str();
        if ["TANK", "DIVER", "JUGGERNAUT", "ENGAGE"]
            .iter()
            .any(|tag| archetype.contains(tag))
        {
            result.frontline += 1;
        }
        if ["MAGE", "AP_", "ENCHANTER"]
            .iter()
            .any(|tag| archetype.contains(tag))
        {
            result.magic += 1;
        }
        if ["MARKSMAN", "AD_", "SKIRMISHER", "FIGHTER", "JUGGERNAUT"]
            .iter()
            .any(|tag| archetype.contains(tag))
        {
            result.physical += 1;
        }
        if ["SUPPORT", "ENCHANTER"]
            .iter()
            .any(|tag| archetype.contains(tag))
        {
            result.utility += 1;
        }
    }
    result.advice = if filled < 3 {
        "Add more picks to get a useful coverage readout.".to_string()
    } else if result.frontline == 0 {
        "Limited frontline: consider a champion who can start fights or absorb pressure."
            .to_string()
    } else if result.magic == 0 || result.physical == 0 {
        "Damage leans heavily one way; mixed damage can make itemization harder to answer."
            .to_string()
    } else if result.utility == 0 {
        "No support-oriented pick in the saved profiles; check for peel, shields or reliable setup."
            .to_string()
    } else {
        "Composition has frontline, mixed damage and utility coverage.".to_string()
    };
    result
}

/// The classic HUD's three objective rows (dragon pit, top pit, Baron), using the same
/// patch-accurate rules as the Path B HUD. Off Summoner's Rift every row reads "—".
fn objective_timers(live: &Value) -> [(String, String); 3] {
    use riftcore_native::domain::live_game::{LiveGameState, ObjectiveKind, ObjectiveTimer};
    let timers = LiveGameState::from_allgamedata(live)
        .map(|state| state.objective_timers())
        .unwrap_or_default();
    let find = |wanted: &[ObjectiveKind]| timers.iter().find(|t| wanted.contains(&t.kind));
    let row = |timer: Option<&ObjectiveTimer>, fallback: &str| {
        timer.map_or_else(
            || (fallback.to_string(), "\u{2014}".to_string()),
            |t| (t.label.clone(), t.formatted_remaining()),
        )
    };
    let baron = find(&[ObjectiveKind::Baron]);
    // After the grubs and the Herald, the top pit's next objective is Baron.
    let pit = find(&[ObjectiveKind::Voidgrubs, ObjectiveKind::RiftHerald]).or(baron);
    [
        row(
            find(&[ObjectiveKind::Dragon, ObjectiveKind::ElderDragon]),
            "DRAGON",
        ),
        row(pit, "GRUBS"),
        row(baron, "BARON"),
    ]
}

fn jungle_route_status(
    game_time: u64,
    start_buff: &str,
) -> Option<(usize, &'static str, u64, bool)> {
    let red_start = [
        (90, 110, "Red Buff"),
        (110, 135, "Krugs"),
        (135, 160, "Raptors"),
        (160, 180, "Wolves"),
        (180, 195, "Blue Buff"),
        (195, 210, "Gromp"),
        (210, 240, "Scuttle Crab"),
    ];
    let blue_start = [
        (90, 110, "Blue Buff"),
        (110, 135, "Gromp"),
        (135, 160, "Wolves"),
        (160, 180, "Raptors"),
        (180, 195, "Red Buff"),
        (195, 210, "Krugs"),
        (210, 240, "Scuttle Crab"),
    ];
    let route = if start_buff == "blue" {
        &blue_start
    } else {
        &red_start
    };
    for (index, (start, end, camp)) in route.iter().enumerate() {
        if game_time < *start {
            return Some((index + 1, camp, start - game_time, false));
        }
        if game_time < *end {
            return Some((index + 1, camp, end - game_time, true));
        }
    }
    None
}

/// The local player's entry in `allPlayers`. The Live Client API only reports team,
/// items and scores there, not under `activePlayer`.
fn active_player_entry(live: &Value) -> Option<&Value> {
    let players = live.get("allPlayers")?.as_array()?;
    players.get(active_player_index(live, players)?)
}

/// Scoreboard sections: the local player's team first, labelled as allies. Falls back
/// to neutral side names when the local player cannot be matched (e.g. spectating).
fn scoreboard_teams(live: &Value) -> [(&'static str, &'static str, egui::Color32); 2] {
    let own = active_player_entry(live).and_then(|player| player.get("team"));
    match own.and_then(Value::as_str) {
        Some(team) if team.eq_ignore_ascii_case("CHAOS") => {
            [("CHAOS", "YOUR TEAM", CYAN), ("ORDER", "ENEMY TEAM", RED)]
        }
        Some(_) => [("ORDER", "YOUR TEAM", CYAN), ("CHAOS", "ENEMY TEAM", RED)],
        None => [("ORDER", "BLUE TEAM", CYAN), ("CHAOS", "RED TEAM", RED)],
    }
}

/// Index of the local player in `allPlayers`.
fn active_player_index(live: &Value, players: &[Value]) -> Option<usize> {
    let active = live.get("activePlayer").unwrap_or(&Value::Null);
    let active_riot_id = active.get("riotId").and_then(Value::as_str);
    let active_name = active
        .get("summonerName")
        .or_else(|| active.get("riotIdGameName"))
        .and_then(Value::as_str);
    let active_champion = active.get("championName").and_then(Value::as_str);
    players.iter().position(|player| {
        active_riot_id
            .zip(player.get("riotId").and_then(Value::as_str))
            .is_some_and(|(active_id, player_id)| active_id == player_id)
            || active_name
                .zip(player.get("summonerName").and_then(Value::as_str))
                .is_some_and(|(active_name, player_name)| active_name == player_name)
            || (active_name.is_none()
                && active_riot_id.is_none()
                && active_champion
                    .zip(player.get("championName").and_then(Value::as_str))
                    .is_some_and(|(active_name, player_name)| active_name == player_name))
    })
}

fn allied_teammates(live: &Value) -> Vec<(String, String, String)> {
    let Some(players) = live.get("allPlayers").and_then(Value::as_array) else {
        return Vec::new();
    };
    let active_index = active_player_index(live, players);
    let Some(active_index) = active_index else {
        return Vec::new();
    };
    let Some(team) = players[active_index].get("team").and_then(Value::as_str) else {
        return Vec::new();
    };

    players
        .iter()
        .enumerate()
        .filter(|(index, player)| {
            *index != active_index && player.get("team").and_then(Value::as_str) == Some(team)
        })
        .take(4)
        .map(|(index, player)| {
            let role = player
                .get("position")
                .or_else(|| player.get("assignedPosition"))
                .and_then(Value::as_str)
                .filter(|role| !role.is_empty())
                .unwrap_or("ALLY")
                .to_ascii_uppercase();
            let champion = player
                .get("championName")
                .and_then(Value::as_str)
                .unwrap_or("Ally")
                .to_string();
            let key = player
                .get("riotId")
                .or_else(|| player.get("summonerName"))
                .and_then(Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| format!("{champion}-{index}"));
            (role, champion, format!("ally-ult-{key}"))
        })
        .collect()
}

fn render_manual_camp_timers(ui: &mut egui::Ui, timers: &mut HashMap<String, Instant>) {
    let camps = [
        ("BLUE BUFF", 300),
        ("GROMP", 135),
        ("WOLVES", 135),
        ("RAPTORS", 135),
        ("RED BUFF", 300),
        ("KRUGS", 135),
        ("SCUTTLE", 150),
    ];
    ui.label(
        RichText::new("Click a camp when it dies to start its respawn timer.")
            .size(9.5)
            .color(SECONDARY),
    );
    ui.add_space(5.0);
    for (camp, duration) in camps {
        let key = format!(
            "jungle-camp-{}",
            camp.to_ascii_lowercase().replace(' ', "-")
        );
        let remaining = timers
            .get(&key)
            .map(|ready_at| ready_at.saturating_duration_since(Instant::now()).as_secs())
            .unwrap_or(0);
        ui.horizontal(|ui| {
            ui.add_sized(
                [115.0, 31.0],
                egui::Label::new(RichText::new(camp).size(9.5).strong().color(TEXT)),
            );
            let label = if remaining > 0 {
                format!("RESPAWNS  {}  ·  RESET", format_hud_clock(remaining))
            } else {
                format!("START  {}", format_hud_clock(duration))
            };
            if ui
                .add_sized(
                    [240.0, 31.0],
                    egui::Button::new(
                        RichText::new(label)
                            .size(9.5)
                            .strong()
                            .color(if remaining > 0 { RED } else { GREEN }),
                    ),
                )
                .clicked()
            {
                if remaining > 0 {
                    timers.remove(&key);
                } else {
                    timers.insert(key, Instant::now() + Duration::from_secs(duration));
                }
            }
        });
    }
}

fn render_objective_timer_cards(ui: &mut egui::Ui, live: Option<&Value>) {
    let Some(live) = live else {
        ui.label(
            RichText::new("Waiting for live game data.")
                .size(10.0)
                .color(SECONDARY),
        );
        return;
    };
    for (label, timer) in objective_timers(live) {
        let ready = timer == "READY";
        let display_timer = if ready { "ALIVE" } else { timer.as_str() };
        Frame::new()
            .fill(Color32::from_rgba_unmultiplied(13, 20, 31, 230))
            .stroke(Stroke::new(1.0, if ready { GREEN } else { BORDER }))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(label).size(10.0).strong().color(GOLD));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(display_timer)
                                .size(13.0)
                                .monospace()
                                .strong()
                                .color(if ready { GREEN } else { TEXT }),
                        );
                    });
                });
            });
        ui.add_space(5.0);
    }
}

fn render_minimap_guide_pins(
    ctx: &egui::Context,
    area: egui::Rect,
    live: Option<&Value>,
    settings: &AppSettings,
    game_time: u64,
) {
    let Some(live) = live else { return };
    let game_mode = live
        .get("gameData")
        .and_then(|data| data.get("gameMode"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    if game_mode != "CLASSIC"
        || (!settings.hud_map_objective_pins
            && !settings.hud_map_route_pins
            && !settings.hud_map_ward_pins)
    {
        return;
    }

    // Same map scale as the Path B HUD; "Dock size" (zoom) sizes only the dock, not the map.
    let scale = riftcore_native::overlay::minimap::minimap_scale_factor(settings.hud_minimap_scale);
    let size = (area.height() * 0.25 * scale).clamp(150.0, area.height() * 0.48);
    let left = if settings.hud_minimap_flip {
        area.left() + 8.0 + settings.hud_minimap_nudge_x
    } else {
        area.right() - size - 8.0 + settings.hud_minimap_nudge_x
    };
    let bottom = area.bottom() - 8.0 + settings.hud_minimap_nudge_y;
    // Moving League's minimap to the left doesn't mirror the map, so the pins aren't mirrored.
    let map_point = |x: f32, y: f32| egui::pos2(left + size * x / 100.0, bottom - size * y / 100.0);
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::unique("riftcore-minimap-guide-pins"),
    ));

    let mut own_team = None;
    let active = live.get("activePlayer").unwrap_or(&Value::Null);
    let active_riot_id = active.get("riotId").and_then(Value::as_str);
    let active_name = active
        .get("summonerName")
        .or_else(|| active.get("riotIdGameName"))
        .and_then(Value::as_str);
    if let Some(players) = live.get("allPlayers").and_then(Value::as_array) {
        own_team = players.iter().find_map(|player| {
            let matches_active = active_riot_id
                .zip(player.get("riotId").and_then(Value::as_str))
                .is_some_and(|(active_id, player_id)| active_id == player_id)
                || active_name
                    .zip(player.get("summonerName").and_then(Value::as_str))
                    .is_some_and(|(active_name, player_name)| active_name == player_name);
            matches_active.then(|| {
                player
                    .get("team")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .eq_ignore_ascii_case("CHAOS")
            })
        });
    }
    let is_red_side = own_team.unwrap_or(false);

    if settings.hud_map_route_pins && settings.hud_jungle_pathing {
        let route: [(&str, f32, f32, u64, u64); 7] =
            match (is_red_side, settings.jungle_start_buff == "blue") {
                (true, false) => [
                    ("R", 48.0, 73.0, 90, 110),
                    ("K", 43.5, 82.0, 110, 135),
                    ("R", 53.0, 64.0, 135, 160),
                    ("W", 74.0, 56.5, 160, 180),
                    ("B", 74.5, 47.5, 180, 195),
                    ("G", 86.0, 44.0, 195, 210),
                    ("S", 29.5, 64.0, 210, 240),
                ],
                (true, true) => [
                    ("B", 74.5, 47.5, 90, 110),
                    ("G", 86.0, 44.0, 110, 135),
                    ("W", 74.0, 56.5, 135, 160),
                    ("R", 53.0, 64.0, 160, 180),
                    ("R", 48.0, 73.0, 180, 195),
                    ("K", 43.5, 82.0, 195, 210),
                    ("S", 29.5, 64.0, 210, 240),
                ],
                (false, false) => [
                    ("R", 52.0, 27.0, 90, 110),
                    ("K", 56.5, 18.0, 110, 135),
                    ("R", 47.0, 36.0, 135, 160),
                    ("W", 26.0, 43.5, 160, 180),
                    ("B", 25.5, 52.5, 180, 195),
                    ("G", 14.0, 56.0, 195, 210),
                    ("S", 70.0, 35.0, 210, 240),
                ],
                (false, true) => [
                    ("B", 25.5, 52.5, 90, 110),
                    ("G", 14.0, 56.0, 110, 135),
                    ("W", 26.0, 43.5, 135, 160),
                    ("R", 47.0, 36.0, 160, 180),
                    ("R", 52.0, 27.0, 180, 195),
                    ("K", 56.5, 18.0, 195, 210),
                    ("S", 70.0, 35.0, 210, 240),
                ],
            };
        let active_step =
            jungle_route_status(game_time, &settings.jungle_start_buff).map(|(step, _, _, _)| step);
        for (index, (label, x, y, _, _)) in route.iter().enumerate() {
            let is_active = active_step == Some(index + 1);
            let center = map_point(*x, *y);
            let fill = if is_active {
                CYAN
            } else {
                Color32::from_rgba_unmultiplied(7, 16, 27, 228)
            };
            painter.circle_filled(center, 8.5, fill);
            painter.circle_stroke(
                center,
                8.5,
                Stroke::new(1.3, if is_active { TEXT } else { CYAN }),
            );
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                format!("{}{}", index + 1, label),
                FontId::proportional(7.5),
                if is_active { BG } else { TEXT },
            );
        }
    }

    if settings.hud_map_ward_pins {
        for (label, x, y) in [
            ("W", 62.0, 40.0),
            ("W", 38.0, 60.0),
            ("W", 80.0, 75.0),
            ("W", 20.0, 25.0),
            ("W", 64.0, 84.0),
            ("W", 36.0, 16.0),
        ] {
            let center = map_point(x, y);
            painter.circle_filled(center, 7.0, Color32::from_rgb(8, 85, 62));
            painter.circle_stroke(center, 7.0, Stroke::new(1.0, GREEN));
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                label,
                FontId::proportional(8.0),
                TEXT,
            );
        }
    }

    if settings.hud_map_objective_pins {
        let timers = objective_timers(live);
        let pit_label = match timers[1].0.as_str() {
            "HERALD" => "H",
            "BARON" => "B",
            _ => "G",
        };
        for (label, x, y, timer) in [
            ("D", 65.5, 29.5, timers[0].1.as_str()),
            (pit_label, 34.0, 69.5, timers[1].1.as_str()),
        ] {
            let center = map_point(x, y);
            painter.circle_stroke(center, 12.0, Stroke::new(1.6, GOLD));
            painter.circle_filled(center, 2.5, GOLD);
            let label_center = center + Vec2::new(31.0, 20.0);
            let marker = egui::Rect::from_center_size(label_center, Vec2::new(50.0, 17.0));
            painter.rect_filled(
                marker,
                egui::CornerRadius::same(7),
                Color32::from_rgba_unmultiplied(6, 13, 24, 235),
            );
            painter.rect_stroke(
                marker,
                egui::CornerRadius::same(7),
                Stroke::new(1.0, GOLD),
                egui::StrokeKind::Inside,
            );
            painter.text(
                label_center,
                egui::Align2::CENTER_CENTER,
                format!("{label} {timer}"),
                FontId::proportional(8.0),
                TEXT,
            );
        }
    }
}

fn format_hud_clock(seconds: u64) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

fn session_picks(session: &Value, team_key: &str) -> [u32; 5] {
    let mut picks = [0; 5];
    if let Some(members) = session.get(team_key).and_then(Value::as_array) {
        for (index, member) in members.iter().take(5).enumerate() {
            let champion_id = member
                .get("championId")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let champion_id = if champion_id == 0 {
                member
                    .get("championPickIntent")
                    .and_then(Value::as_u64)
                    .unwrap_or(0)
            } else {
                champion_id
            } as u32;
            picks[index] = champion_id;
        }
    }
    picks
}

fn champ_select_local_member(session: &Value) -> Option<&Value> {
    let local_cell = session.get("localPlayerCellId").and_then(Value::as_u64)?;
    session
        .get("myTeam")?
        .as_array()?
        .iter()
        .find(|member| member.get("cellId").and_then(Value::as_u64) == Some(local_cell))
}

/// The local player's champion once it is locked in: their pick action is completed,
/// or, in modes without pick actions (ARAM), the champion they were given.
fn champ_select_locked_champion(session: &Value) -> Option<u32> {
    let local_cell = session.get("localPlayerCellId").and_then(Value::as_u64)?;
    let picks = session
        .get("actions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|group| {
            group
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or(std::slice::from_ref(group))
        })
        .filter(|action| {
            action.get("actorCellId").and_then(Value::as_u64) == Some(local_cell)
                && action.get("type").and_then(Value::as_str) == Some("pick")
        })
        .collect::<Vec<_>>();
    let champion = if picks.is_empty() {
        champ_select_local_member(session)?.get("championId")
    } else {
        picks
            .iter()
            .find(|action| action.get("completed").and_then(Value::as_bool) == Some(true))?
            .get("championId")
    };
    champion
        .and_then(Value::as_u64)
        .filter(|id| *id > 0)
        .map(|id| id as u32)
}

/// Maps the local player's champ select `assignedPosition` to a build role.
fn champ_select_assigned_role(session: &Value) -> Option<&'static str> {
    let position = champ_select_local_member(session)?
        .get("assignedPosition")?
        .as_str()?;
    match position.to_ascii_lowercase().as_str() {
        "top" => Some("TOP"),
        "jungle" => Some("JUNGLE"),
        "middle" | "mid" => Some("MID"),
        "bottom" | "adc" => Some("ADC"),
        "utility" | "support" => Some("SUPPORT"),
        _ => None,
    }
}

fn champ_select_local_champion(session: &Value) -> Option<u32> {
    let local_cell = session.get("localPlayerCellId").and_then(Value::as_u64)?;
    let selected_from_member = session
        .get("myTeam")
        .and_then(Value::as_array)
        .and_then(|team| {
            team.iter()
                .find(|member| member.get("cellId").and_then(Value::as_u64) == Some(local_cell))
        })
        .and_then(|member| {
            member
                .get("championId")
                .and_then(Value::as_u64)
                .filter(|champion_id| *champion_id > 0)
                .or_else(|| {
                    member
                        .get("championPickIntent")
                        .and_then(Value::as_u64)
                        .filter(|champion_id| *champion_id > 0)
                })
        });
    if selected_from_member.is_some() {
        return selected_from_member.map(|champion_id| champion_id as u32);
    }
    session
        .get("actions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|group| {
            group
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or(std::slice::from_ref(group))
        })
        .find(|action| {
            action.get("actorCellId").and_then(Value::as_u64) == Some(local_cell)
                && action.get("type").and_then(Value::as_str) == Some("pick")
        })
        .and_then(|action| action.get("championId"))
        .and_then(Value::as_u64)
        .filter(|champion_id| *champion_id > 0)
        .map(|champion_id| champion_id as u32)
}

fn render_draft_team(
    ui: &mut egui::Ui,
    title: &str,
    description: &str,
    picks: &mut [u32; 5],
    champions: &[Champion],
    textures: &std::collections::HashMap<String, egui::TextureHandle>,
    accent: Color32,
) {
    panel(ui, |ui| {
        panel_title(ui, title, description);
        for (index, role) in ["TOP", "JUNGLE", "MID", "ADC", "SUPPORT"]
            .iter()
            .enumerate()
        {
            let picked_champ = champions
                .iter()
                .find(|champion| champion.id == picks[index]);
            let (role_icon, role_color) = match *role {
                "TOP" => ("⚔ TOP", Color32::from_rgb(198, 120, 221)),
                "JUNGLE" => ("🌿 JNG", Color32::from_rgb(97, 175, 239)),
                "MID" => ("⚡ MID", Color32::from_rgb(229, 192, 123)),
                "ADC" => ("🏹 BOT", Color32::from_rgb(224, 108, 117)),
                _ => ("🛡 SUP", Color32::from_rgb(152, 195, 121)),
            };

            Frame::new()
                .fill(INSET)
                .stroke(Stroke::new(
                    1.0,
                    if picked_champ.is_some() {
                        CYAN_DIM
                    } else {
                        RAISED
                    },
                ))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        badge(ui, role_icon, role_color);
                        ui.add_space(4.0);
                        if let Some(champ) = picked_champ {
                            avatar(ui, &champ.name, 32.0, accent, textures.get(&champ.key));
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(&champ.name).size(12.5).strong().color(TEXT),
                                );
                                ui.label(
                                    RichText::new(champ.roles.join(" / "))
                                        .size(9.0)
                                        .color(SECONDARY),
                                );
                            });
                        } else {
                            let (placeholder_rect, _) =
                                ui.allocate_exact_size(Vec2::splat(32.0), Sense::hover());
                            ui.painter().rect_stroke(
                                placeholder_rect,
                                egui::CornerRadius::same(6),
                                Stroke::new(1.0, BORDER),
                                egui::StrokeKind::Inside,
                            );
                            ui.painter().text(
                                placeholder_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "+",
                                FontId::proportional(16.0),
                                MUTED,
                            );
                            ui.label(
                                RichText::new("Empty slot")
                                    .size(12.0)
                                    .italics()
                                    .color(MUTED),
                            );
                        }

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if picked_champ.is_some()
                                && action_button(ui, "×", false, RED).clicked()
                            {
                                picks[index] = 0;
                            }
                            let selected_text =
                                picked_champ.map(|c| c.name.as_str()).unwrap_or("Pick...");
                            egui::ComboBox::from_id_salt(format!("{title}-{role}-{index}"))
                                .selected_text(selected_text)
                                .width(120.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut picks[index], 0, "Empty slot");
                                    for champion in champions {
                                        ui.selectable_value(
                                            &mut picks[index],
                                            champion.id,
                                            champion.name.as_str(),
                                        );
                                    }
                                });
                        });
                    });
                });
            ui.add_space(5.0);
        }
    });
}

fn role_matches(champion_role: &str, filter: &str) -> bool {
    let normalized = champion_role.to_ascii_uppercase();
    match filter {
        "MID" => normalized == "MID" || normalized == "MIDDLE",
        "ADC" => normalized == "ADC" || normalized == "BOTTOM" || normalized == "BOT",
        "SUPPORT" => normalized == "SUPPORT" || normalized == "UTILITY",
        _ => normalized == filter,
    }
}

const RUNE_TREES: &[(u32, &str)] = &[
    (8000, "Precision"),
    (8100, "Domination"),
    (8200, "Sorcery"),
    (8400, "Resolve"),
    (8300, "Inspiration"),
];

const SUMMONER_SPELLS: &[(i64, &str)] = &[
    (4, "Flash"),
    (12, "Teleport"),
    (14, "Ignite"),
    (11, "Smite"),
    (32, "Mark / Snowball"),
    (6, "Ghost"),
    (7, "Heal"),
    (3, "Exhaust"),
    (21, "Barrier"),
    (1, "Cleanse"),
];

fn tree_name(id: u32) -> &'static str {
    RUNE_TREES
        .iter()
        .find(|(tree_id, _)| *tree_id == id)
        .map(|(_, name)| *name)
        .unwrap_or("Precision")
}

fn spell_name(id: i64) -> &'static str {
    SUMMONER_SPELLS
        .iter()
        .find(|(spell_id, _)| *spell_id == id)
        .map(|(_, name)| *name)
        .unwrap_or("Spell")
}

fn rune_options(tree: u32, row: usize) -> Vec<(i64, String)> {
    RUNE_CATALOG
        .as_array()
        .and_then(|trees| {
            trees
                .iter()
                .find(|entry| entry.get("id").and_then(Value::as_u64) == Some(tree as u64))
        })
        .and_then(|entry| entry.get("slots"))
        .and_then(Value::as_array)
        .and_then(|slots| slots.get(row))
        .and_then(|slot| slot.get("runes"))
        .and_then(Value::as_array)
        .map(|runes| {
            runes
                .iter()
                .filter_map(|rune| {
                    Some((
                        rune.get("id")?.as_u64()? as i64,
                        rune.get("name")?.as_str()?.to_owned(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn selected_rune_name(tree: u32, row: usize, selected: i64) -> String {
    selected_rune_name_from_options(&rune_options(tree, row), selected)
}

fn selected_rune_name_from_options(options: &[(i64, String)], selected: i64) -> String {
    options
        .iter()
        .find(|(id, _)| *id == selected)
        .map(|(_, name)| name.clone())
        .unwrap_or_else(|| "Select rune".to_string())
}

fn rune_tree_color(tree: u32) -> Color32 {
    match tree {
        8100 => Color32::from_rgb(238, 105, 113),
        8000 => Color32::from_rgb(231, 189, 105),
        8200 => Color32::from_rgb(111, 181, 246),
        8400 => Color32::from_rgb(108, 205, 151),
        8300 => Color32::from_rgb(195, 143, 236),
        _ => CYAN,
    }
}

const RUNE_ORB_DARK: Color32 = Color32::from_rgb(11, 13, 18);

/// Result of drawing one row of rune orbs.
struct RuneRowResult {
    clicked: Option<i64>,
    selected_center: Option<egui::Pos2>,
}

fn rune_tree_picker(
    ui: &mut egui::Ui,
    selected: u32,
    excluded: Option<u32>,
    textures: &HashMap<i64, egui::TextureHandle>,
) -> Option<u32> {
    let mut clicked = None;
    let count = RUNE_TREES
        .iter()
        .filter(|(id, _)| Some(*id) != excluded)
        .count()
        .max(1) as f32;
    // Fit all trees in the card: each orb takes side + 10 (glow padding) + 2 spacing.
    let side = ((ui.available_width() + 2.0) / count - 12.0).clamp(20.0, 32.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for (id, name) in RUNE_TREES
            .iter()
            .copied()
            .filter(|(id, _)| Some(*id) != excluded)
        {
            let response = rune_icon_card(
                ui,
                id as i64,
                selected == id,
                rune_tree_color(id),
                side,
                textures,
            );
            if response.clicked() {
                clicked = Some(id);
            }
            response.on_hover_text(name);
        }
    });
    clicked
}

/// Small uppercase label on the rail line, with the chosen value underneath in the accent colour.
fn rune_section_heading(ui: &mut egui::Ui, title: &str, detail: &str, accent: Color32) {
    // Painted at a fixed height so the app's large default item spacing can't pull the two lines apart.
    let has_detail = !detail.is_empty();
    let height = if has_detail { 31.0 } else { 14.0 };
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    let painter = ui.painter();
    let dot = egui::pos2(rect.left() + 4.0, rect.top() + 6.0);
    painter.circle_filled(dot, 5.5, accent.gamma_multiply(0.2));
    painter.circle_filled(dot, 3.0, accent);
    painter.text(
        egui::pos2(rect.left() + 14.0, dot.y),
        egui::Align2::LEFT_CENTER,
        title,
        FontId::proportional(8.5),
        MUTED,
    );
    if has_detail {
        painter.text(
            egui::pos2(rect.left() + 14.0, rect.top() + 22.0),
            egui::Align2::LEFT_CENTER,
            detail,
            FontId::proportional(12.5),
            lerp_color(accent, TEXT, 0.4),
        );
    }
}

/// A row of round rune orbs. Unselected orbs are dimmed so the chosen path stands out.
fn rune_icon_row(
    ui: &mut egui::Ui,
    options: &[(i64, String)],
    selected: i64,
    textures: &HashMap<i64, egui::TextureHandle>,
    accent: Color32,
    side: f32,
    lead: f32,
) -> RuneRowResult {
    let mut result = RuneRowResult {
        clicked: None,
        selected_center: None,
    };
    if options.is_empty() {
        return result;
    }
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        if lead > 0.0 {
            ui.add_space(lead);
        }
        for (id, name) in options {
            let is_selected = selected == *id;
            let response = rune_icon_card(ui, *id, is_selected, accent, side, textures);
            if is_selected {
                result.selected_center = Some(response.rect.center());
            }
            if response.clicked() {
                result.clicked = Some(*id);
            }
            let description = rune_description(*id);
            response.on_hover_text(if description.is_empty() {
                name.clone()
            } else {
                format!("{name}\n{description}")
            });
        }
    });
    result
}

/// Round rune orb: soft halo + double ring when selected, dimmed when not, lifts on hover.
fn rune_icon_card(
    ui: &mut egui::Ui,
    id: i64,
    selected: bool,
    accent: Color32,
    side: f32,
    textures: &HashMap<i64, egui::TextureHandle>,
) -> egui::Response {
    let pad = 5.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side + pad * 2.0), Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let hover =
        ui.ctx()
            .animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.12);
    let active = ui
        .ctx()
        .animate_bool_with_time(response.id.with("selected"), selected, 0.18);
    let emphasis = active.max(hover * 0.75);
    let center = rect.center();
    let radius = side * 0.5 * (1.0 + 0.07 * hover);
    let painter = ui.painter();

    if active > 0.01 {
        for step in 0..5 {
            let fade = 1.0 - step as f32 / 5.0;
            painter.circle_filled(
                center,
                radius + 1.0 + step as f32 * 1.2,
                accent.gamma_multiply(0.11 * active * fade),
            );
        }
    }
    painter.circle_filled(
        center,
        radius,
        lerp_color(RUNE_ORB_DARK, accent, 0.06 + 0.16 * emphasis),
    );
    if let Some(texture) = textures.get(&id) {
        let image_radius = radius - 3.0;
        let image_rect = egui::Rect::from_center_size(center, Vec2::splat(image_radius * 2.0));
        let alpha = 0.36 + 0.64 * emphasis;
        egui::Image::new((texture.id(), image_rect.size()))
            .corner_radius(egui::CornerRadius::same(image_radius.round() as u8))
            .tint(Color32::from_white_alpha((alpha * 255.0) as u8))
            .paint_at(ui, image_rect);
    }
    let painter = ui.painter();
    painter.circle_stroke(
        center,
        radius,
        Stroke::new(
            0.8 + active * 1.0,
            lerp_color(BORDER, accent, emphasis * (0.55 + 0.45 * active)),
        ),
    );
    if side >= 38.0 && active > 0.01 {
        painter.circle_stroke(
            center,
            radius + 3.5,
            Stroke::new(1.0, accent.gamma_multiply(0.5 * active)),
        );
    }
    response
}

/// Card with a tree-tinted vertical gradient and a faint tree emblem in the corner.
fn rune_card<R>(
    ui: &mut egui::Ui,
    accent: Color32,
    emblem: Option<&egui::TextureHandle>,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let background = ui.painter().add(egui::Shape::Noop);
    let inner = Frame::new()
        .stroke(Stroke::new(1.0, lerp_color(BORDER, accent, 0.5)))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(12))
        .show(ui, add_contents);
    let rect = inner.response.rect;
    let mut shapes = vec![rounded_gradient_mesh(
        rect,
        12.0,
        lerp_color(INSET, accent, 0.14),
        INSET,
    )];
    if let Some(texture) = emblem {
        let size = 78.0;
        let emblem_rect = egui::Rect::from_min_size(
            egui::pos2(rect.right() - size - 4.0, rect.top() + 4.0),
            Vec2::splat(size),
        );
        shapes.push(egui::Shape::image(
            texture.id(),
            emblem_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::from_white_alpha(20),
        ));
    }
    ui.painter().set(background, egui::Shape::Vec(shapes));
    inner.inner
}

/// Rounded rect filled with a top-to-bottom gradient (fan mesh; colour is linear in y so it's exact).
fn rounded_gradient_mesh(
    rect: egui::Rect,
    radius: f32,
    top: Color32,
    bottom: Color32,
) -> egui::Shape {
    let radius = radius.min(rect.width() * 0.5).min(rect.height() * 0.5);
    let corners = [
        (
            egui::pos2(rect.right() - radius, rect.top() + radius),
            -90.0_f32,
        ),
        (
            egui::pos2(rect.right() - radius, rect.bottom() - radius),
            0.0,
        ),
        (
            egui::pos2(rect.left() + radius, rect.bottom() - radius),
            90.0,
        ),
        (egui::pos2(rect.left() + radius, rect.top() + radius), 180.0),
    ];
    let color_at = |point: egui::Pos2| {
        lerp_color(
            top,
            bottom,
            ((point.y - rect.top()) / rect.height().max(1.0)).clamp(0.0, 1.0),
        )
    };
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.center(), color_at(rect.center()));
    for (corner, start) in corners {
        for step in 0..=6 {
            let angle = (start + 90.0 * step as f32 / 6.0).to_radians();
            let point = corner + Vec2::new(angle.cos(), angle.sin()) * radius;
            mesh.colored_vertex(point, color_at(point));
        }
    }
    let count = mesh.vertices.len() as u32 - 1;
    for index in 0..count {
        mesh.add_triangle(0, 1 + index, 1 + (index + 1) % count);
    }
    egui::Shape::mesh(mesh)
}

/// The "rune path": a vertical rail through the heading dots with a branch to each chosen rune.
fn paint_rune_rail(
    ui: &egui::Ui,
    slot: egui::layers::ShapeIdx,
    rail_x: f32,
    top_y: f32,
    nodes: &[egui::Pos2],
    accent: Color32,
) {
    let Some(last) = nodes.last() else {
        return;
    };
    let mut shapes = vec![egui::Shape::line_segment(
        [egui::pos2(rail_x, top_y), egui::pos2(rail_x, last.y)],
        Stroke::new(2.0, accent.gamma_multiply(0.35)),
    )];
    for node in nodes {
        shapes.push(egui::Shape::line_segment(
            [egui::pos2(rail_x, node.y), *node],
            Stroke::new(1.5, accent.gamma_multiply(0.28)),
        ));
        shapes.push(egui::Shape::circle_filled(
            egui::pos2(rail_x, node.y),
            3.5,
            accent,
        ));
    }
    ui.painter().set(slot, egui::Shape::Vec(shapes));
}

/// Width of a key's summoner spell dropdown, in points.
const SPELL_MENU_WIDTH: f32 = 184.0;

/// The summoner spell keys and their accents, in slot order (0 = D, 1 = F).
const SPELL_KEYS: [(&str, Color32); 2] = [("D", CYAN), ("F", GOLD)];

/// Stable id of a key card (`slot` 0 = D, 1 = F). Its dropdown and rows derive from it.
fn spell_slot_id(ui: &egui::Ui, slot: usize) -> egui::Id {
    ui.make_persistent_id(("summoner-spell-slot", slot))
}

/// Puts `spell` on key `slot` (0 = D, 1 = F). Picking the spell the other key holds swaps
/// the two keys, so one spell never sits on both.
fn assign_spell(spell_one: &mut i64, spell_two: &mut i64, slot: usize, spell: i64) {
    let (target, other) = if slot == 0 {
        (spell_one, spell_two)
    } else {
        (spell_two, spell_one)
    };
    if *other == spell {
        std::mem::swap(target, other);
    } else {
        *target = spell;
    }
}

/// The D and F key cards. Each opens a dropdown of the summoner spells.
fn summoner_spell_dropdowns(
    ui: &mut egui::Ui,
    spell_one: &mut i64,
    spell_two: &mut i64,
    textures: &HashMap<i64, egui::TextureHandle>,
) {
    let slot_width = ((ui.available_width() - 6.0) * 0.5).clamp(84.0, 122.0);
    let card_ids = [0, 1].map(|slot| spell_slot_id(ui, slot));
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        for (slot, &(key, accent)) in SPELL_KEYS.iter().enumerate() {
            let (current, other) = if slot == 0 {
                (*spell_one, *spell_two)
            } else {
                (*spell_two, *spell_one)
            };
            let other_key = SPELL_KEYS[1 - slot];
            let card = spell_slot_button(
                ui,
                card_ids[slot],
                key,
                spell_name(current),
                textures.get(&current),
                accent,
                slot_width,
            );
            egui::Popup::from_toggle_button_response(&card)
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .width(SPELL_MENU_WIDTH)
                .gap(4.0)
                .frame(
                    Frame::new()
                        .fill(CARD)
                        .stroke(Stroke::new(1.0, lerp_color(BORDER, accent, 0.45)))
                        .corner_radius(egui::CornerRadius::same(10))
                        .inner_margin(egui::Margin::same(5))
                        .shadow(ui.style().visuals.popup_shadow),
                )
                .show(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    for &(id, name) in SUMMONER_SPELLS {
                        let row = spell_menu_row(
                            ui,
                            card.id.with(("spell", id)),
                            name,
                            textures.get(&id),
                            id == current,
                            (id == other).then_some(other_key),
                            accent,
                        );
                        if row.clicked() {
                            assign_spell(spell_one, spell_two, slot, id);
                            ui.close();
                        }
                    }
                });
        }
    });
}

/// One row of a key's summoner spell dropdown: icon, name, then a check on the key's
/// current spell or an "on D" / "on F" tag on the spell the other key holds.
fn spell_menu_row(
    ui: &mut egui::Ui,
    id: egui::Id,
    name: &str,
    icon: Option<&egui::TextureHandle>,
    current: bool,
    other_key: Option<(&str, Color32)>,
    accent: Color32,
) -> egui::Response {
    let (_, rect) = ui.allocate_space(Vec2::new(ui.available_width(), 28.0));
    let response = ui
        .interact(rect, id, Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    let hover = ui
        .ctx()
        .animate_bool_with_time(id.with("hover"), response.hovered(), 0.1);
    let emphasis = if current { 1.0 } else { hover * 0.6 };
    if emphasis > 0.0 {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(6),
            lerp_color(CARD, accent, 0.16 * emphasis),
        );
    }
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 16.0, rect.center().y),
        Vec2::splat(20.0),
    );
    if let Some(texture) = icon {
        egui::Image::new((texture.id(), icon_rect.size()))
            .corner_radius(egui::CornerRadius::same(4))
            .paint_at(ui, icon_rect);
    } else {
        ui.painter()
            .rect_filled(icon_rect, egui::CornerRadius::same(4), INSET);
    }
    let painter = ui.painter();
    let mut text_right = rect.right() - 8.0;
    if current {
        let check = egui::pos2(rect.right() - 14.0, rect.center().y);
        painter.add(egui::Shape::line(
            vec![
                check + Vec2::new(-4.0, 0.0),
                check + Vec2::new(-1.0, 3.0),
                check + Vec2::new(4.5, -3.5),
            ],
            Stroke::new(1.6, accent),
        ));
        text_right = check.x - 10.0;
    } else if let Some((key, key_accent)) = other_key {
        let tag =
            painter.layout_no_wrap(format!("on {key}"), FontId::proportional(8.5), key_accent);
        let tag_rect = egui::Rect::from_min_size(
            egui::pos2(rect.right() - 18.0 - tag.size().x, rect.center().y - 8.0),
            Vec2::new(tag.size().x + 10.0, 16.0),
        );
        painter.rect_filled(
            tag_rect,
            egui::CornerRadius::same(8),
            lerp_color(CARD, key_accent, 0.18),
        );
        painter.galley(tag_rect.center() - tag.size() * 0.5, tag, key_accent);
        text_right = tag_rect.left() - 6.0;
    }
    let text_left = icon_rect.right() + 8.0;
    paint_truncated_text(
        painter,
        egui::pos2(text_left, rect.center().y),
        name,
        FontId::proportional(11.0),
        lerp_color(SECONDARY, TEXT, if current { 1.0 } else { hover }),
        text_right - text_left,
    );
    match other_key {
        Some((key, _)) if !current => {
            response.on_hover_text(format!("{name} is on {key}; picking it swaps the two keys"))
        }
        _ => response,
    }
}

/// Paints one line of `text`, left-aligned and vertically centred on `left_center`, and cuts
/// it with "…" when it is wider than `max_width`.
fn paint_truncated_text(
    painter: &egui::Painter,
    left_center: egui::Pos2,
    text: &str,
    font: FontId,
    color: Color32,
    max_width: f32,
) {
    let mut job = egui::text::LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap = egui::text::TextWrapping::truncate_at_width(max_width.max(1.0));
    let galley = painter.layout_job(job);
    let pos = egui::pos2(left_center.x, left_center.y - galley.size().y * 0.5);
    painter.galley(pos, galley, color);
}

/// Big D / F key card showing the assigned spell. Clicking it opens the key's dropdown; the
/// chevron flips and the card lights up while that dropdown is open.
fn spell_slot_button(
    ui: &mut egui::Ui,
    id: egui::Id,
    key: &str,
    name: &str,
    icon: Option<&egui::TextureHandle>,
    accent: Color32,
    width: f32,
) -> egui::Response {
    let (_, rect) = ui.allocate_space(Vec2::new(width, 42.0));
    let response = ui
        .interact(rect, id, Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    let open = egui::Popup::is_id_open(ui.ctx(), egui::Popup::default_response_id(&response));
    let hover = ui
        .ctx()
        .animate_bool_with_time(id.with("hover"), response.hovered(), 0.12);
    let on = ui.ctx().animate_bool_with_time(id.with("open"), open, 0.16);
    let emphasis = on.max(hover * 0.5);
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        egui::CornerRadius::same(9),
        lerp_color(CARD, accent, 0.04 + 0.12 * emphasis),
    );
    painter.rect_stroke(
        rect,
        egui::CornerRadius::same(9),
        Stroke::new(1.0 + on * 0.6, lerp_color(BORDER, accent, emphasis)),
        egui::StrokeKind::Inside,
    );
    let (icon_rect, text_rect) = spell_slot_layout(rect);
    if let Some(texture) = icon {
        egui::Image::new((texture.id(), icon_rect.size()))
            .corner_radius(egui::CornerRadius::same(6))
            .paint_at(ui, icon_rect);
    } else {
        ui.painter()
            .rect_filled(icon_rect, egui::CornerRadius::same(6), INSET);
    }
    let painter = ui.painter();
    painter.rect_stroke(
        icon_rect,
        egui::CornerRadius::same(6),
        Stroke::new(1.0, lerp_color(BORDER, accent, 0.4 + 0.6 * on)),
        egui::StrokeKind::Outside,
    );
    let key_y = rect.center().y - 7.0;
    painter.text(
        egui::pos2(text_rect.left(), key_y),
        egui::Align2::LEFT_CENTER,
        key,
        FontId::proportional(8.5),
        accent,
    );
    // Chevron on the key row, so the name below gets the card's full width. It points down
    // while closed and flips up as the dropdown opens.
    let chevron = egui::pos2(rect.right() - 11.0, key_y);
    let tip = 2.5 * (1.0 - 2.0 * on);
    painter.add(egui::Shape::line(
        vec![
            chevron + Vec2::new(-4.0, -tip),
            chevron + Vec2::new(0.0, tip),
            chevron + Vec2::new(4.0, -tip),
        ],
        Stroke::new(1.4, lerp_color(MUTED, accent, emphasis)),
    ));
    paint_truncated_text(
        painter,
        egui::pos2(text_rect.left(), rect.center().y + 6.0),
        name,
        FontId::proportional(11.5),
        TEXT,
        text_rect.width(),
    );
    response
}

/// Where a key card of `card` size puts its icon and its text. Cards narrower than 110 pt (the
/// default window gives ~90 pt) shrink the icon from 30 to 24 pt so names like "Teleport" fit.
fn spell_slot_layout(card: egui::Rect) -> (egui::Rect, egui::Rect) {
    let icon_size = (24.0 + (card.width() - 90.0) * 0.3).clamp(24.0, 30.0);
    let icon = egui::Rect::from_center_size(
        egui::pos2(card.left() + 7.0 + icon_size * 0.5, card.center().y),
        Vec2::splat(icon_size),
    );
    let text = egui::Rect::from_min_max(
        egui::pos2(icon.right() + 7.0, card.top()),
        egui::pos2(card.right() - 5.0, card.bottom()),
    );
    (icon, text)
}

fn rune_description(id: i64) -> String {
    let Some(trees) = RUNE_CATALOG.as_array() else {
        return String::new();
    };
    let Some(description) = trees.iter().find_map(|tree| {
        tree.get("slots")
            .and_then(Value::as_array)?
            .iter()
            .flat_map(|slot| {
                slot.get("runes")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
            })
            .find(|rune| rune.get("id").and_then(Value::as_i64) == Some(id))
            .and_then(|rune| rune.get("shortDesc").and_then(Value::as_str))
    }) else {
        return String::new();
    };
    let mut clean = String::new();
    let mut in_tag = false;
    for ch in description.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => clean.push(ch),
            _ => {}
        }
    }
    let clean = clean
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    truncate_rune_label(&clean, 180)
}

fn truncate_rune_label(text: &str, max_chars: usize) -> String {
    let max_chars = max_chars.max(4);
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    text.chars()
        .take(max_chars.saturating_sub(1))
        .chain(std::iter::once('…'))
        .collect()
}

fn normalize_rune_setup(
    primary_tree: u32,
    secondary_tree: u32,
    secondary_rows: &mut [usize; 2],
    perks: &mut [i64; 9],
) {
    for (slot, row) in [(0usize, 0usize), (1, 1), (2, 2), (3, 3)] {
        let options = rune_options(primary_tree, row);
        if !options.iter().any(|(id, _)| *id == perks[slot]) {
            perks[slot] = options.first().map(|(id, _)| *id).unwrap_or(5008);
        }
    }
    secondary_rows[0] = secondary_rows[0].clamp(1, 3);
    secondary_rows[1] = secondary_rows[1].clamp(1, 3);
    if secondary_rows[0] == secondary_rows[1] {
        secondary_rows[1] = (1..=3).find(|row| *row != secondary_rows[0]).unwrap_or(2);
    }
    for (choice, &row) in secondary_rows.iter().enumerate() {
        let options = rune_options(secondary_tree, row);
        let slot = 4 + choice;
        if !options.iter().any(|(id, _)| *id == perks[slot]) {
            perks[slot] = options.first().map(|(id, _)| *id).unwrap_or(9111);
        }
    }
    for row in 0..3 {
        let slot = 6 + row;
        let options = stat_shard_options(row);
        if !options.iter().any(|(id, _)| *id == perks[slot]) {
            perks[slot] = options.first().map(|(id, _)| *id).unwrap_or(5008);
        }
    }
}

fn rune_id_preferred(tree: u32, row: usize, preferred_names: &[&str]) -> i64 {
    let options = rune_options(tree, row);
    preferred_names
        .iter()
        .find_map(|preferred| {
            options
                .iter()
                .find(|(_, name)| name.to_lowercase().contains(&preferred.to_lowercase()))
                .map(|(id, _)| *id)
        })
        .or_else(|| options.first().map(|(id, _)| *id))
        .unwrap_or(5008)
}

fn rune_row(tree: u32, rune_id: i64) -> Option<usize> {
    RUNE_CATALOG
        .as_array()?
        .iter()
        .find(|entry| entry.get("id").and_then(Value::as_u64) == Some(tree as u64))?
        .get("slots")?
        .as_array()?
        .iter()
        .position(|slot| {
            slot.get("runes")
                .and_then(Value::as_array)
                .is_some_and(|runes| {
                    runes
                        .iter()
                        .any(|rune| rune.get("id").and_then(Value::as_i64) == Some(rune_id))
                })
        })
}

fn secondary_rows_for_perks(tree: u32, first: i64, second: i64) -> [usize; 2] {
    let first_row = rune_row(tree, first).filter(|row| *row > 0).unwrap_or(1);
    let second_row = rune_row(tree, second)
        .filter(|row| *row > 0 && *row != first_row)
        .unwrap_or_else(|| (1..=3).find(|row| *row != first_row).unwrap_or(2));
    [first_row, second_row]
}

fn stat_shard_options(row: usize) -> &'static [(i64, &'static str)] {
    match row {
        0 => &[
            (5008, "Adaptive Force"),
            (5005, "Attack Speed"),
            (5007, "Ability Haste"),
        ],
        1 => &[
            (5008, "Adaptive Force"),
            (5010, "Move Speed"),
            (5001, "Scaling Health"),
        ],
        _ => &[
            (5011, "Flat Health"),
            (5013, "Tenacity / Slow Resist"),
            (5001, "Scaling Health"),
        ],
    }
}

fn rune_page(champion: &str, primary: u32, secondary: u32, perks: &[i64; 9]) -> Value {
    json!({
        "name": format!("RiftCore {champion}"),
        "primaryStyleId": primary,
        "subStyleId": secondary,
        "selectedPerkIds": perks,
        "current": true
    })
}

fn preferred_spell_order(first: i64, second: i64, flash_slot: &str) -> (i64, i64) {
    if first == 4 || second == 4 {
        let other = if first == 4 { second } else { first };
        if flash_slot.eq_ignore_ascii_case("D") {
            (4, other)
        } else {
            (other, 4)
        }
    } else {
        (first, second)
    }
}

fn champion_item_set(champion: &Champion, build: &crate::builds::BuildRecommendation) -> Value {
    let items = |ids: &[u32]| -> Vec<Value> {
        ids.iter()
            .map(|id| json!({ "id": id.to_string(), "count": 1 }))
            .collect()
    };
    let mut starting = items(&build.starting_items);
    if build.starting_items.contains(&2003) {
        if let Some(potion) = starting.iter_mut().find(|item| item["id"] == "2003") {
            potion["count"] = json!(2);
        }
    }
    let mut early = vec![json!({ "id": build.boots.to_string(), "count": 1 })];
    match build.role.as_str() {
        "ADC" => early.extend(items(&[1001, 1038, 1037])),
        "MID" => early.extend(items(&[1082, 3802, 3108])),
        "JUNGLE" => early.extend(items(&[1001, 3133, 3067])),
        "SUPPORT" => early.extend(items(&[1001, 3067, 3114])),
        _ => early.extend(items(&[1001, 3077, 3067])),
    }
    let mut situational = build.situational_items.clone();
    let extra = match build.role.as_str() {
        "ADC" | "TOP" => [3033, 6609],
        "MID" => [3165, 3135],
        "SUPPORT" => [3075, 3107],
        _ => [0, 0],
    };
    for id in extra.into_iter().filter(|id| *id != 0) {
        if !situational.contains(&id) && crate::builds::item_available(id) {
            situational.push(id);
        }
    }
    situational.truncate(6);
    let elixir = if matches!(build.role.as_str(), "MID" | "SUPPORT") {
        2139
    } else {
        2140
    };
    let safe_name = champion
        .name
        .chars()
        .filter(|ch| {
            ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace() || *ch == '\'' || *ch == '.'
        })
        .collect::<String>();
    json!({
        "title": format!("RiftCore: {safe_name} {}", build.role),
        "type": "custom",
        "map": "any",
        "mode": "any",
        "priority": true,
        "sortrank": 1,
        "associatedChampions": [champion.id],
        "associatedMaps": [],
        "blocks": [
            { "type": "Starting Items & Potions", "items": starting },
            { "type": "Early Buys & Boots", "items": early },
            { "type": "Core Build", "items": items(&build.core_items) },
            { "type": "Situational Counters", "items": items(&situational) },
            { "type": "Consumables & Elixirs", "items": [
                { "id": "2055", "count": 2 },
                { "id": elixir.to_string(), "count": 1 }
            ]}
        ]
    })
}

fn settings_file() -> PathBuf {
    std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default())
        .join("RiftCore")
        .join("settings.json")
}

fn apply_native_theme(ctx: &egui::Context, theme: &str) {
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    let mut visuals = style.visuals.clone();
    let (panel, window, card, accent, accent_dim) = match theme {
        "ember" => (
            Color32::from_rgb(17, 13, 17),
            Color32::from_rgb(22, 18, 25),
            Color32::from_rgb(31, 25, 33),
            Color32::from_rgb(255, 155, 97),
            Color32::from_rgb(66, 38, 33),
        ),
        "void" => (
            Color32::from_rgb(9, 10, 22),
            Color32::from_rgb(14, 15, 31),
            Color32::from_rgb(21, 22, 42),
            Color32::from_rgb(174, 147, 255),
            Color32::from_rgb(41, 35, 74),
        ),
        _ => (BG, SURFACE, CARD, CYAN, CYAN_DIM),
    };
    visuals.panel_fill = panel;
    visuals.window_fill = window;
    visuals.widgets.noninteractive.bg_fill = card;
    visuals.widgets.inactive.bg_fill = card;
    visuals.widgets.hovered.bg_fill = accent_dim;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, accent);
    visuals.widgets.active.bg_fill = accent_dim;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, accent);
    visuals.selection.bg_fill = accent_dim;
    visuals.selection.stroke = Stroke::new(1.0, accent);
    style.visuals = visuals;
    ctx.set_style_of(egui::Theme::Dark, style);
}

fn install_font_family(ctx: &egui::Context, family: &str) {
    let mut definitions = egui::FontDefinitions::default();
    let (name, data): (&str, &'static [u8]) = match family {
        "outfit" => (
            "RiftCore Outfit",
            include_bytes!("../assets/fonts/Outfit.ttf"),
        ),
        "inter" => (
            "RiftCore Inter",
            include_bytes!("../assets/fonts/Inter.ttf"),
        ),
        _ => (
            "RiftCore Plus Jakarta",
            include_bytes!("../assets/fonts/PlusJakartaSans.ttf"),
        ),
    };
    definitions.font_data.insert(
        name.to_string(),
        std::sync::Arc::new(egui::FontData::from_static(data)),
    );
    definitions
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, name.to_string());
    ctx.set_fonts(definitions);
}

#[cfg(windows)]
fn build_tray_icon() -> Option<TrayIcon> {
    let menu = Menu::new();
    let show = MenuItem::with_id("riftcore-show", "Open RiftCore", true, None);
    let hud = MenuItem::with_id("riftcore-hud", "Toggle in-game HUD", true, None);
    let exit = MenuItem::with_id("riftcore-exit", "Exit RiftCore", true, None);
    menu.append(&show).ok()?;
    menu.append(&hud).ok()?;
    menu.append(&exit).ok()?;

    let image = image::load_from_memory(crate::assets::RIFTCORE_LOGO).ok()?;
    let icon = image
        .resize_exact(32, 32, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    let icon = tray_icon::Icon::from_rgba(icon.into_raw(), 32, 32).ok()?;
    TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_icon(icon)
        .with_tooltip("RiftCore Companion")
        // Left click opens the app; right click shows the menu.
        .with_menu_on_left_click(false)
        .build()
        .ok()
}

#[cfg(windows)]
fn play_ready_alert() {
    #[link(name = "user32")]
    extern "system" {
        fn MessageBeep(kind: u32) -> i32;
    }
    unsafe {
        let _ = MessageBeep(0x0000_0030);
    }
}

#[cfg(not(windows))]
fn play_ready_alert() {}

fn load_settings(path: &std::path::Path) -> AppSettings {
    std::fs::read(path)
        .ok()
        .and_then(|content| serde_json::from_slice::<AppSettings>(&content).ok())
        .unwrap_or_default()
}

fn start_hotkey_listener(
    ctx: egui::Context,
) -> (Receiver<HotkeyEvent>, Option<std::thread::JoinHandle<()>>) {
    let (sender, receiver) = std::sync::mpsc::channel();
    #[cfg(windows)]
    let handle = std::thread::Builder::new()
        .name("riftcore-hotkeys".to_string())
        .spawn(move || {
            #[repr(C)]
            struct Point {
                x: i32,
                y: i32,
            }
            #[repr(C)]
            struct Message {
                hwnd: *mut std::ffi::c_void,
                message: u32,
                wparam: usize,
                lparam: isize,
                time: u32,
                point: Point,
                private: u32,
            }
            #[link(name = "user32")]
            extern "system" {
                fn RegisterHotKey(
                    hwnd: *mut std::ffi::c_void,
                    id: i32,
                    modifiers: u32,
                    key: u32,
                ) -> i32;
                fn UnregisterHotKey(hwnd: *mut std::ffi::c_void, id: i32) -> i32;
                fn GetMessageW(
                    message: *mut Message,
                    hwnd: *mut std::ffi::c_void,
                    minimum: u32,
                    maximum: u32,
                ) -> i32;
            }
            const MOD_ALT: u32 = 0x0001;
            const MOD_SHIFT: u32 = 0x0004;
            const MOD_NOREPEAT: u32 = 0x4000;
            const VK_OEM_3: u32 = 0x00C0;
            const VK_SPACE: u32 = 0x20;
            const WM_HOTKEY: u32 = 0x0312;
            unsafe {
                let alt = MOD_ALT | MOD_NOREPEAT;
                let alt_registered = RegisterHotKey(std::ptr::null_mut(), 1, alt, VK_OEM_3) != 0;
                let shift_registered =
                    RegisterHotKey(std::ptr::null_mut(), 2, alt | MOD_SHIFT, VK_OEM_3) != 0;
                // Alt+Shift+Space is the fallback HUD toggle for layouts without a ` key.
                // (Ctrl+Space is avoided: it is IDE autocomplete / IME switching everywhere.)
                let space_registered =
                    RegisterHotKey(std::ptr::null_mut(), 3, alt | MOD_SHIFT, VK_SPACE) != 0;
                let mut message: Message = std::mem::zeroed();
                loop {
                    let result = GetMessageW(&mut message, std::ptr::null_mut(), 0, 0);
                    if result <= 0 {
                        break;
                    }
                    if message.message == WM_HOTKEY {
                        let event = match message.wparam {
                            1 | 3 => HotkeyEvent {
                                toggle_overlay: true,
                                toggle_passthrough: false,
                            },
                            2 => HotkeyEvent {
                                toggle_overlay: false,
                                toggle_passthrough: true,
                            },
                            _ => continue,
                        };
                        let _ = sender.send(event);
                        // Wake the UI even while it is hidden so the toggle applies at once.
                        ctx.request_repaint();
                    }
                }
                if alt_registered {
                    UnregisterHotKey(std::ptr::null_mut(), 1);
                }
                if shift_registered {
                    UnregisterHotKey(std::ptr::null_mut(), 2);
                }
                if space_registered {
                    UnregisterHotKey(std::ptr::null_mut(), 3);
                }
            }
        })
        .ok();
    #[cfg(not(windows))]
    let handle = {
        drop((ctx, sender));
        None
    };
    (receiver, handle)
}

#[cfg(test)]
mod tft_text_tests {
    use super::clean_tft_text;

    #[test]
    fn strips_riot_tooltip_markup() {
        assert_eq!(
            clean_tft_text("Gain a champion. <br><br><rules>Champion: 0</rules>"),
            "Gain a champion."
        );
        assert_eq!(
            clean_tft_text("Deal <magicDamage>50</magicDamage> damage"),
            "Deal 50 damage"
        );
        assert_eq!(clean_tft_text("plain text"), "plain text");
        assert_eq!(clean_tft_text("broken <tag"), "broken");
    }
}

#[cfg(test)]
mod live_and_champ_select_tests {
    use super::*;
    use serde_json::json;

    fn live(team: &str) -> Value {
        json!({
            "activePlayer": {"riotId": "Me#EUW"},
            "allPlayers": [
                {"riotId": "Other#EUW", "team": "ORDER"},
                {"riotId": "Me#EUW", "team": team, "items": [{"itemID": 1055}]}
            ]
        })
    }

    #[test]
    fn scoreboard_lists_own_team_first() {
        let labels = |live: &Value| scoreboard_teams(live).map(|(team, label, _)| (team, label));
        assert_eq!(
            labels(&live("CHAOS")),
            [("CHAOS", "YOUR TEAM"), ("ORDER", "ENEMY TEAM")]
        );
        assert_eq!(
            labels(&live("ORDER")),
            [("ORDER", "YOUR TEAM"), ("CHAOS", "ENEMY TEAM")]
        );
        assert_eq!(labels(&json!({}))[0], ("ORDER", "BLUE TEAM"));
        let chaos = live("CHAOS");
        let me = active_player_entry(&chaos).unwrap();
        assert!(me.get("items").is_some());
    }

    #[test]
    fn champ_select_role_and_lock_in() {
        let mut session = json!({
            "localPlayerCellId": 2,
            "myTeam": [{"cellId": 2, "assignedPosition": "utility", "championId": 0,
                        "championPickIntent": 25}],
            "actions": [[{"actorCellId": 2, "type": "pick", "championId": 25,
                          "completed": false}]]
        });
        assert_eq!(champ_select_assigned_role(&session), Some("SUPPORT"));
        assert_eq!(champ_select_locked_champion(&session), None);
        session["actions"][0][0]["completed"] = json!(true);
        assert_eq!(champ_select_locked_champion(&session), Some(25));
        session["myTeam"][0]["assignedPosition"] = json!("bottom");
        assert_eq!(champ_select_assigned_role(&session), Some("ADC"));
        // ARAM: no pick actions, the given champion counts as locked.
        let aram = json!({"localPlayerCellId": 1, "actions": [],
                          "myTeam": [{"cellId": 1, "championId": 99, "assignedPosition": ""}]});
        assert_eq!(champ_select_locked_champion(&aram), Some(99));
        assert_eq!(champ_select_assigned_role(&aram), None);
    }
}

#[cfg(test)]
mod spell_dropdown_tests {
    use super::*;

    const FLASH: i64 = 4;
    const TELEPORT: i64 = 12;
    const IGNITE: i64 = 14;

    #[test]
    fn assign_spell_sets_swaps_and_ignores_a_re_pick() {
        let (mut d, mut f) = (FLASH, TELEPORT);
        assign_spell(&mut d, &mut f, 0, IGNITE);
        assert_eq!((d, f), (IGNITE, TELEPORT), "D takes a free spell");
        assign_spell(&mut d, &mut f, 1, IGNITE);
        assert_eq!(
            (d, f),
            (TELEPORT, IGNITE),
            "F taking D's spell swaps the keys"
        );
        assign_spell(&mut d, &mut f, 1, IGNITE);
        assert_eq!(
            (d, f),
            (TELEPORT, IGNITE),
            "re-picking the current spell is a no-op"
        );
        assign_spell(&mut d, &mut f, 0, IGNITE);
        assert_eq!((d, f), (IGNITE, TELEPORT), "D taking F's spell swaps back");
        assign_spell(&mut d, &mut f, 1, FLASH);
        assert_eq!((d, f), (IGNITE, FLASH), "F takes a free spell");
    }

    /// Runs `summoner_spell_dropdowns` headlessly, one egui frame at a time.
    struct Picker {
        ctx: egui::Context,
        spells: (i64, i64),
        card_ids: [egui::Id; 2],
    }

    impl Picker {
        fn new() -> Self {
            let mut picker = Self {
                ctx: egui::Context::default(),
                spells: (FLASH, TELEPORT),
                card_ids: [egui::Id::NULL; 2],
            };
            picker.frame(Vec::new());
            picker
        }

        fn frame(&mut self, events: Vec<egui::Event>) {
            let raw = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(480.0, 640.0),
                )),
                events,
                ..Default::default()
            };
            let textures = HashMap::new();
            let (one, two) = (&mut self.spells.0, &mut self.spells.1);
            let mut card_ids = self.card_ids;
            self.ctx
                .run_ui(raw, |ui| {
                    card_ids = [0, 1].map(|slot| spell_slot_id(ui, slot));
                    summoner_spell_dropdowns(ui, one, two, &textures);
                })
                .drop_without_applying_deltas();
            self.card_ids = card_ids;
        }

        /// Moves the pointer to `pos`, presses and releases, then runs one more frame: a
        /// popup that just opened spends its first frame sizing itself.
        fn click(&mut self, pos: egui::Pos2) {
            let button = |pressed| egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            };
            self.frame(vec![egui::Event::PointerMoved(pos)]);
            self.frame(vec![button(true)]);
            self.frame(vec![button(false)]);
            self.frame(Vec::new());
        }

        fn shown(&self, id: egui::Id) -> egui::Response {
            self.ctx
                .read_response(id)
                .expect("widget was shown last frame")
        }

        fn card(&self, slot: usize) -> egui::Rect {
            self.shown(self.card_ids[slot]).rect
        }

        fn row(&self, slot: usize, spell: i64) -> egui::Rect {
            self.shown(self.card_ids[slot].with(("spell", spell))).rect
        }

        fn is_open(&self, slot: usize) -> bool {
            let card = self.shown(self.card_ids[slot]);
            egui::Popup::is_id_open(&self.ctx, egui::Popup::default_response_id(&card))
        }
    }

    #[test]
    fn a_key_card_opens_its_dropdown_and_a_pick_assigns_and_closes_it() {
        let mut picker = Picker::new();
        assert!(!picker.is_open(0) && !picker.is_open(1));

        picker.click(picker.card(0).center());
        assert!(picker.is_open(0), "clicking D opens D's dropdown");
        assert!(!picker.is_open(1));
        let ignite = picker.row(0, IGNITE);
        assert!(
            ignite.top() > picker.card(0).bottom(),
            "the list opens below the card"
        );
        picker.click(ignite.center());
        assert_eq!(picker.spells, (IGNITE, TELEPORT));
        assert!(!picker.is_open(0), "a pick closes the dropdown");

        picker.click(picker.card(0).center());
        picker.click(picker.row(0, IGNITE).center());
        assert_eq!(
            picker.spells,
            (IGNITE, TELEPORT),
            "re-picking D's own spell changes nothing"
        );
        assert!(!picker.is_open(0));

        picker.click(picker.card(1).center());
        assert!(picker.is_open(1) && !picker.is_open(0));
        picker.click(picker.row(1, IGNITE).center());
        assert_eq!(
            picker.spells,
            (TELEPORT, IGNITE),
            "F taking D's spell swaps"
        );
        assert!(!picker.is_open(1));
    }

    #[test]
    fn escape_outside_clicks_and_the_card_close_it_without_changes() {
        let mut picker = Picker::new();

        picker.click(picker.card(0).center());
        assert!(picker.is_open(0));
        picker.frame(vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        picker.frame(Vec::new());
        assert!(!picker.is_open(0), "Escape closes it");

        picker.click(picker.card(1).center());
        assert!(picker.is_open(1));
        picker.click(egui::pos2(470.0, 630.0));
        assert!(!picker.is_open(1), "a click outside closes it");

        picker.click(picker.card(0).center());
        picker.click(picker.card(0).center());
        assert!(!picker.is_open(0), "clicking the card again closes it");

        picker.click(picker.card(0).center());
        picker.click(picker.card(1).center());
        assert!(
            picker.is_open(1) && !picker.is_open(0),
            "opening F closes D"
        );
        picker.click(picker.card(0).center());
        assert!(
            picker.is_open(0) && !picker.is_open(1),
            "opening D closes F"
        );
        assert_eq!(picker.spells, (FLASH, TELEPORT), "nothing was picked");
    }

    /// The default 1380x880 window (with the app's 1.06 zoom) leaves about 90 pt per key card,
    /// and wide windows up to 122 pt. Every name but "Mark / Snowball" must fit uncut there in
    /// the default font.
    #[test]
    fn spell_names_fit_on_key_cards_from_the_default_window_up() {
        const MARK: i64 = 32;
        let ctx = egui::Context::default();
        install_font_family(&ctx, "plus-jakarta");
        let mut cut = Vec::new();
        // Fonts given to set_fonts apply from the next frame on, so measure in the second one.
        for _ in 0..2 {
            cut.clear();
            ctx.run_ui(egui::RawInput::default(), |ui| {
                for width in [90.0, 104.0, 122.0] {
                    let card = egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(width, 42.0));
                    let (_, text) = spell_slot_layout(card);
                    for &(id, name) in SUMMONER_SPELLS.iter().filter(|(id, _)| *id != MARK) {
                        let natural = ui
                            .painter()
                            .layout_no_wrap(name.to_owned(), FontId::proportional(11.5), TEXT)
                            .size()
                            .x;
                        if natural > text.width() {
                            cut.push(format!(
                                "{name} ({id}) needs {natural:.1} pt but has {:.1} on a {width} pt card",
                                text.width()
                            ));
                        }
                    }
                }
            })
            .drop_without_applying_deltas();
        }
        assert!(cut.is_empty(), "{cut:?}");
    }
}
