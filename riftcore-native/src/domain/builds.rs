//! Role-aware champion build profiles, presets, skill orders, and LCU item sets.
//!
//! Combines `assets/champion_build_profiles.json` with profiles for the 4 new
//! champions (`Locke`, `Mel`, `Yunara`, `Zaahen`), fixes Jungle/Support item
//! and spell presets (including `3869` Celestial Opposition vs `3870` Dream
//! Maker, `3002` Trailblazer vs `2503` Blackfire Torch, and `6610` Sundered Sky
//! vs `6694` Serylda's Grudge), and fixes Udyr's `"Q > R > E"` skill order.

use crate::domain::champions::{self, Champion, Role};
use crate::domain::items::{self, MAP_SUMMONERS_RIFT};
use crate::domain::runes::{self, RunePagePayload};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Champion combat/build archetype driving baseline item, rune, and spell presets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Archetype {
    BurstMage,
    BattleMage,
    ApAssassin,
    AdAssassin,
    CritMarksman,
    #[serde(rename = "ONHIT_MARKSMAN")]
    OnHitMarksman,
    Juggernaut,
    DiverFighter,
    Skirmisher,
    Tank,
    Enchanter,
    EngageSupport,
    ApSupport,
}

impl Archetype {
    /// Canonical uppercase identifier matching `champion_build_profiles.json`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Archetype::BurstMage => "BURST_MAGE",
            Archetype::BattleMage => "BATTLE_MAGE",
            Archetype::ApAssassin => "AP_ASSASSIN",
            Archetype::AdAssassin => "AD_ASSASSIN",
            Archetype::CritMarksman => "CRIT_MARKSMAN",
            Archetype::OnHitMarksman => "ONHIT_MARKSMAN",
            Archetype::Juggernaut => "JUGGERNAUT",
            Archetype::DiverFighter => "DIVER_FIGHTER",
            Archetype::Skirmisher => "SKIRMISHER",
            Archetype::Tank => "TANK",
            Archetype::Enchanter => "ENCHANTER",
            Archetype::EngageSupport => "ENGAGE_SUPPORT",
            Archetype::ApSupport => "AP_SUPPORT",
        }
    }

    /// Human-readable display label (e.g. `"BURST MAGE"`).
    pub fn display_name(self) -> String {
        self.as_str().replace('_', " ")
    }
}

/// Complete role-resolved build recommendation for a champion.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildProfile {
    pub champion_id: u32,
    pub champion_key: String,
    pub champion_name: String,
    pub archetype: Archetype,
    pub role: Role,
    pub default_role: Role,
    pub skill_priority: String,
    pub skill_order: Vec<String>,
    pub tip: String,
    pub starting_items: Vec<u32>,
    pub boots: u32,
    pub core_items: Vec<u32>,
    pub situational_items: Vec<u32>,
    pub primary_tree: u32,
    pub secondary_tree: u32,
    pub keystone: u32,
    pub rune_ids: Vec<u32>,
    pub spell_one: i64,
    pub spell_two: i64,
}

impl BuildProfile {
    /// Returns `skill_priority` with spaces stripped (e.g. `"Q>R>E"` for Udyr).
    pub fn compact_skill_priority(&self) -> String {
        self.skill_priority.replace(' ', "")
    }
}

/// Typed LCU `/lol-item-sets/v1/item-sets/{id}/sets` item entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemSetEntry {
    pub id: String,
    pub count: u32,
}

/// Typed LCU item set block.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemSetBlock {
    #[serde(rename = "type")]
    pub kind: String,
    pub items: Vec<ItemSetEntry>,
}

/// Typed LCU custom shop item set payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemSetPayload {
    pub title: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub map: String,
    pub mode: String,
    pub priority: bool,
    pub sortrank: u32,
    pub associated_champions: Vec<u32>,
    pub associated_maps: Vec<u32>,
    pub blocks: Vec<ItemSetBlock>,
}

