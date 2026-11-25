// RiftCore Native — domain::analytics
// Pure match normalization (KP%, DMG%, Gold%, DPM, GPM, VPM), multi-game trend reports,
// and data-grounded Ally vs Enemy draft composition comparisons (no fabricated stats).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Normalized single-match performance summary built from LCU or Riot Match-v5 data.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NormalizedMatchSummary {
    pub match_id: String,
    pub champion_id: u32,
    pub champion_name: String,
    pub role: String,
    pub queue_id: u64,
    pub game_mode: String,
    pub win: bool,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub kda: f64,
    pub cs: u32,
    pub cs_per_min: f64,
    pub duration_secs: u64,
    pub gold: u64,
    pub damage: u64,
    pub vision: u32,
    pub wards_placed: u32,
    pub wards_killed: u32,
    pub control_wards: u32,
    pub objective_damage: u64,
    pub turret_damage: u64,
    pub total_cc_secs: u64,
    pub team_kills: u32,
    pub team_damage: u64,
    pub team_gold: u64,
    pub kp_pct: f64,
    pub dmg_pct: f64,
    pub gold_pct: f64,
    pub dpm: f64,
    pub gpm: f64,
    pub vpm: f64,
    pub items: Vec<u32>,
    pub game_creation_ms: u64,
}

