//! Application shell chrome: custom titlebar, navigation sidebar, banners, and modals.

use crate::domain::coach::{CoachProvider, OfflineHeuristicCoach};
use crate::domain::mayhem::{filter_cards, CardMode, CardTier};
use crate::domain::protocol::{CoreCommand, CoreNotice, GameflowPhase};
use crate::services::updater::{release_asset_url, RELEASES_FALLBACK_URL};
use crate::ui::pages::{self, Page, PageContext};
use crate::ui::theme::{Spacing, ThemePalette};
use crate::ui::widgets::{
    action_button, avatar, badge, inset_card, item_slot, kda_display, metric_card, notice_banner,
    paint_command_grid, toggle_switch,
};
use eframe::egui::{
    self, Align, Color32, CornerRadius, FontId, Frame, Layout, Margin, RichText, ScrollArea, Sense,
    Stroke, Vec2,
};
use serde_json::Value;

const KOFI_URL: &str = "https://ko-fi.com/X6I629C814";

pub fn render_titlebar(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let egui_ctx = ui.ctx().clone();
    let palette = *ctx.palette;

    egui::Panel::top("riftcore-titlebar")
        .exact_size(Spacing::TITLEBAR_HEIGHT)
        .frame(
            Frame::new()
                .fill(palette.surface)
                .stroke(Stroke::new(1.0, palette.border))
                .inner_margin(Margin::symmetric(12, 0)),
        )
        .show(ui, |ui| {
            let titlebar_rect = ui.max_rect();
            let rail_y = titlebar_rect.top() + 1.0;
            ui.painter().line_segment(
                [
                    egui::pos2(titlebar_rect.left(), rail_y),
                    egui::pos2(titlebar_rect.right(), rail_y),
                ],
                Stroke::new(1.0, ThemePalette::with_alpha(palette.accent, 72)),
            );
            ui.painter().line_segment(
                [
                    egui::pos2(titlebar_rect.left(), rail_y),
                    egui::pos2(
                        (titlebar_rect.left() + 196.0).min(titlebar_rect.right()),
                        rail_y,
                    ),
                ],
                Stroke::new(2.0, palette.accent),
            );
            if titlebar_rect.width() > 256.0 {
                ui.painter().line_segment(
                    [
                        egui::pos2(titlebar_rect.left() + 196.0, rail_y),
                        egui::pos2(titlebar_rect.left() + 252.0, rail_y),
                    ],
                    Stroke::new(2.0, palette.violet),
                );
            }

            let drag_response = ui.interact(
                ui.max_rect().shrink(1.0),
                egui::Id::unique("riftcore-titlebar-drag-area"),
                Sense::click_and_drag(),
            );
            if drag_response.drag_started() {
                egui_ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            } else if drag_response.double_clicked() {
                let maximized = egui_ctx.input(|input| input.viewport().maximized.unwrap_or(false));
                egui_ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
            }

            let full_size = Vec2::new(ui.available_width(), ui.available_height());
            ui.allocate_ui_with_layout(full_size, Layout::left_to_right(Align::Center), |ui| {
                if let Some(logo_tex) = ctx.textures.logo(&egui_ctx) {
                    let logo = egui::Image::new((logo_tex.id(), Vec2::splat(24.0)))
                        .corner_radius(CornerRadius::same(6))
                        .sense(Sense::click_and_drag());
                    if ui.add(logo).drag_started() {
                        egui_ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                    ui.add_space(7.0);
                }

                ui.vertical(|ui| {
                    let brand = ui.add(
                        egui::Label::new(
                            RichText::new("RIFTCORE")
                                .size(11.5)
                                .strong()
                                .color(palette.text),
                        )
                        .sense(Sense::click_and_drag()),
                    );
                    if brand.drag_started() {
                        egui_ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                    ui.label(
                        RichText::new("COMPANION")
                            .size(7.0)
                            .strong()
                            .color(palette.muted),
                    );
                });
                ui.add_space(8.0);
                titlebar_badge(ui, "NATIVE", palette.accent);
                ui.add_space(8.0);

                let (dot, status) = if ctx.state.parsed_live_game.is_some()
                    || ctx.snapshot.phase == GameflowPhase::InProgress
                {
                    (palette.green, "Live Match".to_string())
                } else if ctx.snapshot.connected {
                    (
                        palette.green,
                        format!("League Connected · {}", ctx.snapshot.phase.display_label()),
                    )
                } else if ctx.snapshot.riot_client_open {
                    (palette.gold, "Riot Client Ready".to_string())
                } else {
                    (palette.red, "Client Offline".to_string())
                };
                titlebar_status(ui, &palette, &status, dot);

                if !ctx.snapshot.connected && ctx.snapshot.riot_client_open {
                    ui.add_space(6.0);
                    if titlebar_action_button(ui, &palette, "LAUNCH", false, palette.accent)
                        .clicked()
                    {
                        ctx.send_command(CoreCommand::LaunchLeague);
                    }
                }

                // Right-aligned controls fill the exact remaining titlebar width without magic spacers.
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if window_button(ui, &palette, "×", palette.red).clicked() {
                        egui_ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    let maximized =
                        egui_ctx.input(|input| input.viewport().maximized.unwrap_or(false));
                    if window_button(
                        ui,
                        &palette,
                        if maximized { "❐" } else { "□" },
                        palette.secondary,
                    )
                    .clicked()
                    {
                        egui_ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
                    }
                    if window_button(ui, &palette, "—", palette.secondary).clicked() {
                        egui_ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }

                    let overlay_label = if ctx.state.overlay_mode {
                        if ctx.state.overlay_waiting_for_window {
                            "WAITING"
                        } else {
                            "HUD ON"
                        }
                    } else {
                        "HUD"
                    };
                    if titlebar_action_button(
                        ui,
                        &palette,
                        overlay_label,
                        ctx.state.overlay_mode,
                        palette.accent,
                    )
                    .clicked()
                    {
                        ctx.effects.set_overlay = Some((!ctx.state.overlay_mode, false));
                    }

                    if titlebar_action_button(
                        ui,
                        &palette,
                        if ctx.state.pinned { "PINNED" } else { "PIN" },
                        ctx.state.pinned,
                        palette.gold,
                    )
                    .clicked()
                    {
                        ctx.state.pinned = !ctx.state.pinned;
                        ctx.store.settings.always_on_top = ctx.state.pinned;
                        ctx.effects.apply_always_on_top = true;
                        ctx.request_save();
                    }
                });
            });
        });
}

pub fn render_sidebar(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;

    egui::Panel::left("riftcore-navigation")
        .exact_size(Spacing::SIDEBAR_WIDTH)
        .resizable(false)
        .frame(
            Frame::new()
                .fill(palette.surface)
                .inner_margin(Margin::same(14))
                .stroke(Stroke::new(1.0, palette.border)),
        )
        .show(ui, |ui| {
            ScrollArea::vertical()
                .id_salt("nav-scroll-v4")
                .show(ui, |ui| {
                    nav_section(ui, &palette, "MATCH & LIVE");
                    nav_button(
                        ctx,
                        ui,
                        Page::ChampSelect,
                        "CS",
                        "Champ Select",
                        "LIVE",
                        palette.accent,
                    );
                    nav_button(
                        ctx,
                        ui,
                        Page::Draft,
                        "DR",
                        "Draft Coach",
                        "SIM",
                        palette.red,
                    );
                    nav_button(
                        ctx,
                        ui,
                        Page::Lobby,
                        "LB",
                        "Lobby Scout",
                        "LIVE",
                        palette.green,
                    );
                    nav_button(ctx, ui, Page::Hud, "HD", "In-Game HUD", "", palette.accent);

                    nav_section(ui, &palette, "GAME MODES & META");
                    nav_button(
                        ctx,
                        ui,
                        Page::Tft,
                        "TF",
                        "TFT Companion",
                        "SET 18",
                        palette.gold,
                    );
                    nav_button(
                        ctx,
                        ui,
                        Page::Modes,
                        "MD",
                        "Arena & Modes",
                        "MODES",
                        palette.gold,
                    );
                    nav_button(
                        ctx,
                        ui,
                        Page::TierList,
                        "BD",
                        "Builds & Roster",
                        "",
                        palette.accent,
                    );
                    nav_button(
                        ctx,
                        ui,
                        Page::ChampionPool,
                        "PL",
                        "Champion Pool",
                        "",
                        palette.accent,
                    );
                    nav_button(
                        ctx,
                        ui,
                        Page::Profile,
                        "PR",
                        "Profile & Analysis",
                        "",
                        palette.accent,
                    );

                    nav_section(ui, &palette, "CONFIGURATION");
                    nav_button(
                        ctx,
                        ui,
                        Page::Settings,
                        "CF",
                        "Preferences",
                        "",
                        palette.accent,
                    );

                    ui.add_space(6.0);
                    if action_button(ui, &palette, "QUICK TOUR", false, palette.gold).clicked() {
                        ctx.state.show_quick_tour = true;
                    }
                    if action_button(ui, &palette, "WHAT'S NEW", false, palette.accent).clicked() {
                        ctx.state.show_whats_new = true;
                    }

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("QUICK AUTOMATION")
                            .size(10.0)
                            .strong()
                            .color(palette.muted),
                    );
                    ui.add_space(4.0);

                    if sidebar_toggle_row(
                        ui,
                        &palette,
                        "Auto Accept Queue",
                        &mut ctx.store.settings.auto_accept,
                    ) {
                        ctx.send_command(CoreCommand::SetAutoAccept(
                            ctx.store.settings.auto_accept,
                        ));
                        ctx.request_save();
                    }
                    if sidebar_toggle_row(
                        ui,
                        &palette,
                        "Auto Import Runes",
                        &mut ctx.store.settings.auto_import_runes,
                    ) {
                        ctx.request_save();
                    }
                    if sidebar_toggle_row(
                        ui,
                        &palette,
                        "Audio Alerts",
                        &mut ctx.store.settings.sound_alerts,
                    ) {
                        ctx.request_save();
                    }

                    ui.add_space(8.0);
                    if ui
                        .add_sized(
                            [ui.available_width(), 34.0],
                            egui::Button::new(
                                RichText::new("Fuel on Ko-fi")
                                    .size(12.0)
                                    .strong()
                                    .color(palette.text),
                            )
                            .fill(palette.card)
                            .stroke(Stroke::new(1.0, palette.accent_dim)),
                        )
                        .clicked()
                    {
                        ctx.open_url(KOFI_URL);
                    }
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!("VERSION  {}", env!("CARGO_PKG_VERSION")))
                            .size(10.0)
                            .color(palette.muted),
                    );
                });
        });
}

