// RiftCore Native — domain::protocol
// Frozen cross-layer contract between services, overlay, and UI layers.
// Pure sync types only — no async, no tokio, no non-serializable Instant handles.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Gate constant: active-game live coaching disabled pending Riot policy review.
/// Post-game analytics and pre-game draft coaching remain enabled.
pub const LIVE_COACH_ENABLED: bool = false;

/// League Client gameflow phases mapped from `/lol-gameflow/v1/gameflow-phase`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameflowPhase {
    #[default]
    None,
    Lobby,
    Matchmaking,
    ReadyCheck,
    ChampSelect,
    GameStart,
    InProgress,
    WaitingForStats,
    PreEndOfGame,
    EndOfGame,
    TerminatedInError,
    Unknown,
}

impl GameflowPhase {
    pub fn from_lcu(raw: &str) -> Self {
        match raw.trim() {
            "" | "None" => Self::None,
            "Lobby" => Self::Lobby,
            "Matchmaking" => Self::Matchmaking,
            "ReadyCheck" => Self::ReadyCheck,
            "ChampSelect" => Self::ChampSelect,
            "GameStart" => Self::GameStart,
            "InProgress" => Self::InProgress,
            "WaitingForStats" => Self::WaitingForStats,
            "PreEndOfGame" => Self::PreEndOfGame,
            "EndOfGame" => Self::EndOfGame,
            "TerminatedInError" => Self::TerminatedInError,
            _ => Self::Unknown,
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            Self::None => "Idle",
            Self::Lobby => "In Lobby",
            Self::Matchmaking => "In Queue",
            Self::ReadyCheck => "Match Found",
            Self::ChampSelect => "Champion Select",
            Self::GameStart => "Loading Match",
            Self::InProgress => "In Game",
            Self::WaitingForStats => "Waiting for Stats",
            Self::PreEndOfGame => "Pre-End of Game",
            Self::EndOfGame => "Post-Game",
            Self::TerminatedInError => "Client Error",
            Self::Unknown => "Unknown",
        }
    }
}

/// Compact K/D/A summary included in cross-layer snapshots.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct KdaSummary {
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
}

/// Frozen cross-layer snapshot published from background workers to UI & overlay layers.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CoreSnapshot {
    pub connected: bool,
    pub riot_client_open: bool,
    pub phase: GameflowPhase,
    pub phase_raw: String,

    // Typed active player & match summary
    pub summoner_name: String,
    pub summoner_tag: String,
    pub summoner_puuid: String,
    pub champion_id: u32,
    pub champion_name: String,
    pub kda: KdaSummary,
    pub gold: f64,
    pub cs: u32,
    pub game_time_secs: f64,
    pub game_mode: String,

    // Structured LCU / Riot API / Live Client Data payloads
    pub summoner: Option<Value>,
    pub ranked: Option<Value>,
    pub match_history: Option<Value>,
    pub champ_select: Option<Value>,
    pub ready_check: Option<Value>,
    pub live_game: Option<Value>,
    pub riot_lookup: Option<Value>,
    pub riot_api_status: Option<String>,
    pub latest_release: Option<Value>,
    pub update_status: Option<String>,

    // Overlay and notification state
    pub overlay_enabled: bool,
    pub last_updated_epoch_ms: Option<u64>,
    pub notice: Option<CoreNotice>,
}

/// Commands dispatched from UI / overlay layers to background workers or overlay host.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CoreCommand {
    SetAutoAccept(bool),
    AcceptMatch,
    ImportSpells {
        spell1: i64,
        spell2: i64,
    },
    ImportRunes(Value),
    ExportItemSet(Value),
    LookupRiotId {
        riot_id: String,
        region: String,
        api_key: String,
    },
    ValidateRiotApiKey {
        region: String,
        api_key: String,
    },
    CheckUpdates,
    InstallUpdate,
    LaunchLeague,
    SetLeagueWindowMode(u8),
    Refresh,
    UpdateTarget {
        hwnd: Option<isize>,
        bounds: Option<[i32; 4]>,
    },
    SetEnabled(bool),
    SetHudMode(String),
    SetMousePassthrough(bool),
    Shutdown,
}

