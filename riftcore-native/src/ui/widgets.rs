//! Shared, theme-aware egui UI primitives and domain widgets.

use crate::domain::analytics::NormalizedMatchSummary;
use crate::domain::champions::{champion_by_id, Role};
use crate::domain::items;
use crate::domain::protocol::CoreNotice;
use crate::domain::protocol::CoreSnapshot;
use crate::domain::runes;
use crate::ui::textures::TextureManager;
use crate::ui::theme::{lerp_color, role_color, rune_tree_color, Spacing, ThemePalette};
use eframe::egui::{
    self, Align, Color32, CornerRadius, FontId, Frame, Layout, Margin, RichText, Sense, Stroke,
    TextureHandle, Vec2,
};

/// Result returned by [`search_box`].
#[derive(Clone, Copy, Debug, Default)]
pub struct SearchBoxOutput {
    pub changed: bool,
    pub submitted: bool,
}

/// Paints the subtle tactical grid & radar arcs behind the main workspace.
pub fn paint_command_grid(ui: &egui::Ui, palette: &ThemePalette) {
    let rect = ui.max_rect();
    let painter = ui.painter_at(rect);
    let step = 42.0;

    let columns = (rect.width() / step).ceil() as usize;
    for column in 0..=columns {
        let x = rect.left() + column as f32 * step;
        let color = if column % 4 == 0 {
            palette.grid_major
        } else {
            palette.grid_minor
        };
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            Stroke::new(1.0, color),
        );
    }

    let rows = (rect.height() / step).ceil() as usize;
    for row in 0..=rows {
        let y = rect.top() + row as f32 * step;
        let color = if row % 4 == 0 {
            palette.grid_major
        } else {
            palette.grid_minor
        };
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            Stroke::new(1.0, color),
        );
    }

    let radar_center = egui::pos2(rect.right() - 52.0, rect.top() + 112.0);
    for radius in [34.0, 48.0, 62.0] {
        painter.circle_stroke(
            radar_center,
            radius,
            Stroke::new(1.0, ThemePalette::with_alpha(palette.accent, 16)),
        );
    }
}

/// Renders the decorative tactical radar core icon in page headers.
pub fn render_tactical_core(ui: &mut egui::Ui, signal: Color32, palette: &ThemePalette) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(56.0), Sense::hover());
    let painter = ui.painter();
    let center = rect.center();
    let tint = |alpha| ThemePalette::with_alpha(signal, alpha);

    painter.circle_filled(center, 25.0, tint(14));
    painter.circle_stroke(center, 26.0, Stroke::new(1.0, tint(75)));
    painter.circle_stroke(center, 19.0, Stroke::new(1.0, tint(120)));
    painter.circle_stroke(center, 11.5, Stroke::new(1.0, tint(170)));
    painter.line_segment(
        [
            egui::pos2(rect.left() + 2.0, center.y),
            egui::pos2(rect.right() - 2.0, center.y),
        ],
        Stroke::new(1.0, tint(65)),
    );
    painter.line_segment(
        [
            egui::pos2(center.x, rect.top() + 2.0),
            egui::pos2(center.x, rect.bottom() - 2.0),
        ],
        Stroke::new(1.0, tint(65)),
    );
    painter.rect_filled(
        egui::Rect::from_center_size(center, Vec2::splat(14.0)),
        CornerRadius::same(3),
        palette.inset,
    );
    painter.rect_stroke(
        egui::Rect::from_center_size(center, Vec2::splat(14.0)),
        CornerRadius::same(3),
        Stroke::new(1.0, tint(190)),
        egui::StrokeKind::Inside,
    );
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        "RC",
        FontId::monospace(7.5),
        palette.text,
    );

    for (angle, color) in [
        (0.0_f32, signal),
        (90.0, palette.accent),
        (180.0, palette.violet),
        (270.0, signal),
    ] {
        let radians = angle.to_radians();
        let node = center + Vec2::new(radians.cos() * 25.5, radians.sin() * 25.5);
        painter.circle_filled(node, 2.2, color);
    }
}

