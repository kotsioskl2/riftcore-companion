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

    /// True on Howling Abyss: ARAM, ARAM Mayhem (reported as `KIWI`) or any other map 12 game.
    /// Gates the ARAM-only HUD such as the health relic timers.
    pub fn is_aram(&self) -> bool {
        is_aram_mode(&self.game_mode, self.map_number)
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

    /// Computes SR-gated epic objective timers (Dragon/Elder, Voidgrubs/Herald, Baron) at the
    /// snapshot's own game time. See [`Self::objective_timers_at`].
    pub fn objective_timers(&self) -> Vec<ObjectiveTimer> {
        self.objective_timers_at(self.game_time_secs)
    }

    /// Computes SR-gated epic objective timers at `game_time_secs` (e.g. the HUD's smoothed
    /// clock, which runs ahead of the ~1 s snapshots). Returns an empty vector outside
    /// Summoner's Rift, never duplicates BARON, and is sorted soonest-first (READY first).
    ///
    /// Countdowns use whole game seconds so they tick together with the mm:ss game clock.
    pub fn objective_timers_at(&self, game_time_secs: f64) -> Vec<ObjectiveTimer> {
        if !self.is_summoners_rift() {
            return Vec::new();
        }

        let now = whole_game_secs(game_time_secs);
        let event_time = |ev: &LiveGameEvent| {
            if ev.event_time.is_finite() {
                ev.event_time.max(0.0)
            } else {
                0.0
            }
        };
        let event_times = |name: &'static str| {
            self.events
                .iter()
                .filter(move |ev| ev.event_name == name)
                .map(event_time)
        };
        let timer = |kind, label: &str, ready_at: f64, respawn_secs: u64| {
            let remaining_secs = countdown_secs(ready_at, now);
            ObjectiveTimer {
                kind,
                label: label.to_string(),
                remaining_secs,
                ready: remaining_secs == 0,
                respawn_secs,
            }
        };

        // Summoner's Rift timings (patch 26.x, League wiki): drakes from 5:00 and 5 min after
        // each kill; once a team has four drakes (Dragon Soul) every later spawn is the Elder
        // Dragon, 6 min after the previous kill. Voidgrubs: one group of three at 8:00, gone at
        // 14:45. Rift Herald: once at 15:00, gone at 19:45. Baron: 20:00, then 6 min after a kill.
        let dragon_kills: Vec<&LiveGameEvent> = self
            .events
            .iter()
            .filter(|ev| ev.event_name == "DragonKill")
            .collect();
        let last_dragon = dragon_kills
            .iter()
            .copied()
            .max_by(|a, b| event_time(a).total_cmp(&event_time(b)));
        let elder_next =
            last_dragon.is_some_and(is_elder_kill) || self.dragon_soul_taken(&dragon_kills);
        let dragon_respawn = if elder_next { 360 } else { 300 };
        let dragon_ready_at =
            last_dragon.map_or(300.0, |ev| event_time(ev) + dragon_respawn as f64);
        let (dragon_kind, dragon_label) = if elder_next {
            (ObjectiveKind::ElderDragon, "ELDER")
        } else {
            (ObjectiveKind::Dragon, "DRAGON")
        };
        let mut timers = vec![timer(
            dragon_kind,
            dragon_label,
            dragon_ready_at,
            dragon_respawn,
        )];

        // Top pit: Voidgrubs (no respawn), then the Rift Herald (no respawn), then Baron.
        let grubs_cleared = event_times("HordeKill").count() >= 3;
        let herald_killed = event_times("HeraldKill").next().is_some();
        if now < 480.0 {
            timers.push(timer(ObjectiveKind::Voidgrubs, "GRUBS", 480.0, 0));
        } else if now < 885.0 && !grubs_cleared {
            // Up (READY) until all three die or the camp leaves at 14:45.
            timers.push(timer(ObjectiveKind::Voidgrubs, "GRUBS", now, 0));
        } else if now < 1185.0 && !herald_killed {
            timers.push(timer(ObjectiveKind::RiftHerald, "HERALD", 900.0, 0));
        }

        // Single canonical Baron timer (never duplicated when top pit transitions to Baron).
        let baron_ready_at = event_times("BaronKill")
            .reduce(f64::max)
            .map_or(1200.0, |t| t + 360.0);
        timers.push(timer(ObjectiveKind::Baron, "BARON", baron_ready_at, 360));

        // Structural deduplication guard by ObjectiveKind.
        let mut seen = HashSet::new();
        timers.retain(|entry| seen.insert(entry.kind));
        timers.sort_by_key(|t| (t.remaining_secs, t.kind.pit_order()));
        timers
    }

    /// True once either team has taken four elemental drakes (Dragon Soul); every later dragon
    /// spawn is then the Elder Dragon. Kills whose killer can't be matched to a team still count
    /// toward the total: seven elemental drakes always mean one team has four.
    fn dragon_soul_taken(&self, dragon_kills: &[&LiveGameEvent]) -> bool {
        let (mut order, mut chaos, mut total) = (0, 0, 0);
        for kill in dragon_kills.iter().filter(|ev| !is_elder_kill(ev)) {
            total += 1;
            match kill
                .killer_name
                .as_deref()
                .and_then(|name| self.team_of(name))
            {
                Some(team) if team.eq_ignore_ascii_case("ORDER") => order += 1,
                Some(team) if team.eq_ignore_ascii_case("CHAOS") => chaos += 1,
                _ => {}
            }
        }
        order >= 4 || chaos >= 4 || total >= 7
    }

    /// Team ("ORDER" or "CHAOS") of the player whose summoner name or Riot ID is `name`.
    fn team_of(&self, name: &str) -> Option<&str> {
        let name = name.trim();
        if name.is_empty() {
            return None;
        }
        self.all_players
            .iter()
            .find(|player| {
                [
                    &player.summoner_name,
                    &player.riot_id,
                    &player.riot_id_game_name,
                ]
                .iter()
                .any(|candidate| !candidate.is_empty() && candidate.eq_ignore_ascii_case(name))
            })
            .map(|player| player.team.as_str())
    }
}

