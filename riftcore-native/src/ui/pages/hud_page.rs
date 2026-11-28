//! In-Game HUD control center: live match summary, ARAM relic countdowns,
//! layout mockup, and compliant HUD module settings.
//!
//! Riot policy compliance:
//! - Enemy summoner spell cooldown tracking UI is strictly omitted.
//! - Active-game coaching insights remain gated behind `LIVE_COACH_ENABLED` (= false).

use crate::domain::items;
use crate::domain::live_game::LiveGameState;
use crate::domain::protocol::{CoreCommand, LIVE_COACH_ENABLED};
use crate::ui::pages::{HudSettingsTab, PageContext};
use crate::ui::theme::{Spacing, ThemePalette};
use crate::ui::widgets::{
    action_button, avatar, badge, item_slot, kda_display, metric_card, page_heading, panel,
    panel_title, toggle_row,
};
use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Sense, Stroke, Vec2};
use std::time::{Duration, Instant};

pub fn render(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;

    page_heading(
        ui,
        &palette,
        "LIVE COMPANION",
        "In-Game HUD",
        "Native transparent overlay for live match telemetry, objective timers, and minimap guides.",
        ctx.snapshot,
    );

    if let Some(live) = ctx.state.parsed_live_game.clone() {
        ui.horizontal_wrapped(|ui| {
            let label = if ctx.state.overlay_mode {
                "HIDE NATIVE OVERLAY"
            } else {
                "SHOW NATIVE OVERLAY"
            };
            if action_button(ui, &palette, label, ctx.state.overlay_mode, palette.accent).clicked()
            {
                ctx.effects.set_overlay = Some((!ctx.state.overlay_mode, false));
            }
            if action_button(ui, &palette, "REFRESH LIVE DATA", false, palette.secondary).clicked()
            {
                ctx.send_command(CoreCommand::Refresh);
            }
            badge(ui, &live.game_mode, palette.accent);
        });
        ui.add_space(8.0);
        render_live_summary(ctx, ui, &live);

        if ctx.store.settings.hud_aram_health_timers && live.is_aram() {
            ui.add_space(8.0);
            render_aram_relic_timers(ctx, ui);
        }

        if LIVE_COACH_ENABLED {
            // Reserved gate: never active while LIVE_COACH_ENABLED == false.
        }
    } else {
        ui.columns(2, |columns| {
            panel(&mut columns[0], &palette, |ui| {
                ui.horizontal(|ui| {
                    avatar(ui, &palette, "HUD", 42.0, palette.accent, None);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("NATIVE IN-GAME OVERLAY")
                                .size(14.0)
                                .strong()
                                .color(palette.text),
                        );
                        let (status, color) =
                            if ctx.state.overlay_mode && ctx.state.overlay_waiting_for_window {
                                ("WAITING FOR LEAGUE", palette.gold)
                            } else if ctx.state.overlay_mode {
                                ("OVERLAY ENABLED", palette.accent)
                            } else {
                                ("STANDBY", palette.secondary)
                            };
                        badge(ui, status, color);
                    });
                });
                ui.add_space(6.0);
                ui.add(
                    egui::Label::new(
                        RichText::new("Works over League's Fullscreen, Borderless and Windowed modes. If the HUD stays hidden over Fullscreen on your PC, Borderless is a reliable fallback.")
                            .size(10.5)
                            .color(palette.secondary),
                    )
                    .wrap(),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Hotkeys: Alt + ` or Alt + Shift + Space (HUD)  ·  Alt + Shift + ` (Click-through)")
                        .size(9.5)
                        .monospace()
                        .color(palette.muted),
                );
                ui.add_space(8.0);
                let overlay_btn = if ctx.state.overlay_mode {
                    "DISABLE IN-GAME HUD"
                } else {
                    "ENABLE IN-GAME HUD"
                };
                if action_button(
                    ui,
                    &palette,
                    overlay_btn,
                    ctx.state.overlay_mode,
                    palette.accent,
                )
                .clicked()
                {
                    ctx.store.settings.hud_preview_mode = false;
                    ctx.effects.set_overlay = Some((!ctx.state.overlay_mode, false));
                }
            });

            panel(&mut columns[1], &palette, |ui| {
                panel_title(
                    ui,
                    &palette,
                    "HUD LAYOUT MOCKUP",
                    "Schematic placement preview; does not cover your desktop",
                );
                render_hud_mockup(ui, &palette, &ctx.store.settings.hud_mode);
            });
        });
    }

    ui.add_space(10.0);
    render_hud_settings(ctx, ui);
}

