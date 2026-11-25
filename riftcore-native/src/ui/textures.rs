//! Lazy on-demand texture decoding and caching manager for champion portraits,
//! rune icons, TFT catalog icons, item icons, and the RiftCore brand mark.

use crate::assets;
use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use std::collections::{HashMap, HashSet};

/// Caps PNG decodes + uploads per rendered frame so scrolling large rosters
/// never blocks the UI thread or drops frame pacing.
const MAX_DECODES_PER_FRAME: usize = 10;

/// Lazy texture cache backed by embedded `crate::assets` byte slices.
pub struct TextureManager {
    champion_sources: HashMap<String, &'static [u8]>,
    tft_sources: HashMap<&'static str, &'static [u8]>,
    rune_sources: HashMap<i64, &'static [u8]>,

    logo: Option<TextureHandle>,
    champions: HashMap<String, TextureHandle>,
    tft: HashMap<String, TextureHandle>,
    runes: HashMap<i64, TextureHandle>,
    items: HashMap<u32, TextureHandle>,

    missing_champions: HashSet<String>,
    missing_tft: HashSet<String>,
    missing_runes: HashSet<i64>,

    decodes_this_frame: usize,
}

impl Default for TextureManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TextureManager {
    /// Builds source lookup tables without decoding any PNG payloads upfront.
    pub fn new() -> Self {
        let mut champion_sources = HashMap::with_capacity(assets::CHAMPION_PORTRAITS.len() * 2);
        for &(key, bytes) in assets::CHAMPION_PORTRAITS {
            champion_sources.insert(key.to_string(), bytes);
            champion_sources.insert(key.to_ascii_lowercase(), bytes);
        }

        let mut tft_sources = HashMap::with_capacity(assets::TFT_ICONS.len());
        for &(key, bytes) in assets::TFT_ICONS {
            tft_sources.insert(key, bytes);
        }

        let mut rune_sources = HashMap::with_capacity(assets::RUNE_ICONS.len());
        for &(id, bytes) in assets::RUNE_ICONS {
            rune_sources.insert(id, bytes);
        }

        Self {
            champion_sources,
            tft_sources,
            rune_sources,
            logo: None,
            champions: HashMap::new(),
            tft: HashMap::new(),
            runes: HashMap::new(),
            items: HashMap::new(),
            missing_champions: HashSet::new(),
            missing_tft: HashSet::new(),
            missing_runes: HashSet::new(),
            decodes_this_frame: 0,
        }
    }

    /// Resets the per-frame PNG decode budget at the start of `eframe::App::ui`.
    pub fn begin_frame(&mut self) {
        self.decodes_this_frame = 0;
    }

    pub fn decodes_this_frame(&self) -> usize {
        self.decodes_this_frame
    }

    pub fn cached_counts(&self) -> (usize, usize, usize, usize) {
        (
            self.champions.len(),
            self.runes.len(),
            self.tft.len(),
            self.items.len(),
        )
    }

    pub fn has_champion_asset(&self, key: &str) -> bool {
        self.champion_bytes(key).is_some()
    }

    pub fn has_rune_asset(&self, id: i64) -> bool {
        self.rune_sources.contains_key(&id)
    }

    pub fn has_tft_asset(&self, image_key: &str) -> bool {
        self.tft_sources.contains_key(image_key)
    }

    /// Returns the RiftCore logo texture, decoding once on first access.
    pub fn logo(&mut self, ctx: &egui::Context) -> Option<&TextureHandle> {
        if self.logo.is_none() {
            self.logo = decode_png_texture(
                ctx,
                "riftcore-logo".to_string(),
                assets::RIFTCORE_LOGO,
                Some(64),
            );
        }
        self.logo.as_ref()
    }

    /// Returns a champion portrait texture, decoding lazily within the frame budget.
    pub fn champion(&mut self, ctx: &egui::Context, key: &str) -> Option<&TextureHandle> {
        let norm = key.trim();
        if norm.is_empty() {
            return None;
        }
        if self.champions.contains_key(norm) {
            return self.champions.get(norm);
        }
        if self.missing_champions.contains(norm) {
            return None;
        }
        let Some(bytes) = self.champion_bytes(norm) else {
            self.missing_champions.insert(norm.to_string());
            return None;
        };
        if self.decodes_this_frame >= MAX_DECODES_PER_FRAME {
            ctx.request_repaint();
            return None;
        }
        self.decodes_this_frame += 1;
        if let Some(handle) = decode_png_texture(ctx, format!("champion-{norm}"), bytes, Some(64)) {
            self.champions.insert(norm.to_string(), handle);
            return self.champions.get(norm);
        }
        self.missing_champions.insert(norm.to_string());
        None
    }

