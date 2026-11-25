//! League of Legends rune tree metadata, stat shards, and LCU rune page builder.
//!
//! Replaces legacy `"Eyeball Collection"` references with the current Domination
//! slot-2 rune `8140` (`"Grisly Mementos"`).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Rune tree IDs from Riot Data Dragon `runesReforged.json`.
pub const TREE_PRECISION: u32 = 8000;
pub const TREE_DOMINATION: u32 = 8100;
pub const TREE_SORCERY: u32 = 8200;
pub const TREE_INSPIRATION: u32 = 8300;
pub const TREE_RESOLVE: u32 = 8400;

/// Domination slot-2 rune ID (replaces legacy Eyeball Collection).
pub const RUNE_GRISLY_MEMENTOS: i64 = 8140;

/// Default adaptive force shard fallback ID.
pub const SHARD_ADAPTIVE_FORCE: i64 = 5008;
/// Attack speed offense shard ID.
pub const SHARD_ATTACK_SPEED: i64 = 5005;
/// Ability haste offense shard ID.
pub const SHARD_ABILITY_HASTE: i64 = 5007;
/// Move speed flex shard ID.
pub const SHARD_MOVE_SPEED: i64 = 5010;
/// Scaling health shard ID.
pub const SHARD_SCALING_HEALTH: i64 = 5001;
/// Flat health defense shard ID.
pub const SHARD_FLAT_HEALTH: i64 = 5011;
/// Tenacity & slow resist defense shard ID.
pub const SHARD_TENACITY: i64 = 5013;

/// Single rune choice inside a tree slot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuneInfo {
    pub id: i64,
    pub key: String,
    pub icon: String,
    pub name: String,
    pub short_desc: String,
    pub long_desc: String,
}

/// One row (tier) inside a rune tree (`slots[0]` = Keystones, `slots[1..=3]` = minor runes).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuneSlot {
    pub runes: Vec<RuneInfo>,
}

/// Complete rune path (e.g. Precision `8000`, Domination `8100`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuneTree {
    pub id: u32,
    pub key: String,
    pub icon: String,
    pub name: String,
    pub slots: Vec<RuneSlot>,
}

/// Stat shard option for rows 0..=2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatShard {
    pub id: i64,
    pub name: &'static str,
}

/// Typed LCU `/lol-perks/v1/pages` payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunePagePayload {
    pub name: String,
    pub primary_style_id: u32,
    pub sub_style_id: u32,
    pub selected_perk_ids: Vec<i64>,
    pub current: bool,
}

impl RunePagePayload {
    /// Converts this rune page payload to a `serde_json::Value` for LCU submission.
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

struct RuneCatalog {
    trees: Vec<RuneTree>,
    by_tree_id: HashMap<u32, usize>,
    by_rune_id: HashMap<i64, (u32, usize, usize)>,
}

static CATALOG: LazyLock<RuneCatalog> = LazyLock::new(load_catalog);

/// Canonical display names for the 5 rune trees.
pub const RUNE_TREE_NAMES: &[(u32, &str)] = &[
    (TREE_PRECISION, "Precision"),
    (TREE_DOMINATION, "Domination"),
    (TREE_SORCERY, "Sorcery"),
    (TREE_RESOLVE, "Resolve"),
    (TREE_INSPIRATION, "Inspiration"),
];

/// Returns all 5 rune trees parsed from `runesReforged.json`.
pub fn all_trees() -> &'static [RuneTree] {
    &CATALOG.trees
}

/// Looks up a rune tree by its style ID (e.g. `8100` for Domination).
pub fn tree_by_id(id: u32) -> Option<&'static RuneTree> {
    CATALOG
        .by_tree_id
        .get(&id)
        .and_then(|&idx| CATALOG.trees.get(idx))
}

/// Returns the display name for a rune tree ID (defaults to `"Precision"`).
pub fn tree_name(id: u32) -> &'static str {
    tree_by_id(id)
        .map(|tree| tree.name.as_str())
        .or_else(|| {
            RUNE_TREE_NAMES
                .iter()
                .find(|(tree_id, _)| *tree_id == id)
                .map(|(_, name)| *name)
        })
        .unwrap_or("Precision")
}

/// Looks up a rune by its numeric perk ID.
pub fn rune_by_id(id: i64) -> Option<&'static RuneInfo> {
    let &(tree_idx, row_idx, rune_idx) = CATALOG.by_rune_id.get(&id)?;
    CATALOG
        .trees
        .get(tree_idx as usize)?
        .slots
        .get(row_idx)?
        .runes
        .get(rune_idx)
}

