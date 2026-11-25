//! Champion Select workspace: searchable roster grid, role builds, skill progression,
//! full 9-perk + 2-spell editor, and live Champ Select session panel.

use crate::domain::builds::{self, BuildProfile};
use crate::domain::champions::{self, Champion, Role};
use crate::domain::protocol::{CoreCommand, CoreNotice};
use crate::domain::runes;
use crate::ui::pages::{preferred_spell_order, spell_name, PageContext, SUMMONER_SPELLS};
use crate::ui::textures::TextureManager;
use crate::ui::theme::{lerp_color, ThemePalette};
use crate::ui::widgets::{
    action_button, avatar, badge, filter_chip, inset_card, item_slot, page_heading, panel,
    panel_title, role_tag, rune_icon_card, search_box, spell_choice_chip, tree_accent,
};
use eframe::egui::{
    self, Align, Color32, CornerRadius, FontId, Layout, RichText, ScrollArea, Sense, Stroke, Vec2,
};
use serde_json::Value;

pub fn render(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;
    page_heading(
        ui,
        &palette,
        "PRE-GAME SYSTEM",
        "Champion Select",
        "Choose your champion, inspect role builds and skill paths, or tune every rune and spell.",
        ctx.snapshot,
    );

    let search_needle = ctx
        .state
        .champ_select
        .champion_search
        .trim()
        .to_ascii_lowercase();
    let role_filter = ctx.state.champ_select.role_filter;
    let all_champs = champions::all_champions();
    let matches: Vec<&'static Champion> = all_champs
        .iter()
        .filter(|champ| {
            let matches_search = search_needle.is_empty()
                || champ.name.to_ascii_lowercase().contains(&search_needle)
                || champ.title.to_ascii_lowercase().contains(&search_needle)
                || champ
                    .roles
                    .iter()
                    .any(|role| role.as_str().to_ascii_lowercase().contains(&search_needle));
            let matches_role = role_filter.is_none_or(|role| champ.supports_role(role));
            matches_search && matches_role
        })
        .collect();

    panel(ui, &palette, |ui| {
        ui.horizontal_wrapped(|ui| {
            let search_w = (ui.available_width() - 290.0).clamp(180.0, 420.0);
            search_box(
                ui,
                &palette,
                "champ-select-search",
                &mut ctx.state.champ_select.champion_search,
                &format!("Find a champion · {} available", all_champs.len()),
                Some(search_w),
            );

            let mut auto_sync = ctx.store.settings.auto_import_runes
                && ctx.store.settings.auto_import_spells
                && ctx.store.settings.auto_import_item_sets;
            if ui.checkbox(&mut auto_sync, "AUTO-SYNC").changed() {
                ctx.store.settings.auto_import_runes = auto_sync;
                ctx.store.settings.auto_import_spells = auto_sync;
                ctx.store.settings.auto_import_item_sets = auto_sync;
                ctx.request_save();
            }

            if action_button(ui, &palette, "REFRESH CLIENT", false, palette.accent).clicked() {
                ctx.send_command(CoreCommand::Refresh);
                ctx.set_notice(CoreNotice::Info(
                    "Refreshing League Client connection…".to_string(),
                ));
            }
        });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if filter_chip(
                ui,
                &palette,
                "ALL",
                ctx.state.champ_select.role_filter.is_none(),
                palette.accent,
            )
            .clicked()
            {
                ctx.state.champ_select.role_filter = None;
            }
            for role in Role::ALL {
                if filter_chip(
                    ui,
                    &palette,
                    role.as_str(),
                    ctx.state.champ_select.role_filter == Some(role),
                    palette.accent,
                )
                .clicked()
                {
                    ctx.state.champ_select.role_filter = Some(role);
                }
            }
            ui.add_space(6.0);
            ui.label(
                RichText::new(format!("{} of {}", matches.len(), all_champs.len()))
                    .size(10.0)
                    .color(palette.muted),
            );
        });
    });

    ui.add_space(8.0);
    let pane_height = (ui.available_height() - 4.0).clamp(280.0, 620.0);
    let total_w = ui.available_width();
    let left_w = (total_w * 0.45).clamp(310.0, 500.0);
    let right_w = (total_w - left_w - 10.0).max(320.0);
    let mut clicked_champion: Option<u32> = None;

    ui.horizontal_top(|ui| {
        ui.allocate_ui_with_layout(
            Vec2::new(left_w, pane_height),
            Layout::top_down(Align::LEFT),
            |ui| {
                panel(ui, &palette, |ui| {
                    ui.set_min_height((pane_height - 30.0).max(240.0));
                    panel_title(
                        ui,
                        &palette,
                        "CHOOSE CHAMPION",
                        &format!("{} champions match", matches.len()),
                    );
                    let avail_width = ui.available_width();
                    let num_cols = ((avail_width - 14.0) / 102.0).floor().max(3.0) as usize;
                    let card_w = ((avail_width - 14.0 - ((num_cols - 1) as f32 * 8.0))
                        / num_cols as f32)
                        .max(76.0);

                    ScrollArea::vertical()
                        .id_salt("champion-roster-scroll-v4")
                        .auto_shrink([false, false])
                        .scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                        )
                        .max_height((pane_height - 92.0).max(180.0))
                        .show(ui, |ui| {
                            let egui_ctx = ui.ctx().clone();
                            egui::Grid::new("champion-grid-v4")
                                .num_columns(num_cols)
                                .spacing([8.0, 8.0])
                                .show(ui, |ui| {
                                    for (index, champ) in matches.iter().enumerate() {
                                        let selected =
                                            ctx.state.champ_select.selected_champion == champ.id;
                                        let (rect, response) = ui.allocate_exact_size(
                                            Vec2::new(card_w, 108.0),
                                            Sense::click(),
                                        );
                                        let anim_id =
                                            ui.make_persistent_id(("champion-card", champ.id));
                                        let hover_t = ui.ctx().animate_bool_with_time(
                                            anim_id.with("hover"),
                                            response.hovered(),
                                            0.14,
                                        );
                                        let selected_t = ui.ctx().animate_bool_with_time(
                                            anim_id.with("selected"),
                                            selected,
                                            0.18,
                                        );
                                        let emphasis = selected_t.max(hover_t * 0.52);
                                        let border =
                                            lerp_color(palette.border, palette.accent, emphasis);
                                        ui.painter().rect_filled(
                                            rect,
                                            CornerRadius::same(10),
                                            lerp_color(
                                                palette.card,
                                                ThemePalette::with_alpha(palette.accent, 38),
                                                emphasis,
                                            ),
                                        );
                                        ui.painter().rect_stroke(
                                            rect,
                                            CornerRadius::same(10),
                                            Stroke::new(1.0 + selected_t, border),
                                            egui::StrokeKind::Inside,
                                        );
                                        if let Some(texture) =
                                            ctx.textures.champion(&egui_ctx, &champ.key)
                                        {
                                            let portrait_size = 64.0 + 4.0 * hover_t + selected_t;
                                            let image_rect = egui::Rect::from_center_size(
                                                egui::pos2(rect.center().x, rect.top() + 38.0),
                                                Vec2::splat(portrait_size),
                                            );
                                            egui::Image::new((texture.id(), image_rect.size()))
                                                .corner_radius(CornerRadius::same(9))
                                                .paint_at(ui, image_rect);
                                        } else {
                                            ui.painter().text(
                                                egui::pos2(rect.center().x, rect.top() + 38.0),
                                                egui::Align2::CENTER_CENTER,
                                                crate::ui::widgets::initials(&champ.name),
                                                FontId::proportional(20.0),
                                                palette.accent,
                                            );
                                        }
                                        ui.painter().rect_filled(
                                            egui::Rect::from_min_max(
                                                egui::pos2(rect.left() + 1.0, rect.bottom() - 26.0),
                                                egui::pos2(rect.right() - 1.0, rect.bottom() - 1.0),
                                            ),
                                            CornerRadius::same(9),
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
                                                CornerRadius::same(2),
                                                ThemePalette::with_alpha(
                                                    palette.accent,
                                                    (selected_t * 235.0) as u8,
                                                ),
                                            );
                                        }
                                        ui.painter().text(
                                            egui::pos2(rect.center().x, rect.bottom() - 13.0),
                                            egui::Align2::CENTER_CENTER,
                                            &champ.name,
                                            FontId::proportional(10.5),
                                            lerp_color(palette.text, palette.accent, selected_t),
                                        );
                                        if response.clicked() {
                                            clicked_champion = Some(champ.id);
                                        }
                                        if index % num_cols == num_cols - 1 {
                                            ui.end_row();
                                        }
                                    }
                                });

                            if matches.is_empty() {
                                inset_card(ui, &palette, palette.border, |ui| {
                                    ui.label(
                                        RichText::new("No champions match those filters.")
                                            .color(palette.secondary),
                                    );
                                });
                            }
                        });
                });
            },
        );

        ui.add_space(10.0);

        ui.allocate_ui_with_layout(
            Vec2::new(right_w, pane_height),
            Layout::top_down(Align::LEFT),
            |ui| {
                panel(ui, &palette, |ui| {
                    ui.set_min_height((pane_height - 30.0).max(240.0));
                    ScrollArea::vertical()
                        .id_salt("champion-details-scroll-v4")
                        .auto_shrink([false, false])
                        .scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                        )
                        .max_height((pane_height - 46.0).max(220.0))
                        .show(ui, |ui| {
                            if let Some(champ) =
                                champions::champion_by_id(ctx.state.champ_select.selected_champion)
                            {
                                render_champion_details(ctx, ui, champ);
                            }
                        });
                });
            },
        );
    });

    if let Some(champ_id) = clicked_champion {
        ctx.select_champion_setup(champ_id, None, true);
    }
}

