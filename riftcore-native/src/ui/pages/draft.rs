//! Pre-game Draft Coach: 5v5 draft simulator, live Champ Select sync,
//! Ally vs Enemy composition comparison, and grounded `OfflineHeuristicCoach` notes.

use crate::domain::analytics::{compare_drafts, DraftChampionInput};
use crate::domain::builds;
use crate::domain::champions::{self, Role};
use crate::domain::coach::{CoachProvider, OfflineHeuristicCoach};
use crate::domain::protocol::CoreNotice;
use crate::ui::pages::{champ_select::render_team_panel, session_picks, PageContext};
use crate::ui::theme::ThemePalette;
use crate::ui::widgets::{
    action_button, avatar, badge, inset_card, page_heading, panel, panel_title, progress_bar,
    role_tag, stat_bar,
};
use eframe::egui::{self, Align, Color32, CornerRadius, FontId, Layout, RichText, Sense, Stroke, Vec2};

pub fn render(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;

    page_heading(
        ui,
        &palette,
        "TEAM COMPOSITION",
        "Draft Coach",
        "Build a 5v5 draft or sync picks from the live League Client session for composition & counter analysis.",
        ctx.snapshot.connected,
        ctx.snapshot.riot_client_open,
        ctx.snapshot.phase.display_label(),
    );

    ui.horizontal_wrapped(|ui| {
        if action_button(
            ui,
            &palette,
            "SYNC LIVE DRAFT",
            ctx.snapshot.champ_select.is_some(),
            palette.accent,
        )
        .clicked()
        {
            if let Some(session) = ctx.snapshot.champ_select.as_ref() {
                ctx.state.draft.allies = session_picks(session, "myTeam");
                ctx.state.draft.enemies = session_picks(session, "theirTeam");
                ctx.set_notice(CoreNotice::Success(
                    "Draft board synced from the League Client".to_string(),
                ));
            } else {
                ctx.set_notice(CoreNotice::Warning(
                    "Open champion select to sync the live draft".to_string(),
                ));
            }
        }
        if action_button(ui, &palette, "CLEAR BOARD", false, palette.red).clicked() {
            ctx.state.draft.allies = [0; 5];
            ctx.state.draft.enemies = [0; 5];
        }
        ui.label(
            RichText::new("Local pre-game simulation only; picks are not sent to League Client.")
                .size(10.0)
                .color(palette.muted),
        );
    });

    if ctx.snapshot.champ_select.is_some() {
        ui.add_space(8.0);
        render_team_panel(ctx, ui);
    }

    ui.add_space(10.0);
    let egui_ctx = ui.ctx().clone();
    let mut allies = ctx.state.draft.allies;
    let mut enemies = ctx.state.draft.enemies;

    ui.columns(2, |columns| {
        render_draft_team(
            &mut columns[0],
            &palette,
            ctx.textures,
            &egui_ctx,
            "ALLY DRAFT",
            "Your team picks",
            &mut allies,
            palette.accent,
        );
        render_draft_team(
            &mut columns[1],
            &palette,
            ctx.textures,
            &egui_ctx,
            "ENEMY DRAFT",
            "Opponent picks",
            &mut enemies,
            palette.red,
        );
    });
    ctx.state.draft.allies = allies;
    ctx.state.draft.enemies = enemies;

    let ally_inputs = build_draft_inputs(&allies);
    let enemy_inputs = build_draft_inputs(&enemies);
    let comparison = compare_drafts(&ally_inputs, &enemy_inputs);
    let coach = OfflineHeuristicCoach;
    let coaching_notes = coach.draft_advice(&comparison);

    ui.add_space(12.0);
    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            "COMPOSITION READOUT & PRE-GAME COACH",
            "Grounded frontline, damage split, crowd control, and counter-itemization cues from selected champions",
        );

        ui.columns(2, |columns| {
            for (col_idx, label, profile, accent) in [
                (0usize, "ALLY", &comparison.ally, palette.accent),
                (1usize, "ENEMY", &comparison.enemy, palette.red),
            ] {
                let ui = &mut columns[col_idx];
                inset_card(ui, &palette, palette.border, |ui| {
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
                                &format!("{}/5 PICKS", profile.picks_count),
                                if profile.picks_count == 5 {
                                    palette.green
                                } else {
                                    palette.gold
                                },
                            );
                        });
                    });
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("DAMAGE PROFILE")
                                .size(9.5)
                                .strong()
                                .color(palette.muted),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!(
                                    "Physical {:.0}% · Magic {:.0}%",
                                    profile.physical_pct, profile.magic_pct
                                ))
                                .size(9.5)
                                .color(palette.secondary),
                            );
                        });
                    });
                    progress_bar(
                        ui,
                        &palette,
                        (profile.physical_pct as f32) / 100.0,
                        palette.red,
                        6.0,
                    );
                    ui.add_space(8.0);

                    stat_bar(
                        ui,
                        &palette,
                        "Frontline & Durability",
                        &format!("{} / 5", profile.frontline_score.min(5)),
                        (profile.frontline_score as f32 / 5.0).min(1.0),
                        palette.accent,
                    );
                    stat_bar(
                        ui,
                        &palette,
                        "Hard Crowd Control",
                        &format!("{} / 5", profile.hard_cc_score.min(5)),
                        (profile.hard_cc_score as f32 / 5.0).min(1.0),
                        palette.violet,
                    );
                    stat_bar(
                        ui,
                        &palette,
                        "Engage & Initiation",
                        &format!("{} / 5", profile.engage_score.min(5)),
                        (profile.engage_score as f32 / 5.0).min(1.0),
                        palette.gold,
                    );
                    stat_bar(
                        ui,
                        &palette,
                        "Waveclear & Peel",
                        &format!(
                            "{} clear · {} peel",
                            profile.waveclear_score, profile.peel_score
                        ),
                        ((profile.waveclear_score + profile.peel_score) as f32 / 6.0).min(1.0),
                        palette.green,
                    );
                });
            }
        });

        ui.add_space(10.0);
        ui.columns(2, |columns| {
            inset_card(&mut columns[0], &palette, palette.accent, |ui| {
                ui.label(
                    RichText::new("ALLY COVERAGE & BALANCE")
                        .size(10.0)
                        .strong()
                        .color(palette.accent),
                );
                ui.add(
                    egui::Label::new(
                        RichText::new(&comparison.damage_balance_summary)
                            .size(11.0)
                            .color(palette.text),
                    )
                    .wrap(),
                );
                ui.add_space(4.0);
                if comparison.ally_missing_coverage.is_empty() {
                    if comparison.ally.picks_count >= 3 {
                        badge(ui, "NO CRITICAL GAPS DETECTED", palette.green);
                    }
                } else {
                    for gap in &comparison.ally_missing_coverage {
                        ui.label(
                            RichText::new(format!("• Missing: {gap}"))
                                .size(10.5)
                                .color(palette.gold),
                        );
                    }
                }
            });

            inset_card(&mut columns[1], &palette, palette.gold, |ui| {
                ui.label(
                    RichText::new("ENEMY THREAT & COUNTER FOCUS")
                        .size(10.0)
                        .strong()
                        .color(palette.gold),
                );
                ui.label(
                    RichText::new(format!(
                        "Defensive priority: {}",
                        comparison.recommended_defensive_focus
                    ))
                    .size(11.0)
                    .strong()
                    .color(palette.text),
                );
                if !comparison.recommended_counter_tags.is_empty() {
                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        for tag in &comparison.recommended_counter_tags {
                            badge(ui, tag, palette.gold);
                        }
                    });
                }
                for threat in &comparison.enemy_threats {
                    ui.label(
                        RichText::new(format!("• {threat}"))
                            .size(10.5)
                            .color(palette.secondary),
                    );
                }
            });
        });

        ui.add_space(10.0);
        inset_card(ui, &palette, palette.accent, |ui| {
            ui.label(
                RichText::new("PRE-GAME DRAFT COACH NOTES")
                    .size(10.5)
                    .strong()
                    .color(palette.accent),
            );
            ui.add_space(4.0);
            for line in &coaching_notes {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("•").size(12.0).color(palette.accent));
                    ui.add(
                        egui::Label::new(RichText::new(line).size(11.0).color(palette.text))
                            .wrap(),
                    );
                });
            }
        });
    });
}

