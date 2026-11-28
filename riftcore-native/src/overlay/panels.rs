//! Overlay HUD panels rendered inside the isolated overlay `egui::Context` (`zoom_factor = 1.0`).
//!
//! Riot Vanguard & Third-Party Policy Compliance:
//! - NO enemy summoner spell cooldown tracking (`render_enemy_spell_timers`, `SPELLS` tab,
//!   or enemy spell callouts).
//! - NO subjective `"rank"` (`S+`/`S`/`A`) labels on Mayhem/Arena cards.
//! - Active-game live coaching (`LiveBuildAdvice` / `CoachInsight`) is strictly gated behind
//!   `crate::domain::protocol::LIVE_COACH_ENABLED` (`false`).

use crate::domain::builds::{self, BuildProfile};
use crate::domain::champions::{self, Champion, Role};
use crate::domain::items;
use crate::domain::live_game::{self, countdown_secs, CsPace, LiveGameState};
use crate::domain::mayhem::{self, CardMode, CardTier, MayhemCard};
use crate::domain::protocol::LIVE_COACH_ENABLED;
use crate::overlay::host::{OverlayPanelSettings, OverlayRenderState, OverlayTimerTab};
use crate::overlay::hud_modes::{
    clamp_logical_panel_pos, default_relic_pos, default_toolbar_pos, league_ability_bar_zone,
    HudPresentationMode,
};
use crate::overlay::minimap::{jungle_route_status, minimap_square, render_minimap_guide_pins};
use eframe::egui::{
    self, Align, Color32, CornerRadius, Frame, Id, Layout, Margin, Pos2, Rect, RichText,
    ScrollArea, Stroke, Vec2,
};
use std::collections::HashMap;

const CARD_BG: Color32 = Color32::from_rgb(13, 21, 37);
const BORDER: Color32 = Color32::from_rgb(42, 62, 92);
const CYAN: Color32 = Color32::from_rgb(66, 229, 232);
const GOLD: Color32 = Color32::from_rgb(255, 197, 103);
const GREEN: Color32 = Color32::from_rgb(92, 224, 149);
const RED: Color32 = Color32::from_rgb(255, 108, 122);
const TEXT: Color32 = Color32::from_rgb(239, 247, 255);
const SECONDARY: Color32 = Color32::from_rgb(154, 173, 200);
const MUTED: Color32 = Color32::from_rgb(98, 118, 146);

/// Compile-time compliance assertion: enemy spell timers remain removed from overlay UI.
pub const OVERLAY_ENEMY_SPELL_TIMERS_REMOVED: bool = true;

fn format_clock(seconds: u64) -> String {
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

fn panel_fill(opacity: f32, boost_alpha: u8) -> Color32 {
    let base = (opacity.clamp(0.35, 1.0) * boost_alpha as f32).round() as u8;
    Color32::from_rgba_unmultiplied(7, 12, 22, base)
}

fn overlay_button(ui: &mut egui::Ui, label: &str, active: bool, accent: Color32) -> egui::Response {
    let fill = if active {
        Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 44)
    } else {
        Color32::from_rgba_unmultiplied(15, 24, 42, 220)
    };
    let stroke = if active { accent } else { BORDER };
    let text_color = if active { accent } else { TEXT };
    ui.add(
        egui::Button::new(RichText::new(label).size(9.5).strong().color(text_color))
            .fill(fill)
            .stroke(Stroke::new(1.0, stroke))
            .corner_radius(CornerRadius::same(7)),
    )
}

/// Saves a panel position after an edit-layout drag (same flow as the other movable panels).
fn remember_drag(
    positions: &mut HashMap<String, [f32; 2]>,
    updates: &mut Vec<(String, [f32; 2])>,
    key: &str,
    response: &egui::Response,
) {
    if response.drag_stopped() {
        let pos = [response.rect.min.x, response.rect.min.y];
        positions.insert(key.to_string(), pos);
        updates.push((key.to_string(), pos));
    }
}

/// Gap kept between RiftCore panels, and between them and League's HUD, in points.
const PANEL_GAP: f32 = 6.0;

/// A HUD panel area. No fade-in: the HUD only repaints when something changed, so a fading
/// panel could stay half-transparent until the next game-clock second.
fn overlay_area(id: Id) -> egui::Area {
    egui::Area::new(id).fade_in(false)
}

/// Size the panel `id` had when last laid out (`nominal` before its first layout).
fn measured_size(ctx: &egui::Context, id: Id, nominal: Vec2) -> Vec2 {
    ctx.memory(|mem| mem.area_rect(id))
        .map_or(nominal, |rect| rect.size())
}

/// A saved (dragged) panel position, clamped on screen with the panel's real size so it
/// survives resolution changes; `default` when the panel was never moved.
fn saved_or(
    positions: &HashMap<String, [f32; 2]>,
    key: &str,
    area: Rect,
    size: Vec2,
    default: Pos2,
) -> Pos2 {
    positions
        .get(key)
        .map_or(default, |pos| clamp_logical_panel_pos(*pos, area, size))
}

/// Shows panel `id`; when it had not been laid out yet (egui's invisible sizing pass) or its
/// size changed, asks egui to lay the frame out once more, so the frame that reaches the
/// screen shows every panel at its final size and place.
fn show_settled<R>(
    ctx: &egui::Context,
    id: Id,
    area: egui::Area,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let before = ctx.memory(|mem| mem.area_rect(id));
    let shown = area.show(ctx, add_contents);
    let settled =
        before.is_some_and(|rect| (rect.size() - shown.response.rect.size()).length() < 0.5);
    if !settled {
        ctx.request_discard("RiftCore HUD panel laid out");
    }
    shown
}

/// Whole seconds left on manual timer `key` at game second `now` (0 when not running).
fn timer_left(timers: &HashMap<String, f64>, key: &str, now: f64) -> u64 {
    timers.get(key).map_or(0, |end| countdown_secs(*end, now))
}

/// Sort key for timer lists: running timers first, soonest first; idle entries keep their
/// order (use with a stable sort).
fn soonest_first(remaining: u64) -> (bool, u64) {
    (remaining == 0, remaining)
}

fn overlay_metric(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    ui.vertical(|ui| {
        ui.label(RichText::new(label).size(8.5).strong().color(MUTED));
        ui.label(RichText::new(value).size(12.5).strong().color(color));
    });
}

/// Resolves whether the active player is missing a slot-6 vision trinket in their live items.
pub fn missing_trinket_reminder(live: Option<&LiveGameState>) -> bool {
    let Some(live) = live else {
        return false;
    };
    let has_trinket =
        live.active_player.items.iter().any(|item| {
            item.item_id > 0 && (item.slot == 6 || items::has_trinket_tag(item.item_id))
        });
    !has_trinket
}

/// Resolves the personal champion build recommendation for the active player (`activePlayer -> allPlayers[]`).
pub fn resolve_personal_build(live: &LiveGameState) -> Option<(&'static Champion, BuildProfile)> {
    let mut query = live.active_player.champion_name.as_str();
    if query.is_empty() {
        if let Some(idx) = live.active_player.all_players_index {
            if let Some(player) = live.all_players.get(idx) {
                query = if !player.champion_name.is_empty() {
                    player.champion_name.as_str()
                } else {
                    player.raw_champion_name.as_str()
                };
            }
        }
    }
    let champ = champions::champion_by_name_or_key(query)?;
    let requested_role = Role::from_str_lossy(&live.active_player.position);
    let profile = builds::recommend(&champ.key, &champ.name, &champ.roles, requested_role);
    Some((champ, profile))
}

/// Derived live HUD extras (level, next skill/item, KP, team score, CS pace, vision).
#[derive(Default)]
struct HudExtras {
    level: u32,
    next_skill: Option<&'static str>,
    pace: Color32,
    cs10: Option<u32>,
    kp: String,
    score: Option<(u32, u32)>,
    next_item: Option<(String, Option<u32>, bool)>,
    vision: u32,
}

