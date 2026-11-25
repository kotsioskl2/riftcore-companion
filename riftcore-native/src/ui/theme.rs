//! Dynamic theme palette, typography installation, spacing tokens, and color helpers.

use crate::domain::champions::Role;
use crate::domain::runes::{
    TREE_DOMINATION, TREE_INSPIRATION, TREE_PRECISION, TREE_RESOLVE, TREE_SORCERY,
};
use eframe::egui::{self, Color32, FontId, Stroke};

/// Supported application color themes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ThemeMode {
    #[default]
    Hextech,
    Ember,
    Void,
}

impl ThemeMode {
    pub fn from_str_lossy(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "ember" => Self::Ember,
            "void" => Self::Void,
            _ => Self::Hextech,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hextech => "hextech",
            Self::Ember => "ember",
            Self::Void => "void",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Hextech => "Hextech",
            Self::Ember => "Ember",
            Self::Void => "Void",
        }
    }
}

/// Supported bundled proportional font families.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FontChoice {
    #[default]
    PlusJakarta,
    Outfit,
    Inter,
}

impl FontChoice {
    pub fn from_str_lossy(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "outfit" => Self::Outfit,
            "inter" => Self::Inter,
            _ => Self::PlusJakarta,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PlusJakarta => "plus-jakarta",
            Self::Outfit => "outfit",
            Self::Inter => "inter",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::PlusJakarta => "Plus Jakarta Sans",
            Self::Outfit => "Outfit",
            Self::Inter => "Inter",
        }
    }
}

/// Layout spacing and geometry tokens shared across chrome, widgets, and pages.
pub struct Spacing;

impl Spacing {
    pub const TITLEBAR_HEIGHT: f32 = 38.0;
    pub const SIDEBAR_WIDTH: f32 = 224.0;
    pub const PAGE_MARGIN: i8 = 18;
    pub const PANEL_MARGIN: i8 = 14;
    pub const CARD_MARGIN: i8 = 10;
    pub const RADIUS_SM: u8 = 6;
    pub const RADIUS_MD: u8 = 8;
    pub const RADIUS_LG: u8 = 12;
    pub const RADIUS_MODAL: u8 = 16;
    pub const SECTION_GAP: f32 = 10.0;
}

/// Full runtime theme palette consumed by every custom painter, frame, and widget.
/// Replacing static compile-time color constants ensures theme switches re-skin
/// panels, borders, accents, and backgrounds consistently across all views.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemePalette {
    pub mode: ThemeMode,
    pub bg: Color32,
    pub surface: Color32,
    pub card: Color32,
    pub raised: Color32,
    pub inset: Color32,
    pub accent: Color32,
    pub accent_dim: Color32,
    pub violet: Color32,
    pub red: Color32,
    pub gold: Color32,
    pub green: Color32,
    pub text: Color32,
    pub secondary: Color32,
    pub muted: Color32,
    pub border: Color32,
    pub grid_minor: Color32,
    pub grid_major: Color32,
}

impl Default for ThemePalette {
    fn default() -> Self {
        Self::for_mode(ThemeMode::Hextech)
    }
}

impl ThemePalette {
    pub fn from_name(raw: &str) -> Self {
        Self::for_mode(ThemeMode::from_str_lossy(raw))
    }