fn render_champion_details(ctx: &mut PageContext<'_>, ui: &mut egui::Ui, champ: &'static Champion) {
    let palette = *ctx.palette;
    let role = ctx.state.champ_select.build_role;
    let recommendation = builds::build_for_champion(champ.id, role)
        .cloned()
        .unwrap_or_else(|| builds::recommend(&champ.key, &champ.name, &champ.roles, Some(role)));

    // Champion Hero Card
    inset_card(ui, &palette, palette.border, |ui| {
        ui.horizontal(|ui| {
            let egui_ctx = ui.ctx().clone();
            let portrait = ctx.textures.champion(&egui_ctx, &champ.key);
            avatar(ui, &palette, &champ.name, 56.0, palette.accent, portrait);
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(champ.name.to_ascii_uppercase())
                            .size(19.0)
                            .strong()
                            .color(palette.text),
                    );
                    badge(ui, &recommendation.archetype.display_name(), palette.accent);
                });
                ui.label(
                    RichText::new(&champ.title)
                        .size(11.0)
                        .color(palette.secondary),
                );
                ui.add_space(3.0);
                ui.horizontal_wrapped(|ui| {
                    for &r in &champ.roles {
                        role_tag(ui, r, &palette);
                    }
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!(
                            "Skill Max: {}",
                            recommendation.compact_skill_priority()
                        ))
                        .size(10.5)
                        .strong()
                        .color(palette.gold),
                    );
                });
            });
        });
    });

    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        for (tab, label, accent) in [
            (0usize, "BUILD", palette.accent),
            (1, "RUNES & SPELLS", palette.gold),
            (2, "SESSION", palette.secondary),
        ] {
            if filter_chip(
                ui,
                &palette,
                label,
                ctx.state.champ_select.detail_tab == tab,
                accent,
            )
            .clicked()
            {
                ctx.state.champ_select.detail_tab = tab;
            }
        }
    });
    ui.add_space(6.0);

    match ctx.state.champ_select.detail_tab {
        0 => render_build_tab(ctx, ui, champ, &recommendation),
        1 => render_rune_editor(ctx, ui, champ, &recommendation),
        _ => render_team_panel(ctx, ui),
    }
}

