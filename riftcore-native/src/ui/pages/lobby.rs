//! Lobby Scout view: live team roster, local ranked summary, and Riot ID scouting
//! with working Post-Game Analysis modal support.

use crate::ui::pages::{champ_select::render_team_panel, profile, PageContext};
use crate::ui::widgets::page_heading;
use eframe::egui;

pub fn render(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    let palette = *ctx.palette;

    page_heading(
        ui,
        &palette,
        "LOBBY INTELLIGENCE",
        "Lobby Scout",
        "Inspect live team picks and look up public rank, mastery, and recent match performance by Riot ID.",
        ctx.snapshot,
    );

    render_team_panel(ctx, ui);
    ui.add_space(10.0);
    profile::render_profile_summary(ctx, ui);
    ui.add_space(10.0);
    profile::render_riot_search(ctx, ui);
    ui.add_space(8.0);
    profile::render_riot_lookup(ctx, ui);
}