    pub const fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Hextech => Self {
                mode,
                bg: Color32::from_rgb(5, 9, 17),
                surface: Color32::from_rgb(10, 17, 30),
                card: Color32::from_rgb(15, 25, 42),
                raised: Color32::from_rgb(22, 38, 59),
                inset: Color32::from_rgb(11, 17, 27),
                accent: Color32::from_rgb(66, 229, 232),
                accent_dim: Color32::from_rgb(13, 57, 73),
                violet: Color32::from_rgb(157, 117, 255),
                red: Color32::from_rgb(255, 86, 126),
                gold: Color32::from_rgb(255, 197, 103),
                green: Color32::from_rgb(82, 230, 173),
                text: Color32::from_rgb(239, 247, 255),
                secondary: Color32::from_rgb(164, 185, 208),
                muted: Color32::from_rgb(110, 136, 162),
                border: Color32::from_rgb(31, 55, 79),
                grid_minor: Color32::from_rgba_unmultiplied(40, 133, 171, 11),
                grid_major: Color32::from_rgba_unmultiplied(80, 105, 200, 10),
            },
            ThemeMode::Ember => Self {
                mode,
                bg: Color32::from_rgb(13, 10, 14),
                surface: Color32::from_rgb(20, 16, 22),
                card: Color32::from_rgb(29, 23, 31),
                raised: Color32::from_rgb(42, 33, 44),
                inset: Color32::from_rgb(21, 16, 23),
                accent: Color32::from_rgb(255, 155, 97),
                accent_dim: Color32::from_rgb(72, 41, 31),
                violet: Color32::from_rgb(236, 121, 162),
                red: Color32::from_rgb(255, 92, 116),
                gold: Color32::from_rgb(255, 209, 107),
                green: Color32::from_rgb(98, 226, 168),
                text: Color32::from_rgb(252, 244, 240),
                secondary: Color32::from_rgb(209, 185, 178),
                muted: Color32::from_rgb(150, 128, 125),
                border: Color32::from_rgb(72, 50, 56),
                grid_minor: Color32::from_rgba_unmultiplied(214, 112, 72, 10),
                grid_major: Color32::from_rgba_unmultiplied(190, 92, 122, 10),
            },
            ThemeMode::Void => Self {
                mode,
                bg: Color32::from_rgb(7, 8, 19),
                surface: Color32::from_rgb(13, 14, 30),
                card: Color32::from_rgb(20, 21, 42),
                raised: Color32::from_rgb(31, 32, 61),
                inset: Color32::from_rgb(14, 15, 32),
                accent: Color32::from_rgb(174, 147, 255),
                accent_dim: Color32::from_rgb(46, 38, 86),
                violet: Color32::from_rgb(98, 218, 255),
                red: Color32::from_rgb(255, 94, 142),
                gold: Color32::from_rgb(255, 198, 112),
                green: Color32::from_rgb(92, 232, 184),
                text: Color32::from_rgb(242, 240, 255),
                secondary: Color32::from_rgb(180, 178, 216),
                muted: Color32::from_rgb(122, 120, 162),
                border: Color32::from_rgb(48, 46, 86),
                grid_minor: Color32::from_rgba_unmultiplied(132, 108, 235, 11),
                grid_major: Color32::from_rgba_unmultiplied(78, 180, 235, 10),
            },
        }
    }

    pub fn with_alpha(color: Color32, alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
    }
}

/// Configures typography, spacing, and theme visuals on the egui context without
/// forcing a hardcoded `zoom_factor` (so 1120x740 minimum window size retains full
/// logical layout space across DPI levels).
pub fn apply_native_theme(ctx: &egui::Context, theme_name: &str) -> ThemePalette {
    let palette = ThemePalette::from_name(theme_name);
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();

    style.spacing.item_spacing = egui::Vec2::new(8.0, 8.0);
    style.spacing.button_padding = egui::Vec2::new(11.0, 7.0);
    style.spacing.interact_size.y = 30.0;
    style
        .text_styles
        .insert(egui::TextStyle::Body, FontId::proportional(13.5));
    style
        .text_styles
        .insert(egui::TextStyle::Button, FontId::proportional(12.0));
    style
        .text_styles
        .insert(egui::TextStyle::Small, FontId::proportional(11.0));
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, FontId::monospace(12.0));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, FontId::proportional(21.0));

    let mut visuals = egui::Visuals::dark();
    visuals.dark_mode = true;
    visuals.panel_fill = palette.bg;
    visuals.window_fill = palette.surface;
    visuals.extreme_bg_color = palette.inset;
    visuals.override_text_color = Some(palette.text);
    visuals.widgets.noninteractive.bg_fill = palette.card;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.secondary);
    visuals.widgets.inactive.bg_fill = palette.card;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.secondary);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(Spacing::RADIUS_SM);
    visuals.widgets.hovered.bg_fill = palette.raised;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, palette.accent);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(Spacing::RADIUS_SM);
    visuals.widgets.active.bg_fill = palette.accent_dim;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, palette.accent);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(Spacing::RADIUS_SM);
    visuals.selection.bg_fill = palette.accent_dim;
    visuals.selection.stroke = Stroke::new(1.0, palette.accent);
    visuals.window_stroke = Stroke::new(1.0, palette.border);

    style.visuals = visuals;
    ctx.set_visuals(style.visuals.clone());
    ctx.set_style_of(egui::Theme::Dark, style);
    palette
}

