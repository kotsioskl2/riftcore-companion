//! Game Modes Hub: Summoner's Rift, ARAM, Arena, URF, One for All, and ARAM Mayhem.
//!
//! Riot policy compliance:
//! - Uses `crate::domain::mayhem` where all `S+`/`S`/`A` subjective rank labels are removed.

use crate::domain::builds;
use crate::domain::champions::{self, Champion};
use crate::domain::mayhem::{filter_cards, CardMode, CardTier};
use crate::domain::protocol::{CoreCommand, CoreNotice};
use crate::domain::runes;
use crate::ui::pages::{spell_name, Page, PageContext};
use crate::ui::widgets::{
    action_button, avatar, badge, inset_card, metric_card, page_heading, panel, panel_title,
    search_box,
};
use eframe::egui::{self, RichText};

pub fn render(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;

    page_heading(
        ui,
        &palette,
        "GAME MODE CENTER",
        "Game Modes",
        "Mode-specific setup, rune & spell imports, strategy prompts, and ARAM Mayhem / Arena augment cards.",
        ctx.snapshot,
    );

    ui.horizontal_wrapped(|ui| {
        for (index, label) in ["RIFT", "ARAM", "ARENA", "URF", "ONE FOR ALL", "ARAM MAYHEM"]
            .iter()
            .enumerate()
        {
            let accent = if index == 1 || index == 5 {
                palette.gold
            } else if index == 4 {
                palette.violet
            } else if index == 3 {
                palette.red
            } else {
                palette.accent
            };
            if action_button(
                ui,
                &palette,
                label,
                ctx.state.modes.selected_mode == index,
                accent,
            )
            .clicked()
            {
                ctx.state.modes.selected_mode = index;
            }
        }
        if action_button(
            ui,
            &palette,
            "OPEN MAYHEM & ARENA CARD GUIDE",
            false,
            palette.gold,
        )
        .clicked()
        {
            ctx.state.show_mayhem_cards = true;
        }
    });

    ui.add_space(10.0);
    let mode_index = ctx.state.modes.selected_mode.min(5);
    let (mode_name, subtitle, details, expected_mode, accent) = match mode_index {
        1 => (
            "ARAM",
            "HOWLING ABYSS",
            "Choose ARAM runes and Mark/Snowball spells, then inspect a champion's build. Patch balance modifiers are not guessed.",
            "ARAM",
            palette.gold,
        ),
        2 => (
            "ARENA",
            "ARENA 2V2",
            "Build complementary duos around engage, durability, and sustained damage. Browse Arena augments below.",
            "CHERRY",
            palette.gold,
        ),
        3 => (
            "URF",
            "ULTRA RAPID FIRE",
            "Prioritize reliable crowd control, mobility, and low-cooldown repeatable spell rotations.",
            "URF",
            palette.red,
        ),
        4 => (
            "ONE FOR ALL",
            "ONE FOR ALL",
            "Coordinate your team vote around wave clear, engage, and range flexibility into mirror or counter comps.",
            "ONEFORALL",
            palette.violet,
        ),
        5 => (
            "ARAM MAYHEM",
            "ARAM MAYHEM",
            "Browse curated Mayhem and Arena augments by rarity tier, mode fit, and tactical effect.",
            "ARAM",
            palette.gold,
        ),
        _ => (
            "SUMMONER'S RIFT",
            "RANKED & NORMAL",
            "Role-based builds, rune pages, summoner spells, and objective timing references.",
            "CLASSIC",
            palette.accent,
        ),
    };

    let detected = ctx
        .state
        .parsed_live_game
        .as_ref()
        .map(|g| g.game_mode.as_str())
        .unwrap_or("—");

    panel(ui, &palette, |ui| {
        panel_title(ui, &palette, subtitle, details);
        ui.horizontal_wrapped(|ui| {
            metric_card(
                ui,
                &palette,
                "CLIENT",
                if ctx.snapshot.connected {
                    "CONNECTED"
                } else {
                    "OFFLINE"
                },
                if ctx.snapshot.connected {
                    palette.green
                } else {
                    palette.muted
                },
            );
            metric_card(
                ui,
                &palette,
                "LIVE MODE",
                detected,
                if detected.eq_ignore_ascii_case(expected_mode) {
                    palette.green
                } else {
                    palette.gold
                },
            );
        });
    });

    ui.add_space(10.0);
    if let Some(champ) = champions::champion_by_id(ctx.state.champ_select.selected_champion) {
        let build = builds::build_for_champion(champ.id, ctx.state.champ_select.build_role)
            .cloned()
            .unwrap_or_else(|| builds::recommend(&champ.key, &champ.name, &champ.roles, None));
        let (spell_one, spell_two) = match mode_index {
            1 | 5 => (4, 32), // Flash + Mark/Snowball
            2 | 3 => (4, 14), // Flash + Ignite
            4 => (4, 6),      // Flash + Ghost
            _ => (build.spell_one, build.spell_two),
        };

        panel(ui, &palette, |ui| {
            panel_title(
                ui,
                &palette,
                &format!("{mode_name} SETUP · {}", champ.name),
                "Champion build profile and explicit rune/spell import",
            );
            ui.horizontal(|ui| {
                let egui_ctx = ui.ctx().clone();
                let portrait = ctx.textures.champion(&egui_ctx, &champ.key);
                avatar(ui, &palette, &champ.name, 52.0, accent, portrait);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "{} · {}",
                            build.role.as_str(),
                            build.archetype.display_name()
                        ))
                        .size(12.0)
                        .strong()
                        .color(palette.text),
                    );
                    ui.label(
                        RichText::new(format!("Skill order: {}", build.compact_skill_priority()))
                            .size(10.5)
                            .color(palette.accent),
                    );
                    ui.add(
                        egui::Label::new(
                            RichText::new(&build.tip)
                                .size(10.5)
                                .color(palette.secondary),
                        )
                        .wrap(),
                    );
                });
            });
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if action_button(ui, &palette, "OPEN FULL BUILD", false, palette.accent).clicked() {
                    ctx.navigate_to_champion(champ.id, Some(build.role));
                }
                if action_button(ui, &palette, "APPLY MODE RUNES + SPELLS", false, accent).clicked()
                {
                    let (primary, secondary, perks) = match mode_index {
                        1 | 5 => (
                            runes::TREE_DOMINATION,
                            runes::TREE_PRECISION,
                            vec![
                                8128,
                                8143,
                                runes::RUNE_GRISLY_MEMENTOS,
                                8135,
                                8009,
                                9105,
                                5008,
                                5008,
                                5001,
                            ],
                        ),
                        2 => (
                            runes::TREE_PRECISION,
                            runes::TREE_RESOLVE,
                            vec![8010, 9111, 9104, 8299, 8473, 8451, 5008, 5008, 5001],
                        ),
                        3 => (
                            runes::TREE_DOMINATION,
                            runes::TREE_SORCERY,
                            vec![
                                8128,
                                8139,
                                runes::RUNE_GRISLY_MEMENTOS,
                                8135,
                                8226,
                                8210,
                                5008,
                                5008,
                                5001,
                            ],
                        ),
                        _ => (
                            build.primary_tree,
                            build.secondary_tree,
                            build.rune_ids.iter().map(|&id| i64::from(id)).collect(),
                        ),
                    };
                    let page = if mode_index == 0 {
                        builds::rune_page(&champ.name, &build)
                    } else {
                        runes::build_rune_page(
                            &format!("RiftCore {mode_name} {}", champ.name),
                            primary,
                            secondary,
                            &perks,
                        )
                    };
                    ctx.send_command(CoreCommand::ImportRunes(page.to_json()));
                    ctx.send_command(CoreCommand::ImportSpells {
                        spell1: spell_one,
                        spell2: spell_two,
                    });
                    ctx.set_notice(CoreNotice::Info(format!(
                        "Applying {mode_name} runes and spells…"
                    )));
                }
                if action_button(ui, &palette, "OPEN LIVE HUD", false, palette.secondary).clicked()
                {
                    ctx.state.active_page = Page::Hud;
                }
                ui.label(
                    RichText::new(format!(
                        "Suggested spells: {} + {}",
                        spell_name(spell_one),
                        spell_name(spell_two)
                    ))
                    .size(10.0)
                    .color(palette.gold),
                );
            });
        });
    }

    ui.add_space(10.0);
    match mode_index {
        1 => {
            panel(ui, &palette, |ui| {
                panel_title(
                    ui,
                    &palette,
                    "ARAM TEAM PLAN",
                    "Bridge composition balance checklist",
                );
                ui.add(
                    egui::Label::new(
                        RichText::new("Mark/Snowball enables melee engage; balance range, wave clear, reliable engage, and sustain. Patch-specific balance modifiers are intentionally omitted.")
                            .size(10.5)
                            .color(palette.secondary),
                    )
                    .wrap(),
                );
            });
        }
        2 => {
            panel(ui, &palette, |ui| {
                panel_title(
                    ui,
                    &palette,
                    "DUO SYNERGY IDEAS",
                    "Pair strategy prompts · no fabricated win-rate rankings",
                );
                for (pair, strategy) in [
                    ("Taric + Master Yi", "Protect a melee carry through an all-in and layer Cosmic Radiance invulnerability."),
                    ("Ivern + Rengar", "Create brush angles for repeated leap resets and shield explosions."),
                    ("Alistar + Poppy", "Chain knockbacks into ring edges as the circle closes."),
                    ("Kayle + Braum", "Absorb burst engage while scaling carry ramps up."),
                    ("Singed + Cassiopeia", "Share poison triggers for instant Twin Fang resets in tight arenas."),
                ] {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(pair).size(10.5).strong().color(palette.gold));
                        ui.label(
                            RichText::new(strategy)
                                .size(10.0)
                                .color(palette.secondary),
                        );
                    });
                }
            });
        }
        3 => {
            panel(ui, &palette, |ui| {
                panel_title(
                    ui,
                    &palette,
                    "URF BUILD PRINCIPLES",
                    "General picks guidance",
                );
                ui.add(
                    egui::Label::new(
                        RichText::new("Favor low-windup crowd control, dashes, and repeatable spell rotations. Check client tooltip cooldown caps before committing.")
                            .size(10.5)
                            .color(palette.secondary),
                    )
                    .wrap(),
                );
            });
        }
        4 => {
            panel(ui, &palette, |ui| {
                panel_title(
                    ui,
                    &palette,
                    "ONE FOR ALL CHECKLIST",
                    "Vote evaluation guide",
                );
                ui.add(
                    egui::Label::new(
                        RichText::new("Prioritize reliable wave clear, engage tools, and mixed damage patterns so all 5 copies cannot be countered by a single defensive item.")
                            .size(10.5)
                            .color(palette.secondary),
                    )
                    .wrap(),
                );
            });
        }
        5 => render_mayhem_inline_catalog(ctx, ui),
        _ => {
            panel(ui, &palette, |ui| {
                panel_title(
                    ui,
                    &palette,
                    "SUMMONER'S RIFT MACRO CHECKLIST",
                    "Lane priority and objective setup timings",
                );
                ui.add(
                    egui::Label::new(
                        RichText::new("Push mid/side waves 45s prior to Dragon, Voidgrubs (05:00), Rift Herald (14:00), or Baron (20:00) to establish river vision safely.")
                            .size(10.5)
                            .color(palette.secondary),
                    )
                    .wrap(),
                );
            });
        }
    }

    ui.add_space(10.0);
    render_mode_champion_picker(ctx, ui);
}