fn hud_extras(
    live: Option<&LiveGameState>,
    cs10: Option<u32>,
    cspm: f64,
    build: bool,
) -> HudExtras {
    let mut x = HudExtras {
        level: 1,
        pace: TEXT,
        cs10,
        kp: "KP —".into(),
        ..Default::default()
    };
    let Some(live) = live else { return x };
    let p = &live.active_player;
    x.level = p.level.max(1);
    x.pace = match live_game::cs_pace(cspm, live.game_time_secs) {
        CsPace::Good => GREEN,
        CsPace::Fair => GOLD,
        CsPace::Low => RED,
        CsPace::Neutral => TEXT,
    };
    x.vision = p.scores.ward_score.max(0.0).round() as u32;
    if !p.team.is_empty() {
        let (ally, enemy) = live_game::team_kill_score(&live.all_players, &p.team);
        x.score = Some((ally, enemy));
        if let Some(pct) = live_game::kill_participation_pct(p.scores.kills, p.scores.assists, ally)
        {
            x.kp = format!("KP {pct}%");
        }
    }
    if let Some((_, b)) = build.then(|| resolve_personal_build(live)).flatten() {
        let a = &p.abilities;
        let ranks = [
            a.q.ability_level,
            a.w.ability_level,
            a.e.ability_level,
            a.r.ability_level,
        ];
        x.next_skill = live_game::next_skill_up(x.level, ranks, &b.skill_order);
        let owned: Vec<u32> = p.items.iter().map(|i| i.item_id).collect();
        x.next_item = live_game::next_build_item(&b.core_items, &owned).map(|id| {
            let cost = items::item_total_cost(id).filter(|c| *c > 0);
            let buy = cost.is_some_and(|c| p.current_gold >= c as f64);
            (items::item_name(id), cost, buy)
        });
    }
    x
}

fn chip(ui: &mut egui::Ui, text: &str, color: Color32, filled: bool) {
    let alpha = if filled { 70 } else { 24 };
    Frame::new()
        .fill(Color32::from_rgba_unmultiplied(
            color.r(),
            color.g(),
            color.b(),
            alpha,
        ))
        .stroke(Stroke::new(1.0, color))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(6, 1))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(9.5).strong().color(color))
        });
}

/// "LEVEL UP: Q" / "NEXT <item>" chips (gated by skill-order / item-goal toggles).
fn next_chips(ui: &mut egui::Ui, x: &HudExtras, skill: bool, item: bool) {
    if let Some(s) = x.next_skill.filter(|_| skill) {
        chip(ui, &format!("LEVEL UP: {s}"), GOLD, true);
    }
    if let Some((name, cost, buy)) = x.next_item.as_ref().filter(|_| item) {
        let mut text = format!("NEXT {name}");
        if let Some(c) = cost {
            text.push_str(&format!(" · {c}g"));
        }
        if *buy {
            text.push_str(" · CAN BUY");
        }
        chip(ui, &text, if *buy { GREEN } else { SECONDARY }, *buy);
    }
}

/// Filters Mayhem & Arena augment cards cleanly without subjective rank strings.
pub fn filter_mayhem_cards(
    mode_filter: &str,
    tier_filter: &str,
    query: &str,
) -> Vec<&'static MayhemCard> {
    let mode = match mode_filter.trim().to_ascii_lowercase().as_str() {
        "aram" => Some(CardMode::Aram),
        "arena" => Some(CardMode::Arena),
        _ => None,
    };
    let tier = if tier_filter.eq_ignore_ascii_case("ALL") {
        None
    } else {
        CardTier::from_str_lossy(tier_filter)
    };
    mayhem::filter_cards(mode, tier, query)
}