/// Decides whether a match is still live from the stream of client snapshots. A single
/// live-client poll can time out under load; that must not close the HUD (and reopen it a
/// second later) or forget that the user dismissed it, so the match only counts as over after
/// [`Self::GRACE`] without live data.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LiveGameLatch {
    seen_at: Option<std::time::Instant>,
}

impl LiveGameLatch {
    pub const GRACE: std::time::Duration = std::time::Duration::from_secs(5);

    /// Feeds whether the latest snapshot had live match data; returns whether the match is live.
    pub fn update(&mut self, has_live_data: bool, now: std::time::Instant) -> bool {
        if has_live_data {
            self.seen_at = Some(now);
        }
        self.seen_at
            .is_some_and(|seen| now.saturating_duration_since(seen) < Self::GRACE)
    }
}

fn is_elder_kill(ev: &LiveGameEvent) -> bool {
    ev.dragon_type
        .as_deref()
        .is_some_and(|kind| kind.eq_ignore_ascii_case("Elder"))
}

impl ObjectiveKind {
    /// Stable tie-break order for equally soon timers: dragon pit, top pit, then Baron.
    fn pit_order(self) -> u8 {
        match self {
            Self::Dragon | Self::ElderDragon => 0,
            Self::Voidgrubs | Self::RiftHerald => 1,
            Self::Baron => 2,
        }
    }
}

/// Whole game seconds shown by the mm:ss clock; negative or non-finite input reads as 0.
pub fn whole_game_secs(game_time_secs: f64) -> f64 {
    if game_time_secs.is_finite() {
        game_time_secs.max(0.0).floor()
    } else {
        0.0
    }
}