fn render_live_summary(ctx: &mut PageContext<'_>, ui: &mut egui::Ui, live: &LiveGameState) {
    let palette = *ctx.palette;
    let time = live.game_time_secs.max(0.0) as u64;
    let scores = &live.active_player.scores;
    let cs_min = scores.creep_score as f64 / (live.game_time_secs / 60.0).max(1.0);

    ui.horizontal_wrapped(|ui| {
        metric_card(
            ui,
            &palette,
            "GAME CLOCK",
            &format!("{:02}:{:02}", time / 60, time % 60),
            palette.accent,
        );
        metric_card(
            ui,
            &palette,
            "K / D / A",
            &format!("{} / {} / {}", scores.kills, scores.deaths, scores.assists),
            palette.text,
        );
        metric_card(
            ui,
            &palette,
            "CS (PACE)",
            &format!("{} ({:.1}/m)", scores.creep_score, cs_min),
            palette.gold,
        );
        metric_card(
            ui,
            &palette,
            "CURRENT GOLD",
            &format!("{:.0}g", live.active_player.current_gold),
            palette.green,
        );
        if !live.active_player.champion_name.is_empty() {
            metric_card(
                ui,
                &palette,
                "CHAMPION",
                &format!(
                    "{} · Lv {}",
                    live.active_player.champion_name, live.active_player.level
                ),
                palette.violet,
            );
        }
    });

    ui.add_space(8.0);
    ui.columns(2, |columns| {
        panel(&mut columns[0], &palette, |ui| {
            panel_title(
                ui,
                &palette,
                "ACTIVE BUILD & ABILITIES",
                "Resolved from Live Client Data",
            );
            let ab = &live.active_player.abilities;
            ui.horizontal_wrapped(|ui| {
                for (key, lvl) in [
                    ("Q", ab.q.ability_level),
                    ("W", ab.w.ability_level),
                    ("E", ab.e.ability_level),
                    ("R", ab.r.ability_level),
                ] {
                    badge(ui, &format!("{key} {lvl}"), palette.accent);
                }
            });
            ui.add_space(6.0);
            if live.active_player.items.is_empty() {
                ui.label(
                    RichText::new("No items purchased yet.")
                        .size(10.5)
                        .color(palette.muted),
                );
            } else {
                ui.horizontal_wrapped(|ui| {
                    for item in live.active_player.items.iter().filter(|i| i.item_id > 0) {
                        item_slot(ui, &palette, ctx.textures, item.item_id, 26.0, true);
                    }
                });
                let has_trinket = live
                    .active_player
                    .items
                    .iter()
                    .any(|item| item.slot == 6 && items::is_trinket_id(item.item_id));
                if !has_trinket && ctx.store.settings.hud_trinket_reminder {
                    ui.add_space(4.0);
                    badge(ui, "TRINKET SLOT EMPTY", palette.gold);
                }
            }
        });

        panel(&mut columns[1], &palette, |ui| {
            let timers = live.objective_timers();
            panel_title(
                ui,
                &palette,
                "OBJECTIVE TIMERS",
                if live.is_summoners_rift() {
                    "Summoner's Rift epic objective schedule"
                } else {
                    "Objective timers apply on Summoner's Rift"
                },
            );
            if timers.is_empty() {
                ui.label(
                    RichText::new("No SR epic objective timers for this map.")
                        .size(10.5)
                        .color(palette.secondary),
                );
            } else {
                for timer in timers {
                    ui.horizontal(|ui| {
                        badge(
                            ui,
                            &timer.label,
                            if timer.ready {
                                palette.green
                            } else {
                                palette.gold
                            },
                        );
                        ui.label(
                            RichText::new(timer.formatted_remaining())
                                .size(12.0)
                                .monospace()
                                .strong()
                                .color(if timer.ready {
                                    palette.green
                                } else {
                                    palette.text
                                }),
                        );
                    });
                    ui.add_space(3.0);
                }
            }
        });
    });

    ui.add_space(8.0);
    panel(ui, &palette, |ui| {
        panel_title(ui, &palette, "LIVE SCOREBOARD", &live.game_mode);
        if live.all_players.is_empty() {
            ui.label(
                RichText::new("Waiting for player roster…")
                    .size(10.5)
                    .color(palette.secondary),
            );
            return;
        }
        egui::Grid::new("live-roster-v4")
            .num_columns(4)
            .spacing([14.0, 4.0])
            .show(ui, |ui| {
                for h in ["TEAM", "SUMMONER", "CHAMPION", "K / D / A  ·  CS"] {
                    ui.label(RichText::new(h).size(9.5).strong().color(palette.muted));
                }
                ui.end_row();
                for team in ["ORDER", "CHAOS"] {
                    let team_label = if team == "ORDER" { "ORDER" } else { "CHAOS" };
                    let team_color = if team == "ORDER" {
                        palette.accent
                    } else {
                        palette.red
                    };
                    for player in live
                        .all_players
                        .iter()
                        .filter(|p| p.team.eq_ignore_ascii_case(team))
                    {
                        ui.label(
                            RichText::new(team_label)
                                .size(9.5)
                                .strong()
                                .color(team_color),
                        );
                        let name = if !player.summoner_name.is_empty() {
                            player.summoner_name.as_str()
                        } else {
                            player.riot_id_game_name.as_str()
                        };
                        ui.label(RichText::new(name).size(11.0).color(palette.text));
                        ui.label(
                            RichText::new(&player.champion_name)
                                .size(10.5)
                                .color(palette.secondary),
                        );
                        ui.horizontal(|ui| {
                            kda_display(
                                ui,
                                &palette,
                                player.scores.kills,
                                player.scores.deaths,
                                player.scores.assists,
                                None,
                            );
                            ui.label(
                                RichText::new(format!("· {} CS", player.scores.creep_score))
                                    .size(10.0)
                                    .color(palette.gold),
                            );
                        });
                        ui.end_row();
                    }
                }
            });
    });
}