/// Primary card container with tactical accent top rail.
pub fn panel(ui: &mut egui::Ui, palette: &ThemePalette, add_contents: impl FnOnce(&mut egui::Ui)) {
    let inner_width =
        (ui.available_width() - f32::from(Spacing::PANEL_MARGIN) * 2.0 - 2.0).max(0.0);
    let frame = Frame::new()
        .fill(palette.card)
        .stroke(Stroke::new(1.0, palette.border))
        .corner_radius(CornerRadius::same(Spacing::RADIUS_MD))
        .inner_margin(Margin::same(Spacing::PANEL_MARGIN))
        .show(ui, |ui| {
            ui.set_min_width(inner_width);
            add_contents(ui);
        });
    let rect = frame.response.rect;
    let painter = ui.painter();
    painter.line_segment(
        [
            egui::pos2(rect.left() + 12.0, rect.top() + 1.0),
            egui::pos2((rect.left() + 52.0).min(rect.right()), rect.top() + 1.0),
        ],
        Stroke::new(1.5, palette.accent),
    );
    if rect.width() > 72.0 {
        painter.line_segment(
            [
                egui::pos2(rect.left() + 52.0, rect.top() + 1.0),
                egui::pos2(rect.left() + 66.0, rect.top() + 1.0),
            ],
            Stroke::new(1.5, palette.violet),
        );
    }
}

/// Nested sub-card used inside panels.
pub fn inset_card(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    stroke_color: Color32,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    Frame::new()
        .fill(palette.inset)
        .stroke(Stroke::new(1.0, stroke_color))
        .corner_radius(CornerRadius::same(Spacing::RADIUS_MD))
        .inner_margin(Margin::same(Spacing::CARD_MARGIN))
        .show(ui, add_contents);
}

/// Standard panel header with accent pip and wrapping subtitle.
pub fn panel_title(ui: &mut egui::Ui, palette: &ThemePalette, title: &str, description: &str) {
    ui.horizontal(|ui| {
        let (accent, _) = ui.allocate_exact_size(Vec2::splat(11.0), Sense::hover());
        ui.painter().rect_stroke(
            accent,
            CornerRadius::same(2),
            Stroke::new(1.0, palette.accent),
            egui::StrokeKind::Inside,
        );
        ui.painter()
            .circle_filled(accent.center(), 2.2, palette.accent);
        ui.add_space(2.0);
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(14.5).strong().color(palette.text));
            if !description.is_empty() {
                ui.add(
                    egui::Label::new(
                        RichText::new(description)
                            .size(11.5)
                            .color(palette.secondary),
                    )
                    .wrap(),
                );
            }
        });
    });
    ui.add_space(8.0);
}

/// Responsive page header banner with wrapping left copy and right-anchored status core.
pub fn page_heading(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    eyebrow: &str,
    title: &str,
    description: &str,
    snapshot: &CoreSnapshot,
) {
    let connected = snapshot.connected;
    let riot_client_open = snapshot.riot_client_open;
    let phase_label = snapshot.phase.display_label();
    let (link_label, link_color) = if connected {
        ("LIVE LINK", palette.green)
    } else if riot_client_open {
        ("CLIENT READY", palette.gold)
    } else {
        ("CLIENT OFFLINE", palette.muted)
    };

    let heading = Frame::new()
        .fill(palette.surface)
        .stroke(Stroke::new(1.0, palette.border))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(18, 14))
        .show(ui, |ui| {
            let total_w = ui.available_width();
            let right_reserved = 215.0;
            let left_w = (total_w - right_reserved).max(240.0);
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new(left_w, 0.0),
                    Layout::top_down(Align::LEFT),
                    |ui| {
                        ui.label(
                            RichText::new(format!("RIFTCORE // {}", eyebrow.to_ascii_uppercase()))
                                .size(9.5)
                                .strong()
                                .monospace()
                                .color(palette.accent),
                        );
                        ui.add_space(2.0);
                        ui.label(RichText::new(title).size(26.0).strong().color(palette.text));
                        ui.add(
                            egui::Label::new(
                                RichText::new(description)
                                    .size(12.5)
                                    .color(palette.secondary),
                            )
                            .wrap(),
                        );
                    },
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.vertical(|ui| {
                        badge(ui, link_label, link_color);
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(format!("PHASE // {}", phase_label.to_ascii_uppercase()))
                                .size(8.5)
                                .strong()
                                .monospace()
                                .color(palette.muted),
                        );
                        ui.label(
                            RichText::new("TACTICAL CORE / NATIVE")
                                .size(8.0)
                                .monospace()
                                .color(palette.muted),
                        );
                    });
                    ui.add_space(10.0);
                    render_tactical_core(ui, link_color, palette);
                });
            });
        });

    let rect = heading.response.rect;
    let painter = ui.painter();
    painter.line_segment(
        [
            egui::pos2(rect.left() + 18.0, rect.top() + 1.0),
            egui::pos2((rect.left() + 154.0).min(rect.right()), rect.top() + 1.0),
        ],
        Stroke::new(2.0, palette.accent),
    );
    if rect.width() > 200.0 {
        painter.line_segment(
            [
                egui::pos2(rect.left() + 154.0, rect.top() + 1.0),
                egui::pos2(rect.left() + 194.0, rect.top() + 1.0),
            ],
            Stroke::new(2.0, palette.violet),
        );
    }
    ui.add_space(10.0);
}

