//! TFT Hub page: Set 18 unit/trait/item browser, Set 13 archived comps, crafting mixer, roll odds, and 4x7 tactical board.

use crate::domain::tft::{self, TftCombinedRecipe, TftEntry, TftRollOddsRow};
use crate::ui::pages::{PageContext, TftDataCategory, TftTab};
use crate::ui::theme::ThemePalette;
use crate::ui::widgets::{
    action_button, avatar, badge, filter_chip, inset_card, page_heading, panel, panel_title,
    search_box,
};
use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Vec2};
use serde_json::Value;

pub fn render(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;
    let meta = tft::set_metadata();

    page_heading(
        ui,
        &palette,
        "TFT HUB",
        "TFT Companion",
        &format!(
            "Set {} (patch {}) tactical reference plus archived Set 13 comps, item mixer, and 4x7 board planner",
            meta.set, meta.patch_version
        ),
        ctx.snapshot,
    );

    ui.horizontal_wrapped(|ui| {
        for (tab, label) in [
            (TftTab::Comps, "COMPS ARCHIVE"),
            (TftTab::Items, "ITEMS & MIXER"),
            (TftTab::Odds, "ROLL ODDS"),
            (TftTab::Augments, "AUGMENTS"),
            (TftTab::Board, "BOARD PLANNER"),
            (TftTab::SetData, "SET 18 DATA"),
        ] {
            if filter_chip(
                ui,
                &palette,
                label,
                ctx.state.tft.tab == tab,
                palette.accent,
            )
            .clicked()
            {
                ctx.state.tft.tab = tab;
            }
        }
    });

    ui.add_space(10.0);

    match ctx.state.tft.tab {
        TftTab::Comps => render_comps_tab(ctx, ui),
        TftTab::Items => render_items_tab(ctx, ui),
        TftTab::Odds => render_odds_tab(ctx, ui),
        TftTab::Augments => render_augments_tab(ctx, ui),
        TftTab::Board => render_board_tab(ctx, ui),
        TftTab::SetData => render_set_data_tab(ctx, ui),
    }
}

fn render_comps_tab(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let comps = legacy_comps();
    let query = ctx.state.tft.comp_search.trim().to_ascii_lowercase();
    let filtered: Vec<&TftLegacyComp> = comps
        .iter()
        .filter(|comp| {
            query.is_empty()
                || comp.name.to_ascii_lowercase().contains(&query)
                || comp.style.to_ascii_lowercase().contains(&query)
                || comp
                    .traits
                    .iter()
                    .any(|t| t.to_ascii_lowercase().contains(&query))
                || comp
                    .units
                    .iter()
                    .any(|u| u.to_ascii_lowercase().contains(&query))
        })
        .collect();

    if ctx.state.tft.selected_comp.is_empty() {
        if let Some(first) = filtered.first() {
            ctx.state.tft.selected_comp = first.id.clone();
        }
    }

    let available_width = ui.available_width();
    let stacked = available_width < 880.0;

    if stacked {
        render_comp_list(ctx, ui, &filtered);
        ui.add_space(10.0);
        render_comp_detail(ctx, ui, &filtered);
    } else {
        let left_width = (available_width * 0.36).clamp(280.0, 360.0);
        let right_width = (available_width - left_width - 12.0).max(380.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(left_width, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| render_comp_list(ctx, ui, &filtered),
            );
            ui.add_space(12.0);
            ui.allocate_ui_with_layout(
                Vec2::new(right_width, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| render_comp_detail(ctx, ui, &filtered),
            );
        });
    }
}