pub fn render_main(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;

    egui::CentralPanel::default()
        .frame(
            Frame::new()
                .fill(palette.bg)
                .inner_margin(Margin::same(Spacing::PAGE_MARGIN)),
        )
        .show(ui, |ui| {
            paint_command_grid(ui, &palette);
            render_banners(ctx, ui);

            if ctx.state.active_page == Page::ChampSelect {
                pages::render_active_page(ctx, ui);
            } else {
                ScrollArea::vertical()
                    .id_salt(("main-scroll-v4", ctx.state.active_page))
                    .auto_shrink([false, false])
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .show(ui, |ui| {
                        pages::render_active_page(ctx, ui);
                    });
            }
        });
}

fn render_banners(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;

    if let Some(notice) = ctx.state.active_notice().cloned() {
        notice_banner(ui, &palette, &notice);
        ui.add_space(8.0);
    }

    let update_status = ctx.snapshot.update_status.as_deref().unwrap_or("");
    let update_available = update_status.contains("is available");
    let verification_failed = update_status.contains("SHA-256")
        || update_status.contains("Authenticode")
        || update_status.contains("SHA256SUMS")
        || update_status.contains("verification failed");

    if !ctx.state.update_banner_dismissed && (update_available || verification_failed) {
        let latest = ctx.snapshot.latest_release.as_ref();
        let version = latest
            .and_then(|r| r.get("tag_name"))
            .and_then(Value::as_str)
            .unwrap_or("New version");
        let release_url = latest
            .and_then(|r| r.get("html_url"))
            .and_then(Value::as_str)
            .unwrap_or(RELEASES_FALLBACK_URL)
            .to_string();
        let has_verified_assets = latest.is_some_and(|r| {
            release_asset_url(r, "RiftCore.exe").is_some()
                && release_asset_url(r, "SHA256SUMS.txt").is_some()
        });
        let update_busy = update_status.starts_with("Downloading");

        let border_color = if verification_failed {
            palette.red
        } else {
            palette.gold
        };
        Frame::new()
            .fill(palette.surface)
            .stroke(Stroke::new(1.0, border_color))
            .corner_radius(CornerRadius::same(Spacing::RADIUS_LG))
            .inner_margin(Margin::symmetric(13, 9))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    badge(
                        ui,
                        if verification_failed {
                            "MANUAL DOWNLOAD"
                        } else {
                            "UPDATE READY"
                        },
                        border_color,
                    );
                    ui.label(
                        RichText::new(if verification_failed {
                            update_status.to_string()
                        } else {
                            format!("RiftCore {version} is available")
                        })
                        .size(11.5)
                        .color(palette.text),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if action_button(ui, &palette, "×", false, palette.secondary).clicked() {
                            ctx.state.update_banner_dismissed = true;
                        }
                        if verification_failed || !has_verified_assets {
                            if action_button(ui, &palette, "OPEN IN BROWSER", false, palette.gold)
                                .clicked()
                            {
                                ctx.open_url(release_url.clone());
                            }
                        } else if action_button(
                            ui,
                            &palette,
                            if update_busy {
                                "VERIFYING…"
                            } else {
                                "UPDATE & RESTART"
                            },
                            update_busy,
                            palette.accent,
                        )
                        .clicked()
                            && !update_busy
                        {
                            ctx.send_command(CoreCommand::InstallUpdate);
                        }
                    });
                });
            });
        ui.add_space(8.0);
    }

    let waiting_for_accept = ctx.snapshot.ready_check.as_ref().is_some_and(|check| {
        check.get("state").and_then(Value::as_str) == Some("InProgress")
            && check.get("playerResponse").and_then(Value::as_str) == Some("None")
    });
    if waiting_for_accept {
        Frame::new()
            .fill(ThemePalette::with_alpha(palette.green, 24))
            .stroke(Stroke::new(1.0, palette.green))
            .corner_radius(CornerRadius::same(Spacing::RADIUS_MD))
            .inner_margin(Margin::same(10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("MATCH FOUND")
                                .size(13.5)
                                .strong()
                                .color(palette.text),
                        );
                        ui.label(
                            RichText::new(if ctx.store.settings.auto_accept {
                                "Auto-accept is enabled"
                            } else {
                                "Ready check is waiting for your response"
                            })
                            .size(10.5)
                            .color(palette.secondary),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if action_button(ui, &palette, "ACCEPT MATCH", false, palette.green)
                            .clicked()
                        {
                            ctx.send_command(CoreCommand::AcceptMatch);
                            ctx.set_notice(CoreNotice::Info("Accepting the match…".to_string()));
                        }
                    });
                });
            });
        ui.add_space(8.0);
    }
}

