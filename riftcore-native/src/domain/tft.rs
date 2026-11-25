//! Teamfight Tactics (TFT) Set 18 catalog and Set 13 legacy archive data.
//!
//! Loads current-set champions, traits, items, and augments from embedded Data
//! Dragon JSONs while keeping Set 13 archive guides free of fabricated win rates.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::LazyLock;

/// One TFT unit, trait, item, or augment entry for the active set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TftEntry {
    pub id: String,
    pub name: String,
    pub cost: Option<u32>,
    pub tier: Option<u32>,
    pub description: String,
    pub image_key: String,
}

/// Summary metadata for the embedded active TFT set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TftSetMetadata {
    pub set: u32,
    pub patch_version: String,
    pub champion_count: usize,
    pub trait_count: usize,
    pub item_count: usize,
    pub augment_count: usize,
}

/// Archived Set 13 shop roll probability row.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TftRollOddsRow {
    pub level: u32,
    pub cost1: u32,
    pub cost2: u32,
    pub cost3: u32,
    pub cost4: u32,
    pub cost5: u32,
    pub recommended_round: String,
    pub xp_to_next: u32,
    pub gold_cost_estimate: u32,
}

/// Archived Set 13 base item component.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TftComponent {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub stats: String,
    pub description: String,
}

/// Archived Set 13 combined item recipe.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TftCombinedRecipe {
    pub id: String,
    pub name: String,
    pub component1: String,
    pub component2: String,
    pub stats: String,
    pub description: String,
    pub category: String,
    pub best_champions: Vec<String>,
}

struct TftCatalog {
    metadata: TftSetMetadata,
    champions: Vec<TftEntry>,
    traits: Vec<TftEntry>,
    items: Vec<TftEntry>,
    augments: Vec<TftEntry>,
}

static CATALOG: LazyLock<TftCatalog> = LazyLock::new(load_catalog);
static LEGACY_DATA: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../assets/data/tft_legacy.json")).unwrap_or(Value::Null)
});

/// Returns the active TFT set number detected from `tft-champion.json` (Set 18).
pub fn current_set() -> u32 {
    CATALOG.metadata.set
}

/// Returns summary metadata for the active embedded TFT catalog.
pub fn set_metadata() -> &'static TftSetMetadata {
    &CATALOG.metadata
}

/// Returns all active-set TFT units with `cost > 0`.
pub fn champions() -> &'static [TftEntry] {
    &CATALOG.champions
}

/// Returns all active-set TFT traits.
pub fn traits() -> &'static [TftEntry] {
    &CATALOG.traits
}

/// Returns all active-set TFT items.
pub fn items() -> &'static [TftEntry] {
    &CATALOG.items
}

/// Returns all active-set TFT augments.
pub fn augments() -> &'static [TftEntry] {
    &CATALOG.augments
}

/// Looks up a trait in the current set by ID or case-insensitive name.
pub fn trait_by_id_or_name(query: &str) -> Option<&'static TftEntry> {
    let needle = query.trim();
    CATALOG.traits.iter().find(|entry| {
        entry.id.eq_ignore_ascii_case(needle) || entry.name.eq_ignore_ascii_case(needle)
    })
}

/// Looks up an augment in the current set by ID or case-insensitive name.
pub fn augment_by_id_or_name(query: &str) -> Option<&'static TftEntry> {
    let needle = query.trim();
    CATALOG.augments.iter().find(|entry| {
        entry.id.eq_ignore_ascii_case(needle) || entry.name.eq_ignore_ascii_case(needle)
    })
}

/// Returns the archived Set 13 JSON payload for legacy reference tabs.
pub fn legacy_data() -> &'static Value {
    &LEGACY_DATA
}

/// Returns typed Set 13 archived base components.
pub fn legacy_components() -> Vec<TftComponent> {
    LEGACY_DATA
        .get("TFT_COMPONENTS")
        .cloned()
        .and_then(|val| serde_json::from_value(val).ok())
        .unwrap_or_default()
}

/// Returns typed Set 13 archived combined item recipes.
pub fn legacy_recipes() -> Vec<TftCombinedRecipe> {
    LEGACY_DATA
        .get("TFT_COMBINED_ITEMS")
        .cloned()
        .and_then(|val| serde_json::from_value(val).ok())
        .unwrap_or_default()
}

/// Returns typed Set 13 archived shop roll odds.
pub fn legacy_roll_odds() -> Vec<TftRollOddsRow> {
    LEGACY_DATA
        .get("TFT_ROLL_ODDS")
        .cloned()
        .and_then(|val| serde_json::from_value(val).ok())
        .unwrap_or_default()
}