/// Returns all `(id, name)` options in `tree_id` at slot `row` (`0` = keystone, `1..=3` = minor).
pub fn rune_options(tree_id: u32, row: usize) -> Vec<(i64, &'static str)> {
    tree_by_id(tree_id)
        .and_then(|tree| tree.slots.get(row))
        .map(|slot| {
            slot.runes
                .iter()
                .map(|rune| (rune.id, rune.name.as_str()))
                .collect()
        })
        .unwrap_or_default()
}

/// Finds the row index (`0..=3`) of `rune_id` inside `tree_id`.
pub fn rune_row(tree_id: u32, rune_id: i64) -> Option<usize> {
    tree_by_id(tree_id)?
        .slots
        .iter()
        .position(|slot| slot.runes.iter().any(|rune| rune.id == rune_id))
}

/// Resolves a preferred rune name within `(tree_id, row)`, mapping legacy
/// `"Eyeball Collection"` queries cleanly to `"Grisly Mementos"` (`8140`).
pub fn rune_id_preferred(tree_id: u32, row: usize, preferred_names: &[&str]) -> i64 {
    let options = rune_options(tree_id, row);
    for preferred in preferred_names {
        let lower = preferred.trim().to_ascii_lowercase();
        let needle = if lower == "eyeball collection" {
            "grisly mementos"
        } else {
            lower.as_str()
        };
        if let Some((id, _)) = options
            .iter()
            .find(|(_, name)| name.to_ascii_lowercase().contains(needle))
        {
            return *id;
        }
    }
    options
        .first()
        .map(|(id, _)| *id)
        .unwrap_or(SHARD_ADAPTIVE_FORCE)
}

/// Returns the stat shard options for shard row `0` (Offense), `1` (Flex), or `2` (Defense).
pub fn stat_shard_options(row: usize) -> &'static [StatShard] {
    match row {
        0 => &[
            StatShard {
                id: SHARD_ADAPTIVE_FORCE,
                name: "Adaptive Force",
            },
            StatShard {
                id: SHARD_ATTACK_SPEED,
                name: "Attack Speed",
            },
            StatShard {
                id: SHARD_ABILITY_HASTE,
                name: "Ability Haste",
            },
        ],
        1 => &[
            StatShard {
                id: SHARD_ADAPTIVE_FORCE,
                name: "Adaptive Force",
            },
            StatShard {
                id: SHARD_MOVE_SPEED,
                name: "Move Speed",
            },
            StatShard {
                id: SHARD_SCALING_HEALTH,
                name: "Scaling Health",
            },
        ],
        _ => &[
            StatShard {
                id: SHARD_FLAT_HEALTH,
                name: "Flat Health",
            },
            StatShard {
                id: SHARD_TENACITY,
                name: "Tenacity / Slow Resist",
            },
            StatShard {
                id: SHARD_SCALING_HEALTH,
                name: "Scaling Health",
            },
        ],
    }
}

/// Returns the standard 9-perk Burst Domination/Sorcery preset using `Grisly Mementos` (`8140`).
pub fn burst_preset_perks() -> [i64; 9] {
    [
        rune_id_preferred(TREE_DOMINATION, 0, &["Electrocute"]),
        rune_id_preferred(TREE_DOMINATION, 1, &["Sudden Impact"]),
        rune_id_preferred(TREE_DOMINATION, 2, &["Grisly Mementos"]),
        rune_id_preferred(TREE_DOMINATION, 3, &["Treasure Hunter", "Ultimate Hunter"]),
        rune_id_preferred(TREE_SORCERY, 1, &["Manaflow Band"]),
        rune_id_preferred(TREE_SORCERY, 2, &["Transcendence"]),
        SHARD_ADAPTIVE_FORCE,
        SHARD_ADAPTIVE_FORCE,
        SHARD_SCALING_HEALTH,
    ]
}

/// Computes the two distinct minor rows (`1..=3`) for a pair of secondary perks.
pub fn secondary_rows_for_perks(tree_id: u32, first: i64, second: i64) -> [usize; 2] {
    let first_row = rune_row(tree_id, first).filter(|row| *row > 0).unwrap_or(1);
    let second_row = rune_row(tree_id, second)
        .filter(|row| *row > 0 && *row != first_row)
        .unwrap_or_else(|| (1..=3).find(|row| *row != first_row).unwrap_or(2));
    [first_row, second_row]
}

