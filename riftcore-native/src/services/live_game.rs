use crate::domain::live_game::LiveGameState;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::fmt;
use std::time::Duration;

/// Riot Live Client Data endpoint (HTTPS on localhost:2999 with Riot's self-signed cert).
pub const LIVE_GAME_ENDPOINT: &str = "https://127.0.0.1:2999/liveclientdata/allgamedata";
pub const LIVE_GAME_POLL_INTERVAL: Duration = Duration::from_millis(250);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_millis(250);
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_millis(750);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveGameError {
    HttpStatus(u16),
    InvalidResponse(String),
    Transport(String),
}

impl fmt::Display for LiveGameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HttpStatus(code) => write!(f, "Live Client Data API returned HTTP {code}"),
            Self::InvalidResponse(msg) => {
                write!(f, "Live Client Data API returned invalid JSON: {msg}")
            }
            Self::Transport(msg) => write!(f, "Live Client Data connection failed: {msg}"),
        }
    }
}

impl std::error::Error for LiveGameError {}

#[derive(Clone, Debug, PartialEq)]
pub struct LiveGamePoll {
    pub state: LiveGameState,
    pub raw: Value,
}

#[derive(Clone, Debug)]
pub struct LiveGameClient {
    http: Client,
    endpoint: String,
}

impl LiveGameClient {
    pub fn build_http_client() -> Result<Client, LiveGameError> {
        Client::builder()
            .danger_accept_invalid_certs(true)
            .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
            .timeout(DEFAULT_REQUEST_TIMEOUT)
            .user_agent(concat!("RiftCore Companion/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|error| LiveGameError::Transport(error.to_string()))
    }

    pub fn new() -> Result<Self, LiveGameError> {
        Ok(Self {
            http: Self::build_http_client()?,
            endpoint: LIVE_GAME_ENDPOINT.to_string(),
        })
    }

    pub fn with_endpoint(endpoint: impl Into<String>) -> Result<Self, LiveGameError> {
        Ok(Self {
            http: Self::build_http_client()?,
            endpoint: endpoint.into(),
        })
    }

    pub fn with_http_and_endpoint(http: Client, endpoint: impl Into<String>) -> Self {
        Self {
            http,
            endpoint: endpoint.into(),
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Polls the Live Client Data API once.
    ///
    /// Returns `Ok(None)` when League is not in an active match (connection refused,
    /// timeout, or HTTP 404 while loading). Returns `Err` only on unexpected protocol
    /// or JSON errors from a live response.
    pub async fn poll(&self) -> Result<Option<LiveGamePoll>, LiveGameError> {
        let response = match self.http.get(&self.endpoint).send().await {
            Ok(resp) => resp,
            Err(err) if err.is_connect() || err.is_timeout() => return Ok(None),
            Err(err) => {
                // On Windows localhost, connection resets / refused during match start/end
                // mean the live game server is not currently active.
                let msg = err.to_string().to_ascii_lowercase();
                if msg.contains("connection refused")
                    || msg.contains("actively refused")
                    || msg.contains("timed out")
                {
                    return Ok(None);
                }
                return Err(LiveGameError::Transport(err.to_string()));
            }
        };

        let status = response.status();
        if status == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !status.is_success() {
            return Err(LiveGameError::HttpStatus(status.as_u16()));
        }

        let raw = response
            .json::<Value>()
            .await
            .map_err(|err| LiveGameError::InvalidResponse(err.to_string()))?;

        let Some(state) = LiveGameState::from_allgamedata(&raw) else {
            return Ok(None);
        };

        Ok(Some(LiveGamePoll { state, raw }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    #[test]
    fn poll_when_game_not_running_returns_ok_none() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener); // Close immediately so port refuses connections.

        let client =
            LiveGameClient::with_endpoint(format!("http://{addr}/liveclientdata/allgamedata"))
                .unwrap();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let res = rt.block_on(client.poll());
        assert_eq!(res, Ok(None));
    }

    #[test]
    fn poll_404_loading_screen_returns_ok_none() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let srv = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            stream.write_all(resp.as_bytes()).unwrap();
        });

        let client =
            LiveGameClient::with_endpoint(format!("http://{addr}/liveclientdata/allgamedata"))
                .unwrap();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let res = rt.block_on(client.poll());
        assert_eq!(res, Ok(None));
        srv.join().unwrap();
    }

    #[test]
    fn poll_parses_allgamedata_into_live_game_state() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let body = r#"{
            "gameData": {
                "gameTime": 420.0,
                "gameMode": "CLASSIC",
                "mapName": "Map11",
                "mapNumber": 11
            },
            "activePlayer": {
                "summonerName": "PlayerOne",
                "level": 7,
                "currentGold": 950.5
            },
            "allPlayers": [
                {
                    "summonerName": "PlayerOne",
                    "championName": "Jinx",
                    "team": "ORDER",
                    "position": "BOTTOM",
                    "level": 7,
                    "scores": { "kills": 3, "deaths": 0, "assists": 2, "creepScore": 75, "wardScore": 6.0 },
                    "items": []
                }
            ],
            "events": { "Events": [] }
        }"#;

        let srv = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(resp.as_bytes()).unwrap();
        });

        let client =
            LiveGameClient::with_endpoint(format!("http://{addr}/liveclientdata/allgamedata"))
                .unwrap();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let poll = rt
            .block_on(client.poll())
            .expect("poll succeeds")
            .expect("live game active");
        assert_eq!(poll.state.active_player.champion_name, "Jinx");
        assert_eq!(poll.state.active_player.scores.kills, 3);
        assert_eq!(poll.state.active_player.scores.creep_score, 75);
        assert!((poll.state.game_time_secs - 420.0).abs() < f64::EPSILON);
        srv.join().unwrap();
    }

    #[test]
    fn poll_invalid_json_returns_error() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let body = "{not valid json";

        let srv = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(resp.as_bytes()).unwrap();
        });

        let client =
            LiveGameClient::with_endpoint(format!("http://{addr}/liveclientdata/allgamedata"))
                .unwrap();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let res = rt.block_on(client.poll());
        assert!(matches!(res, Err(LiveGameError::InvalidResponse(_))));
        srv.join().unwrap();
    }
}
