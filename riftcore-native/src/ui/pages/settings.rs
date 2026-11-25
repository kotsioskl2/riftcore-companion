//! Settings workspace: gameplay automation, League client controls, window/overlay
//! behavior, HUD configuration, Riot API credentials, appearance and updates.
//!
//! All side effects (saving, theme/font application, window level, passthrough,
//! opening URLs) are routed through `PendingUiEffects` / `CoreCommand` so the page
//! stays a pure render over `PageContext`.

use crate::domain::protocol::{CoreCommand, CoreNotice};
use crate::ui::pages::{hud_page, PageContext, SettingsSection};
use crate::ui::theme::{FontChoice, ThemeMode, ThemePalette};
use crate::ui::widgets::{action_button, page_heading, panel, panel_title, toggle_row};
use eframe::egui::{self, Align, Color32, Layout, RichText};
use serde_json::Value;

/// Platform routing values accepted by the Riot API service.
pub const RIOT_REGIONS: [&str; 11] = [
    "eun1", "euw1", "na1", "kr", "br1", "la1", "la2", "oc1", "tr1", "ru", "jp1",
];

pub fn render(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;
    page_heading(
        ui,
        &palette,
        "RIFTCORE CONTROL",
        "Preferences",
        "Choose a category to manage gameplay, the HUD, Riot API, and app behavior.",
        ctx.snapshot,
    );
    ui.add_space(10.0);

    ui.horizontal_wrapped(|ui| {
        for (section, label, accent) in [
            (SettingsSection::Gameplay, "GAMEPLAY", palette.accent),
            (SettingsSection::Overlay, "OVERLAY", palette.gold),
            (SettingsSection::RiotApi, "RIOT API", palette.green),
            (SettingsSection::App, "APP", palette.secondary),
        ] {
            let active = ctx.state.settings.section == section;
            if action_button(ui, &palette, label, active, accent).clicked() {
                ctx.state.settings.section = section;
            }
        }
    });
    ui.add_space(10.0);

    match ctx.state.settings.section {
        SettingsSection::Gameplay => {
            render_automation(ctx, ui, &palette);
            ui.add_space(10.0);
            render_league_client(ctx, ui, &palette);
        }
        SettingsSection::Overlay => {
            render_window_overlay(ctx, ui, &palette);
            ui.add_space(10.0);
            hud_page::render_hud_settings(ctx, ui);
        }
        SettingsSection::RiotApi => render_riot_api(ctx, ui, &palette),
        SettingsSection::App => {
            render_appearance(ctx, ui, &palette);
            ui.add_space(10.0);
            render_updates(ctx, ui, &palette);
        }
    }

    ui.add_space(12.0);
    panel(ui, &palette, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("PREFERENCES")
                        .size(11.0)
                        .strong()
                        .color(palette.text),
                );
                ui.label(
                    RichText::new("Saved locally. Save after changing your Riot API key.")
                        .size(10.5)
                        .color(palette.muted),
                );
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if action_button(ui, &palette, "SAVE PREFERENCES", false, palette.accent).clicked()
                {
                    ctx.send_command(CoreCommand::SetAutoAccept(ctx.store.settings.auto_accept));
                    ctx.request_save();
                    ctx.set_notice(CoreNotice::Success("Preferences saved".to_string()));
                }
            });
        });
    });
}

fn render_automation(ctx: &mut PageContext<'_>, ui: &mut egui::Ui, palette: &ThemePalette) {
    panel(ui, palette, |ui| {
        panel_title(ui, palette, "GAMEPLAY AUTOMATION", "League client actions");
        let s = &mut ctx.store.settings;
        let mut changed = false;
        changed |= toggle_row(
            ui,
            palette,
            "Auto accept queue",
            "Accept the ready check automatically",
            &mut s.auto_accept,
        );
        changed |= toggle_row(
            ui,
            palette,
            "Auto import runes",
            "Apply the selected rune page in champion select",
            &mut s.auto_import_runes,
        );
        changed |= toggle_row(
            ui,
            palette,
            "Auto import summoner spells",
            "Apply the selected spells in champion select",
            &mut s.auto_import_spells,
        );
        changed |= toggle_row(
            ui,
            palette,
            "Auto export item sets",
            "Add the selected build to the League shop",
            &mut s.auto_import_item_sets,
        );
        changed |= toggle_row(
            ui,
            palette,
            "Audio alerts",
            "Play a cue when a ready check appears",
            &mut s.sound_alerts,
        );
        if changed {
            let auto_accept = s.auto_accept;
            ctx.send_command(CoreCommand::SetAutoAccept(auto_accept));
            ctx.request_save();
        }
    });
}

