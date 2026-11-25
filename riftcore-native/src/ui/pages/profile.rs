//! Profile & Analysis view: connected summoner card, ranked progress,
//! multi-game `PlayerTrendReport`, local & Riot ID match history, and modal analysis trigger.

use crate::domain::analytics::{
    compute_player_trend_report, normalize_local_match, normalize_riot_match,
    NormalizedMatchSummary, PlayerTrendReport, TrendDirection,
};
use crate::domain::champions;
use crate::domain::protocol::{CoreCommand, CoreNotice};
use crate::ui::pages::PageContext;
use crate::ui::theme::{tier_color, ThemePalette};
use crate::ui::widgets::{
    action_button, avatar, badge, inset_card, match_summary_row, page_heading, panel, panel_title,
    progress_bar, search_box,
};
use eframe::egui::{self, Align, Layout, RichText};
use serde_json::Value;
use std::collections::HashMap;

pub fn render(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;

    page_heading(
        ui,
        &palette,
        "SUMMONER INSIGHTS",
        "Profile & Analysis",
        "Ranked standing, multi-game performance trends, local match history, and Riot ID scouting.",
        ctx.snapshot,
    );

    render_profile_summary(ctx, ui);
    ui.add_space(10.0);
    render_local_match_history(ctx, ui);
    ui.add_space(10.0);
    render_riot_search(ctx, ui);
    ui.add_space(8.0);
    render_riot_lookup(ctx, ui);
}

/// Shared summoner + ranked queue card used by both Profile and Lobby Scout.
pub fn render_profile_summary(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;

    if let Some(summoner) = ctx.snapshot.summoner.as_ref() {
        let game_name = summoner
            .get("gameName")
            .or_else(|| summoner.get("displayName"))
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .unwrap_or("Summoner");
        let tag_line = summoner
            .get("tagLine")
            .and_then(Value::as_str)
            .unwrap_or("—");

        panel(ui, &palette, |ui| {
            ui.horizontal(|ui| {
                avatar(ui, &palette, game_name, 54.0, palette.accent, None);
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(game_name)
                                .size(21.0)
                                .strong()
                                .color(palette.text),
                        );
                        ui.add_space(6.0);
                        badge(
                            ui,
                            &format!("Level {}", num(summoner, "summonerLevel")),
                            palette.accent,
                        );
                    });
                    ui.label(
                        RichText::new(format!("#{tag_line}"))
                            .size(12.0)
                            .color(palette.secondary),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if action_button(ui, &palette, "REFRESH PROFILE", false, palette.accent)
                        .clicked()
                    {
                        ctx.send_command(CoreCommand::Refresh);
                    }
                });
            });
        });

        ui.add_space(10.0);
        panel(ui, &palette, |ui| {
            panel_title(
                ui,
                &palette,
                "RANKED QUEUES",
                "Current season rank and ladder progress",
            );
            if let Some(queues) = ctx
                .snapshot
                .ranked
                .as_ref()
                .and_then(|ranked| ranked.get("queues"))
                .and_then(Value::as_array)
            {
                if queues.is_empty() {
                    ui.label(
                        RichText::new("No ranked queue records found yet.")
                            .size(11.0)
                            .color(palette.secondary),
                    );
                } else {
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

                            inset_card(col, &palette, palette.border, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(friendly_name)
                                            .size(12.0)
                                            .strong()
                                            .color(palette.text),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        badge(ui, &format!("{tier} {division}"), color);
                                    });
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
                                    ui.label(
                                        RichText::new("/ 100 LP").size(10.0).color(palette.muted),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if total > 0 {
                                            badge(
                                                ui,
                                                &format!("{wr:.1}% WR"),
                                                if wr >= 55.0 {
                                                    palette.green
                                                } else if wr >= 48.0 {
                                                    palette.accent
                                                } else {
                                                    palette.secondary
                                                },
                                            );
                                        }
                                    });
                                });
                                progress_bar(ui, &palette, lp_val / 100.0, color, 6.0);
                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("{wins}W  {losses}L"))
                                            .size(11.0)
                                            .color(palette.secondary),
                                    );
                                    if total > 0 {
                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                ui.label(
                                                    RichText::new(format!("{total} Total"))
                                                        .size(9.5)
                                                        .color(palette.muted),
                                                );
                                            },
                                        );
                                    }
                                });
                            });
                            col.add_space(8.0);
                        }
                    });
                }
            } else {
                ui.label(
                    RichText::new("Ranked data will appear when the League Client is online.")
                        .size(11.5)
                        .color(palette.secondary),
                );
            }
        });
    } else {
        panel(ui, &palette, |ui| {
            ui.label(
                RichText::new(if ctx.snapshot.connected {
                    "Loading summoner profile…"
                } else {
                    "Connect to the League Client to load your ranked profile."
                })
                .size(12.0)
                .color(palette.secondary),
            );
        });
    }
}