/// Main entry point rendering the HUD inside an isolated overlay `egui::Context` (`zoom_factor = 1.0`).
pub fn render_overlay_ui(
    ui: &mut egui::Ui,
    state: &mut OverlayRenderState,
    interactive_rects_out: &mut Vec<Rect>,
) {
    interactive_rects_out.clear();
    let ctx = ui.ctx().clone();

    egui::CentralPanel::default()
        .frame(
            Frame::new()
                .fill(Color32::TRANSPARENT)
                .inner_margin(Margin::same(14)),
        )
        .show(ui, |ui| {
            let area = ui.available_rect_before_wrap();
            if let Some(g) = state.live_game.as_ref() {
                if g.game_time_secs < 600.0 {
                    state.cs_at_10 = None;
                } else if g.game_time_secs < 630.0 && state.cs_at_10.is_none() {
                    state.cs_at_10 = Some(g.active_player.scores.creep_score);
                }
            }
            // Smoothed game clock (whole seconds); every countdown below is derived from it.
            let clock = state.clock_secs;
            let now_secs = clock as f64;
            state.manual_timers.retain(|_, end| *end > now_secs);
            if state.settings.hud_mode != HudPresentationMode::Integrated && state.show_mayhem_cards
            {
                // Compact modes swap their small window for the cards drawer while it is open.
                render_mayhem_drawer(&ctx, area, state, interactive_rects_out);
                return;
            }
            let live_ref = state.live_game.as_ref();
            let scores = live_ref.map(|g| &g.active_player.scores);
            let (kills, deaths, assists, creep_score) = scores
                .map(|s| (s.kills, s.deaths, s.assists, s.creep_score))
                .unwrap_or((0, 0, 0, 0));
            let kda = format!("{kills} / {deaths} / {assists}");
            let cs_per_minute = creep_score as f64 / (clock as f64 / 60.0).max(1.0);
            let current_gold = live_ref
                .map(|g| g.active_player.current_gold.max(0.0) as u64)
                .unwrap_or(0);
            let set = &state.settings;
            let x = hud_extras(
                live_ref,
                state.cs_at_10,
                cs_per_minute,
                set.hud_skill_order || set.hud_item_goal,
            );
            let (skill_on, item_on) = (set.hud_skill_order, set.hud_item_goal);
            let game_mode = live_ref
                .map(|g| g.game_mode.as_str())
                .filter(|m| !m.is_empty())
                .unwrap_or(if state.preview_mode {
                    "PREVIEW"
                } else {
                    "LIVE"
                });

            match state.settings.hud_mode {
                HudPresentationMode::Bar => {
                    let bar = Frame::new()
                        .fill(panel_fill(state.settings.opacity, 238))
                        .stroke(Stroke::new(1.0, CYAN))
                        .corner_radius(CornerRadius::same(12))
                        .inner_margin(Margin::symmetric(12, 7))
                        .show(ui, |ui| {
                            ui.horizontal_centered(|ui| {
                                ui.label(RichText::new("RIFTCORE").size(10.0).strong().color(CYAN));
                                ui.separator();
                                ui.label(
                                    RichText::new(format_clock(clock))
                                        .size(16.0)
                                        .monospace()
                                        .strong()
                                        .color(TEXT),
                                );
                                chip(ui, &format!("LV {}", x.level), CYAN, false);
                                ui.label(RichText::new(&kda).size(12.0).strong().color(TEXT));
                                ui.label(
                                    RichText::new(format!("{creep_score} CS"))
                                        .size(10.5)
                                        .color(GOLD),
                                );
                                if state.settings.hud_cs_benchmark {
                                    ui.label(
                                        RichText::new(format!("{cs_per_minute:.1}/m"))
                                            .size(10.0)
                                            .color(x.pace),
                                    );
                                }
                                if state.settings.hud_gold_lead {
                                    ui.label(
                                        RichText::new(format!("{current_gold} G"))
                                            .size(10.0)
                                            .color(GREEN),
                                    );
                                }
                                next_chips(ui, &x, skill_on, false);
                                if state.settings.hud_mayhem_cards
                                    && overlay_button(ui, "CARDS", state.show_mayhem_cards, GOLD)
                                        .clicked()
                                {
                                    state.show_mayhem_cards = !state.show_mayhem_cards;
                                }
                                if overlay_button(ui, "×", false, RED).clicked() {
                                    state.pending_actions.request_close_overlay = true;
                                }
                            });
                        });
                    interactive_rects_out.push(bar.response.rect);
                    return;
                }
                HudPresentationMode::Pill => {
                    let pill = Frame::new()
                        .fill(panel_fill(state.settings.opacity, 238))
                        .stroke(Stroke::new(1.0, GOLD))
                        .corner_radius(CornerRadius::same(20))
                        .inner_margin(Margin::symmetric(13, 7))
                        .show(ui, |ui| {
                            ui.horizontal_centered(|ui| {
                                ui.label(
                                    RichText::new(format_clock(clock))
                                        .size(14.0)
                                        .monospace()
                                        .strong()
                                        .color(TEXT),
                                );
                                chip(ui, &format!("LV {}", x.level), CYAN, false);
                                ui.label(RichText::new(&kda).size(11.0).strong().color(TEXT));
                                ui.label(
                                    RichText::new(format!("{creep_score} CS"))
                                        .size(10.0)
                                        .color(GOLD),
                                );
                                if state.settings.hud_cs_benchmark {
                                    ui.label(
                                        RichText::new(format!("{cs_per_minute:.1}/m"))
                                            .size(9.5)
                                            .color(x.pace),
                                    );
                                }
                                next_chips(ui, &x, skill_on, false);
                                if state.settings.hud_mayhem_cards
                                    && overlay_button(ui, "CARDS", state.show_mayhem_cards, GOLD)
                                        .clicked()
                                {
                                    state.show_mayhem_cards = !state.show_mayhem_cards;
                                }
                                if overlay_button(ui, "×", false, RED).clicked() {
                                    state.pending_actions.request_close_overlay = true;
                                }
                            });
                        });
                    interactive_rects_out.push(pill.response.rect);
                    return;
                }
                HudPresentationMode::Card => {
                    let card = Frame::new()
                        .fill(panel_fill(state.settings.opacity, 240))
                        .stroke(Stroke::new(1.0, CYAN))
                        .corner_radius(CornerRadius::same(16))
                        .inner_margin(Margin::same(14))
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
                                            "{}  ·  {game_mode}",
                                            format_clock(clock)
                                        ))
                                        .size(15.0)
                                        .strong()
                                        .color(TEXT),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if overlay_button(ui, "×", false, RED).clicked() {
                                        state.pending_actions.request_close_overlay = true;
                                    }
                                });
                            });
                            ui.separator();
                            ui.horizontal(|ui| {
                                overlay_metric(ui, "K / D / A", &kda, TEXT);
                                overlay_metric(ui, "CS", &creep_score.to_string(), GOLD);
                                overlay_metric(
                                    ui,
                                    "CS / MIN",
                                    &format!("{cs_per_minute:.1}"),
                                    x.pace,
                                );
                                if let Some(cs10) = x.cs10 {
                                    overlay_metric(ui, "CS@10", &cs10.to_string(), SECONDARY);
                                }
                            });
                            ui.horizontal(|ui| {
                                overlay_metric(ui, "LEVEL", &x.level.to_string(), CYAN);
                                if state.settings.hud_gold_lead {
                                    let kp = x.kp.trim_start_matches("KP ");
                                    overlay_metric(ui, "KILL PART.", kp, TEXT);
                                    let team =
                                        x.score.map_or("—".into(), |(a, e)| format!("{a} – {e}"));
                                    overlay_metric(ui, "TEAM", &team, TEXT);
                                }
                                if state.settings.hud_trinket_reminder {
                                    overlay_metric(ui, "VISION", &x.vision.to_string(), SECONDARY);
                                }
                            });
                            ui.horizontal_wrapped(|ui| next_chips(ui, &x, skill_on, item_on));
                            if state.settings.hud_gold_lead {
                                ui.label(
                                    RichText::new(format!("CURRENT GOLD   {current_gold}"))
                                        .size(11.0)
                                        .color(GREEN),
                                );
                            }
                            if state.settings.hud_objective_timers {
                                if let Some(live) = live_ref {
                                    for timer in live.objective_timers_at(now_secs) {
                                        ui.label(
                                            RichText::new(format!(
                                                "{:<9}  {}",
                                                timer.label,
                                                timer.formatted_remaining()
                                            ))
                                            .size(10.5)
                                            .monospace()
                                            .color(TEXT),
                                        );
                                    }
                                }
                            }
                            ui.horizontal(|ui| {
                                if state.settings.hud_mayhem_cards
                                    && overlay_button(
                                        ui,
                                        "MAYHEM & ARENA CARDS",
                                        state.show_mayhem_cards,
                                        GOLD,
                                    )
                                    .clicked()
                                {
                                    state.show_mayhem_cards = !state.show_mayhem_cards;
                                }
                            });
                        });
                    interactive_rects_out.push(card.response.rect);
                    return;
                }
                HudPresentationMode::Integrated => {}
            }

            // Integrated full-viewport HUD. League's HUD (ability bar, minimap, chat) scales
            // with the game size; RiftCore's panels keep clear of it and of each other:
            // scoreboard and toolbar top-centre, ally ults top-left, player panel top-right,
            // objectives dock right above League's minimap, one drawer at a time under the toolbar.
            let screen = ctx.content_rect();
            let minimap = minimap_square(screen, &state.settings);
            let ability_bar = league_ability_bar_zone(screen);
            render_minimap_guide_pins(&ctx, screen, live_ref, &state.settings, clock);
            let edit = state.settings.hud_edit_layout;

            let top_id = Id::unique("riftcore-hud-top");
            let top_size = measured_size(&ctx, top_id, Vec2::new(600.0, 70.0));
            let top_moved = state.settings.hud_positions.contains_key("scoreboard");
            let top_pos = saved_or(
                &state.settings.hud_positions,
                "scoreboard",
                area,
                top_size,
                Pos2::new(area.center().x - top_size.x / 2.0, area.top() + 20.0),
            );
            let top_resp = show_settled(
                &ctx,
                top_id,
                overlay_area(top_id).current_pos(top_pos).movable(edit),
                |ui| {
                    Frame::new()
                        .fill(panel_fill(state.settings.opacity, 228))
                        .stroke(Stroke::new(1.0, CYAN))
                        .inner_margin(Margin::same(10))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("◆ RIFTCORE").size(11.0).strong().color(CYAN),
                                );
                                ui.separator();
                                ui.label(
                                    RichText::new(format_clock(clock))
                                        .monospace()
                                        .size(18.0)
                                        .strong()
                                        .color(TEXT),
                                );
                                ui.separator();
                                chip(ui, &format!("LV {}", x.level), CYAN, false);
                                ui.label(RichText::new(&kda).size(13.0).strong().color(TEXT));
                                if state.settings.hud_gold_lead {
                                    ui.label(RichText::new(&x.kp).size(10.5).color(SECONDARY));
                                    if let Some((a, e)) = x.score {
                                        ui.label(
                                            RichText::new(format!("{a} – {e}"))
                                                .size(11.5)
                                                .strong()
                                                .color(TEXT),
                                        );
                                    }
                                }
                                ui.separator();
                                ui.label(
                                    RichText::new(format!("{creep_score} CS"))
                                        .size(11.5)
                                        .color(GOLD),
                                );
                                if state.settings.hud_cs_benchmark {
                                    ui.separator();
                                    ui.label(
                                        RichText::new(format!("{cs_per_minute:.1}/m"))
                                            .size(11.0)
                                            .color(x.pace),
                                    );
                                    if let Some(cs10) = x.cs10 {
                                        let t = format!("@10 {cs10}");
                                        ui.label(RichText::new(t).size(10.0).color(SECONDARY));
                                    }
                                }
                                if state.settings.hud_gold_lead {
                                    ui.separator();
                                    ui.label(
                                        RichText::new(format!("{current_gold} G"))
                                            .size(11.0)
                                            .color(GREEN),
                                    );
                                }
                                if state.settings.hud_trinket_reminder {
                                    let t = format!("VIS {}", x.vision);
                                    ui.label(RichText::new(t).size(10.0).color(SECONDARY));
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(game_mode).size(9.0).strong().color(MUTED));
                                next_chips(ui, &x, skill_on, item_on);
                            });
                        });
                },
            );
            remember_drag(
                &mut state.settings.hud_positions,
                &mut state.pending_actions.updated_positions,
                "scoreboard",
                &top_resp.response,
            );
            if edit {
                interactive_rects_out.push(top_resp.response.rect);
            }

            // Objectives dock: right above League's minimap (which already includes the nudge),
            // aligned with its outer edge. Summoner's Rift only: other maps have no timers.
            if live_ref.is_none_or(LiveGameState::is_summoners_rift) {
                let dock_id = Id::unique("riftcore-hud-objectives");
                let dock_size = measured_size(&ctx, dock_id, Vec2::new(172.0, 80.0));
                let dock_default = Pos2::new(
                    if state.settings.hud_minimap_flip {
                        minimap.left()
                    } else {
                        minimap.right() - dock_size.x
                    },
                    minimap.top() - PANEL_GAP - dock_size.y,
                );
                let dock_pos = saved_or(
                    &state.settings.hud_positions,
                    "objectives",
                    area,
                    dock_size,
                    dock_default,
                );
                let dock_resp = show_settled(
                    &ctx,
                    dock_id,
                    overlay_area(dock_id).current_pos(dock_pos).movable(edit),
                    |ui| {
                        render_objectives_dock(ui, live_ref, &state.settings, now_secs);
                    },
                );
                remember_drag(
                    &mut state.settings.hud_positions,
                    &mut state.pending_actions.updated_positions,
                    "objectives",
                    &dock_resp.response,
                );
                if edit {
                    interactive_rects_out.push(dock_resp.response.rect);
                }
            }

            // Ally ult reference: top-left, under nothing League draws there.
            let mut ally_bottom = None;
            if state.settings.hud_ally_ult_timers {
                if let Some(live) = live_ref {
                    let mut allies: Vec<_> = live
                        .allied_teammates()
                        .into_iter()
                        .map(|ally| {
                            let left = timer_left(&state.manual_timers, &ally.timer_key, now_secs);
                            (ally, left)
                        })
                        .collect();
                    allies.sort_by_key(|(_, left)| soonest_first(*left));
                    if !allies.is_empty() {
                        let ally_id = Id::unique("riftcore-hud-ally-ults");
                        let ally_resp = show_settled(
                            &ctx,
                            ally_id,
                            overlay_area(ally_id)
                                .fixed_pos(Pos2::new(area.left() + 18.0, area.top() + 116.0)),
                            |ui| {
                                Frame::new()
                                    .fill(panel_fill(state.settings.opacity, 232))
                                    .stroke(Stroke::new(1.0, BORDER))
                                    .corner_radius(CornerRadius::same(10))
                                    .inner_margin(Margin::same(9))
                                    .show(ui, |ui| {
                                        ui.set_min_width(185.0);
                                        ui.label(
                                            RichText::new("ALLY R REFERENCE")
                                                .size(9.5)
                                                .strong()
                                                .color(CYAN),
                                        );
                                        ui.label(
                                            RichText::new("Manual 100s reference")
                                                .size(8.5)
                                                .color(MUTED),
                                        );
                                        for (ally, remaining) in allies {
                                            let btn_label = if remaining > 0 {
                                                format!("{} · RESET", format_clock(remaining))
                                            } else {
                                                "MARK R USED".to_string()
                                            };
                                            let color = if remaining > 0 { RED } else { GREEN };
                                            ui.horizontal(|ui| {
                                                ui.add_sized(
                                                    [70.0, 22.0],
                                                    egui::Label::new(
                                                        RichText::new(format!(
                                                            "{} · {}",
                                                            ally.role, ally.champion_name
                                                        ))
                                                        .size(9.0)
                                                        .color(TEXT),
                                                    ),
                                                );
                                                if ui
                                                    .add_sized(
                                                        [102.0, 22.0],
                                                        egui::Button::new(
                                                            RichText::new(btn_label)
                                                                .size(8.5)
                                                                .strong()
                                                                .color(color),
                                                        ),
                                                    )
                                                    .clicked()
                                                {
                                                    if remaining > 0 {
                                                        state.manual_timers.remove(&ally.timer_key);
                                                    } else {
                                                        state.manual_timers.insert(
                                                            ally.timer_key,
                                                            now_secs + 100.0,
                                                        );
                                                    }
                                                }
                                            });
                                        }
                                    });
                            },
                        );
                        ally_bottom = Some(ally_resp.response.rect.bottom());
                        interactive_rects_out.push(ally_resp.response.rect);
                    }
                }
            }

            if state.settings.hud_aram_health_timers && live_ref.is_some_and(LiveGameState::is_aram)
            {
                // Default: left edge, mid-height (under the ally panel), clear of League's chat.
                let relic_id = Id::unique("riftcore-hud-aram-relics");
                let relic_size = measured_size(&ctx, relic_id, Vec2::new(220.0, 100.0));
                let mut relic_default = default_relic_pos(area);
                if let Some(bottom) = ally_bottom {
                    relic_default.y = relic_default.y.max(bottom + PANEL_GAP);
                }
                let relic_pos = saved_or(
                    &state.settings.hud_positions,
                    "relics",
                    area,
                    relic_size,
                    relic_default,
                );
                let relic_resp = show_settled(
                    &ctx,
                    relic_id,
                    overlay_area(relic_id).current_pos(relic_pos).movable(edit),
                    |ui| {
                        render_aram_relic_timers(
                            ui,
                            &mut state.aram_relic_seconds,
                            &mut state.manual_timers,
                            state.settings.opacity,
                            now_secs,
                        );
                    },
                );
                remember_drag(
                    &mut state.settings.hud_positions,
                    &mut state.pending_actions.updated_positions,
                    "relics",
                    &relic_resp.response,
                );
                interactive_rects_out.push(relic_resp.response.rect);
            }

            // Player panel: top-right, under League's own top-right stats.
            let player_id = Id::unique("riftcore-hud-player");
            let player_size = measured_size(&ctx, player_id, Vec2::new(200.0, 140.0));
            let player_pos = saved_or(
                &state.settings.hud_positions,
                "player",
                area,
                player_size,
                Pos2::new((area.right() - 220.0).max(20.0), area.top() + 98.0),
            );
            let player_resp = show_settled(
                &ctx,
                player_id,
                overlay_area(player_id)
                    .current_pos(player_pos)
                    .movable(edit),
                |ui| {
                    Frame::new()
                        .fill(panel_fill(state.settings.opacity, 212))
                        .stroke(Stroke::new(1.0, BORDER))
                        .inner_margin(Margin::same(10))
                        .show(ui, |ui| {
                            ui.set_min_width(180.0);
                            if state.settings.hud_skill_order {
                                ui.label(
                                    RichText::new("ABILITY RANKS")
                                        .size(10.0)
                                        .strong()
                                        .color(CYAN),
                                );
                                let (q, w, e, r) = live_ref
                                    .map(|g| {
                                        let a = &g.active_player.abilities;
                                        (
                                            a.q.ability_level,
                                            a.w.ability_level,
                                            a.e.ability_level,
                                            a.r.ability_level,
                                        )
                                    })
                                    .unwrap_or((0, 0, 0, 0));
                                ui.horizontal(|ui| {
                                    for (slot, rank) in [("Q", q), ("W", w), ("E", e), ("R", r)] {
                                        ui.label(
                                            RichText::new(format!("{slot} {rank}"))
                                                .size(10.5)
                                                .monospace()
                                                .color(TEXT),
                                        );
                                    }
                                });
                            }
                            if state.settings.hud_item_goal {
                                ui.add_space(5.0);
                                ui.label(
                                    RichText::new("INVENTORY").size(10.0).strong().color(GOLD),
                                );
                                if let Some(live) = live_ref {
                                    let equipped: Vec<_> = live
                                        .active_player
                                        .items
                                        .iter()
                                        .filter(|it| it.item_id > 0)
                                        .take(7)
                                        .collect();
                                    if equipped.is_empty() {
                                        ui.label(
                                            RichText::new("No items yet").size(10.0).color(MUTED),
                                        );
                                    } else {
                                        for item in equipped {
                                            let name = if item.display_name.is_empty() {
                                                items::item_name(item.item_id)
                                            } else {
                                                item.display_name.clone()
                                            };
                                            let text_label = if item.count > 1 {
                                                format!("{name} ×{}", item.count)
                                            } else {
                                                name
                                            };
                                            ui.label(
                                                RichText::new(text_label)
                                                    .size(10.0)
                                                    .color(SECONDARY),
                                            );
                                        }
                                    }
                                }
                            }
                            if state.settings.hud_trinket_reminder
                                && missing_trinket_reminder(live_ref)
                            {
                                ui.add_space(5.0);
                                ui.label(
                                    RichText::new("TRINKET AVAILABLE")
                                        .size(10.0)
                                        .strong()
                                        .color(GOLD),
                                );
                            }
                            // Active-game live coaching gate (LIVE_COACH_ENABLED == false)
                            if LIVE_COACH_ENABLED && state.effective_live_coaching_enabled() {
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new("LIVE COACH INSIGHT")
                                        .size(9.5)
                                        .strong()
                                        .color(CYAN),
                                );
                            }
                        });
                },
            );
            remember_drag(
                &mut state.settings.hud_positions,
                &mut state.pending_actions.updated_positions,
                "player",
                &player_resp.response,
            );
            if edit {
                interactive_rects_out.push(player_resp.response.rect);
            }

            // Toolbar: top-centre under the scoreboard, never over League's ability bar.
            let toolbar_id = Id::unique("riftcore-hud-bottom");
            let toolbar_size = measured_size(&ctx, toolbar_id, Vec2::new(460.0, 38.0));
            let mut toolbar_top = default_toolbar_pos(area);
            if !top_moved {
                toolbar_top.y = toolbar_top
                    .y
                    .max(top_resp.response.rect.bottom() + PANEL_GAP);
            }
            let toolbar_area = match state.settings.hud_positions.get("toolbar") {
                Some(pos) => overlay_area(toolbar_id).current_pos(clamp_logical_panel_pos(
                    *pos,
                    area,
                    toolbar_size,
                )),
                None => overlay_area(toolbar_id)
                    .pivot(egui::Align2::CENTER_TOP)
                    .current_pos(toolbar_top),
            };
            let toolbar_resp = show_settled(&ctx, toolbar_id, toolbar_area.movable(edit), |ui| {
                Frame::new()
                    .fill(panel_fill(state.settings.opacity, 218))
                    .stroke(Stroke::new(1.0, BORDER))
                    .inner_margin(Margin::same(8))
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
                            if !state.passthrough_active {
                                ui.label(
                                    RichText::new("SMART INPUT").size(9.0).strong().color(GOLD),
                                );
                            }
                            if state.settings.hud_mayhem_cards
                                && overlay_button(ui, "MAYHEM CARDS", state.show_mayhem_cards, GOLD)
                                    .clicked()
                            {
                                state.show_mayhem_cards = !state.show_mayhem_cards;
                                if state.show_mayhem_cards {
                                    state.show_build_panel = false;
                                    state.show_timers_panel = false;
                                }
                            }
                            if overlay_button(ui, "BUILD", state.show_build_panel, CYAN).clicked() {
                                state.show_build_panel = !state.show_build_panel;
                                if state.show_build_panel {
                                    state.show_mayhem_cards = false;
                                    state.show_timers_panel = false;
                                }
                            }
                            if overlay_button(ui, "TIMERS", state.show_timers_panel, GOLD).clicked()
                            {
                                state.show_timers_panel = !state.show_timers_panel;
                                if state.show_timers_panel {
                                    state.show_build_panel = false;
                                    state.show_mayhem_cards = false;
                                }
                            }
                            if state.settings.hud_jungle_pathing
                                && live_ref.is_some_and(LiveGameState::is_summoners_rift)
                            {
                                if let Some((step, camp, remaining, active)) =
                                    jungle_route_status(clock, &state.settings.jungle_start_buff)
                                {
                                    ui.label(
                                        RichText::new(format!(
                                            "JG #{step} {camp} {}{}",
                                            if active { "NOW " } else { "IN " },
                                            format_clock(remaining)
                                        ))
                                        .size(9.5)
                                        .color(if active { GREEN } else { SECONDARY }),
                                    );
                                }
                            }
                        });
                    });
            });
            remember_drag(
                &mut state.settings.hud_positions,
                &mut state.pending_actions.updated_positions,
                "toolbar",
                &toolbar_resp.response,
            );
            interactive_rects_out.push(toolbar_resp.response.rect);

            // Drawers share one slot: centred under the toolbar, above League's ability bar.
            // Only one is open at a time (the app can also open the cards drawer directly).
            if state.show_mayhem_cards {
                state.show_build_panel = false;
                state.show_timers_panel = false;
            } else if state.show_build_panel {
                state.show_timers_panel = false;
            }
            let toolbar_bottom = if state.settings.hud_positions.contains_key("toolbar") {
                toolbar_top.y + toolbar_size.y
            } else {
                toolbar_resp.response.rect.bottom()
            };
            let slot = Rect::from_min_max(
                Pos2::new(area.left(), toolbar_bottom + PANEL_GAP),
                Pos2::new(area.right(), ability_bar.top() - PANEL_GAP),
            );
            if state.show_mayhem_cards {
                render_mayhem_drawer(&ctx, slot, state, interactive_rects_out);
            }
            if state.show_build_panel {
                render_build_drawer(&ctx, area, slot, state, interactive_rects_out);
            }
            if state.show_timers_panel {
                render_timers_drawer(&ctx, area, slot, state, interactive_rects_out);
            }
        });
}