/// Compact pill badge with translucent tinted background.
pub fn badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    Frame::new()
        .fill(ThemePalette::with_alpha(color, 30))
        .stroke(Stroke::new(0.8, ThemePalette::with_alpha(color, 95)))
        .corner_radius(CornerRadius::same(5))
        .inner_margin(Margin::symmetric(7, 3))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .size(9.0)
                    .strong()
                    .monospace()
                    .color(color),
            );
        });
}

/// Role pill badge formatted from `domain::champions::Role`.
pub fn role_tag(ui: &mut egui::Ui, role: Role, palette: &ThemePalette) {
    badge(ui, role.short_label(), role_color(role, palette));
}

/// Color-coded K / D / A summary display with optional ratio suffix.
pub fn kda_display(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    kills: u32,
    deaths: u32,
    assists: u32,
    kda_ratio: Option<f64>,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        ui.label(
            RichText::new(kills.to_string())
                .size(11.0)
                .strong()
                .color(palette.green),
        );
        ui.label(RichText::new("/").size(10.5).color(palette.muted));
        ui.label(
            RichText::new(deaths.to_string())
                .size(11.0)
                .strong()
                .color(palette.red),
        );
        ui.label(RichText::new("/").size(10.5).color(palette.muted));
        ui.label(
            RichText::new(assists.to_string())
                .size(11.0)
                .strong()
                .color(palette.accent),
        );
        if let Some(ratio) = kda_ratio {
            let ratio_color = if ratio >= 4.0 {
                palette.gold
            } else if ratio >= 2.5 {
                palette.green
            } else {
                palette.secondary
            };
            ui.label(
                RichText::new(format!("({ratio:.2})"))
                    .size(9.5)
                    .color(ratio_color),
            );
        }
    });
}

/// Smooth progress bar used across Draft Coach, Ranked LP, and Trends.
pub fn progress_bar(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    fraction: f32,
    color: Color32,
    height: f32,
) {
    let width = ui.available_width().max(40.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let radius = (height * 0.5).round() as u8;
    ui.painter()
        .rect_filled(rect, CornerRadius::same(radius), palette.inset);
    let fill_width = (rect.width() * fraction.clamp(0.0, 1.0)).max(0.0);
    if fill_width > 0.0 {
        let fill_rect = egui::Rect::from_min_size(rect.min, Vec2::new(fill_width, height));
        ui.painter()
            .rect_filled(fill_rect, CornerRadius::same(radius), color);
    }
}

/// Labeled metric + progress bar row.
pub fn stat_bar(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    label: &str,
    value_label: &str,
    fraction: f32,
    color: Color32,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(10.5).color(palette.text));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(value_label)
                    .size(10.0)
                    .color(palette.secondary),
            );
        });
    });
    progress_bar(ui, palette, fraction, color, 5.0);
    ui.add_space(4.0);
}

/// KPI metric card with subtle border.
pub fn metric_card(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    label: &str,
    value: &str,
    color: Color32,
) {
    Frame::new()
        .fill(palette.surface)
        .stroke(Stroke::new(1.0, palette.border))
        .corner_radius(CornerRadius::same(Spacing::RADIUS_MD))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_min_width(112.0);
            ui.label(RichText::new(label).size(9.5).strong().color(palette.muted));
            ui.label(RichText::new(value).size(15.5).strong().color(color));
        });
}

/// Primary action button themed to the active palette.
pub fn action_button(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    text: &str,
    active: bool,
    accent: Color32,
) -> egui::Response {
    let fill = if active {
        ThemePalette::with_alpha(accent, 42)
    } else {
        palette.raised
    };
    ui.add(
        egui::Button::new(RichText::new(text).size(11.5).strong().color(if active {
            accent
        } else {
            palette.text
        }))
        .fill(fill)
        .stroke(Stroke::new(
            if active { 1.1 } else { 0.8 },
            if active { accent } else { palette.border },
        )),
    )
}

