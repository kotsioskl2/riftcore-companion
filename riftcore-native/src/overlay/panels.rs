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
use crate::domain::live_game::LiveGameState;
use crate::domain::mayhem::{self, CardMode, CardTier, MayhemCard};
use crate::domain::protocol::LIVE_COACH_ENABLED;
use crate::overlay::host::{OverlayRenderState, OverlayTimerTab};
use crate::overlay::hud_modes::{clamp_logical_panel_pos, HudPresentationMode};
use crate::overlay::minimap::{jungle_route_status, render_minimap_guide_pins};
use eframe::egui::{
    self, Align, Color32, CornerRadius, Frame, Id, Layout, Margin, Rect, RichText, ScrollArea,
    Stroke, Vec2,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};

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
            let live_ref = state.live_game.as_ref();
            let clock = live_ref
                .map(|g| g.game_time_secs.max(0.0) as u64)
                .unwrap_or(state.clock_secs);
            let scores = live_ref.map(|g| &g.active_player.scores);
            let (kills, deaths, assists, creep_score) = scores
                .map(|s| (s.kills, s.deaths, s.assists, s.creep_score))
                .unwrap_or((0, 0, 0, 0));
            let kda = format!("{kills} / {deaths} / {assists}");
            let cs_per_minute = creep_score as f64 / (clock as f64 / 60.0).max(1.0);
            let current_gold = live_ref
                .map(|g| g.active_player.current_gold.max(0.0) as u64)
                .unwrap_or(0);
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
                                            .color(GOLD),
                                    );
                                }
                                if state.settings.hud_gold_lead {
                                    ui.label(
                                        RichText::new(format!("{current_gold} G"))
                                            .size(10.0)
                                            .color(GREEN),
                                    );
                                }
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
                    if state.show_mayhem_cards {
                        render_mayhem_drawer(&ctx, area, state, interactive_rects_out);
                    }
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
                                ui.label(RichText::new(&kda).size(11.0).strong().color(TEXT));
                                ui.label(
                                    RichText::new(format!("{creep_score} CS"))
                                        .size(10.0)
                                        .color(GOLD),
                                );
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
                    if state.show_mayhem_cards {
                        render_mayhem_drawer(&ctx, area, state, interactive_rects_out);
                    }
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
                                    GOLD,
                                );
                            });
                            if state.settings.hud_gold_lead {
                                ui.label(
                                    RichText::new(format!("CURRENT GOLD   {current_gold}"))
                                        .size(11.0)
                                        .color(GREEN),
                                );
                            }
                            if state.settings.hud_objective_timers {
                                if let Some(live) = live_ref {
                                    for timer in live.objective_timers() {
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
                    if state.show_mayhem_cards {
                        render_mayhem_drawer(&ctx, area, state, interactive_rects_out);
                    }
                    return;
                }
                HudPresentationMode::Integrated => {}
            }

            // Integrated full-viewport HUD
            render_minimap_guide_pins(&ctx, area, live_ref, &state.settings, clock);

            let top_default = [area.center().x - 285.0, area.top() + 20.0];
            let top_pos = clamp_logical_panel_pos(
                state
                    .settings
                    .hud_positions
                    .get("scoreboard")
                    .copied()
                    .unwrap_or(top_default),
                area,
                Vec2::new(420.0, 56.0),
            );
            let top_resp = egui::Area::new(Id::unique("riftcore-hud-top"))
                .current_pos(top_pos)
                .movable(state.settings.hud_edit_layout)
                .show(&ctx, |ui| {
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
                                ui.label(RichText::new(&kda).size(13.0).strong().color(TEXT));
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
                                            .color(GOLD),
                                    );
                                }
                                if state.settings.hud_gold_lead {
                                    ui.separator();
                                    ui.label(
                                        RichText::new(format!("{current_gold} G"))
                                            .size(11.0)
                                            .color(GREEN),
                                    );
                                }
                            });
                            ui.label(RichText::new(game_mode).size(9.0).strong().color(MUTED));
                        });
                });
            if top_resp.response.drag_stopped() {
                let pos = [top_resp.response.rect.min.x, top_resp.response.rect.min.y];
                state
                    .settings
                    .hud_positions
                    .insert("scoreboard".to_string(), pos);
                state
                    .pending_actions
                    .updated_positions
                    .push(("scoreboard".to_string(), pos));
            }
            if state.settings.hud_edit_layout {
                interactive_rects_out.push(top_resp.response.rect);
            }

            let dock_default = if state.settings.hud_minimap_flip {
                [
                    area.left() + 16.0 + state.settings.hud_minimap_nudge_x,
                    area.bottom() - 150.0 + state.settings.hud_minimap_nudge_y,
                ]
            } else {
                [
                    area.right() - 175.0 + state.settings.hud_minimap_nudge_x,
                    area.bottom() - 150.0 + state.settings.hud_minimap_nudge_y,
                ]
            };
            let dock_pos = clamp_logical_panel_pos(
                state
                    .settings
                    .hud_positions
                    .get("objectives")
                    .copied()
                    .unwrap_or(dock_default),
                area,
                Vec2::new(165.0, 110.0),
            );
            let dock_resp = egui::Area::new(Id::unique("riftcore-hud-objectives"))
                .current_pos(dock_pos)
                .movable(state.settings.hud_edit_layout)
                .show(&ctx, |ui| {
                    Frame::new()
                        .fill(panel_fill(state.settings.opacity, 212))
                        .stroke(Stroke::new(1.0, BORDER))
                        .inner_margin(Margin::same(10))
                        .show(ui, |ui| {
                            ui.set_min_width(
                                150.0 * state.settings.hud_minimap_zoom.clamp(0.75, 1.35),
                            );
                            ui.label(RichText::new("OBJECTIVES").size(10.0).strong().color(CYAN));
                            if state.settings.hud_objective_timers {
                                if let Some(live) = live_ref {
                                    let timers = live.objective_timers();
                                    if timers.is_empty() {
                                        ui.label(RichText::new("SR only").size(9.5).color(MUTED));
                                    } else {
                                        for timer in timers {
                                            ui.label(
                                                RichText::new(format!(
                                                    "{:<8} {}",
                                                    timer.label,
                                                    timer.formatted_remaining()
                                                ))
                                                .size(
                                                    (11.0 * state.settings.hud_minimap_zoom)
                                                        .clamp(9.0, 15.0),
                                                )
                                                .monospace()
                                                .color(TEXT),
                                            );
                                        }
                                    }
                                }
                            }
                        });
                });
            if dock_resp.response.drag_stopped() {
                let pos = [dock_resp.response.rect.min.x, dock_resp.response.rect.min.y];
                state
                    .settings
                    .hud_positions
                    .insert("objectives".to_string(), pos);
                state
                    .pending_actions
                    .updated_positions
                    .push(("objectives".to_string(), pos));
            }
            if state.settings.hud_edit_layout {
                interactive_rects_out.push(dock_resp.response.rect);
            }

            if state.settings.hud_aram_health_timers && game_mode.eq_ignore_ascii_case("ARAM") {
                let relic_resp = egui::Area::new(Id::unique("riftcore-hud-aram-relics"))
                    .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(20.0, -110.0))
                    .show(&ctx, |ui| {
                        render_aram_relic_timers(
                            ui,
                            &mut state.aram_relic_seconds,
                            &mut state.manual_timers,
                            state.settings.opacity,
                        );
                    });
                interactive_rects_out.push(relic_resp.response.rect);
            }

            if state.settings.hud_ally_ult_timers {
                if let Some(live) = live_ref {
                    let allies = live.allied_teammates();
                    if !allies.is_empty() {
                        let ally_resp = egui::Area::new(Id::unique("riftcore-hud-ally-ults"))
                            .fixed_pos(egui::pos2(area.left() + 18.0, area.top() + 116.0))
                            .show(&ctx, |ui| {
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
                                        let now = Instant::now();
                                        for ally in allies {
                                            let remaining = state
                                                .manual_timers
                                                .get(&ally.timer_key)
                                                .map(|end| {
                                                    end.saturating_duration_since(now).as_secs()
                                                })
                                                .unwrap_or(0);
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
                                                            now + Duration::from_secs(100),
                                                        );
                                                    }
                                                }
                                            });
                                        }
                                    });
                            });
                        interactive_rects_out.push(ally_resp.response.rect);
                    }
                }
            }

            let player_default = [(area.right() - 220.0).max(20.0), area.top() + 98.0];
            let player_pos = clamp_logical_panel_pos(
                state
                    .settings
                    .hud_positions
                    .get("player")
                    .copied()
                    .unwrap_or(player_default),
                area,
                Vec2::new(200.0, 140.0),
            );
            let player_resp = egui::Area::new(Id::unique("riftcore-hud-player"))
                .current_pos(player_pos)
                .movable(state.settings.hud_edit_layout)
                .show(&ctx, |ui| {
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
                });
            if player_resp.response.drag_stopped() {
                let pos = [
                    player_resp.response.rect.min.x,
                    player_resp.response.rect.min.y,
                ];
                state
                    .settings
                    .hud_positions
                    .insert("player".to_string(), pos);
                state
                    .pending_actions
                    .updated_positions
                    .push(("player".to_string(), pos));
            }
            if state.settings.hud_edit_layout {
                interactive_rects_out.push(player_resp.response.rect);
            }

            let toolbar_resp = egui::Area::new(Id::unique("riftcore-hud-bottom"))
                .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -18.0))
                .show(&ctx, |ui| {
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
                                    && overlay_button(
                                        ui,
                                        "MAYHEM CARDS",
                                        state.show_mayhem_cards,
                                        GOLD,
                                    )
                                    .clicked()
                                {
                                    state.show_mayhem_cards = !state.show_mayhem_cards;
                                    if state.show_mayhem_cards {
                                        state.show_build_panel = false;
                                    }
                                }
                                if overlay_button(ui, "BUILD", state.show_build_panel, CYAN)
                                    .clicked()
                                {
                                    state.show_build_panel = !state.show_build_panel;
                                    if state.show_build_panel {
                                        state.show_mayhem_cards = false;
                                        state.show_timers_panel = false;
                                    }
                                }
                                if overlay_button(ui, "TIMERS", state.show_timers_panel, GOLD)
                                    .clicked()
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
                                        jungle_route_status(
                                            clock,
                                            &state.settings.jungle_start_buff,
                                        )
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
            interactive_rects_out.push(toolbar_resp.response.rect);

            if state.show_mayhem_cards {
                render_mayhem_drawer(&ctx, area, state, interactive_rects_out);
            }
            if state.show_build_panel {
                render_build_drawer(&ctx, area, state, interactive_rects_out);
            }
            if state.show_timers_panel {
                render_timers_drawer(&ctx, area, state, clock, interactive_rects_out);
            }
        });
}