/// Body of the objectives dock: soonest objective first, sized by the dock-size setting.
fn render_objectives_dock(
    ui: &mut egui::Ui,
    live: Option<&LiveGameState>,
    settings: &OverlayPanelSettings,
    now_secs: f64,
) {
    let zoom = settings.hud_minimap_zoom.clamp(0.75, 1.35);
    Frame::new()
        .fill(panel_fill(settings.opacity, 212))
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_min_width(150.0 * zoom);
            ui.label(RichText::new("OBJECTIVES").size(10.0).strong().color(CYAN));
            if !settings.hud_objective_timers {
                return;
            }
            let Some(live) = live else {
                return;
            };
            for timer in live.objective_timers_at(now_secs) {
                ui.label(
                    RichText::new(format!(
                        "{:<8} {}",
                        timer.label,
                        timer.formatted_remaining()
                    ))
                    .size((11.0 * zoom).clamp(9.0, 15.0))
                    .monospace()
                    .color(TEXT),
                );
            }
        });
}

/// Where a drawer goes: its saved spot (kept on screen, may use the space down to the
/// bottom), else centred at the top of `slot` and no taller than the slot.
fn drawer_place(
    state: &OverlayRenderState,
    key: &str,
    area: Rect,
    slot: Rect,
    size: Vec2,
) -> (Pos2, f32) {
    match state.settings.hud_positions.get(key) {
        Some(pos) => (clamp_logical_panel_pos(*pos, area, size), area.bottom()),
        None => (
            Pos2::new(
                (slot.center().x - size.x / 2.0).max(slot.left()),
                slot.top(),
            ),
            slot.bottom(),
        ),
    }
}

