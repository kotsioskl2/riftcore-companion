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

/// Pins drawn over League's minimap. Map percentages run from League's bottom-left (blue
/// base, 0 %) to top-right (red base, 100 %); flipping the minimap to the left moves the whole
/// square, it does not mirror the map.
#[derive(Clone, Debug, PartialEq)]
pub struct MinimapGuideModel {
    pub is_red_side: bool,
    /// Side length of the minimap square, in HUD points.
    pub size_px: f32,
    pub left: f32,
    pub bottom: f32,
    /// Minimap sits bottom-left (League's flip setting) instead of bottom-right.
    pub flipped_left: bool,
    pub route_pins: Vec<RouteStepPin>,
    pub ward_pins: Vec<WardMapPin>,
    pub objective_pins: Vec<ObjectiveMapPin>,
}

/// Minimap square at map scale 1.0, as a share of the HUD height (calibrated at 1080p).
const MINIMAP_SIZE_PER_HEIGHT: f32 = 0.2435;
/// Gap between League's minimap and the screen edges, as a share of the HUD height.
const MINIMAP_MARGIN_PER_HEIGHT: f32 = 0.0204;
/// Minimap side (points) the pin sizes below are designed for (1080p, default map scale).
const PIN_DESIGN_SIZE: f32 = 284.0;

impl MinimapGuideModel {
    pub fn map_point(&self, x_pct: f32, y_pct: f32) -> Pos2 {
        Pos2::new(
            self.left + self.size_px * x_pct / 100.0,
            self.bottom - self.size_px * y_pct / 100.0,
        )
    }

    /// League's minimap square the pins are drawn inside.
    pub fn square(&self) -> Rect {
        Rect::from_min_size(
            Pos2::new(self.left, self.bottom - self.size_px),
            Vec2::splat(self.size_px),
        )
    }

    /// Pin and label sizes follow the minimap so they keep their place at any resolution.
    fn pin_scale(&self) -> f32 {
        (self.size_px / PIN_DESIGN_SIZE).clamp(0.6, 1.6)
    }

    /// Radius of route-step pins (ward pins are a bit smaller).
    pub fn route_pin_radius(&self) -> f32 {
        8.5 * self.pin_scale()
    }

    pub fn ward_pin_radius(&self) -> f32 {
        7.0 * self.pin_scale()
    }

    pub fn objective_ring_radius(&self) -> f32 {
        12.0 * self.pin_scale()
    }

    /// Timer label of an objective pin centred at `center`: down-right of the pin, moved back
    /// inside the minimap square when it would stick out.
    pub fn objective_label_rect(&self, center: Pos2) -> Rect {
        let scale = self.pin_scale();
        let label = Rect::from_center_size(
            center + Vec2::new(31.0, 20.0) * scale,
            Vec2::new(52.0, 17.0) * scale,
        );
        keep_inside(label, self.square())
    }
}

/// Moves `rect` (without resizing) so it lies inside `bounds` where it fits.
fn keep_inside(rect: Rect, bounds: Rect) -> Rect {
    let max = (bounds.max - rect.size()).max(bounds.min);
    rect.translate(rect.min.clamp(bounds.min, max) - rect.min)
}

