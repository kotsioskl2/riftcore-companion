//! Tier List & Champion Meta Explorer page: filterable champion matrix with responsive Table and Card views.

use crate::domain::builds::{self, BuildProfile};
use crate::domain::champions::{self, Champion, Role};
use crate::ui::pages::{PageContext, TierSort};
use crate::ui::theme::role_color;
use crate::ui::widgets::{
    action_button, avatar, badge, filter_chip, inset_card, item_slot, page_heading, panel,
    panel_title, role_tag, search_box,
};
use eframe::egui::{self, RichText, Vec2};

struct TierRow<'a> {
    champion: &'a Champion,
    primary_role: Role,
    profile: &'a BuildProfile,
    favorite: bool,
}

pub fn render(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;

    page_heading(
        ui,
        &palette,
        "TIER LIST & META EXPLORER",
        "Champion Meta Matrix",
        "Complete 172-champion matrix with offline build archetypes, skill priorities, and instant Champ Select jump",
        ctx.snapshot,
    );

    panel(ui, &palette, |ui| {
        ui.horizontal_wrapped(|ui| {
            let all_active = ctx.state.tier_list.role_filter.is_none();
            if filter_chip(ui, &palette, "ALL ROLES", all_active, palette.accent).clicked() {
                ctx.state.tier_list.role_filter = None;
            }
            for role in Role::ALL {
                let active = ctx.state.tier_list.role_filter == Some(role);
                if filter_chip(
                    ui,
                    &palette,
                    role.short_label(),
                    active,
                    role_color(role, &palette),
                )
                .clicked()
                {
                    ctx.state.tier_list.role_filter = Some(role);
                }
            }

            ui.add_space(12.0);
            for (sort, label) in [
                (TierSort::Name, "SORT: NAME"),
                (TierSort::Role, "SORT: ROLE"),
                (TierSort::Archetype, "SORT: ARCHETYPE"),
            ] {
                if filter_chip(
                    ui,
                    &palette,
                    label,
                    ctx.state.tier_list.sort == sort,
                    palette.violet,
                )
                .clicked()
                {
                    ctx.state.tier_list.sort = sort;
                }
            }

            ui.add_space(12.0);
            let view_cards = ctx.state.tier_list.view_cards;
            if filter_chip(ui, &palette, "TABLE", !view_cards, palette.gold).clicked() {
                ctx.state.tier_list.view_cards = false;
            }
            if filter_chip(ui, &palette, "CARDS", view_cards, palette.gold).clicked() {
                ctx.state.tier_list.view_cards = true;
            }
        });

        ui.add_space(8.0);
        search_box(
            ui,
            &palette,
            "tier-list-search",
            &mut ctx.state.tier_list.search,
            "Filter champions by name, role, or archetype...",
            None,
        );
    });

    ui.add_space(10.0);

    let role_filter = ctx.state.tier_list.role_filter;
    let query = ctx.state.tier_list.search.trim().to_ascii_lowercase();

    let mut rows: Vec<TierRow<'static>> = champions::all_champions()
        .iter()
        .filter(|champ| role_filter.is_none_or(|r| champ.supports_role(r)))
        .filter_map(|champ| {
            let primary_role = role_filter
                .filter(|&r| champ.supports_role(r))
                .unwrap_or_else(|| champ.primary_role());
            let profile = builds::build_for_champion(champ.id, primary_role)
                .or_else(|| builds::default_build_for_champion(champ.id))?;
            if !query.is_empty() {
                let matches = champ.name.to_ascii_lowercase().contains(&query)
                    || champ.key.to_ascii_lowercase().contains(&query)
                    || profile
                        .archetype
                        .display_name()
                        .to_ascii_lowercase()
                        .contains(&query)
                    || champ
                        .roles
                        .iter()
                        .any(|r| r.as_str().to_ascii_lowercase().contains(&query));
                if !matches {
                    return None;
                }
            }
            let favorite = ctx.store.settings.favorite_champions.contains(&champ.id);
            Some(TierRow {
                champion: champ,
                primary_role,
                profile,
                favorite,
            })
        })
        .collect();

    match ctx.state.tier_list.sort {
        TierSort::Name => rows.sort_by(|a, b| a.champion.name.cmp(&b.champion.name)),
        TierSort::Role => rows.sort_by(|a, b| {
            a.primary_role
                .as_str()
                .cmp(b.primary_role.as_str())
                .then(a.champion.name.cmp(&b.champion.name))
        }),
        TierSort::Archetype => rows.sort_by(|a, b| {
            a.profile
                .archetype
                .as_str()
                .cmp(b.profile.archetype.as_str())
                .then(a.champion.name.cmp(&b.champion.name))
        }),
    }

    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            &format!("CHAMPION MATRIX ({})", rows.len()),
            "Click ★ to pin to favorites or OPEN SETUP to load full runes, items, and skill path",
        );

        if rows.is_empty() {
            ui.label(
                RichText::new("No champions match the selected role or search query.")
                    .size(12.0)
                    .color(palette.muted),
            );
            return;
        }

        if ctx.state.tier_list.view_cards {
            render_cards_view(ctx, ui, &rows);
        } else {
            render_table_view(ctx, ui, &rows);
        }
    });
}