impl ItemSetPayload {
    /// Converts this item set payload into `serde_json::Value` for LCU upload.
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawProfile {
    archetype: Archetype,
    role: Role,
    skill_priority: String,
    tip: String,
}

struct BuildCatalog {
    by_champ_role: HashMap<(u32, Role), BuildProfile>,
    default_by_id: HashMap<u32, BuildProfile>,
}

static CATALOG: LazyLock<BuildCatalog> = LazyLock::new(load_catalog);

/// Returns the pre-indexed static build profile for `(champion_id, role)`.
///
/// Returns `None` only if `champion_id` is not present in `champions::all_champions()`.
pub fn build_for_champion(id: u32, role: Role) -> Option<&'static BuildProfile> {
    CATALOG.by_champ_role.get(&(id, role))
}

/// Returns the default (primary role) build profile for `champion_id`.
pub fn default_build_for_champion(id: u32) -> Option<&'static BuildProfile> {
    CATALOG.default_by_id.get(&id)
}

/// Resolves a build recommendation by champion key/name and optional requested role.
pub fn recommend(
    key: &str,
    name: &str,
    roles: &[Role],
    requested_role: Option<Role>,
) -> BuildProfile {
    if let Some(champ) =
        champions::champion_by_name_or_key(key).or_else(|| champions::champion_by_name_or_key(name))
    {
        let role = requested_role.unwrap_or_else(|| {
            default_build_for_champion(champ.id)
                .map(|profile| profile.default_role)
                .unwrap_or_else(|| champ.primary_role())
        });
        if let Some(profile) = build_for_champion(champ.id, role) {
            return profile.clone();
        }
    }

    let role = requested_role
        .or_else(|| roles.first().copied())
        .unwrap_or(Role::Mid);
    assemble_profile(
        0,
        key.to_string(),
        name.to_string(),
        Archetype::DiverFighter,
        role,
        role,
        "Q > E > W".to_string(),
        "Focus on safe trades and your champion's core power spikes.".to_string(),
    )
}

/// Builds the canonical LCU rune page payload for a champion build.
pub fn rune_page(champion: &str, build: &BuildProfile) -> RunePagePayload {
    let perks: Vec<i64> = build.rune_ids.iter().map(|&id| i64::from(id)).collect();
    runes::build_rune_page(
        &format!("RiftCore {champion} {}", build.role),
        build.primary_tree,
        build.secondary_tree,
        &perks,
    )
}

/// Builds the canonical LCU custom shop item set payload for a champion build.
pub fn champion_item_set(champion: &Champion, build: &BuildProfile) -> ItemSetPayload {
    let to_entries = |ids: &[u32]| -> Vec<ItemSetEntry> {
        ids.iter()
            .map(|id| ItemSetEntry {
                id: id.to_string(),
                count: 1,
            })
            .collect()
    };

    let mut starting = to_entries(&build.starting_items);
    if build.starting_items.contains(&2003) {
        if let Some(potion) = starting.iter_mut().find(|item| item.id == "2003") {
            potion.count = 2;
        }
    }

    let mut early = vec![ItemSetEntry {
        id: build.boots.to_string(),
        count: 1,
    }];
    let early_components: &[u32] = match build.role {
        Role::Adc => &[1001, 1038, 1037],
        Role::Mid => &[1082, 3802, 3108],
        Role::Jungle => &[1001, 3133, 3067],
        Role::Support => &[1001, 3067, 3114],
        Role::Top => &[1001, 3077, 3067],
    };
    early.extend(to_entries(early_components));

    let mut situational = build.situational_items.clone();
    let extra: [u32; 2] = match build.role {
        Role::Adc | Role::Top => [3033, 6609],
        Role::Mid => [3165, 3135],
        Role::Support => [3075, 3107],
        Role::Jungle => [0, 0],
    };
    for id in extra.into_iter().filter(|&id| id != 0) {
        if !situational.contains(&id) && items::item_available_on_map(id, MAP_SUMMONERS_RIFT) {
            situational.push(id);
        }
    }
    situational.truncate(6);

    let elixir = if matches!(build.role, Role::Mid | Role::Support) {
        2139 // Elixir of Sorcery
    } else {
        2140 // Elixir of Wrath
    };

    let safe_name: String = champion
        .name
        .chars()
        .filter(|ch| {
            ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace() || *ch == '\'' || *ch == '.'
        })
        .collect();

    ItemSetPayload {
        title: format!("RiftCore: {safe_name} {}", build.role),
        kind: "custom".to_string(),
        map: "any".to_string(),
        mode: "any".to_string(),
        priority: true,
        sortrank: 1,
        associated_champions: vec![champion.id],
        associated_maps: Vec::new(),
        blocks: vec![
            ItemSetBlock {
                kind: "Starting Items & Potions".to_string(),
                items: starting,
            },
            ItemSetBlock {
                kind: "Early Buys & Boots".to_string(),
                items: early,
            },
            ItemSetBlock {
                kind: "Core Build".to_string(),
                items: to_entries(&build.core_items),
            },
            ItemSetBlock {
                kind: "Situational Counters".to_string(),
                items: to_entries(&situational),
            },
            ItemSetBlock {
                kind: "Consumables & Elixirs".to_string(),
                items: vec![
                    ItemSetEntry {
                        id: "2055".to_string(),
                        count: 2,
                    },
                    ItemSetEntry {
                        id: elixir.to_string(),
                        count: 1,
                    },
                ],
            },
        ],
    }
}