fn load_catalog() -> TftCatalog {
    let champions_json: Value =
        serde_json::from_str(include_str!("../../assets/data/tft-champion.json"))
            .unwrap_or(Value::Null);
    let patch_version = champions_json
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let raw_champions = entries(&champions_json);
    let set = raw_champions
        .iter()
        .filter_map(|(key, _)| set_from_key(key))
        .max()
        .unwrap_or(0);
    let marker = format!("TFTSet{set}");
    let set_token = format!("_{set}_");

    let mut champions: Vec<TftEntry> = raw_champions
        .into_iter()
        .filter(|(key, value)| {
            key.contains(&marker) && value.get("cost").and_then(Value::as_u64).unwrap_or(0) > 0
        })
        .filter_map(|(_, value)| entry(&value, "champion"))
        .collect();
    champions.sort_by(|a, b| a.cost.cmp(&b.cost).then_with(|| a.name.cmp(&b.name)));

    let traits_json: Value = serde_json::from_str(include_str!("../../assets/data/tft-trait.json"))
        .unwrap_or(Value::Null);
    let mut traits: Vec<TftEntry> = entries(&traits_json)
        .into_iter()
        .filter(|(_, value)| {
            let id = value.get("id").and_then(Value::as_str).unwrap_or_default();
            let image = value
                .get("image")
                .and_then(|image| image.get("full"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            id.contains(&set_token) || image.contains(&set_token)
        })
        .filter_map(|(_, value)| entry(&value, "trait"))
        .collect();
    traits.sort_by(|a, b| a.name.cmp(&b.name));

    let items_json: Value = serde_json::from_str(include_str!("../../assets/data/tft-item.json"))
        .unwrap_or(Value::Null);
    let mut items: Vec<TftEntry> = entries(&items_json)
        .into_iter()
        .filter(|(key, _)| key.contains(&marker))
        .filter_map(|(_, value)| entry(&value, "item"))
        .collect();
    items.sort_by(|a, b| a.name.cmp(&b.name));

    let augments_json: Value =
        serde_json::from_str(include_str!("../../assets/data/tft-augments.json"))
            .unwrap_or(Value::Null);
    let mut augments: Vec<TftEntry> = entries(&augments_json)
        .into_iter()
        .filter(|(_, value)| {
            let id = value.get("id").and_then(Value::as_str).unwrap_or_default();
            id.contains(&set_token)
        })
        .filter_map(|(_, value)| entry(&value, "augment"))
        .collect();
    augments.sort_by(|a, b| a.name.cmp(&b.name));

    let metadata = TftSetMetadata {
        set,
        patch_version,
        champion_count: champions.len(),
        trait_count: traits.len(),
        item_count: items.len(),
        augment_count: augments.len(),
    };

    TftCatalog {
        metadata,
        champions,
        traits,
        items,
        augments,
    }
}

fn entries(root: &Value) -> Vec<(String, Value)> {
    root.get("data")
        .and_then(Value::as_object)
        .map(|data| {
            data.iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn entry(value: &Value, kind: &str) -> Option<TftEntry> {
    let id = value.get("id")?.as_str()?.to_owned();
    let cost = value.get("cost").and_then(Value::as_u64).map(|v| v as u32);
    let tier = value.get("tier").and_then(Value::as_u64).map(|v| v as u32);
    Some(TftEntry {
        image_key: format!("{kind}_{}", safe_asset_key(&id)),
        id,
        name: value.get("name")?.as_str()?.to_owned(),
        cost: cost.or(tier),
        tier,
        description: value
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
    })
}

fn safe_asset_key(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn set_from_key(key: &str) -> Option<u32> {
    let start = key.find("TFTSet")? + "TFTSet".len();
    let digits: String = key[start..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_set_is_18_with_traits_and_augments() {
        assert_eq!(current_set(), 18);
        let meta = set_metadata();
        assert_eq!(meta.set, 18);
        assert_eq!(meta.patch_version, "16.22.1");
        assert!(!champions().is_empty());
        assert!(!traits().is_empty());
        assert!(!items().is_empty());
        assert!(!augments().is_empty());

        assert!(trait_by_id_or_name("Adaptor").is_some());
        assert!(augment_by_id_or_name("Big Grab Bag").is_some());
    }

    #[test]
    fn legacy_set_13_tables_load_cleanly() {
        assert!(!legacy_components().is_empty());
        assert!(!legacy_recipes().is_empty());
        assert_eq!(legacy_roll_odds().len(), 10);
    }
}
