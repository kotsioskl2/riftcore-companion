//! Live benchmarking and item-value comparisons built from the Live Client Data scoreboard.
//!
//! Everything here uses only what League's own scoreboard (Tab) shows every player: level,
//! K/D/A, creep score and items. Unspent gold is private to each player, so gold comparisons
//! use the value of the items players hold (Data Dragon total cost), never "gold earned".

use crate::domain::items;
use crate::domain::live_game::{LiveGameState, LivePlayer};

/// Lane order used to pair allies with enemies (`allPlayers[].position`).
const LANE_ORDER: [&str; 5] = ["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"];

/// Gold value of one player's inventory: each item's Data Dragon total cost (the live
/// `price` when the item is unknown), times its stack count for consumables.
pub fn item_value(player: &LivePlayer) -> u32 {
    player
        .items
        .iter()
        .filter(|item| item.item_id > 0)
        .map(|item| {
            let cost = items::item_total_cost(item.item_id)
                .filter(|cost| *cost > 0)
                .unwrap_or(item.price);
            if item.consumable {
                cost * item.count.max(1)
            } else {
                cost
            }
        })
        .sum()
}

fn lane_rank(position: &str) -> usize {
    let position = position.trim().to_ascii_uppercase();
    let position = match position.as_str() {
        "MID" => "MIDDLE",
        "ADC" | "BOT" => "BOTTOM",
        "SUPPORT" => "UTILITY",
        other => other,
    };
    LANE_ORDER
        .iter()
        .position(|lane| *lane == position)
        .unwrap_or(LANE_ORDER.len())
}

/// Index of the active player in `allPlayers`, and their team.
fn me(live: &LiveGameState) -> Option<(usize, &LivePlayer)> {
    let idx = live.active_player.all_players_index?;
    live.all_players.get(idx).map(|player| (idx, player))
}

fn team_of<'a>(live: &'a LiveGameState, team: &str, ally: bool) -> Vec<&'a LivePlayer> {
    live.all_players
        .iter()
        .filter(|p| p.team.eq_ignore_ascii_case(team) == ally && !p.team.is_empty())
        .collect()
}

/// The enemy in the active player's lane (same `position`); `None` without lane positions
/// (blind pick, ARAM, custom games) or when no enemy shares it.
pub fn lane_opponent(live: &LiveGameState) -> Option<&LivePlayer> {
    let (_, me) = me(live)?;
    let lane = lane_rank(&me.position);
    if lane >= LANE_ORDER.len() || me.team.is_empty() {
        return None;
    }
    team_of(live, &me.team, false)
        .into_iter()
        .find(|p| lane_rank(&p.position) == lane)
}

/// Creep score difference (yours minus your lane opponent's).
pub fn lane_cs_diff(live: &LiveGameState) -> Option<i32> {
    let (_, me) = me(live)?;
    let opp = lane_opponent(live)?;
    Some(me.scores.creep_score as i32 - opp.scores.creep_score as i32)
}

/// One benchmark row: your value and the comparison value (higher is better for all rows).
#[derive(Clone, Debug, PartialEq)]
pub struct BenchmarkLine {
    pub label: &'static str,
    pub you: f64,
    pub them: f64,
    /// Decimal places to show.
    pub decimals: usize,
}

impl BenchmarkLine {
    /// `Some(true)` when you are ahead, `Some(false)` when behind, `None` when level.
    pub fn ahead(&self) -> Option<bool> {
        let step = 0.5 * 10f64.powi(-(self.decimals as i32));
        if (self.you - self.them).abs() < step {
            None
        } else {
            Some(self.you > self.them)
        }
    }

    pub fn format(&self, value: f64) -> String {
        format!("{value:.*}", self.decimals)
    }
}

/// You versus your lane opponent (or the enemy team's average when there is none).
#[derive(Clone, Debug, PartialEq)]
pub struct Benchmark {
    /// Who the right-hand column is: the opponent's champion, or `"Enemy avg"`.
    pub versus: String,
    pub lines: Vec<BenchmarkLine>,
}