fn render_mayhem_inline_catalog(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;

    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            "ARAM MAYHEM & ARENA AUGMENTS",
            "Curated augment guide by tier and mode fit (no subjective S+/S/A tiers)",
        );
        ui.horizontal_wrapped(|ui| {
            for mode in [CardMode::Both, CardMode::Aram, CardMode::Arena] {
                if action_button(
                    ui,
                    &palette,
                    &mode.as_str().to_ascii_uppercase(),
                    ctx.state.modes.mayhem_mode == mode,
                    palette.gold,
                )
                .clicked()
                {
                    ctx.state.modes.mayhem_mode = mode;
                }
            }
            ui.add_space(6.0);
            if action_button(
                ui,
                &palette,
                "ALL TIERS",
                ctx.state.modes.mayhem_tier.is_none(),
                palette.accent,
            )
            .clicked()
            {
                ctx.state.modes.mayhem_tier = None;
            }
            for tier in [CardTier::Prismatic, CardTier::Gold, CardTier::Silver] {
                if action_button(
                    ui,
                    &palette,
                    &tier.as_str().to_ascii_uppercase(),
                    ctx.state.modes.mayhem_tier == Some(tier),
                    palette.accent,
                )
                .clicked()
                {
                    ctx.state.modes.mayhem_tier = Some(tier);
                }
            }
        });
        ui.add_space(6.0);
        search_box(
            ui,
            &palette,
            "modes-mayhem-search",
            &mut ctx.state.modes.mayhem_search,
            "Search augment name, tag, or champion fit…",
            Some(ui.available_width().min(420.0)),
        );
        ui.add_space(6.0);

        let filtered = filter_cards(
            Some(ctx.state.modes.mayhem_mode),
            ctx.state.modes.mayhem_tier,
            &ctx.state.modes.mayhem_search,
        );
        if filtered.is_empty() {
            ui.label(
                RichText::new("No augments match those filters.")
                    .size(10.5)
                    .color(palette.secondary),
            );
        } else {
            for card in filtered {
                let tint = match card.tier {
                    CardTier::Prismatic => palette.violet,
                    CardTier::Gold => palette.gold,
                    CardTier::Silver => palette.secondary,
                };
                inset_card(ui, &palette, palette.border, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(&card.name)
                                .size(12.0)
                                .strong()
                                .color(palette.text),
                        );
                        badge(ui, card.tier.as_str(), tint);
                        badge(ui, card.mode.as_str(), palette.gold);
                        badge(ui, &card.tag, palette.accent);
                    });
                    ui.label(
                        RichText::new(&card.archetype)
                            .size(9.5)
                            .color(palette.secondary),
                    );
                    ui.add(
                        egui::Label::new(
                            RichText::new(&card.description)
                                .size(10.5)
                                .color(palette.text),
                        )
                        .wrap(),
                    );
                    ui.add(
                        egui::Label::new(RichText::new(&card.tip).size(10.0).color(palette.gold))
                            .wrap(),
                    );
                });
                ui.add_space(5.0);
            }
        }
    });
}