/// Whole seconds left until `ready_at` (game seconds) at game time `now`, rounded up so the
/// countdown reads 00:01 during the last second and 0 (READY) from `ready_at` on. Never negative.
pub fn countdown_secs(ready_at: f64, now: f64) -> u64 {
    let left = ready_at - now;
    if left.is_finite() && left > 0.0 {
        // The epsilon keeps float noise (e.g. 100.000000001) from adding a whole second.
        (left - 1e-6).ceil().max(0.0) as u64
    } else {
        0
    }
}

/// [`LiveGameState::is_aram`] for callers holding raw `gameMode` / `mapNumber` values.
pub fn is_aram_mode(game_mode: &str, map_number: u32) -> bool {
    let mode = game_mode.trim();
    map_number == crate::domain::items::MAP_ARAM
        || mode.eq_ignore_ascii_case("ARAM")
        || mode.eq_ignore_ascii_case("KIWI")
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

/// Ability slot letters in `[Q, W, E, R]` rank order.
pub const SKILL_SLOTS: [&str; 4] = ["Q", "W", "E", "R"];

/// Next ability to rank up when a skill point is unspent (`ranks` = `[Q, W, E, R]`).
/// Follows `order` (18-level recommended order), respecting rank n -> level 2n-1, R at
/// 6/11/16 (max 3). Orders with > 3 R ranks (Udyr) treat R as a basic 6-rank ability.
pub fn next_skill_up(level: u32, ranks: [u32; 4], order: &[String]) -> Option<&'static str> {
    if ranks.iter().sum::<u32>() >= level {
        return None;
    }
    let slot_of = |s: &str| {
        SKILL_SLOTS
            .iter()
            .position(|k| k.eq_ignore_ascii_case(s.trim()))
    };
    let r_basic = order.iter().filter(|s| slot_of(s) == Some(3)).count() > 3;
    let can_rank = |slot: usize| {
        let rank = ranks[slot];
        if slot == 3 && !r_basic {
            rank < 3 && level >= [6, 11, 16][rank as usize]
        } else {
            rank < if r_basic { 6 } else { 5 } && level > 2 * rank
        }
    };
    let mut expected = [0u32; 4];
    for slot in order.iter().filter_map(|s| slot_of(s)) {
        expected[slot] += 1;
        if expected[slot] > ranks[slot] && can_rank(slot) {
            return Some(SKILL_SLOTS[slot]);
        }
    }
    [3, 0, 2, 1]
        .into_iter()
        .find(|&slot| can_rank(slot))
        .map(|slot| SKILL_SLOTS[slot])
}

/// Team kill totals `(your team, enemy team)` for `team` (`ORDER`/`CHAOS`).
pub fn team_kill_score(players: &[LivePlayer], team: &str) -> (u32, u32) {
    players.iter().fold((0, 0), |(ally, enemy), p| {
        if p.team.eq_ignore_ascii_case(team) {
            (ally + p.scores.kills, enemy)
        } else {
            (ally, enemy + p.scores.kills)
        }
    })
}

/// Kill participation percent; `None` when the team has no kills yet.
pub fn kill_participation_pct(kills: u32, assists: u32, team_kills: u32) -> Option<u32> {
    (team_kills > 0).then(|| ((kills + assists) * 100 / team_kills).min(100))
}

/// CS/min pace bucket: `None` before 2:00, else green (>= 8), gold (6-8), red (< 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CsPace {
    Neutral,
    Good,
    Fair,
    Low,
}

pub fn cs_pace(cs_per_min: f64, game_time_secs: f64) -> CsPace {
    match cs_per_min {
        _ if game_time_secs < 120.0 => CsPace::Neutral,
        v if v >= 8.0 => CsPace::Good,
        v if v >= 6.0 => CsPace::Fair,
        _ => CsPace::Low,
    }
}