/// Discrete domain events emitted across state transitions and live telemetry updates.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CoreEvent {
    ClientConnected,
    ClientDisconnected,
    PhaseChanged {
        from: GameflowPhase,
        to: GameflowPhase,
    },
    ReadyCheckPopped,
    ChampSelectChanged {
        local_champion_id: u32,
        assigned_role: String,
    },
    GameStarted {
        game_mode: String,
        map_name: String,
    },
    GameEnded {
        win: Option<bool>,
        duration_secs: u64,
    },
    PlayerKill {
        killer: String,
        victim: String,
        assisters: Vec<String>,
        game_time_secs: f64,
    },
    ObjectiveTaken {
        objective: String,
        dragon_type: Option<String>,
        killer: String,
        game_time_secs: f64,
    },
    MatchHistoryUpdated {
        count: usize,
    },
    RiotLookupCompleted {
        riot_id: String,
        ok: bool,
    },
    Notice(CoreNotice),
}

/// Structured user-facing notifications and operational errors.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreNotice {
    Info(String),
    Success(String),
    Warning(String),
    OverlayError(String),
    UpdateAvailable { version: String, downloadable: bool },
    LcuError(String),
    RiotApiError(String),
}

impl CoreNotice {
    pub fn message(&self) -> String {
        match self {
            Self::Info(msg)
            | Self::Success(msg)
            | Self::Warning(msg)
            | Self::OverlayError(msg)
            | Self::LcuError(msg)
            | Self::RiotApiError(msg) => msg.clone(),
            Self::UpdateAvailable {
                version,
                downloadable,
            } => {
                if *downloadable {
                    format!("RiftCore {version} is available")
                } else {
                    format!("RiftCore {version} is available on GitHub Releases")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_coach_gate_is_false_by_default() {
        const { assert!(!LIVE_COACH_ENABLED) };
    }

    #[test]
    fn maps_lcu_phases_and_roundtrips_snapshot() {
        assert_eq!(
            GameflowPhase::from_lcu("ChampSelect"),
            GameflowPhase::ChampSelect
        );
        assert_eq!(
            GameflowPhase::from_lcu("InProgress"),
            GameflowPhase::InProgress
        );
        assert_eq!(
            GameflowPhase::from_lcu("CustomOther"),
            GameflowPhase::Unknown
        );

        let snap = CoreSnapshot {
            connected: true,
            phase: GameflowPhase::InProgress,
            phase_raw: "InProgress".to_string(),
            summoner_name: "Tester".to_string(),
            champion_id: 103,
            champion_name: "Ahri".to_string(),
            kda: KdaSummary {
                kills: 4,
                deaths: 1,
                assists: 6,
            },
            gold: 2150.0,
            cs: 145,
            notice: Some(CoreNotice::UpdateAvailable {
                version: "v2.3.0".to_string(),
                downloadable: true,
            }),
            ..Default::default()
        };

        let json = serde_json::to_string(&snap).expect("serialize CoreSnapshot");
        let parsed: CoreSnapshot = serde_json::from_str(&json).expect("deserialize CoreSnapshot");
        assert_eq!(parsed, snap);
        assert_eq!(
            parsed.notice.as_ref().map(CoreNotice::message).as_deref(),
            Some("RiftCore v2.3.0 is available")
        );
    }

    #[test]
    fn commands_and_events_are_clone_debug_and_serializable() {
        let cmd = CoreCommand::UpdateTarget {
            hwnd: Some(12345),
            bounds: Some([0, 0, 1920, 1080]),
        };
        let event = CoreEvent::PlayerKill {
            killer: "Ahri".to_string(),
            victim: "Zed".to_string(),
            assisters: vec!["Lee Sin".to_string()],
            game_time_secs: 455.2,
        };
        let cmd_json = serde_json::to_string(&cmd).expect("serialize CoreCommand");
        let ev_json = serde_json::to_string(&event).expect("serialize CoreEvent");
        assert_eq!(serde_json::from_str::<CoreCommand>(&cmd_json).unwrap(), cmd);
        assert_eq!(serde_json::from_str::<CoreEvent>(&ev_json).unwrap(), event);
    }
}
