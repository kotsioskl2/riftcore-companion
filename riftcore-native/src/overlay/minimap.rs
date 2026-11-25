//! Summoner's Rift minimap objective, jungle-route, and vision pin renderer.
//!
//! Strictly gated to Summoner's Rift (`LiveGameState::is_summoners_rift()`) and uses
//! [`LiveGameState::objective_timers`] so BARON is never duplicated.

use crate::domain::live_game::{LiveGameState, ObjectiveKind};
use crate::overlay::host::OverlayPanelSettings;
use eframe::egui::{self, Color32, FontId, Pos2, Rect, Stroke, Vec2};

const BG: Color32 = Color32::from_rgb(5, 9, 17);
const CYAN: Color32 = Color32::from_rgb(66, 229, 232);
const GOLD: Color32 = Color32::from_rgb(255, 197, 103);
const GREEN: Color32 = Color32::from_rgb(92, 224, 149);
const TEXT: Color32 = Color32::from_rgb(239, 247, 255);

#[derive(Clone, Debug, PartialEq)]
pub struct RouteStepPin {
    pub step: usize,
    pub short_label: &'static str,
    pub camp_name: &'static str,
    pub x_pct: f32,
    pub y_pct: f32,
    pub active: bool,
    pub remaining_secs: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObjectiveMapPin {
    pub short_label: &'static str,
    pub kind: ObjectiveKind,
    pub timer_text: String,
    pub x_pct: f32,
    pub y_pct: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WardMapPin {
    pub short_label: &'static str,
    pub x_pct: f32,
    pub y_pct: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MinimapGuideModel {
    pub is_red_side: bool,
    pub size_px: f32,
    pub left: f32,
    pub bottom: f32,
    pub flip_x: bool,
    pub route_pins: Vec<RouteStepPin>,
    pub ward_pins: Vec<WardMapPin>,
    pub objective_pins: Vec<ObjectiveMapPin>,
}

impl MinimapGuideModel {
    pub fn map_point(&self, x_pct: f32, y_pct: f32) -> Pos2 {
        let x = if self.flip_x { 100.0 - x_pct } else { x_pct };
        Pos2::new(
            self.left + self.size_px * x / 100.0,
            self.bottom - self.size_px * y_pct / 100.0,
        )
    }
}

/// Computes the current Summoner's Rift opening jungle clear step `(step, camp_name, secs, active)`.
pub fn jungle_route_status(
    game_time: u64,
    start_buff: &str,
) -> Option<(usize, &'static str, u64, bool)> {
    const RED_START: [(u64, u64, &str); 7] = [
        (90, 110, "Red Buff"),
        (110, 135, "Krugs"),
        (135, 160, "Raptors"),
        (160, 180, "Wolves"),
        (180, 195, "Blue Buff"),
        (195, 210, "Gromp"),
        (210, 240, "Scuttle Crab"),
    ];
    const BLUE_START: [(u64, u64, &str); 7] = [
        (90, 110, "Blue Buff"),
        (110, 135, "Gromp"),
        (135, 160, "Wolves"),
        (160, 180, "Raptors"),
        (180, 195, "Red Buff"),
        (195, 210, "Krugs"),
        (210, 240, "Scuttle Crab"),
    ];
    let route = if start_buff.eq_ignore_ascii_case("blue") {
        &BLUE_START
    } else {
        &RED_START
    };
    for (index, (start, end, camp)) in route.iter().enumerate() {
        if game_time < *start {
            return Some((index + 1, camp, start - game_time, false));
        }
        if game_time < *end {
            return Some((index + 1, camp, end - game_time, true));
        }
    }
    None
}

/// Builds a typed minimap guide model, returning `None` outside Summoner's Rift or when
/// all minimap pin categories are disabled.
pub fn build_minimap_guide_model(
    area: Rect,
    live: &LiveGameState,
    settings: &OverlayPanelSettings,
    game_time: u64,
) -> Option<MinimapGuideModel> {
    if !live.is_summoners_rift() {
        return None;
    }
    if !settings.hud_map_objective_pins && !settings.hud_map_route_pins && !settings.hud_map_ward_pins
    {
        return None;
    }

    let raw_scale = if settings.hud_minimap_scale > 1.5 {
        settings.hud_minimap_scale / 2.0
    } else {
        settings.hud_minimap_scale
    }
    .clamp(0.55, 1.5);
    let max_size = (area.height() * 0.48).max(150.0);
    let size_px = (area.height() * 0.25 * raw_scale * settings.hud_minimap_zoom.clamp(0.75, 1.35))
        .clamp(150.0, max_size);
    let left = if settings.hud_minimap_flip {
        area.left() + 8.0 + settings.hud_minimap_nudge_x
    } else {
        area.right() - size_px - 8.0 + settings.hud_minimap_nudge_x
    };
    let bottom = area.bottom() - 8.0 + settings.hud_minimap_nudge_y;

    let is_red_side = live.active_player.team.eq_ignore_ascii_case("CHAOS");

    let mut route_pins = Vec::new();
    if settings.hud_map_route_pins && settings.hud_jungle_pathing {
        let route: [(&str, &str, f32, f32, u64, u64); 7] =
            match (is_red_side, settings.jungle_start_buff.eq_ignore_ascii_case("blue")) {
                (true, false) => [
                    ("R", "Red Buff", 48.0, 73.0, 90, 110),
                    ("K", "Krugs", 43.5, 82.0, 110, 135),
                    ("R", "Raptors", 53.0, 64.0, 135, 160),
                    ("W", "Wolves", 74.0, 56.5, 160, 180),
                    ("B", "Blue Buff", 74.5, 47.5, 180, 195),
                    ("G", "Gromp", 86.0, 44.0, 195, 210),
                    ("S", "Scuttle Crab", 29.5, 64.0, 210, 240),
                ],
                (true, true) => [
                    ("B", "Blue Buff", 74.5, 47.5, 90, 110),
                    ("G", "Gromp", 86.0, 44.0, 110, 135),
                    ("W", "Wolves", 74.0, 56.5, 135, 160),
                    ("R", "Raptors", 53.0, 64.0, 160, 180),
                    ("R", "Red Buff", 48.0, 73.0, 180, 195),
                    ("K", "Krugs", 43.5, 82.0, 195, 210),
                    ("S", "Scuttle Crab", 29.5, 64.0, 210, 240),
                ],
                (false, false) => [
                    ("R", "Red Buff", 52.0, 27.0, 90, 110),
                    ("K", "Krugs", 56.5, 18.0, 110, 135),
                    ("R", "Raptors", 47.0, 36.0, 135, 160),
                    ("W", "Wolves", 26.0, 43.5, 160, 180),
                    ("B", "Blue Buff", 25.5, 52.5, 180, 195),
                    ("G", "Gromp", 14.0, 56.0, 195, 210),
                    ("S", "Scuttle Crab", 70.0, 35.0, 210, 240),
                ],
                (false, true) => [
                    ("B", "Blue Buff", 25.5, 52.5, 90, 110),
                    ("G", "Gromp", 14.0, 56.0, 110, 135),
                    ("W", "Wolves", 26.0, 43.5, 135, 160),
                    ("R", "Raptors", 47.0, 36.0, 160, 180),
                    ("R", "Red Buff", 52.0, 27.0, 180, 195),
                    ("K", "Krugs", 56.5, 18.0, 195, 210),
                    ("S", "Scuttle Crab", 70.0, 35.0, 210, 240),
                ],
            };
        let active_step = jungle_route_status(game_time, &settings.jungle_start_buff)
            .map(|(step, _, _, _)| step);
        for (idx, &(short_label, camp_name, x_pct, y_pct, start, end)) in route.iter().enumerate() {
            let step = idx + 1;
            let active = active_step == Some(step);
            let remaining_secs = if game_time < start {
                start - game_time
            } else if game_time < end {
                end - game_time
            } else {
                0
            };
            route_pins.push(RouteStepPin {
                step,
                short_label,
                camp_name,
                x_pct,
                y_pct,
                active,
                remaining_secs,
            });
        }
    }

    let ward_pins = if settings.hud_map_ward_pins {
        vec![
            WardMapPin { short_label: "W", x_pct: 62.0, y_pct: 40.0 },
            WardMapPin { short_label: "W", x_pct: 38.0, y_pct: 60.0 },
            WardMapPin { short_label: "W", x_pct: 80.0, y_pct: 75.0 },
            WardMapPin { short_label: "W", x_pct: 20.0, y_pct: 25.0 },
            WardMapPin { short_label: "W", x_pct: 64.0, y_pct: 84.0 },
            WardMapPin { short_label: "W", x_pct: 36.0, y_pct: 16.0 },
        ]
    } else {
        Vec::new()
    };

    let mut objective_pins = Vec::new();
    if settings.hud_map_objective_pins {
        let timers = live.objective_timers();
        if let Some(south) = timers
            .iter()
            .find(|t| matches!(t.kind, ObjectiveKind::Dragon | ObjectiveKind::ElderDragon))
        {
            let short = if south.kind == ObjectiveKind::ElderDragon {
                "E"
            } else {
                "D"
            };
            objective_pins.push(ObjectiveMapPin {
                short_label: short,
                kind: south.kind,
                timer_text: south.formatted_remaining(),
                x_pct: 65.5,
                y_pct: 29.5,
            });
        }

        // North pit: Voidgrubs / Rift Herald pre-20m, transitioning cleanly to canonical BARON.
        let north_timer = if game_time < 1185 {
            timers
                .iter()
                .find(|t| matches!(t.kind, ObjectiveKind::RiftHerald | ObjectiveKind::Voidgrubs))
                .or_else(|| timers.iter().find(|t| t.kind == ObjectiveKind::Baron))
        } else {
            timers.iter().find(|t| t.kind == ObjectiveKind::Baron)
        };
        if let Some(north) = north_timer {
            let short = match north.kind {
                ObjectiveKind::RiftHerald => "H",
                ObjectiveKind::Baron => "B",
                _ => "G",
            };
            objective_pins.push(ObjectiveMapPin {
                short_label: short,
                kind: north.kind,
                timer_text: north.formatted_remaining(),
                x_pct: 34.0,
                y_pct: 69.5,
            });
        }
    }

    Some(MinimapGuideModel {
        is_red_side,
        size_px,
        left,
        bottom,
        flip_x: settings.hud_minimap_flip,
        route_pins,
        ward_pins,
        objective_pins,
    })
}

/// Draws Summoner's Rift minimap route, ward, and objective pins inside the overlay context.
pub fn render_minimap_guide_pins(
    ctx: &egui::Context,
    area: Rect,
    live: Option<&LiveGameState>,
    settings: &OverlayPanelSettings,
    game_time: u64,
) {
    let Some(live) = live else {
        return;
    };
    let Some(model) = build_minimap_guide_model(area, live, settings, game_time) else {
        return;
    };

    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::unique("riftcore-minimap-guide-pins"),
    ));

    for pin in &model.route_pins {
        let center = model.map_point(pin.x_pct, pin.y_pct);
        let fill = if pin.active {
            CYAN
        } else {
            Color32::from_rgba_unmultiplied(7, 16, 27, 228)
        };
        painter.circle_filled(center, 8.5, fill);
        painter.circle_stroke(
            center,
            8.5,
            Stroke::new(1.3, if pin.active { TEXT } else { CYAN }),
        );
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            format!("{}{}", pin.step, pin.short_label),
            FontId::proportional(7.5),
            if pin.active { BG } else { TEXT },
        );
    }