/// Animated filter/tab pill chip.
pub fn filter_chip(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    text: &str,
    selected: bool,
    accent: Color32,
) -> egui::Response {
    let width = (text.chars().count() as f32 * 6.4 + 22.0).max(44.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 28.0), Sense::click());
    let hovered =
        ui.ctx()
            .animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.12);
    let active = ui
        .ctx()
        .animate_bool_with_time(response.id.with("selected"), selected, 0.16);
    let emphasis = active.max(hovered * 0.5);
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(5),
        lerp_color(palette.card, ThemePalette::with_alpha(accent, 48), emphasis),
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(5),
        Stroke::new(
            0.8 + active * 0.7,
            lerp_color(palette.border, accent, emphasis),
        ),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        FontId::proportional(9.5),
        lerp_color(palette.secondary, accent, active),
    );
    response
}

/// Search input with inline clear (`×`) button and Enter-to-submit detection.
pub fn search_box(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    id_salt: &str,
    query: &mut String,
    hint: &str,
    width: Option<f32>,
) -> SearchBoxOutput {
    let mut output = SearchBoxOutput::default();
    let total_w = width.unwrap_or_else(|| ui.available_width()).max(140.0);
    let clear_reserve = if query.is_empty() { 0.0 } else { 28.0 };
    let input_w = (total_w - clear_reserve - 6.0).max(100.0);

    ui.horizontal(|ui| {
        let resp = ui.add_sized(
            [input_w, 30.0],
            egui::TextEdit::singleline(query)
                .id_salt(id_salt)
                .hint_text(hint),
        );
        output.changed = resp.changed();
        output.submitted =
            resp.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));

        if !query.is_empty()
            && ui
                .add_sized(
                    [26.0, 30.0],
                    egui::Button::new(RichText::new("×").size(12.0).color(palette.secondary))
                        .fill(palette.inset)
                        .stroke(Stroke::new(1.0, palette.border)),
                )
                .on_hover_text("Clear search")
                .clicked()
        {
            query.clear();
            output.changed = true;
        }
    });

    output
}

/// Unified animated boolean toggle switch used across Sidebar, HUD, and Settings.
pub fn toggle_switch(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    value: &mut bool,
) -> egui::Response {
    let desired_size = Vec2::new(36.0, 20.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }
    let how_on = ui.ctx().animate_bool(response.id, *value);
    let bg_color = lerp_color(palette.raised, palette.accent, how_on);
    ui.painter()
        .rect_filled(rect, CornerRadius::same(10), bg_color);
    let circle_x = egui::lerp((rect.left() + 10.0)..=(rect.right() - 10.0), how_on);
    let center = egui::pos2(circle_x, rect.center().y);
    ui.painter().circle_filled(center, 7.0, Color32::WHITE);
    response
}

/// Consistent labeled toggle row (replaces legacy inconsistent checkboxes & switches).
pub fn toggle_row(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    title: &str,
    subtitle: &str,
    value: &mut bool,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        let text_w = (ui.available_width() - 48.0).max(120.0);
        ui.allocate_ui_with_layout(
            Vec2::new(text_w, 0.0),
            Layout::top_down(Align::LEFT),
            |ui| {
                ui.label(RichText::new(title).size(12.0).strong().color(palette.text));
                if !subtitle.is_empty() {
                    ui.add(
                        egui::Label::new(
                            RichText::new(subtitle).size(10.0).color(palette.secondary),
                        )
                        .wrap(),
                    );
                }
            },
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if toggle_switch(ui, palette, value).changed() {
                changed = true;
            }
        });
    });
    ui.add_space(4.0);
    changed
}

/// Circular portrait or initials avatar with colored ring.
pub fn avatar(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    label: &str,
    size: f32,
    color: Color32,
    portrait: Option<&TextureHandle>,
) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    if let Some(texture) = portrait {
        egui::Image::new((texture.id(), rect.size()))
            .corner_radius(CornerRadius::same((size * 0.48) as u8))
            .paint_at(ui, rect);
    } else {
        ui.painter()
            .circle_filled(rect.center(), size * 0.48, palette.inset);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            initials(label),
            FontId::proportional(size * 0.30),
            color,
        );
    }
    ui.painter()
        .circle_stroke(rect.center(), size * 0.48, Stroke::new(1.4, color));
}