fn render_aram_relic_timers(
    ui: &mut egui::Ui,
    relic_seconds: &mut String,
    timers: &mut HashMap<String, Instant>,
    opacity: f32,
) {
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
            let now = Instant::now();
            ui.horizontal(|ui| {
                for side in ["LEFT", "RIGHT"] {
                    let key = format!("aram-relic-{side}");
                    let remaining = timers
                        .get(&key)
                        .map(|end| end.saturating_duration_since(now).as_secs())
                        .unwrap_or(0);
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
                        let secs = relic_seconds.trim().parse::<u64>().ok().filter(|s| *s > 0);
                        let label = format!("START {side}");
                        if ui
                            .add_enabled(
                                secs.is_some(),
                                egui::Button::new(RichText::new(label).size(9.5).color(GOLD)),
                            )
                            .clicked()
                        {
                            if let Some(s) = secs {
                                timers.insert(key, now + Duration::from_secs(s));
                            }
                        }
                    }
                }
            });
        });
}

fn render_mayhem_drawer(
    ctx: &egui::Context,
    area: Rect,
    state: &mut OverlayRenderState,
    interactive_rects_out: &mut Vec<Rect>,
) {
    let pos = egui::pos2(
        (area.right() - 412.0).max(area.left() + 8.0),
        area.top() + 8.0,
    );
    let resp = egui::Area::new(Id::unique("riftcore-overlay-mayhem-cards"))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            Frame::new()
                .fill(panel_fill(state.settings.opacity, 248))
                .stroke(Stroke::new(1.0, GOLD))
                .corner_radius(CornerRadius::same(14))
                .inner_margin(Margin::same(12))
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
                    ScrollArea::vertical()
                        .id_salt("overlay-mayhem-scroll")
                        .max_height((area.height() - 210.0).clamp(160.0, 440.0))
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
        });
    interactive_rects_out.push(resp.response.rect);
}