fn render_aram_relic_timers(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;
    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            "ARAM HEALTH RELIC TIMERS",
            "Enter the displayed respawn time for this match, then start a manual countdown",
        );
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Respawn seconds")
                    .size(10.5)
                    .color(palette.secondary),
            );
            ui.add_sized(
                [90.0, 26.0],
                egui::TextEdit::singleline(&mut ctx.state.hud_page.aram_relic_seconds)
                    .hint_text("seconds"),
            );
        });
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for side in ["LEFT", "RIGHT"] {
                let key = format!("aram-relic-{side}");
                let remaining = ctx
                    .state
                    .hud_page
                    .manual_timers
                    .get(&key)
                    .map(|ready_at| ready_at.saturating_duration_since(Instant::now()).as_secs())
                    .unwrap_or(0);
                if remaining > 0 {
                    ui.label(
                        RichText::new(format!(
                            "{side} {:02}:{:02}",
                            remaining / 60,
                            remaining % 60
                        ))
                        .size(10.5)
                        .monospace()
                        .color(palette.green),
                    );
                    if action_button(ui, &palette, "RESET", false, palette.secondary).clicked() {
                        ctx.state.hud_page.manual_timers.remove(&key);
                    }
                } else {
                    let seconds = ctx
                        .state
                        .hud_page
                        .aram_relic_seconds
                        .trim()
                        .parse::<u64>()
                        .ok()
                        .filter(|&s| s > 0);
                    if action_button(
                        ui,
                        &palette,
                        &format!("START {side}"),
                        seconds.is_none(),
                        palette.gold,
                    )
                    .clicked()
                    {
                        if let Some(secs) = seconds {
                            ctx.state
                                .hud_page
                                .manual_timers
                                .insert(key, Instant::now() + Duration::from_secs(secs));
                        }
                    }
                }
            }
        });
    });
}