    /// Returns a rune or rune-tree icon texture, decoding lazily within the frame budget.
    pub fn rune(&mut self, ctx: &egui::Context, id: i64) -> Option<&TextureHandle> {
        if self.runes.contains_key(&id) {
            return self.runes.get(&id);
        }
        if self.missing_runes.contains(&id) {
            return None;
        }
        let Some(&bytes) = self.rune_sources.get(&id) else {
            self.missing_runes.insert(id);
            return None;
        };
        if self.decodes_this_frame >= MAX_DECODES_PER_FRAME {
            ctx.request_repaint();
            return None;
        }
        self.decodes_this_frame += 1;
        if let Some(handle) = decode_png_texture(ctx, format!("rune-{id}"), bytes, Some(48)) {
            self.runes.insert(id, handle);
            return self.runes.get(&id);
        }
        self.missing_runes.insert(id);
        None
    }

    /// Returns a TFT unit/trait/item/augment icon texture, decoding lazily within budget.
    pub fn tft(&mut self, ctx: &egui::Context, image_key: &str) -> Option<&TextureHandle> {
        let key = image_key.trim();
        if key.is_empty() {
            return None;
        }
        if self.tft.contains_key(key) {
            return self.tft.get(key);
        }
        if self.missing_tft.contains(key) {
            return None;
        }
        let Some(&bytes) = self.tft_sources.get(key) else {
            self.missing_tft.insert(key.to_string());
            return None;
        };
        if self.decodes_this_frame >= MAX_DECODES_PER_FRAME {
            ctx.request_repaint();
            return None;
        }
        self.decodes_this_frame += 1;
        if let Some(handle) = decode_png_texture(ctx, format!("tft-{key}"), bytes, Some(48)) {
            self.tft.insert(key.to_string(), handle);
            return self.tft.get(key);
        }
        self.missing_tft.insert(key.to_string());
        None
    }

    /// Looks up an optional cached item icon texture (if registered).
    pub fn item(&self, id: u32) -> Option<&TextureHandle> {
        self.items.get(&id)
    }

    /// Registers a decoded item texture into the manager cache.
    pub fn register_item_texture(&mut self, id: u32, handle: TextureHandle) {
        self.items.insert(id, handle);
    }

    fn champion_bytes(&self, key: &str) -> Option<&'static [u8]> {
        self.champion_sources
            .get(key)
            .copied()
            .or_else(|| self.champion_sources.get(&key.to_ascii_lowercase()).copied())
    }
}

/// Decodes the embedded RiftCore logo into raw RGBA pixels for native window/tray icons.
pub fn logo_rgba(size: u32) -> Option<(Vec<u8>, u32, u32)> {
    let img = image::load_from_memory(assets::RIFTCORE_LOGO).ok()?;
    let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
    let rgba = resized.into_rgba8();
    let (w, h) = rgba.dimensions();
    Some((rgba.into_raw(), w, h))
}

fn decode_png_texture(
    ctx: &egui::Context,
    name: String,
    bytes: &[u8],
    size: Option<u32>,
) -> Option<TextureHandle> {
    let image = image::load_from_memory(bytes).ok()?;
    let image = match size {
        Some(target) => {
            if image.width() == target && image.height() == target {
                image
            } else {
                image.resize_exact(target, target, image::imageops::FilterType::Triangle)
            }
        }
        None => image,
    }
    .into_rgba8();
    let dimensions = [image.width() as usize, image.height() as usize];
    let color_image = ColorImage::from_rgba_unmultiplied(dimensions, image.as_raw());
    Some(ctx.load_texture(name, color_image, TextureOptions::LINEAR))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texture_manager_indexes_assets_lazily_without_decoding_upfront() {
        let mgr = TextureManager::new();
        assert_eq!(mgr.cached_counts(), (0, 0, 0, 0));
        assert!(mgr.has_champion_asset("Ahri"));
        assert!(mgr.has_champion_asset("fiddlesticks"));
        assert!(mgr.has_rune_asset(8140));
        assert!(mgr.has_rune_asset(8112));
        assert!(!mgr.has_champion_asset("NonExistentChampionXYZ"));
    }

    #[test]
    fn logo_rgba_decodes_cleanly() {
        let (rgba, w, h) = logo_rgba(32).expect("embedded logo must decode");
        assert_eq!((w, h), (32, 32));
        assert_eq!(rgba.len(), 32 * 32 * 4);
    }
}