/// Size multiplier of the "Map scale" setting. The slider (1.0 to 3.0) uses League `game.cfg`
/// `MinimapScale` units (2.0 = 1.0x), so the size grows steadily with it; values below 1.0
/// can only come from old settings files that stored the multiplier itself.
pub fn minimap_scale_factor(setting: f32) -> f32 {
    let setting = if setting.is_finite() { setting } else { 2.0 };
    if setting >= 1.0 {
        setting / 2.0
    } else {
        setting
    }
    .clamp(0.5, 1.5)
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

/// League's minimap square inside the HUD `screen` (the HUD window covering the game, in points):
/// bottom-right, or bottom-left when flipped. Sized from the screen height, it lines up at any
/// resolution and Windows scaling; map scale and the x/y nudge (points) apply, and the result
/// always stays on screen. "Dock size" (`hud_minimap_zoom`) sizes only the objectives dock, so
/// it never pulls the pins off League's minimap.
pub fn minimap_square(screen: Rect, settings: &OverlayPanelSettings) -> Rect {
    let height = screen.height().max(1.0);
    let max_size = (height * 0.5).min(screen.width()).max(1.0);
    let size =
        (height * MINIMAP_SIZE_PER_HEIGHT * minimap_scale_factor(settings.hud_minimap_scale))
            .clamp(1.0, max_size);
    let margin = height * MINIMAP_MARGIN_PER_HEIGHT;
    let left = if settings.hud_minimap_flip {
        screen.left() + margin
    } else {
        screen.right() - margin - size
    };
    let nudge = Vec2::new(
        finite_or(settings.hud_minimap_nudge_x, 0.0),
        finite_or(settings.hud_minimap_nudge_y, 0.0),
    );
    let square = Rect::from_min_size(
        Pos2::new(left, screen.bottom() - margin - size) + nudge,
        Vec2::splat(size),
    );
    keep_inside(square, screen)
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

/// Builds a typed minimap guide model for a HUD window covering the game (`screen`, in points)
/// at whole game second `game_time`, returning `None` outside Summoner's Rift or when all
/// minimap pin categories are disabled.
pub fn build_minimap_guide_model(
    screen: Rect,
    live: &LiveGameState,
    settings: &OverlayPanelSettings,
    game_time: u64,
) -> Option<MinimapGuideModel> {
    if !live.is_summoners_rift() {
        return None;
    }
    if !settings.hud_map_objective_pins
        && !settings.hud_map_route_pins
        && !settings.hud_map_ward_pins
    {
        return None;
    }

    let square = minimap_square(screen, settings);
    let size_px = square.width();
    let left = square.left();
    let bottom = square.bottom();

    let is_red_side = live.active_player.team.eq_ignore_ascii_case("CHAOS");

    let mut route_pins = Vec::new();
    if settings.hud_map_route_pins && settings.hud_jungle_pathing {
        let route: [(&str, &str, f32, f32, u64, u64); 7] = match (
            is_red_side,
            settings.jungle_start_buff.eq_ignore_ascii_case("blue"),
        ) {
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
        let active_step =
            jungle_route_status(game_time, &settings.jungle_start_buff).map(|(step, _, _, _)| step);
        for (idx, &(short_label, camp_name, x_pct, y_pct, start, end)) in route.iter().enumerate() {
            let step = idx + 1;
            let active = active_step == Some(step);
            let remaining_secs = if game_time < start {
                start - game_time
            } else {
                end.saturating_sub(game_time)
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
            WardMapPin {
                short_label: "W",
                x_pct: 62.0,
                y_pct: 40.0,
            },
            WardMapPin {
                short_label: "W",
                x_pct: 38.0,
                y_pct: 60.0,
            },
            WardMapPin {
                short_label: "W",
                x_pct: 80.0,
                y_pct: 75.0,
            },
            WardMapPin {
                short_label: "W",
                x_pct: 20.0,
                y_pct: 25.0,
            },
            WardMapPin {
                short_label: "W",
                x_pct: 64.0,
                y_pct: 84.0,
            },
            WardMapPin {
                short_label: "W",
                x_pct: 36.0,
                y_pct: 16.0,
            },
        ]
    } else {
        Vec::new()
    };

    let mut objective_pins = Vec::new();
    if settings.hud_map_objective_pins {
        let timers = live.objective_timers_at(game_time as f64);
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
        flipped_left: settings.hud_minimap_flip,
        route_pins,
        ward_pins,
        objective_pins,
    })
}

/// Draws Summoner's Rift minimap route, ward, and objective pins inside the overlay context,
/// clipped to League's minimap square on the HUD window `screen`.
pub fn render_minimap_guide_pins(
    ctx: &egui::Context,
    screen: Rect,
    live: Option<&LiveGameState>,
    settings: &OverlayPanelSettings,
    game_time: u64,
) {
    let Some(live) = live else {
        return;
    };
    let Some(model) = build_minimap_guide_model(screen, live, settings, game_time) else {
        return;
    };

    let painter = ctx
        .layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::unique("riftcore-minimap-guide-pins"),
        ))
        .with_clip_rect(model.square());
    let scale = model.pin_scale();
    let font = |size: f32| FontId::proportional((size * scale).max(6.5));

    let route_radius = model.route_pin_radius();
    for pin in &model.route_pins {
        let center = model.map_point(pin.x_pct, pin.y_pct);
        let fill = if pin.active {
            CYAN
        } else {
            Color32::from_rgba_unmultiplied(7, 16, 27, 228)
        };
        painter.circle_filled(center, route_radius, fill);
        painter.circle_stroke(
            center,
            route_radius,
            Stroke::new(1.3, if pin.active { TEXT } else { CYAN }),
        );
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            format!("{}{}", pin.step, pin.short_label),
            font(7.5),
            if pin.active { BG } else { TEXT },
        );
    }

    let ward_radius = model.ward_pin_radius();
    for pin in &model.ward_pins {
        let center = model.map_point(pin.x_pct, pin.y_pct);
        painter.circle_filled(center, ward_radius, Color32::from_rgb(8, 85, 62));
        painter.circle_stroke(center, ward_radius, Stroke::new(1.0, GREEN));
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            pin.short_label,
            font(8.0),
            TEXT,
        );
    }

    for pin in &model.objective_pins {
        let center = model.map_point(pin.x_pct, pin.y_pct);
        painter.circle_stroke(
            center,
            model.objective_ring_radius(),
            Stroke::new(1.6, GOLD),
        );
        painter.circle_filled(center, 2.5 * scale, GOLD);
        let marker = model.objective_label_rect(center);
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
            marker.center(),
            egui::Align2::CENTER_CENTER,
            format!("{} {}", pin.short_label, pin.timer_text),
            font(8.0),
            TEXT,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::live_game::LiveGameState;
    use serde_json::json;

    fn sr_live() -> LiveGameState {
        LiveGameState::from_allgamedata(&json!({
            "gameData": { "gameTime": 100.0, "gameMode": "CLASSIC", "mapNumber": 11 },
            "activePlayer": { "summonerName": "P1" },
            "allPlayers": [{ "summonerName": "P1", "team": "CHAOS", "championName": "Lee Sin" }],
            "events": { "Events": [] }
        }))
        .expect("valid SR payload")
    }

    #[test]
    fn minimap_pins_stay_inside_a_square_that_scales_with_the_game_not_windows_scaling() {
        let live = sr_live();
        let all_pins = OverlayPanelSettings {
            hud_map_ward_pins: true,
            ..OverlayPanelSettings::default()
        };
        for (w, h) in [(1280, 720), (1920, 1080), (2560, 1440), (3840, 2160)] {
            // The square measured in physical pixels must not depend on Windows scaling.
            let mut physical: Option<Rect> = None;
            for dpi in [96, 120, 144, 192] {
                let ppp = crate::overlay::hud_modes::hud_pixels_per_point(dpi, [w, h]);
                // The Integrated HUD window keeps a 1 px inset inside the game.
                let screen = Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new((w - 1) as f32 / ppp, (h - 1) as f32 / ppp),
                );
                for flip in [false, true] {
                    let settings = OverlayPanelSettings {
                        hud_minimap_flip: flip,
                        ..all_pins.clone()
                    };
                    let model =
                        build_minimap_guide_model(screen, &live, &settings, 100).expect("SR model");
                    let square = model.square();
                    let tag = format!("{w}x{h}@{dpi} flip={flip}");
                    assert!(screen.contains_rect(square), "{tag}");
                    // Bottom-right normally, bottom-left when flipped.
                    assert_eq!(square.center().x < screen.center().x, flip, "{tag}");
                    assert!(
                        square.bottom() > screen.bottom() - 0.03 * screen.height(),
                        "{tag}"
                    );
                    let pins = model
                        .route_pins
                        .iter()
                        .map(|p| (p.x_pct, p.y_pct, model.route_pin_radius()))
                        .chain(
                            model
                                .ward_pins
                                .iter()
                                .map(|p| (p.x_pct, p.y_pct, model.ward_pin_radius())),
                        )
                        .chain(
                            model
                                .objective_pins
                                .iter()
                                .map(|p| (p.x_pct, p.y_pct, model.objective_ring_radius())),
                        );
                    for (x, y, radius) in pins {
                        let center = model.map_point(x, y);
                        let pin = Rect::from_center_size(center, Vec2::splat(radius * 2.0));
                        assert!(square.contains_rect(pin), "{tag} pin {x},{y}");
                        assert!(
                            square.contains_rect(model.objective_label_rect(center)),
                            "{tag} label {x},{y}"
                        );
                    }
                    if !flip {
                        let px = Rect::from_min_max(
                            (square.min.to_vec2() * ppp).to_pos2(),
                            (square.max.to_vec2() * ppp).to_pos2(),
                        );
                        match physical {
                            Some(first) => {
                                assert!((px.min - first.min).length() < 1.5, "{tag}");
                                assert!((px.width() - first.width()).abs() < 1.5, "{tag}");
                            }
                            None => physical = Some(px),
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn minimap_square_honours_scale_and_nudge_and_stays_on_screen() {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(1919.0, 1079.0));
        let base = minimap_square(screen, &OverlayPanelSettings::default());
        // Calibrated at 1080p: ~284 pt square, ~22 pt from the bottom-right corner.
        assert!((base.width() - 283.75).abs() < 1.0, "{base:?}");
        assert!(
            (screen.right() - base.right() - 22.0).abs() < 1.0,
            "{base:?}"
        );
        let with = |f: fn(&mut OverlayPanelSettings)| {
            let mut settings = OverlayPanelSettings::default();
            f(&mut settings);
            minimap_square(screen, &settings)
        };
        assert!(with(|s| s.hud_minimap_scale = 3.0).width() > base.width());
        // "Dock size" resizes the dock only; the pins stay on League's minimap.
        assert_eq!(with(|s| s.hud_minimap_zoom = 1.35), base);
        // The map scale slider (1.0 to 3.0) only ever grows the square as it moves right.
        let mut last = 0.0;
        for step in 0..=200 {
            let setting = 1.0 + step as f32 * 0.01;
            let factor = minimap_scale_factor(setting);
            assert!(factor >= last, "{setting}: {factor} < {last}");
            last = factor;
        }
        let nudged = with(|s| {
            s.hud_minimap_nudge_x = -40.0;
            s.hud_minimap_nudge_y = -25.0;
        });
        assert!((nudged.min - base.min - Vec2::new(-40.0, -25.0)).length() < 0.01);
        // Nudges past the edge (or garbage values) never push the square off screen.
        for settings in [
            OverlayPanelSettings {
                hud_minimap_nudge_x: 250.0,
                hud_minimap_nudge_y: 250.0,
                ..OverlayPanelSettings::default()
            },
            OverlayPanelSettings {
                hud_minimap_flip: true,
                hud_minimap_nudge_x: -250.0,
                hud_minimap_scale: f32::NAN,
                hud_minimap_zoom: f32::INFINITY,
                ..OverlayPanelSettings::default()
            },
        ] {
            let square = minimap_square(screen, &settings);
            assert!(screen.contains_rect(square), "{square:?}");
            assert!(square.width() > 100.0, "{square:?}");
        }
    }
}