pub fn render_dialogs(ctx: &mut PageContext<'_>, egui_ctx: &egui::Context) {
    let palette = *ctx.palette;
    let viewport = egui_ctx.content_rect();
    let max_dialog_h = (viewport.height() - 100.0).clamp(360.0, 680.0);
    let max_dialog_w = (viewport.width() - 60.0).clamp(420.0, 920.0);

    if ctx.state.show_whats_new {
        let mut dismissed = false;
        egui::Modal::new(egui::Id::unique("riftcore-whats-new-modal"))
            .frame(
                Frame::new()
                    .fill(palette.surface)
                    .stroke(Stroke::new(1.0, palette.border))
                    .corner_radius(CornerRadius::same(Spacing::RADIUS_MODAL))
                    .inner_margin(Margin::same(20)),
            )
            .show(egui_ctx, |ui| {
                ui.set_min_width(480.0f32.min(max_dialog_w));
                ui.set_max_width(580.0f32.min(max_dialog_w));
                ui.label(
                    RichText::new(format!("RIFTCORE {}", env!("CARGO_PKG_VERSION")))
                        .size(10.0)
                        .strong()
                        .color(palette.accent),
                );
                ui.heading(RichText::new("What's new").size(23.0).strong().color(palette.text));
                ui.label(
                    RichText::new("Native Rust companion architecture, compliance, and UI upgrades.")
                        .size(11.5)
                        .color(palette.secondary),
                );
                ui.add_space(10.0);

                ScrollArea::vertical()
                    .id_salt("whats-new-scroll")
                    .max_height((max_dialog_h - 120.0).max(200.0))
                    .show(ui, |ui| {
                        for (title, detail) in [
                            ("Full runtime themes", "Hextech, Ember, and Void now dynamically re-theme panels, borders, accents, and grid lines."),
                            ("Pre-game & post-game coaching", "Draft Coach evaluates ally/enemy damage splits & counter tags; Post-Game Analysis provides telemetry-grounded takeaways."),
                            ("DPAPI secret protection", "Your Riot API key is encrypted per-user with Windows DPAPI and never stored in plaintext."),
                            ("Accurate champion & item builds", "Corrected 21 champion role assignments, Udyr Q>R>E progression, Jungle/Support presets, and Grisly Mementos (8140)."),
                            ("Lazy texture loading", "Embedded portraits, runes, and TFT icons load smoothly on demand without stalling startup."),
                            ("Verified auto-updates", "Release updates enforce HTTPS allowlists, SHA-256 checksums, and Authenticode signatures with browser fallback."),
                        ] {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("◆").size(12.0).color(palette.accent));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(title).size(12.0).strong().color(palette.text));
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(detail).size(11.5).color(palette.secondary),
                                        )
                                        .wrap(),
                                    );
                                });
                            });
                            ui.add_space(6.0);
                        }
                    });
                ui.add_space(10.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if action_button(ui, &palette, "CONTINUE", false, palette.accent).clicked() {
                        dismissed = true;
                    }
                });
            });
        if dismissed {
            ctx.state.show_whats_new = false;
            if !ctx.store.settings.first_run_tour_seen {
                ctx.state.show_quick_tour = true;
            }
        }
    }

    if ctx.state.show_quick_tour {
        let mut close = false;
        let mut open_champ_select = false;
        egui::Modal::new(egui::Id::unique("riftcore-quick-tour-modal"))
            .frame(
                Frame::new()
                    .fill(palette.surface)
                    .stroke(Stroke::new(1.0, palette.border))
                    .corner_radius(CornerRadius::same(Spacing::RADIUS_MODAL))
                    .inner_margin(Margin::same(20)),
            )
            .show(egui_ctx, |ui| {
                ui.set_min_width(480.0f32.min(max_dialog_w));
                ui.set_max_width(640.0f32.min(max_dialog_w));
                ui.label(
                    RichText::new("QUICK TOUR · 2 MINUTES")
                        .size(10.0)
                        .strong()
                        .color(palette.gold),
                );
                ui.heading(
                    RichText::new("Your companion, at a glance")
                        .size(22.0)
                        .strong()
                        .color(palette.text),
                );
                ui.add_space(8.0);
                ScrollArea::vertical()
                    .id_salt("quick-tour-scroll")
                    .max_height((max_dialog_h - 130.0).max(200.0))
                    .show(ui, |ui| {
                        for (step, title, detail) in [
                            ("01", "Prepare your pick", "Champ Select builds, rune editor, summoner spells, shop item sets, and client automation."),
                            ("02", "Plan the draft", "Draft Coach compares 5v5 damage/CC/frontline profiles; Champion Pool tracks blind, counter, and pocket picks."),
                            ("03", "Use the live HUD", "Configure Integrated, Top Bar, Tactical Card, or Minimap Pill layouts and objective/ally/ARAM timers."),
                            ("04", "Review performance", "Scout Riot IDs, view multi-game trends, and analyze matches with grounded post-game coaching."),
                        ] {
                            ui.horizontal(|ui| {
                                badge(ui, step, palette.accent);
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(title).size(12.0).strong().color(palette.text));
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(detail).size(11.5).color(palette.secondary),
                                        )
                                        .wrap(),
                                    );
                                });
                            });
                            ui.add_space(8.0);
                        }
                        inset_card(ui, &palette, palette.border, |ui| {
                            ui.label(RichText::new("GLOBAL SHORTCUTS").size(11.5).strong().color(palette.accent));
                            ui.label(
                                RichText::new("Alt + ` or Alt + Shift + Space: Toggle HUD   ·   Alt + Shift + `: Toggle click-through")
                                    .size(11.0)
                                    .monospace()
                                    .color(palette.text),
                            );
                        });
                    });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if action_button(ui, &palette, "OPEN CHAMP SELECT", false, palette.accent)
                        .clicked()
                    {
                        open_champ_select = true;
                        close = true;
                    }
                    if action_button(ui, &palette, "DONE", false, palette.secondary).clicked() {
                        close = true;
                    }
                });
            });
        if close {
            ctx.state.show_quick_tour = false;
            ctx.store.settings.first_run_tour_seen = true;
            ctx.request_save();
            if open_champ_select {
                ctx.state.active_page = Page::ChampSelect;
            }
        }
    }

    if ctx.state.show_mayhem_cards {
        let mut close = false;
        egui::Modal::new(egui::Id::unique("riftcore-mayhem-guide-modal"))
            .frame(
                Frame::new()
                    .fill(palette.surface)
                    .stroke(Stroke::new(1.0, palette.border))
                    .corner_radius(CornerRadius::same(Spacing::RADIUS_MODAL))
                    .inner_margin(Margin::same(18)),
            )
            .show(egui_ctx, |ui| {
                ui.set_min_width(640.0f32.min(max_dialog_w));
                ui.set_max_width(880.0f32.min(max_dialog_w));
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("MODE REFERENCE")
                                .size(10.0)
                                .strong()
                                .color(palette.gold),
                        );
                        ui.heading(
                            RichText::new("Mayhem & Arena Augments")
                                .size(21.0)
                                .strong()
                                .color(palette.text),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if action_button(ui, &palette, "CLOSE", false, palette.secondary).clicked() {
                            close = true;
                        }
                    });
                });
                ui.add_space(8.0);
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
                crate::ui::widgets::search_box(
                    ui,
                    &palette,
                    "mayhem-modal-search",
                    &mut ctx.state.modes.mayhem_search,
                    "Search augment name, archetype, or effect…",
                    Some(ui.available_width()),
                );
                let filtered = filter_cards(
                    Some(ctx.state.modes.mayhem_mode),
                    ctx.state.modes.mayhem_tier,
                    &ctx.state.modes.mayhem_search,
                );
                ui.label(
                    RichText::new(format!("{} cards", filtered.len()))
                        .size(10.0)
                        .color(palette.muted),
                );
                ScrollArea::vertical()
                    .id_salt("mayhem-modal-scroll")
                    .max_height((max_dialog_h - 190.0).max(200.0))
                    .show(ui, |ui| {
                        if filtered.is_empty() {
                            ui.label(
                                RichText::new("No cards match those filters.")
                                    .size(11.0)
                                    .color(palette.secondary),
                            );
                        }
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
                                    egui::Label::new(
                                        RichText::new(&card.tip).size(10.0).color(palette.gold),
                                    )
                                    .wrap(),
                                );
                            });
                            ui.add_space(6.0);
                        }
                    });
                ui.label(
                    RichText::new("In-game augment availability and values vary by patch; always check in-game tooltips.")
                        .size(9.5)
                        .color(palette.muted),
                );
            });
        if close {
            ctx.state.show_mayhem_cards = false;
        }
    }

    if let Some(summary) = ctx.state.profile.selected_analysis_match.clone() {
        let mut close = false;
        let outcome_color = if summary.win {
            palette.green
        } else {
            palette.red
        };
        let coach = OfflineHeuristicCoach;
        let advice = coach.post_game_advice(&summary);

        egui::Modal::new(egui::Id::unique("riftcore-postgame-analysis-modal"))
            .frame(
                Frame::new()
                    .fill(palette.surface)
                    .stroke(Stroke::new(1.2, outcome_color))
                    .corner_radius(CornerRadius::same(Spacing::RADIUS_MODAL))
                    .inner_margin(Margin::same(20)),
            )
            .show(egui_ctx, |ui| {
                ui.set_min_width(540.0f32.min(max_dialog_w));
                ui.set_max_width(720.0f32.min(max_dialog_w));
                ui.horizontal(|ui| {
                    let champ_key = crate::domain::champions::champion_by_id(summary.champion_id)
                        .map(|c| c.key.as_str())
                        .unwrap_or(&summary.champion_name);
                    let portrait = ctx.textures.champion(egui_ctx, champ_key);
                    avatar(
                        ui,
                        &palette,
                        &summary.champion_name,
                        48.0,
                        outcome_color,
                        portrait,
                    );
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            badge(
                                ui,
                                if summary.win { "VICTORY" } else { "DEFEAT" },
                                outcome_color,
                            );
                            ui.label(
                                RichText::new(&summary.champion_name)
                                    .size(18.0)
                                    .strong()
                                    .color(palette.text),
                            );
                            if !summary.role.is_empty() && summary.role != "UNKNOWN" {
                                badge(ui, &summary.role, palette.accent);
                            }
                        });
                        ui.label(
                            RichText::new(format!(
                                "{} · {}:{:02}",
                                summary.game_mode,
                                summary.duration_secs / 60,
                                summary.duration_secs % 60
                            ))
                            .size(11.0)
                            .color(palette.secondary),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if action_button(ui, &palette, "CLOSE", false, palette.secondary).clicked()
                        {
                            close = true;
                        }
                    });
                });

                ui.add_space(10.0);
                ScrollArea::vertical()
                    .id_salt("post-game-analysis-scroll")
                    .max_height((max_dialog_h - 140.0).max(200.0))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            metric_card(
                                ui,
                                &palette,
                                "KDA RATIO",
                                &format!("{:.2}", summary.kda),
                                palette.accent,
                            );
                            metric_card(
                                ui,
                                &palette,
                                "CS / MIN",
                                &format!("{:.1}", summary.cs_per_min),
                                palette.gold,
                            );
                            metric_card(
                                ui,
                                &palette,
                                "KILL PART.",
                                &format!("{:.0}%", summary.kp_pct),
                                palette.green,
                            );
                            metric_card(
                                ui,
                                &palette,
                                "DMG SHARE",
                                &format!("{:.1}% ({:.0}/m)", summary.dmg_pct, summary.dpm),
                                palette.red,
                            );
                            metric_card(
                                ui,
                                &palette,
                                "GOLD / MIN",
                                &format!("{:.0} ({:.1}%)", summary.gpm, summary.gold_pct),
                                palette.gold,
                            );
                            metric_card(
                                ui,
                                &palette,
                                "VISION / MIN",
                                &format!("{:.2} ({} CW)", summary.vpm, summary.control_wards),
                                palette.violet,
                            );
                        });

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("SCORE LINE")
                                    .size(10.0)
                                    .strong()
                                    .color(palette.muted),
                            );
                            kda_display(
                                ui,
                                &palette,
                                summary.kills,
                                summary.deaths,
                                summary.assists,
                                Some(summary.kda),
                            );
                        });

                        if !summary.items.is_empty() {
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new("FINAL INVENTORY")
                                    .size(9.5)
                                    .strong()
                                    .color(palette.muted),
                            );
                            ui.horizontal_wrapped(|ui| {
                                for &item_id in summary.items.iter().filter(|&&id| id > 0) {
                                    item_slot(ui, &palette, ctx.textures, item_id, 30.0, true);
                                }
                            });
                        }

                        ui.add_space(8.0);
                        inset_card(ui, &palette, palette.accent, |ui| {
                            ui.label(
                                RichText::new("POST-GAME COACHING NOTES")
                                    .size(10.5)
                                    .strong()
                                    .color(palette.accent),
                            );
                            ui.add_space(4.0);
                            for line in &advice {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("•").size(12.0).color(palette.accent));
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(line).size(11.0).color(palette.text),
                                        )
                                        .wrap(),
                                    );
                                });
                            }
                        });
                    });
            });
        if close {
            ctx.state.profile.selected_analysis_match = None;
        }
    }

    if ctx.state.show_close_confirmation {
        let mut cancel = false;
        let mut hide = false;
        let mut exit = false;
        egui::Modal::new(egui::Id::unique("riftcore-close-confirmation-modal"))
            .frame(
                Frame::new()
                    .fill(palette.surface)
                    .stroke(Stroke::new(1.0, palette.border))
                    .corner_radius(CornerRadius::same(Spacing::RADIUS_MODAL))
                    .inner_margin(Margin::same(20)),
            )
            .show(egui_ctx, |ui| {
                ui.set_min_width(400.0);
                ui.heading(
                    RichText::new("Keep RiftCore nearby?")
                        .size(20.0)
                        .strong()
                        .color(palette.text),
                );
                ui.add(
                    egui::Label::new(
                        RichText::new("Minimizing to the tray keeps League monitoring, hotkeys, and the HUD available in the background.")
                            .size(11.5)
                            .color(palette.secondary),
                    )
                    .wrap(),
                );
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    let tray_label = if ctx.state.tray_available {
                        "MINIMIZE TO TRAY"
                    } else {
                        "MINIMIZE TO TASKBAR"
                    };
                    if action_button(ui, &palette, tray_label, false, palette.accent).clicked() {
                        hide = true;
                    }
                    if action_button(ui, &palette, "EXIT", false, palette.red).clicked() {
                        exit = true;
                    }
                    if action_button(ui, &palette, "CANCEL", false, palette.secondary).clicked() {
                        cancel = true;
                    }
                });
            });
        if hide {
            ctx.state.show_close_confirmation = false;
            ctx.store.settings.close_confirmation_seen = true;
            ctx.store.settings.close_to_tray = ctx.state.tray_available;
            ctx.request_save();
            if ctx.state.tray_available {
                egui_ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            } else {
                egui_ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
        } else if exit {
            ctx.state.show_close_confirmation = false;
            ctx.store.settings.close_confirmation_seen = true;
            ctx.store.settings.close_to_tray = false;
            ctx.request_save();
            ctx.state.force_exit = true;
            egui_ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if cancel {
            ctx.state.show_close_confirmation = false;
        }
    }
}