fn render_comp_list(ctx: &mut PageContext<'_>, ui: &mut egui::Ui, filtered: &[&TftLegacyComp]) {
    let palette = *ctx.palette;
    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            "ARCHIVED SET 13 COMPS",
            "Reference compositions preserved offline",
        );
        search_box(
            ui,
            &palette,
            "tft-comp-search",
            &mut ctx.state.tft.comp_search,
            "Search comp, unit, trait, or style...",
            None,
        );
        ui.add_space(8.0);

        if filtered.is_empty() {
            ui.label(
                RichText::new("No archived TFT comps match the current filter.")
                    .size(11.5)
                    .color(palette.muted),
            );
            return;
        }

        egui::ScrollArea::vertical()
            .id_salt("tft-comps-list")
            .max_height(460.0)
            .show(ui, |ui| {
                for comp in filtered {
                    let selected = ctx.state.tft.selected_comp == comp.id;
                    let fill = if selected {
                        ThemePalette::with_alpha(palette.accent, 28)
                    } else {
                        palette.inset
                    };
                    let stroke = if selected {
                        Stroke::new(1.5, palette.accent)
                    } else {
                        Stroke::new(1.0, palette.border)
                    };
                    let response = Frame::new()
                        .fill(fill)
                        .stroke(stroke)
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(Margin::symmetric(10, 8))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal(|ui| {
                                badge(ui, &comp.tier, tier_badge_color(&palette, &comp.tier));
                                ui.label(
                                    RichText::new(&comp.name)
                                        .size(12.5)
                                        .strong()
                                        .color(palette.text),
                                );
                            });
                            ui.add_space(3.0);
                            ui.label(
                                RichText::new(format!(
                                    "{} · {}",
                                    comp.style,
                                    comp.traits.join(" · ")
                                ))
                                .size(10.5)
                                .color(palette.muted),
                            );
                        })
                        .response
                        .interact(egui::Sense::click());
                    if response.clicked() {
                        ctx.state.tft.selected_comp = comp.id.clone();
                    }
                    ui.add_space(5.0);
                }
            });
    });
}

fn render_comp_detail(ctx: &mut PageContext<'_>, ui: &mut egui::Ui, filtered: &[&TftLegacyComp]) {
    let palette = *ctx.palette;
    let selected = filtered
        .iter()
        .copied()
        .find(|comp| comp.id == ctx.state.tft.selected_comp)
        .or_else(|| filtered.first().copied());

    panel(ui, &palette, |ui| {
        let Some(comp) = selected else {
            ui.label(
                RichText::new("Select a composition from the list.")
                    .size(12.0)
                    .color(palette.muted),
            );
            return;
        };

        ui.horizontal_wrapped(|ui| {
            badge(ui, &comp.tier, tier_badge_color(&palette, &comp.tier));
            ui.label(
                RichText::new(&comp.name)
                    .size(17.0)
                    .strong()
                    .color(palette.text),
            );
            badge(ui, &comp.style, palette.violet);
            if !comp.difficulty.is_empty() {
                badge(ui, &comp.difficulty, palette.gold);
            }
        });
        ui.add_space(6.0);
        ui.label(
            RichText::new(&comp.summary)
                .size(11.5)
                .color(palette.secondary),
        );

        ui.add_space(10.0);
        ui.label(
            RichText::new("SYNERGY TRAITS")
                .size(10.5)
                .strong()
                .color(palette.muted),
        );
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for trait_name in &comp.traits {
                badge(ui, trait_name, palette.accent);
            }
        });

        ui.add_space(10.0);
        ui.label(
            RichText::new("CORE BOARD UNITS")
                .size(10.5)
                .strong()
                .color(palette.muted),
        );
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for unit in &comp.units {
                inset_card(ui, &palette, palette.border, |ui| {
                    ui.horizontal(|ui| {
                        render_tft_unit_thumb(ctx, ui, unit, 24.0);
                        ui.label(RichText::new(unit).size(11.5).strong().color(palette.text));
                    });
                });
            }
        });

        ui.add_space(10.0);
        ui.columns(2, |cols| {
            inset_card(&mut cols[0], &palette, palette.gold, |ui| {
                ui.label(
                    RichText::new("PRIORITY ITEMS")
                        .size(10.5)
                        .strong()
                        .color(palette.gold),
                );
                ui.add_space(4.0);
                for item in &comp.items {
                    ui.label(
                        RichText::new(format!("• {item}"))
                            .size(11.0)
                            .color(palette.text),
                    );
                }
            });
            inset_card(&mut cols[1], &palette, palette.green, |ui| {
                ui.label(
                    RichText::new("TARGET AUGMENTS")
                        .size(10.5)
                        .strong()
                        .color(palette.green),
                );
                ui.add_space(4.0);
                for aug in &comp.augments {
                    ui.label(
                        RichText::new(format!("• {aug}"))
                            .size(11.0)
                            .color(palette.text),
                    );
                }
            });
        });

        if !comp.tips.is_empty() {
            ui.add_space(10.0);
            inset_card(ui, &palette, palette.accent, |ui| {
                ui.label(
                    RichText::new("TEMPO & POSITIONING NOTES")
                        .size(10.5)
                        .strong()
                        .color(palette.accent),
                );
                ui.add_space(3.0);
                ui.label(
                    RichText::new(&comp.tips)
                        .size(11.5)
                        .color(palette.secondary),
                );
            });
        }
    });
}