/// Installs the selected bundled proportional font while preserving egui's default
/// symbol/emoji fallback chain.
pub fn install_font_family(ctx: &egui::Context, family: &str) {
    let mut definitions = egui::FontDefinitions::default();
    let (name, data): (&str, &'static [u8]) = match FontChoice::from_str_lossy(family) {
        FontChoice::Outfit => (
            "RiftCore Outfit",
            include_bytes!("../../assets/fonts/Outfit.ttf"),
        ),
        FontChoice::Inter => (
            "RiftCore Inter",
            include_bytes!("../../assets/fonts/Inter.ttf"),
        ),
        FontChoice::PlusJakarta => (
            "RiftCore Plus Jakarta",
            include_bytes!("../../assets/fonts/PlusJakartaSans.ttf"),
        ),
    };
    definitions.font_data.insert(
        name.to_string(),
        std::sync::Arc::new(egui::FontData::from_static(data)),
    );
    definitions
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, name.to_string());
    ctx.set_fonts(definitions);
}

/// Linearly interpolates between two `Color32` values.
pub fn lerp_color(start: Color32, end: Color32, amount: f32) -> Color32 {
    let amount = amount.clamp(0.0, 1.0);
    let channel =
        |from: u8, to: u8| (from as f32 + (to as f32 - from as f32) * amount).round() as u8;
    Color32::from_rgba_unmultiplied(
        channel(start.r(), end.r()),
        channel(start.g(), end.g()),
        channel(start.b(), end.b()),
        channel(start.a(), end.a()),
    )
}

/// Canonical ranked tier badge colors.
pub fn tier_color(tier: &str) -> Color32 {
    match tier.trim().to_ascii_uppercase().as_str() {
        "IRON" => Color32::from_rgb(130, 118, 118),
        "BRONZE" => Color32::from_rgb(184, 128, 92),
        "SILVER" => Color32::from_rgb(176, 196, 210),
        "GOLD" => Color32::from_rgb(238, 190, 74),
        "PLATINUM" => Color32::from_rgb(52, 188, 152),
        "EMERALD" => Color32::from_rgb(38, 206, 128),
        "DIAMOND" => Color32::from_rgb(112, 169, 255),
        "MASTER" => Color32::from_rgb(187, 98, 255),
        "GRANDMASTER" => Color32::from_rgb(243, 84, 84),
        "CHALLENGER" => Color32::from_rgb(252, 209, 92),
        _ => Color32::from_rgb(138, 152, 174),
    }
}

/// Canonical accent color for each League rune tree ID.
pub fn rune_tree_color(tree_id: u32, fallback: Color32) -> Color32 {
    match tree_id {
        TREE_DOMINATION => Color32::from_rgb(238, 105, 113),
        TREE_PRECISION => Color32::from_rgb(231, 189, 105),
        TREE_SORCERY => Color32::from_rgb(111, 181, 246),
        TREE_RESOLVE => Color32::from_rgb(108, 205, 151),
        TREE_INSPIRATION => Color32::from_rgb(195, 143, 236),
        _ => fallback,
    }
}

/// Semantic accent color for each Summoner's Rift role.
pub fn role_color(role: Role, palette: &ThemePalette) -> Color32 {
    match role {
        Role::Top => Color32::from_rgb(198, 128, 232),
        Role::Jungle => palette.green,
        Role::Mid => palette.gold,
        Role::Adc => palette.red,
        Role::Support => palette.accent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_palettes_are_distinct_and_parse_safely() {
        let hex = ThemePalette::from_name("hextech");
        let ember = ThemePalette::from_name("ember");
        let void = ThemePalette::from_name("void");
        let fallback = ThemePalette::from_name("unknown-theme");

        assert_eq!(hex.mode, ThemeMode::Hextech);
        assert_eq!(fallback, hex);
        assert_ne!(hex.accent, ember.accent);
        assert_ne!(hex.accent, void.accent);
        assert_ne!(ember.bg, void.bg);
    }

    #[test]
    fn font_choice_and_rune_colors_resolve_cleanly() {
        assert_eq!(FontChoice::from_str_lossy("outfit"), FontChoice::Outfit);
        assert_eq!(FontChoice::from_str_lossy("INTER"), FontChoice::Inter);
        assert_eq!(FontChoice::from_str_lossy(""), FontChoice::PlusJakarta);

        let fallback = ThemePalette::default().accent;
        assert_ne!(rune_tree_color(TREE_DOMINATION, fallback), fallback);
        assert_ne!(rune_tree_color(TREE_PRECISION, fallback), fallback);
        assert_eq!(rune_tree_color(9999, fallback), fallback);
    }

    #[test]
    fn lerp_color_clamps_endpoints() {
        let a = Color32::from_rgb(0, 0, 0);
        let b = Color32::from_rgb(100, 200, 250);
        assert_eq!(lerp_color(a, b, -1.0), a);
        assert_eq!(lerp_color(a, b, 2.0), b);
        assert_eq!(lerp_color(a, b, 0.5), Color32::from_rgb(50, 100, 125));
    }
}