fn render_table_view(ctx: &mut PageContext<'_>, ui: &mut egui::Ui, rows: &[TierRow<'_>]) {
    let palette = *ctx.palette;
    let total_w = ui.available_width().max(760.0);
    let mut toggle_fav: Option<u32> = None;
    let mut navigate_id: Option<u32> = None;

    egui::ScrollArea::vertical()
        .id_salt("tier-list-table-scroll")
        .max_height(500.0)
        .show(ui, |ui| {
            egui::Grid::new("tier-list-table-grid")
                .num_columns(5)
                .spacing(Vec2::new(10.0, 6.0))
                .striped(true)
                .show(ui, |ui| {
                    for (idx, header) in [
                        "CHAMPION",
                        "ROLES",
                        "ARCHETYPE",
                        "CORE ITEMS & SKILLS",
                        "ACTIONS",
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        let min_w = match idx {
                            0 => (total_w * 0.24).max(165.0),
                            1 => (total_w * 0.14).max(95.0),
                            2 => (total_w * 0.15).max(115.0),
                            3 => (total_w * 0.30).max(200.0),
                            _ => 125.0,
                        };
                        ui.allocate_ui_with_layout(
                            Vec2::new(min_w, 20.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.label(
                                    RichText::new(header)
                                        .size(10.5)
                                        .strong()
                                        .color(palette.accent),
                                );
                            },
                        );
                    }
                    ui.end_row();

                    for row in rows {
                        ui.horizontal(|ui| {
                            let tex = ctx.textures.champion(ui.ctx(), &row.champion.key);
                            avatar(
                                ui,
                                &palette,
                                &row.champion.name,
                                26.0,
                                role_color(row.primary_role, &palette),
                                tex,
                            );
                            ui.label(
                                RichText::new(&row.champion.name)
                                    .size(12.0)
                                    .strong()
                                    .color(palette.text),
                            );
                        });

                        ui.horizontal_wrapped(|ui| {
                            for &r in &row.champion.roles {
                                role_tag(ui, r, &palette);
                            }
                        });

                        ui.horizontal(|ui| {
                            badge(ui, &row.profile.archetype.display_name(), palette.violet);
                        });

                        ui.horizontal(|ui| {
                            for &item_id in row.profile.core_items.iter().take(3) {
                                item_slot(ui, &palette, ctx.textures, item_id, 24.0, false);
                            }
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(row.profile.compact_skill_priority())
                                    .size(10.5)
                                    .strong()
                                    .color(palette.gold),
                            );
                        });

                        ui.horizontal(|ui| {
                            let star = if row.favorite { "★" } else { "☆" };
                            let star_color = if row.favorite {
                                palette.gold
                            } else {
                                palette.muted
                            };
                            if action_button(ui, &palette, star, row.favorite, star_color).clicked()
                            {
                                toggle_fav = Some(row.champion.id);
                            }
                            if action_button(ui, &palette, "OPEN", false, palette.accent).clicked()
                            {
                                navigate_id = Some(row.champion.id);
                            }
                        });
                        ui.end_row();
                    }
                });
        });

    if let Some(id) = toggle_fav {
        toggle_favorite_champion(ctx, id);
    }
    if let Some(id) = navigate_id {
        ctx.navigate_to_champion(id, None);
    }
}