fn render_items_tab(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;
    let components = tft::legacy_components();
    let recipes = tft::legacy_recipes();

    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            "COMPONENT CRAFTING MIXER",
            "Combine two base components to preview completed recipe effects",
        );

        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("Component 1:")
                    .size(11.0)
                    .strong()
                    .color(palette.muted),
            );
            for comp in &components {
                let active = ctx.state.tft.component_one == comp.id;
                if filter_chip(ui, &palette, &comp.name, active, palette.accent)
                    .on_hover_text(&comp.stats)
                    .clicked()
                {
                    ctx.state.tft.component_one = comp.id.clone();
                }
            }
        });

        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("Component 2:")
                    .size(11.0)
                    .strong()
                    .color(palette.muted),
            );
            for comp in &components {
                let active = ctx.state.tft.component_two == comp.id;
                if filter_chip(ui, &palette, &comp.name, active, palette.violet)
                    .on_hover_text(&comp.stats)
                    .clicked()
                {
                    ctx.state.tft.component_two = comp.id.clone();
                }
            }
        });

        ui.add_space(8.0);
        let matched = find_recipe(
            &recipes,
            &ctx.state.tft.component_one,
            &ctx.state.tft.component_two,
        );
        inset_card(ui, &palette, palette.gold, |ui| {
            if let Some(recipe) = matched {
                ui.horizontal_wrapped(|ui| {
                    badge(ui, "RECIPE RESULT", palette.gold);
                    if !recipe.category.is_empty() {
                        badge(ui, &recipe.category, palette.violet);
                    }
                    ui.label(
                        RichText::new(&recipe.name)
                            .size(14.0)
                            .strong()
                            .color(palette.text),
                    );
                });
                ui.add_space(4.0);
                ui.label(
                    RichText::new(&recipe.stats)
                        .size(11.5)
                        .strong()
                        .color(palette.gold),
                );
                ui.label(
                    RichText::new(&recipe.description)
                        .size(11.5)
                        .color(palette.secondary),
                );
                if !recipe.best_champions.is_empty() {
                    ui.label(
                        RichText::new(format!("Best on: {}", recipe.best_champions.join(", ")))
                            .size(10.5)
                            .color(palette.muted),
                    );
                }
            } else {
                ui.label(
                    RichText::new("Select two components to inspect the crafted item.")
                        .size(11.5)
                        .color(palette.muted),
                );
            }
        });
    });

    ui.add_space(10.0);
    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            "SET 18 LIVE ITEM COMPENDIUM",
            "Bundled item icons and stats from current TFT metadata",
        );
        search_box(
            ui,
            &palette,
            "tft-item-search",
            &mut ctx.state.tft.item_search,
            "Search Set 18 items...",
            None,
        );
        ui.add_space(8.0);

        let query = ctx.state.tft.item_search.trim().to_ascii_lowercase();
        let items: Vec<&TftEntry> = tft::items()
            .iter()
            .filter(|entry| {
                query.is_empty()
                    || entry.name.to_ascii_lowercase().contains(&query)
                    || entry.description.to_ascii_lowercase().contains(&query)
            })
            .collect();

        if items.is_empty() {
            ui.label(
                RichText::new("No Set 18 items match the search filter.")
                    .size(11.5)
                    .color(palette.muted),
            );
            return;
        }

        let cols = if ui.available_width() > 960.0 { 3 } else { 2 };
        let col_width =
            ((ui.available_width() - 12.0 * (cols as f32 - 1.0)) / cols as f32).max(200.0);

        egui::ScrollArea::vertical()
            .id_salt("tft-items-grid")
            .max_height(380.0)
            .show(ui, |ui| {
                for row in items.chunks(cols) {
                    ui.horizontal_top(|ui| {
                        for (idx, item) in row.iter().enumerate() {
                            if idx > 0 {
                                ui.add_space(10.0);
                            }
                            ui.allocate_ui_with_layout(
                                Vec2::new(col_width, 76.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    inset_card(ui, &palette, palette.border, |ui| {
                                        ui.set_min_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            render_tft_entry_icon(ctx, ui, item, 28.0);
                                            ui.vertical(|ui| {
                                                ui.label(
                                                    RichText::new(&item.name)
                                                        .size(12.0)
                                                        .strong()
                                                        .color(palette.text),
                                                );
                                                ui.label(
                                                    RichText::new(&item.description)
                                                        .size(10.5)
                                                        .color(palette.muted),
                                                );
                                            });
                                        });
                                    });
                                },
                            );
                        }
                    });
                    ui.add_space(6.0);
                }
            });
    });
}