/// First recommended core item not yet in `owned` item ids.
pub fn next_build_item(core: &[u32], owned: &[u32]) -> Option<u32> {
    core.iter()
        .copied()
        .find(|id| *id > 0 && !owned.contains(id))
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
        // Soonest first: Baron (260 s) before Elder (310 s).
        assert_eq!(timers_late[0].kind, ObjectiveKind::Baron);
        assert_eq!(timers_late[0].remaining_secs, 260);
        assert_eq!(timers_late[1].kind, ObjectiveKind::ElderDragon);
        assert_eq!(timers_late[1].remaining_secs, 310);

        // 3. Non-SR mode (ARAM) -> SR-gated to empty
        let aram = json!({
            "gameData": { "gameTime": 400.0, "gameMode": "ARAM", "mapName": "Map12", "mapNumber": 12 },
            "activePlayer": { "summonerName": "P1" },
            "allPlayers": [],
            "events": { "Events": [] }
        });
        assert!(objective_timers_from_json(&aram).is_empty());
    }

    #[test]
    fn aram_mayhem_counts_as_aram_but_not_summoners_rift() {
        let game = |mode: &str, map: u32| {
            LiveGameState::from_allgamedata(
                &json!({ "gameData": { "gameMode": mode, "mapNumber": map } }),
            )
            .expect("valid payload")
        };
        // ARAM Mayhem's Live Client gameMode is "KIWI" on Howling Abyss (map 12).
        for (mode, map) in [("ARAM", 12), ("KIWI", 12), ("kiwi", 0), ("SOMEEVENT", 12)] {
            let state = game(mode, map);
            assert!(state.is_aram(), "{mode} on map {map}");
            assert!(!state.is_summoners_rift(), "{mode} on map {map}");
        }
        for (mode, map) in [("CLASSIC", 11), ("CHERRY", 30), ("URF", 11), ("", 0)] {
            assert!(!game(mode, map).is_aram(), "{mode} on map {map}");
        }
        assert!(is_aram_mode(" Kiwi ", 0));
    }

    fn sr_state(game_time: f64, events: Value) -> LiveGameState {
        LiveGameState::from_allgamedata(&json!({
            "gameData": { "gameTime": game_time, "gameMode": "CLASSIC", "mapNumber": 11 },
            "activePlayer": { "summonerName": "P1" },
            "allPlayers": [{ "summonerName": "P1", "team": "ORDER", "championName": "Ahri" }],
            "events": { "Events": events }
        }))
        .expect("valid SR payload")
    }

    #[test]
    fn objective_timers_count_down_whole_seconds_soonest_first_and_never_negative() {
        // Dragon killed at 10:00.4 respawns at 15:00.4; all three grubs died by 10:30 and don't
        // respawn, so the top pit's next objective is the Herald at 15:00.
        let state = sr_state(
            700.0,
            json!([
                { "EventName": "DragonKill", "EventTime": 600.4, "DragonType": "Fire" },
                { "EventName": "HordeKill", "EventTime": 628.0 },
                { "EventName": "HordeKill", "EventTime": 629.0 },
                { "EventName": "HordeKill", "EventTime": 630.0 }
            ]),
        );
        let labels = |timers: &[ObjectiveTimer]| {
            timers
                .iter()
                .map(|t| (t.label.clone(), t.remaining_secs))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            labels(&state.objective_timers_at(700.0)),
            [
                ("HERALD".to_string(), 200),
                ("DRAGON".to_string(), 201),
                ("BARON".to_string(), 500)
            ]
        );
        // Between snapshots the smoothed clock drives the countdown, one whole second at a time.
        let at = |t: f64| state.objective_timers_at(t);
        assert_eq!(at(700.9)[0].remaining_secs, 200);
        assert_eq!(at(701.0)[0].remaining_secs, 199);
        // Last second reads 00:01, then READY (never negative) and ready timers sort first.
        let dragon = |t: f64| {
            at(t)
                .into_iter()
                .find(|timer| timer.kind == ObjectiveKind::Dragon)
                .expect("dragon timer")
        };
        assert_eq!(dragon(900.0).formatted_remaining(), "00:01");
        assert_eq!(dragon(901.0).remaining_secs, 0);
        assert!(dragon(5000.0).ready);
        assert_eq!(at(901.0)[0].label, "DRAGON");
        for garbage in [f64::NAN, f64::INFINITY, -50.0] {
            let timers = state.objective_timers_at(garbage);
            assert!(timers.iter().all(|t| t.remaining_secs <= 1200), "{garbage}");
        }
    }

    #[test]
    fn top_pit_follows_patch_26_grub_and_herald_timings() {
        let pit = |events: Value, now: f64| {
            sr_state(now, events)
                .objective_timers_at(now)
                .into_iter()
                .find(|t| matches!(t.kind, ObjectiveKind::Voidgrubs | ObjectiveKind::RiftHerald))
                .map(|t| (t.label, t.remaining_secs))
        };
        let grub = |t: f64| json!({ "EventName": "HordeKill", "EventTime": t });
        let grubs = |left: u64| Some(("GRUBS".to_string(), left));
        let herald = |left: u64| Some(("HERALD".to_string(), left));
        // One group of three at 8:00; it doesn't respawn.
        assert_eq!(pit(json!([]), 0.0), grubs(480));
        assert_eq!(pit(json!([]), 400.0), grubs(80));
        assert_eq!(pit(json!([]), 480.0), grubs(0));
        // Two of three killed: still up.
        assert_eq!(pit(json!([grub(500.0), grub(501.0)]), 560.0), grubs(0));
        // All three dead at 8:22: the Herald at 15:00 is next.
        let cleared = json!([grub(500.0), grub(501.0), grub(502.0)]);
        assert_eq!(pit(cleared.clone(), 600.0), herald(300));
        // Untouched grubs leave at 14:45.
        assert_eq!(pit(json!([]), 884.0), grubs(0));
        assert_eq!(pit(json!([]), 885.0), herald(15));
        // Herald is up from 15:00 until she is killed or leaves at 19:45.
        assert_eq!(pit(cleared.clone(), 1000.0), herald(0));
        let herald_dead = json!([{ "EventName": "HeraldKill", "EventTime": 960.0 }]);
        assert_eq!(pit(herald_dead, 1000.0), None);
        assert_eq!(pit(cleared, 1185.0), None);
    }

    #[test]
    fn live_game_latch_rides_out_a_failed_poll_but_ends_with_the_match() {
        let t0 = std::time::Instant::now();
        let at = |secs: f64| t0 + std::time::Duration::from_secs_f64(secs);
        let mut latch = LiveGameLatch::default();
        assert!(!latch.update(false, at(0.0)), "no match yet");
        assert!(latch.update(true, at(1.0)));
        // One failed poll (about a second without data) keeps the match live.
        assert!(latch.update(false, at(2.0)));
        assert!(latch.update(true, at(2.9)));
        // The match ended: live until the grace runs out, then over.
        assert!(latch.update(false, at(7.8)));
        assert!(!latch.update(false, at(7.9)));
        assert!(!latch.update(false, at(60.0)));
        // The next match is picked up at once.
        assert!(latch.update(true, at(61.0)));
    }

    #[test]
    fn elder_dragon_follows_dragon_soul() {
        let state = |kills: &[(&str, &str, f64)], now: f64| {
            let events: Vec<_> = kills
                .iter()
                .map(|(killer, kind, t)| {
                    json!({ "EventName": "DragonKill", "EventTime": t, "DragonType": kind, "KillerName": killer })
                })
                .collect();
            LiveGameState::from_allgamedata(&json!({
                "gameData": { "gameTime": now, "gameMode": "CLASSIC", "mapNumber": 11 },
                "activePlayer": { "summonerName": "Blue1" },
                "allPlayers": [
                    { "summonerName": "Blue1", "team": "ORDER", "championName": "Ahri" },
                    { "summonerName": "Red1", "team": "CHAOS", "championName": "Zed" }
                ],
                "events": { "Events": events }
            }))
            .expect("valid SR payload")
        };
        let dragon = |kills: &[(&str, &str, f64)], now: f64| {
            state(kills, now)
                .objective_timers_at(now)
                .into_iter()
                .find(|t| matches!(t.kind, ObjectiveKind::Dragon | ObjectiveKind::ElderDragon))
                .map(|t| (t.label, t.remaining_secs))
        };
        let three = [
            ("Blue1", "Fire", 300.0),
            ("Red1", "Water", 700.0),
            ("Blue1", "Earth", 1100.0),
            ("Blue1", "Air", 1500.0),
        ];
        // Blue has three drakes: the next spawn is still an elemental drake, 5 min later.
        assert_eq!(dragon(&three, 1600.0), Some(("DRAGON".to_string(), 200)));
        // Blue's fourth drake is Dragon Soul: the Elder Dragon spawns 6 min later...
        let mut soul = three.to_vec();
        soul.push(("Blue1", "Air", 1900.0));
        assert_eq!(dragon(&soul[..], 2000.0), Some(("ELDER".to_string(), 260)));
        // ...and respawns 6 min after each Elder kill.
        let mut elder = soul.clone();
        elder.push(("Red1", "Elder", 2260.0));
        assert_eq!(dragon(&elder[..], 2300.0), Some(("ELDER".to_string(), 320)));
        // Killers that can't be matched to a team: seven drakes always mean a soul.
        let unknown: Vec<_> = (0..7)
            .map(|i| ("?", "Fire", 300.0 + 400.0 * f64::from(i)))
            .collect();
        assert_eq!(
            dragon(&unknown[..6], 2400.0),
            Some(("DRAGON".to_string(), 200))
        );
        assert_eq!(
            dragon(&unknown[..], 2800.0),
            Some(("ELDER".to_string(), 260))
        );
    }

    fn order(p: &str) -> Vec<String> {
        crate::domain::builds::skill_order(p)
    }

    #[test]
    fn next_skill_follows_order_and_rank_rules() {
        let o = order("Q > E > W");
        assert_eq!(next_skill_up(1, [0, 0, 0, 0], &o), Some("Q"));
        assert_eq!(next_skill_up(1, [1, 0, 0, 0], &o), None);
        assert_eq!(next_skill_up(3, [1, 0, 1, 0], &o), Some("W"));
        assert_eq!(next_skill_up(6, [3, 1, 1, 0], &o), Some("R"));
        // Off-order ranks: level 2 can't take Q rank 2 (needs level 3).
        assert_eq!(next_skill_up(2, [1, 0, 0, 0], &o), Some("E"));
        assert_eq!(next_skill_up(18, [5, 5, 5, 3], &o), None);
    }

    #[test]
    fn next_skill_udyr_treats_r_as_basic() {
        let o = order("Q>R>E");
        assert_eq!(next_skill_up(2, [1, 0, 0, 0], &o), Some("R"));
        assert_eq!(next_skill_up(5, [2, 0, 1, 1], &o), Some("Q"));
        assert_eq!(next_skill_up(11, [6, 0, 1, 3], &o), Some("R"));
    }

    #[test]
    fn kill_participation_and_team_sides() {
        let mk = |team: &str, kills| LivePlayer {
            team: team.into(),
            scores: LivePlayerScores {
                kills,
                ..Default::default()
            },
            ..Default::default()
        };
        let players = [mk("ORDER", 4), mk("ORDER", 8), mk("CHAOS", 9)];
        assert_eq!(team_kill_score(&players, "ORDER"), (12, 9));
        assert_eq!(team_kill_score(&players, "CHAOS"), (9, 12));
        assert_eq!(kill_participation_pct(4, 5, 12), Some(75));
        assert_eq!(kill_participation_pct(0, 0, 0), None);
        assert_eq!(cs_pace(9.0, 60.0), CsPace::Neutral);
        assert_eq!(cs_pace(8.0, 600.0), CsPace::Good);
        assert_eq!(cs_pace(6.5, 600.0), CsPace::Fair);
        assert_eq!(cs_pace(5.9, 600.0), CsPace::Low);
        assert_eq!(next_build_item(&[3078, 3053], &[3078]), Some(3053));
        assert_eq!(next_build_item(&[3078], &[3078]), None);
    }
}