fn nav_button(
    ctx: &mut PageContext<'_>,
    ui: &mut egui::Ui,
    page: Page,
    glyph: &str,
    label: &str,
    badge_text: &str,
    accent: Color32,
) {
    let palette = *ctx.palette;
    let selected = ctx.state.active_page == page;
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 40.0), Sense::click());
    let fill = if selected {
        ThemePalette::with_alpha(accent, 28)
    } else if response.hovered() {
        palette.raised
    } else {
        Color32::TRANSPARENT
    };
    ui.painter()
        .rect_filled(rect, CornerRadius::same(Spacing::RADIUS_SM), fill);
    if selected || response.hovered() {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(Spacing::RADIUS_SM),
            Stroke::new(
                if selected { 1.0 } else { 0.7 },
                if selected {
                    ThemePalette::with_alpha(accent, 130)
                } else {
                    palette.border
                },
            ),
            egui::StrokeKind::Inside,
        );
    }
    if selected {
        ui.painter().rect_filled(
            egui::Rect::from_min_max(
                rect.min + Vec2::new(1.0, 7.0),
                rect.min + Vec2::new(4.0, rect.height() - 7.0),
            ),
            CornerRadius::same(1),
            accent,
        );
    }
    let foreground = if response.hovered() || selected {
        palette.text
    } else {
        palette.secondary
    };
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 20.0, rect.center().y),
        Vec2::new(26.0, 26.0),
    );
    ui.painter().rect_filled(
        icon_rect,
        CornerRadius::same(6),
        ThemePalette::with_alpha(accent, if selected { 42 } else { 18 }),
    );
    ui.painter().rect_stroke(
        icon_rect,
        CornerRadius::same(6),
        Stroke::new(
            0.8,
            ThemePalette::with_alpha(accent, if selected { 135 } else { 52 }),
        ),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        icon_rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        FontId::monospace(10.0),
        if selected { accent } else { palette.secondary },
    );
    ui.painter().text(
        egui::pos2(rect.left() + 40.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.5),
        foreground,
    );
    if !badge_text.is_empty() {
        let width = badge_text.chars().count() as f32 * 6.2 + 12.0;
        let badge_rect = egui::Rect::from_center_size(
            egui::pos2(rect.right() - width * 0.5 - 8.0, rect.center().y),
            Vec2::new(width, 19.0),
        );
        ui.painter().rect_filled(
            badge_rect,
            CornerRadius::same(5),
            ThemePalette::with_alpha(accent, 22),
        );
        ui.painter().rect_stroke(
            badge_rect,
            CornerRadius::same(5),
            Stroke::new(0.9, ThemePalette::with_alpha(accent, 85)),
            egui::StrokeKind::Inside,
        );
        ui.painter().text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            badge_text,
            FontId::monospace(8.5),
            accent,
        );
    }
    if response.clicked() {
        ctx.state.active_page = page;
        if page == Page::Hud && ctx.state.parsed_live_game.is_some() {
            ctx.set_notice(CoreNotice::Info("HUD view selected".to_string()));
        }
    }
}