fn render_local_match_history(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;
    let puuid = ctx.snapshot.summoner_puuid.as_str();
    let champ_names: HashMap<u32, String> = champions::all_champions()
        .iter()
        .map(|c| (c.id, c.name.clone()))
        .collect();

    let summaries: Vec<NormalizedMatchSummary> = ctx
        .snapshot
        .match_history
        .as_ref()
        .and_then(|history| history.get("games"))
        .and_then(|games| games.get("games"))
        .and_then(Value::as_array)
        .map(|games| {
            games
                .iter()
                .filter_map(|g| normalize_local_match(g, puuid, &champ_names))
                .collect()
        })
        .unwrap_or_default();

    if !summaries.is_empty() {
        render_trend_report_panel(ctx, ui, "RECENT PERFORMANCE TRENDS", &summaries);
        ui.add_space(10.0);
    }

    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            "RECENT LOCAL MATCHES",
            "Loaded directly from your League Client history · click ANALYZE for post-game breakdown",
        );
        if summaries.is_empty() {
            ui.label(
                RichText::new(if ctx.snapshot.connected {
                    "No recent match history is available yet."
                } else {
                    "Connect the League Client to load recent matches."
                })
                .size(10.5)
                .color(palette.secondary),
            );
        } else {
            for summary in &summaries {
                if match_summary_row(ui, &palette, ctx.textures, summary) {
                    ctx.state.profile.selected_analysis_match = Some(summary.clone());
                }
            }
        }
    });
}

fn render_trend_report_panel(
    ctx: &mut PageContext<'_>,
    ui: &mut egui::Ui,
    title: &str,
    matches: &[NormalizedMatchSummary],
) {
    let palette = *ctx.palette;
    let window = ctx.state.profile.trend_window.max(5);
    let report: PlayerTrendReport = compute_player_trend_report(matches, window);

    panel(ui, &palette, |ui| {
        ui.horizontal_wrapped(|ui| {
            panel_title(
                ui,
                &palette,
                title,
                &format!(
                    "Sample of {} games ({}W - {}L · {:.1}% WR)",
                    report.sample_count, report.wins, report.losses, report.win_rate_pct
                ),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                for n in [20usize, 10, 5] {
                    if action_button(
                        ui,
                        &palette,
                        &format!("LAST {n}"),
                        ctx.state.profile.trend_window == n,
                        palette.accent,
                    )
                    .clicked()
                    {
                        ctx.state.profile.trend_window = n;
                    }
                }
            });
        });

        ui.horizontal_wrapped(|ui| {
            trend_metric_card(
                ui,
                &palette,
                "AVG KDA",
                &format!("{:.2}", report.avg_kda),
                report.kda_trend.direction,
                palette.accent,
            );
            trend_metric_card(
                ui,
                &palette,
                "CS / MIN",
                &format!("{:.1}", report.avg_cs_per_min),
                report.cs_per_min_trend.direction,
                palette.gold,
            );
            trend_metric_card(
                ui,
                &palette,
                "KILL PART.",
                &format!("{:.0}%", report.avg_kp_pct),
                report.kp_pct_trend.direction,
                palette.green,
            );
            trend_metric_card(
                ui,
                &palette,
                "DMG SHARE",
                &format!("{:.1}%", report.avg_dmg_pct),
                report.dmg_pct_trend.direction,
                palette.red,
            );
            trend_metric_card(
                ui,
                &palette,
                "GOLD / MIN",
                &format!("{:.0}", report.avg_gpm),
                report.gpm_trend.direction,
                palette.gold,
            );
            trend_metric_card(
                ui,
                &palette,
                "VISION / MIN",
                &format!("{:.2}", report.avg_vpm),
                report.vpm_trend.direction,
                palette.violet,
            );
        });

        if !report.by_champion.is_empty() {
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new("TOP CHAMPIONS")
                        .size(9.5)
                        .strong()
                        .color(palette.muted),
                );
                for champ in report.by_champion.iter().take(4) {
                    badge(
                        ui,
                        &format!(
                            "{} ({}G · {:.0}% WR · {:.2} KDA)",
                            champ.champion_name, champ.games, champ.win_rate_pct, champ.avg_kda
                        ),
                        palette.accent,
                    );
                }
            });
        }
    });
}

/// Shared Riot ID search box used by both Profile and Lobby Scout.
pub fn render_riot_search(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;

    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            "RIOT ID SCOUT",
            "Ranked entries, top champion masteries, trends, and match history",
        );
        let mut submit = false;
        ui.horizontal_wrapped(|ui| {
            let search_res = search_box(
                ui,
                &palette,
                "riot-id-scout-input",
                &mut ctx.state.profile.riot_id_search,
                "GameName#TAG",
                Some(280.0),
            );
            if search_res.submitted {
                submit = true;
            }
            let loading = ctx.state.profile.lookup_loading;
            if action_button(
                ui,
                &palette,
                if loading {
                    "SEARCHING…"
                } else {
                    "SCOUT ACCOUNT"
                },
                loading,
                palette.accent,
            )
            .clicked()
                && !loading
            {
                submit = true;
            }
            if ctx.store.settings.riot_api_key.trim().is_empty() {
                ui.label(
                    RichText::new("Add a Riot API key in Preferences to enable Riot ID scouting.")
                        .size(10.0)
                        .color(palette.gold),
                );
            }
        });

        if submit && !ctx.state.profile.lookup_loading {
            let query = ctx.state.profile.riot_id_search.trim().to_string();
            if query.is_empty() {
                ctx.set_notice(CoreNotice::Warning(
                    "Enter a Riot ID like GameName#TAG".to_string(),
                ));
            } else {
                ctx.state.profile.lookup_loading = true;
                ctx.send_command(CoreCommand::LookupRiotId {
                    riot_id: query,
                    region: ctx.store.settings.riot_region.clone(),
                    api_key: ctx.store.settings.riot_api_key.clone(),
                });
            }
        }
    });
}