/// Renders an item icon slot (using cached texture if registered, or a styled item
/// tile with Data Dragon name/gold/description hover tooltip).
pub fn item_slot(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    textures: &TextureManager,
    item_id: u32,
    size: f32,
    show_label: bool,
) {
    let meta = items::item_by_id(item_id);
    let item_name = meta.map(|m| m.name.as_str()).unwrap_or("Empty Slot");
    let tint = match meta {
        Some(m) if m.is_trinket() => palette.gold,
        Some(m) if m.has_tag("Boots") => palette.green,
        Some(m) if m.has_tag("SpellDamage") || m.has_tag("Mana") => palette.violet,
        Some(m) if m.has_tag("Armor") || m.has_tag("Health") || m.has_tag("SpellBlock") => {
            palette.accent
        }
        Some(m)
            if m.has_tag("Damage") || m.has_tag("CriticalStrike") || m.has_tag("AttackSpeed") =>
        {
            palette.red
        }
        Some(_) => palette.accent,
        None => palette.border,
    };

    let response = ui
        .horizontal(|ui| {
            let (rect, rect_resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
            if item_id > 0 {
                if let Some(tex) = textures.item(item_id) {
                    egui::Image::new((tex.id(), rect.size()))
                        .corner_radius(CornerRadius::same(6))
                        .paint_at(ui, rect);
                } else {
                    ui.painter().rect_filled(
                        rect,
                        CornerRadius::same(6),
                        ThemePalette::with_alpha(tint, 28),
                    );
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        initials(item_name),
                        FontId::proportional((size * 0.36).clamp(9.0, 14.0)),
                        tint,
                    );
                }
            } else {
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(6), palette.inset);
            }
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(6),
                Stroke::new(1.0, if item_id > 0 { tint } else { palette.border }),
                egui::StrokeKind::Inside,
            );
            if show_label && item_id > 0 {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(item_name)
                            .size(10.5)
                            .strong()
                            .color(palette.text),
                    );
                    if let Some(item) = meta {
                        if item.gold.total > 0 {
                            ui.label(
                                RichText::new(format!("{}g", item.gold.total))
                                    .size(9.0)
                                    .color(palette.gold),
                            );
                        }
                    }
                });
            }
            rect_resp
        })
        .inner;

    if let Some(item) = meta {
        let desc = items::sanitize_description(&item.description);
        let tip = if desc.is_empty() {
            format!("{} ({}g)", item.name, item.gold.total)
        } else {
            let trimmed: String = desc.chars().take(200).collect();
            format!("{} ({}g)\n{}", item.name, item.gold.total, trimmed)
        };
        response.on_hover_text(tip);
    }
}

/// Banner colored cleanly by `CoreNotice` severity.
pub fn notice_banner(ui: &mut egui::Ui, palette: &ThemePalette, notice: &CoreNotice) {
    let color = match notice {
        CoreNotice::Success(_) => palette.green,
        CoreNotice::Warning(_) | CoreNotice::UpdateAvailable { .. } => palette.gold,
        CoreNotice::LcuError(_) | CoreNotice::RiotApiError(_) | CoreNotice::OverlayError(_) => {
            palette.red
        }
        CoreNotice::Info(_) => palette.accent,
    };
    Frame::new()
        .fill(ThemePalette::with_alpha(color, 26))
        .stroke(Stroke::new(1.0, ThemePalette::with_alpha(color, 130)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(12, 9))
        .show(ui, |ui| {
            ui.label(RichText::new(notice.message()).size(12.5).color(color));
        });
}

/// Interactive rune icon tile (supports both tree icons and individual rune icons).
pub fn rune_icon_card(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    id: i64,
    selected: bool,
    accent: Color32,
    side: f32,
    texture: Option<&TextureHandle>,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click());
    let hover =
        ui.ctx()
            .animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.12);
    let active = ui
        .ctx()
        .animate_bool_with_time(response.id.with("selected"), selected, 0.16);
    let emphasis = active.max(hover * 0.55);
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(8),
        lerp_color(palette.card, ThemePalette::with_alpha(accent, 48), emphasis),
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(8),
        Stroke::new(
            0.8 + active * 0.8,
            lerp_color(palette.border, accent, emphasis),
        ),
        egui::StrokeKind::Inside,
    );
    if let Some(texture) = texture {
        let image_rect = rect.shrink(4.0);
        egui::Image::new((texture.id(), image_rect.size()))
            .corner_radius(CornerRadius::same(6))
            .paint_at(ui, image_rect);
    } else {
        let fallback = runes::rune_by_id(id)
            .map(|r| initials(&r.name))
            .unwrap_or_else(|| runes::tree_name(id as u32).chars().take(2).collect());
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            fallback,
            FontId::proportional(10.0),
            accent,
        );
    }
    response
}