#[allow(clippy::too_many_arguments)]
fn render_draft_team(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    textures: &mut crate::ui::textures::TextureManager,
    egui_ctx: &egui::Context,
    title: &str,
    description: &str,
    picks: &mut [u32; 5],
    accent: Color32,
) {
    let all_champs = champions::all_champions();
    panel(ui, palette, |ui| {
        panel_title(ui, palette, title, description);
        for (index, &role) in Role::ALL.iter().enumerate() {
            let picked = champions::champion_by_id(picks[index]);
            let stroke = if picked.is_some() {
                ThemePalette::with_alpha(accent, 110)
            } else {
                palette.border
            };

            inset_card(ui, palette, stroke, |ui| {
                ui.horizontal(|ui| {
                    role_tag(ui, role, palette);
                    ui.add_space(4.0);
                    if let Some(champ) = picked {
                        let portrait = textures.champion(egui_ctx, &champ.key);
                        avatar(ui, palette, &champ.name, 30.0, accent, portrait);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(&champ.name)
                                    .size(12.0)
                                    .strong()
                                    .color(palette.text),
                            );
                            let role_list = champ
                                .roles
                                .iter()
                                .map(|r| r.as_str())
                                .collect::<Vec<_>>()
                                .join(" / ");
                            ui.label(
                                RichText::new(role_list)
                                    .size(9.0)
                                    .color(palette.secondary),
                            );
                        });
                    } else {
                        let (placeholder_rect, _) =
                            ui.allocate_exact_size(Vec2::splat(30.0), Sense::hover());
                        ui.painter().rect_stroke(
                            placeholder_rect,
                            CornerRadius::same(6),
                            Stroke::new(1.0, palette.border),
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().text(
                            placeholder_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "+",
                            FontId::proportional(15.0),
                            palette.muted,
                        );
                        ui.label(
                            RichText::new("Empty slot")
                                .size(11.5)
                                .italics()
                                .color(palette.muted),
                        );
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if picked.is_some()
                            && action_button(ui, palette, "×", false, palette.red).clicked()
                        {
                            picks[index] = 0;
                        }
                        let selected_text = picked.map(|c| c.name.as_str()).unwrap_or("Pick…");
                        egui::ComboBox::from_id_salt(format!("{title}-{}-{index}", role.as_str()))
                            .selected_text(selected_text)
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut picks[index], 0, "Empty slot");
                                for champ in all_champs {
                                    ui.selectable_value(
                                        &mut picks[index],
                                        champ.id,
                                        champ.name.as_str(),
                                    );
                                }
                            });
                    });
                });
            });
            ui.add_space(4.0);
        }
    });
}

fn build_draft_inputs(ids: &[u32; 5]) -> Vec<DraftChampionInput> {
    ids.iter()
        .enumerate()
        .filter(|(_, &id)| id != 0)
        .filter_map(|(slot_idx, &id)| {
            let champ = champions::champion_by_id(id)?;
            let role = Role::ALL[slot_idx];
            let profile = builds::build_for_champion(champ.id, role)
                .or_else(|| builds::default_build_for_champion(champ.id));
            let archetype = profile
                .map(|p| p.archetype.as_str())
                .unwrap_or("DIVER_FIGHTER");
            Some(DraftChampionInput::from_archetype(
                champ.id,
                &champ.name,
                role.as_str(),
                archetype,
            ))
        })
        .collect()
}
