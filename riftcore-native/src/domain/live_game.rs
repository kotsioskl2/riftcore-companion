// RiftCore Native — domain::live_game
// Typed Live Client Data (`allgamedata`) parsing, activePlayer->allPlayers[] resolution,
// rawDescription spell cooldown parsing, and SR-gated ObjectiveTimer calculation.
//
// POLICY COMPLIANCE:
// - Enemy summoner spell cooldown tracking is REMOVED and must not be exposed here.
// - Active-game live coaching is gated by `LIVE_COACH_ENABLED` (= false).

use crate::domain::protocol::LIVE_COACH_ENABLED;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

const _: bool = LIVE_COACH_ENABLED;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LivePlayerScores {
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub creep_score: u32,
    pub ward_score: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LivePlayerItem {
    pub item_id: u32,
    pub count: u32,
    pub display_name: String,
    pub price: u32,
    pub slot: i32,
    pub can_use: bool,
    pub consumable: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LiveSpellInfo {
    pub display_name: String,
    pub raw_description: String,
    pub parsed_cooldowns: Vec<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LiveSummonerSpells {
    pub spell_one: LiveSpellInfo,
    pub spell_two: LiveSpellInfo,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LivePlayer {
    pub summoner_name: String,
    pub riot_id: String,
    pub riot_id_game_name: String,
    pub riot_id_tag_line: String,
    pub champion_name: String,
    pub raw_champion_name: String,
    pub team: String,
    pub position: String,
    pub level: u32,
    pub is_dead: bool,
    pub respawn_timer: f64,
    pub is_bot: bool,
    pub scores: LivePlayerScores,
    pub items: Vec<LivePlayerItem>,
    pub summoner_spells: LiveSummonerSpells,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AbilitySlot {
    pub id: String,
    pub display_name: String,
    pub ability_level: u32,
    pub raw_description: String,
    pub parsed_cooldowns: Vec<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ActiveAbilities {
    pub q: AbilitySlot,
    pub w: AbilitySlot,
    pub e: AbilitySlot,
    pub r: AbilitySlot,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ChampionStats {
    pub current_health: f64,
    pub max_health: f64,
    pub attack_damage: f64,
    pub ability_power: f64,
    pub armor: f64,
    pub magic_resist: f64,
    pub attack_speed: f64,
    pub move_speed: f64,
    pub ability_haste: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ActivePlayerState {
    pub summoner_name: String,
    pub riot_id: String,
    pub riot_id_game_name: String,
    pub champion_name: String,
    pub team: String,
    pub position: String,
    pub level: u32,
    pub current_gold: f64,
    pub scores: LivePlayerScores,
    pub items: Vec<LivePlayerItem>,
    pub abilities: ActiveAbilities,
    pub champion_stats: ChampionStats,
    pub all_players_index: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LiveGameEvent {
    pub event_id: Option<u64>,
    pub event_name: String,
    pub event_time: f64,
    pub killer_name: Option<String>,
    pub victim_name: Option<String>,
    pub assisters: Vec<String>,
    pub dragon_type: Option<String>,
    pub stolen: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AllyTeammate {
    pub role: String,
    pub champion_name: String,
    pub timer_key: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectiveKind {
    Dragon,
    ElderDragon,
    Voidgrubs,
    RiftHerald,
    Baron,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectiveTimer {
    pub kind: ObjectiveKind,
    pub label: String,
    pub remaining_secs: u64,
    pub ready: bool,
    pub respawn_secs: u64,
}

impl ObjectiveTimer {
    pub fn formatted_remaining(&self) -> String {
        if self.ready || self.remaining_secs == 0 {
            "READY".to_string()
        } else {
            format!(
                "{:02}:{:02}",
                self.remaining_secs / 60,
                self.remaining_secs % 60
            )
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LiveGameState {
    pub game_time_secs: f64,
    pub game_mode: String,
    pub map_name: String,
    pub map_number: u32,
    pub map_terrain: String,
    pub active_player: ActivePlayerState,
    pub all_players: Vec<LivePlayer>,
    pub events: Vec<LiveGameEvent>,
}

impl LiveGameState {
    /// Parse Riot Live Client Data (`allgamedata` JSON) into a typed `LiveGameState`.
    /// Resolves `activePlayer` against `allPlayers[]` by `summonerName` (with `riotId` /
    /// `riotIdGameName` / `championName` fallbacks) so scores, items, and team/position are populated.
    pub fn from_allgamedata(raw: &Value) -> Option<Self> {
        if !raw.is_object() {
            return None;
        }
        let game_data = raw.get("gameData")?;
        let game_time_secs = game_data
            .get("gameTime")
            .and_then(Value::as_f64)
            .filter(|v| v.is_finite() && *v >= 0.0)
            .unwrap_or(0.0);
        let game_mode = str_field(game_data, "gameMode");
        let map_name = str_field(game_data, "mapName");
        let map_number = game_data
            .get("mapNumber")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32;
        let map_terrain = str_field(game_data, "mapTerrain");

        let all_players = raw
            .get("allPlayers")
            .and_then(Value::as_array)
            .map(|players| players.iter().map(parse_player).collect::<Vec<_>>())
            .unwrap_or_default();

        let active_raw = raw.get("activePlayer").unwrap_or(&Value::Null);
        let active_player = resolve_active_player(active_raw, &all_players);

        let events = raw
            .get("events")
            .and_then(|e| e.get("Events"))
            .and_then(Value::as_array)
            .map(|arr| arr.iter().filter_map(parse_event).collect::<Vec<_>>())
            .unwrap_or_default();

        Some(Self {
            game_time_secs,
            game_mode,
            map_name,
            map_number,
            map_terrain,
            active_player,
            all_players,
            events,
        })
    }

    /// True only for Summoner's Rift game modes / Map 11.
    pub fn is_summoners_rift(&self) -> bool {
        let mode = self.game_mode.trim().to_ascii_uppercase();
        if mode == "ARAM"
            || mode == "CHERRY"
            || mode == "ARENA"
            || mode == "URF"
            || mode == "ONEFORALL"
            || mode == "STRAWBERRY"
            || mode == "NEXUSBLITZ"
            || mode == "TFT"
        {
            return false;
        }
        if matches!(mode.as_str(), "CLASSIC" | "SWIFTPLAY" | "PRACTICETOOL") {
            return self.map_number == 0
                || self.map_number == 11
                || self.map_name.is_empty()
                || self.map_name.eq_ignore_ascii_case("Map11")
                || self.map_name.to_ascii_lowercase().contains("summoner");
        }
        self.map_number == 11 || self.map_name.eq_ignore_ascii_case("Map11")
    }

    /// Returns allied teammates (excluding active player) for manual ally R reference.
    pub fn allied_teammates(&self) -> Vec<AllyTeammate> {
        let own_team = if !self.active_player.team.is_empty() {
            self.active_player.team.as_str()
        } else if let Some(idx) = self.active_player.all_players_index {
            self.all_players[idx].team.as_str()
        } else {
            return Vec::new();
        };
        if own_team.is_empty() {
            return Vec::new();
        }
        self.all_players
            .iter()
            .enumerate()
            .filter(|(idx, player)| {
                Some(*idx) != self.active_player.all_players_index
                    && player.team.eq_ignore_ascii_case(own_team)
            })
            .take(4)
            .map(|(idx, player)| {
                let role = if player.position.is_empty() {
                    "ALLY".to_string()
                } else {
                    player.position.to_ascii_uppercase()
                };
                let champion_name = if player.champion_name.is_empty() {
                    "Ally".to_string()
                } else {
                    player.champion_name.clone()
                };
                let key_id = if !player.riot_id.is_empty() {
                    player.riot_id.clone()
                } else if !player.summoner_name.is_empty() {
                    player.summoner_name.clone()
                } else {
                    format!("{champion_name}-{idx}")
                };
                AllyTeammate {
                    role,
                    champion_name,
                    timer_key: format!("ally-ult-{key_id}"),
                }
            })
            .collect()
    }

    /// Computes SR-gated epic objective timers (Dragon/Elder, Voidgrubs/Herald, Baron).
    /// Returns an empty vector on non-Summoner's Rift modes and guarantees at most one BARON entry.
    pub fn objective_timers(&self) -> Vec<ObjectiveTimer> {
        if !self.is_summoners_rift() {
            return Vec::new();
        }

        let game_time = self.game_time_secs.max(0.0) as u64;
        let event_time = |ev: &LiveGameEvent| ev.event_time.max(0.0) as u64;
        let last_event_time = |name: &str| {
            self.events
                .iter()
                .filter(|ev| ev.event_name == name)
                .map(event_time)
                .max()
        };

        let dragon_kills: Vec<&LiveGameEvent> = self
            .events
            .iter()
            .filter(|ev| ev.event_name == "DragonKill")
            .collect();
        let last_dragon = dragon_kills.iter().copied().max_by_key(|ev| event_time(ev));
        let is_elder = last_dragon
            .and_then(|ev| ev.dragon_type.as_deref())
            .is_some_and(|kind| kind.eq_ignore_ascii_case("Elder"));
        let dragon_respawn = if is_elder { 360 } else { 300 };
        let dragon_remaining = last_dragon
            .map(|ev| event_time(ev).saturating_add(dragon_respawn))
            .unwrap_or(300)
            .saturating_sub(game_time);
        let dragon_kind = if is_elder {
            ObjectiveKind::ElderDragon
        } else {
            ObjectiveKind::Dragon
        };
        let dragon_label = if is_elder { "ELDER" } else { "DRAGON" };

        let mut timers = vec![ObjectiveTimer {
            kind: dragon_kind,
            label: dragon_label.to_string(),
            remaining_secs: dragon_remaining,
            ready: dragon_remaining == 0,
            respawn_secs: dragon_respawn,
        }];

        let horde_kills: Vec<&LiveGameEvent> = self
            .events
            .iter()
            .filter(|ev| ev.event_name == "HordeKill")
            .collect();
        let latest_horde = horde_kills.iter().copied().map(event_time).max();
        let herald_killed = last_event_time("HeraldKill").is_some();

        if game_time < 300 {
            let grubs_rem = 300 - game_time;
            timers.push(ObjectiveTimer {
                kind: ObjectiveKind::Voidgrubs,
                label: "GRUBS".to_string(),
                remaining_secs: grubs_rem,
                ready: false,
                respawn_secs: 240,
            });
        } else if game_time < 840 {
            if horde_kills.len() >= 6 {
                let herald_rem = 840 - game_time;
                timers.push(ObjectiveTimer {
                    kind: ObjectiveKind::RiftHerald,
                    label: "HERALD".to_string(),
                    remaining_secs: herald_rem,
                    ready: false,
                    respawn_secs: 0,
                });
            } else if horde_kills.len() >= 3 {
                let next_wave = latest_horde.unwrap_or(game_time).saturating_add(240);
                let grubs_rem = if next_wave <= 840 && game_time < next_wave {
                    next_wave - game_time
                } else {
                    0
                };
                timers.push(ObjectiveTimer {
                    kind: ObjectiveKind::Voidgrubs,
                    label: "GRUBS".to_string(),
                    remaining_secs: grubs_rem,
                    ready: grubs_rem == 0,
                    respawn_secs: 240,
                });
            } else {
                timers.push(ObjectiveTimer {
                    kind: ObjectiveKind::Voidgrubs,
                    label: "GRUBS".to_string(),
                    remaining_secs: 0,
                    ready: true,
                    respawn_secs: 240,
                });
            }
        } else if game_time < 1185 && !herald_killed {
            timers.push(ObjectiveTimer {
                kind: ObjectiveKind::RiftHerald,
                label: "HERALD".to_string(),
                remaining_secs: 0,
                ready: true,
                respawn_secs: 0,
            });
        }

        // Single canonical Baron timer (never duplicated when top pit transitions to Baron).
        let baron_remaining = last_event_time("BaronKill")
            .map(|t| t.saturating_add(360))
            .unwrap_or(1200)
            .saturating_sub(game_time);
        timers.push(ObjectiveTimer {
            kind: ObjectiveKind::Baron,
            label: "BARON".to_string(),
            remaining_secs: baron_remaining,
            ready: baron_remaining == 0,
            respawn_secs: 360,
        });

        // Structural deduplication guard by ObjectiveKind.
        let mut seen = HashSet::new();
        timers.retain(|entry| seen.insert(entry.kind));
        timers
    }
}

/// Convenience helper for raw `allgamedata` JSON payloads.
pub fn objective_timers_from_json(raw: &Value) -> Vec<ObjectiveTimer> {
    LiveGameState::from_allgamedata(raw)
        .map(|state| state.objective_timers())
        .unwrap_or_default()
}

/// Extracts cooldown numeric values from Riot `rawDescription` strings
/// (e.g. `"300s Cooldown"`, `"Cooldown: 12 / 10 / 8 / 6 / 4"`, `"<font>90</font> second cooldown"`).
pub fn parse_spell_cooldowns(raw_description: &str) -> Vec<f64> {
    let cleaned = strip_markup_tags(raw_description);
    let lower = cleaned.to_ascii_lowercase();

    // 1. Prefer numbers immediately following "cooldown" (e.g., "Cooldown: 14 / 12 / 10")
    if let Some(pos) = lower.find("cooldown") {
        let after = &cleaned[pos + "cooldown".len()..];
        let after_nums = extract_cooldown_sequence(after);
        if !after_nums.is_empty() {
            return after_nums;
        }

        // 2. Otherwise inspect immediately before "cooldown" (e.g., "300s Cooldown", "90 second cooldown")
        let before = cleaned[..pos].trim_end();
        let before_trimmed = strip_trailing_unit_words(before);
        if let Some(num) = trailing_number(before_trimmed) {
            return vec![num];
        }
    }

    // 3. Fallback for bracketed or standalone seconds notations like "[180s]" or "180s"
    for token in cleaned.split_whitespace() {
        let trimmed = token.trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '.');
        if let Some(num_str) = trimmed
            .strip_suffix('s')
            .or_else(|| trimmed.strip_suffix('S'))
        {
            if let Ok(val) = num_str.parse::<f64>() {
                if val.is_finite() && val > 0.0 {
                    return vec![val];
                }
            }
        }
    }

    Vec::new()
}

/// Convenience wrapper returning the primary (rank 1 / base) cooldown in seconds if parsed.
pub fn parse_primary_cooldown(raw_description: &str) -> Option<f64> {
    parse_spell_cooldowns(raw_description).into_iter().next()
}

fn resolve_active_player(active_raw: &Value, all_players: &[LivePlayer]) -> ActivePlayerState {
    let summoner_name = str_field(active_raw, "summonerName");
    let riot_id = str_field(active_raw, "riotId");
    let riot_id_game_name = str_field(active_raw, "riotIdGameName");
    let raw_champion_name = str_field(active_raw, "championName");

    let matched_idx = find_active_player_index(
        all_players,
        &summoner_name,
        &riot_id,
        &riot_id_game_name,
        &raw_champion_name,
    );
    let matched = matched_idx.map(|idx| &all_players[idx]);

    let resolved_summoner_name = if !summoner_name.is_empty() {
        summoner_name
    } else {
        matched.map(|p| p.summoner_name.clone()).unwrap_or_default()
    };
    let resolved_riot_id = if !riot_id.is_empty() {
        riot_id
    } else {
        matched.map(|p| p.riot_id.clone()).unwrap_or_default()
    };
    let resolved_riot_id_game_name = if !riot_id_game_name.is_empty() {
        riot_id_game_name
    } else {
        matched
            .map(|p| p.riot_id_game_name.clone())
            .unwrap_or_default()
    };
    let resolved_champion_name = if !raw_champion_name.is_empty() {
        raw_champion_name
    } else {
        matched.map(|p| p.champion_name.clone()).unwrap_or_default()
    };

    let active_scores = active_raw.get("scores").map(parse_scores);
    let scores = match (active_scores, matched) {
        (Some(s), Some(p)) if is_empty_scores(&s) => p.scores.clone(),
        (Some(s), _) => s,
        (None, Some(p)) => p.scores.clone(),
        (None, None) => LivePlayerScores::default(),
    };

    let items = active_raw
        .get("items")
        .and_then(Value::as_array)
        .filter(|arr| !arr.is_empty())
        .map(|arr| arr.iter().map(parse_item).collect())
        .or_else(|| matched.map(|p| p.items.clone()))
        .unwrap_or_default();

    let level = active_raw
        .get("level")
        .and_then(Value::as_u64)
        .map(|v| v as u32)
        .filter(|v| *v > 0)
        .or_else(|| matched.map(|p| p.level))
        .unwrap_or(1);

    let current_gold = active_raw
        .get("currentGold")
        .and_then(Value::as_f64)
        .filter(|v| v.is_finite())
        .unwrap_or(0.0);

    let abilities = active_raw
        .get("abilities")
        .map(parse_abilities)
        .unwrap_or_default();
    let champion_stats = active_raw
        .get("championStats")
        .map(parse_champion_stats)
        .unwrap_or_default();

    ActivePlayerState {
        summoner_name: resolved_summoner_name,
        riot_id: resolved_riot_id,
        riot_id_game_name: resolved_riot_id_game_name,
        champion_name: resolved_champion_name,
        team: matched.map(|p| p.team.clone()).unwrap_or_default(),
        position: matched.map(|p| p.position.clone()).unwrap_or_default(),
        level,
        current_gold,
        scores,
        items,
        abilities,
        champion_stats,
        all_players_index: matched_idx,
    }
}

fn find_active_player_index(
    all_players: &[LivePlayer],
    summoner_name: &str,
    riot_id: &str,
    riot_id_game_name: &str,
    champion_name: &str,
) -> Option<usize> {
    let norm = |s: &str| s.trim().to_ascii_lowercase();
    let s_norm = norm(summoner_name);
    let r_norm = norm(riot_id);
    let rg_norm = norm(riot_id_game_name);
    let c_norm = norm(champion_name);

    // 1. Exact summonerName match (case-insensitive)
    if !s_norm.is_empty() {
        if let Some(idx) = all_players
            .iter()
            .position(|p| norm(&p.summoner_name) == s_norm)
        {
            return Some(idx);
        }
        // SummonerName without #TAG compared to riotIdGameName or prefix
        let s_base = s_norm.split('#').next().unwrap_or(&s_norm);
        if let Some(idx) = all_players.iter().position(|p| {
            norm(&p.riot_id_game_name) == s_base
                || norm(&p.summoner_name).split('#').next() == Some(s_base)
        }) {
            return Some(idx);
        }
    }

    // 2. Exact riotId match
    if !r_norm.is_empty() {
        if let Some(idx) = all_players.iter().position(|p| norm(&p.riot_id) == r_norm) {
            return Some(idx);
        }
    }

    // 3. riotIdGameName match
    if !rg_norm.is_empty() {
        if let Some(idx) = all_players.iter().position(|p| {
            norm(&p.riot_id_game_name) == rg_norm
                || norm(&p.summoner_name) == rg_norm
                || norm(&p.riot_id).split('#').next() == Some(rg_norm.as_str())
        }) {
            return Some(idx);
        }
    }

    // 4. Champion name fallback when unambiguous
    if !c_norm.is_empty() {
        let mut matches = all_players
            .iter()
            .enumerate()
            .filter(|(_, p)| norm(&p.champion_name) == c_norm);
        if let Some((first_idx, _)) = matches.next() {
            if matches.next().is_none() {
                return Some(first_idx);
            }
        }
    }

    None
}

fn is_empty_scores(scores: &LivePlayerScores) -> bool {
    scores.kills == 0
        && scores.deaths == 0
        && scores.assists == 0
        && scores.creep_score == 0
        && scores.ward_score == 0.0
}

fn parse_player(raw: &Value) -> LivePlayer {
    let spells = raw.get("summonerSpells").unwrap_or(&Value::Null);
    LivePlayer {
        summoner_name: str_field(raw, "summonerName"),
        riot_id: str_field(raw, "riotId"),
        riot_id_game_name: str_field(raw, "riotIdGameName"),
        riot_id_tag_line: str_field(raw, "riotIdTagLine"),
        champion_name: str_field(raw, "championName"),
        raw_champion_name: str_field(raw, "rawChampionName"),
        team: str_field(raw, "team"),
        position: str_field(raw, "position"),
        level: raw.get("level").and_then(Value::as_u64).unwrap_or(1) as u32,
        is_dead: raw.get("isDead").and_then(Value::as_bool).unwrap_or(false),
        respawn_timer: raw
            .get("respawnTimer")
            .and_then(Value::as_f64)
            .filter(|v| v.is_finite())
            .unwrap_or(0.0),
        is_bot: raw.get("isBot").and_then(Value::as_bool).unwrap_or(false),
        scores: raw.get("scores").map(parse_scores).unwrap_or_default(),
        items: raw
            .get("items")
            .and_then(Value::as_array)
            .map(|items| items.iter().map(parse_item).collect())
            .unwrap_or_default(),
        summoner_spells: LiveSummonerSpells {
            spell_one: parse_spell_info(spells.get("summonerSpellOne").unwrap_or(&Value::Null)),
            spell_two: parse_spell_info(spells.get("summonerSpellTwo").unwrap_or(&Value::Null)),
        },
    }
}

fn parse_scores(raw: &Value) -> LivePlayerScores {
    LivePlayerScores {
        kills: raw.get("kills").and_then(Value::as_u64).unwrap_or(0) as u32,
        deaths: raw.get("deaths").and_then(Value::as_u64).unwrap_or(0) as u32,
        assists: raw.get("assists").and_then(Value::as_u64).unwrap_or(0) as u32,
        creep_score: raw.get("creepScore").and_then(Value::as_u64).unwrap_or(0) as u32,
        ward_score: raw
            .get("wardScore")
            .and_then(Value::as_f64)
            .filter(|v| v.is_finite())
            .unwrap_or(0.0),
    }
}

fn parse_item(raw: &Value) -> LivePlayerItem {
    LivePlayerItem {
        item_id: raw.get("itemID").and_then(Value::as_u64).unwrap_or(0) as u32,
        count: raw.get("count").and_then(Value::as_u64).unwrap_or(0) as u32,
        display_name: str_field(raw, "displayName"),
        price: raw.get("price").and_then(Value::as_u64).unwrap_or(0) as u32,
        slot: raw.get("slot").and_then(Value::as_i64).unwrap_or(-1) as i32,
        can_use: raw.get("canUse").and_then(Value::as_bool).unwrap_or(false),
        consumable: raw
            .get("consumable")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }
}

fn parse_spell_info(raw: &Value) -> LiveSpellInfo {
    let raw_description = str_field(raw, "rawDescription");
    let parsed_cooldowns = parse_spell_cooldowns(&raw_description);
    LiveSpellInfo {
        display_name: str_field(raw, "displayName"),
        raw_description,
        parsed_cooldowns,
    }
}

fn parse_abilities(raw: &Value) -> ActiveAbilities {
    let slot = |key: &str| -> AbilitySlot {
        let entry = raw.get(key).unwrap_or(&Value::Null);
        let raw_description = str_field(entry, "rawDescription");
        let parsed_cooldowns = parse_spell_cooldowns(&raw_description);
        AbilitySlot {
            id: str_field(entry, "id"),
            display_name: str_field(entry, "displayName"),
            ability_level: entry
                .get("abilityLevel")
                .and_then(Value::as_u64)
                .unwrap_or(0) as u32,
            raw_description,
            parsed_cooldowns,
        }
    };
    ActiveAbilities {
        q: slot("Q"),
        w: slot("W"),
        e: slot("E"),
        r: slot("R"),
    }
}

fn parse_champion_stats(raw: &Value) -> ChampionStats {
    let num = |key: &str| {
        raw.get(key)
            .and_then(Value::as_f64)
            .filter(|v| v.is_finite())
            .unwrap_or(0.0)
    };
    ChampionStats {
        current_health: num("currentHealth"),
        max_health: num("maxHealth"),
        attack_damage: num("attackDamage"),
        ability_power: num("abilityPower"),
        armor: num("armor"),
        magic_resist: num("magicResist"),
        attack_speed: num("attackSpeed"),
        move_speed: num("moveSpeed"),
        ability_haste: num("abilityHaste"),
    }
}

fn parse_event(raw: &Value) -> Option<LiveGameEvent> {
    let event_name = raw.get("EventName").and_then(Value::as_str)?.to_string();
    Some(LiveGameEvent {
        event_id: raw.get("EventID").and_then(Value::as_u64),
        event_name,
        event_time: raw
            .get("EventTime")
            .and_then(Value::as_f64)
            .filter(|v| v.is_finite() && *v >= 0.0)
            .unwrap_or(0.0),
        killer_name: raw
            .get("KillerName")
            .and_then(Value::as_str)
            .map(str::to_owned),
        victim_name: raw
            .get("VictimName")
            .and_then(Value::as_str)
            .map(str::to_owned),
        assisters: raw
            .get("Assisters")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
        dragon_type: raw
            .get("DragonType")
            .and_then(Value::as_str)
            .map(str::to_owned),
        stolen: raw.get("Stolen").and_then(Value::as_str).map(str::to_owned),
    })
}

fn str_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn strip_markup_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => {
                in_tag = true;
                out.push(' ');
            }
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn extract_cooldown_sequence(slice: &str) -> Vec<f64> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut started = false;

    for ch in slice.chars() {
        if ch.is_ascii_digit() || (ch == '.' && !current.is_empty() && !current.contains('.')) {
            current.push(ch);
            started = true;
        } else {
            if !current.is_empty() {
                if let Ok(val) = current.parse::<f64>() {
                    if val.is_finite() && val > 0.0 {
                        values.push(val);
                    }
                }
                current.clear();
            }
            if started {
                let allowed_separator =
                    ch.is_whitespace() || matches!(ch, '/' | '-' | ':' | 's' | 'S' | ',');
                if !allowed_separator {
                    break;
                }
            } else if !(ch.is_whitespace() || matches!(ch, ':' | '-' | '(' | '[')) {
                break;
            }
        }
    }
    if !current.is_empty() {
        if let Ok(val) = current.parse::<f64>() {
            if val.is_finite() && val > 0.0 {
                values.push(val);
            }
        }
    }
    values
}

fn strip_trailing_unit_words(input: &str) -> &str {
    let mut s = input.trim_end();
    for suffix in ["seconds", "second", "secs", "sec", "s"] {
        if let Some(stripped) = s.strip_suffix(suffix) {
            if stripped.is_empty()
                || stripped
                    .chars()
                    .last()
                    .is_some_and(|ch| ch.is_ascii_digit() || ch.is_whitespace())
            {
                s = stripped.trim_end();
                break;
            }
        }
    }
    s
}

fn trailing_number(input: &str) -> Option<f64> {
    let end = input.len();
    let mut start = end;
    for (idx, ch) in input.char_indices().rev() {
        if ch.is_ascii_digit() || ch == '.' {
            start = idx;
        } else {
            break;
        }
    }
    if start == end {
        return None;
    }
    input[start..end]
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && *v > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn resolves_active_player_from_all_players_by_summoner_name() {
        let payload = json!({
            "gameData": {
                "gameTime": 612.5,
                "gameMode": "CLASSIC",
                "mapName": "Map11",
                "mapNumber": 11
            },
            "activePlayer": {
                "summonerName": "RiftTester",
                "level": 9,
                "currentGold": 1450.25,
                "championStats": {
                    "attackDamage": 115.0,
                    "abilityPower": 220.0
                }
            },
            "allPlayers": [
                {
                    "summonerName": "AllyTop",
                    "championName": "Aatrox",
                    "team": "ORDER",
                    "position": "TOP",
                    "level": 9,
                    "scores": { "kills": 2, "deaths": 1, "assists": 3, "creepScore": 140, "wardScore": 11.0 },
                    "items": []
                },
                {
                    "summonerName": "RiftTester",
                    "riotId": "RiftTester#EUW1",
                    "championName": "Ahri",
                    "team": "ORDER",
                    "position": "MIDDLE",
                    "level": 9,
                    "scores": { "kills": 5, "deaths": 1, "assists": 4, "creepScore": 162, "wardScore": 14.5 },
                    "items": [{ "itemID": 6655, "count": 1, "displayName": "Luden's Companion", "price": 2850, "slot": 0, "canUse": false }]
                }
            ],
            "events": { "Events": [] }
        });

        let state = LiveGameState::from_allgamedata(&payload).expect("valid allgamedata");
        assert_eq!(state.active_player.champion_name, "Ahri");
        assert_eq!(state.active_player.team, "ORDER");
        assert_eq!(state.active_player.position, "MIDDLE");
        assert_eq!(state.active_player.scores.kills, 5);
        assert_eq!(state.active_player.scores.creep_score, 162);
        assert_eq!(state.active_player.items.len(), 1);
        assert_eq!(state.active_player.all_players_index, Some(1));
        assert_eq!(state.allied_teammates().len(), 1);
        assert_eq!(state.allied_teammates()[0].champion_name, "Aatrox");
    }

    #[test]
    fn parses_raw_description_cooldowns() {
        assert_eq!(
            parse_spell_cooldowns(
                "Teleport to an allied unit. <font color='#999999'>360s Cooldown</font>"
            ),
            vec![360.0]
        );
        assert_eq!(
            parse_spell_cooldowns("Cooldown: 12 / 10.5 / 9 / 7.5 / 6s"),
            vec![12.0, 10.5, 9.0, 7.5, 6.0]
        );
        assert_eq!(
            parse_primary_cooldown("Grants brief untargetability (90 second cooldown)."),
            Some(90.0)
        );
    }

    #[test]
    fn sr_objective_timers_never_duplicate_baron_and_gate_non_sr() {
        // 1. Pre-Baron (< 1200s) with Herald killed after 14:00 -> must NOT emit two BARON rows
        let late_mid_sr = json!({
            "gameData": { "gameTime": 1000.0, "gameMode": "CLASSIC", "mapName": "Map11", "mapNumber": 11 },
            "activePlayer": { "summonerName": "P1" },
            "allPlayers": [{ "summonerName": "P1", "team": "ORDER", "championName": "Ahri" }],
            "events": {
                "Events": [
                    { "EventName": "DragonKill", "EventTime": 800.0, "DragonType": "Infernal" },
                    { "EventName": "HeraldKill", "EventTime": 900.0 }
                ]
            }
        });
        let timers = objective_timers_from_json(&late_mid_sr);
        let baron_count = timers
            .iter()
            .filter(|t| t.kind == ObjectiveKind::Baron || t.label == "BARON")
            .count();
        assert_eq!(baron_count, 1, "must never emit duplicate BARON timers");
        assert_eq!(timers.len(), 2);
        assert_eq!(timers[0].label, "DRAGON");
        assert_eq!(timers[0].remaining_secs, 100);
        assert_eq!(timers[1].label, "BARON");
        assert_eq!(timers[1].remaining_secs, 200);

        // 2. Post-20:00 (>= 1200s) -> exactly one BARON timer
        let post_baron_sr = json!({
            "gameData": { "gameTime": 1400.0, "gameMode": "CLASSIC", "mapName": "Map11", "mapNumber": 11 },
            "activePlayer": { "summonerName": "P1" },
            "allPlayers": [{ "summonerName": "P1", "team": "ORDER", "championName": "Ahri" }],
            "events": {
                "Events": [
                    { "EventName": "DragonKill", "EventTime": 1350.0, "DragonType": "Elder" },
                    { "EventName": "BaronKill", "EventTime": 1300.0 }
                ]
            }
        });
        let timers_late = objective_timers_from_json(&post_baron_sr);
        assert_eq!(
            timers_late
                .iter()
                .filter(|t| t.kind == ObjectiveKind::Baron)
                .count(),
            1
        );
        assert_eq!(timers_late[0].kind, ObjectiveKind::ElderDragon);
        assert_eq!(timers_late[0].remaining_secs, 310);
        assert_eq!(timers_late[1].remaining_secs, 260);

        // 3. Non-SR mode (ARAM) -> SR-gated to empty
        let aram = json!({
            "gameData": { "gameTime": 400.0, "gameMode": "ARAM", "mapName": "Map12", "mapNumber": 12 },
            "activePlayer": { "summonerName": "P1" },
            "allPlayers": [],
            "events": { "Events": [] }
        });
        assert!(objective_timers_from_json(&aram).is_empty());
    }
}
