// RiftCore Native — domain::coach
// Pre-game draft coaching and post-game match coaching traits & offline heuristic provider.
// Active-game coaching is strictly gated by `LIVE_COACH_ENABLED` (= false).

use crate::domain::analytics::{DraftComparison, NormalizedMatchSummary};
use crate::domain::protocol::LIVE_COACH_ENABLED;

/// Extensible synchronous coaching interface for offline heuristics or future AI providers.
pub trait CoachProvider: Send + Sync {
    fn post_game_advice(&self, summary: &NormalizedMatchSummary) -> Vec<String>;
    fn draft_advice(&self, draft: &DraftComparison) -> Vec<String>;
    // Active-game advice: GATED — only implement if LIVE_COACH_ENABLED
}

/// Offline rule-based coaching engine grounded solely in verified match and draft telemetry.
#[derive(Clone, Copy, Debug, Default)]
pub struct OfflineHeuristicCoach;

impl OfflineHeuristicCoach {
    pub fn is_live_coaching_enabled(&self) -> bool {
        LIVE_COACH_ENABLED
    }
}

impl CoachProvider for OfflineHeuristicCoach {
    fn post_game_advice(&self, summary: &NormalizedMatchSummary) -> Vec<String> {
        let mut advice = Vec::new();
        let is_support = summary.role.eq_ignore_ascii_case("SUPPORT");
        let is_aram = summary.queue_id == 450 || summary.game_mode.eq_ignore_ascii_case("ARAM");
        let duration_mins = (summary.duration_secs / 60).max(1);

        // 1. CS / Farm pacing (non-Support, non-ARAM)
        if !is_support && !is_aram && summary.duration_secs >= 600 {
            if summary.cs_per_min >= 8.0 {
                advice.push(format!(
                    "Strong farm pace at {:.1} CS/min ({} CS in {}m), keeping item timings ahead of curve.",
                    summary.cs_per_min, summary.cs, duration_mins
                ));
            } else if summary.cs_per_min < 5.5 {
                advice.push(format!(
                    "Farm averaged {:.1} CS/min ({} CS in {}m); catching side-lane waves between objective setups will accelerate core items.",
                    summary.cs_per_min, summary.cs, duration_mins
                ));
            }
        }

        // 2. Kill Participation
        if summary.team_kills > 0 {
            let takedowns = summary.kills + summary.assists;
            if summary.kp_pct >= 65.0 {
                advice.push(format!(
                    "High skirmish involvement with {:.0}% kill participation ({} of {} team kills).",
                    summary.kp_pct, takedowns, summary.team_kills
                ));
            } else if summary.kp_pct < 40.0
                && (summary.role == "JUNGLE" || summary.role == "SUPPORT" || !summary.win)
            {
                advice.push(format!(
                    "Kill participation was {:.0}% ({} of {} team kills); look for earlier rotations to river skirmishes and objective contests.",
                    summary.kp_pct, takedowns, summary.team_kills
                ));
            }
        }

        // 3. Damage conversion & gold share
        if summary.team_damage > 0 && !is_support {
            if summary.dmg_pct >= 28.0 {
                advice.push(format!(
                    "Primary damage output at {:.1}% of team champion damage ({:.0} DPM).",
                    summary.dmg_pct, summary.dpm
                ));
            } else if summary.gold_pct >= 22.0 && summary.dmg_pct < 16.0 {
                advice.push(format!(
                    "Held {:.1}% of team gold ({:.0} GPM) with {:.1}% damage share ({:.0} DPM); look to convert item leads in decisive mid-game fights.",
                    summary.gold_pct, summary.gpm, summary.dmg_pct, summary.dpm
                ));
            }
        }

        // 4. Survivability
        if summary.deaths >= 7 {
            advice.push(format!(
                "Recorded {} deaths ({:.2} KDA); tracking missing enemies before pushing past river will reduce avoidable shutdowns.",
                summary.deaths, summary.kda
            ));
        } else if summary.deaths <= 2 && summary.kda >= 4.0 {
            advice.push(format!(
                "Disciplined positioning with {} deaths and {:.2} KDA.",
                summary.deaths, summary.kda
            ));
        }

        // 5. Vision (non-ARAM)
        if !is_aram && summary.duration_secs >= 600 {
            if is_support && summary.vpm < 1.3 {
                advice.push(format!(
                    "Vision averaged {:.2}/min ({} placed, {} cleared); refresh ward lines around dragon/baron 45s prior to spawn.",
                    summary.vpm, summary.wards_placed, summary.wards_killed
                ));
            } else if !is_support && summary.vpm < 0.45 {
                advice.push(format!(
                    "Vision averaged {:.2}/min ({} control wards bought); keeping trinket on cooldown improves flank safety.",
                    summary.vpm, summary.control_wards
                ));
            } else if summary.control_wards >= 3 || (is_support && summary.vpm >= 1.8) {
                advice.push(format!(
                    "Consistent vision control at {:.2}/min ({} control wards, {} wards cleared).",
                    summary.vpm, summary.control_wards, summary.wards_killed
                ));
            }
        }

        // 6. Objective / Structure impact
        if summary.turret_damage >= 5000 {
            advice.push(format!(
                "Converted map pressure into {} turret damage.",
                summary.turret_damage
            ));
        } else if summary.objective_damage >= 12000 {
            advice.push(format!(
                "Contributed {} damage to neutral objectives.",
                summary.objective_damage
            ));
        }

        if advice.is_empty() {
            advice.push(format!(
                "Steady match profile on {} ({:.2} KDA, {:.1} CS/min, {:.0} GPM, {:.0}% KP).",
                summary.champion_name, summary.kda, summary.cs_per_min, summary.gpm, summary.kp_pct
            ));
        }

        advice
    }