fn render_league_client(ctx: &mut PageContext<'_>, ui: &mut egui::Ui, palette: &ThemePalette) {
    panel(ui, palette, |ui| {
        panel_title(ui, palette, "LEAGUE CLIENT", "Launch and display controls");
        ui.horizontal_wrapped(|ui| {
            if action_button(ui, palette, "LAUNCH LEAGUE", false, palette.accent).clicked() {
                ctx.send_command(CoreCommand::LaunchLeague);
            }
            if action_button(ui, palette, "BORDERLESS WINDOW", false, palette.accent).clicked() {
                ctx.send_command(CoreCommand::SetLeagueWindowMode(2));
            }
            if action_button(ui, palette, "FULLSCREEN", false, palette.secondary).clicked() {
                ctx.send_command(CoreCommand::SetLeagueWindowMode(0));
            }
        });
        ui.add_space(6.0);
        ui.label(
            RichText::new(
                "Display changes back up game.cfg before writing. Restart League to apply them.",
            )
            .size(10.0)
            .color(palette.muted),
        );
    });
}

fn render_window_overlay(ctx: &mut PageContext<'_>, ui: &mut egui::Ui, palette: &ThemePalette) {
    panel(ui, palette, |ui| {
        panel_title(
            ui,
            palette,
            "WINDOW & OVERLAY",
            "Desktop behavior and HUD input",
        );
        if toggle_row(
            ui,
            palette,
            "Always on top",
            "Keep the companion above other windows",
            &mut ctx.store.settings.always_on_top,
        ) {
            ctx.state.pinned = ctx.store.settings.always_on_top;
            ctx.effects.apply_always_on_top = true;
            ctx.request_save();
        }
        if toggle_row(
            ui,
            palette,
            "Force click-through",
            "Pass every click to League; HUD controls become unavailable",
            &mut ctx.state.overlay_mouse_passthrough,
        ) {
            ctx.effects.set_passthrough = Some(ctx.state.overlay_mouse_passthrough);
        }
        ui.label(
            RichText::new("Alt + ~ toggles the HUD. Alt + Shift + ~ toggles click-through.")
                .size(10.0)
                .color(palette.muted),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Flash key")
                    .size(11.5)
                    .color(palette.secondary),
            );
            let flash = &mut ctx.store.settings.flash_slot_preference;
            let current = if flash == "D" { "D" } else { "F" };
            let mut changed = false;
            egui::ComboBox::from_id_salt("flash-slot")
                .selected_text(current)
                .show_ui(ui, |ui| {
                    for key in ["F", "D"] {
                        changed |= ui.selectable_value(flash, key.to_string(), key).changed();
                    }
                });
            if changed {
                ctx.request_save();
            }
        });
    });
}

fn render_riot_api(ctx: &mut PageContext<'_>, ui: &mut egui::Ui, palette: &ThemePalette) {
    panel(ui, palette, |ui| {
        panel_title(ui, palette, "RIOT API", "Player scouting and match history");
        ui.label(
            RichText::new("PLATFORM REGION")
                .size(10.0)
                .strong()
                .color(palette.muted),
        );
        let mut region_changed = false;
        {
            let region = &mut ctx.store.settings.riot_region;
            egui::ComboBox::from_id_salt("riot-region")
                .selected_text(region.to_ascii_uppercase())
                .width(220.0)
                .show_ui(ui, |ui| {
                    for option in RIOT_REGIONS {
                        region_changed |= ui
                            .selectable_value(
                                region,
                                option.to_string(),
                                option.to_ascii_uppercase(),
                            )
                            .changed();
                    }
                });
        }
        if region_changed {
            ctx.request_save();
        }

        ui.add_space(8.0);
        ui.label(
            RichText::new("API KEY")
                .size(10.0)
                .strong()
                .color(palette.muted),
        );
        let show = ctx.state.settings.show_api_key;
        ui.horizontal(|ui| {
            let width = (ui.available_width() - 70.0).max(160.0);
            ui.add_sized(
                [width, 34.0],
                egui::TextEdit::singleline(&mut ctx.store.settings.riot_api_key)
                    .password(!show)
                    .hint_text("RGAPI-…"),
            );
            if action_button(
                ui,
                palette,
                if show { "HIDE" } else { "SHOW" },
                show,
                palette.secondary,
            )
            .clicked()
            {
                ctx.state.settings.show_api_key = !show;
            }
        });
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            let has_key = !ctx.store.settings.riot_api_key.trim().is_empty();
            if action_button(ui, palette, "TEST API KEY", false, palette.accent).clicked() {
                if has_key {
                    let region = ctx.store.settings.riot_region.clone();
                    let api_key = ctx.store.settings.riot_api_key.trim().to_string();
                    ctx.send_command(CoreCommand::ValidateRiotApiKey { region, api_key });
                } else {
                    ctx.set_notice(CoreNotice::Warning(
                        "Enter a Riot API key first".to_string(),
                    ));
                }
            }
            if let Some(status) = ctx.snapshot.riot_api_status.as_deref() {
                let color = if status.contains("verified") {
                    palette.green
                } else {
                    palette.gold
                };
                ui.label(RichText::new(status).size(10.0).color(color));
            }
        });
        ui.add_space(6.0);
        ui.label(
            RichText::new(
                "The key is encrypted with Windows DPAPI for your user account and sent only to Riot API endpoints.",
            )
            .size(10.0)
            .color(palette.muted),
        );
    });
}