fn render_mode_champion_picker(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;
    let mut selected_id: Option<u32> = None;

    panel(ui, &palette, |ui| {
        panel_title(
            ui,
            &palette,
            "CHAMPION PICKER",
            "Choose a champion for this mode's build and rune/spell setup",
        );
        search_box(
            ui,
            &palette,
            "modes-champ-search",
            &mut ctx.state.modes.search,
            "Search champions…",
            Some(280.0),
        );
        ui.add_space(6.0);

        let query = ctx.state.modes.search.trim().to_ascii_lowercase();
        let visible: Vec<&'static Champion> = champions::all_champions()
            .iter()
            .filter(|champ| query.is_empty() || champ.name.to_ascii_lowercase().contains(&query))
            .take(24)
            .collect();

        let num_cols = ((ui.available_width() - 12.0) / 145.0).floor().max(2.0) as usize;
        let egui_ctx = ui.ctx().clone();

        egui::Grid::new("modes-champ-grid-v4")
            .num_columns(num_cols)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                for (index, champ) in visible.iter().enumerate() {
                    let is_selected = ctx.state.champ_select.selected_champion == champ.id;
                    ui.horizontal(|ui| {
                        let portrait = ctx.textures.champion(&egui_ctx, &champ.key);
                        avatar(ui, &palette, &champ.name, 28.0, palette.accent, portrait);
                        if action_button(ui, &palette, &champ.name, is_selected, palette.accent)
                            .clicked()
                        {
                            selected_id = Some(champ.id);
                        }
                    });
                    if index % num_cols == num_cols - 1 {
                        ui.end_row();
                    }
                }
            });
    });

    if let Some(id) = selected_id {
        ctx.select_champion_setup(id, None, false);
    }
}
