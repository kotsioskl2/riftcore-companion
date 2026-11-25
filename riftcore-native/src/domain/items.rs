//! League of Legends item catalog parsed from Riot Data Dragon `item.json`.
//!
//! Exposes full item metadata (gold costs, build tree `from`/`into`, map
//! availability, and tags), ID lookup helpers, and canonical Trinket checks.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::sync::LazyLock;

/// Data Dragon map ID for Summoner's Rift.
pub const MAP_SUMMONERS_RIFT: u32 = 11;
/// Data Dragon map ID for Howling Abyss (ARAM).
pub const MAP_ARAM: u32 = 12;
/// Data Dragon map ID for Arena.
pub const MAP_ARENA: u32 = 30;

/// Stealth Ward (Yellow Trinket).
pub const TRINKET_STEALTH_WARD: u32 = 3340;
/// Farsight Alteration (Blue Trinket).
pub const TRINKET_FARSIGHT_ALTERATION: u32 = 3363;
/// Oracle Lens (Red Sweeper Trinket).
pub const TRINKET_ORACLE_LENS: u32 = 3364;
/// Fiddlesticks passive Scarecrow Effigy trinket.
pub const TRINKET_SCARECROW_EFFIGY: u32 = 3330;
/// Arena Arcane Sweeper trinket.
pub const TRINKET_ARCANE_SWEEPER: u32 = 3348;

/// Standard slot-6 vision trinket item IDs.
pub const STANDARD_TRINKET_IDS: &[u32] = &[
    TRINKET_STEALTH_WARD,
    TRINKET_FARSIGHT_ALTERATION,
    TRINKET_ORACLE_LENS,
    TRINKET_SCARECROW_EFFIGY,
    TRINKET_ARCANE_SWEEPER,
];

/// Gold pricing and shop purchasability metadata for an item.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemGold {
    pub base: u32,
    pub total: u32,
    pub sell: u32,
    pub purchasable: bool,
}

/// Complete League item record loaded from Data Dragon `item.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub plaintext: String,
    pub description: String,
    pub gold: ItemGold,
    pub from: Vec<u32>,
    pub into: Vec<u32>,
    pub maps: BTreeMap<u32, bool>,
    pub tags: Vec<String>,
    pub depth: u32,
    pub in_store: bool,
    pub hide_from_all: bool,
    pub required_champion: Option<String>,
    pub special_recipe: u32,
}

impl Item {
    /// Returns `true` if this item is enabled on the given Data Dragon `map_id`.
    pub fn available_on_map(&self, map_id: u32) -> bool {
        self.maps.get(&map_id).copied().unwrap_or(false)
    }

    /// Returns `true` if this item carries `tag` (case-insensitive).
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(tag))
    }

    /// Returns `true` if this item is an equipped vision trinket or has the `Trinket` tag.
    pub fn is_trinket(&self) -> bool {
        is_trinket_id(self.id) || self.has_tag("Trinket")
    }
}

struct ItemCatalog {
    version: String,
    by_id: HashMap<u32, Item>,
    ordered: Vec<Item>,
}

static CATALOG: LazyLock<ItemCatalog> = LazyLock::new(load_catalog);

#[derive(Deserialize)]
struct RawItemFile {
    #[serde(default)]
    version: String,
    #[serde(default)]
    data: HashMap<String, RawItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawItem {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    plaintext: String,
    #[serde(default)]
    gold: RawGold,
    #[serde(default)]
    from: Vec<String>,
    #[serde(default)]
    into: Vec<String>,
    #[serde(default)]
    maps: HashMap<String, bool>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    depth: u32,
    in_store: Option<bool>,
    #[serde(default)]
    hide_from_all: bool,
    required_champion: Option<String>,
    #[serde(default)]
    special_recipe: u32,
}

#[derive(Default, Deserialize)]
struct RawGold {
    #[serde(default)]
    base: u32,
    #[serde(default)]
    total: u32,
    #[serde(default)]
    sell: u32,
    #[serde(default)]
    purchasable: bool,
}

/// Returns the Data Dragon patch version embedded in `item.json` (e.g. `"16.22.1"`).
pub fn data_version() -> &'static str {
    &CATALOG.version
}

/// Returns all named items sorted by numeric item ID.
pub fn all_items() -> &'static [Item] {
    &CATALOG.ordered
}

/// Looks up full item metadata by numeric Riot item ID.
pub fn item_by_id(id: u32) -> Option<&'static Item> {
    CATALOG.by_id.get(&id)
}

/// Returns the item's localized name or `"Item {id}"` if unknown.
pub fn item_name(id: u32) -> String {
    item_by_id(id)
        .map(|item| item.name.clone())
        .unwrap_or_else(|| format!("Item {id}"))
}

/// Returns `true` if `id` exists in the embedded Data Dragon item catalog.
pub fn item_available(id: u32) -> bool {
    CATALOG.by_id.contains_key(&id)
}