fn render_build_tab(
    ctx: &mut PageContext<'_>,
    ui: &mut egui::Ui,
    champ: &'static Champion,
    initial_rec: &BuildProfile,
) {
    let palette = *ctx.palette;
    let mut role_changed = false;

    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            "CHAMPION BUILD",
            "Role-specific starting path, skill order, item sets, and shop export",
        );

        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("ROLE")
                    .size(9.5)
                    .strong()
                    .color(palette.muted),
            );
            let mut active_role = ctx.state.champ_select.build_role;
            egui::ComboBox::from_id_salt("champion-build-role-v4")
                .selected_text(active_role.as_str())
                .show_ui(ui, |ui| {
                    let available_roles: &[Role] = if champ.roles.is_empty() {
                        &Role::ALL
                    } else {
                        &champ.roles
                    };
                    for &r in available_roles {
                        if ui
                            .selectable_value(&mut active_role, r, r.as_str())
                            .changed()
                        {
                            role_changed = true;
                        }
                    }
                    // Also allow off-meta roles cleanly separated
                    for &r in &Role::ALL {
                        if !available_roles.contains(&r)
                            && ui
                                .selectable_value(
                                    &mut active_role,
                                    r,
                                    format!("{} (Flex)", r.as_str()),
                                )
                                .changed()
                        {
                            role_changed = true;
                        }
                    }
                });
            if role_changed {
                ctx.state.champ_select.build_role = active_role;
                ctx.persist_active_champion_setup();
            }
            badge(ui, &initial_rec.archetype.display_name(), palette.accent);
        });

        let rec = if role_changed {
            builds::build_for_champion(champ.id, ctx.state.champ_select.build_role)
                .cloned()
                .unwrap_or_else(|| {
                    builds::recommend(
                        &champ.key,
                        &champ.name,
                        &champ.roles,
                        Some(ctx.state.champ_select.build_role),
                    )
                })
        } else {
            initial_rec.clone()
        };

        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("SKILL PRIORITY")
                    .size(9.5)
                    .strong()
                    .color(palette.muted),
            );
            ui.label(
                RichText::new(rec.compact_skill_priority())
                    .size(12.0)
                    .strong()
                    .color(palette.gold),
            );
        });
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (idx, skill) in rec.skill_order.iter().enumerate() {
                let lvl = idx + 1;
                let color = match skill.as_str() {
                    "R" => palette.gold,
                    "Q" => palette.accent,
                    "W" => palette.green,
                    _ => palette.violet,
                };
                egui::Frame::new()
                    .fill(ThemePalette::with_alpha(color, 26))
                    .stroke(Stroke::new(0.8, ThemePalette::with_alpha(color, 110)))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(egui::Margin::symmetric(5, 3))
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(lvl.to_string())
                                    .size(7.5)
                                    .color(palette.muted),
                            );
                            ui.label(RichText::new(skill).size(9.5).strong().color(color));
                        });
                    });
            }
        });

        ui.add_space(8.0);
        ui.label(
            RichText::new("STARTING & BOOTS")
                .size(9.5)
                .strong()
                .color(palette.muted),
        );
        ui.horizontal_wrapped(|ui| {
            for &item_id in &rec.starting_items {
                item_slot(ui, &palette, ctx.textures, item_id, 26.0, true);
            }
            item_slot(ui, &palette, ctx.textures, rec.boots, 26.0, true);
        });

        ui.add_space(6.0);
        ui.label(
            RichText::new("CORE ITEMS")
                .size(9.5)
                .strong()
                .color(palette.muted),
        );
        ui.horizontal_wrapped(|ui| {
            for &item_id in &rec.core_items {
                item_slot(ui, &palette, ctx.textures, item_id, 28.0, true);
            }
        });

        ui.add_space(6.0);
        ui.label(
            RichText::new("SITUATIONAL OPTIONS")
                .size(9.5)
                .strong()
                .color(palette.muted),
        );
        ui.horizontal_wrapped(|ui| {
            for &item_id in &rec.situational_items {
                item_slot(ui, &palette, ctx.textures, item_id, 24.0, true);
            }
        });

        ui.add_space(6.0);
        ui.add(
            egui::Label::new(RichText::new(&rec.tip).size(10.5).color(palette.secondary)).wrap(),
        );

        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if action_button(ui, &palette, "APPLY BUILD RUNES", false, palette.accent).clicked() {
                let cs = &mut ctx.state.champ_select;
                cs.primary_tree = rec.primary_tree;
                cs.secondary_tree = rec.secondary_tree;
                cs.selected_perks = std::array::from_fn(|index| {
                    rec.rune_ids
                        .get(index)
                        .copied()
                        .map(i64::from)
                        .unwrap_or(runes::SHARD_ADAPTIVE_FORCE)
                });
                cs.secondary_rune_rows = runes::secondary_rows_for_perks(
                    cs.secondary_tree,
                    cs.selected_perks[4],
                    cs.selected_perks[5],
                );
                runes::normalize_rune_setup(
                    cs.primary_tree,
                    cs.secondary_tree,
                    &mut cs.secondary_rune_rows,
                    &mut cs.selected_perks,
                );
                let page = builds::rune_page(&champ.name, &rec);
                ctx.send_command(CoreCommand::ImportRunes(page.to_json()));
                ctx.persist_active_champion_setup();
                ctx.set_notice(CoreNotice::Info(
                    "Sending recommended rune page to League Client…".to_string(),
                ));
            }

            if action_button(ui, &palette, "APPLY BUILD SPELLS", false, palette.red).clicked() {
                let (spell_one, spell_two) = preferred_spell_order(
                    rec.spell_one,
                    rec.spell_two,
                    &ctx.store.settings.flash_slot_preference,
                );
                ctx.state.champ_select.spell_one = spell_one;
                ctx.state.champ_select.spell_two = spell_two;
                ctx.send_command(CoreCommand::ImportSpells {
                    spell1: spell_one,
                    spell2: spell_two,
                });
                ctx.persist_active_champion_setup();
                ctx.set_notice(CoreNotice::Info(
                    "Sending recommended summoner spells to League Client…".to_string(),
                ));
            }

            if action_button(ui, &palette, "EXPORT ITEM SET", false, palette.gold).clicked() {
                ctx.send_command(CoreCommand::ExportItemSet(builds::champion_item_set_json(
                    champ, &rec,
                )));
                ctx.set_notice(CoreNotice::Info(
                    "Adding champion build to the in-game shop…".to_string(),
                ));
            }
        });
    });
}

