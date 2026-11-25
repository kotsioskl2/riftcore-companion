//! ARAM Mayhem & Arena augment card catalog (without ungrounded rank fields).
//!
//! Per Riot Developer Policy compliance, the subjective `"rank"` (`S+`/`S`/`A`)
//! labels from `mayhem_cards.json` are intentionally omitted from `MayhemCard`.

use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// Augment rarity tier in ARAM Mayhem & Arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardTier {
    Prismatic,
    Gold,
    Silver,
}

impl CardTier {
    /// Canonical tier display label.
    pub const fn as_str(self) -> &'static str {
        match self {
            CardTier::Prismatic => "Prismatic",
            CardTier::Gold => "Gold",
            CardTier::Silver => "Silver",
        }
    }

    /// Parses a tier filter label (`"Prismatic"`, `"Gold"`, `"Silver"`, case-insensitive).
    pub fn from_str_lossy(value: &str) -> Option<CardTier> {
        match value.trim().to_ascii_lowercase().as_str() {
            "prismatic" => Some(CardTier::Prismatic),
            "gold" => Some(CardTier::Gold),
            "silver" => Some(CardTier::Silver),
            _ => None,
        }
    }
}

/// Game mode availability for a Mayhem / Arena augment card.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardMode {
    #[serde(rename = "ARAM")]
    Aram,
    Arena,
    Both,
}

impl CardMode {
    /// Canonical mode string (`"ARAM"`, `"Arena"`, `"Both"`).
    pub const fn as_str(self) -> &'static str {
        match self {
            CardMode::Aram => "ARAM",
            CardMode::Arena => "Arena",
            CardMode::Both => "Both",
        }
    }

    /// Returns `true` if `self` matches the requested filter mode.
    pub fn matches_filter(self, filter: CardMode) -> bool {
        matches!((self, filter), (_, CardMode::Both) | (CardMode::Both, _)) || self == filter
    }
}

/// Curated ARAM Mayhem / Arena augment card metadata (with `rank` stripped).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MayhemCard {
    pub id: String,
    pub name: String,
    pub tier: CardTier,
    pub mode: CardMode,
    pub archetype: String,
    pub description: String,
    pub tip: String,
    pub color: String,
    pub tag: String,
}

static CARDS: LazyLock<Vec<MayhemCard>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../assets/data/mayhem_cards.json")).unwrap_or_default()
});

/// Returns all curated ARAM Mayhem & Arena cards without subjective `rank` ratings.
pub fn all_cards() -> &'static [MayhemCard] {
    &CARDS
}

/// Looks up a Mayhem/Arena card by its slug ID.
pub fn card_by_id(id: &str) -> Option<&'static MayhemCard> {
    CARDS.iter().find(|card| card.id.eq_ignore_ascii_case(id))
}

/// Filters Mayhem/Arena cards by optional mode, optional tier, and free-text query.
pub fn filter_cards(
    mode: Option<CardMode>,
    tier: Option<CardTier>,
    query: &str,
) -> Vec<&'static MayhemCard> {
    let needle = query.trim().to_ascii_lowercase();
    CARDS
        .iter()
        .filter(|card| {
            let mode_ok = mode.is_none_or(|filter| card.mode.matches_filter(filter));
            let tier_ok = tier.is_none_or(|filter| card.tier == filter);
            let text_ok = needle.is_empty()
                || card.name.to_ascii_lowercase().contains(&needle)
                || card.archetype.to_ascii_lowercase().contains(&needle)
                || card.description.to_ascii_lowercase().contains(&needle)
                || card.tip.to_ascii_lowercase().contains(&needle)
                || card.tag.to_ascii_lowercase().contains(&needle);
            mode_ok && tier_ok && text_ok
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mayhem_cards_load_without_rank_field() {
        let cards = all_cards();
        assert_eq!(cards.len(), 26);

        let jg = card_by_id("jeweled-gauntlet").expect("Jeweled Gauntlet must load");
        assert_eq!(jg.tier, CardTier::Prismatic);
        assert_eq!(jg.mode, CardMode::Both);

        // Verify re-serialization never emits a rank field
        let serialized = serde_json::to_value(jg).expect("serialize card");
        let obj = serialized.as_object().expect("JSON object");
        assert!(!obj.contains_key("rank"), "rank field must not be present");
    }

    #[test]
    fn filter_cards_handles_mode_tier_and_query() {
        let aram_prismatics = filter_cards(Some(CardMode::Aram), Some(CardTier::Prismatic), "");
        assert!(!aram_prismatics.is_empty());
        assert!(aram_prismatics.iter().all(
            |card| card.tier == CardTier::Prismatic && card.mode.matches_filter(CardMode::Aram)
        ));

        let crit = filter_cards(None, None, "Crit");
        assert!(crit.iter().any(|card| card.id == "jeweled-gauntlet"));
    }
}