    fn draft_advice(&self, draft: &DraftComparison) -> Vec<String> {
        let mut advice = Vec::new();

        if draft.ally.picks_count < 2 {
            advice.push(
                "Add at least 2 ally picks to evaluate frontline, damage split, and utility coverage."
                    .to_string(),
            );
            return advice;
        }

        // Ally coverage checks
        if draft.ally.picks_count >= 3 && draft.ally.frontline_score == 0 {
            advice.push(format!(
                "No frontline detected across {} ally picks; add a tank, juggernaut, or engage support to absorb pressure.",
                draft.ally.picks_count
            ));
        }
        if draft.ally.physical_pct >= 75.0 {
            advice.push(format!(
                "Ally damage leans {:.0}% physical ({} AD vs {} AP threats); adding magic damage prevents easy enemy armor stacking.",
                draft.ally.physical_pct, draft.ally.physical_threats, draft.ally.magic_threats
            ));
        } else if draft.ally.magic_pct >= 75.0 {
            advice.push(format!(
                "Ally damage leans {:.0}% magic ({} AP vs {} AD threats); adding physical DPS stops enemy MR stacking.",
                draft.ally.magic_pct, draft.ally.magic_threats, draft.ally.physical_threats
            ));
        }
        if draft.ally.picks_count >= 3 && draft.ally.hard_cc_score <= 1 {
            advice.push(format!(
                "Low hard-CC density (score {}); prioritize reliable catch or disengage on remaining slots.",
                draft.ally.hard_cc_score
            ));
        }
        if draft.ally.picks_count >= 4
            && draft.ally.utility_count == 0
            && draft.ally.peel_score == 0
        {
            advice.push(
                "Limited peel and utility on ally comp; carries will need careful positioning against dive."
                    .to_string(),
            );
        }
        if draft.ally.picks_count >= 3 && draft.ally.waveclear_score == 0 {
            advice.push(
                "Low waveclear across ally picks; mid priority and siege defense may be contested."
                    .to_string(),
            );
        }

        // Enemy matchup & counter-itemization cues
        if draft.enemy.picks_count >= 2 {
            if draft.enemy.physical_pct >= 70.0 {
                advice.push(format!(
                    "Enemy comp is {:.0}% physical damage; Plated Steelcaps and early Armor gain high value.",
                    draft.enemy.physical_pct
                ));
            } else if draft.enemy.magic_pct >= 70.0 {
                advice.push(format!(
                    "Enemy comp is {:.0}% magic damage; Mercury's Treads and early Magic Resist gain high value.",
                    draft.enemy.magic_pct
                ));
            }
            if draft.enemy.healing_threat_count >= 2 {
                advice.push(format!(
                    "Enemy comp has {} sustain/healing picks; plan early Grievous Wounds.",
                    draft.enemy.healing_threat_count
                ));
            }
            if draft.enemy.shielding_count >= 2 {
                advice.push(format!(
                    "Enemy comp has {} heavy shielding picks; Serpent's Fang or burst timing helps break shields.",
                    draft.enemy.shielding_count
                ));
            }
            if draft.enemy.frontline_score >= 2 {
                advice.push(format!(
                    "Enemy comp fields {} durable frontliners; prioritize % penetration or sustained DPS.",
                    draft.enemy.frontline_score
                ));
            }
            if draft.enemy.engage_score >= 2 {
                advice.push(format!(
                    "Enemy comp brings {} engage threats; track key initiation cooldowns and value peel/stasis.",
                    draft.enemy.engage_score
                ));
            }
        }

        if draft.ally.picks_count >= 4 && draft.ally_missing_coverage.is_empty() {
            advice.push(format!(
                "Balanced ally core ({:.0}% AD / {:.0}% AP, {} frontline, {} hard CC).",
                draft.ally.physical_pct,
                draft.ally.magic_pct,
                draft.ally.frontline_score,
                draft.ally.hard_cc_score
            ));
        }

        advice
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::analytics::{compare_drafts, DraftChampionInput};

    #[test]
    fn live_coach_gate_remains_disabled() {
        let coach = OfflineHeuristicCoach;
        assert!(!LIVE_COACH_ENABLED);
        assert!(!coach.is_live_coaching_enabled());
    }

    #[test]
    fn offline_coach_gives_grounded_post_game_advice() {
        let coach = OfflineHeuristicCoach;
        let mut summary = NormalizedMatchSummary {
            champion_name: "Jinx".to_string(),
            role: "ADC".to_string(),
            queue_id: 420,
            win: false,
            kills: 3,
            deaths: 8,
            assists: 4,
            cs: 140,
            duration_secs: 1800,
            gold: 11000,
            damage: 13000,
            vision: 10,
            control_wards: 0,
            team_kills: 22,
            team_damage: 90000,
            team_gold: 45000,
            ..Default::default()
        };
        summary.recompute_derived();

        let advice = coach.post_game_advice(&summary);
        assert!(!advice.is_empty());
        assert!(advice.iter().any(|line| line.contains("CS/min")));
        assert!(advice
            .iter()
            .any(|line| line.to_ascii_lowercase().contains("kill participation")));
        assert!(advice.iter().any(|line| line.contains("8 deaths")));
        assert!(advice.iter().any(|line| line.contains("Vision")));
    }

    #[test]
    fn offline_coach_does_not_penalize_support_cs() {
        let coach = OfflineHeuristicCoach;
        let mut summary = NormalizedMatchSummary {
            champion_name: "Thresh".to_string(),
            role: "SUPPORT".to_string(),
            queue_id: 420,
            win: true,
            kills: 1,
            deaths: 2,
            assists: 18,
            cs: 35,
            duration_secs: 1800,
            gold: 8500,
            damage: 9000,
            vision: 60,
            control_wards: 5,
            wards_killed: 9,
            team_kills: 24,
            team_damage: 85000,
            team_gold: 50000,
            ..Default::default()
        };
        summary.recompute_derived();

        let advice = coach.post_game_advice(&summary);
        assert!(!advice.iter().any(|line| line.contains("CS/min")));
        assert!(advice
            .iter()
            .any(|line| line.to_ascii_lowercase().contains("kill participation")));
        assert!(advice.iter().any(|line| line.contains("vision control")));
    }

    #[test]
    fn offline_coach_gives_draft_gap_and_counter_advice() {
        let coach = OfflineHeuristicCoach;
        let ally = vec![
            DraftChampionInput::from_archetype(266, "Aatrox", "TOP", "DIVER_FIGHTER"),
            DraftChampionInput::from_archetype(64, "Lee Sin", "JUNGLE", "DIVER_FIGHTER"),
            DraftChampionInput::from_archetype(238, "Zed", "MID", "AD_ASSASSIN"),
            DraftChampionInput::from_archetype(222, "Jinx", "ADC", "CRIT_MARKSMAN"),
        ];
        let enemy = vec![
            DraftChampionInput::from_archetype(122, "Darius", "TOP", "JUGGERNAUT"),
            DraftChampionInput::from_archetype(254, "Vi", "JUNGLE", "DIVER_FIGHTER"),
            DraftChampionInput::from_archetype(91, "Talon", "MID", "AD_ASSASSIN"),
            DraftChampionInput::from_archetype(267, "Nami", "SUPPORT", "ENCHANTER"),
        ];

        let cmp = compare_drafts(&ally, &enemy);
        let advice = coach.draft_advice(&cmp);
        assert!(advice.iter().any(|line| line.contains("physical")));
        assert!(advice.iter().any(|line| line.contains("Plated Steelcaps")));
        assert!(advice.iter().any(|line| line.contains("Grievous Wounds")));
    }
}