fn render_odds_tab(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;
    let rows = tft::legacy_roll_odds();

    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            "SHOP ROLL ODDS BY TACTICIAN LEVEL",
            "Equalized fixed-width odds grid for quick level timing checks",
        );

        let total_w = ui.available_width().max(520.0);
        let col_w = ((total_w - 36.0) / 7.0).max(70.0);

        egui::Grid::new("tft-roll-odds-table")
            .num_columns(7)
            .min_col_width(col_w)
            .max_col_width(col_w + 16.0)
            .spacing(Vec2::new(6.0, 6.0))
            .striped(true)
            .show(ui, |ui| {
                for header in [
                    "LEVEL", "1-COST", "2-COST", "3-COST", "4-COST", "5-COST", "TIMING",
                ] {
                    ui.label(
                        RichText::new(header)
                            .size(11.0)
                            .strong()
                            .color(palette.accent),
                    );
                }
                ui.end_row();

                for row in &rows {
                    render_odds_row(ui, &palette, row);
                }
            });

        ui.add_space(12.0);
        inset_card(ui, &palette, palette.gold, |ui| {
            ui.label(
                RichText::new("LEVELING INTERVAL REFERENCE")
                    .size(11.0)
                    .strong()
                    .color(palette.gold),
            );
            ui.add_space(4.0);
            for tip in [
                "Level 4 on 2-1 for win-streak tempo or early 2-cost pair pressure.",
                "Level 6 on 3-2 to stabilize 2-cost / 3-cost reroll comps or roll lightly for pairs.",
                "Level 7 on 4-1 when contesting 3-cost carries; hold 50g for 4-cost fast-8 lines.",
                "Level 8 on 4-2 / 4-5 with 30g+ roll down to hit primary 4-cost carries.",
            ] {
                ui.label(
                    RichText::new(format!("• {tip}"))
                        .size(11.0)
                        .color(palette.secondary),
                );
            }
        });
    });
}

fn render_odds_row(ui: &mut egui::Ui, palette: &ThemePalette, row: &TftRollOddsRow) {
    ui.label(
        RichText::new(format!("Lv {}", row.level))
            .size(11.5)
            .strong()
            .color(palette.text),
    );
    let tiers = [
        (row.cost1, palette.muted),
        (row.cost2, palette.green),
        (row.cost3, palette.accent),
        (row.cost4, palette.violet),
        (row.cost5, palette.gold),
    ];
    for (pct, color) in tiers {
        let display_color = if pct == 0 { palette.muted } else { color };
        ui.label(
            RichText::new(format!("{pct}%"))
                .size(11.5)
                .color(display_color),
        );
    }
    ui.label(
        RichText::new(format!(
            "{} · {} XP · ~{}g",
            row.recommended_round, row.xp_to_next, row.gold_cost_estimate
        ))
        .size(10.5)
        .color(palette.secondary),
    );
    ui.end_row();
}