/// Converts a typed `ItemSetPayload` into JSON (matches legacy helper call sites).
pub fn champion_item_set_json(champion: &Champion, build: &BuildProfile) -> Value {
    json!(champion_item_set(champion, build))
}

/// Generates an 18-level skill order from a priority string (e.g. `"Q > E > W"`).
///
/// For 4-stance champions like Udyr whose priority includes `"R"` (`"Q > R > E"`
/// or `"Q>R>E"`), avoids injecting duplicate forced `"R"` slots at levels
/// 6/11/16 and instead levels all three stances up to rank 6 (`6 + 6 + 6 = 18`).
pub fn skill_order(priority: &str) -> Vec<String> {
    let skills: Vec<&str> = priority
        .split('>')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .collect();
    let (first, second, third) = match skills.as_slice() {
        [first, second, third, ..] => (*first, *second, *third),
        _ => ("Q", "E", "W"),
    };

    let includes_r = [first, second, third]
        .iter()
        .any(|skill| skill.eq_ignore_ascii_case("R"));

    if includes_r {
        let mut order = vec![first, second, third];
        let (mut first_count, mut second_count) = (1usize, 1usize);
        for _level in 4..=18 {
            if first_count < 6 {
                order.push(first);
                first_count += 1;
            } else if second_count < 6 {
                order.push(second);
                second_count += 1;
            } else {
                order.push(third);
            }
        }
        return order.into_iter().map(str::to_owned).collect();
    }

    let mut order = vec![first, second, third];
    let (mut first_count, mut second_count) = (1usize, 1usize);
    for level in 4..=18 {
        if matches!(level, 6 | 11 | 16) {
            order.push("R");
        } else if first_count < 5 {
            order.push(first);
            first_count += 1;
        } else if second_count < 5 {
            order.push(second);
            second_count += 1;
        } else {
            order.push(third);
        }
    }
    order.into_iter().map(str::to_owned).collect()
}

fn load_catalog() -> BuildCatalog {
    let raw_profiles: HashMap<String, RawProfile> =
        serde_json::from_str(include_str!("../../assets/champion_build_profiles.json"))
            .unwrap_or_default();

    let mut normalized_profiles: HashMap<String, RawProfile> = raw_profiles
        .into_iter()
        .map(|(k, v)| (champions::normalize_identifier(&k), v))
        .collect();

    // Ensure the 4 newly added champions have curated profiles
    for (key, profile) in missing_new_champion_profiles() {
        normalized_profiles
            .entry(champions::normalize_identifier(key))
            .or_insert(profile);
    }

    // Enforce Udyr's canonical "Q>R>E" priority
    if let Some(udyr) = normalized_profiles.get_mut("udyr") {
        udyr.skill_priority = "Q>R>E".to_string();
    }

    let champions_list = champions::all_champions();
    let mut by_champ_role = HashMap::with_capacity(champions_list.len() * Role::ALL.len());
    let mut default_by_id = HashMap::with_capacity(champions_list.len());

    for champ in champions_list {
        let profile = find_raw_profile(&normalized_profiles, champ);
        let (archetype, default_role, skill_priority, tip) = match profile {
            Some(p) => (p.archetype, p.role, p.skill_priority.clone(), p.tip.clone()),
            None => (
                default_archetype_for_role(champ.primary_role()),
                champ.primary_role(),
                "Q > E > W".to_string(),
                "Focus on safe trades and your champion's core power spikes.".to_string(),
            ),
        };

        for &role in &Role::ALL {
            let built = assemble_profile(
                champ.id,
                champ.key.clone(),
                champ.name.clone(),
                archetype,
                role,
                default_role,
                skill_priority.clone(),
                tip.clone(),
            );
            if role == default_role {
                default_by_id.insert(champ.id, built.clone());
            }
            by_champ_role.insert((champ.id, role), built);
        }
    }

    BuildCatalog {
        by_champ_role,
        default_by_id,
    }
}