fn render_cards_view(ctx: &mut PageContext<'_>, ui: &mut egui::Ui, rows: &[TierRow<'_>]) {
    let palette = *ctx.palette;
    let available = ui.available_width().max(520.0);
    let cols = if available >= 1140.0 {
        4
    } else if available >= 780.0 {
        3
    } else {
        2
    };
    let gap = 10.0;
    let card_w = ((available - gap * (cols as f32 - 1.0)) / cols as f32).max(210.0);

    let mut toggle_fav: Option<u32> = None;
    let mut navigate_id: Option<u32> = None;

    egui::ScrollArea::vertical()
        .id_salt("tier-list-cards-scroll")
        .max_height(500.0)
        .show(ui, |ui| {
            for chunk in rows.chunks(cols) {
                ui.horizontal_top(|ui| {
                    for (idx, row) in chunk.iter().enumerate() {
                        if idx > 0 {
                            ui.add_space(gap);
                        }
                        ui.allocate_ui_with_layout(
                            Vec2::new(card_w, 128.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                let stroke = if row.favorite {
                                    palette.gold
                                } else {
                                    palette.border
                                };
                                inset_card(ui, &palette, stroke, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        let tex =
                                            ctx.textures.champion(ui.ctx(), &row.champion.key);
                                        avatar(
                                            ui,
                                            &palette,
                                            &row.champion.name,
                                            32.0,
                                            role_color(row.primary_role, &palette),
                                            tex,
                                        );
                                        ui.vertical(|ui| {
                                            ui.label(
                                                RichText::new(&row.champion.name)
                                                    .size(12.5)
                                                    .strong()
                                                    .color(palette.text),
                                            );
                                            ui.horizontal_wrapped(|ui| {
                                                role_tag(ui, row.primary_role, &palette);
                                                badge(
                                                    ui,
                                                    &row.profile.archetype.display_name(),
                                                    palette.violet,
                                                );
                                            });
                                        });
                                    });

                                    ui.add_space(6.0);
                                    ui.horizontal(|ui| {
                                        for &item_id in row.profile.core_items.iter().take(3) {
                                            item_slot(
                                                ui,
                                                &palette,
                                                ctx.textures,
                                                item_id,
                                                24.0,
                                                false,
                                            );
                                        }
                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new(row.profile.compact_skill_priority())
                                                .size(10.5)
                                                .strong()
                                                .color(palette.gold),
                                        );
                                    });

                                    ui.add_space(6.0);
                                    ui.horizontal(|ui| {
                                        let star = if row.favorite { "★ FAV" } else { "☆ FAV" };
                                        let star_color = if row.favorite {
                                            palette.gold
                                        } else {
                                            palette.muted
                                        };
                                        if action_button(
                                            ui,
                                            &palette,
                                            star,
                                            row.favorite,
                                            star_color,
                                        )
                                        .clicked()
                                        {
                                            toggle_fav = Some(row.champion.id);
                                        }
                                        if action_button(
                                            ui,
                                            &palette,
                                            "BUILD",
                                            false,
                                            palette.accent,
                                        )
                                        .clicked()
                                        {
                                            navigate_id = Some(row.champion.id);
                                        }
                                    });
                                });
                            },
                        );
                    }
                });
                ui.add_space(gap);
            }
        });

    if let Some(id) = toggle_fav {
        toggle_favorite_champion(ctx, id);
    }
    if let Some(id) = navigate_id {
        ctx.navigate_to_champion(id, None);
    }
}

fn toggle_favorite_champion(ctx: &mut PageContext<'_>, champ_id: u32) {
    if let Some(pos) = ctx
        .store
        .settings
        .favorite_champions
        .iter()
        .position(|&id| id == champ_id)
    {
        ctx.store.settings.favorite_champions.remove(pos);
    } else {
        ctx.store.settings.favorite_champions.push(champ_id);
    }
    ctx.request_save();
}