/// Shared HUD module configuration section used by both `In-Game HUD` and `Preferences -> Overlay`.
pub fn render_hud_settings(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;

    ui.horizontal_wrapped(|ui| {
        for (tab, label) in [
            (HudSettingsTab::Timers, "TIMERS"),
            (HudSettingsTab::Stats, "STATS"),
            (HudSettingsTab::Guides, "GUIDES"),
            (HudSettingsTab::Layout, "LAYOUT"),
        ] {
            if action_button(
                ui,
                &palette,
                label,
                ctx.state.hud_page.tab == tab,
                palette.accent,
            )
            .clicked()
            {
                ctx.state.hud_page.tab = tab;
            }
        }
        ui.label(
            RichText::new("HUD MODULES")
                .size(10.0)
                .strong()
                .color(palette.muted),
        );
    });
    ui.add_space(6.0);

    let mut changed = false;
    let s = &mut ctx.store.settings;

    match ctx.state.hud_page.tab {
        HudSettingsTab::Timers => ui.columns(2, |columns| {
            panel(&mut columns[0], &palette, |ui| {
                panel_title(
                    ui,
                    &palette,
                    "OBJECTIVES & ALLIES",
                    "Compliant live game timers",
                );
                changed |= toggle_row(
                    ui,
                    &palette,
                    "Objective Timers",
                    "Dragon, Voidgrubs, Rift Herald, and Baron",
                    &mut s.hud_objective_timers,
                );
                changed |= toggle_row(
                    ui,
                    &palette,
                    "Allied Ultimate Timers",
                    "Manual click timers for allied R cooldowns",
                    &mut s.hud_ally_ult_timers,
                );
            });
            panel(&mut columns[1], &palette, |ui| {
                panel_title(
                    ui,
                    &palette,
                    "CAMPS & RELICS",
                    "Manual camp and mode-specific countdowns",
                );
                changed |= toggle_row(
                    ui,
                    &palette,
                    "Jungle Camp Respawns",
                    "Manual camp respawn dock",
                    &mut s.hud_manual_camp_timers,
                );
                changed |= toggle_row(
                    ui,
                    &palette,
                    "ARAM Health Relic Timers",
                    "Manual health relic countdowns",
                    &mut s.hud_aram_health_timers,
                );
            });
        }),
        HudSettingsTab::Stats => ui.columns(2, |columns| {
            panel(&mut columns[0], &palette, |ui| {
                panel_title(ui, &palette, "STATS & ECONOMY", "Live match progress");
                changed |= toggle_row(
                    ui,
                    &palette,
                    "Current Gold Tracker",
                    "Display active held gold",
                    &mut s.hud_gold_lead,
                );
                changed |= toggle_row(
                    ui,
                    &palette,
                    "CS Per Minute Pace",
                    "Display live CS/min rate",
                    &mut s.hud_cs_benchmark,
                );
            });
            panel(&mut columns[1], &palette, |ui| {
                panel_title(
                    ui,
                    &palette,
                    "PLAYER TELEMETRY",
                    "Inventory and ability ranks",
                );
                changed |= toggle_row(
                    ui,
                    &palette,
                    "Ability Rank Display",
                    "Show current Q/W/E/R skill ranks",
                    &mut s.hud_skill_order,
                );
                changed |= toggle_row(
                    ui,
                    &palette,
                    "Inventory Display",
                    "Show equipped items on HUD",
                    &mut s.hud_item_goal,
                );
                changed |= toggle_row(
                    ui,
                    &palette,
                    "Trinket Ward Reminder",
                    "Alert when slot-6 trinket is empty",
                    &mut s.hud_trinket_reminder,
                );
            });
        }),
        HudSettingsTab::Guides => ui.columns(2, |columns| {
            panel(&mut columns[0], &palette, |ui| {
                panel_title(
                    ui,
                    &palette,
                    "ROUTE & MODE REFERENCES",
                    "Jungle and mode reference",
                );
                changed |= toggle_row(
                    ui,
                    &palette,
                    "Mayhem & Arena Cards Guide",
                    "Quick augment reference in ARAM/Arena",
                    &mut s.hud_mayhem_cards,
                );
                changed |= toggle_row(
                    ui,
                    &palette,
                    "Jungle Opening Pathing",
                    "First-clear route reminder",
                    &mut s.hud_jungle_pathing,
                );
                if s.hud_jungle_pathing {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Start route:")
                                .size(10.5)
                                .color(palette.secondary),
                        );
                        for (side, label, accent) in [
                            ("red", "RED", palette.red),
                            ("blue", "BLUE", palette.accent),
                        ] {
                            if action_button(
                                ui,
                                &palette,
                                label,
                                s.jungle_start_buff == side,
                                accent,
                            )
                            .clicked()
                            {
                                s.jungle_start_buff = side.to_string();
                                changed = true;
                            }
                        }
                    });
                }
            });
            panel(&mut columns[1], &palette, |ui| {
                panel_title(ui, &palette, "MINIMAP PINS", "Visual reference markers");
                changed |= toggle_row(
                    ui,
                    &palette,
                    "Objective Countdown Pins",
                    "Markers on Dragon/Baron pits",
                    &mut s.hud_map_objective_pins,
                );
                changed |= toggle_row(
                    ui,
                    &palette,
                    "Jungle Route Step Pins",
                    "Numbered first-clear steps",
                    &mut s.hud_map_route_pins,
                );
                changed |= toggle_row(
                    ui,
                    &palette,
                    "Static Ward Guide Pins",
                    "Common ward location markers",
                    &mut s.hud_map_ward_pins,
                );
            });
        }),
        HudSettingsTab::Layout => ui.columns(2, |columns| {
            panel(&mut columns[0], &palette, |ui| {
                panel_title(
                    ui,
                    &palette,
                    "HUD PRESENTATION",
                    "Choose overlay arrangement",
                );
                ui.horizontal_wrapped(|ui| {
                    for (mode, label) in [
                        ("integrated", "INTEGRATED"),
                        ("bar", "TOP BAR"),
                        ("card", "TACTICAL CARD"),
                        ("pill", "MINIMAP PILL"),
                    ] {
                        if action_button(ui, &palette, label, s.hud_mode == mode, palette.accent)
                            .clicked()
                        {
                            s.hud_mode = mode.to_string();
                            ctx.commands
                                .push(CoreCommand::SetHudMode(s.hud_mode.clone()));
                            changed = true;
                        }
                    }
                });
                ui.add_space(6.0);
                if toggle_row(
                    ui,
                    &palette,
                    "Edit overlay layout",
                    "Unlock movable HUD panels inside League",
                    &mut s.hud_edit_layout,
                ) {
                    if s.hud_edit_layout {
                        ctx.state.overlay_mouse_passthrough = false;
                        ctx.effects.set_passthrough = Some(false);
                    }
                    changed = true;
                }
            });
            panel(&mut columns[1], &palette, |ui| {
                panel_title(
                    ui,
                    &palette,
                    "MINIMAP TIMER DOCK",
                    "Adjust scale, position, and reset saved drag offsets",
                );
                changed |= ui
                    .horizontal(|ui| {
                        ui.label(
                            RichText::new("Map scale")
                                .size(10.5)
                                .color(palette.secondary),
                        );
                        ui.add(
                            egui::Slider::new(&mut s.hud_minimap_scale, 1.0..=3.0).show_value(true),
                        )
                        .changed()
                    })
                    .inner;
                changed |= ui
                    .horizontal(|ui| {
                        ui.label(
                            RichText::new("Dock size")
                                .size(10.5)
                                .color(palette.secondary),
                        );
                        ui.add(
                            egui::Slider::new(&mut s.hud_minimap_zoom, 0.75..=1.35)
                                .show_value(true),
                        )
                        .changed()
                    })
                    .inner;
                changed |= toggle_row(
                    ui,
                    &palette,
                    "Minimap flipped to left",
                    "Mirror minimap overlay anchors",
                    &mut s.hud_minimap_flip,
                );
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Offset").size(10.0).color(palette.secondary));
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut s.hud_minimap_nudge_x)
                                .range(-250.0..=250.0)
                                .speed(2.0)
                                .suffix(" x"),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut s.hud_minimap_nudge_y)
                                .range(-250.0..=250.0)
                                .speed(2.0)
                                .suffix(" y"),
                        )
                        .changed();
                    if action_button(ui, &palette, "RESET LAYOUT", false, palette.secondary)
                        .clicked()
                    {
                        s.hud_minimap_nudge_x = 0.0;
                        s.hud_minimap_nudge_y = 0.0;
                        s.hud_minimap_zoom = 1.0;
                        s.hud_positions.clear();
                        changed = true;
                    }
                });
            });
        }),
    }

    if changed {
        ctx.request_save();
    }
}