fn render_augments_tab(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;
    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            "TFT AUGMENT DATABASE",
            "Set 18 augments with instant keyword filtering",
        );
        search_box(
            ui,
            &palette,
            "tft-augment-search",
            &mut ctx.state.tft.augment_search,
            "Search augment name or effect...",
            None,
        );
        ui.add_space(8.0);

        let query = ctx.state.tft.augment_search.trim().to_ascii_lowercase();
        let augments: Vec<&TftEntry> = tft::augments()
            .iter()
            .filter(|entry| {
                query.is_empty()
                    || entry.name.to_ascii_lowercase().contains(&query)
                    || entry.description.to_ascii_lowercase().contains(&query)
            })
            .collect();

        if augments.is_empty() {
            ui.label(
                RichText::new("No augments match the current search.")
                    .size(11.5)
                    .color(palette.muted),
            );
            return;
        }

        egui::ScrollArea::vertical()
            .id_salt("tft-augments-list")
            .max_height(460.0)
            .show(ui, |ui| {
                for aug in augments {
                    inset_card(ui, &palette, palette.border, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            render_tft_entry_icon(ctx, ui, aug, 26.0);
                            ui.vertical(|ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(
                                        RichText::new(&aug.name)
                                            .size(12.0)
                                            .strong()
                                            .color(palette.text),
                                    );
                                    if let Some(tier) = aug.tier {
                                        badge(ui, augment_tier_label(tier), palette.violet);
                                    }
                                });
                                if !aug.description.is_empty() {
                                    ui.label(
                                        RichText::new(&aug.description)
                                            .size(10.5)
                                            .color(palette.muted),
                                    );
                                }
                            });
                        });
                    });
                    ui.add_space(5.0);
                }
            });
    });
}

fn render_board_tab(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;
    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            "4x7 TACTICAL BOARD PLANNER",
            "Click a hex slot to assign a unit name or Set 18 champion portrait",
        );

        for row in 0..4usize {
            ui.horizontal(|ui| {
                if row % 2 == 1 {
                    ui.add_space(28.0);
                }
                for col in 0..7usize {
                    let idx = row * 7 + col;
                    let unit_name = ctx
                        .store
                        .settings
                        .tft_board
                        .get(idx)
                        .cloned()
                        .unwrap_or_default();
                    let selected = ctx.state.tft.selected_cell == Some(idx);

                    let fill = if selected {
                        ThemePalette::with_alpha(palette.accent, 38)
                    } else if unit_name.is_empty() {
                        palette.inset
                    } else {
                        ThemePalette::with_alpha(palette.violet, 26)
                    };
                    let stroke = if selected {
                        Stroke::new(1.5, palette.accent)
                    } else if unit_name.is_empty() {
                        Stroke::new(1.0, palette.border)
                    } else {
                        Stroke::new(1.0, palette.violet)
                    };

                    let cell_resp = Frame::new()
                        .fill(fill)
                        .stroke(stroke)
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::same(6))
                        .show(ui, |ui| {
                            ui.set_min_size(Vec2::new(72.0, 58.0));
                            ui.vertical_centered(|ui| {
                                if unit_name.is_empty() {
                                    ui.label(
                                        RichText::new(format!("R{}C{}", row + 1, col + 1))
                                            .size(9.5)
                                            .color(palette.muted),
                                    );
                                } else {
                                    render_tft_unit_thumb(ctx, ui, &unit_name, 26.0);
                                    ui.label(
                                        RichText::new(&unit_name)
                                            .size(9.5)
                                            .strong()
                                            .color(palette.text),
                                    );
                                }
                            });
                        })
                        .response
                        .interact(egui::Sense::click());

                    if cell_resp.clicked() {
                        ctx.state.tft.selected_cell = Some(idx);
                        ctx.state.tft.unit_input = unit_name;
                    }
                    ui.add_space(4.0);
                }
            });
            ui.add_space(4.0);
        }

        ui.add_space(10.0);
        inset_card(ui, &palette, palette.border, |ui| {
            if let Some(cell_idx) = ctx.state.tft.selected_cell {
                let row = cell_idx / 7 + 1;
                let col = cell_idx % 7 + 1;
                ui.horizontal_wrapped(|ui| {
                    badge(ui, &format!("HEX R{row}C{col}"), palette.accent);
                    let edit = ui.add(
                        egui::TextEdit::singleline(&mut ctx.state.tft.unit_input)
                            .desired_width(210.0)
                            .hint_text("Enter unit name (e.g. Ahri, Jhin)"),
                    );
                    let submit = edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let place =
                        action_button(ui, &palette, "PLACE UNIT", true, palette.accent).clicked();
                    if submit || place {
                        let trimmed = ctx.state.tft.unit_input.trim().to_string();
                        set_board_cell(&mut ctx.store.settings.tft_board, cell_idx, trimmed);
                        ctx.request_save();
                    }
                    if action_button(ui, &palette, "CLEAR HEX", false, palette.muted).clicked() {
                        set_board_cell(&mut ctx.store.settings.tft_board, cell_idx, String::new());
                        ctx.state.tft.unit_input.clear();
                        ctx.request_save();
                    }
                });

                ui.add_space(6.0);
                ui.label(
                    RichText::new("Quick pick Set 18 champions:")
                        .size(10.0)
                        .color(palette.muted),
                );
                ui.horizontal_wrapped(|ui| {
                    for champ in tft::champions().iter().take(14) {
                        if filter_chip(ui, &palette, &champ.name, false, palette.violet).clicked() {
                            set_board_cell(
                                &mut ctx.store.settings.tft_board,
                                cell_idx,
                                champ.name.clone(),
                            );
                            ctx.state.tft.unit_input = champ.name.clone();
                            ctx.request_save();
                        }
                    }
                });
            } else {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new("Select any hex above to place or clear a unit.")
                            .size(11.5)
                            .color(palette.muted),
                    );
                    if ctx
                        .store
                        .settings
                        .tft_board
                        .iter()
                        .any(|cell| !cell.is_empty())
                        && action_button(ui, &palette, "CLEAR ENTIRE BOARD", true, palette.red)
                            .clicked()
                    {
                        ctx.store.settings.tft_board.fill(String::new());
                        ctx.request_save();
                    }
                });
            }
        });
    });
}