fn render_aram_relic_timers(
    ui: &mut egui::Ui,
    relic_seconds: &mut String,
    timers: &mut HashMap<String, f64>,
    opacity: f32,
    now: f64,
) {
    /// Longest relic respawn the entry box accepts (anything else is a typo).
    const MAX_RELIC_SECS: u64 = 600;
    Frame::new()
        .fill(panel_fill(opacity, 232))
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.label(
                RichText::new("ARAM HEALTH RELIC TIMERS")
                    .size(10.0)
                    .strong()
                    .color(CYAN),
            );
            ui.label(
                RichText::new("Manual countdown from displayed respawn seconds")
                    .size(8.5)
                    .color(MUTED),
            );
            ui.horizontal(|ui| {
                ui.label(RichText::new("Respawn s").size(9.5).color(SECONDARY));
                ui.add_sized(
                    [72.0, 24.0],
                    egui::TextEdit::singleline(relic_seconds).hint_text("seconds"),
                );
            });
            let mut sides = ["LEFT", "RIGHT"].map(|side| {
                let key = format!("aram-relic-{side}");
                let remaining = timer_left(timers, &key, now);
                (side, key, remaining)
            });
            sides.sort_by_key(|(_, _, remaining)| soonest_first(*remaining));
            ui.horizontal(|ui| {
                for (side, key, remaining) in sides {
                    if remaining > 0 {
                        ui.label(
                            RichText::new(format!("{side} {}", format_clock(remaining)))
                                .size(9.5)
                                .monospace()
                                .color(GREEN),
                        );
                        if overlay_button(ui, "RESET", false, SECONDARY).clicked() {
                            timers.remove(&key);
                        }
                    } else {
                        let secs = relic_seconds
                            .trim()
                            .parse::<u64>()
                            .ok()
                            .filter(|s| (1..=MAX_RELIC_SECS).contains(s));
                        let label = format!("START {side}");
                        if ui
                            .add_enabled(
                                secs.is_some(),
                                egui::Button::new(RichText::new(label).size(9.5).color(GOLD)),
                            )
                            .clicked()
                        {
                            if let Some(s) = secs {
                                timers.insert(key, now + s as f64);
                            }
                        }
                    }
                }
            });
        });
}

