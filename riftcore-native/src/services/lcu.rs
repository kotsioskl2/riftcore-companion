use crate::platform::process::{
    discover_lcu_lockfile, is_riot_client_open as platform_is_riot_client_open, parse_lockfile_str,
    LcuLockfile,
};
use reqwest::{Client, Method, StatusCode};
use serde_json::Value;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use zeroize::Zeroizing;

pub const LCU_USER: &str = "riot";
pub const LOCKFILE_ENV_VAR: &str = "RIFTCORE_LOCKFILE";
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_millis(500);
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_millis(1500);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LcuError {
    NotConnected,
    InvalidPort,
    UnsupportedOperation(String),
    HttpStatus { status: u16, body: String },
    RequestFailed(String),
    InvalidResponse(String),
    Io(String),
}

impl fmt::Display for LcuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConnected => write!(f, "League Client is not connected"),
            Self::InvalidPort => write!(f, "League Client lockfile contains an invalid port"),
            Self::UnsupportedOperation(msg) => write!(f, "{msg}"),
            Self::HttpStatus { status, body } => {
                if body.trim().is_empty() {
                    write!(f, "League Client returned HTTP {status}")
                } else {
                    write!(f, "League Client returned HTTP {status}: {body}")
                }
            }
            Self::RequestFailed(msg) => write!(f, "League Client request failed: {msg}"),
            Self::InvalidResponse(msg) => {
                write!(f, "League Client returned unreadable data: {msg}")
            }
            Self::Io(msg) => write!(f, "League Client I/O error: {msg}"),
        }
    }
}

impl std::error::Error for LcuError {}

#[derive(Clone, PartialEq, Eq)]
pub struct LcuCredentials {
    pub pid: u32,
    pub port: u16,
    password: Zeroizing<String>,
    pub protocol: String,
    pub install_dir: Option<PathBuf>,
    pub lockfile_path: Option<PathBuf>,
}

impl fmt::Debug for LcuCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LcuCredentials")
            .field("pid", &self.pid)
            .field("port", &self.port)
            .field("password", &"<redacted>")
            .field("protocol", &self.protocol)
            .field("install_dir", &self.install_dir)
            .field("lockfile_path", &self.lockfile_path)
            .finish()
    }
}

impl LcuCredentials {
    pub fn new(
        pid: u32,
        port: u16,
        password: impl Into<String>,
        protocol: impl Into<String>,
        install_dir: Option<PathBuf>,
        lockfile_path: Option<PathBuf>,
    ) -> Result<Self, LcuError> {
        if port == 0 {
            return Err(LcuError::InvalidPort);
        }
        let password = password.into();
        if password.trim().is_empty() {
            return Err(LcuError::InvalidResponse(
                "missing LCU lockfile password".to_string(),
            ));
        }
        let protocol = protocol.into();
        let protocol = if protocol.trim().is_empty() {
            "https".to_string()
        } else {
            protocol.trim().to_ascii_lowercase()
        };
        Ok(Self {
            pid,
            port,
            password: Zeroizing::new(password),
            protocol,
            install_dir,
            lockfile_path,
        })
    }

    pub fn from_lockfile(
        lockfile: LcuLockfile,
        lockfile_path: Option<PathBuf>,
    ) -> Result<Self, LcuError> {
        Self::new(
            lockfile.pid,
            lockfile.port,
            lockfile.token,
            lockfile.protocol,
            lockfile.install_dir,
            lockfile_path,
        )
    }

    pub fn password(&self) -> &str {
        self.password.as_str()
    }

    pub fn base_url(&self) -> String {
        format!("{}://127.0.0.1:{}", self.protocol, self.port)
    }
}

#[derive(Clone)]
pub struct LcuClient {
    http: Client,
    credentials: LcuCredentials,
    base_url_override: Option<String>,
}

impl fmt::Debug for LcuClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LcuClient")
            .field("credentials", &self.credentials)
            .field("base_url_override", &self.base_url_override)
            .finish()
    }
}