fn render_set_data_tab(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;
    let meta = tft::set_metadata();

    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            &format!("SET {} — OFFLINE CATALOG", meta.set),
            &format!(
                "Patch {} · {} units · {} traits · {} items · {} augments",
                meta.patch_version,
                meta.champion_count,
                meta.trait_count,
                meta.item_count,
                meta.augment_count
            ),
        );

        ui.horizontal_wrapped(|ui| {
            for (category, label) in [
                (TftDataCategory::Units, "UNITS"),
                (TftDataCategory::Traits, "TRAITS"),
                (TftDataCategory::Items, "ITEMS"),
            ] {
                if filter_chip(
                    ui,
                    &palette,
                    label,
                    ctx.state.tft.data_category == category,
                    palette.accent,
                )
                .clicked()
                {
                    ctx.state.tft.data_category = category;
                }
            }
        });

        ui.add_space(6.0);
        search_box(
            ui,
            &palette,
            "tft-data-search",
            &mut ctx.state.tft.data_search,
            "Filter Set 18 entries...",
            None,
        );
        ui.add_space(8.0);

        let entries = match ctx.state.tft.data_category {
            TftDataCategory::Units => tft::champions(),
            TftDataCategory::Traits => tft::traits(),
            TftDataCategory::Items => tft::items(),
        };
        let query = ctx.state.tft.data_search.trim().to_ascii_lowercase();
        let filtered: Vec<&TftEntry> = entries
            .iter()
            .filter(|entry| {
                query.is_empty()
                    || entry.name.to_ascii_lowercase().contains(&query)
                    || entry.id.to_ascii_lowercase().contains(&query)
                    || entry.description.to_ascii_lowercase().contains(&query)
            })
            .collect();

        if filtered.is_empty() {
            ui.label(
                RichText::new("No Set 18 entries match the current filter.")
                    .size(11.5)
                    .color(palette.muted),
            );
            return;
        }

        egui::ScrollArea::vertical()
            .id_salt("tft-set-data-scroll")
            .max_height(430.0)
            .show(ui, |ui| {
                for entry in filtered {
                    inset_card(ui, &palette, palette.border, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            render_tft_entry_icon(ctx, ui, entry, 28.0);
                            ui.vertical(|ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(
                                        RichText::new(&entry.name)
                                            .size(12.0)
                                            .strong()
                                            .color(palette.text),
                                    );
                                    if let Some(cost) = entry.cost.filter(|cost| *cost > 0) {
                                        badge(ui, &format!("{cost}g"), palette.gold);
                                    }
                                    if let Some(tier) = entry.tier {
                                        badge(ui, &format!("TIER {tier}"), palette.accent);
                                    }
                                });
                                if !entry.description.is_empty() {
                                    ui.label(
                                        RichText::new(&entry.description)
                                            .size(10.5)
                                            .color(palette.muted),
                                    );
                                }
                            });
                        });
                    });
                    ui.add_space(5.0);
                }
            });
    });
}