/// Cards drawer, centred at the top of `slot` and no taller than it (in compact modes the
/// slot is the whole HUD window).
fn render_mayhem_drawer(
    ctx: &egui::Context,
    slot: Rect,
    state: &mut OverlayRenderState,
    interactive_rects_out: &mut Vec<Rect>,
) {
    const MARGIN: f32 = 12.0;
    let width = 390.0_f32.min(slot.width() - 2.0 * MARGIN - 2.0).max(200.0);
    let pos = Pos2::new(
        (slot.center().x - width / 2.0 - MARGIN - 1.0).max(slot.left()),
        slot.top(),
    );
    let id = Id::unique("riftcore-overlay-mayhem-cards");
    let resp = show_settled(
        ctx,
        id,
        overlay_area(id)
            .order(egui::Order::Foreground)
            .fixed_pos(pos),
        |ui| {
            Frame::new()
                .fill(panel_fill(state.settings.opacity, 248))
                .stroke(Stroke::new(1.0, GOLD))
                .corner_radius(CornerRadius::same(14))
                .inner_margin(Margin::same(MARGIN as i8))
                .show(ui, |ui| {
                    ui.set_width(width);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new("MAYHEM & ARENA")
                                    .size(9.0)
                                    .strong()
                                    .color(GOLD),
                            );
                            ui.label(
                                RichText::new("Augment guide")
                                    .size(16.0)
                                    .strong()
                                    .color(TEXT),
                            );
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if overlay_button(ui, "CLOSE", false, SECONDARY).clicked() {
                                state.show_mayhem_cards = false;
                            }
                        });
                    });
                    ui.horizontal_wrapped(|ui| {
                        for mode in ["Both", "ARAM", "Arena"] {
                            if overlay_button(
                                ui,
                                &mode.to_ascii_uppercase(),
                                state.mayhem_mode.eq_ignore_ascii_case(mode),
                                GOLD,
                            )
                            .clicked()
                            {
                                state.mayhem_mode = mode.to_string();
                            }
                        }
                    });
                    ui.add(
                        egui::TextEdit::singleline(&mut state.mayhem_search)
                            .hint_text("Search augments or champion fit…")
                            .desired_width(ui.available_width()),
                    );
                    ui.horizontal_wrapped(|ui| {
                        for tier in ["ALL", "Prismatic", "Gold", "Silver"] {
                            if overlay_button(
                                ui,
                                &tier.to_ascii_uppercase(),
                                state.mayhem_tier.eq_ignore_ascii_case(tier),
                                CYAN,
                            )
                            .clicked()
                            {
                                state.mayhem_tier = tier.to_string();
                            }
                        }
                    });
                    let filtered = filter_mayhem_cards(
                        &state.mayhem_mode,
                        &state.mayhem_tier,
                        &state.mayhem_search,
                    );
                    // The card list scrolls inside what is left of the slot.
                    let list_height =
                        (slot.bottom() - ui.cursor().top() - MARGIN - 1.0).clamp(80.0, 440.0);
                    ScrollArea::vertical()
                        .id_salt("overlay-mayhem-scroll")
                        .max_height(list_height)
                        .show(ui, |ui| {
                            if filtered.is_empty() {
                                ui.label(
                                    RichText::new("No augments match those filters.")
                                        .size(10.0)
                                        .color(SECONDARY),
                                );
                            }
                            for card in filtered {
                                ui.add_space(4.0);
                                Frame::new()
                                    .fill(CARD_BG)
                                    .stroke(Stroke::new(1.0, BORDER))
                                    .corner_radius(CornerRadius::same(8))
                                    .inner_margin(Margin::same(8))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(&card.name)
                                                    .size(11.0)
                                                    .strong()
                                                    .color(TEXT),
                                            );
                                            ui.label(
                                                RichText::new(card.tier.as_str())
                                                    .size(9.0)
                                                    .color(GOLD),
                                            );
                                            ui.label(
                                                RichText::new(card.mode.as_str())
                                                    .size(8.5)
                                                    .color(CYAN),
                                            );
                                        });
                                        ui.label(
                                            RichText::new(&card.archetype)
                                                .size(9.0)
                                                .color(SECONDARY),
                                        );
                                        ui.label(RichText::new(&card.tip).size(9.5).color(TEXT));
                                    });
                            }
                        });
                });
        },
    );
    interactive_rects_out.push(resp.response.rect);
}

fn render_build_drawer(
    ctx: &egui::Context,
    area: Rect,
    slot: Rect,
    state: &mut OverlayRenderState,
    interactive_rects_out: &mut Vec<Rect>,
) {
    let Some(live) = state.live_game.as_ref() else {
        return;
    };
    let Some((champ, build)) = resolve_personal_build(live) else {
        return;
    };

    let id = Id::unique("riftcore-overlay-build");
    let size = measured_size(ctx, id, Vec2::new(318.0, 250.0));
    let (pos, bottom_limit) = drawer_place(state, "build", area, slot, size);
    let current_gold = live.active_player.current_gold.max(0.0) as u64;
    let mut export = false;
    let mut close = false;
    let area_builder = overlay_area(id)
        .current_pos(pos)
        .movable(state.settings.hud_edit_layout);
    let resp = show_settled(ctx, id, area_builder, |ui| {
        Frame::new()
            .fill(panel_fill(state.settings.opacity, 246))
            .stroke(Stroke::new(1.0, CYAN))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.set_width(292.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("PERSONAL BUILD")
                                .size(9.0)
                                .strong()
                                .color(CYAN),
                        );
                        ui.label(
                            RichText::new(format!("{} · {}", champ.name, build.role))
                                .size(15.0)
                                .strong()
                                .color(TEXT),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        close = overlay_button(ui, "×", false, RED).clicked();
                    });
                });
                if let Some(&target_item) = build.core_items.get(1).or(build.core_items.first()) {
                    ui.label(
                        RichText::new(format!(
                            "NEXT CORE TARGET  ·  {}  ·  {current_gold}g held",
                            items::item_name(target_item)
                        ))
                        .size(10.0)
                        .strong()
                        .color(GREEN),
                    );
                }
                ui.add_space(6.0);
                let starting_text = build
                    .starting_items
                    .iter()
                    .map(|&id| items::item_name(id))
                    .collect::<Vec<_>>()
                    .join("  ·  ");
                ui.label(
                    RichText::new(format!("STARTING  {starting_text}"))
                        .size(9.5)
                        .color(SECONDARY),
                );
                // Leave room under the list for the skill order and sync button.
                let list_height = (bottom_limit - ui.cursor().top() - 64.0).clamp(60.0, 168.0);
                ScrollArea::vertical()
                    .id_salt("overlay-live-build-items")
                    .max_height(list_height)
                    .show(ui, |ui| {
                        ui.add_space(6.0);
                        ui.label(RichText::new("CORE BUILD").size(9.0).strong().color(GOLD));
                        for (idx, &id) in build.core_items.iter().enumerate() {
                            ui.label(
                                RichText::new(format!("{}. {}", idx + 1, items::item_name(id)))
                                    .size(10.0)
                                    .color(TEXT),
                            );
                        }
                        ui.add_space(5.0);
                        ui.label(RichText::new("SITUATIONAL").size(9.0).strong().color(GOLD));
                        for &id in &build.situational_items {
                            ui.label(
                                RichText::new(items::item_name(id))
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
                if overlay_button(ui, "SYNC BUILD TO LEAGUE SHOP", false, CYAN).clicked() {
                    export = true;
                }
                if LIVE_COACH_ENABLED && state.effective_live_coaching_enabled() {
                    ui.label(RichText::new(&build.tip).size(9.0).color(MUTED));
                }
            });
    });
    interactive_rects_out.push(resp.response.rect);
    remember_drag(
        &mut state.settings.hud_positions,
        &mut state.pending_actions.updated_positions,
        "build",
        &resp.response,
    );
    if close {
        state.show_build_panel = false;
    }
    if export {
        let item_set = builds::champion_item_set(champ, &build).to_json();
        state.pending_actions.export_item_set = Some(item_set);
        state.pending_actions.notice =
            Some("Sending recommended build to League shop…".to_string());
    }
}