fn nav_section(ui: &mut egui::Ui, palette: &ThemePalette, label: &str) {
    ui.add_space(11.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("// {label}"))
                .size(8.5)
                .strong()
                .monospace()
                .color(palette.accent),
        );
        let (line, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
        ui.painter().line_segment(
            [line.left_center(), line.right_center()],
            Stroke::new(1.0, palette.border),
        );
    });
    ui.add_space(5.0);
}

fn sidebar_toggle_row(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    label: &str,
    value: &mut bool,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(11.5).color(palette.secondary));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if toggle_switch(ui, palette, value).changed() {
                changed = true;
            }
        });
    });
    ui.add_space(3.0);
    changed
}

fn titlebar_badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(46.0, 18.0), Sense::hover());
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(6),
        ThemePalette::with_alpha(color, 18),
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(6),
        Stroke::new(0.8, ThemePalette::with_alpha(color, 110)),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        FontId::monospace(8.0),
        color,
    );
}

fn titlebar_status(ui: &mut egui::Ui, palette: &ThemePalette, text: &str, color: Color32) {
    let width = (25.0 + text.chars().count() as f32 * 5.2).clamp(90.0, 260.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 21.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(7), palette.card);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(7),
        Stroke::new(1.0, palette.border),
        egui::StrokeKind::Inside,
    );
    ui.painter()
        .circle_filled(egui::pos2(rect.left() + 9.0, rect.center().y), 3.0, color);
    ui.painter().text(
        egui::pos2(rect.left() + 16.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        text,
        FontId::proportional(9.5),
        palette.secondary,
    );
}

fn titlebar_action_button(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    text: &str,
    active: bool,
    accent: Color32,
) -> egui::Response {
    let width = (text.chars().count() as f32 * 6.0 + 16.0).max(40.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 24.0), Sense::click());
    let fill = if active {
        ThemePalette::with_alpha(accent, 38)
    } else if response.hovered() {
        palette.raised
    } else {
        palette.card
    };
    ui.painter().rect_filled(rect, CornerRadius::same(7), fill);
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(7),
        Stroke::new(
            if active { 1.0 } else { 0.7 },
            if active { accent } else { palette.border },
        ),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        FontId::proportional(9.0),
        if active { accent } else { palette.text },
    );
    response
}

fn window_button(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    text: &str,
    color: Color32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(
            rect,
            CornerRadius::same(5),
            if color == palette.red {
                ThemePalette::with_alpha(palette.red, 58)
            } else {
                palette.raised
            },
        );
    }
    let glyph_color = if color == palette.red && !response.hovered() {
        palette.muted
    } else {
        color
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        FontId::proportional(11.0),
        glyph_color,
    );
    response
}