    for pin in &model.ward_pins {
        let center = model.map_point(pin.x_pct, pin.y_pct);
        painter.circle_filled(center, 7.0, Color32::from_rgb(8, 85, 62));
        painter.circle_stroke(center, 7.0, Stroke::new(1.0, GREEN));
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            pin.short_label,
            FontId::proportional(8.0),
            TEXT,
        );
    }

    for pin in &model.objective_pins {
        let center = model.map_point(pin.x_pct, pin.y_pct);
        painter.circle_stroke(center, 12.0, Stroke::new(1.6, GOLD));
        painter.circle_filled(center, 2.5, GOLD);
        let label_center = center + Vec2::new(31.0, 20.0);
        let marker = Rect::from_center_size(label_center, Vec2::new(52.0, 17.0));
        painter.rect_filled(
            marker,
            egui::CornerRadius::same(7),
            Color32::from_rgba_unmultiplied(6, 13, 24, 235),
        );
        painter.rect_stroke(
            marker,
            egui::CornerRadius::same(7),
            Stroke::new(1.0, GOLD),
            egui::StrokeKind::Inside,
        );
        painter.text(
            label_center,
            egui::Align2::CENTER_CENTER,
            format!("{} {}", pin.short_label, pin.timer_text),
            FontId::proportional(8.0),
            TEXT,
        );
    }
}
