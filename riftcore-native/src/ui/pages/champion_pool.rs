//! Champion Pool Planner page: per-role 3-slot draft plan (Blind / Counter / Pocket) + favorite roster.

use crate::domain::builds;
use crate::domain::champions::{self, Champion, Role};
use crate::ui::pages::PageContext;
use crate::ui::theme::role_color;
use crate::ui::widgets::{
    action_button, avatar, badge, filter_chip, inset_card, page_heading, panel, panel_title,
    role_tag, search_box,
};
use eframe::egui::{self, CornerRadius, Frame, Margin, RichText, Stroke, Vec2};

const SLOT_LABELS: [&str; 3] = ["BLIND PICK", "COUNTER PICK", "POCKET PICK"];

pub fn render(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;

    page_heading(
        ui,
        &palette,
        "CHAMPION POOL PLANNER",
        "Curate your 3-slot role matrix (Blind / Counter / Pocket) and pin favorite builds for fast Champ Select access",
    );

    panel(ui, &palette, |ui| {
        ui.horizontal_wrapped(|ui| {
            for &role in Role::ALL {
                let active = ctx.state.champion_pool.role == role;
                if filter_chip(
                    ui,
                    &palette,
                    role.as_str(),
                    active,
                    role_color(&palette, role.as_str()),
                ) {
                    ctx.state.champion_pool.role = role;
                    ctx.state.champion_pool.assign_slot = None;
                }
            }
        });
    });

    ui.add_space(10.0);

    let active_role = ctx.state.champion_pool.role;
    let role_key = active_role.as_str().to_string();
    let slots = ctx
        .store
        .settings
        .champion_pool
        .get(&role_key)
        .cloned()
        .unwrap_or_default();

    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            &format!("{} ROLE PLAN", active_role.as_str()),
            "Select a slot to assign from the roster below, or jump straight into its saved setup",
        );

        let cols = if ui.available_width() >= 760.0 { 3 } else { 1 };
        let gap = 10.0;
        let card_w = ((ui.available_width() - gap * (cols as f32 - 1.0)) / cols as f32).max(210.0);

        let mut open_champ: Option<u32> = None;
        let mut clear_slot: Option<usize> = None;

        if cols == 3 {
            ui.horizontal_top(|ui| {
                for slot_idx in 0..3 {
                    if slot_idx > 0 {
                        ui.add_space(gap);
                    }
                    let champ_id = slots.get(slot_idx).copied().unwrap_or(0);
                    render_plan_slot(
                        ctx,
                        ui,
                        card_w,
                        slot_idx,
                        champ_id,
                        active_role,
                        &mut open_champ,
                        &mut clear_slot,
                    );
                }
            });
        } else {
            for slot_idx in 0..3 {
                let champ_id = slots.get(slot_idx).copied().unwrap_or(0);
                render_plan_slot(
                    ctx,
                    ui,
                    ui.available_width(),
                    slot_idx,
                    champ_id,
                    active_role,
                    &mut open_champ,
                    &mut clear_slot,
                );
                ui.add_space(6.0);
            }
        }

        if let Some(slot_idx) = clear_slot {
            if let Some(entry) = ctx.store.settings.champion_pool.get_mut(&role_key) {
                if slot_idx < entry.len() {
                    entry[slot_idx] = 0;
                    ctx.request_save();
                }
            }
        }
        if let Some(champ_id) = open_champ {
            ctx.navigate_to_champion(champ_id, Some(active_role));
        }
    });

    ui.add_space(10.0);

    panel(ui, &palette, |ui| {
        let subtitle = if let Some(slot_idx) = ctx.state.champion_pool.assign_slot {
            format!(
                "Assigning {} for {} — click any champion card below",
                SLOT_LABELS[slot_idx],
                active_role.as_str()
            )
        } else {
            format!(
                "Favorites pinned first; toggle ★ or inspect builds for {}",
                active_role.as_str()
            )
        };
        panel_title(ui, &palette, "ROLE & FAVORITE ROSTER", &subtitle);

        ui.horizontal_wrapped(|ui| {
            search_box(
                ui,
                &palette,
                &mut ctx.state.champion_pool.search,
                "Search champions to assign or favorite...",
            );
            if ctx.state.champion_pool.assign_slot.is_some()
                && action_button(ui, &palette, "CANCEL ASSIGN", palette.muted)
            {
                ctx.state.champion_pool.assign_slot = None;
            }
        });
        ui.add_space(8.0);

        let query = ctx.state.champion_pool.search.trim().to_ascii_lowercase();
        let mut roster: Vec<&Champion> = champions::all_champions()
            .iter()
            .filter(|champ| {
                let matches_role = query.is_empty()
                    && champ.roles.contains(&active_role)
                    || !query.is_empty() && champ.name.to_ascii_lowercase().contains(&query);
                matches_role
            })
            .collect();

        roster.sort_by(|a, b| {
            let a_fav = ctx.store.settings.favorite_champions.contains(&a.id);
            let b_fav = ctx.store.settings.favorite_champions.contains(&b.id);
            b_fav.cmp(&a_fav).then(a.name.cmp(b.name))
        });

        if roster.is_empty() {
            ui.label(
                RichText::new("No champions match the current filter.")
                    .size(11.5)
                    .color(palette.muted),
            );
            return;
        }

        let avail = ui.available_width().max(520.0);
        let cols = if avail >= 1100.0 {
            4
        } else if avail >= 760.0 {
            3
        } else {
            2
        };
        let gap = 10.0;
        let card_w = ((avail - gap * (cols as f32 - 1.0)) / cols as f32).max(200.0);

        let mut assign_champ: Option<u32> = None;
        let mut toggle_fav: Option<u32> = None;
        let mut open_champ: Option<u32> = None;

        egui::ScrollArea::vertical()
            .id_salt("champion-pool-roster-scroll")
            .max_height(420.0)
            .show(ui, |ui| {
                for chunk in roster.chunks(cols) {
                    ui.horizontal_top(|ui| {
                        for (idx, champ) in chunk.iter().enumerate() {
                            if idx > 0 {
                                ui.add_space(gap);
                            }
                            let is_fav = ctx.store.settings.favorite_champions.contains(&champ.id);
                            let assign_mode = ctx.state.champion_pool.assign_slot.is_some();
                            let build_role = if champ.roles.contains(&active_role) {
                                active_role
                            } else {
                                champ.roles[0]
                            };
                            let profile = builds::build_for_champion(champ.id, build_role);

                            ui.allocate_ui_with_layout(
                                Vec2::new(card_w, 108.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    inset_card(ui, &palette, |ui| {
                                        ui.set_min_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            let tex =
                                                ctx.textures.champion(ui.ctx(), champ.key);
                                            avatar(
                                                ui,
                                                &palette,
                                                tex.as_ref(),
                                                champ.name,
                                                30.0,
                                                role_color(&palette, build_role.as_str()),
                                            );
                                            ui.vertical(|ui| {
                                                ui.label(
                                                    RichText::new(champ.name)
                                                        .size(12.0)
                                                        .strong()
                                                        .color(palette.text),
                                                );
                                                ui.horizontal_wrapped(|ui| {
                                                    role_tag(ui, &palette, build_role);
                                                    badge(
                                                        ui,
                                                        profile.archetype.label(),
                                                        palette.violet,
                                                    );
                                                });
                                            });
                                        });
                                        ui.add_space(6.0);
                                        ui.horizontal(|ui| {
                                            let star = if is_fav { "★" } else { "☆" };
                                            let star_col =
                                                if is_fav { palette.gold } else { palette.muted };
                                            if action_button(ui, &palette, star, star_col) {
                                                toggle_fav = Some(champ.id);
                                            }
                                            if assign_mode {
                                                if action_button(
                                                    ui,
                                                    &palette,
                                                    "ASSIGN",
                                                    palette.green,
                                                ) {
                                                    assign_champ = Some(champ.id);
                                                }
                                            } else if action_button(
                                                ui,
                                                &palette,
                                                "SETUP",
                                                palette.accent,
                                            ) {
                                                open_champ = Some(champ.id);
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

        if let Some(champ_id) = toggle_fav {
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
        if let Some(champ_id) = assign_champ {
            if let Some(slot_idx) = ctx.state.champion_pool.assign_slot.take() {
                let entry = ctx
                    .store
                    .settings
                    .champion_pool
                    .entry(role_key)
                    .or_default();
                if slot_idx < entry.len() {
                    entry[slot_idx] = champ_id;
                    ctx.request_save();
                }
            }
        }
        if let Some(champ_id) = open_champ {
            ctx.navigate_to_champion(champ_id, Some(active_role));
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn render_plan_slot(
    ctx: &mut PageContext<'_>,
    ui: &mut egui::Ui,
    width: f32,
    slot_idx: usize,
    champ_id: u32,
    role: Role,
    open_champ: &mut Option<u32>,
    clear_slot: &mut Option<usize>,
) {
    let palette = *ctx.palette;
    let assigning = ctx.state.champion_pool.assign_slot == Some(slot_idx);
    let accent = match slot_idx {
        0 => palette.accent,
        1 => palette.gold,
        _ => palette.violet,
    };
    let fill = if assigning {
        crate::ui::theme::ThemePalette::with_alpha(accent, 30)
    } else {
        palette.surface_inset
    };
    let stroke = if assigning {
        Stroke::new(1.5, accent)
    } else {
        Stroke::new(1.0, palette.border)
    };

    ui.allocate_ui_with_layout(
        Vec2::new(width, 120.0),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            Frame::new()
                .fill(fill)
                .stroke(stroke)
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::same(10))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        badge(ui, SLOT_LABELS[slot_idx], accent);
                    });
                    ui.add_space(6.0);

                    if let Some(champ) = champions::champion_by_id(champ_id) {
                        let profile = builds::build_for_champion(champ.id, role);
                        ui.horizontal(|ui| {
                            let tex = ctx.textures.champion(ui.ctx(), champ.key);
                            avatar(
                                ui,
                                &palette,
                                tex.as_ref(),
                                champ.name,
                                34.0,
                                role_color(&palette, role.as_str()),
                            );
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(champ.name)
                                        .size(13.0)
                                        .strong()
                                        .color(palette.text),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "{} · {}",
                                        profile.archetype.label(),
                                        profile.compact_skill_priority()
                                    ))
                                    .size(10.5)
                                    .color(palette.muted),
                                );
                            });
                        });
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if action_button(ui, &palette, "OPEN BUILD", accent) {
                                *open_champ = Some(champ.id);
                            }
                            if action_button(ui, &palette, "SWAP", palette.violet) {
                                ctx.state.champion_pool.assign_slot = Some(slot_idx);
                            }
                            if action_button(ui, &palette, "CLEAR", palette.red) {
                                *clear_slot = Some(slot_idx);
                            }
                        });
                    } else {
                        ui.label(
                            RichText::new("Empty draft slot")
                                .size(11.5)
                                .color(palette.muted),
                        );
                        ui.add_space(8.0);
                        let label = if assigning {
                            "PICK FROM ROSTER BELOW"
                        } else {
                            "ASSIGN CHAMPION"
                        };
                        if action_button(ui, &palette, label, accent) {
                            ctx.state.champion_pool.assign_slot = if assigning {
                                None
                            } else {
                                Some(slot_idx)
                            };
                        }
                    }
                });
        },
    );
}