fn find_recipe<'a>(
    recipes: &'a [TftCombinedRecipe],
    first: &str,
    second: &str,
) -> Option<&'a TftCombinedRecipe> {
    recipes.iter().find(|r| {
        (r.component1 == first && r.component2 == second)
            || (r.component1 == second && r.component2 == first)
    })
}

fn render_tft_entry_icon(
    ctx: &mut PageContext<'_>,
    ui: &mut egui::Ui,
    entry: &TftEntry,
    size: f32,
) {
    let palette = *ctx.palette;
    let tex = ctx.textures.tft(ui.ctx(), &entry.image_key).cloned();
    avatar(
        ui,
        &palette,
        &entry.name,
        size,
        palette.accent,
        tex.as_ref(),
    );
}

fn render_tft_unit_thumb(ctx: &mut PageContext<'_>, ui: &mut egui::Ui, unit_name: &str, size: f32) {
    let matched = tft::champions()
        .iter()
        .find(|entry| entry.name.eq_ignore_ascii_case(unit_name));
    if let Some(entry) = matched {
        render_tft_entry_icon(ctx, ui, entry, size);
    } else {
        avatar(ui, ctx.palette, unit_name, size, ctx.palette.violet, None);
    }
}

fn tier_badge_color(palette: &ThemePalette, tier: &str) -> Color32 {
    if tier.starts_with('S') {
        palette.gold
    } else if tier.starts_with('A') {
        palette.accent
    } else {
        palette.muted
    }
}

fn augment_tier_label(tier: u32) -> &'static str {
    match tier {
        1 => "SILVER",
        2 => "GOLD",
        3 => "PRISMATIC",
        _ => "AUGMENT",
    }
}

fn set_board_cell(board: &mut Vec<String>, index: usize, value: String) {
    if board.len() <= index {
        board.resize(index + 1, String::new());
    }
    board[index] = value;
}

/// Archived Set 13 meta composition flattened from `TFT_META_COMPS` for display.
struct TftLegacyComp {
    id: String,
    name: String,
    tier: String,
    style: String,
    difficulty: String,
    summary: String,
    traits: Vec<String>,
    units: Vec<String>,
    items: Vec<String>,
    augments: Vec<String>,
    tips: String,
}

fn legacy_comps() -> Vec<TftLegacyComp> {
    let data = tft::legacy_data();
    let augment_names: Vec<(&str, &str)> = data
        .get("TFT_AUGMENTS")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(|aug| Some((aug.get("id")?.as_str()?, aug.get("name")?.as_str()?)))
                .collect()
        })
        .unwrap_or_default();
    data.get("TFT_META_COMPS")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .map(|comp| parse_legacy_comp(comp, &augment_names))
                .collect()
        })
        .unwrap_or_default()
}

fn json_text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn json_names(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(|v| v.get("name").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn core_items(comp: &Value, key: &str) -> Vec<String> {
    let Some(core) = comp.get(key) else {
        return Vec::new();
    };
    let champion = json_text(core, "champion");
    core.get("items")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(Value::as_str)
                .map(|item| format!("{item} ({champion})"))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_legacy_comp(comp: &Value, augment_names: &[(&str, &str)]) -> TftLegacyComp {
    let mut items = core_items(comp, "mainCarry");
    items.extend(core_items(comp, "mainTank"));
    let augments = comp
        .get("recommendedAugments")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(Value::as_str)
                .map(|id| {
                    augment_names
                        .iter()
                        .find(|(aug_id, _)| *aug_id == id)
                        .map_or_else(|| id.replace('_', " "), |(_, name)| (*name).to_string())
                })
                .collect()
        })
        .unwrap_or_default();
    let tips = [
        json_text(comp, "levelingGuide"),
        json_text(comp, "positioningTip"),
    ]
    .into_iter()
    .filter(|tip| !tip.is_empty())
    .collect::<Vec<_>>()
    .join("\n");
    TftLegacyComp {
        id: json_text(comp, "id"),
        name: json_text(comp, "name"),
        tier: json_text(comp, "tier"),
        style: json_text(comp, "playstyle"),
        difficulty: json_text(comp, "difficulty"),
        summary: json_text(comp, "tagline"),
        traits: json_names(comp, "traits"),
        units: json_names(comp, "champions"),
        items,
        augments,
        tips,
    }
}