#[derive(Default)]
struct Stats {
    cs_per_min: f64,
    item_gold_per_min: f64,
    kp: f64,
    kda: f64,
    level: f64,
}

fn kda_ratio(p: &LivePlayer) -> f64 {
    (p.scores.kills + p.scores.assists) as f64 / p.scores.deaths.max(1) as f64
}

fn team_kills(live: &LiveGameState, team: &str) -> u32 {
    team_of(live, team, true)
        .iter()
        .map(|p| p.scores.kills)
        .sum()
}

fn stats(live: &LiveGameState, p: &LivePlayer, minutes: f64) -> Stats {
    let kills = team_kills(live, &p.team);
    Stats {
        cs_per_min: p.scores.creep_score as f64 / minutes,
        item_gold_per_min: item_value(p) as f64 / minutes,
        kp: if kills > 0 {
            ((p.scores.kills + p.scores.assists) as f64 * 100.0 / kills as f64).min(100.0)
        } else {
            0.0
        },
        kda: kda_ratio(p),
        level: p.level as f64,
    }
}

/// Benchmarks the active player; `None` without a resolved active player or enemies.
pub fn benchmark(live: &LiveGameState) -> Option<Benchmark> {
    let (_, me) = me(live)?;
    if me.team.is_empty() {
        return None;
    }
    let minutes = (live.game_time_secs / 60.0).max(1.0);
    let mine = stats(live, me, minutes);
    let (versus, theirs) = match lane_opponent(live) {
        Some(opp) => (opp.champion_name.clone(), stats(live, opp, minutes)),
        None => {
            let enemies = team_of(live, &me.team, false);
            if enemies.is_empty() {
                return None;
            }
            let n = enemies.len() as f64;
            let mut avg = Stats::default();
            for enemy in enemies {
                let s = stats(live, enemy, minutes);
                avg.cs_per_min += s.cs_per_min / n;
                avg.item_gold_per_min += s.item_gold_per_min / n;
                avg.kp += s.kp / n;
                avg.kda += s.kda / n;
                avg.level += s.level / n;
            }
            ("Enemy avg".to_string(), avg)
        }
    };
    let line = |label, you, them, decimals| BenchmarkLine {
        label,
        you,
        them,
        decimals,
    };
    Some(Benchmark {
        versus,
        lines: vec![
            line("CS/M", mine.cs_per_min, theirs.cs_per_min, 1),
            line(
                "ITEM G/M",
                mine.item_gold_per_min,
                theirs.item_gold_per_min,
                0,
            ),
            line("KP%", mine.kp, theirs.kp, 0),
            line("KDA", mine.kda, theirs.kda, 1),
            line("LVL", mine.level, theirs.level, 0),
        ],
    })
}

/// One lane pairing on the item-value board.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemValueRow {
    pub ally: Option<(String, u32)>,
    pub enemy: Option<(String, u32)>,
    /// True on the active player's row.
    pub is_me: bool,
}

impl ItemValueRow {
    /// Ally value minus enemy value (a missing side counts as 0).
    pub fn diff(&self) -> i64 {
        self.ally.as_ref().map_or(0, |a| a.1 as i64) - self.enemy.as_ref().map_or(0, |e| e.1 as i64)
    }
}

/// Item value per lane matchup plus team totals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemValueBoard {
    pub rows: Vec<ItemValueRow>,
    pub ally_total: u32,
    pub enemy_total: u32,
}

impl ItemValueBoard {
    pub fn team_diff(&self) -> i64 {
        self.ally_total as i64 - self.enemy_total as i64
    }
}