fn render_appearance(ctx: &mut PageContext<'_>, ui: &mut egui::Ui, palette: &ThemePalette) {
    panel(ui, palette, |ui| {
        panel_title(ui, palette, "APPEARANCE & BEHAVIOR", "Tune the app itself");

        ui.label(
            RichText::new("INTERFACE THEME")
                .size(10.0)
                .strong()
                .color(palette.muted),
        );
        let current_theme = ThemeMode::from_str_lossy(&ctx.store.settings.theme);
        let mut theme_changed = false;
        egui::ComboBox::from_id_salt("native-theme")
            .selected_text(current_theme.display_name())
            .width(220.0)
            .show_ui(ui, |ui| {
                for mode in [ThemeMode::Hextech, ThemeMode::Ember, ThemeMode::Void] {
                    if ui
                        .selectable_label(mode == current_theme, mode.display_name())
                        .clicked()
                        && mode != current_theme
                    {
                        ctx.store.settings.theme = mode.as_str().to_string();
                        theme_changed = true;
                    }
                }
            });

        ui.add_space(8.0);
        ui.label(
            RichText::new("TYPOGRAPHY")
                .size(10.0)
                .strong()
                .color(palette.muted),
        );
        let current_font = FontChoice::from_str_lossy(&ctx.store.settings.font_family);
        let mut font_changed = false;
        egui::ComboBox::from_id_salt("native-font")
            .selected_text(current_font.display_name())
            .width(220.0)
            .show_ui(ui, |ui| {
                for font in [
                    FontChoice::PlusJakarta,
                    FontChoice::Outfit,
                    FontChoice::Inter,
                ] {
                    if ui
                        .selectable_label(font == current_font, font.display_name())
                        .clicked()
                        && font != current_font
                    {
                        ctx.store.settings.font_family = font.as_str().to_string();
                        font_changed = true;
                    }
                }
            });

        if theme_changed {
            ctx.effects.apply_theme = true;
        }
        if font_changed {
            ctx.effects.apply_font = true;
        }
        if theme_changed || font_changed {
            ctx.request_save();
        }

        ui.add_space(6.0);
        ui.label(
            RichText::new("Fonts are bundled locally, so the app keeps its appearance offline.")
                .size(10.0)
                .color(palette.muted),
        );
        ui.add_space(8.0);
        if toggle_row(
            ui,
            palette,
            "Close to system tray",
            "Keep RiftCore running in the tray when you close its window",
            &mut ctx.store.settings.close_to_tray,
        ) {
            ctx.request_save();
        }
        ui.horizontal_wrapped(|ui| {
            if action_button(ui, palette, "QUICK TOUR", false, palette.gold).clicked() {
                ctx.state.show_quick_tour = true;
            }
            if action_button(ui, palette, "WHAT'S NEW", false, palette.accent).clicked() {
                ctx.state.show_whats_new = true;
            }
        });
    });
}

/// What the update panel should offer, derived purely from the snapshot.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct UpdateOffer<'a> {
    pub available: bool,
    pub busy: bool,
    pub auto_install: bool,
    pub release_url: Option<&'a str>,
}

