//! Loading-screen scouting: per-player cards (rank, champion experience, recent form) built
//! from the Riot API with the user's own developer key.
//!
//! Inputs are the raw Riot API responses (`league-v4` entries, `champion-mastery-v4` for the
//! champion the player locked in, and a few recent `match-v5` matches), so the tag logic is
//! testable without network access. Tags describe the player's own public history only.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A player to scout, taken from Live Client Data `allPlayers`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoutTarget {
    /// `GameName#TAG`.
    pub riot_id: String,
    pub champion_name: String,
    /// Numeric champion id for the mastery lookup (0 when unknown).
    pub champion_id: u32,
    /// Live `position` (`TOP`, `JUNGLE`, `MIDDLE`, `BOTTOM`, `UTILITY` or empty).
    pub position: String,
    pub ally: bool,
}

/// How a tag reads: good news for the player's team, bad news, or neutral information.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TagTone {
    Good,
    Bad,
    Neutral,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScoutTag {
    pub text: String,
    pub tone: TagTone,
}

/// Scouting result for one player.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScoutCard {
    pub riot_id: String,
    pub champion_name: String,
    pub ally: bool,
    /// `"Gold II 45 LP"`, `"Unranked"`.
    pub rank: String,
    /// Ranked wins/losses in the shown queue.
    pub ranked_record: Option<(u32, u32)>,
    /// Mastery points on this champion (`Some(0)` when never played).
    pub mastery_points: Option<u64>,
    /// Recent matches looked at, games on this champion among them, and wins on it.
    pub recent_games: u32,
    pub champ_games: u32,
    pub champ_wins: u32,
    /// Average KDA ratio over the recent matches.
    pub recent_kda: Option<f64>,
    pub tags: Vec<ScoutTag>,
    /// Set when the lookup failed; the other fields are then empty.
    pub error: Option<String>,
}

impl ScoutCard {
    /// A card that only reports why the player could not be scouted.
    pub fn failed(target: &ScoutTarget, error: impl Into<String>) -> Self {
        Self {
            riot_id: target.riot_id.clone(),
            champion_name: target.champion_name.clone(),
            ally: target.ally,
            rank: String::new(),
            ranked_record: None,
            mastery_points: None,
            recent_games: 0,
            champ_games: 0,
            champ_wins: 0,
            recent_kda: None,
            tags: Vec::new(),
            error: Some(error.into()),
        }
    }

    /// The Riot ID's game name (without `#TAG`).
    pub fn display_name(&self) -> &str {
        self.riot_id
            .split_once('#')
            .map_or(self.riot_id.as_str(), |(name, _)| name)
    }

    /// Ranked win rate in percent.
    pub fn ranked_win_rate(&self) -> Option<u32> {
        let (w, l) = self.ranked_record?;
        (w + l > 0).then(|| (w * 100 + (w + l) / 2) / (w + l))
    }
}

/// Recent matches fetched per player (keeps a 10-player scout inside a development key's
/// 100 requests / 2 minutes).
pub const RECENT_MATCHES: usize = 4;

/// Riot's ranked tiers that have no divisions.
fn apex(tier: &str) -> bool {
    matches!(
        tier.to_ascii_uppercase().as_str(),
        "MASTER" | "GRANDMASTER" | "CHALLENGER"
    )
}

fn title_case(raw: &str) -> String {
    let lower = raw.to_ascii_lowercase();
    let mut chars = lower.chars();
    chars
        .next()
        .map(|c| c.to_ascii_uppercase().to_string() + chars.as_str())
        .unwrap_or_default()
}

/// Role name for a `teamPosition` / live `position`.
pub fn role_label(position: &str) -> Option<&'static str> {
    Some(match position.trim().to_ascii_uppercase().as_str() {
        "TOP" => "Top",
        "JUNGLE" => "Jungle",
        "MIDDLE" | "MID" => "Mid",
        "BOTTOM" | "BOT" | "ADC" => "Bot",
        "UTILITY" | "SUPPORT" => "Support",
        _ => return None,
    })
}

/// The solo queue entry, or flex when the player has no solo rank.
fn ranked_entry(ranked: &Value) -> Option<&Value> {
    let entries = ranked.as_array()?;
    let queue = |q: &str| {
        entries
            .iter()
            .find(|e| e.get("queueType").and_then(Value::as_str) == Some(q))
    };
    queue("RANKED_SOLO_5x5").or_else(|| queue("RANKED_FLEX_SR"))
}