impl NormalizedMatchSummary {
    /// Computes derived ratios (`kda`, `cs_per_min`, `kp_pct`, `dmg_pct`, `gold_pct`, `dpm`, `gpm`, `vpm`)
    /// from the stored raw match counters and team totals.
    pub fn recompute_derived(&mut self) {
        let duration_mins = (self.duration_secs as f64 / 60.0).max(1.0);
        self.kda = (self.kills as f64 + self.assists as f64) / (self.deaths.max(1) as f64);
        self.cs_per_min = self.cs as f64 / duration_mins;
        self.kp_pct = if self.team_kills > 0 {
            (((self.kills + self.assists) as f64 / self.team_kills as f64) * 100.0).min(100.0)
        } else {
            0.0
        };
        self.dmg_pct = if self.team_damage > 0 {
            ((self.damage as f64 / self.team_damage as f64) * 100.0).min(100.0)
        } else {
            0.0
        };
        self.gold_pct = if self.team_gold > 0 {
            ((self.gold as f64 / self.team_gold as f64) * 100.0).min(100.0)
        } else {
            0.0
        };
        self.dpm = self.damage as f64 / duration_mins;
        self.gpm = self.gold as f64 / duration_mins;
        self.vpm = self.vision as f64 / duration_mins;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Up,
    #[default]
    Flat,
    Down,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MetricTrend {
    pub current_avg: f64,
    pub previous_avg: f64,
    pub delta: f64,
    pub direction: TrendDirection,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ChampionTrend {
    pub champion_name: String,
    pub games: u32,
    pub wins: u32,
    pub win_rate_pct: f64,
    pub avg_kda: f64,
    pub avg_cs_per_min: f64,
    pub avg_kp_pct: f64,
    pub avg_dmg_pct: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RoleTrend {
    pub role: String,
    pub games: u32,
    pub wins: u32,
    pub win_rate_pct: f64,
    pub avg_kda: f64,
    pub avg_cs_per_min: f64,
    pub avg_kp_pct: f64,
}

/// Multi-game performance aggregation and trend comparison across the last `N` games.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PlayerTrendReport {
    pub window_size: usize,
    pub sample_count: usize,
    pub wins: u32,
    pub losses: u32,
    pub win_rate_pct: f64,
    pub avg_kda: f64,
    pub avg_kills: f64,
    pub avg_deaths: f64,
    pub avg_assists: f64,
    pub avg_cs_per_min: f64,
    pub avg_kp_pct: f64,
    pub avg_dmg_pct: f64,
    pub avg_gold_pct: f64,
    pub avg_dpm: f64,
    pub avg_gpm: f64,
    pub avg_vpm: f64,
    pub kda_trend: MetricTrend,
    pub cs_per_min_trend: MetricTrend,
    pub kp_pct_trend: MetricTrend,
    pub dmg_pct_trend: MetricTrend,
    pub gold_pct_trend: MetricTrend,
    pub dpm_trend: MetricTrend,
    pub gpm_trend: MetricTrend,
    pub vpm_trend: MetricTrend,
    pub by_champion: Vec<ChampionTrend>,
    pub by_role: Vec<RoleTrend>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageProfile {
    #[default]
    Physical,
    Magic,
    Mixed,
    Utility,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DraftChampionInput {
    pub champion_id: u32,
    pub name: String,
    pub role: String,
    pub archetype: String,
    pub damage: DamageProfile,
    pub frontline: bool,
    pub hard_cc: bool,
    pub engage: bool,
    pub peel: bool,
    pub waveclear: bool,
    pub utility: bool,
    pub healing_threat: bool,
    pub shielding: bool,
}

impl DraftChampionInput {
    /// Derives heuristic composition traits from a champion's static archetype string.
    pub fn from_archetype(
        champion_id: u32,
        name: impl Into<String>,
        role: impl Into<String>,
        archetype: impl Into<String>,
    ) -> Self {
        let archetype = archetype.into();
        let arch = archetype.to_ascii_uppercase();
        let (damage, frontline, hard_cc, engage, peel, waveclear, utility, healing_threat, shielding) =
            match arch.as_str() {
                "BURST_MAGE" => (DamageProfile::Magic, false, true, false, false, true, false, false, false),
                "BATTLE_MAGE" => (DamageProfile::Magic, false, true, false, false, true, false, true, false),
                "AP_ASSASSIN" => (DamageProfile::Magic, false, false, false, false, true, false, false, false),
                "AD_ASSASSIN" => (DamageProfile::Physical, false, false, false, false, true, false, false, false),
                "CRIT_MARKSMAN" => (DamageProfile::Physical, false, false, false, false, true, false, false, false),
                "ONHIT_MARKSMAN" => (DamageProfile::Mixed, false, false, false, false, false, false, true, false),
                "JUGGERNAUT" => (DamageProfile::Physical, true, false, false, false, true, false, true, true),
                "DIVER_FIGHTER" => (DamageProfile::Physical, true, true, true, false, false, false, true, false),
                "SKIRMISHER" => (DamageProfile::Physical, false, false, false, false, true, false, true, false),
                "TANK" => (DamageProfile::Magic, true, true, true, true, false, true, false, true),
                "ENCHANTER" => (DamageProfile::Utility, false, true, false, true, false, true, true, true),
                "ENGAGE_SUPPORT" => (DamageProfile::Utility, true, true, true, true, false, true, false, false),
                "AP_SUPPORT" => (DamageProfile::Magic, false, true, false, false, true, true, false, false),
                _ => (DamageProfile::Physical, true, false, true, false, false, false, false, false),
            };

        Self {
            champion_id,
            name: name.into(),
            role: normalize_role(&role.into()),
            archetype,
            damage,
            frontline,
            hard_cc,
            engage,
            peel,
            waveclear,
            utility,
            healing_threat,
            shielding,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TeamCompositionProfile {
    pub picks_count: usize,
    pub physical_threats: usize,
    pub magic_threats: usize,
    pub mixed_threats: usize,
    pub physical_pct: f64,
    pub magic_pct: f64,
    pub frontline_score: usize,
    pub hard_cc_score: usize,
    pub engage_score: usize,
    pub peel_score: usize,
    pub waveclear_score: usize,
    pub utility_count: usize,
    pub healing_threat_count: usize,
    pub shielding_count: usize,
}

/// Side-by-side Ally vs Enemy draft matchup readout with gap and counter-item cues.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DraftComparison {
    pub ally: TeamCompositionProfile,
    pub enemy: TeamCompositionProfile,
    pub ally_missing_coverage: Vec<String>,
    pub enemy_threats: Vec<String>,
    pub recommended_defensive_focus: String,
    pub recommended_counter_tags: Vec<String>,
    pub damage_balance_summary: String,
}

/// Normalizes a single LCU `/lol-match-history/v1/products/lol/{puuid}/matches` game object.
pub fn normalize_local_match(
    game: &Value,
    puuid: &str,
    champion_names: &HashMap<u32, String>,
) -> Option<NormalizedMatchSummary> {
    let identities = game
        .get("participantIdentities")
        .and_then(Value::as_array)?;
    let identity = identities
        .iter()
        .find(|entry| {
            entry
                .get("player")
                .and_then(|p| p.get("puuid"))
                .and_then(Value::as_str)
                .is_some_and(|id| !puuid.is_empty() && id == puuid)
        })
        .or_else(|| identities.first())?;
    let participant_id = identity
        .get("participantId")
        .and_then(Value::as_u64)
        .unwrap_or(1);

    let participants = game.get("participants").and_then(Value::as_array)?;
    let participant = participants
        .iter()
        .find(|p| p.get("participantId").and_then(Value::as_u64) == Some(participant_id))?;
    let stats = participant.get("stats").unwrap_or(&Value::Null);
    let team_id = participant.get("teamId").and_then(Value::as_u64);

    let mut team_kills = 0u32;
    let mut team_damage = 0u64;
    let mut team_gold = 0u64;
    for p in participants {
        if team_id.is_some() && p.get("teamId").and_then(Value::as_u64) != team_id {
            continue;
        }
        let p_stats = p.get("stats").unwrap_or(&Value::Null);
        team_kills += p_stats.get("kills").and_then(Value::as_u64).unwrap_or(0) as u32;
        team_damage += p_stats
            .get("totalDamageDealtToChampions")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        team_gold += p_stats
            .get("goldEarned")
            .and_then(Value::as_u64)
            .unwrap_or(0);
    }

    let raw_duration = game.get("gameDuration").and_then(Value::as_u64).unwrap_or(0);
    let duration_secs = if raw_duration > 86_400 {
        raw_duration / 1000
    } else {
        raw_duration
    };

    let cs = stats
        .get("totalMinionsKilled")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        + stats
            .get("neutralMinionsKilled")
            .and_then(Value::as_u64)
            .unwrap_or(0);

    let champion_id = participant
        .get("championId")
        .and_then(Value::as_u64)
        .unwrap_or(0) as u32;
    let champion_name = champion_names
        .get(&champion_id)
        .cloned()
        .unwrap_or_else(|| "Champion".to_string());

    let timeline = participant.get("timeline").unwrap_or(&Value::Null);
    let lane = timeline.get("lane").and_then(Value::as_str).unwrap_or("");
    let timeline_role = timeline.get("role").and_then(Value::as_str).unwrap_or("");
    let role = normalize_lcu_lane_role(lane, timeline_role);

    let queue_id = game.get("queueId").and_then(Value::as_u64).unwrap_or(0);
    let raw_mode = game
        .get("gameMode")
        .and_then(Value::as_str)
        .unwrap_or("League");

    let items = (0..7)
        .map(|idx| {
            stats
                .get(format!("item{idx}"))
                .and_then(Value::as_u64)
                .unwrap_or(0) as u32
        })
        .collect::<Vec<_>>();

    let mut summary = NormalizedMatchSummary {
        match_id: game
            .get("gameId")
            .and_then(Value::as_u64)
            .map(|id| id.to_string())
            .unwrap_or_default(),
        champion_id,
        champion_name,
        role,
        queue_id,
        game_mode: friendly_mode(raw_mode, queue_id),
        win: stats.get("win").and_then(Value::as_bool).unwrap_or(false),
        kills: stats.get("kills").and_then(Value::as_u64).unwrap_or(0) as u32,
        deaths: stats.get("deaths").and_then(Value::as_u64).unwrap_or(0) as u32,
        assists: stats.get("assists").and_then(Value::as_u64).unwrap_or(0) as u32,
        cs: cs as u32,
        duration_secs,
        gold: stats.get("goldEarned").and_then(Value::as_u64).unwrap_or(0),
        damage: stats
            .get("totalDamageDealtToChampions")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        vision: stats.get("visionScore").and_then(Value::as_u64).unwrap_or(0) as u32,
        wards_placed: stats.get("wardsPlaced").and_then(Value::as_u64).unwrap_or(0) as u32,
        wards_killed: stats.get("wardsKilled").and_then(Value::as_u64).unwrap_or(0) as u32,
        control_wards: stats
            .get("visionWardsBoughtInGame")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32,
        objective_damage: stats
            .get("damageDealtToObjectives")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        turret_damage: stats
            .get("damageDealtToTurrets")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        total_cc_secs: stats
            .get("totalTimeCrowdControlDealt")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        team_kills,
        team_damage,
        team_gold,
        items,
        game_creation_ms: game
            .get("gameCreation")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        ..Default::default()
    };
    summary.recompute_derived();
    Some(summary)
}

/// Normalizes a single Riot Match-v5 match JSON payload for the target `puuid`.
pub fn normalize_riot_match(game: &Value, puuid: &str) -> Option<NormalizedMatchSummary> {
    let info = game.get("info")?;
    let participants = info.get("participants").and_then(Value::as_array)?;
    let participant = participants
        .iter()
        .find(|p| {
            p.get("puuid")
                .and_then(Value::as_str)
                .is_some_and(|id| !puuid.is_empty() && id == puuid)
        })
        .or_else(|| participants.first())?;

    let team_id = participant.get("teamId").and_then(Value::as_u64);
    let mut team_kills = 0u32;
    let mut team_damage = 0u64;
    let mut team_gold = 0u64;
    for p in participants {
        if team_id.is_some() && p.get("teamId").and_then(Value::as_u64) != team_id {
            continue;
        }
        team_kills += p.get("kills").and_then(Value::as_u64).unwrap_or(0) as u32;
        team_damage += p
            .get("totalDamageDealtToChampions")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        team_gold += p.get("goldEarned").and_then(Value::as_u64).unwrap_or(0);
    }

    let raw_duration = info.get("gameDuration").and_then(Value::as_u64).unwrap_or(0);
    let duration_secs =
        if info.get("gameEndTimestamp").is_none() && raw_duration > 86_400 {
            raw_duration / 1000
        } else {
            raw_duration
        };

    let cs = participant
        .get("totalMinionsKilled")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        + participant
            .get("neutralMinionsKilled")
            .and_then(Value::as_u64)
            .unwrap_or(0);

    let queue_id = info.get("queueId").and_then(Value::as_u64).unwrap_or(0);
    let raw_mode = info
        .get("gameMode")
        .and_then(Value::as_str)
        .unwrap_or("League");

    let raw_role = participant
        .get("teamPosition")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .or_else(|| {
            participant
                .get("individualPosition")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
        })
        .unwrap_or("UNKNOWN");

    let items = (0..7)
        .map(|idx| {
            participant
                .get(format!("item{idx}"))
                .and_then(Value::as_u64)
                .unwrap_or(0) as u32
        })
        .collect::<Vec<_>>();

    let match_id = game
        .get("metadata")
        .and_then(|m| m.get("matchId"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| {
            info.get("gameId")
                .and_then(Value::as_u64)
                .map(|id| id.to_string())
        })
        .unwrap_or_default();

    let mut summary = NormalizedMatchSummary {
        match_id,
        champion_id: participant
            .get("championId")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32,
        champion_name: participant
            .get("championName")
            .and_then(Value::as_str)
            .unwrap_or("Champion")
            .to_string(),
        role: normalize_role(raw_role),
        queue_id,
        game_mode: friendly_mode(raw_mode, queue_id),
        win: participant
            .get("win")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        kills: participant.get("kills").and_then(Value::as_u64).unwrap_or(0) as u32,
        deaths: participant
            .get("deaths")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32,
        assists: participant
            .get("assists")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32,
        cs: cs as u32,
        duration_secs,
        gold: participant
            .get("goldEarned")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        damage: participant
            .get("totalDamageDealtToChampions")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        vision: participant
            .get("visionScore")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32,
        wards_placed: participant
            .get("wardsPlaced")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32,
        wards_killed: participant
            .get("wardsKilled")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32,
        control_wards: participant
            .get("detectorWardsPlaced")
            .or_else(|| participant.get("visionWardsBoughtInGame"))
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32,
        objective_damage: participant
            .get("damageDealtToObjectives")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        turret_damage: participant
            .get("damageDealtToTurrets")
            .or_else(|| participant.get("damageDealtToBuildings"))
            .and_then(Value::as_u64)
            .unwrap_or(0),
        total_cc_secs: participant
            .get("totalTimeCCDealt")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        team_kills,
        team_damage,
        team_gold,
        items,
        game_creation_ms: info
            .get("gameCreation")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        ..Default::default()
    };
    summary.recompute_derived();

    // Fallback if Match-v5 team kills unavailable while challenges.killParticipation exists.
    if summary.team_kills == 0 {
        if let Some(kp) = participant
            .get("challenges")
            .and_then(|c| c.get("killParticipation"))
            .and_then(Value::as_f64)
            .filter(|v| v.is_finite() && *v >= 0.0)
        {
            summary.kp_pct = (kp * 100.0).min(100.0);
        }
    }

    Some(summary)
}

/// Aggregates the most recent `last_n` matches (assumed ordered newest-first)
/// into averages, metric trends, champion breakdowns, and role breakdowns.
pub fn compute_player_trend_report(
    matches: &[NormalizedMatchSummary],
    last_n: usize,
) -> PlayerTrendReport {
    let sample_count = if last_n == 0 {
        0
    } else {
        matches.len().min(last_n)
    };
    if sample_count == 0 {
        return PlayerTrendReport {
            window_size: last_n,
            ..Default::default()
        };
    }

    let slice = &matches[..sample_count];
    let n = sample_count as f64;
    let wins = slice.iter().filter(|m| m.win).count() as u32;
    let losses = sample_count as u32 - wins;

    let avg = |f: fn(&NormalizedMatchSummary) -> f64| -> f64 {
        slice.iter().map(f).sum::<f64>() / n
    };

    let build_trend = |f: fn(&NormalizedMatchSummary) -> f64, flat_eps: f64| -> MetricTrend {
        if sample_count < 2 {
            let val = f(&slice[0]);
            return MetricTrend {
                current_avg: val,
                previous_avg: val,
                delta: 0.0,
                direction: TrendDirection::Flat,
            };
        }
        let split = sample_count / 2;
        let newer = &slice[..split];
        let older = &slice[split..];
        let current_avg = newer.iter().map(f).sum::<f64>() / newer.len() as f64;
        let previous_avg = older.iter().map(f).sum::<f64>() / older.len() as f64;
        let delta = current_avg - previous_avg;
        let direction = if delta > flat_eps {
            TrendDirection::Up
        } else if delta < -flat_eps {
            TrendDirection::Down
        } else {
            TrendDirection::Flat
        };
        MetricTrend {
            current_avg,
            previous_avg,
            delta,
            direction,
        }
    };

    let mut champ_groups: HashMap<String, Vec<&NormalizedMatchSummary>> = HashMap::new();
    let mut role_groups: HashMap<String, Vec<&NormalizedMatchSummary>> = HashMap::new();
    for m in slice {
        champ_groups
            .entry(m.champion_name.clone())
            .or_default()
            .push(m);
        role_groups.entry(m.role.clone()).or_default().push(m);
    }

    let mut by_champion = champ_groups
        .into_iter()
        .map(|(champion_name, group)| {
            let games = group.len() as u32;
            let c_wins = group.iter().filter(|m| m.win).count() as u32;
            let count = group.len() as f64;
            ChampionTrend {
                champion_name,
                games,
                wins: c_wins,
                win_rate_pct: (c_wins as f64 / count) * 100.0,
                avg_kda: group.iter().map(|m| m.kda).sum::<f64>() / count,
                avg_cs_per_min: group.iter().map(|m| m.cs_per_min).sum::<f64>() / count,
                avg_kp_pct: group.iter().map(|m| m.kp_pct).sum::<f64>() / count,
                avg_dmg_pct: group.iter().map(|m| m.dmg_pct).sum::<f64>() / count,
            }
        })
        .collect::<Vec<_>>();
    by_champion.sort_by(|a, b| {
        b.games
            .cmp(&a.games)
            .then(b.wins.cmp(&a.wins))
            .then(a.champion_name.cmp(&b.champion_name))
    });

    let mut by_role = role_groups
        .into_iter()
        .map(|(role, group)| {
            let games = group.len() as u32;
            let r_wins = group.iter().filter(|m| m.win).count() as u32;
            let count = group.len() as f64;
            RoleTrend {
                role,
                games,
                wins: r_wins,
                win_rate_pct: (r_wins as f64 / count) * 100.0,
                avg_kda: group.iter().map(|m| m.kda).sum::<f64>() / count,
                avg_cs_per_min: group.iter().map(|m| m.cs_per_min).sum::<f64>() / count,
                avg_kp_pct: group.iter().map(|m| m.kp_pct).sum::<f64>() / count,
            }
        })
        .collect::<Vec<_>>();
    by_role.sort_by(|a, b| {
        b.games
            .cmp(&a.games)
            .then(b.wins.cmp(&a.wins))
            .then(a.role.cmp(&b.role))
    });

    PlayerTrendReport {
        window_size: last_n,
        sample_count,
        wins,
        losses,
        win_rate_pct: (wins as f64 / n) * 100.0,
        avg_kda: avg(|m| m.kda),
        avg_kills: avg(|m| m.kills as f64),
        avg_deaths: avg(|m| m.deaths as f64),
        avg_assists: avg(|m| m.assists as f64),
        avg_cs_per_min: avg(|m| m.cs_per_min),
        avg_kp_pct: avg(|m| m.kp_pct),
        avg_dmg_pct: avg(|m| m.dmg_pct),
        avg_gold_pct: avg(|m| m.gold_pct),
        avg_dpm: avg(|m| m.dpm),
        avg_gpm: avg(|m| m.gpm),
        avg_vpm: avg(|m| m.vpm),
        kda_trend: build_trend(|m| m.kda, 0.25),
        cs_per_min_trend: build_trend(|m| m.cs_per_min, 0.3),
        kp_pct_trend: build_trend(|m| m.kp_pct, 2.5),
        dmg_pct_trend: build_trend(|m| m.dmg_pct, 1.5),
        gold_pct_trend: build_trend(|m| m.gold_pct, 1.0),
        dpm_trend: build_trend(|m| m.dpm, 35.0),
        gpm_trend: build_trend(|m| m.gpm, 15.0),
        vpm_trend: build_trend(|m| m.vpm, 0.08),
        by_champion,
        by_role,
    }
}

/// Evaluates Ally vs Enemy team compositions from static champion archetypes & trait tags.
pub fn compare_drafts(
    ally_picks: &[DraftChampionInput],
    enemy_picks: &[DraftChampionInput],
) -> DraftComparison {
    let ally = summarize_team_comp(ally_picks);
    let enemy = summarize_team_comp(enemy_picks);

    let mut ally_missing_coverage = Vec::new();
    if ally.picks_count >= 3 {
        if ally.frontline_score == 0 {
            ally_missing_coverage.push("Frontline / durability".to_string());
        }
        if ally.magic_threats == 0 {
            ally_missing_coverage.push("Magic damage threat".to_string());
        }
        if ally.physical_threats == 0 {
            ally_missing_coverage.push("Physical damage threat".to_string());
        }
        if ally.hard_cc_score <= 1 {
            ally_missing_coverage.push("Reliable hard crowd control".to_string());
        }
        if ally.waveclear_score == 0 {
            ally_missing_coverage.push("Consistent waveclear".to_string());
        }
        if ally.picks_count >= 4 && ally.utility_count == 0 && ally.peel_score == 0 {
            ally_missing_coverage.push("Peel / utility coverage".to_string());
        }
    }

    let mut enemy_threats = Vec::new();
    let mut recommended_counter_tags = Vec::new();
    if enemy.picks_count >= 2 {
        if enemy.physical_pct >= 70.0 {
            enemy_threats.push(format!(
                "Heavy physical damage ({:.0}% physical)",
                enemy.physical_pct
            ));
            recommended_counter_tags.push("Armor".to_string());
        }
        if enemy.magic_pct >= 70.0 {
            enemy_threats.push(format!("Heavy magic damage ({:.0}% magic)", enemy.magic_pct));
            recommended_counter_tags.push("SpellBlock".to_string());
        }
        if enemy.healing_threat_count >= 2 {
            enemy_threats.push(format!(
                "High sustain / healing ({} threats)",
                enemy.healing_threat_count
            ));
            recommended_counter_tags.push("AntiHeal".to_string());
        }
        if enemy.shielding_count >= 2 {
            enemy_threats.push(format!(
                "Heavy shielding ({} sources)",
                enemy.shielding_count
            ));
            recommended_counter_tags.push("AntiShield".to_string());
        }
        if enemy.frontline_score >= 2 {
            enemy_threats.push(format!(
                "Multi-frontline durability ({} tanks/bruisers)",
                enemy.frontline_score
            ));
            recommended_counter_tags.push("ArmorPenetration".to_string());
            recommended_counter_tags.push("MagicPenetration".to_string());
        }
        if enemy.engage_score >= 2 {
            enemy_threats.push(format!(
                "Strong dive / engage setup ({} initiators)",
                enemy.engage_score
            ));
            recommended_counter_tags.push("Tenacity".to_string());
        }
    }

    let recommended_defensive_focus = if enemy.picks_count < 2 {
        "Pending enemy picks".to_string()
    } else if enemy.physical_pct >= 70.0 && enemy.healing_threat_count >= 2 {
        "Armor + Grievous Wounds".to_string()
    } else if enemy.magic_pct >= 70.0 && enemy.healing_threat_count >= 2 {
        "Magic Resist + Grievous Wounds".to_string()
    } else if enemy.physical_pct >= 70.0 {
        "Armor / Plated Steelcaps".to_string()
    } else if enemy.magic_pct >= 70.0 {
        "Magic Resist / Mercury's Treads".to_string()
    } else if enemy.engage_score >= 2 {
        "Tenacity / Stasis / Peel".to_string()
    } else {
        "Balanced Mixed Defense".to_string()
    };

    let damage_balance_summary = if ally.picks_count == 0 {
        "No ally champions selected.".to_string()
    } else if ally.physical_pct >= 75.0 {
        format!("Ally damage skewed physical ({:.0}% AD / {:.0}% AP).", ally.physical_pct, ally.magic_pct)
    } else if ally.magic_pct >= 75.0 {
        format!("Ally damage skewed magic ({:.0}% AP / {:.0}% AD).", ally.magic_pct, ally.physical_pct)
    } else {
        format!("Mixed ally damage profile ({:.0}% physical / {:.0}% magic).", ally.physical_pct, ally.magic_pct)
    };

    DraftComparison {
        ally,
        enemy,
        ally_missing_coverage,
        enemy_threats,
        recommended_defensive_focus,
        recommended_counter_tags,
        damage_balance_summary,
    }
}

fn summarize_team_comp(picks: &[DraftChampionInput]) -> TeamCompositionProfile {
    let mut profile = TeamCompositionProfile {
        picks_count: picks.len(),
        ..Default::default()
    };
    let mut phys_weight = 0.0f64;
    let mut magic_weight = 0.0f64;

    for pick in picks {
        match pick.damage {
            DamageProfile::Physical => {
                profile.physical_threats += 1;
                phys_weight += 1.0;
            }
            DamageProfile::Magic => {
                profile.magic_threats += 1;
                magic_weight += 1.0;
            }
            DamageProfile::Mixed => {
                profile.mixed_threats += 1;
                profile.physical_threats += 1;
                profile.magic_threats += 1;
                phys_weight += 0.55;
                magic_weight += 0.45;
            }
            DamageProfile::Utility => {
                magic_weight += 0.25;
            }
        }
        if pick.frontline {
            profile.frontline_score += 1;
        }
        if pick.hard_cc {
            profile.hard_cc_score += 1;
        }
        if pick.engage {
            profile.engage_score += 1;
        }
        if pick.peel {
            profile.peel_score += 1;
        }
        if pick.waveclear {
            profile.waveclear_score += 1;
        }
        if pick.utility {
            profile.utility_count += 1;
        }
        if pick.healing_threat {
            profile.healing_threat_count += 1;
        }
        if pick.shielding {
            profile.shielding_count += 1;
        }
    }

    let total_dmg = phys_weight + magic_weight;
    if total_dmg > 0.0 {
        profile.physical_pct = (phys_weight / total_dmg) * 100.0;
        profile.magic_pct = (magic_weight / total_dmg) * 100.0;
    }

    profile
}

pub fn normalize_role(raw: &str) -> String {
    match raw.trim().to_ascii_uppercase().as_str() {
        "TOP" => "TOP".to_string(),
        "JUNGLE" | "JNG" => "JUNGLE".to_string(),
        "MID" | "MIDDLE" => "MID".to_string(),
        "ADC" | "BOT" | "BOTTOM" => "ADC".to_string(),
        "SUPPORT" | "SUP" | "UTILITY" => "SUPPORT".to_string(),
        "" | "NONE" | "INVALID" | "UNKNOWN" => "UNKNOWN".to_string(),
        other => other.to_string(),
    }
}

fn normalize_lcu_lane_role(lane: &str, role: &str) -> String {
    match lane.trim().to_ascii_uppercase().as_str() {
        "TOP" => "TOP".to_string(),
        "JUNGLE" => "JUNGLE".to_string(),
        "MIDDLE" | "MID" => "MID".to_string(),
        "BOTTOM" | "BOT" => match role.trim().to_ascii_uppercase().as_str() {
            "DUO_SUPPORT" | "SUPPORT" => "SUPPORT".to_string(),
            _ => "ADC".to_string(),
        },
        _ => normalize_role(role),
    }
}

pub fn friendly_mode(game_mode: &str, queue_id: u64) -> String {
    match queue_id {
        420 => "Ranked Solo/Duo".to_string(),
        440 => "Ranked Flex".to_string(),
        450 => "ARAM".to_string(),
        400 => "Normal Draft".to_string(),
        430 => "Normal Blind".to_string(),
        480 | 490 => "Swiftplay".to_string(),
        900 | 1010 => "URF".to_string(),
        1020 => "One for All".to_string(),
        1300 => "Nexus Blitz".to_string(),
        1700 => "Arena".to_string(),
        _ => game_mode.replace('_', " "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn computes_normalized_match_summary_from_riot_match_v5() {
        let riot_match = json!({
            "metadata": { "matchId": "EUN1_123456" },
            "info": {
                "gameDuration": 1800, // 30 minutes
                "gameCreation": 1700000000000u64,
                "queueId": 420,
                "gameMode": "CLASSIC",
                "participants": [
                    {
                        "puuid": "target-puuid",
                        "teamId": 100,
                        "championId": 103,
                        "championName": "Ahri",
                        "teamPosition": "MIDDLE",
                        "win": true,
                        "kills": 10,
                        "deaths": 2,
                        "assists": 10,
                        "totalMinionsKilled": 210,
                        "neutralMinionsKilled": 30,
                        "goldEarned": 15000,
                        "totalDamageDealtToChampions": 30000,
                        "visionScore": 30,
                        "wardsPlaced": 12,
                        "wardsKilled": 4,
                        "detectorWardsPlaced": 3,
                        "damageDealtToObjectives": 8500,
                        "damageDealtToTurrets": 6200,
                        "totalTimeCCDealt": 24,
                        "item0": 6655, "item1": 3020, "item2": 4645, "item3": 3089, "item4": 0, "item5": 0, "item6": 3363
                    },
                    {
                        "puuid": "ally-1",
                        "teamId": 100,
                        "championId": 64,
                        "championName": "LeeSin",
                        "kills": 15,
                        "deaths": 5,
                        "assists": 5,
                        "goldEarned": 15000,
                        "totalDamageDealtToChampions": 30000
                    },
                    {
                        "puuid": "enemy-1",
                        "teamId": 200,
                        "championId": 238,
                        "championName": "Zed",
                        "kills": 99,
                        "goldEarned": 99999,
                        "totalDamageDealtToChampions": 99999
                    }
                ]
            }
        });

        let summary = normalize_riot_match(&riot_match, "target-puuid").expect("valid match");
        assert_eq!(summary.match_id, "EUN1_123456");
        assert_eq!(summary.role, "MID");
        assert_eq!(summary.cs, 240);
        assert!((summary.cs_per_min - 8.0).abs() < 1e-6);
        assert!((summary.kda - 10.0).abs() < 1e-6);
        assert_eq!(summary.team_kills, 25);
        assert!((summary.kp_pct - 80.0).abs() < 1e-6);
        assert!((summary.dmg_pct - 50.0).abs() < 1e-6);
        assert!((summary.gold_pct - 50.0).abs() < 1e-6);
        assert!((summary.dpm - 1000.0).abs() < 1e-6);
        assert!((summary.gpm - 500.0).abs() < 1e-6);
        assert!((summary.vpm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn computes_local_match_and_player_trend_report() {
        let mut names = HashMap::new();
        names.insert(103, "Ahri".to_string());
        let local_game = json!({
            "gameId": 777,
            "gameDuration": 1200000u64, // ms -> 20 min
            "queueId": 420,
            "gameMode": "CLASSIC",
            "participantIdentities": [{ "participantId": 1, "player": { "puuid": "me" } }],
            "participants": [
                {
                    "participantId": 1,
                    "teamId": 100,
                    "championId": 103,
                    "timeline": { "lane": "MIDDLE", "role": "SOLO" },
                    "stats": {
                        "win": true, "kills": 6, "deaths": 2, "assists": 4,
                        "totalMinionsKilled": 140, "neutralMinionsKilled": 20,
                        "goldEarned": 9000, "totalDamageDealtToChampions": 16000,
                        "visionScore": 18
                    }
                },
                {
                    "participantId": 2,
                    "teamId": 100,
                    "championId": 222,
                    "stats": { "kills": 4, "goldEarned": 9000, "totalDamageDealtToChampions": 16000 }
                }
            ]
        });

        let newest = normalize_local_match(&local_game, "me", &names).expect("local match");
        assert_eq!(newest.duration_secs, 1200);
        assert!((newest.kp_pct - 100.0).abs() < 1e-6);
        assert!((newest.dpm - 800.0).abs() < 1e-6);

        let mut older = newest.clone();
        older.win = false;
        older.kills = 2;
        older.deaths = 6;
        older.assists = 2;
        older.cs = 100;
        older.damage = 10000;
        older.gold = 6500;
        older.vision = 10;
        older.recompute_derived();

        let report = compute_player_trend_report(&[newest, older], 5);
        assert_eq!(report.sample_count, 2);
        assert_eq!(report.wins, 1);
        assert_eq!(report.losses, 1);
        assert_eq!(report.kda_trend.direction, TrendDirection::Up);
        assert_eq!(report.cs_per_min_trend.direction, TrendDirection::Up);
        assert_eq!(report.by_champion.len(), 1);
        assert_eq!(report.by_role[0].role, "MID");
    }

    #[test]
    fn compares_ally_and_enemy_draft_profiles() {
        let ally = vec![
            DraftChampionInput::from_archetype(266, "Aatrox", "TOP", "DIVER_FIGHTER"),
            DraftChampionInput::from_archetype(64, "Lee Sin", "JUNGLE", "DIVER_FIGHTER"),
            DraftChampionInput::from_archetype(238, "Zed", "MID", "AD_ASSASSIN"),
            DraftChampionInput::from_archetype(222, "Jinx", "ADC", "CRIT_MARKSMAN"),
            DraftChampionInput::from_archetype(412, "Thresh", "SUPPORT", "ENGAGE_SUPPORT"),
        ];
        let enemy = vec![
            DraftChampionInput::from_archetype(122, "Darius", "TOP", "JUGGERNAUT"),
            DraftChampionInput::from_archetype(254, "Vi", "JUNGLE", "DIVER_FIGHTER"),
            DraftChampionInput::from_archetype(91, "Talon", "MID", "AD_ASSASSIN"),
            DraftChampionInput::from_archetype(51, "Caitlyn", "ADC", "CRIT_MARKSMAN"),
            DraftChampionInput::from_archetype(267, "Nami", "SUPPORT", "ENCHANTER"),
        ];

        let cmp = compare_drafts(&ally, &enemy);
        assert!(cmp.ally.magic_threats == 0);
        assert!(cmp
            .ally_missing_coverage
            .iter()
            .any(|g| g.contains("Magic damage")));
        assert!(cmp.enemy.physical_pct >= 70.0);
        assert!(cmp.recommended_counter_tags.contains(&"Armor".to_string()));
        assert!(cmp.recommended_counter_tags.contains(&"AntiHeal".to_string()));
    }
}