fn render_timers_drawer(
    ctx: &egui::Context,
    area: Rect,
    slot: Rect,
    state: &mut OverlayRenderState,
    interactive_rects_out: &mut Vec<Rect>,
) {
    let now = state.clock_secs as f64;
    let id = Id::unique("riftcore-overlay-timers");
    let size = measured_size(ctx, id, Vec2::new(414.0, 260.0));
    let (pos, bottom_limit) = drawer_place(state, "timers", area, slot, size);
    let mut copy_callout = false;
    let mut close = false;
    let area_builder = overlay_area(id)
        .current_pos(pos)
        .movable(state.settings.hud_edit_layout);
    let resp = show_settled(ctx, id, area_builder, |ui| {
        Frame::new()
            .fill(panel_fill(state.settings.opacity, 246))
            .stroke(Stroke::new(1.0, GOLD))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::same(11))
            .show(ui, |ui| {
                ui.set_width(390.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("LIVE TIMERS").size(9.0).strong().color(GOLD));
                        ui.label(
                            RichText::new("Manual camp timers & SR objectives")
                                .size(9.0)
                                .color(MUTED),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        close = overlay_button(ui, "×", false, RED).clicked();
                        if overlay_button(ui, "COPY CALL-OUT", false, CYAN).clicked() {
                            copy_callout = true;
                        }
                    });
                });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if state.settings.hud_manual_camp_timers
                        && overlay_button(
                            ui,
                            "CAMPS",
                            state.timer_tab == OverlayTimerTab::Camps,
                            GOLD,
                        )
                        .clicked()
                    {
                        state.timer_tab = OverlayTimerTab::Camps;
                    }
                    if state.settings.hud_objective_timers
                        && overlay_button(
                            ui,
                            "OBJECTIVES",
                            state.timer_tab == OverlayTimerTab::Objectives,
                            GOLD,
                        )
                        .clicked()
                    {
                        state.timer_tab = OverlayTimerTab::Objectives;
                    }
                });
                ui.separator();
                // The list scrolls inside what is left of the drawer's slot.
                let max_height = (bottom_limit - ui.cursor().top() - 12.0).clamp(80.0, 360.0);
                ScrollArea::vertical()
                    .id_salt("riftcore-overlay-timer-list")
                    .max_height(max_height)
                    .show(ui, |ui| match state.timer_tab {
                        OverlayTimerTab::Camps if state.settings.hud_manual_camp_timers => {
                            render_manual_camp_timers(ui, &mut state.manual_timers, now);
                        }
                        OverlayTimerTab::Objectives if state.settings.hud_objective_timers => {
                            if let Some(live) = state.live_game.as_ref() {
                                let timers = live.objective_timers_at(now);
                                if timers.is_empty() {
                                    ui.label(
                                        RichText::new(
                                            "Objective timers activate on Summoner's Rift.",
                                        )
                                        .size(10.0)
                                        .color(SECONDARY),
                                    );
                                } else {
                                    for timer in timers {
                                        ui.horizontal(|ui| {
                                            ui.add_sized(
                                                [120.0, 26.0],
                                                egui::Label::new(
                                                    RichText::new(&timer.label)
                                                        .size(10.0)
                                                        .strong()
                                                        .color(GOLD),
                                                ),
                                            );
                                            ui.label(
                                                RichText::new(timer.formatted_remaining())
                                                    .size(11.0)
                                                    .monospace()
                                                    .strong()
                                                    .color(if timer.ready { GREEN } else { TEXT }),
                                            );
                                        });
                                    }
                                }
                            }
                        }
                        _ => {
                            ui.label(
                                RichText::new("Enable a timer section in Settings → Overlay.")
                                    .size(10.0)
                                    .color(SECONDARY),
                            );
                        }
                    });
            });
    });
    interactive_rects_out.push(resp.response.rect);
    remember_drag(
        &mut state.settings.hud_positions,
        &mut state.pending_actions.updated_positions,
        "timers",
        &resp.response,
    );
    if close {
        state.show_timers_panel = false;
    }
    if copy_callout {
        // (seconds left, text), soonest first.
        let mut callouts = Vec::new();
        if let Some(live) = state.live_game.as_ref() {
            for timer in live.objective_timers_at(now) {
                callouts.push((
                    timer.remaining_secs,
                    format!(
                        "{} {}",
                        timer.label.to_ascii_lowercase(),
                        timer.formatted_remaining()
                    ),
                ));
            }
        }
        for (camp, key) in [
            ("blue", "jungle-camp-blue-buff"),
            ("red", "jungle-camp-red-buff"),
        ] {
            let left = timer_left(&state.manual_timers, key, now);
            if left > 0 {
                // Call out the game time the camp respawns at.
                callouts.push((left, format!("{camp} {}", format_clock(now as u64 + left))));
            }
        }
        callouts.sort_by_key(|(left, _)| *left);
        let message = if callouts.is_empty() {
            "[RiftCore] No active timers".to_string()
        } else {
            let texts: Vec<_> = callouts.into_iter().map(|(_, text)| text).collect();
            format!("[RiftCore] {}", texts.join(" | "))
        };
        ctx.copy_text(message.clone());
        state.pending_actions.copy_clipboard = Some(message);
        state.pending_actions.notice = Some("Timer call-out copied to clipboard".to_string());
    }
}

fn render_manual_camp_timers(ui: &mut egui::Ui, timers: &mut HashMap<String, f64>, now: f64) {
    ui.label(
        RichText::new("Click a camp when cleared to start its manual respawn timer.")
            .size(9.5)
            .color(SECONDARY),
    );
    ui.add_space(5.0);
    for (camp, duration, key, remaining) in manual_camp_rows(timers, now) {
        ui.horizontal(|ui| {
            ui.add_sized(
                [115.0, 28.0],
                egui::Label::new(RichText::new(camp).size(9.5).strong().color(TEXT)),
            );
            let label = if remaining > 0 {
                format!("RESPAWNS  {}  ·  RESET", format_clock(remaining))
            } else {
                format!("START  {}", format_clock(duration))
            };
            if ui
                .add_sized(
                    [235.0, 28.0],
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
                    timers.insert(key, now + duration as f64);
                }
            }
        });
    }
}