fn render_build_drawer(
    ctx: &egui::Context,
    area: Rect,
    state: &mut OverlayRenderState,
    interactive_rects_out: &mut Vec<Rect>,
) {
    let Some(live) = state.live_game.as_ref() else {
        return;
    };
    let Some((champ, build)) = resolve_personal_build(live) else {
        return;
    };

    let default_pos = [
        (area.right() - 318.0).max(area.left() + 8.0),
        area.top() + 260.0,
    ];
    let pos = clamp_logical_panel_pos(
        state
            .settings
            .hud_positions
            .get("build")
            .copied()
            .unwrap_or(default_pos),
        area,
        Vec2::new(316.0, 290.0),
    );
    let current_gold = live.active_player.current_gold.max(0.0) as u64;
    let mut export = false;
    let mut close = false;
    let resp = egui::Area::new(Id::unique("riftcore-overlay-build"))
        .current_pos(pos)
        .movable(state.settings.hud_edit_layout)
        .show(ctx, |ui| {
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
                    if let Some(&target_item) = build.core_items.get(1).or(build.core_items.first())
                    {
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
                    ScrollArea::vertical()
                        .id_salt("overlay-live-build-items")
                        .max_height(168.0)
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
    if resp.response.drag_stopped() {
        let p = [resp.response.rect.min.x, resp.response.rect.min.y];
        state.settings.hud_positions.insert("build".to_string(), p);
        state
            .pending_actions
            .updated_positions
            .push(("build".to_string(), p));
    }
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
    state: &mut OverlayRenderState,
    clock: u64,
    interactive_rects_out: &mut Vec<Rect>,
) {
    let default_pos = [
        (area.left() + 300.0)
            .min(area.right() - 410.0)
            .max(area.left() + 8.0),
        area.top() + 112.0,
    ];
    let pos = clamp_logical_panel_pos(
        state
            .settings
            .hud_positions
            .get("timers")
            .copied()
            .unwrap_or(default_pos),
        area,
        Vec2::new(390.0, 260.0),
    );
    let mut copy_callout = false;
    let mut close = false;
    let resp = egui::Area::new(Id::unique("riftcore-overlay-timers"))
        .current_pos(pos)
        .movable(state.settings.hud_edit_layout)
        .show(ctx, |ui| {
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
                    let max_height = (area.height() - 365.0).clamp(160.0, 360.0);
                    ScrollArea::vertical()
                        .id_salt("riftcore-overlay-timer-list")
                        .max_height(max_height)
                        .show(ui, |ui| match state.timer_tab {
                            OverlayTimerTab::Camps if state.settings.hud_manual_camp_timers => {
                                render_manual_camp_timers(ui, &mut state.manual_timers);
                            }
                            OverlayTimerTab::Objectives if state.settings.hud_objective_timers => {
                                if let Some(live) = state.live_game.as_ref() {
                                    let timers = live.objective_timers();
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
                                                        .color(if timer.ready {
                                                            GREEN
                                                        } else {
                                                            TEXT
                                                        }),
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
    if resp.response.drag_stopped() {
        let p = [resp.response.rect.min.x, resp.response.rect.min.y];
        state.settings.hud_positions.insert("timers".to_string(), p);
        state
            .pending_actions
            .updated_positions
            .push(("timers".to_string(), p));
    }
    if close {
        state.show_timers_panel = false;
    }
    if copy_callout {
        let mut callouts = Vec::new();
        if let Some(live) = state.live_game.as_ref() {
            for timer in live.objective_timers() {
                callouts.push(format!(
                    "{} {}",
                    timer.label.to_ascii_lowercase(),
                    timer.formatted_remaining()
                ));
            }
        }
        let now = Instant::now();
        for (camp, key) in [
            ("blue", "jungle-camp-blue-buff"),
            ("red", "jungle-camp-red-buff"),
        ] {
            if let Some(rem) = state
                .manual_timers
                .get(key)
                .map(|end| end.saturating_duration_since(now).as_secs())
                .filter(|s| *s > 0)
            {
                callouts.push(format!("{camp} {}", format_clock(clock + rem)));
            }
        }
        let message = if callouts.is_empty() {
            "[RiftCore] No active timers".to_string()
        } else {
            format!("[RiftCore] {}", callouts.join(" | "))
        };
        ctx.copy_text(message.clone());
        state.pending_actions.copy_clipboard = Some(message);
        state.pending_actions.notice = Some("Timer call-out copied to clipboard".to_string());
    }
}

fn render_manual_camp_timers(ui: &mut egui::Ui, timers: &mut HashMap<String, Instant>) {
    const CAMPS: [(&str, u64); 7] = [
        ("BLUE BUFF", 300),
        ("GROMP", 135),
        ("WOLVES", 135),
        ("RAPTORS", 135),
        ("RED BUFF", 300),
        ("KRUGS", 135),
        ("SCUTTLE", 150),
    ];
    ui.label(
        RichText::new("Click a camp when cleared to start its manual respawn timer.")
            .size(9.5)
            .color(SECONDARY),
    );
    ui.add_space(5.0);
    let now = Instant::now();
    for (camp, duration) in CAMPS {
        let key = format!(
            "jungle-camp-{}",
            camp.to_ascii_lowercase().replace(' ', "-")
        );
        let remaining = timers
            .get(&key)
            .map(|ready_at| ready_at.saturating_duration_since(now).as_secs())
            .unwrap_or(0);
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
                    timers.insert(key, now + Duration::from_secs(duration));
                }
            }
        });
    }
}
