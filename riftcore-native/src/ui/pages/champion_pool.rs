//! Champion Pool Planner page: per-role 3-slot draft plan (Blind / Counter / Pocket) + favorite roster.

use crate::domain::builds;
use crate::domain::champions::{self, Champion, Role};
use crate::ui::pages::PageContext;
use crate::ui::theme::{role_color, ThemePalette};
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
        "DRAFT PREPARATION",
        "Champion Pool Planner",
        "Curate your 3-slot role matrix (Blind / Counter / Pocket) and pin favorite builds for fast Champ Select access",
        ctx.snapshot,
    );

    panel(ui, &palette, |ui| {
        ui.horizontal_wrapped(|ui| {
            for role in Role::ALL {
                let active = ctx.state.champion_pool.role == role;
                if filter_chip(
                    ui,
                    &palette,
                    role.as_str(),
                    active,
                    role_color(role, &palette),
                )
                .clicked()
                {
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

        let mut actions = SlotActions::default();

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
                        &mut actions,
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
                    &mut actions,
                );
                ui.add_space(6.0);
            }
        }

        if let Some(slot_idx) = actions.clear_slot {
            if let Some(slot) = ctx
                .store
                .settings
                .champion_pool
                .get_mut(&role_key)
                .and_then(|entry| entry.get_mut(slot_idx))
            {
                *slot = 0;
                ctx.request_save();
            }
        }
        if let Some(champ_id) = actions.open_champ {
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
                "champion-pool-search",
                &mut ctx.state.champion_pool.search,
                "Search champions to assign or favorite...",
                Some(320.0),
            );
            if ctx.state.champion_pool.assign_slot.is_some()
                && action_button(ui, &palette, "CANCEL ASSIGN", false, palette.muted).clicked()
            {
                ctx.state.champion_pool.assign_slot = None;
            }
        });
        ui.add_space(8.0);

        let query = ctx.state.champion_pool.search.trim().to_ascii_lowercase();
        let mut roster: Vec<&Champion> = champions::all_champions()
            .iter()
            .filter(|champ| {
                if query.is_empty() {
                    champ.supports_role(active_role)
                } else {
                    champ.name.to_ascii_lowercase().contains(&query)
                }
            })
            .collect();

        roster.sort_by(|a, b| {
            let a_fav = ctx.store.settings.favorite_champions.contains(&a.id);
            let b_fav = ctx.store.settings.favorite_champions.contains(&b.id);
            b_fav.cmp(&a_fav).then_with(|| a.name.cmp(&b.name))
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
                            let build_role = if champ.supports_role(active_role) {
                                active_role
                            } else {
                                champ.primary_role()
                            };
                            let profile = builds::build_for_champion(champ.id, build_role);

                            ui.allocate_ui_with_layout(
                                Vec2::new(card_w, 108.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    let card_stroke = if is_fav {
                                        ThemePalette::with_alpha(palette.gold, 120)
                                    } else {
                                        palette.border
                                    };
                                    inset_card(ui, &palette, card_stroke, |ui| {
                                        ui.set_min_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            let tex = ctx.textures.champion(ui.ctx(), &champ.key);
                                            avatar(
                                                ui,
                                                &palette,
                                                &champ.name,
                                                30.0,
                                                role_color(build_role, &palette),
                                                tex,
                                            );
                                            ui.vertical(|ui| {
                                                ui.label(
                                                    RichText::new(&champ.name)
                                                        .size(12.0)
                                                        .strong()
                                                        .color(palette.text),
                                                );
                                                ui.horizontal_wrapped(|ui| {
                                                    role_tag(ui, build_role, &palette);
                                                    match profile {
                                                        Some(profile) => badge(
                                                            ui,
                                                            &profile.archetype.display_name(),
                                                            palette.violet,
                                                        ),
                                                        None => badge(
                                                            ui,
                                                            "NO BUILD DATA",
                                                            palette.muted,
                                                        ),
                                                    }
                                                });
                                            });
                                        });
                                        ui.add_space(6.0);
                                        ui.horizontal(|ui| {
                                            let star = if is_fav { "★" } else { "☆" };
                                            let star_col =
                                                if is_fav { palette.gold } else { palette.muted };
                                            if action_button(ui, &palette, star, is_fav, star_col)
                                                .clicked()
                                            {
                                                toggle_fav = Some(champ.id);
                                            }
                                            if assign_mode {
                                                if action_button(
                                                    ui,
                                                    &palette,
                                                    "ASSIGN",
                                                    true,
                                                    palette.green,
                                                )
                                                .clicked()
                                                {
                                                    assign_champ = Some(champ.id);
                                                }
                                            } else if action_button(
                                                ui,
                                                &palette,
                                                "SETUP",
                                                false,
                                                palette.accent,
                                            )
                                            .clicked()
                                            {
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
                if entry.len() < SLOT_LABELS.len() {
                    entry.resize(SLOT_LABELS.len(), 0);
                }
                if let Some(slot) = entry.get_mut(slot_idx) {
                    *slot = champ_id;
                    ctx.request_save();
                }
            }
        }
        if let Some(champ_id) = open_champ {
            ctx.navigate_to_champion(champ_id, Some(active_role));
        }
    });
}

/// Deferred slot interactions collected while rendering plan cards.
#[derive(Default)]
struct SlotActions {
    open_champ: Option<u32>,
    clear_slot: Option<usize>,
}

fn render_plan_slot(
    ctx: &mut PageContext<'_>,
    ui: &mut egui::Ui,
    width: f32,
    slot_idx: usize,
    champ_id: u32,
    role: Role,
    actions: &mut SlotActions,
) {
    let palette = *ctx.palette;
    let assigning = ctx.state.champion_pool.assign_slot == Some(slot_idx);
    let accent = match slot_idx {
        0 => palette.accent,
        1 => palette.gold,
        _ => palette.violet,
    };
    let fill = if assigning {
        ThemePalette::with_alpha(accent, 30)
    } else {
        palette.inset
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
                            let tex = ctx.textures.champion(ui.ctx(), &champ.key);
                            avatar(
                                ui,
                                &palette,
                                &champ.name,
                                34.0,
                                role_color(role, &palette),
                                tex,
                            );
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(&champ.name)
                                        .size(13.0)
                                        .strong()
                                        .color(palette.text),
                                );
                                ui.label(
                                    RichText::new(profile.map_or_else(
                                        || "No build data for this role".to_string(),
                                        |profile| {
                                            format!(
                                                "{} · {}",
                                                profile.archetype.display_name(),
                                                profile.compact_skill_priority()
                                            )
                                        },
                                    ))
                                    .size(10.5)
                                    .color(palette.muted),
                                );
                            });
                        });
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if action_button(ui, &palette, "OPEN BUILD", false, accent).clicked() {
                                actions.open_champ = Some(champ.id);
                            }
                            if action_button(ui, &palette, "SWAP", assigning, palette.violet)
                                .clicked()
                            {
                                ctx.state.champion_pool.assign_slot = Some(slot_idx);
                            }
                            if action_button(ui, &palette, "CLEAR", false, palette.red).clicked() {
                                actions.clear_slot = Some(slot_idx);
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
                        if action_button(ui, &palette, label, assigning, accent).clicked() {
                            ctx.state.champion_pool.assign_slot =
                                if assigning { None } else { Some(slot_idx) };
                        }
                    }
                });
        },
    );
}