/// Manual camp rows `(camp, respawn secs, timer key, secs left)`: respawning camps first,
/// soonest first, then the idle camps in map order.
fn manual_camp_rows(
    timers: &HashMap<String, f64>,
    now: f64,
) -> Vec<(&'static str, u64, String, u64)> {
    const CAMPS: [(&str, u64); 7] = [
        ("BLUE BUFF", 300),
        ("GROMP", 135),
        ("WOLVES", 135),
        ("RAPTORS", 135),
        ("RED BUFF", 300),
        ("KRUGS", 135),
        ("SCUTTLE", 150),
    ];
    let mut rows: Vec<_> = CAMPS
        .into_iter()
        .map(|(camp, duration)| {
            let key = format!(
                "jungle-camp-{}",
                camp.to_ascii_lowercase().replace(' ', "-")
            );
            let remaining = timer_left(timers, &key, now);
            (camp, duration, key, remaining)
        })
        .collect();
    rows.sort_by_key(|row| soonest_first(row.3));
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overlay::hud_modes::{
        compute_overlay_rect_px, hud_pixels_per_point, league_chat_zone, DisplayPresentationMode,
        TargetWindowState,
    };
    use serde_json::json;

    /// A busy late-game snapshot: big numbers, full inventory, four allies.
    fn busy_game(mode: &str, map: u32) -> LiveGameState {
        let item = |id: u32, slot: i32, name: &str| json!({ "itemID": id, "slot": slot, "count": 1, "displayName": name });
        let player = |name: &str, champ: &str, team: &str, role: &str| {
            json!({ "summonerName": name, "championName": champ, "team": team,
                    "position": role, "scores": { "kills": 9 } })
        };
        LiveGameState::from_allgamedata(&json!({
            "gameData": { "gameTime": 1985.4, "gameMode": mode, "mapNumber": map },
            "activePlayer": {
                "summonerName": "PlayerOne", "championName": "Ahri", "level": 18,
                "currentGold": 12345.0,
                "abilities": { "Q": {"abilityLevel": 5}, "W": {"abilityLevel": 5},
                               "E": {"abilityLevel": 5}, "R": {"abilityLevel": 2} },
                "scores": { "kills": 25, "deaths": 15, "assists": 30, "creepScore": 450,
                            "wardScore": 88.0 },
                "items": [
                    item(6655, 0, "Luden's Companion"), item(3089, 1, "Rabadon's Deathcap"),
                    item(3157, 2, "Zhonya's Hourglass"), item(3135, 3, "Void Staff"),
                    item(3165, 4, "Morellonomicon"), item(3020, 5, "Sorcerer's Shoes"),
                    item(2003, 6, "Health Potion")
                ]
            },
            "allPlayers": [
                player("PlayerOne", "Ahri", "ORDER", "MIDDLE"),
                player("A1", "Aurelion Sol", "ORDER", "TOP"),
                player("A2", "Nunu & Willump", "ORDER", "JUNGLE"),
                player("A3", "Miss Fortune", "ORDER", "BOTTOM"),
                player("A4", "Nautilus", "ORDER", "UTILITY"),
                player("E1", "Garen", "CHAOS", "TOP")
            ],
            "events": { "Events": [
                { "EventName": "DragonKill", "EventTime": 1500.0, "DragonType": "Fire" }
            ] }
        }))
        .expect("valid payload")
    }

    /// Renders one HUD frame headlessly on a `screen` (points) at `ppp` and returns the
    /// interactive rects `render_overlay_ui` reports.
    fn frame(
        ctx: &egui::Context,
        state: &mut OverlayRenderState,
        screen: Vec2,
        ppp: f32,
    ) -> Vec<Rect> {
        let mut raw = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, screen)),
            ..Default::default()
        };
        if let Some(viewport) = raw.viewports.get_mut(&egui::ViewportId::ROOT) {
            viewport.native_pixels_per_point = Some(ppp);
        }
        let mut rects = Vec::new();
        ctx.run_ui(raw, |ui| render_overlay_ui(ui, state, &mut rects))
            .drop_without_applying_deltas();
        rects
    }

    fn overlaps(a: Rect, b: Rect) -> bool {
        a.shrink(0.5).intersects(b.shrink(0.5))
    }

    /// Game resolutions with Windows scaling (DPI).
    const DISPLAYS: [([i32; 2], u32); 9] = [
        ([1280, 720], 96),
        ([1600, 900], 120),
        ([1920, 1080], 96),
        ([1920, 1080], 144),
        ([1920, 1080], 192),
        ([2560, 1440], 120),
        ([2560, 1440], 192),
        ([3840, 2160], 144),
        ([3840, 2160], 192),
    ];

    /// Integrated HUD canvas (points): the game minus the 1 px inset, at the HUD's scale.
    fn canvas(display: ([i32; 2], u32)) -> (Vec2, f32) {
        let ([w, h], dpi) = display;
        let ppp = hud_pixels_per_point(dpi, [w, h]);
        (Vec2::new((w - 1) as f32 / ppp, (h - 1) as f32 / ppp), ppp)
    }

    #[test]
    fn integrated_panels_avoid_each_other_and_league_hud_at_every_resolution() {
        for display in DISPLAYS {
            let (size, ppp) = canvas(display);
            let screen = Rect::from_min_size(Pos2::ZERO, size);
            for (aram, flip, drawer, big_map) in [
                (false, false, 0, false),
                (false, false, 1, false),
                (false, false, 2, false),
                (false, false, 3, false),
                (false, true, 0, false),
                (false, true, 3, false),
                (true, false, 0, false),
                (true, true, 1, false),
                // Largest map scale and dock size the settings sliders allow.
                (false, false, 0, true),
                (false, true, 0, true),
                (false, false, 3, true),
            ] {
                let mut state = OverlayRenderState::default();
                state.apply_live_game(Some(if aram {
                    busy_game("ARAM", 12)
                } else {
                    busy_game("CLASSIC", 11)
                }));
                // Edit mode reports every panel as interactive; passthrough off shows the
                // widest toolbar.
                state.settings.hud_edit_layout = true;
                state.settings.hud_minimap_flip = flip;
                if big_map {
                    state.settings.hud_minimap_scale = 3.0;
                    state.settings.hud_minimap_zoom = 1.35;
                }
                state.passthrough_active = false;
                state.show_mayhem_cards = drawer == 1;
                state.show_build_panel = drawer == 2;
                state.show_timers_panel = drawer == 3;
                let ctx = egui::Context::default();
                let first = frame(&ctx, &mut state, size, ppp);
                let rects = frame(&ctx, &mut state, size, ppp);
                let tag = format!(
                    "{display:?} aram={aram} flip={flip} drawer={drawer} big_map={big_map}"
                );
                assert_eq!(first, rects, "first frame already final: {tag}");
                // scoreboard, ally ults, player panel, toolbar + dock (SR) or relics (ARAM).
                let panels = 5 + usize::from(drawer > 0);
                assert_eq!(rects.len(), panels, "{tag}: {rects:?}");

                let zones = [
                    ("ability bar", league_ability_bar_zone(screen)),
                    ("minimap", minimap_square(screen, &state.settings)),
                    ("chat", league_chat_zone(screen, flip)),
                ];
                for (i, rect) in rects.iter().enumerate() {
                    assert!(
                        screen.expand(0.5).contains_rect(*rect),
                        "{tag}: {rect:?} off screen"
                    );
                    for (name, zone) in zones {
                        assert!(
                            !overlaps(*rect, zone),
                            "{tag}: {rect:?} covers League's {name} {zone:?}"
                        );
                    }
                    for other in &rects[i + 1..] {
                        assert!(
                            !overlaps(*rect, *other),
                            "{tag}: {rect:?} overlaps {other:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn aram_mayhem_gets_the_same_panels_as_aram() {
        let (size, ppp) = canvas(([1920, 1080], 96));
        let rects_for = |mode: &str| {
            let mut state = OverlayRenderState::default();
            state.apply_live_game(Some(busy_game(mode, 12)));
            state.settings.hud_edit_layout = true;
            state.passthrough_active = false;
            let ctx = egui::Context::default();
            frame(&ctx, &mut state, size, ppp);
            frame(&ctx, &mut state, size, ppp)
        };
        let aram = rects_for("ARAM");
        // Scoreboard, ally ults, player panel, toolbar and the health relic timers.
        assert_eq!(aram.len(), 5, "{aram:?}");
        // ARAM Mayhem reports gameMode "KIWI".
        assert_eq!(rects_for("KIWI"), aram);
    }

    #[test]
    fn compact_modes_fit_their_window_and_keep_clear_of_league_hud() {
        for display in DISPLAYS {
            let ([w, h], dpi) = display;
            let target = TargetWindowState {
                hwnd: 1,
                client_rect_px: [0, 0, w, h],
                monitor_dpi: dpi,
                is_foreground: true,
                presentation: DisplayPresentationMode::ExclusiveFullscreen,
            };
            let ppp = target.hud_pixels_per_point();
            let game = Rect::from_min_size(Pos2::ZERO, Vec2::new(w as f32, h as f32) / ppp);
            let settings = OverlayPanelSettings::default();
            let zones = [
                league_ability_bar_zone(game),
                minimap_square(game, &settings),
                league_chat_zone(game, false),
            ];
            for mode in [
                HudPresentationMode::Bar,
                HudPresentationMode::Pill,
                HudPresentationMode::Card,
            ] {
                for cards in [false, true] {
                    let mut state = OverlayRenderState::default();
                    state.apply_live_game(Some(busy_game("CLASSIC", 11)));
                    state.settings.hud_mode = mode;
                    state.show_mayhem_cards = cards;
                    let px = compute_overlay_rect_px(&target, None, mode, cards);
                    let window = Rect::from_min_max(
                        Pos2::new(px[0] as f32, px[1] as f32) / ppp,
                        Pos2::new(px[2] as f32, px[3] as f32) / ppp,
                    );
                    let ctx = egui::Context::default();
                    frame(&ctx, &mut state, window.size(), ppp);
                    let rects = frame(&ctx, &mut state, window.size(), ppp);
                    let tag = format!("{display:?} {mode:?} cards={cards}");
                    assert_eq!(rects.len(), 1, "{tag}: one panel at a time");
                    let local = Rect::from_min_size(Pos2::ZERO, window.size());
                    // Nothing clipped: the stat readouts and buttons fit inside the window.
                    assert!(
                        local.expand(0.5).contains_rect(rects[0]),
                        "{tag}: {rects:?}"
                    );
                    for zone in zones {
                        assert!(
                            !overlaps(window, zone),
                            "{tag}: window {window:?} vs {zone:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn saved_positions_stay_on_screen_after_a_resolution_change() {
        let (size, ppp) = canvas(([1280, 720], 96));
        let screen = Rect::from_min_size(Pos2::ZERO, size);
        for saved in [[3200.0, 1900.0], [-400.0, -250.0], [1200.0, 700.0]] {
            let mut state = OverlayRenderState::default();
            state.apply_live_game(Some(busy_game("CLASSIC", 11)));
            state.settings.hud_edit_layout = true;
            for key in [
                "scoreboard",
                "objectives",
                "player",
                "toolbar",
                "build",
                "timers",
            ] {
                state.settings.hud_positions.insert(key.to_string(), saved);
            }
            for drawer in [2, 3] {
                state.show_build_panel = drawer == 2;
                state.show_timers_panel = drawer == 3;
                let ctx = egui::Context::default();
                frame(&ctx, &mut state, size, ppp);
                let rects = frame(&ctx, &mut state, size, ppp);
                assert_eq!(rects.len(), 6, "{saved:?}");
                for rect in rects {
                    assert!(
                        screen.expand(0.5).contains_rect(rect),
                        "{saved:?}: {rect:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn manual_timers_follow_the_game_clock_and_list_soonest_first() {
        let mut timers = HashMap::new();
        timers.insert("jungle-camp-krugs".to_string(), 600.0 + 135.0);
        timers.insert("jungle-camp-blue-buff".to_string(), 540.0 + 300.0);
        let rows = |timers: &HashMap<String, f64>, now: f64| {
            manual_camp_rows(timers, now)
                .into_iter()
                .map(|(camp, _, _, left)| (camp, left))
                .collect::<Vec<_>>()
        };
        let at_600 = rows(&timers, 600.0);
        assert_eq!(at_600[..2], [("KRUGS", 135), ("BLUE BUFF", 240)]);
        // Idle camps follow in map order.
        let idle: Vec<_> = at_600[2..].iter().map(|row| row.0).collect();
        assert_eq!(idle, ["GROMP", "WOLVES", "RAPTORS", "RED BUFF", "SCUTTLE"]);
        // Last second reads 00:01; at respawn the camp is idle again (never negative).
        assert_eq!(rows(&timers, 734.0)[0], ("KRUGS", 1));
        assert_eq!(rows(&timers, 735.0)[0], ("BLUE BUFF", 105));
        assert!(rows(&timers, 5000.0).iter().all(|row| row.1 == 0));
        assert_eq!(timer_left(&timers, "jungle-camp-krugs", f64::NAN), 0);

        // The panels drop finished timers and count on the HUD's game clock.
        let mut state = OverlayRenderState::default();
        state.apply_live_game(Some(busy_game("CLASSIC", 11)));
        state.manual_timers = timers;
        state
            .manual_timers
            .insert("ally-ult-A1".to_string(), 1985.0 + 40.0);
        let ctx = egui::Context::default();
        frame(&ctx, &mut state, Vec2::new(1919.0, 1079.0), 1.0);
        assert_eq!(state.clock_secs, 1985);
        assert_eq!(
            state.manual_timers.len(),
            1,
            "finished camp timers are dropped"
        );
        assert_eq!(timer_left(&state.manual_timers, "ally-ult-A1", 1985.0), 40);
    }
}