impl LcuClient {
    /// Builds the shared reqwest client configured for LCU self-signed TLS
    /// (`danger_accept_invalid_certs(true)` — Policy::none() verification).
    pub fn build_http_client() -> Result<Client, LcuError> {
        Client::builder()
            .danger_accept_invalid_certs(true)
            .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
            .timeout(DEFAULT_REQUEST_TIMEOUT)
            .user_agent(concat!("RiftCore Companion/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|error| LcuError::RequestFailed(error.to_string()))
    }

    pub fn new(credentials: LcuCredentials) -> Result<Self, LcuError> {
        let http = Self::build_http_client()?;
        Ok(Self {
            http,
            credentials,
            base_url_override: None,
        })
    }

    pub fn with_http_client(http: Client, credentials: LcuCredentials) -> Self {
        Self {
            http,
            credentials,
            base_url_override: None,
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url_override = Some(base_url.into().trim_end_matches('/').to_string());
        self
    }

    pub fn credentials(&self) -> &LcuCredentials {
        &self.credentials
    }

    pub fn update_credentials(&mut self, credentials: LcuCredentials) {
        self.credentials = credentials;
    }

    fn endpoint_url(&self, endpoint: &str) -> String {
        let base = self
            .base_url_override
            .as_deref()
            .map(str::to_owned)
            .unwrap_or_else(|| self.credentials.base_url());
        if endpoint.starts_with('/') {
            format!("{base}{endpoint}")
        } else {
            format!("{base}/{endpoint}")
        }
    }

    pub async fn request(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<Value, LcuError> {
        let url = self.endpoint_url(endpoint);
        let mut req = self
            .http
            .request(method, url)
            .basic_auth(LCU_USER, Some(self.credentials.password()));
        if let Some(payload) = body {
            req = req.json(&payload);
        }
        let response = req
            .send()
            .await
            .map_err(|error| LcuError::RequestFailed(error.to_string()))?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|error| LcuError::RequestFailed(error.to_string()))?;
        if !status.is_success() {
            return Err(LcuError::HttpStatus {
                status: status.as_u16(),
                body: text,
            });
        }
        if text.trim().is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_str(&text).map_err(|error| LcuError::InvalidResponse(error.to_string()))
    }

    pub async fn get_json(&self, endpoint: &str) -> Result<Value, LcuError> {
        self.request(Method::GET, endpoint, None).await
    }

    pub async fn get_optional_json(&self, endpoint: &str) -> Result<Option<Value>, LcuError> {
        match self.get_json(endpoint).await {
            Ok(value) => Ok(Some(value)),
            Err(LcuError::HttpStatus { status, .. })
                if status == StatusCode::NOT_FOUND.as_u16() =>
            {
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    pub async fn gameflow_phase(&self) -> Result<String, LcuError> {
        let value = self.get_json("/lol-gameflow/v1/gameflow-phase").await?;
        Ok(value.as_str().unwrap_or("None").to_string())
    }

    pub async fn current_summoner(&self) -> Result<Value, LcuError> {
        self.get_json("/lol-summoner/v1/current-summoner").await
    }

    pub async fn ranked_stats(&self) -> Result<Value, LcuError> {
        self.get_json("/lol-ranked/v1/current-ranked-stats").await
    }

    pub async fn match_history(
        &self,
        puuid: &str,
        beg_index: u32,
        end_index: u32,
    ) -> Result<Value, LcuError> {
        let endpoint = format!(
            "/lol-match-history/v1/products/lol/{puuid}/matches?begIndex={beg_index}&endIndex={end_index}"
        );
        self.get_json(&endpoint).await
    }

    pub async fn champ_select_session(&self) -> Result<Option<Value>, LcuError> {
        self.get_optional_json("/lol-champ-select/v1/session").await
    }

    pub async fn ready_check(&self) -> Result<Option<Value>, LcuError> {
        self.get_optional_json("/lol-matchmaking/v1/ready-check")
            .await
    }

    pub async fn accept_match(&self) -> Result<(), LcuError> {
        self.request(
            Method::POST,
            "/lol-matchmaking/v1/ready-check/accept",
            Some(Value::Object(Default::default())),
        )
        .await?;
        Ok(())
    }

    pub async fn import_spells(&self, spell1: i64, spell2: i64) -> Result<(), LcuError> {
        let body = serde_json::json!({ "spell1Id": spell1, "spell2Id": spell2 });
        self.request(
            Method::PATCH,
            "/lol-champ-select/v1/session/my-selection",
            Some(body),
        )
        .await?;
        Ok(())
    }

    pub async fn import_runes(&self, mut page: Value) -> Result<String, LcuError> {
        let pages_val = self.get_json("/lol-perks/v1/pages").await?;
        let pages = pages_val.as_array().ok_or_else(|| {
            LcuError::InvalidResponse("expected array from /lol-perks/v1/pages".to_string())
        })?;

        let target = pages.iter().find_map(|entry| {
            let owned = entry
                .get("isDeletable")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                || entry
                    .get("isEditable")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
            let name = entry
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if owned && name.to_ascii_lowercase().contains("riftcore") {
                entry.get("id").and_then(Value::as_i64)
            } else {
                None
            }
        });

        if let Some(object) = page.as_object_mut() {
            object.insert("current".to_string(), Value::Bool(true));
            object.remove("isActive");
            let has_name = object
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| !name.trim().is_empty());
            if !has_name {
                object.insert(
                    "name".to_string(),
                    Value::String("RiftCore Build".to_string()),
                );
            }
        }

        let (result, existing_id) = if let Some(id) = target {
            if let Some(object) = page.as_object_mut() {
                object.insert("id".to_string(), Value::from(id));
            }
            (
                self.request(
                    Method::PUT,
                    &format!("/lol-perks/v1/pages/{id}"),
                    Some(page.clone()),
                )
                .await,
                Some(id),
            )
        } else {
            if let Some(object) = page.as_object_mut() {
                object.remove("id");
            }
            (
                self.request(Method::POST, "/lol-perks/v1/pages", Some(page.clone()))
                    .await,
                None,
            )
        };

        let created = result?;
        let page_id = existing_id.or_else(|| created.get("id").and_then(Value::as_i64));
        let Some(id) = page_id else {
            return Ok(
                "Rune page was saved, but League Client did not return its page ID".to_string(),
            );
        };
        self.request(
            Method::PUT,
            "/lol-perks/v1/currentpage",
            Some(Value::from(id)),
        )
        .await?;
        Ok("Rune page imported and activated".to_string())
    }

    pub async fn export_item_set(
        &self,
        item_set: Value,
        cached_summoner: Option<&Value>,
    ) -> Result<String, LcuError> {
        let fetched;
        let summoner = match cached_summoner {
            Some(s) => s,
            None => {
                fetched = self.current_summoner().await?;
                &fetched
            }
        };
        let id = summoner
            .get("summonerId")
            .or_else(|| summoner.get("accountId"))
            .and_then(|v| {
                v.as_u64()
                    .map(|n| n.to_string())
                    .or_else(|| v.as_str().map(str::to_owned))
            })
            .ok_or_else(|| {
                LcuError::InvalidResponse(
                    "League Client did not provide a summonerId or accountId".to_string(),
                )
            })?;

        let endpoint = format!("/lol-item-sets/v1/item-sets/{id}/sets");
        let mut existing = match self.get_optional_json(&endpoint).await? {
            Some(value) => value,
            None => serde_json::json!({ "itemSets": [], "timestamp": 0 }),
        };

        if existing
            .get_mut("itemSets")
            .and_then(Value::as_array_mut)
            .is_none()
        {
            if let Some(obj) = existing.as_object_mut() {
                obj.insert("itemSets".to_string(), Value::Array(Vec::new()));
            } else {
                existing = serde_json::json!({ "itemSets": [], "timestamp": 0 });
            }
        }

        let sets = existing
            .get_mut("itemSets")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| {
                LcuError::InvalidResponse("Could not prepare in-game item sets array".to_string())
            })?;

        let champion_id = item_set
            .get("associatedChampions")
            .and_then(Value::as_array)
            .and_then(|ids| ids.first())
            .cloned()
            .unwrap_or(Value::Null);

        sets.retain(|set| {
            let is_riftcore = set
                .get("title")
                .and_then(Value::as_str)
                .is_some_and(|title| title.starts_with("RiftCore:"));
            let same_champion = set
                .get("associatedChampions")
                .and_then(Value::as_array)
                .is_some_and(|ids| ids.iter().any(|entry| entry == &champion_id));
            !(is_riftcore && same_champion)
        });
        sets.push(item_set);

        self.request(Method::PUT, &endpoint, Some(existing)).await?;
        Ok("In-game item set added. Open the shop with P to use it.".to_string())
    }
}

/// Parses an LCU lockfile string (`process:pid:port:password:protocol`) using
/// `platform::process::parse_lockfile_str`, enforcing `u16` port (`1..=65535`) and active `u32` PID.
pub fn parse_lockfile_contents(
    content: &str,
    lockfile_path: &Path,
    verify_pid: bool,
) -> Option<LcuCredentials> {
    let parsed = parse_lockfile_str(content, lockfile_path, verify_pid)?;
    LcuCredentials::from_lockfile(parsed, Some(lockfile_path.to_path_buf())).ok()
}

/// Resolves candidate lockfile paths in strict priority order:
/// 1. `RIFTCORE_LOCKFILE` env var (when provided and non-empty — overrides auto-discovery)
/// 2. Parent directory of active `LeagueClientUx.exe`
/// 3. Standard Riot Games installation folders across C..F drives
pub fn resolve_lockfile_candidates(
    env_override: Option<&str>,
    ux_exe_path: Option<&Path>,
) -> Vec<PathBuf> {
    if let Some(override_path) = env_override.map(str::trim).filter(|s| !s.is_empty()) {
        return vec![PathBuf::from(override_path)];
    }

    let mut candidates = Vec::new();
    if let Some(parent) = ux_exe_path.and_then(Path::parent) {
        candidates.push(parent.join("lockfile"));
    }
    for drive in ['C', 'D', 'E', 'F'] {
        candidates.push(PathBuf::from(format!(
            "{drive}:\\Riot Games\\League of Legends\\lockfile"
        )));
        candidates.push(PathBuf::from(format!(
            "{drive}:\\Riot Games\\League of Legends (PBE)\\lockfile"
        )));
        candidates.push(PathBuf::from(format!(
            "{drive}:\\Program Files\\Riot Games\\League of Legends\\lockfile"
        )));
    }
    candidates
}

pub fn discover_credentials() -> Option<LcuCredentials> {
    let lockfile = discover_lcu_lockfile()?;
    let lockfile_path = lockfile
        .install_dir
        .as_ref()
        .map(|dir| dir.join("lockfile"));
    LcuCredentials::from_lockfile(lockfile, lockfile_path).ok()
}

pub fn is_riot_client_open() -> bool {
    platform_is_riot_client_open()
}

pub fn launch_league() -> Result<String, LcuError> {
    let configured = std::env::var("RIOT_CLIENT_INSTALL_DIR")
        .ok()
        .map(|directory| PathBuf::from(directory).join("RiotClientServices.exe"));
    let candidates = configured
        .into_iter()
        .chain(['C', 'D', 'E', 'F'].into_iter().map(|drive| {
            PathBuf::from(format!(
                "{drive}:\\Riot Games\\Riot Client\\RiotClientServices.exe"
            ))
        }));

    for executable in candidates {
        if executable.is_file() {
            Command::new(&executable)
                .args([
                    "--launch-product=league_of_legends",
                    "--launch-patchline=live",
                ])
                .spawn()
                .map_err(|error| LcuError::Io(format!("Could not launch Riot Client: {error}")))?;
            return Ok("Launching League of Legends through Riot Client".to_string());
        }
    }
    Err(LcuError::UnsupportedOperation(
        "RiotClientServices.exe was not found. Start League from Riot Client or install it in a standard Riot Games folder.".to_string(),
    ))
}

pub fn find_league_game_cfg(install_dir: Option<&Path>) -> Option<PathBuf> {
    let from_client = install_dir.map(|directory| directory.join("Config").join("game.cfg"));
    let configured = std::env::var("LEAGUE_OF_LEGENDS_INSTALL_DIR")
        .ok()
        .map(|directory| PathBuf::from(directory).join("Config").join("game.cfg"));
    let candidates =
        from_client
            .into_iter()
            .chain(configured)
            .chain(['C', 'D', 'E', 'F'].into_iter().flat_map(|drive| {
                [
                    format!("{drive}:\\Riot Games\\League of Legends\\Config\\game.cfg"),
                    format!("{drive}:\\Riot Games\\League of Legends (PBE)\\Config\\game.cfg"),
                    format!(
                        "{drive}:\\Program Files\\Riot Games\\League of Legends\\Config\\game.cfg"
                    ),
                ]
                .into_iter()
                .map(PathBuf::from)
            }));
    candidates.into_iter().find(|path| path.is_file())
}

pub fn set_league_window_mode(mode: u8, install_dir: Option<&Path>) -> Result<String, LcuError> {
    if !matches!(mode, 0 | 2) {
        return Err(LcuError::UnsupportedOperation(
            "Unsupported League display mode".to_string(),
        ));
    }
    let Some(path) = find_league_game_cfg(install_dir) else {
        return Err(LcuError::Io(
            "League game.cfg was not found; launch League once and try again".to_string(),
        ));
    };
    let mut content = std::fs::read_to_string(&path)
        .map_err(|error| LcuError::Io(format!("Could not read League game.cfg: {error}")))?;

    let newline = if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut found = false;
    let mut lines = content
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("WindowMode=") {
                found = true;
                let indent = &line[..line.len() - line.trim_start().len()];
                format!("{indent}WindowMode={mode}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>();

    if !found {
        if let Some(general) = lines.iter().position(|line| line.trim() == "[General]") {
            lines.insert(general + 1, format!("WindowMode={mode}"));
        } else {
            if lines.last().is_some_and(|line| !line.is_empty()) {
                lines.push(String::new());
            }
            lines.push("[General]".to_string());
            lines.push(format!("WindowMode={mode}"));
        }
    }
    content = lines.join(newline);
    content.push_str(newline);

    let mut backup = path.as_os_str().to_os_string();
    backup.push(".riftcore-backup");
    let backup = PathBuf::from(backup);
    if !backup.exists() {
        std::fs::copy(&path, &backup)
            .map_err(|error| LcuError::Io(format!("Could not back up League game.cfg: {error}")))?;
    }
    std::fs::write(&path, content)
        .map_err(|error| LcuError::Io(format!("Could not update League game.cfg: {error}")))?;

    Ok(format!(
        "League display mode set to {}. Restart the game to apply it.",
        if mode == 2 {
            "borderless"
        } else {
            "fullscreen"
        }
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    #[test]
    fn parse_lockfile_valid_u16_port_and_pid() {
        let raw = "LeagueClient:4242:51399:s3cr3t-pw:https\n";
        let path = Path::new("C:\\Riot Games\\League of Legends\\lockfile");
        let creds = parse_lockfile_contents(raw, path, false).expect("valid lockfile must parse");

        assert_eq!(creds.pid, 4242);
        assert_eq!(creds.port, 51399u16);
        assert_eq!(creds.password(), "s3cr3t-pw");
        assert_eq!(creds.protocol, "https");
        assert_eq!(creds.base_url(), "https://127.0.0.1:51399");
        assert_eq!(
            creds.install_dir,
            Some(PathBuf::from("C:\\Riot Games\\League of Legends"))
        );
    }

    #[test]
    fn parse_lockfile_rejects_invalid_ports_and_pids() {
        let path = Path::new("C:\\Riot Games\\League of Legends\\lockfile");
        assert!(parse_lockfile_contents("LeagueClient:123:65536:pw:https", path, false).is_none());
        assert!(parse_lockfile_contents("LeagueClient:123:0:pw:https", path, false).is_none());
        assert!(parse_lockfile_contents("LeagueClient:123:-1:pw:https", path, false).is_none());
        assert!(parse_lockfile_contents("LeagueClient:abc:2999:pw:https", path, false).is_none());
        assert!(parse_lockfile_contents("LeagueClient:0:2999:pw:https", path, false).is_none());
        assert!(parse_lockfile_contents("LeagueClient:123:2999::https", path, false).is_none());
        assert!(parse_lockfile_contents("LeagueClient:123:2999:pw", path, false).is_none());
    }

    #[test]
    fn lockfile_env_override_takes_strict_priority_over_ux_and_defaults() {
        let ux = PathBuf::from("D:\\Games\\Riot Games\\League of Legends\\LeagueClientUx.exe");
        let overridden = resolve_lockfile_candidates(Some("E:\\Custom\\lockfile"), Some(&ux));
        assert_eq!(overridden, vec![PathBuf::from("E:\\Custom\\lockfile")]);

        let fallback = resolve_lockfile_candidates(Some("   "), Some(&ux));
        assert_eq!(
            fallback.first(),
            Some(&PathBuf::from(
                "D:\\Games\\Riot Games\\League of Legends\\lockfile"
            ))
        );
        assert!(fallback.len() > 1);
    }

    #[test]
    fn credentials_debug_redacts_password() {
        let creds = LcuCredentials::new(100, 12345, "super-secret-token", "https", None, None)
            .expect("valid creds");
        let formatted = format!("{creds:?}");
        assert!(!formatted.contains("super-secret-token"));
        assert!(formatted.contains("<redacted>"));
    }

    #[test]
    fn lcu_client_sends_basic_auth_and_parses_json() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let addr = listener.local_addr().unwrap();

        let srv = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 2048];
            let read = stream.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..read]);
            // Basic base64("riot:my-token") == "cmlvdDpteS10b2tlbg=="
            assert!(
                req.to_ascii_lowercase()
                    .contains("authorization: basic cmlvddptes10b2tlbg==")
                    || req.contains("Authorization: Basic cmlvdDpteS10b2tlbg=="),
                "unexpected request headers: {req}"
            );
            let body = "\"ChampSelect\"";
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(resp.as_bytes()).unwrap();
        });

        let creds = LcuCredentials::new(100, addr.port(), "my-token", "http", None, None).unwrap();
        let client = LcuClient::new(creds).unwrap();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let phase = rt.block_on(client.gameflow_phase()).unwrap();
        assert_eq!(phase, "ChampSelect");
        srv.join().unwrap();
    }

    #[test]
    fn export_item_set_fails_closed_on_server_error_without_wiping_sets() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let addr = listener.local_addr().unwrap();

        let srv = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 2048];
            let _ = stream.read(&mut buf);
            let resp = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 5\r\nConnection: close\r\n\r\nerror";
            stream.write_all(resp.as_bytes()).unwrap();
        });

        let creds = LcuCredentials::new(100, addr.port(), "my-token", "http", None, None).unwrap();
        let client = LcuClient::new(creds).unwrap();
        let summoner = serde_json::json!({ "summonerId": 999 });
        let item_set =
            serde_json::json!({ "title": "RiftCore: Ahri MID", "associatedChampions": [103] });
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let err = rt
            .block_on(client.export_item_set(item_set, Some(&summoner)))
            .expect_err("500 GET must not proceed to PUT wipe");
        assert!(matches!(err, LcuError::HttpStatus { status: 500, .. }));
        srv.join().unwrap();
    }

    #[test]
    fn import_runes_preserves_caller_page_name() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let addr = listener.local_addr().unwrap();

        let srv = std::thread::spawn(move || {
            for step in 0..3 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buf = [0u8; 4096];
                let read = stream.read(&mut buf).unwrap();
                let req = String::from_utf8_lossy(&buf[..read]);
                let body = match step {
                    0 => r#"[{"id":7,"name":"RiftCore Old","isEditable":true}]"#,
                    1 => {
                        assert!(
                            req.contains("RiftCore Ahri MID"),
                            "expected custom name preserved: {req}"
                        );
                        r#"{"id":7}"#
                    }
                    _ => "{}",
                };
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream.write_all(resp.as_bytes()).unwrap();
            }
        });

        let creds = LcuCredentials::new(100, addr.port(), "my-token", "http", None, None).unwrap();
        let client = LcuClient::new(creds).unwrap();
        let page = serde_json::json!({
            "name": "RiftCore Ahri MID",
            "primaryStyleId": 8100,
            "subStyleId": 8200,
            "selectedPerkIds": [8112]
        });
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let msg = rt.block_on(client.import_runes(page)).unwrap();
        assert!(msg.contains("activated"));
        srv.join().unwrap();
    }
}