/// Ensures all 9 selected perk slots match valid options in the chosen trees and shard rows.
pub fn normalize_rune_setup(
    primary_tree: u32,
    secondary_tree: u32,
    secondary_rows: &mut [usize; 2],
    perks: &mut [i64; 9],
) {
    for (row, perk) in perks.iter_mut().enumerate().take(4) {
        let options = rune_options(primary_tree, row);
        if !options.iter().any(|(id, _)| *id == *perk) {
            *perk = options
                .first()
                .map(|(id, _)| *id)
                .unwrap_or(SHARD_ADAPTIVE_FORCE);
        }
    }

    secondary_rows[0] = secondary_rows[0].clamp(1, 3);
    secondary_rows[1] = secondary_rows[1].clamp(1, 3);
    if secondary_rows[0] == secondary_rows[1] {
        secondary_rows[1] = (1..=3).find(|row| *row != secondary_rows[0]).unwrap_or(2);
    }

    for (&row, perk) in secondary_rows.iter().zip(perks[4..6].iter_mut()) {
        let options = rune_options(secondary_tree, row);
        if !options.iter().any(|(id, _)| *id == *perk) {
            *perk = options.first().map(|(id, _)| *id).unwrap_or(9111);
        }
    }

    for row in 0..3 {
        let slot = 6 + row;
        let options = stat_shard_options(row);
        if !options.iter().any(|shard| shard.id == perks[slot]) {
            perks[slot] = options
                .first()
                .map(|shard| shard.id)
                .unwrap_or(SHARD_ADAPTIVE_FORCE);
        }
    }
}

/// Canonical LCU rune page payload builder used across Champ Select, Builds, and Modes.
pub fn build_rune_page(
    page_name: &str,
    primary_tree: u32,
    secondary_tree: u32,
    perks: &[i64],
) -> RunePagePayload {
    RunePagePayload {
        name: page_name.to_string(),
        primary_style_id: primary_tree,
        sub_style_id: secondary_tree,
        selected_perk_ids: perks.to_vec(),
        current: true,
    }
}

/// Strips Riot tooltip XML/HTML markup from rune `shortDesc` / `longDesc` strings.
pub fn clean_rune_description(id: i64, max_chars: usize) -> String {
    let Some(rune) = rune_by_id(id) else {
        return String::new();
    };
    let mut clean = String::with_capacity(rune.short_desc.len());
    let mut in_tag = false;
    for ch in rune.short_desc.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => clean.push(ch),
            _ => {}
        }
    }
    let collapsed = clean
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let limit = max_chars.max(4);
    if collapsed.chars().count() <= limit {
        collapsed
    } else {
        collapsed
            .chars()
            .take(limit.saturating_sub(1))
            .chain(std::iter::once('…'))
            .collect()
    }
}

fn load_catalog() -> RuneCatalog {
    let trees: Vec<RuneTree> =
        serde_json::from_str(include_str!("../../assets/data/runesReforged.json"))
            .unwrap_or_default();

    let mut by_tree_id = HashMap::with_capacity(trees.len());
    let mut by_rune_id = HashMap::new();
    for (tree_idx, tree) in trees.iter().enumerate() {
        by_tree_id.insert(tree.id, tree_idx);
        for (row_idx, slot) in tree.slots.iter().enumerate() {
            for (rune_idx, rune) in slot.runes.iter().enumerate() {
                by_rune_id.insert(rune.id, (tree_idx as u32, row_idx, rune_idx));
            }
        }
    }

    RuneCatalog {
        trees,
        by_tree_id,
        by_rune_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rune_8140_is_grisly_mementos_not_eyeball_collection() {
        let rune = rune_by_id(RUNE_GRISLY_MEMENTOS).expect("Rune 8140 must exist");
        assert_eq!(rune.id, 8140);
        assert_eq!(rune.name, "Grisly Mementos");
        assert_ne!(rune.name, "Eyeball Collection");

        assert_eq!(
            rune_id_preferred(TREE_DOMINATION, 2, &["Grisly Mementos"]),
            8140
        );
        // Legacy callers passing "Eyeball Collection" also resolve safely to 8140
        assert_eq!(
            rune_id_preferred(TREE_DOMINATION, 2, &["Eyeball Collection"]),
            8140
        );

        let burst = burst_preset_perks();
        assert_eq!(burst[2], 8140);
    }

    #[test]
    fn rune_trees_and_page_builder_work() {
        assert_eq!(all_trees().len(), 5);
        assert_eq!(tree_name(TREE_DOMINATION), "Domination");
        let page = build_rune_page(
            "RiftCore Ahri MID",
            TREE_DOMINATION,
            TREE_SORCERY,
            &[8112, 8139, 8140, 8106, 8226, 8210, 5008, 5008, 5001],
        );
        assert_eq!(page.primary_style_id, 8100);
        assert_eq!(page.sub_style_id, 8200);
        assert_eq!(page.selected_perk_ids.len(), 9);
        assert!(page.current);
    }
}