pub(crate) fn update_offer<'a>(
    status: Option<&str>,
    release: Option<&'a Value>,
) -> UpdateOffer<'a> {
    let available = status.is_some_and(|s| s.contains("is available"));
    let busy = status.is_some_and(|s| s.starts_with("Downloading update"));
    let auto_install = release
        .and_then(|r| r.get("assets"))
        .and_then(Value::as_array)
        .is_some_and(|assets| {
            assets
                .iter()
                .any(|a| a.get("name").and_then(Value::as_str) == Some("RiftCore.exe"))
        });
    let release_url = release
        .and_then(|r| r.get("html_url"))
        .and_then(Value::as_str);
    UpdateOffer {
        available,
        busy,
        auto_install,
        release_url,
    }
}

fn update_status_color(status: &str, palette: &ThemePalette) -> Color32 {
    if status.contains("available") {
        palette.gold
    } else if status.starts_with("Could not") || status.contains("error") {
        palette.red
    } else {
        palette.green
    }
}

fn render_updates(ctx: &mut PageContext<'_>, ui: &mut egui::Ui, palette: &ThemePalette) {
    panel(ui, palette, |ui| {
        panel_title(
            ui,
            palette,
            "APP UPDATES",
            &format!("Installed version {}", env!("CARGO_PKG_VERSION")),
        );
        ui.horizontal_wrapped(|ui| {
            if action_button(ui, palette, "CHECK FOR UPDATES", false, palette.accent).clicked() {
                ctx.send_command(CoreCommand::CheckUpdates);
            }
            if let Some(status) = ctx.snapshot.update_status.as_deref() {
                ui.label(
                    RichText::new(status)
                        .size(10.0)
                        .color(update_status_color(status, palette)),
                );
            }
        });

        let offer = update_offer(
            ctx.snapshot.update_status.as_deref(),
            ctx.snapshot.latest_release.as_ref(),
        );
        let mut open_url = None;
        let mut install = false;
        if let Some(url) = offer.release_url {
            if action_button(ui, palette, "VIEW RELEASE PAGE", false, palette.secondary).clicked() {
                open_url = Some(url.to_string());
            }
        }
        if offer.available {
            let label = if offer.busy {
                "DOWNLOADING UPDATE…"
            } else if offer.auto_install {
                "UPDATE & RESTART"
            } else {
                "DOWNLOAD SETUP"
            };
            if action_button(ui, palette, label, offer.busy, palette.accent).clicked()
                && !offer.busy
            {
                if offer.auto_install {
                    install = true;
                } else if let Some(url) = offer.release_url {
                    open_url = Some(url.to_string());
                }
            }
        }
        ui.label(
            RichText::new("Downloads are verified (SHA-256 + Authenticode) before installing.")
                .size(10.0)
                .color(palette.muted),
        );
        if install {
            ctx.send_command(CoreCommand::InstallUpdate);
        }
        if let Some(url) = open_url {
            ctx.open_url(url);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn update_offer_detects_auto_install_asset() {
        let release = json!({
            "html_url": "https://github.com/example/releases/tag/v9",
            "assets": [{"name": "RiftCore.exe"}]
        });
        let offer = update_offer(Some("RiftCore v9 is available"), Some(&release));
        assert!(offer.available && offer.auto_install && !offer.busy);
        assert_eq!(
            offer.release_url,
            Some("https://github.com/example/releases/tag/v9")
        );
    }

    #[test]
    fn update_offer_falls_back_to_setup_without_exe_asset() {
        let release = json!({"assets": [{"name": "RiftCore_Setup.exe"}]});
        let offer = update_offer(Some("Downloading update 40%"), Some(&release));
        assert!(!offer.auto_install && offer.busy && !offer.available);
        assert_eq!(offer.release_url, None);
    }

    #[test]
    fn update_offer_handles_missing_snapshot_data() {
        let offer = update_offer(None, None);
        assert_eq!(
            offer,
            UpdateOffer {
                available: false,
                busy: false,
                auto_install: false,
                release_url: None
            }
        );
    }

    #[test]
    fn regions_are_unique_lowercase_platform_ids() {
        let mut seen = std::collections::HashSet::new();
        for region in RIOT_REGIONS {
            assert_eq!(region, region.to_ascii_lowercase());
            assert!(seen.insert(region));
        }
    }
}
