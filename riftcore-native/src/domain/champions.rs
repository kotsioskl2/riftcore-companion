//! Static League of Legends champion roster and role metadata.
//!
//! Loads the bundled champion catalog (`all_champions.json`), guarantees the
//! latest roster entries (`Locke`, `Mel`, `Yunara`, `Zaahen`), and applies
//! curated role corrections where generic Data Dragon tag heuristics omitted
//! a champion's primary position.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

/// Playable Summoner's Rift positions used for draft, build, and pool filtering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Role {
    Top,
    Jungle,
    Mid,
    Adc,
    Support,
}

impl Role {
    /// All five canonical Summoner's Rift roles in draft order.
    pub const ALL: [Role; 5] = [Role::Top, Role::Jungle, Role::Mid, Role::Adc, Role::Support];

    /// Canonical uppercase label matching Riot's `all_champions.json` schema.
    pub const fn as_str(self) -> &'static str {
        match self {
            Role::Top => "TOP",
            Role::Jungle => "JUNGLE",
            Role::Mid => "MID",
            Role::Adc => "ADC",
            Role::Support => "SUPPORT",
        }
    }

    /// Short uppercase abbreviation used in compact HUD badges.
    pub const fn short_label(self) -> &'static str {
        match self {
            Role::Top => "TOP",
            Role::Jungle => "JNG",
            Role::Mid => "MID",
            Role::Adc => "BOT",
            Role::Support => "SUP",
        }
    }

    /// Parses common LCU/Riot/UI role aliases (e.g. `"MIDDLE"`, `"BOTTOM"`, `"UTILITY"`).
    pub fn from_str_lossy(value: &str) -> Option<Role> {
        match value.trim().to_ascii_uppercase().as_str() {
            "TOP" => Some(Role::Top),
            "JUNGLE" | "JNG" | "JG" => Some(Role::Jungle),
            "MID" | "MIDDLE" => Some(Role::Mid),
            "ADC" | "BOT" | "BOTTOM" => Some(Role::Adc),
            "SUPPORT" | "SUP" | "UTILITY" => Some(Role::Support),
            _ => None,
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Role::from_str_lossy(s).ok_or_else(|| format!("unknown League role: {s}"))
    }
}

/// Canonical metadata for a League of Legends champion.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Champion {
    pub id: u32,
    pub name: String,
    pub key: String,
    pub title: String,
    #[serde(default)]
    pub roles: Vec<Role>,
}

impl Champion {
    /// Returns the champion's primary role, defaulting to `Role::Mid` if unspecified.
    pub fn primary_role(&self) -> Role {
        self.roles.first().copied().unwrap_or(Role::Mid)
    }

    /// Returns `true` if `role` is listed among this champion's viable positions.
    pub fn supports_role(&self, role: Role) -> bool {
        self.roles.contains(&role)
    }
}

struct ChampionCatalog {
    list: Vec<Champion>,
    by_id: HashMap<u32, usize>,
    by_key_norm: HashMap<String, usize>,
}

static CATALOG: LazyLock<ChampionCatalog> = LazyLock::new(load_catalog);

/// Returns the full embedded champion roster with corrected role metadata.
pub fn all_champions() -> &'static [Champion] {
    &CATALOG.list
}

/// Looks up a champion by numeric Riot champion ID.
pub fn champion_by_id(id: u32) -> Option<&'static Champion> {
    CATALOG
        .by_id
        .get(&id)
        .and_then(|&idx| CATALOG.list.get(idx))
}

/// Looks up a champion by Data Dragon key (e.g. `"MonkeyKing"`, `"AurelionSol"`).
pub fn champion_by_key(key: &str) -> Option<&'static Champion> {
    let norm = normalize_identifier(key);
    CATALOG
        .by_key_norm
        .get(&norm)
        .and_then(|&idx| CATALOG.list.get(idx))
}