struct Recent {
    games: u32,
    champ_games: u32,
    champ_wins: u32,
    kda: Option<f64>,
    /// Most played position and how many games it was played.
    main_role: Option<(String, u32)>,
    /// Results newest first.
    results: Vec<bool>,
    team_damage_share: Option<f64>,
}

fn recent_form(matches: &[Value], puuid: &str, champion: &str) -> Recent {
    let mut recent = Recent {
        games: 0,
        champ_games: 0,
        champ_wins: 0,
        kda: None,
        main_role: None,
        results: Vec::new(),
        team_damage_share: None,
    };
    let mut kda_sum = 0.0;
    let mut damage = (0.0, 0u32);
    let mut roles: Vec<(String, u32)> = Vec::new();
    let champ_norm = normalize(champion);
    for m in matches {
        let Some(me) = m
            .pointer("/info/participants")
            .and_then(Value::as_array)
            .and_then(|ps| {
                ps.iter()
                    .find(|p| p.get("puuid").and_then(Value::as_str) == Some(puuid))
            })
        else {
            continue;
        };
        let num = |key: &str| me.get(key).and_then(Value::as_f64).unwrap_or(0.0);
        let win = me.get("win").and_then(Value::as_bool).unwrap_or(false);
        recent.games += 1;
        recent.results.push(win);
        kda_sum += (num("kills") + num("assists")) / num("deaths").max(1.0);
        if let Some(share) = me
            .pointer("/challenges/teamDamagePercentage")
            .and_then(Value::as_f64)
        {
            damage.0 += share;
            damage.1 += 1;
        }
        let played = me.get("championName").and_then(Value::as_str).unwrap_or("");
        if !champ_norm.is_empty() && normalize(played) == champ_norm {
            recent.champ_games += 1;
            recent.champ_wins += u32::from(win);
        }
        let position = me.get("teamPosition").and_then(Value::as_str).unwrap_or("");
        if role_label(position).is_some() {
            match roles.iter_mut().find(|(r, _)| r == position) {
                Some((_, n)) => *n += 1,
                None => roles.push((position.to_string(), 1)),
            }
        }
    }
    if recent.games > 0 {
        recent.kda = Some(kda_sum / recent.games as f64);
    }
    if damage.1 > 0 {
        recent.team_damage_share = Some(damage.0 / damage.1 as f64);
    }
    recent.main_role = roles.into_iter().max_by_key(|(_, n)| *n);
    recent
}

fn normalize(name: &str) -> String {
    name.chars()
        .filter(char::is_ascii_alphanumeric)
        .collect::<String>()
        .to_ascii_lowercase()
}

/// Builds the card for `target` from its Riot API data.
///
/// - `ranked`: `league-v4 entries/by-puuid` array.
/// - `mastery`: `champion-mastery-v4 …/by-champion/{id}`; `Some(Value::Null)` when Riot
///   answered 404 (never played), `None` when the lookup failed.
/// - `matches`: recent `match-v5` match objects.
pub fn build_card(
    target: &ScoutTarget,
    puuid: &str,
    ranked: &Value,
    mastery: Option<&Value>,
    matches: &[Value],
) -> ScoutCard {
    let mut card = ScoutCard::failed(target, "");
    card.error = None;
    let mut tags = Vec::new();
    let mut tag = |text: String, tone| tags.push(ScoutTag { text, tone });

    match ranked_entry(ranked) {
        Some(entry) => {
            let s = |k: &str| entry.get(k).and_then(Value::as_str).unwrap_or("");
            let n = |k: &str| entry.get(k).and_then(Value::as_u64).unwrap_or(0);
            let tier = s("tier");
            let lp = n("leaguePoints");
            card.rank = if apex(tier) {
                format!("{} {lp} LP", title_case(tier))
            } else {
                format!("{} {} {lp} LP", title_case(tier), s("rank"))
            };
            if s("queueType") == "RANKED_FLEX_SR" {
                card.rank.push_str(" (Flex)");
            }
            card.ranked_record = Some((n("wins") as u32, n("losses") as u32));
            let flag = |k: &str| entry.get(k).and_then(Value::as_bool).unwrap_or(false);
            if flag("hotStreak") {
                tag("Hot streak".into(), TagTone::Good);
            }
            if flag("veteran") {
                tag("Veteran".into(), TagTone::Neutral);
            }
            if flag("freshBlood") {
                tag("New to this tier".into(), TagTone::Neutral);
            }
        }
        None => card.rank = "Unranked".to_string(),
    }

    card.mastery_points =
        mastery.map(|m| m.get("championPoints").and_then(Value::as_u64).unwrap_or(0));

    let recent = recent_form(matches, puuid, &target.champion_name);
    card.recent_games = recent.games;
    card.champ_games = recent.champ_games;
    card.champ_wins = recent.champ_wins;
    card.recent_kda = recent.kda;

    // Champion experience.
    let points = card.mastery_points;
    if points.is_some_and(|p| p >= 100_000)
        || (recent.games >= 3 && recent.champ_games * 2 > recent.games)
    {
        tag(format!("{} main", target.champion_name), TagTone::Good);
    } else if points == Some(0) {
        tag("First time on champion".into(), TagTone::Bad);
    } else if points.is_some_and(|p| p < 10_000) {
        tag("Low mastery".into(), TagTone::Bad);
    }

    // Role: main role, or autofilled when the recent main role is a different lane.
    let current_role = role_label(&target.position);
    if let Some((main, games)) = recent.main_role.as_ref().filter(|(_, n)| *n >= 2) {
        let main_label = role_label(main);
        match (current_role, main_label) {
            (Some(now), Some(main)) if now != main && games * 2 > recent.games => {
                tag(format!("Autofilled ({main} main)"), TagTone::Bad);
            }
            (_, Some(main)) => tag(format!("{main} main"), TagTone::Neutral),
            _ => {}
        }
    }

    // Recent form.
    let streak = recent
        .results
        .iter()
        .take_while(|w| **w == recent.results.first().copied().unwrap_or(false))
        .count();
    if streak >= 3 {
        match recent.results.first() {
            Some(true) => tag(format!("{streak} win streak"), TagTone::Good),
            Some(false) => tag(format!("{streak} loss streak"), TagTone::Bad),
            None => {}
        }
    }
    if let Some(share) = recent.team_damage_share {
        let support = current_role == Some("Support");
        if share >= 0.28 {
            tag("High team damage".into(), TagTone::Good);
        } else if share < 0.15 && !support {
            tag("Low team damage".into(), TagTone::Bad);
        }
    }

    card.tags = tags;
    card
}