/// Summoner spell selector chip with D/F slot awareness.
pub fn spell_choice_chip(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    label: &str,
    selected_here: bool,
    selected_other: bool,
    accent: Color32,
) -> egui::Response {
    let width = (label.chars().count() as f32 * 5.8 + 27.0).clamp(62.0, 114.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 27.0), Sense::click());
    let hover =
        ui.ctx()
            .animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.12);
    let color = if selected_here {
        accent
    } else if selected_other {
        palette.gold
    } else {
        palette.border
    };
    let emphasis = if selected_here {
        1.0
    } else if selected_other {
        0.62
    } else {
        hover * 0.5
    };
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(7),
        lerp_color(palette.card, ThemePalette::with_alpha(color, 46), emphasis),
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(7),
        Stroke::new(
            0.8 + if selected_here { 0.6 } else { 0.0 },
            lerp_color(palette.border, color, emphasis),
        ),
        egui::StrokeKind::Inside,
    );
    ui.painter()
        .circle_filled(egui::pos2(rect.left() + 9.0, rect.center().y), 2.5, color);
    ui.painter().text(
        egui::pos2(rect.left() + 16.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(9.0),
        if selected_here {
            palette.text
        } else {
            palette.secondary
        },
    );
    response.on_hover_text(if selected_other {
        format!("{label} is assigned to the other slot; click to swap")
    } else {
        label.to_owned()
    })
}

/// Shared match history row used in both Profile and Lobby Scout.
/// Returns `true` when the user clicks `"ANALYZE"`.
pub fn match_summary_row(
    ui: &mut egui::Ui,
    palette: &ThemePalette,
    textures: &mut TextureManager,
    summary: &NormalizedMatchSummary,
) -> bool {
    let mut analyze_clicked = false;
    let outcome_color = if summary.win {
        palette.green
    } else {
        palette.red
    };
    let champ_key = champion_by_id(summary.champion_id)
        .map(|c| c.key.as_str())
        .unwrap_or(&summary.champion_name);

    inset_card(
        ui,
        palette,
        ThemePalette::with_alpha(outcome_color, 85),
        |ui| {
            ui.horizontal(|ui| {
                let ctx = ui.ctx().clone();
                let portrait = textures.champion(&ctx, champ_key);
                avatar(
                    ui,
                    palette,
                    &summary.champion_name,
                    34.0,
                    outcome_color,
                    portrait,
                );
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        badge(ui, if summary.win { "WIN" } else { "LOSS" }, outcome_color);
                        ui.label(
                            RichText::new(&summary.champion_name)
                                .size(11.5)
                                .strong()
                                .color(palette.text),
                        );
                        ui.label(
                            RichText::new(format!(
                                "{} · {}:{:02}",
                                summary.game_mode,
                                summary.duration_secs / 60,
                                summary.duration_secs % 60
                            ))
                            .size(9.5)
                            .color(palette.muted),
                        );
                    });
                    ui.horizontal(|ui| {
                        kda_display(
                            ui,
                            palette,
                            summary.kills,
                            summary.deaths,
                            summary.assists,
                            Some(summary.kda),
                        );
                        ui.label(
                            RichText::new(format!(
                                "{} CS ({:.1}/m) · {:.0}% KP",
                                summary.cs, summary.cs_per_min, summary.kp_pct
                            ))
                            .size(9.5)
                            .color(palette.secondary),
                        );
                    });
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if action_button(ui, palette, "ANALYZE", false, palette.accent).clicked() {
                        analyze_clicked = true;
                    }
                });
            });
        },
    );
    ui.add_space(5.0);
    analyze_clicked
}

/// Extracts up to two uppercase alphanumeric initials from a display name.
pub fn initials(name: &str) -> String {
    let letters: String = name
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .take(2)
        .collect::<String>()
        .to_ascii_uppercase();
    if letters.is_empty() {
        "RC".to_string()
    } else {
        letters
    }
}

/// Resolves the accent color for a rune tree ID using the active palette fallback.
pub fn tree_accent(tree_id: u32, palette: &ThemePalette) -> Color32 {
    rune_tree_color(tree_id, palette.accent)
}