fn render_hud_mockup(ui: &mut egui::Ui, palette: &ThemePalette, hud_mode: &str) {
    let width = ui.available_width().clamp(260.0, 640.0);
    let height = (width * 0.38).clamp(110.0, 220.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter();

    painter.rect_filled(rect, CornerRadius::same(Spacing::RADIUS_MD), palette.inset);
    painter.rect_stroke(
        rect,
        CornerRadius::same(Spacing::RADIUS_MD),
        Stroke::new(1.0, palette.border),
        egui::StrokeKind::Inside,
    );

    if hud_mode == "bar" || hud_mode == "integrated" {
        let top_center = egui::pos2(rect.center().x, rect.top() + 18.0);
        let top_bar_rect = egui::Rect::from_center_size(top_center, Vec2::new(width * 0.56, 22.0));
        painter.rect_filled(top_bar_rect, CornerRadius::same(6), palette.surface);
        painter.rect_stroke(
            top_bar_rect,
            CornerRadius::same(6),
            Stroke::new(1.0, palette.accent),
            egui::StrokeKind::Inside,
        );
        painter.text(
            top_bar_rect.center(),
            egui::Align2::CENTER_CENTER,
            "14:22  ·  3 / 1 / 5  ·  128 CS (9.0/m)  ·  1,450 G",
            FontId::proportional(9.0),
            palette.text,
        );
    }

    let map_size = height * 0.56;
    let map_rect = egui::Rect::from_min_size(
        egui::pos2(
            rect.right() - map_size - 14.0,
            rect.bottom() - map_size - 12.0,
        ),
        Vec2::splat(map_size),
    );
    painter.rect_filled(map_rect, CornerRadius::same(6), palette.bg);
    painter.rect_stroke(
        map_rect,
        CornerRadius::same(6),
        Stroke::new(1.0, palette.border),
        egui::StrokeKind::Inside,
    );
    painter.text(
        egui::pos2(map_rect.left() + 6.0, map_rect.top() + 6.0),
        egui::Align2::LEFT_TOP,
        if hud_mode == "pill" {
            "PILL · 14:22"
        } else {
            "MINIMAP"
        },
        FontId::proportional(8.5),
        if hud_mode == "pill" {
            palette.gold
        } else {
            palette.muted
        },
    );

    painter.circle_filled(map_rect.center(), 4.0, palette.accent);
    painter.circle_filled(
        egui::pos2(map_rect.left() + 20.0, map_rect.bottom() - 20.0),
        3.5,
        palette.green,
    );
    painter.circle_filled(
        egui::pos2(map_rect.right() - 20.0, map_rect.top() + 20.0),
        3.5,
        palette.red,
    );

    if hud_mode == "card" || hud_mode == "integrated" {
        let card_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 14.0, rect.top() + 20.0),
            Vec2::new(132.0, height * 0.68),
        );
        painter.rect_filled(card_rect, CornerRadius::same(8), palette.surface);
        painter.rect_stroke(
            card_rect,
            CornerRadius::same(8),
            Stroke::new(1.0, palette.gold),
            egui::StrokeKind::Inside,
        );
        painter.text(
            egui::pos2(card_rect.left() + 8.0, card_rect.top() + 8.0),
            egui::Align2::LEFT_TOP,
            "OBJECTIVES",
            FontId::proportional(8.5),
            palette.gold,
        );
        painter.text(
            egui::pos2(card_rect.left() + 8.0, card_rect.top() + 24.0),
            egui::Align2::LEFT_TOP,
            "Dragon:  01:45",
            FontId::proportional(8.5),
            palette.text,
        );
        painter.text(
            egui::pos2(card_rect.left() + 8.0, card_rect.top() + 38.0),
            egui::Align2::LEFT_TOP,
            "Baron:   04:10",
            FontId::proportional(8.5),
            palette.secondary,
        );
        painter.text(
            egui::pos2(card_rect.left() + 8.0, card_rect.top() + 52.0),
            egui::Align2::LEFT_TOP,
            "Ally R:  Ready",
            FontId::proportional(8.5),
            Color32::from_rgb(82, 230, 173),
        );
    }
}