fn render_rune_editor(
    ctx: &mut PageContext<'_>,
    ui: &mut egui::Ui,
    champ: &'static Champion,
    recommendation: &BuildProfile,
) {
    let palette = *ctx.palette;
    let egui_ctx = ui.ctx().clone();
    let mut edited = false;

    panel(ui, &palette, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("RUNE SETUP")
                        .size(14.0)
                        .strong()
                        .color(palette.text),
                );
                ui.label(
                    RichText::new(format!(
                        "{} · {}",
                        champ.name,
                        ctx.state.champ_select.build_role.as_str()
                    ))
                    .size(10.0)
                    .color(palette.secondary),
                );
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                badge(ui, "9 PICKS", palette.accent);
            });
        });

        let meta_perks = std::array::from_fn(|index| {
            recommendation
                .rune_ids
                .get(index)
                .copied()
                .map(i64::from)
                .unwrap_or(runes::SHARD_ADAPTIVE_FORCE)
        });
        let burst_perks = runes::burst_preset_perks();
        let sustain_perks = [
            runes::rune_id_preferred(runes::TREE_RESOLVE, 0, &["Grasp of the Undying"]),
            runes::rune_id_preferred(runes::TREE_RESOLVE, 1, &["Demolish"]),
            runes::rune_id_preferred(runes::TREE_RESOLVE, 2, &["Bone Plating"]),
            runes::rune_id_preferred(runes::TREE_RESOLVE, 3, &["Overgrowth"]),
            runes::rune_id_preferred(runes::TREE_PRECISION, 1, &["Triumph"]),
            runes::rune_id_preferred(
                runes::TREE_PRECISION,
                2,
                &["Legend: Haste", "Legend: Alacrity"],
            ),
            runes::SHARD_ADAPTIVE_FORCE,
            runes::SHARD_ADAPTIVE_FORCE,
            runes::SHARD_SCALING_HEALTH,
        ];

        let mut selected_preset = None;
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("PRESET")
                    .size(8.5)
                    .strong()
                    .color(palette.muted),
            );
            for (name, detail, primary, secondary, perks, accent) in [
                (
                    "META",
                    format!(
                        "{} build",
                        ctx.state
                            .champ_select
                            .build_role
                            .as_str()
                            .to_ascii_lowercase()
                    ),
                    recommendation.primary_tree,
                    recommendation.secondary_tree,
                    meta_perks,
                    palette.accent,
                ),
                (
                    "BURST",
                    "Electrocute + Grisly Mementos".to_string(),
                    runes::TREE_DOMINATION,
                    runes::TREE_SORCERY,
                    burst_perks,
                    palette.red,
                ),
                (
                    "SUSTAIN",
                    "Safer trades".to_string(),
                    runes::TREE_RESOLVE,
                    runes::TREE_PRECISION,
                    sustain_perks,
                    palette.green,
                ),
            ] {
                let response = filter_chip(ui, &palette, name, false, accent);
                if response.clicked() {
                    selected_preset = Some((primary, secondary, perks));
                }
                response.on_hover_text(detail);
            }
        });

        if let Some((primary, secondary, perks)) = selected_preset {
            let cs = &mut ctx.state.champ_select;
            cs.primary_tree = primary;
            cs.secondary_tree = secondary;
            cs.selected_perks = perks;
            cs.secondary_rune_rows = runes::secondary_rows_for_perks(secondary, perks[4], perks[5]);
            edited = true;
        }

        ui.add_space(6.0);
        let primary_color = tree_accent(ctx.state.champ_select.primary_tree, &palette);
        rune_section_heading(
            ui,
            &palette,
            "PRIMARY STYLE",
            runes::tree_name(ctx.state.champ_select.primary_tree),
            primary_color,
        );
        if let Some(tree) = rune_tree_picker(
            ui,
            &palette,
            ctx.state.champ_select.primary_tree,
            None,
            ctx.textures,
            &egui_ctx,
        ) {
            let cs = &mut ctx.state.champ_select;
            cs.primary_tree = tree;
            if cs.secondary_tree == tree {
                cs.secondary_tree = runes::RUNE_TREE_NAMES
                    .iter()
                    .find(|(id, _)| *id != tree)
                    .map(|(id, _)| *id)
                    .unwrap_or(runes::TREE_RESOLVE);
            }
            edited = true;
        }

        ui.add_space(4.0);
        rune_section_heading(
            ui,
            &palette,
            "SECONDARY STYLE",
            runes::tree_name(ctx.state.champ_select.secondary_tree),
            palette.gold,
        );
        if let Some(tree) = rune_tree_picker(
            ui,
            &palette,
            ctx.state.champ_select.secondary_tree,
            Some(ctx.state.champ_select.primary_tree),
            ctx.textures,
            &egui_ctx,
        ) {
            ctx.state.champ_select.secondary_tree = tree;
            edited = true;
        }

        {
            let cs = &mut ctx.state.champ_select;
            runes::normalize_rune_setup(
                cs.primary_tree,
                cs.secondary_tree,
                &mut cs.secondary_rune_rows,
                &mut cs.selected_perks,
            );
        }

        ui.add_space(8.0);
        let primary_tree = ctx.state.champ_select.primary_tree;
        let secondary_tree = ctx.state.champ_select.secondary_tree;
        let p_color = tree_accent(primary_tree, &palette);
        let s_color = tree_accent(secondary_tree, &palette);

        ui.columns(2, |columns| {
            inset_card(&mut columns[0], &palette, p_color, |ui| {
                let keystone_opts = runes::rune_options(primary_tree, 0);
                rune_section_heading(
                    ui,
                    &palette,
                    "KEYSTONE",
                    selected_rune_name(&keystone_opts, ctx.state.champ_select.selected_perks[0]),
                    p_color,
                );
                if let Some(id) = rune_icon_choice_line(
                    ui,
                    &palette,
                    &keystone_opts,
                    ctx.state.champ_select.selected_perks[0],
                    ctx.textures,
                    &egui_ctx,
                    p_color,
                    40.0,
                ) {
                    ctx.state.champ_select.selected_perks[0] = id;
                    edited = true;
                }

                for row in 1..=3 {
                    ui.add_space(4.0);
                    let opts = runes::rune_options(primary_tree, row);
                    rune_section_heading(
                        ui,
                        &palette,
                        &format!("PRIMARY ROW {row}"),
                        selected_rune_name(&opts, ctx.state.champ_select.selected_perks[row]),
                        p_color,
                    );
                    if let Some(id) = rune_icon_choice_line(
                        ui,
                        &palette,
                        &opts,
                        ctx.state.champ_select.selected_perks[row],
                        ctx.textures,
                        &egui_ctx,
                        p_color,
                        34.0,
                    ) {
                        ctx.state.champ_select.selected_perks[row] = id;
                        edited = true;
                    }
                }
            });

            inset_card(&mut columns[1], &palette, s_color, |ui| {
                rune_section_heading(
                    ui,
                    &palette,
                    "SECONDARY RUNES",
                    runes::tree_name(secondary_tree),
                    s_color,
                );
                for choice in 0..2 {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("PICK {}", choice + 1))
                                .size(8.0)
                                .strong()
                                .color(palette.muted),
                        );
                        for row in 1..=3 {
                            let enabled =
                                row != ctx.state.champ_select.secondary_rune_rows[1 - choice];
                            let selected =
                                ctx.state.champ_select.secondary_rune_rows[choice] == row;
                            let response = ui.add_enabled_ui(enabled, |ui| {
                                filter_chip(
                                    ui,
                                    &palette,
                                    &format!("R{row}"),
                                    selected,
                                    palette.gold,
                                )
                            });
                            if response.inner.clicked() {
                                ctx.state.champ_select.secondary_rune_rows[choice] = row;
                                edited = true;
                            }
                        }
                    });
                    let row = ctx.state.champ_select.secondary_rune_rows[choice];
                    let opts = runes::rune_options(secondary_tree, row);
                    let slot = 4 + choice;
                    rune_section_heading(
                        ui,
                        &palette,
                        &format!("ROW {row}"),
                        selected_rune_name(&opts, ctx.state.champ_select.selected_perks[slot]),
                        s_color,
                    );
                    if let Some(id) = rune_icon_choice_line(
                        ui,
                        &palette,
                        &opts,
                        ctx.state.champ_select.selected_perks[slot],
                        ctx.textures,
                        &egui_ctx,
                        s_color,
                        34.0,
                    ) {
                        ctx.state.champ_select.selected_perks[slot] = id;
                        edited = true;
                    }
                }

                ui.add_space(6.0);
                rune_section_heading(
                    ui,
                    &palette,
                    "STAT SHARDS",
                    "Offense · Flex · Defense",
                    palette.accent,
                );
                for (row, label) in [(0usize, "OFFENSE"), (1, "FLEX"), (2, "DEFENSE")] {
                    let options: Vec<(i64, &str)> = runes::stat_shard_options(row)
                        .iter()
                        .map(|s| (s.id, s.name))
                        .collect();
                    let slot = 6 + row;
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(label).size(8.0).strong().color(palette.muted));
                        ui.label(
                            RichText::new(selected_rune_name(
                                &options,
                                ctx.state.champ_select.selected_perks[slot],
                            ))
                            .size(8.5)
                            .color(palette.secondary),
                        );
                    });
                    if let Some(id) = rune_icon_choice_line(
                        ui,
                        &palette,
                        &options,
                        ctx.state.champ_select.selected_perks[slot],
                        ctx.textures,
                        &egui_ctx,
                        palette.accent,
                        30.0,
                    ) {
                        ctx.state.champ_select.selected_perks[slot] = id;
                        edited = true;
                    }
                }
            });
        });

        ui.add_space(8.0);
        inset_card(ui, &palette, palette.border, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (slot, label, id) in [
                    (0usize, "KEY [ D ]", ctx.state.champ_select.spell_one),
                    (1usize, "KEY [ F ]", ctx.state.champ_select.spell_two),
                ] {
                    if filter_chip(
                        ui,
                        &palette,
                        &format!("{label} · {}", spell_name(id)),
                        ctx.state.champ_select.active_spell_slot == slot,
                        if slot == 0 {
                            palette.accent
                        } else {
                            palette.gold
                        },
                    )
                    .clicked()
                    {
                        ctx.state.champ_select.active_spell_slot = slot;
                    }
                }
                ui.label(
                    RichText::new("Select slot, then click spell below to assign")
                        .size(9.0)
                        .color(palette.muted),
                );
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for &(id, name) in SUMMONER_SPELLS {
                    let active_slot = ctx.state.champ_select.active_spell_slot;
                    let selected_here = if active_slot == 0 {
                        ctx.state.champ_select.spell_one == id
                    } else {
                        ctx.state.champ_select.spell_two == id
                    };
                    let selected_other = if active_slot == 0 {
                        ctx.state.champ_select.spell_two == id
                    } else {
                        ctx.state.champ_select.spell_one == id
                    };
                    let response = spell_choice_chip(
                        ui,
                        &palette,
                        name,
                        selected_here,
                        selected_other,
                        if active_slot == 0 {
                            palette.accent
                        } else {
                            palette.gold
                        },
                    );
                    if response.clicked() {
                        let cs = &mut ctx.state.champ_select;
                        if cs.active_spell_slot == 0 {
                            if cs.spell_two == id {
                                std::mem::swap(&mut cs.spell_one, &mut cs.spell_two);
                            } else {
                                cs.spell_one = id;
                            }
                        } else if cs.spell_one == id {
                            std::mem::swap(&mut cs.spell_one, &mut cs.spell_two);
                        } else {
                            cs.spell_two = id;
                        }
                        edited = true;
                    }
                }
            });
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Customizations are saved persistently per champion.")
                    .size(9.5)
                    .color(palette.muted),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if action_button(ui, &palette, "APPLY RUNES + SPELLS", true, palette.accent)
                    .clicked()
                {
                    let cs = &ctx.state.champ_select;
                    let page = runes::build_rune_page(
                        &format!("RiftCore {}", champ.name),
                        cs.primary_tree,
                        cs.secondary_tree,
                        &cs.selected_perks,
                    );
                    let (spell1, spell2) = (cs.spell_one, cs.spell_two);
                    ctx.send_command(CoreCommand::ImportRunes(page.to_json()));
                    ctx.send_command(CoreCommand::ImportSpells { spell1, spell2 });
                    ctx.persist_active_champion_setup();
                    ctx.set_notice(CoreNotice::Info(
                        "Sending rune page and summoner spells to League Client…".to_string(),
                    ));
                }
            });
        });
    });

    if edited {
        let cs = &mut ctx.state.champ_select;
        runes::normalize_rune_setup(
            cs.primary_tree,
            cs.secondary_tree,
            &mut cs.secondary_rune_rows,
            &mut cs.selected_perks,
        );
        ctx.persist_active_champion_setup();
    }
}

