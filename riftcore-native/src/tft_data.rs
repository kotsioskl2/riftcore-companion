use serde_json::Value;
use std::sync::LazyLock;

#[derive(Clone, Debug)]
pub struct TftEntry {
    pub id: String,
    pub name: String,
    pub cost: Option<u32>,
    pub description: String,
    pub image_key: String,
}

struct TftCatalog {
    set: u32,
    champions: Vec<TftEntry>,
    traits: Vec<TftEntry>,
    items: Vec<TftEntry>,
    augments: Vec<TftEntry>,
}

static CATALOG: LazyLock<TftCatalog> = LazyLock::new(load_catalog);
static LEGACY_DATA: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../assets/data/tft_legacy.json")).unwrap_or(Value::Null)
});

pub fn current_set() -> u32 {
    CATALOG.set
}

pub fn champions() -> &'static [TftEntry] {
    &CATALOG.champions
}

pub fn traits() -> &'static [TftEntry] {
    &CATALOG.traits
}

pub fn items() -> &'static [TftEntry] {
    &CATALOG.items
}

pub fn augments() -> &'static [TftEntry] {
    &CATALOG.augments
}

pub fn legacy_data() -> &'static Value {
    &LEGACY_DATA
}

fn load_catalog() -> TftCatalog {
    let champions_json: Value =
        serde_json::from_str(include_str!("../assets/data/tft-champion.json"))
            .unwrap_or(Value::Null);
    let raw_champions = entries(&champions_json);
    let set = raw_champions
        .iter()
        .filter_map(|(key, _)| set_from_key(key))
        .max()
        .unwrap_or(0);
    let marker = format!("TFTSet{set}");
    let champions = raw_champions
        .into_iter()
        .filter(|(key, value)| {
            key.contains(&marker) && value.get("cost").and_then(Value::as_u64).unwrap_or(0) > 0
        })
        .filter_map(|(_, value)| entry(&value, "champion"))
        .collect();

    let traits_json: Value =
        serde_json::from_str(include_str!("../assets/data/tft-trait.json")).unwrap_or(Value::Null);
    let traits = entries(&traits_json)
        .into_iter()
        .filter(|(_, value)| {
            let id = value.get("id").and_then(Value::as_str).unwrap_or_default();
            let image = value
                .get("image")
                .and_then(|image| image.get("full"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            id.contains(&format!("_{set}_")) || image.contains(&format!("_{set}_"))
        })
        .filter_map(|(_, value)| entry(&value, "trait"))
        .collect();

    let items_json: Value =
        serde_json::from_str(include_str!("../assets/data/tft-item.json")).unwrap_or(Value::Null);
    let items = entries(&items_json)
        .into_iter()
        .filter(|(key, _)| key.contains(&marker))
        .filter_map(|(_, value)| entry(&value, "item"))
        .collect();

    let augments_json: Value =
        serde_json::from_str(include_str!("../assets/data/tft-augments.json"))
            .unwrap_or(Value::Null);
    let augments = entries(&augments_json)
        .into_iter()
        .filter(|(_, value)| {
            let id = value.get("id").and_then(Value::as_str).unwrap_or_default();
            id.contains(&format!("_{set}_"))
        })
        .filter_map(|(_, value)| entry(&value, "augment"))
        .collect();

    TftCatalog {
        set,
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
    Some(TftEntry {
        image_key: format!("{kind}_{}", safe_asset_key(&id)),
        id,
        name: value.get("name")?.as_str()?.to_owned(),
        cost: value
            .get("cost")
            .and_then(Value::as_u64)
            .map(|cost| cost as u32)
            .or_else(|| {
                value
                    .get("tier")
                    .and_then(Value::as_u64)
                    .map(|tier| tier as u32)
            }),
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