fn find_raw_profile<'a>(
    profiles: &'a HashMap<String, RawProfile>,
    champ: &Champion,
) -> Option<&'a RawProfile> {
    let key_norm = champions::normalize_identifier(&champ.key);
    let name_norm = champions::normalize_identifier(&champ.name);
    profiles
        .get(&key_norm)
        .or_else(|| profiles.get(&name_norm))
        .or_else(|| match key_norm.as_str() {
            "monkeyking" => profiles.get("wukong"),
            "nunu" => profiles.get("nunuwillump"),
            "renata" => profiles.get("renataglasc"),
            _ => None,
        })
}

fn default_archetype_for_role(role: Role) -> Archetype {
    match role {
        Role::Top => Archetype::DiverFighter,
        Role::Jungle => Archetype::DiverFighter,
        Role::Mid => Archetype::BurstMage,
        Role::Adc => Archetype::CritMarksman,
        Role::Support => Archetype::Enchanter,
    }
}

fn missing_new_champion_profiles() -> [(&'static str, RawProfile); 4] {
    [
        (
            "Locke",
            RawProfile {
                archetype: Archetype::ApAssassin,
                role: Role::Mid,
                skill_priority: "Q > E > W".to_string(),
                tip: "Tag targets with Ritual Nails (Q) before detonating with basic attacks, and hold Purgatory (R) to execute low-health enemies.".to_string(),
            },
        ),
        (
            "Mel",
            RawProfile {
                archetype: Archetype::BurstMage,
                role: Role::Mid,
                skill_priority: "Q > E > W".to_string(),
                tip: "Stack long-range poke safely, hold Rebuttal (W) to reflect key enemy projectiles, and finish low-HP targets with your execute.".to_string(),
            },
        ),
        (
            "Yunara",
            RawProfile {
                archetype: Archetype::CritMarksman,
                role: Role::Adc,
                skill_priority: "Q > E > W".to_string(),
                tip: "Build Cultivation of Spirit (Q) stacks in lane before triggering Unleash, and use Transcend One's Self (R) for rapid skirmish resets.".to_string(),
            },
        ),
        (
            "Zaahen",
            RawProfile {
                archetype: Archetype::Juggernaut,
                role: Role::Top,
                skill_priority: "Q > E > W".to_string(),
                tip: "Build Determination stacks in extended trades and chain Dreaded Return (W) pulls into The Darkin Glaive (Q) knockup.".to_string(),
            },
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn assemble_profile(
    champion_id: u32,
    champion_key: String,
    champion_name: String,
    archetype: Archetype,
    role: Role,
    default_role: Role,
    skill_priority: String,
    tip: String,
) -> BuildProfile {
    let spec = preset_spec(archetype, role);
    BuildProfile {
        champion_id,
        champion_key,
        champion_name,
        archetype,
        role,
        default_role,
        skill_order: skill_order(&skill_priority),
        skill_priority,
        tip,
        starting_items: available_sr(spec.start),
        boots: spec.boots,
        core_items: available_sr(spec.core),
        situational_items: available_sr(spec.situational),
        primary_tree: spec.primary_tree,
        secondary_tree: spec.secondary_tree,
        keystone: spec.keystone,
        rune_ids: spec.perks.to_vec(),
        spell_one: spec.spell_one,
        spell_two: spec.spell_two,
    }
}

fn available_sr(items: Vec<u32>) -> Vec<u32> {
    items
        .into_iter()
        .filter(|&id| items::item_available_on_map(id, MAP_SUMMONERS_RIFT))
        .collect()
}

struct PresetSpec {
    start: Vec<u32>,
    boots: u32,
    core: Vec<u32>,
    situational: Vec<u32>,
    primary_tree: u32,
    secondary_tree: u32,
    keystone: u32,
    perks: [u32; 9],
    spell_one: i64,
    spell_two: i64,
}

fn preset_spec(archetype: Archetype, role: Role) -> PresetSpec {
    let (mut start, mut boots, mut core, mut situational, primary, secondary, keystone, perks) =
        archetype_base(archetype);

    // Role-specific Jungle and Support overrides fix starter items, quest items,
    // boots, and spells across all archetypes (e.g. Ivern, Amumu, Kindred,
    // Karthus, Evelynn, Udyr, Pyke, Senna, Mel).
    match role {
        Role::Jungle => {
            start = jungle_starter(archetype);
            match archetype {
                Archetype::Enchanter => {
                    // Ivern Jungle: remove Support quest Dream Maker (3870) and use real Jungle enchanter core
                    boots = 3158;
                    core = vec![6617, 3107, 6616];
                    situational = vec![3504, 3222, 3190];
                }
                Archetype::EngageSupport => {
                    boots = 3047;
                    core = vec![3068, 3002, 3190];
                    situational = vec![3109, 3075, 4401];
                }
                Archetype::ApSupport => {
                    boots = 3020;
                    core = vec![2503, 6653, 3116];
                    situational = vec![3157, 3165, 3137];
                }
                Archetype::BattleMage => {
                    boots = 3020;
                    core = vec![2503, 6653, 3089];
                    situational = vec![3157, 3135, 3102];
                }
                Archetype::Tank => {
                    boots = 3047;
                    core = vec![3068, 3075, 2504];
                    situational = vec![6664, 3742, 4401];
                }
                _ => {}
            }
        }
        Role::Support => {
            start = vec![3865, 2003]; // World Atlas + Health Potion
            match archetype {
                Archetype::Enchanter => {
                    boots = 3158;
                    core = vec![3870, 6617, 6616]; // 3870 = Dream Maker
                }
                Archetype::EngageSupport => {
                    boots = 3009;
                    core = vec![3869, 3002, 3190]; // 3869 = Celestial Opposition, 3002 = Trailblazer
                }
                Archetype::ApSupport => {
                    boots = 3020;
                    core = vec![3871, 6653, 3116]; // 3871 = Zaz'Zak's Realmspike
                }
                Archetype::BurstMage | Archetype::BattleMage | Archetype::ApAssassin => {
                    boots = 3020;
                    core = vec![3871, 6653, 4645];
                    situational = vec![3157, 3165, 3135];
                }
                Archetype::AdAssassin => {
                    // Pyke Support
                    boots = 3009;
                    core = vec![3877, 3142, 6701]; // 3877 = Bloodsong
                    situational = vec![3814, 6697, 3026];
                }
                Archetype::CritMarksman | Archetype::OnHitMarksman => {
                    // Senna / Ashe Support
                    boots = 3009;
                    core = vec![3877, 3094, 3031];
                    situational = vec![3033, 3814, 3072];
                }
                Archetype::Tank | Archetype::Juggernaut => {
                    boots = 3009;
                    core = vec![3869, 3190, 3109];
                    situational = vec![3075, 3050, 3002];
                }
                Archetype::DiverFighter | Archetype::Skirmisher => {
                    boots = 3009;
                    core = vec![3877, 6692, 3071];
                    situational = vec![3814, 3053, 3026];
                }
            }
        }
        Role::Top | Role::Mid | Role::Adc => {
            // Ensure solo-lane Enchanter/Support archetypes do not suggest World Atlas (3865)
            match archetype {
                Archetype::Enchanter => {
                    start = vec![1056, 2003];
                    core = vec![6617, 6616, 3107];
                }
                Archetype::EngageSupport => {
                    start = vec![1054, 2003];
                    core = vec![3068, 3075, 3190];
                }
                Archetype::ApSupport => {
                    start = vec![1056, 2003];
                    core = vec![2503, 6653, 3157];
                }
                _ => {}
            }
        }
    }

    let (spell_one, spell_two) = spells_for(archetype, role);

    PresetSpec {
        start,
        boots,
        core,
        situational,
        primary_tree: primary,
        secondary_tree: secondary,
        keystone,
        perks,
        spell_one,
        spell_two,
    }
}

fn jungle_starter(archetype: Archetype) -> Vec<u32> {
    let pet = match archetype {
        Archetype::Tank | Archetype::Juggernaut => 1103, // Mosstomper Seedling
        Archetype::BurstMage
        | Archetype::BattleMage
        | Archetype::ApAssassin
        | Archetype::Enchanter
        | Archetype::ApSupport => 1102, // Gustwalker Hatchling
        _ => 1101,                                       // Scorchclaw Pup
    };
    vec![pet, 2031] // Jungle Pet + Refillable Potion
}

fn spells_for(archetype: Archetype, role: Role) -> (i64, i64) {
    let second = match role {
        Role::Jungle => 11,  // Smite
        Role::Support => 14, // Ignite
        Role::Adc => 7,      // Heal
        Role::Mid => match archetype {
            Archetype::BurstMage
            | Archetype::ApAssassin
            | Archetype::AdAssassin
            | Archetype::Skirmisher
            | Archetype::ApSupport => 14, // Ignite
            _ => 12, // Teleport
        },
        Role::Top => match archetype {
            Archetype::Juggernaut | Archetype::OnHitMarksman => 6, // Ghost
            Archetype::Skirmisher => 14,                           // Ignite
            _ => 12,                                               // Teleport
        },
    };
    (4, second)
}

#[allow(clippy::type_complexity)]
fn archetype_base(
    archetype: Archetype,
) -> (Vec<u32>, u32, Vec<u32>, Vec<u32>, u32, u32, u32, [u32; 9]) {
    match archetype {
        Archetype::BurstMage => (
            vec![1056, 2003],
            3020,
            vec![6655, 4645, 3089], // Luden's Echo, Shadowflame, Rabadon's
            vec![3157, 3135, 3102],
            8100,
            8200,
            8112,
            [8112, 8139, 8140, 8106, 8226, 8210, 5008, 5008, 5001],
        ),
        Archetype::BattleMage => (
            vec![1056, 2003],
            3020,
            vec![6657, 6653, 4633], // Rod of Ages, Liandry's, Riftmaker
            vec![3157, 3137, 3102],
            8200,
            8300,
            8230,
            [8230, 8226, 8210, 8236, 8345, 8347, 5008, 5008, 5001],
        ),
        Archetype::ApAssassin => (
            vec![1056, 2003],
            3020,
            vec![3100, 4646, 4645], // Lich Bane, Stormsurge, Shadowflame
            vec![3157, 3089, 3135],
            8100,
            8000,
            8112,
            [8112, 8143, 8140, 8105, 9111, 8014, 5008, 5008, 5001],
        ),
        Archetype::AdAssassin => (
            vec![1055, 2003],
            3158,
            vec![6698, 6701, 6694], // Profane Hydra, Opportunity, Serylda's Grudge
            vec![3814, 3142, 3026],
            8100,
            8200,
            8112,
            [8112, 8143, 8140, 8106, 8210, 8237, 5008, 5008, 5001],
        ),
        Archetype::CritMarksman => (
            vec![1055, 2003],
            3006,
            vec![6672, 3031, 3036], // Kraken Slayer, Infinity Edge, LDR
            vec![3094, 3072, 6673],
            8000,
            8200,
            8008,
            [8008, 9101, 9104, 8014, 8210, 8236, 5005, 5008, 5001],
        ),
        Archetype::OnHitMarksman => (
            vec![1055, 2003],
            3006,
            vec![3153, 3124, 3302], // BotRK, Guinsoo's, Terminus
            vec![3091, 6665, 6673],
            8000,
            8400,
            8005,
            [8005, 9111, 9104, 8299, 8429, 8451, 5005, 5008, 5001],
        ),
        Archetype::Juggernaut => (
            vec![1055, 2003],
            3047,
            vec![6631, 3071, 3053], // Stridebreaker, Black Cleaver, Sterak's
            vec![3742, 4401, 3075],
            8000,
            8400,
            8010,
            [8010, 9111, 9105, 8299, 8429, 8451, 5008, 5008, 5001],
        ),
        Archetype::Skirmisher => (
            vec![1055, 2003],
            3006,
            vec![3153, 3031, 6673], // BotRK, IE, Immortal Shieldbow
            vec![6333, 3156, 3026],
            8000,
            8400,
            8008,
            [8008, 9111, 9104, 8299, 8429, 8451, 5005, 5008, 5001],
        ),
        Archetype::Tank => (
            vec![1054, 2003],
            3047,
            vec![3084, 3068, 3075], // Heartsteel, Sunfire, Thornmail
            vec![2504, 6664, 2502],
            8400,
            8000,
            8437,
            [8437, 8446, 8429, 8451, 9111, 9105, 5008, 5001, 5011],
        ),
        Archetype::Enchanter => (
            vec![3865, 2003],
            3158,
            vec![3870, 6617, 6616], // Dream Maker (3870), Moonstone, SoFW
            vec![3504, 3107, 3222],
            8200,
            8400,
            8214,
            [8214, 8226, 8210, 8237, 8463, 8453, 5008, 5008, 5001],
        ),
        Archetype::EngageSupport => (
            vec![3865, 2003],
            3009,
            vec![3869, 3002, 3190], // Celestial Opposition (3869), Trailblazer (3002), Locket
            vec![3109, 3075, 3050],
            8400,
            8300,
            8439,
            [8439, 8463, 8473, 8451, 8306, 8347, 5007, 5001, 5011],
        ),
        Archetype::ApSupport => (
            vec![3865, 2003],
            3020,
            vec![3871, 6653, 3116], // Zaz'Zak's (3871), Liandry's, Rylai's
            vec![3165, 3137, 3102],
            8200,
            8100,
            8229,
            [8229, 8226, 8210, 8237, 8140, 8106, 5008, 5008, 5001],
        ),
        Archetype::DiverFighter => (
            vec![1055, 2003],
            3047,
            vec![6610, 6692, 3071], // Sundered Sky (6610), Eclipse, Black Cleaver
            vec![3053, 3156, 3026],
            8000,
            8400,
            8010,
            [8010, 9111, 9104, 8299, 8473, 8451, 5008, 5008, 5001],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn udyr_skill_order_is_q_r_e_without_duplicate_r_bug() {
        let udyr = build_for_champion(77, Role::Jungle).expect("Udyr (77) build must exist");
        assert_eq!(udyr.compact_skill_priority(), "Q>R>E");
        assert_eq!(udyr.skill_order.len(), 18);
        assert_eq!(udyr.skill_order[0], "Q");
        assert_eq!(udyr.skill_order[1], "R");
        assert_eq!(udyr.skill_order[2], "E");
        // Previously levels [6, 11, 16] stacked onto second="R", producing 8 R ranks.
        let r_count = udyr.skill_order.iter().filter(|s| *s == "R").count();
        assert_eq!(r_count, 6);
        assert_eq!(udyr.skill_order.iter().filter(|s| *s == "Q").count(), 6);
        assert_eq!(udyr.skill_order.iter().filter(|s| *s == "E").count(), 6);
        // Jungle preset check on Udyr
        assert_eq!(udyr.spell_two, 11);
        assert!(udyr.starting_items.contains(&1103));
        assert!(udyr.starting_items.contains(&2031));
    }

    #[test]
    fn jungle_and_support_presets_use_correct_items_and_spells() {
        // Ivern Jungle must get Jungle Pet + Smite and NOT Dream Maker (3870) or World Atlas (3865)
        let ivern = build_for_champion(427, Role::Jungle).expect("Ivern build must exist");
        assert_eq!(ivern.spell_two, 11);
        assert!(ivern.starting_items.contains(&1102));
        assert!(!ivern.starting_items.contains(&3865));
        assert!(!ivern.core_items.contains(&3869));
        assert!(!ivern.core_items.contains(&3870));

        // Enchanter Support (Lulu = 117) gets World Atlas (3865) + Dream Maker (3870) + Ignite (14)
        let lulu = build_for_champion(117, Role::Support).expect("Lulu build must exist");
        assert_eq!(lulu.spell_two, 14);
        assert!(lulu.starting_items.contains(&3865));
        assert!(lulu.core_items.contains(&3870));
        assert!(!lulu.core_items.contains(&3869));

        // Engage Support (Thresh = 412) gets World Atlas (3865) + Celestial Opposition (3869) + Trailblazer (3002)
        let thresh = build_for_champion(412, Role::Support).expect("Thresh build must exist");
        assert_eq!(thresh.spell_two, 14);
        assert!(thresh.starting_items.contains(&3865));
        assert!(thresh.core_items.contains(&3869));
        assert!(thresh.core_items.contains(&3002));
        assert!(!thresh.core_items.contains(&2503)); // 2503 is Blackfire Torch

        // Pyke Support (555) gets World Atlas (3865) + Bloodsong (3877) + Ignite (14)
        let pyke = build_for_champion(555, Role::Support).expect("Pyke build must exist");
        assert_eq!(pyke.spell_two, 14);
        assert!(pyke.starting_items.contains(&3865));
        assert!(pyke.core_items.contains(&3877));

        // Senna Support (235) gets World Atlas (3865)
        let senna = build_for_champion(235, Role::Support).expect("Senna build must exist");
        assert_eq!(senna.spell_two, 14);
        assert!(senna.starting_items.contains(&3865));

        // Karthus (30) & Amumu (32) Jungle get Jungle Pet + Smite
        let karthus = build_for_champion(30, Role::Jungle).expect("Karthus build must exist");
        assert_eq!(karthus.spell_two, 11);
        assert!(karthus.starting_items.contains(&1102));

        let amumu = build_for_champion(32, Role::Jungle).expect("Amumu build must exist");
        assert_eq!(amumu.spell_two, 11);
        assert!(amumu.starting_items.contains(&1103));
    }

    #[test]
    fn all_champions_including_four_new_have_valid_profiles_and_roles() {
        for champ in champions::all_champions() {
            let default = default_build_for_champion(champ.id)
                .expect("every champion must have a default build");
            assert!(
                champ.supports_role(default.default_role),
                "champion {} ({}) roles {:?} must include default role {}",
                champ.name,
                champ.id,
                champ.roles,
                default.default_role
            );
            assert_eq!(default.skill_order.len(), 18);
            assert!(!default.core_items.is_empty());
        }

        // Verify the 4 newly added champions do not fall back to generic DIVER_FIGHTER
        let mel = default_build_for_champion(800).expect("Mel build");
        assert_eq!(mel.archetype, Archetype::BurstMage);
        assert_eq!(mel.default_role, Role::Mid);

        let locke = default_build_for_champion(805).expect("Locke build");
        assert_eq!(locke.archetype, Archetype::ApAssassin);
        assert_eq!(locke.default_role, Role::Mid);

        let yunara = default_build_for_champion(804).expect("Yunara build");
        assert_eq!(yunara.archetype, Archetype::CritMarksman);
        assert_eq!(yunara.default_role, Role::Adc);

        let zaahen = default_build_for_champion(904).expect("Zaahen build");
        assert_eq!(zaahen.archetype, Archetype::Juggernaut);
        assert_eq!(zaahen.default_role, Role::Top);

        // Verify champion_build_profiles.json parsed cleanly (not fallback defaults)
        let kaisa = default_build_for_champion(145).expect("Kai'Sa build");
        assert_eq!(kaisa.archetype, Archetype::OnHitMarksman);
        assert_eq!(kaisa.default_role, Role::Adc);

        let ahri = default_build_for_champion(103).expect("Ahri build");
        assert_eq!(ahri.archetype, Archetype::BurstMage);
        assert_eq!(ahri.skill_priority, "Q > W > E");

        assert!(build_for_champion(999_999, Role::Mid).is_none());
    }
}