/// Builds the scout list from a live game: everyone except the active player, allies
/// first, skipping bots and players without a Riot ID. `champion_id` resolves a champion
/// name to its numeric id.
pub fn scout_targets(
    live: &crate::domain::live_game::LiveGameState,
    champion_id: impl Fn(&str) -> u32,
) -> Vec<ScoutTarget> {
    let me = live.active_player.all_players_index;
    let my_team = me
        .and_then(|i| live.all_players.get(i))
        .map(|p| p.team.clone())
        .unwrap_or_default();
    let mut targets: Vec<ScoutTarget> = live
        .all_players
        .iter()
        .enumerate()
        .filter(|(i, p)| Some(*i) != me && !p.is_bot && p.riot_id.contains('#'))
        .map(|(_, p)| ScoutTarget {
            riot_id: p.riot_id.clone(),
            champion_name: p.champion_name.clone(),
            champion_id: champion_id(&p.champion_name),
            position: p.position.clone(),
            ally: !my_team.is_empty() && p.team.eq_ignore_ascii_case(&my_team),
        })
        .collect();
    targets.sort_by_key(|t| !t.ally);
    targets
}

/// Identifies a match's roster so a scout runs once per game.
pub fn roster_key(targets: &[ScoutTarget]) -> String {
    let mut ids: Vec<&str> = targets.iter().map(|t| t.riot_id.as_str()).collect();
    ids.sort_unstable();
    ids.join("|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn target(position: &str) -> ScoutTarget {
        ScoutTarget {
            riot_id: "Faker#KR1".into(),
            champion_name: "Ahri".into(),
            champion_id: 103,
            position: position.into(),
            ally: false,
        }
    }

    fn game(champ: &str, pos: &str, win: bool, k: u32, d: u32, a: u32, share: f64) -> Value {
        json!({"info": {"participants": [
            {"puuid": "other", "championName": "Lux", "teamPosition": "MIDDLE", "win": !win},
            {"puuid": "p1", "championName": champ, "teamPosition": pos, "win": win,
             "kills": k, "deaths": d, "assists": a,
             "challenges": {"teamDamagePercentage": share}}
        ]}})
    }

    #[test]
    fn ranked_line_prefers_solo_queue_and_reads_flags() {
        let ranked = json!([
            {"queueType": "RANKED_FLEX_SR", "tier": "SILVER", "rank": "I", "leaguePoints": 3, "wins": 1, "losses": 1},
            {"queueType": "RANKED_SOLO_5x5", "tier": "GOLD", "rank": "II", "leaguePoints": 45,
             "wins": 30, "losses": 20, "hotStreak": true}
        ]);
        let card = build_card(&target("MIDDLE"), "p1", &ranked, None, &[]);
        assert_eq!(card.rank, "Gold II 45 LP");
        assert_eq!(card.ranked_record, Some((30, 20)));
        assert_eq!(card.ranked_win_rate(), Some(60));
        assert!(card
            .tags
            .iter()
            .any(|t| t.text == "Hot streak" && t.tone == TagTone::Good));

        let apex = json!([{"queueType": "RANKED_SOLO_5x5", "tier": "MASTER", "rank": "I", "leaguePoints": 210}]);
        assert_eq!(
            build_card(&target(""), "p1", &apex, None, &[]).rank,
            "Master 210 LP"
        );
        assert_eq!(
            build_card(&target(""), "p1", &json!([]), None, &[]).rank,
            "Unranked"
        );
    }

    #[test]
    fn champion_main_role_and_streak_tags() {
        let matches = vec![
            game("Ahri", "MIDDLE", true, 8, 2, 6, 0.31),
            game("Ahri", "MIDDLE", true, 5, 3, 7, 0.30),
            game("Ahri", "MIDDLE", true, 9, 1, 4, 0.33),
            game("Syndra", "MIDDLE", false, 2, 5, 3, 0.25),
        ];
        let card = build_card(
            &target("MIDDLE"),
            "p1",
            &json!([]),
            Some(&json!({"championPoints": 250000})),
            &matches,
        );
        let texts: Vec<&str> = card.tags.iter().map(|t| t.text.as_str()).collect();
        assert!(texts.contains(&"Ahri main"), "{texts:?}");
        assert!(texts.contains(&"Mid main"), "{texts:?}");
        assert!(texts.contains(&"3 win streak"), "{texts:?}");
        assert!(texts.contains(&"High team damage"), "{texts:?}");
        assert_eq!(
            (card.recent_games, card.champ_games, card.champ_wins),
            (4, 3, 3)
        );
        assert!(card.recent_kda.unwrap() > 3.0);
    }

    #[test]
    fn autofill_first_time_and_loss_streak_tags() {
        let matches = vec![
            game("Garen", "TOP", false, 1, 6, 2, 0.12),
            game("Darius", "TOP", false, 3, 7, 1, 0.14),
            game("Garen", "TOP", false, 0, 4, 3, 0.10),
        ];
        let card = build_card(
            &target("UTILITY"),
            "p1",
            &json!([]),
            Some(&Value::Null),
            &matches,
        );
        let texts: Vec<&str> = card.tags.iter().map(|t| t.text.as_str()).collect();
        assert!(texts.contains(&"First time on champion"), "{texts:?}");
        assert!(texts.contains(&"Autofilled (Top main)"), "{texts:?}");
        assert!(texts.contains(&"3 loss streak"), "{texts:?}");
        // Low damage is not flagged for supports.
        assert!(!texts.contains(&"Low team damage"), "{texts:?}");
        assert_eq!(card.mastery_points, Some(0));
    }

    #[test]
    fn failed_card_and_display_name() {
        let card = ScoutCard::failed(&target("TOP"), "rate limited");
        assert_eq!(card.display_name(), "Faker");
        assert_eq!(card.error.as_deref(), Some("rate limited"));
    }

    #[test]
    fn targets_skip_self_and_bots_and_list_allies_first() {
        let raw = json!({
            "activePlayer": {"summonerName": "Me", "riotId": "Me#EUW"},
            "allPlayers": [
                {"summonerName": "Me", "riotId": "Me#EUW", "championName": "Ahri", "team": "ORDER"},
                {"riotId": "Enemy#EUW", "championName": "Lux", "team": "CHAOS", "position": "MIDDLE"},
                {"riotId": "Ally#EUW", "championName": "Garen", "team": "ORDER", "position": "TOP"},
                {"riotId": "Bot#EUW", "championName": "Annie", "team": "CHAOS", "isBot": true}
            ],
            "gameData": {"gameTime": 0.0, "gameMode": "CLASSIC"}
        });
        let live = crate::domain::live_game::LiveGameState::from_allgamedata(&raw).unwrap();
        let targets = scout_targets(&live, |_| 1);
        let ids: Vec<&str> = targets.iter().map(|t| t.riot_id.as_str()).collect();
        assert_eq!(ids, vec!["Ally#EUW", "Enemy#EUW"]);
        assert!(targets[0].ally && !targets[1].ally);
        assert_eq!(roster_key(&targets), "Ally#EUW|Enemy#EUW");
    }
}