/// Shared Champ Select live session team breakdown used by Champ Select, Draft Coach, and Lobby Scout.
pub fn render_team_panel(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;
    let egui_ctx = ui.ctx().clone();

    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            "CHAMP SELECT SESSION",
            "Live picks from your current League Client lobby",
        );
        if let Some(session) = ctx.snapshot.champ_select.as_ref() {
            ui.columns(2, |cols| {
                for (column, key, heading, accent) in [
                    (0, "myTeam", "YOUR TEAM", palette.accent),
                    (1, "theirTeam", "OPPOSING TEAM", palette.red),
                ] {
                    cols[column].label(RichText::new(heading).size(10.0).strong().color(accent));
                    if let Some(members) = session.get(key).and_then(Value::as_array) {
                        for member in members.iter().take(5) {
                            let id = member
                                .get("championId")
                                .and_then(Value::as_u64)
                                .filter(|&v| v > 0)
                                .or_else(|| {
                                    member
                                        .get("championPickIntent")
                                        .and_then(Value::as_u64)
                                        .filter(|&v| v > 0)
                                })
                                .unwrap_or(0) as u32;
                            let champ = champions::champion_by_id(id);
                            let name = if id == 0 {
                                "Choosing…".to_string()
                            } else {
                                champ
                                    .map(|entry| entry.name.clone())
                                    .unwrap_or_else(|| format!("Champion {id}"))
                            };
                            let role_raw = member
                                .get("assignedPosition")
                                .and_then(Value::as_str)
                                .unwrap_or("");
                            let role = Role::from_str_lossy(role_raw);

                            cols[column].horizontal(|ui| {
                                let portrait =
                                    champ.and_then(|c| ctx.textures.champion(&egui_ctx, &c.key));
                                avatar(ui, &palette, &name, 28.0, accent, portrait);
                                ui.label(
                                    RichText::new(name).size(11.5).strong().color(palette.text),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if let Some(r) = role {
                                        role_tag(ui, r, &palette);
                                    } else {
                                        ui.label(
                                            RichText::new("FLEX").size(9.0).color(palette.muted),
                                        );
                                    }
                                });
                            });
                            cols[column].add_space(3.0);
                        }
                    }
                }
            });
        } else {
            inset_card(ui, &palette, palette.border, |ui| {
                ui.horizontal(|ui| {
                    avatar(ui, &palette, "LS", 40.0, palette.accent, None);
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(if ctx.snapshot.connected {
                                "LOBBY SCOUT STANDBY"
                            } else {
                                "LEAGUE CLIENT DISCONNECTED"
                            })
                            .size(13.0)
                            .strong()
                            .color(palette.text),
                        );
                        ui.add(
                            egui::Label::new(
                                RichText::new(if ctx.snapshot.connected {
                                    "Enter Champion Select to detect live team picks, assigned roles, and opponent selections."
                                } else {
                                    "Launch League of Legends to connect live session intelligence and rune auto-import."
                                })
                                .size(11.0)
                                .color(palette.secondary),
                            )
                            .wrap(),
                        );
                    });
                });
            });
        }
    });
}