/// Pairs both teams lane by lane (Top, Jungle, Mid, Bot, Support; players without a position
/// keep scoreboard order after them). `None` in Arena, where there are more than two teams.
pub fn item_value_board(live: &LiveGameState) -> Option<ItemValueBoard> {
    let mode = live.game_mode.trim().to_ascii_uppercase();
    if mode == "CHERRY" || mode == "ARENA" {
        return None;
    }
    let (my_idx, me) = me(live)?;
    if me.team.is_empty() {
        return None;
    }
    let sorted = |ally: bool| {
        let mut players: Vec<(usize, &LivePlayer)> = live
            .all_players
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.team.is_empty() && p.team.eq_ignore_ascii_case(&me.team) == ally)
            .collect();
        players.sort_by_key(|(idx, p)| (lane_rank(&p.position), *idx));
        players
    };
    let allies = sorted(true);
    let enemies = sorted(false);
    if enemies.is_empty() {
        return None;
    }
    let entry = |p: &LivePlayer| (p.champion_name.clone(), item_value(p));
    let rows = (0..allies.len().max(enemies.len()))
        .map(|i| ItemValueRow {
            ally: allies.get(i).map(|(_, p)| entry(p)),
            enemy: enemies.get(i).map(|(_, p)| entry(p)),
            is_me: allies.get(i).is_some_and(|(idx, _)| *idx == my_idx),
        })
        .collect::<Vec<_>>();
    Some(ItemValueBoard {
        ally_total: allies.iter().map(|(_, p)| item_value(p)).sum(),
        enemy_total: enemies.iter().map(|(_, p)| item_value(p)).sum(),
        rows,
    })
}

/// `+1.2k` / `-850` style gold difference.
pub fn format_gold_diff(diff: i64) -> String {
    let sign = if diff > 0 {
        "+"
    } else if diff < 0 {
        "-"
    } else {
        "±"
    };
    format!("{sign}{}", format_gold(diff.unsigned_abs()))
}

/// `850` / `1.2k` / `12.4k` style gold amount.
pub fn format_gold(gold: u64) -> String {
    if gold >= 1000 {
        format!("{:.1}k", gold as f64 / 1000.0)
    } else {
        gold.to_string()
    }
}

/// Samples of the lane CS difference over game time, for the benchmark sparkline.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CsDiffHistory {
    /// `(game second, cs diff)` in time order.
    pub samples: Vec<(u64, i32)>,
}

impl CsDiffHistory {
    /// Seconds between samples.
    pub const STEP_SECS: u64 = 30;
    /// Samples kept (about 30 minutes).
    pub const MAX_SAMPLES: usize = 60;