/// Shared Riot API lookup result card used by both Profile and Lobby Scout.
pub fn render_riot_lookup(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;
    let Some(lookup) = ctx.snapshot.riot_lookup.clone() else {
        return;
    };

    if let Some(error) = lookup.get("error").and_then(Value::as_str) {
        panel(ui, &palette, |ui| {
            panel_title(ui, &palette, "SCOUT LOOKUP", "Riot API response");
            ui.label(RichText::new(error).size(11.0).color(palette.red));
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

    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            &display_name,
            &format!(
                "{}  ·  Level {}",
                lookup
                    .get("region")
                    .and_then(Value::as_str)
                    .unwrap_or("—")
                    .to_ascii_uppercase(),
                num(summoner, "summonerLevel")
            ),
        );

        ui.label(
            RichText::new("RANKED QUEUES")
                .size(9.5)
                .strong()
                .color(palette.muted),
        );
        if let Some(entries) = lookup.get("ranked").and_then(Value::as_array) {
            if entries.is_empty() {
                ui.label(
                    RichText::new("Unranked or no ranked entries found.")
                        .size(10.5)
                        .color(palette.secondary),
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
                ui.label(
                    RichText::new(format!(
                        "{queue} · {tier} {division} · {} LP · {wins}W / {losses}L · {rate:.1}%",
                        entry
                            .get("leaguePoints")
                            .and_then(Value::as_u64)
                            .unwrap_or(0)
                    ))
                    .size(10.5)
                    .color(tier_color(tier)),
                );
            }
        }

        ui.add_space(6.0);
        ui.label(
            RichText::new("TOP CHAMPION MASTERY")
                .size(9.5)
                .strong()
                .color(palette.muted),
        );
        if let Some(masteries) = lookup.get("masteries").and_then(Value::as_array) {
            let egui_ctx = ui.ctx().clone();
            ui.horizontal_wrapped(|ui| {
                for mastery in masteries.iter().take(5) {
                    let id = mastery
                        .get("championId")
                        .and_then(Value::as_u64)
                        .unwrap_or(0) as u32;
                    let champ = champions::champion_by_id(id);
                    let name = champ.map(|c| c.name.as_str()).unwrap_or("Champion");
                    let portrait = champ.and_then(|c| ctx.textures.champion(&egui_ctx, &c.key));
                    inset_card(ui, &palette, palette.border, |ui| {
                        ui.horizontal(|ui| {
                            avatar(ui, &palette, name, 26.0, palette.accent, portrait);
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(name).size(10.5).strong().color(palette.text),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "M{} · {} pts",
                                        mastery
                                            .get("championLevel")
                                            .and_then(Value::as_u64)
                                            .unwrap_or(0),
                                        mastery
                                            .get("championPoints")
                                            .and_then(Value::as_u64)
                                            .unwrap_or(0)
                                    ))
                                    .size(9.0)
                                    .color(palette.accent),
                                );
                            });
                        });
                    });
                }
            });
        }

        ui.add_space(8.0);
        ui.label(
            RichText::new("RECENT MATCHES")
                .size(9.5)
                .strong()
                .color(palette.muted),
        );
        if let Some(matches) = lookup.get("matches").and_then(Value::as_array) {
            let puuid = account
                .get("puuid")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let summaries: Vec<NormalizedMatchSummary> = matches
                .iter()
                .filter_map(|g| normalize_riot_match(g, puuid))
                .collect();
            for summary in &summaries {
                if match_summary_row(ui, &palette, ctx.textures, summary) {
                    ctx.state.profile.selected_analysis_match = Some(summary.clone());
                }
            }
        }
    });
}

fn trend_metric_card(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    label: &str,
    value: &str,
    dir: TrendDirection,
    color: egui::Color32,
) {
    let (arrow, arrow_color) = match dir {
        TrendDirection::Up => ("▲", palette.green),
        TrendDirection::Down => ("▼", palette.red),
        TrendDirection::Flat => ("•", palette.muted),
    };
    egui::Frame::new()
        .fill(palette.surface)
        .stroke(egui::Stroke::new(1.0, palette.border))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.set_min_width(112.0);
            ui.label(RichText::new(label).size(9.5).strong().color(palette.muted));
            ui.horizontal(|ui| {
                ui.label(RichText::new(value).size(15.0).strong().color(color));
                ui.label(RichText::new(arrow).size(11.0).strong().color(arrow_color));
            });
        });
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