/// Returns `true` if `id` exists and is enabled on `map_id` (e.g. `11` for Summoner's Rift).
pub fn item_available_on_map(id: u32, map_id: u32) -> bool {
    item_by_id(id)
        .map(|item| item.available_on_map(map_id))
        .unwrap_or(false)
}

/// Returns the total gold cost of `id` if present in `item.json`.
pub fn item_total_cost(id: u32) -> Option<u32> {
    item_by_id(id).map(|item| item.gold.total)
}

/// Returns `true` if `id` is a slot-6 vision trinket (`3340`, `3363`, `3364`, `3330`, `3348`,
/// or mode variants `773340`/`773348`).
pub fn is_trinket_id(id: u32) -> bool {
    STANDARD_TRINKET_IDS.contains(&id) || matches!(id, 773340 | 773348)
}

/// Returns `true` if `id` is a standard vision trinket OR carries Data Dragon's `"Trinket"` tag.
pub fn has_trinket_tag(id: u32) -> bool {
    is_trinket_id(id)
        || item_by_id(id)
            .map(|item| item.has_tag("Trinket"))
            .unwrap_or(false)
}

/// Strips HTML/Riot markup tags from raw Data Dragon item descriptions.
pub fn sanitize_description(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut in_tag = false;
    for ch in raw.chars() {
        match ch {
            '<' => {
                in_tag = true;
                if !out.ends_with(' ') && !out.is_empty() {
                    out.push(' ');
                }
            }
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn load_catalog() -> ItemCatalog {
    let parsed: RawItemFile = serde_json::from_str(include_str!("../../assets/data/item.json"))
        .unwrap_or_else(|_| {
            // Fallback in case JSON structure shifts partially
            let root: Value = serde_json::from_str(include_str!("../../assets/data/item.json"))
                .unwrap_or(Value::Null);
            RawItemFile {
                version: root
                    .get("version")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                data: HashMap::new(),
            }
        });

    let mut by_id = HashMap::with_capacity(parsed.data.len());
    let mut ordered = Vec::with_capacity(parsed.data.len());

    for (raw_id, raw) in parsed.data {
        let Ok(id) = raw_id.parse::<u32>() else {
            continue;
        };
        let name = raw.name.trim().to_string();
        if name.is_empty() {
            continue;
        }
        let from = raw
            .from
            .iter()
            .filter_map(|part| part.parse::<u32>().ok())
            .collect();
        let into = raw
            .into
            .iter()
            .filter_map(|part| part.parse::<u32>().ok())
            .collect();
        let maps = raw
            .maps
            .iter()
            .filter_map(|(k, &v)| k.parse::<u32>().ok().map(|map_id| (map_id, v)))
            .collect();

        let item = Item {
            id,
            name,
            plaintext: raw.plaintext,
            description: raw.description,
            gold: ItemGold {
                base: raw.gold.base,
                total: raw.gold.total,
                sell: raw.gold.sell,
                purchasable: raw.gold.purchasable,
            },
            from,
            into,
            maps,
            tags: raw.tags,
            depth: raw.depth,
            in_store: raw.in_store.unwrap_or(true),
            hide_from_all: raw.hide_from_all,
            required_champion: raw.required_champion,
            special_recipe: raw.special_recipe,
        };
        by_id.insert(id, item.clone());
        ordered.push(item);
    }

    ordered.sort_by_key(|item| item.id);

    ItemCatalog {
        version: parsed.version,
        by_id,
        ordered,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_metadata_parses_gold_from_maps_and_tags() {
        assert_eq!(data_version(), "16.22.1");
        assert!(all_items().len() > 500, "item.json must parse full catalog");

        let ie = item_by_id(3031).expect("Infinity Edge (3031) must exist");
        assert_eq!(ie.name, "Infinity Edge");
        assert_eq!(ie.gold.total, 3500);
        assert!(ie.gold.purchasable);
        assert!(ie.available_on_map(MAP_SUMMONERS_RIFT));
        assert!(ie.has_tag("CriticalStrike"));
        assert!(!ie.from.is_empty());
        assert!(ie.from.contains(&1038)); // B. F. Sword

        // Verify corrected Support/Jungle/Bruiser items from item.json
        assert_eq!(item_name(3002), "Trailblazer");
        assert_eq!(item_name(2503), "Blackfire Torch");
        assert_eq!(item_name(3869), "Celestial Opposition");
        assert_eq!(item_name(3870), "Dream Maker");
        assert_eq!(item_name(6610), "Sundered Sky");
    }

    #[test]
    fn trinket_helpers_match_vision_trinkets() {
        for &id in STANDARD_TRINKET_IDS {
            assert!(is_trinket_id(id), "expected trinket {id}");
            assert!(has_trinket_tag(id), "expected trinket tag {id}");
            assert!(item_available(id), "expected trinket {id} in item.json");
        }
        assert!(!is_trinket_id(3031));
        assert!(!is_trinket_id(2055)); // Control Ward is a consumable vision item, not slot-6 trinket
    }
}