    /// Records `diff` at `game_secs` once per [`Self::STEP_SECS`]; a clock that went
    /// backwards (new game) starts a fresh history. Returns `true` when a sample was added.
    pub fn record(&mut self, game_secs: u64, diff: i32) -> bool {
        if let Some(&(last, _)) = self.samples.last() {
            if game_secs < last {
                self.samples.clear();
            } else if game_secs - last < Self::STEP_SECS {
                return false;
            }
        }
        self.samples.push((game_secs, diff));
        if self.samples.len() > Self::MAX_SAMPLES {
            self.samples.remove(0);
        }
        true
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn player(name: &str, champ: &str, team: &str, pos: &str, cs: u32, items: Value) -> Value {
        json!({
            "summonerName": name, "riotId": format!("{name}#EUW"), "championName": champ,
            "team": team, "position": pos, "level": 9,
            "scores": {"kills": 2, "deaths": 1, "assists": 3, "creepScore": cs, "wardScore": 4.0},
            "items": items
        })
    }

    fn game() -> LiveGameState {
        let raw = json!({
            "activePlayer": {"summonerName": "Me", "riotId": "Me#EUW", "level": 9, "currentGold": 500.0},
            "allPlayers": [
                player("Me", "Ahri", "ORDER", "MIDDLE", 120,
                    json!([{"itemID": 1056, "price": 400, "count": 1, "slot": 0},
                           {"itemID": 2003, "price": 50, "count": 3, "slot": 1, "consumable": true}])),
                player("Top", "Garen", "ORDER", "TOP", 100, json!([])),
                player("EnemyMid", "Lux", "CHAOS", "MIDDLE", 100,
                    json!([{"itemID": 999999, "price": 1100, "count": 1, "slot": 0}])),
                player("EnemyTop", "Darius", "CHAOS", "TOP", 90, json!([]))
            ],
            "gameData": {"gameTime": 600.0, "gameMode": "CLASSIC", "mapNumber": 11}
        });
        LiveGameState::from_allgamedata(&raw).expect("parses")
    }

    #[test]
    fn item_value_uses_data_dragon_cost_and_stacks_consumables() {
        let live = game();
        let me = &live.all_players[0];
        let ring = items::item_total_cost(1056).unwrap_or(400);
        let potion = items::item_total_cost(2003).unwrap_or(50);
        assert_eq!(item_value(me), ring + potion * 3);
        // Unknown item ids fall back to the live price.
        assert_eq!(item_value(&live.all_players[2]), 1100);
    }

    #[test]
    fn lane_opponent_and_cs_diff_follow_position() {
        let live = game();
        assert_eq!(
            lane_opponent(&live).map(|p| p.champion_name.as_str()),
            Some("Lux")
        );
        assert_eq!(lane_cs_diff(&live), Some(20));
    }

    #[test]
    fn benchmark_compares_against_lane_opponent() {
        let live = game();
        let bench = benchmark(&live).expect("benchmark");
        assert_eq!(bench.versus, "Lux");
        let cs = &bench.lines[0];
        assert_eq!(cs.label, "CS/M");
        assert!((cs.you - 12.0).abs() < 1e-9);
        assert!((cs.them - 10.0).abs() < 1e-9);
        assert_eq!(cs.ahead(), Some(true));
        let kp = bench.lines.iter().find(|l| l.label == "KP%").unwrap();
        // 5 takedowns of the team's 4 kills caps at 100 %.
        assert!((kp.you - 100.0).abs() < 1e-9);
        let lvl = bench.lines.iter().find(|l| l.label == "LVL").unwrap();
        assert_eq!(lvl.ahead(), None);
    }

    #[test]
    fn benchmark_without_positions_uses_enemy_average() {
        let mut live = game();
        for p in &mut live.all_players {
            p.position.clear();
        }
        let bench = benchmark(&live).expect("benchmark");
        assert_eq!(bench.versus, "Enemy avg");
        assert!((bench.lines[0].them - 9.5).abs() < 1e-9);
        assert_eq!(lane_cs_diff(&live), None);
    }

    #[test]
    fn item_board_pairs_lanes_and_totals_teams() {
        let live = game();
        let board = item_value_board(&live).expect("board");
        assert_eq!(board.rows.len(), 2);
        assert_eq!(board.rows[0].ally.as_ref().unwrap().0, "Garen");
        assert_eq!(board.rows[0].enemy.as_ref().unwrap().0, "Darius");
        assert!(board.rows[1].is_me);
        assert_eq!(
            board.rows[1].enemy.as_ref().unwrap(),
            &("Lux".to_string(), 1100)
        );
        assert_eq!(board.enemy_total, 1100);
        assert_eq!(board.team_diff(), board.ally_total as i64 - 1100);
    }

    #[test]
    fn item_board_is_hidden_in_arena() {
        let mut live = game();
        live.game_mode = "CHERRY".into();
        assert!(item_value_board(&live).is_none());
    }

    #[test]
    fn gold_formatting() {
        assert_eq!(format_gold(850), "850");
        assert_eq!(format_gold(1250), "1.2k");
        assert_eq!(format_gold_diff(1250), "+1.2k");
        assert_eq!(format_gold_diff(-400), "-400");
        assert_eq!(format_gold_diff(0), "±0");
    }

    #[test]
    fn cs_history_samples_every_step_and_resets_on_new_game() {
        let mut h = CsDiffHistory::default();
        assert!(h.record(60, 2));
        assert!(!h.record(75, 3));
        assert!(h.record(90, 4));
        assert_eq!(h.samples, vec![(60, 2), (90, 4)]);
        assert!(h.record(10, 0));
        assert_eq!(h.samples, vec![(10, 0)]);
        for i in 0..100 {
            h.record(40 + i * 30, i as i32);
        }
        assert_eq!(h.samples.len(), CsDiffHistory::MAX_SAMPLES);
    }
}
