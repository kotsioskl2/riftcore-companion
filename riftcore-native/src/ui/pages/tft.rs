//! TFT Hub page: Set 18 unit/trait/item browser, Set 13 archived comps, crafting mixer, roll odds, and 4x7 tactical board.

use crate::domain::tft::{
    self, TftCombinedRecipe, TftComponent, TftEntry, TftLegacyComp, TftRollOddsRow,
};
use crate::ui::pages::{PageContext, TftDataCategory, TftTab};
use crate::ui::theme::{CornerRadius as _, ThemePalette};
use crate::ui::widgets::{
    action_button, avatar, badge, filter_chip, inset_card, page_heading, panel, panel_title,
    search_box,
};
use eframe::egui::{self, Color32, CornerRadius, FontId, Frame, Margin, RichText, Stroke, Vec2};

pub fn render(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;
    let meta = tft::set_metadata();

    page_heading(
        ui,
        &palette,
        "TFT HUB",
        &format!(
            "Set {} ({}) tactical reference plus archived Set 13 comps, item mixer, and 4x7 board planner",
            meta.set_number, meta.set_name
        ),
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
            if filter_chip(ui, &palette, label, ctx.state.tft.tab == tab, palette.accent) {
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
    let palette = *ctx.palette;
    let comps = tft::legacy_comps();
    let query = ctx.state.tft.comp_search.trim().to_ascii_lowercase();
    let filtered: Vec<&TftLegacyComp> = comps
        .iter()
        .filter(|comp| {
            query.is_empty()
                || comp.name.to_ascii_lowercase().contains(&query)
                || comp.style.to_ascii_lowercase().contains(&query)
                || comp.traits.iter().any(|t| t.to_ascii_lowercase().contains(&query))
                || comp.units.iter().any(|u| u.to_ascii_lowercase().contains(&query))
        })
        .collect();

    if ctx.state.tft.selected_comp.is_empty() {
        if let Some(first) = filtered.first() {
            ctx.state.tft.selected_comp = first.name.to_string();
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
            &mut ctx.state.tft.comp_search,
            "Search comp, unit, trait, or style...",
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
                    let selected = ctx.state.tft.selected_comp == comp.name;
                    let fill = if selected {
                        ThemePalette::with_alpha(palette.accent, 28)
                    } else {
                        palette.surface_inset
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
                                badge(ui, comp.tier, tier_badge_color(&palette, comp.tier));
                                ui.label(
                                    RichText::new(comp.name)
                                        .size(12.5)
                                        .strong()
                                        .color(palette.text),
                                );
                            });
                            ui.add_space(3.0);
                            ui.label(
                                RichText::new(format!("{} · {}", comp.style, comp.traits.join(" · ")))
                                    .size(10.5)
                                    .color(palette.muted),
                            );
                        })
                        .response
                        .interact(egui::Sense::click());
                    if response.clicked() {
                        ctx.state.tft.selected_comp = comp.name.to_string();
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
        .find(|comp| comp.name == ctx.state.tft.selected_comp)
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
            badge(ui, comp.tier, tier_badge_color(&palette, comp.tier));
            ui.label(
                RichText::new(comp.name)
                    .size(17.0)
                    .strong()
                    .color(palette.text),
            );
            badge(ui, comp.style, palette.violet);
        });
        ui.add_space(6.0);
        ui.label(
            RichText::new(comp.summary)
                .size(11.5)
                .color(palette.text_secondary),
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
            for trait_name in comp.traits {
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
            for unit in comp.units {
                inset_card(ui, &palette, |ui| {
                    ui.horizontal(|ui| {
                        render_tft_unit_thumb(ctx, ui, unit, 24.0);
                        ui.label(RichText::new(*unit).size(11.5).strong().color(palette.text));
                    });
                });
            }
        });

        ui.add_space(10.0);
        ui.columns(2, |cols| {
            inset_card(&mut cols[0], &palette, |ui| {
                ui.label(
                    RichText::new("PRIORITY ITEMS")
                        .size(10.5)
                        .strong()
                        .color(palette.gold),
                );
                ui.add_space(4.0);
                for item in comp.items {
                    ui.label(
                        RichText::new(format!("• {item}"))
                            .size(11.0)
                            .color(palette.text),
                    );
                }
            });
            inset_card(&mut cols[1], &palette, |ui| {
                ui.label(
                    RichText::new("TARGET AUGMENTS")
                        .size(10.5)
                        .strong()
                        .color(palette.green),
                );
                ui.add_space(4.0);
                for aug in comp.augments {
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
            inset_card(ui, &palette, |ui| {
                ui.label(
                    RichText::new("TEMPO & POSITIONING NOTES")
                        .size(10.5)
                        .strong()
                        .color(palette.accent),
                );
                ui.add_space(3.0);
                ui.label(
                    RichText::new(comp.tips)
                        .size(11.5)
                        .color(palette.text_secondary),
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
            for comp in components {
                let active = ctx.state.tft.component_one == comp.id;
                if filter_chip(ui, &palette, comp.name, active, palette.accent) {
                    ctx.state.tft.component_one = comp.id.to_string();
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
            for comp in components {
                let active = ctx.state.tft.component_two == comp.id;
                if filter_chip(ui, &palette, comp.name, active, palette.violet) {
                    ctx.state.tft.component_two = comp.id.to_string();
                }
            }
        });

        ui.add_space(8.0);
        let matched = find_recipe(
            recipes,
            &ctx.state.tft.component_one,
            &ctx.state.tft.component_two,
        );
        inset_card(ui, &palette, |ui| {
            if let Some(recipe) = matched {
                ui.horizontal_wrapped(|ui| {
                    badge(ui, "RECIPE RESULT", palette.gold);
                    ui.label(
                        RichText::new(recipe.name)
                            .size(14.0)
                            .strong()
                            .color(palette.text),
                    );
                });
                ui.add_space(4.0);
                ui.label(
                    RichText::new(recipe.effect)
                        .size(11.5)
                        .color(palette.text_secondary),
                );
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
            &mut ctx.state.tft.item_search,
            "Search Set 18 items...",
        );
        ui.add_space(8.0);

        let query = ctx.state.tft.item_search.trim().to_ascii_lowercase();
        let items: Vec<&TftEntry> = tft::items()
            .iter()
            .filter(|entry| {
                query.is_empty()
                    || entry.name.to_ascii_lowercase().contains(&query)
                    || entry.summary.to_ascii_lowercase().contains(&query)
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
        let col_width = ((ui.available_width() - 12.0 * (cols as f32 - 1.0)) / cols as f32).max(200.0);

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
                                    inset_card(ui, &palette, |ui| {
                                        ui.set_min_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            render_tft_entry_icon(ctx, ui, item, 28.0);
                                            ui.vertical(|ui| {
                                                ui.label(
                                                    RichText::new(item.name)
                                                        .size(12.0)
                                                        .strong()
                                                        .color(palette.text),
                                                );
                                                ui.label(
                                                    RichText::new(item.summary)
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
        let col_w = ((total_w - 30.0) / 6.0).max(70.0);

        egui::Grid::new("tft-roll-odds-table")
            .num_columns(6)
            .min_col_width(col_w)
            .max_col_width(col_w + 16.0)
            .spacing(Vec2::new(6.0, 6.0))
            .striped(true)
            .show(ui, |ui| {
                for header in ["LEVEL", "1-COST", "2-COST", "3-COST", "4-COST", "5-COST"] {
                    ui.label(
                        RichText::new(header)
                            .size(11.0)
                            .strong()
                            .color(palette.accent),
                    );
                }
                ui.end_row();

                for row in rows {
                    render_odds_row(ui, &palette, row);
                }
            });

        ui.add_space(12.0);
        inset_card(ui, &palette, |ui| {
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
                        .color(palette.text_secondary),
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
        (row.cost_1, palette.muted),
        (row.cost_2, palette.green),
        (row.cost_3, palette.accent),
        (row.cost_4, palette.violet),
        (row.cost_5, palette.gold),
    ];
    for (pct, color) in tiers {
        let display_color = if pct == 0 { palette.text_muted } else { color };
        ui.label(RichText::new(format!("{pct}%")).size(11.5).color(display_color));
    }
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
            &mut ctx.state.tft.augment_search,
            "Search augment name or effect...",
        );
        ui.add_space(8.0);

        let query = ctx.state.tft.augment_search.trim().to_ascii_lowercase();
        let augments: Vec<&TftEntry> = tft::augments()
            .iter()
            .filter(|entry| {
                query.is_empty()
                    || entry.name.to_ascii_lowercase().contains(&query)
                    || entry.summary.to_ascii_lowercase().contains(&query)
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
                    inset_card(ui, &palette, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            render_tft_entry_icon(ctx, ui, aug, 26.0);
                            ui.vertical(|ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(
                                        RichText::new(aug.name)
                                            .size(12.0)
                                            .strong()
                                            .color(palette.text),
                                    );
                                    if !aug.tags.is_empty() {
                                        badge(ui, aug.tags, palette.violet);
                                    }
                                });
                                if !aug.summary.is_empty() {
                                    ui.label(
                                        RichText::new(aug.summary)
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
                    let key = idx.to_string();
                    let unit_name = ctx
                        .store
                        .settings
                        .tft_board
                        .get(&key)
                        .cloned()
                        .unwrap_or_default();
                    let selected = ctx.state.tft.selected_cell == Some(idx);

                    let fill = if selected {
                        ThemePalette::with_alpha(palette.accent, 38)
                    } else if unit_name.is_empty() {
                        palette.surface_inset
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
                                            .color(palette.text_muted),
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
        inset_card(ui, &palette, |ui| {
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
                    let submit = edit.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if submit || action_button(ui, &palette, "PLACE UNIT", palette.accent) {
                        let trimmed = ctx.state.tft.unit_input.trim().to_string();
                        if trimmed.is_empty() {
                            ctx.store.settings.tft_board.remove(&cell_idx.to_string());
                        } else {
                            ctx.store
                                .settings
                                .tft_board
                                .insert(cell_idx.to_string(), trimmed);
                        }
                        ctx.request_save();
                    }
                    if action_button(ui, &palette, "CLEAR HEX", palette.muted) {
                        ctx.store.settings.tft_board.remove(&cell_idx.to_string());
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
                        if filter_chip(ui, &palette, champ.name, false, palette.violet) {
                            ctx.store
                                .settings
                                .tft_board
                                .insert(cell_idx.to_string(), champ.name.to_string());
                            ctx.state.tft.unit_input = champ.name.to_string();
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
                    if !ctx.store.settings.tft_board.is_empty()
                        && action_button(ui, &palette, "CLEAR ENTIRE BOARD", palette.red)
                    {
                        ctx.store.settings.tft_board.clear();
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
            &format!("SET {} — {}", meta.set_number, meta.set_name),
            &format!("Patch {} offline catalog", meta.patch),
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
                ) {
                    ctx.state.tft.data_category = category;
                }
            }
        });

        ui.add_space(6.0);
        search_box(
            ui,
            &palette,
            &mut ctx.state.tft.data_search,
            "Filter Set 18 entries...",
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
                    || entry.tags.to_ascii_lowercase().contains(&query)
                    || entry.summary.to_ascii_lowercase().contains(&query)
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
                    inset_card(ui, &palette, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            render_tft_entry_icon(ctx, ui, entry, 28.0);
                            ui.vertical(|ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(
                                        RichText::new(entry.name)
                                            .size(12.0)
                                            .strong()
                                            .color(palette.text),
                                    );
                                    if !entry.tags.is_empty() {
                                        badge(ui, entry.tags, palette.accent);
                                    }
                                    if entry.cost > 0 {
                                        badge(ui, &format!("{}g", entry.cost), palette.gold);
                                    }
                                });
                                if !entry.summary.is_empty() {
                                    ui.label(
                                        RichText::new(entry.summary)
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
        (r.first == first && r.second == second) || (r.first == second && r.second == first)
    })
}

fn render_tft_entry_icon(
    ctx: &mut PageContext<'_>,
    ui: &mut egui::Ui,
    entry: &TftEntry,
    size: f32,
) {
    let tex = if !entry.image_key.is_empty() {
        ctx.textures.tft(ui.ctx(), entry.image_key)
    } else {
        None
    };
    avatar(ui, ctx.palette, tex.as_ref(), entry.name, size, ctx.palette.accent);
}

fn render_tft_unit_thumb(ctx: &mut PageContext<'_>, ui: &mut egui::Ui, unit_name: &str, size: f32) {
    let matched = tft::champions()
        .iter()
        .find(|entry| entry.name.eq_ignore_ascii_case(unit_name));
    if let Some(entry) = matched {
        render_tft_entry_icon(ctx, ui, entry, size);
    } else {
        avatar(ui, ctx.palette, None, unit_name, size, ctx.palette.violet);
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