/// Looks up a champion by display name, Data Dragon key, or known alias.
pub fn champion_by_name_or_key(query: &str) -> Option<&'static Champion> {
    let norm = normalize_identifier(query);
    if norm.is_empty() {
        return None;
    }
    if let Some(&idx) = CATALOG.by_key_norm.get(&norm) {
        return CATALOG.list.get(idx);
    }
    let aliased = match norm.as_str() {
        "wukong" => "monkeyking",
        "nunuwillump" => "nunu",
        "renataglasc" => "renata",
        _ => return None,
    };
    CATALOG
        .by_key_norm
        .get(aliased)
        .and_then(|&idx| CATALOG.list.get(idx))
}

/// Resolves a champion display name by numeric ID, falling back to `"Champion"`.
pub fn champion_name_or_default(id: u32) -> &'static str {
    champion_by_id(id)
        .map(|champ| champ.name.as_str())
        .unwrap_or("Champion")
}

/// Returns all champions viable in `role`.
pub fn champions_for_role(role: Role) -> Vec<&'static Champion> {
    CATALOG
        .list
        .iter()
        .filter(|champ| champ.supports_role(role))
        .collect()
}

/// Normalizes champion identifiers by stripping non-alphanumeric chars and lowercasing.
pub fn normalize_identifier(input: &str) -> String {
    input
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn load_catalog() -> ChampionCatalog {
    let mut list: Vec<Champion> =
        serde_json::from_str(include_str!("../../../all_champions.json")).unwrap_or_default();

    // Ensure the 4 newest champions exist even if loaded against an older manifest.
    for required in required_new_champions() {
        if !list.iter().any(|champ| champ.id == required.id) {
            list.push(required);
        }
    }

    for champ in &mut list {
        if let Some(corrected) = role_correction(champ.id) {
            champ.roles = corrected.to_vec();
        }
    }

    list.sort_by(|left, right| left.name.cmp(&right.name));

    let mut by_id = HashMap::with_capacity(list.len());
    let mut by_key_norm = HashMap::with_capacity(list.len() * 2);
    for (idx, champ) in list.iter().enumerate() {
        by_id.insert(champ.id, idx);
        by_key_norm.insert(normalize_identifier(&champ.key), idx);
        by_key_norm.insert(normalize_identifier(&champ.name), idx);
    }

    ChampionCatalog {
        list,
        by_id,
        by_key_norm,
    }
}

fn required_new_champions() -> [Champion; 4] {
    [
        Champion {
            id: 805,
            name: "Locke".to_string(),
            key: "Locke".to_string(),
            title: "the Ashen Exorcist".to_string(),
            roles: vec![Role::Mid, Role::Jungle],
        },
        Champion {
            id: 800,
            name: "Mel".to_string(),
            key: "Mel".to_string(),
            title: "the Soul's Reflection".to_string(),
            roles: vec![Role::Mid, Role::Support],
        },
        Champion {
            id: 804,
            name: "Yunara".to_string(),
            key: "Yunara".to_string(),
            title: "the Unbroken Faith".to_string(),
            roles: vec![Role::Adc],
        },
        Champion {
            id: 904,
            name: "Zaahen".to_string(),
            key: "Zaahen".to_string(),
            title: "The Unsundered".to_string(),
            roles: vec![Role::Top, Role::Jungle],
        },
    ]
}

/// Curated role corrections for the 21 champions whose DDragon-tag-derived roles
/// omitted their actual primary Summoner's Rift position.
fn role_correction(champion_id: u32) -> Option<&'static [Role]> {
    match champion_id {
        166 => Some(&[Role::Mid, Role::Top, Role::Adc]), // Akshan
        32 => Some(&[Role::Jungle, Role::Support]),      // Amumu
        9 => Some(&[Role::Jungle, Role::Support]),       // Fiddlesticks
        3 => Some(&[Role::Mid, Role::Support, Role::Top]), // Galio
        104 => Some(&[Role::Jungle]),                    // Graves
        427 => Some(&[Role::Jungle, Role::Support]),     // Ivern
        30 => Some(&[Role::Jungle, Role::Mid]),          // Karthus
        10 => Some(&[Role::Top, Role::Mid]),             // Kayle
        85 => Some(&[Role::Top, Role::Mid]),             // Kennen
        203 => Some(&[Role::Jungle]),                    // Kindred
        895 => Some(&[Role::Adc]),                       // Nilah
        20 => Some(&[Role::Jungle]),                     // Nunu & Willump
        133 => Some(&[Role::Top]),                       // Quinn
        33 => Some(&[Role::Jungle, Role::Top]),          // Rammus
        113 => Some(&[Role::Jungle, Role::Top]),         // Sejuani
        147 => Some(&[Role::Mid, Role::Support, Role::Adc]), // Seraphine
        72 => Some(&[Role::Jungle, Role::Top]),          // Skarner
        17 => Some(&[Role::Top, Role::Jungle, Role::Support]), // Teemo
        157 => Some(&[Role::Mid, Role::Top, Role::Adc]), // Yasuo
        777 => Some(&[Role::Mid, Role::Top]),            // Yone
        154 => Some(&[Role::Jungle, Role::Top, Role::Support]), // Zac
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roster_includes_all_four_new_champions() {
        let locke = champion_by_id(805).expect("Locke (805) must exist");
        assert_eq!(locke.name, "Locke");
        assert_eq!(locke.key, "Locke");
        assert!(locke.supports_role(Role::Mid));
        assert!(locke.supports_role(Role::Jungle));

        let mel = champion_by_id(800).expect("Mel (800) must exist");
        assert_eq!(mel.name, "Mel");
        assert!(mel.supports_role(Role::Mid));
        assert!(mel.supports_role(Role::Support));

        let yunara = champion_by_id(804).expect("Yunara (804) must exist");
        assert_eq!(yunara.name, "Yunara");
        assert!(yunara.supports_role(Role::Adc));

        let zaahen = champion_by_id(904).expect("Zaahen (904) must exist");
        assert_eq!(zaahen.name, "Zaahen");
        assert!(zaahen.supports_role(Role::Top));
        assert!(zaahen.supports_role(Role::Jungle));

        assert!(all_champions().len() >= 173);
    }

    #[test]
    fn all_21_role_mismatches_are_corrected() {
        let expected_primary = [
            (166, Role::Mid),    // Akshan
            (32, Role::Jungle),  // Amumu
            (9, Role::Jungle),   // Fiddlesticks
            (3, Role::Mid),      // Galio
            (104, Role::Jungle), // Graves
            (427, Role::Jungle), // Ivern
            (30, Role::Jungle),  // Karthus
            (10, Role::Top),     // Kayle
            (85, Role::Top),     // Kennen
            (203, Role::Jungle), // Kindred
            (895, Role::Adc),    // Nilah
            (20, Role::Jungle),  // Nunu & Willump
            (133, Role::Top),    // Quinn
            (33, Role::Jungle),  // Rammus
            (113, Role::Jungle), // Sejuani
            (147, Role::Mid),    // Seraphine
            (72, Role::Jungle),  // Skarner
            (17, Role::Top),     // Teemo
            (157, Role::Mid),    // Yasuo
            (777, Role::Mid),    // Yone
            (154, Role::Jungle), // Zac
        ];

        assert_eq!(expected_primary.len(), 21);
        for (id, role) in expected_primary {
            let champ = champion_by_id(id).expect("champion should exist");
            assert!(
                champ.supports_role(role),
                "champion {} ({}) should support {}",
                champ.name,
                champ.id,
                role
            );
            assert_eq!(
                champ.primary_role(),
                role,
                "champion {} ({}) should have primary role {}",
                champ.name,
                champ.id,
                role
            );
        }
    }

    #[test]
    fn name_and_key_aliases_resolve_cleanly() {
        let wukong = champion_by_name_or_key("MonkeyKing").expect("MonkeyKing resolves");
        assert_eq!(wukong.id, 62);
        assert_eq!(champion_by_name_or_key("Wukong").map(|c| c.id), Some(62));
        assert_eq!(
            champion_by_name_or_key("Nunu & Willump").map(|c| c.id),
            Some(20)
        );
        assert_eq!(
            champion_by_name_or_key("Renata Glasc").map(|c| c.id),
            Some(888)
        );
        assert_eq!(
            champion_by_name_or_key("FiddleSticks").map(|c| c.id),
            Some(9)
        );
    }
}