fn selected_rune_name<'a>(options: &[(i64, &'a str)], selected: i64) -> &'a str {
    options
        .iter()
        .find(|(id, _)| *id == selected)
        .map(|(_, name)| *name)
        .unwrap_or("Select rune")
}

fn rune_tree_picker(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    selected: u32,
    excluded: Option<u32>,
    textures: &mut TextureManager,
    egui_ctx: &egui::Context,
) -> Option<u32> {
    let mut clicked = None;
    ui.horizontal_wrapped(|ui| {
        for &(id, name) in runes::RUNE_TREE_NAMES {
            if Some(id) == excluded {
                continue;
            }
            let accent = tree_accent(id, palette);
            let tex = textures.rune(egui_ctx, id as i64);
            let response =
                rune_icon_card(ui, palette, id as i64, selected == id, accent, 34.0, tex);
            if response.clicked() {
                clicked = Some(id);
            }
            response.on_hover_text(name);
        }
    });
    clicked
}

fn rune_section_heading(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    title: &str,
    detail: &str,
    accent: Color32,
) {
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
        ui.painter().circle_filled(rect.center(), 3.5, accent);
        ui.label(RichText::new(title).size(9.5).strong().color(palette.text));
        ui.label(RichText::new(detail).size(9.0).color(palette.muted));
    });
    ui.add_space(3.0);
}

#[allow(clippy::too_many_arguments)]
fn rune_icon_choice_line(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    options: &[(i64, &str)],
    selected: i64,
    textures: &mut TextureManager,
    egui_ctx: &egui::Context,
    accent: Color32,
    side: f32,
) -> Option<i64> {
    if options.is_empty() {
        return None;
    }
    let mut clicked = None;
    ui.horizontal_wrapped(|ui| {
        for &(id, name) in options {
            let tex = textures.rune(egui_ctx, id);
            let response = rune_icon_card(ui, palette, id, selected == id, accent, side, tex);
            if response.clicked() {
                clicked = Some(id);
            }
            let desc = runes::clean_rune_description(id, 180);
            response.on_hover_text(if desc.is_empty() {
                name.to_string()
            } else {
                format!("{name}\n{desc}")
            });
        }
    });
    clicked
}
