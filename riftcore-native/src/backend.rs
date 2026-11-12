use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppSettings {
    pub auto_accept: bool,
    pub auto_import_runes: bool,
    pub auto_import_spells: bool,
    pub auto_import_item_sets: bool,
    pub flash_slot_preference: String,
    pub sound_alerts: bool,
    pub always_on_top: bool,
    pub close_to_tray: bool,
    pub close_confirmation_seen: bool,
    pub theme: String,
    pub font_family: String,
    pub first_run_tour_seen: bool,
    pub last_seen_version: String,
    pub riot_region: String,
    pub riot_api_key: String,
    pub hud_gold_lead: bool,
    pub hud_cs_benchmark: bool,
    pub hud_objective_timers: bool,
    pub hud_ally_ult_timers: bool,
    pub hud_skill_order: bool,
    pub hud_item_goal: bool,
    pub hud_trinket_reminder: bool,
    pub hud_mayhem_cards: bool,
    pub hud_jungle_pathing: bool,
    pub hud_map_objective_pins: bool,
    pub hud_map_route_pins: bool,
    pub hud_map_ward_pins: bool,
    pub jungle_start_buff: String,
    pub hud_aram_health_timers: bool,
    pub hud_mode: String,
    pub hud_minimap_scale: f32,
    pub hud_minimap_flip: bool,
    pub hud_minimap_zoom: f32,
    pub hud_minimap_nudge_x: f32,
    pub hud_minimap_nudge_y: f32,
    pub hud_edit_layout: bool,
    pub hud_positions: HashMap<String, [f32; 2]>,
    pub favorite_champions: Vec<u32>,
    pub champion_pool: HashMap<String, Vec<u32>>,
    pub tft_board: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_accept: true,
            auto_import_runes: true,
            auto_import_spells: true,
            auto_import_item_sets: true,
            flash_slot_preference: "F".to_string(),
            sound_alerts: true,
            always_on_top: false,
            close_to_tray: true,
            close_confirmation_seen: false,
            theme: "hextech".to_string(),
            font_family: "plus-jakarta".to_string(),
            first_run_tour_seen: false,
            last_seen_version: String::new(),
            riot_region: "eun1".to_string(),
            riot_api_key: String::new(),
            hud_gold_lead: true,
            hud_cs_benchmark: true,
            hud_objective_timers: true,
            hud_ally_ult_timers: true,
            hud_skill_order: true,
            hud_item_goal: true,
            hud_trinket_reminder: true,
            hud_mayhem_cards: true,
            hud_jungle_pathing: true,
            hud_map_objective_pins: true,
            hud_map_route_pins: true,
            hud_map_ward_pins: false,
            jungle_start_buff: "red".to_string(),
            hud_aram_health_timers: true,
            hud_mode: "integrated".to_string(),
            hud_minimap_scale: 2.16,
            hud_minimap_flip: false,
            hud_minimap_zoom: 1.0,
            hud_minimap_nudge_x: 0.0,
            hud_minimap_nudge_y: 0.0,
            hud_edit_layout: false,
            hud_positions: HashMap::new(),
            favorite_champions: Vec::new(),
            champion_pool: HashMap::from([
                ("TOP".to_string(), vec![266, 122, 24]),
                ("JUNGLE".to_string(), vec![64, 254, 121]),
                ("MID".to_string(), vec![103, 238, 517]),
                ("ADC".to_string(), vec![222, 51, 67]),
                ("SUPPORT".to_string(), vec![412, 53, 99]),
            ]),
            tft_board: vec![String::new(); 28],
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CoreSnapshot {
    pub connected: bool,
    pub riot_client_open: bool,
    pub phase: String,
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
    pub notice: Option<String>,
    pub last_updated: Option<Instant>,
}

#[derive(Debug)]
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
}

#[derive(Clone, Debug)]
struct LcuCredentials {
    port: String,
    token: String,
    install_dir: Option<PathBuf>,
}

pub fn start_core(ctx: eframe::egui::Context) -> (Receiver<CoreSnapshot>, Sender<CoreCommand>) {
    let (snapshot_tx, snapshot_rx) = mpsc::channel();
    let (command_tx, command_rx) = mpsc::channel();

    thread::Builder::new()
        .name("riftcore-client-monitor".to_string())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    let _ = snapshot_tx.send(CoreSnapshot {
                        notice: Some(format!(
                            "Unable to start the League connection service: {error}"
                        )),
                        ..Default::default()
                    });
                    return;
                }
            };
            let client = match Client::builder()
                .danger_accept_invalid_certs(true)
                .connect_timeout(Duration::from_millis(500))
                .timeout(Duration::from_millis(1200))
                .user_agent("RiftCore Companion")
                .build()
            {
                Ok(client) => client,
                Err(error) => {
                    let _ = snapshot_tx.send(CoreSnapshot {
                        notice: Some(format!(
                            "Unable to initialize the League connection: {error}"
                        )),
                        ..Default::default()
                    });
                    return;
                }
            };
            let riot_client = match Client::builder()
                .connect_timeout(Duration::from_secs(4))
                .timeout(Duration::from_secs(8))
                .user_agent(format!("RiftCore Companion/{}", env!("CARGO_PKG_VERSION")))
                .build()
            {
                Ok(client) => client,
                Err(error) => {
                    let _ = snapshot_tx.send(CoreSnapshot {
                        notice: Some(format!("Unable to initialize Riot API access: {error}")),
                        ..Default::default()
                    });
                    return;
                }
            };

            let mut snapshot = CoreSnapshot::default();
            let mut credentials: Option<LcuCredentials> = None;
            let mut auto_accept = true;
            let mut refresh_credentials = Instant::now() - Duration::from_secs(10);
            let mut refresh_profile = Instant::now() - Duration::from_secs(10);
            let mut refresh_history = Instant::now() - Duration::from_secs(60);
            let mut refresh_live = Instant::now() - Duration::from_secs(10);
            let mut refresh_champ_select = Instant::now() - Duration::from_secs(10);
            let mut refresh_ready = Instant::now() - Duration::from_secs(10);
            let mut did_auto_accept = false;

            loop {
                while let Ok(command) = command_rx.try_recv() {
                    match command {
                        CoreCommand::SetAutoAccept(enabled) => auto_accept = enabled,
                        CoreCommand::Refresh => {
                            refresh_credentials = Instant::now() - Duration::from_secs(10);
                            refresh_profile = Instant::now() - Duration::from_secs(10);
                        }
                        CoreCommand::AcceptMatch => {
                            snapshot.notice = Some(match credentials.as_ref() {
                                Some(creds) => runtime.block_on(accept_match(&client, creds)),
                                None => "League Client is not connected".to_string(),
                            });
                        }
                        CoreCommand::ImportSpells { spell1, spell2 } => {
                            snapshot.notice = Some(match credentials.as_ref() {
                                Some(creds) => {
                                    runtime.block_on(import_spells(&client, creds, spell1, spell2))
                                }
                                None => "League Client is not connected".to_string(),
                            });
                        }
                        CoreCommand::ImportRunes(page) => {
                            snapshot.notice = Some(match credentials.as_ref() {
                                Some(creds) => runtime.block_on(import_runes(&client, creds, page)),
                                None => "League Client is not connected".to_string(),
                            });
                        }
                        CoreCommand::ExportItemSet(item_set) => {
                            snapshot.notice = Some(match credentials.as_ref() {
                                Some(creds) => {
                                    runtime.block_on(export_item_set(&client, creds, item_set))
                                }
                                None => "League Client is not connected".to_string(),
                            });
                        }
                        CoreCommand::LookupRiotId { riot_id, region, api_key } => {
                            snapshot.riot_lookup = None;
                            snapshot.notice = Some(format!("Looking up {riot_id} through Riot API…"));
                            let _ = snapshot_tx.send(snapshot.clone());
                            snapshot.riot_lookup = Some(match runtime.block_on(lookup_riot_account(
                                &riot_client, &riot_id, &region, &api_key,
                            )) {
                                Ok(data) => data,
                                Err(error) => serde_json::json!({ "error": error }),
                            });
                            snapshot.notice = snapshot.riot_lookup.as_ref()
                                .and_then(|result| result.get("error"))
                                .and_then(Value::as_str)
                                .map(str::to_owned)
                                .or_else(|| Some("Riot account lookup complete".to_string()));
                        }
                        CoreCommand::ValidateRiotApiKey { region, api_key } => {
                            snapshot.riot_api_status = Some(runtime.block_on(validate_riot_key(
                                &riot_client, &region, &api_key,
                            )));
                        }
                        CoreCommand::CheckUpdates => {
                            snapshot.update_status = Some("Checking GitHub Releases…".to_string());
                            let _ = snapshot_tx.send(snapshot.clone());
                            match runtime.block_on(fetch_latest_release(&riot_client)) {
                                Ok(release) => {
                                    let latest = release.get("tag_name").and_then(Value::as_str).unwrap_or("unknown");
                                    let current = env!("CARGO_PKG_VERSION");
                                    snapshot.update_status = Some(if is_newer_version(latest, current) {
                                        if release_asset(&release, "RiftCore.exe").is_some() {
                                            format!("RiftCore {latest} is available")
                                        } else {
                                            format!("RiftCore {latest} is available; download setup from the release page")
                                        }
                                    } else {
                                        format!("You have the latest release ({current})")
                                    });
                                    snapshot.latest_release = Some(release);
                                }
                                Err(error) => {
                                    snapshot.update_status = Some(error.clone());
                                    snapshot.latest_release = Some(serde_json::json!({ "error": error }));
                                }
                            }
                        }
                        CoreCommand::InstallUpdate => {
                            snapshot.update_status = Some("Downloading update package…".to_string());
                            let _ = snapshot_tx.send(snapshot.clone());
                            let result = match snapshot.latest_release.as_ref() {
                                Some(release) => runtime.block_on(download_and_prepare_update(
                                    &riot_client,
                                    release,
                                )),
                                None => Err("Check for updates before installing".to_string()),
                            };
                            snapshot.update_status = Some(match result {
                                Ok(()) => "Installing update and restarting…".to_string(),
                                Err(error) => error,
                            });
                        }
                        CoreCommand::LaunchLeague => {
                            snapshot.notice = Some(launch_league());
                        }
                        CoreCommand::SetLeagueWindowMode(mode) => {
                            snapshot.notice = Some(set_league_window_mode(mode, credentials.as_ref()));
                        }
                    }
                }

                if refresh_credentials.elapsed() >= Duration::from_secs(2) {
                    credentials = detect_league_client();
                    snapshot.connected = credentials.is_some();
                    snapshot.riot_client_open = is_riot_client_open();
                    refresh_credentials = Instant::now();
                    if credentials.is_none() {
                        snapshot.phase = "None".to_string();
                        snapshot.summoner = None;
                        snapshot.ranked = None;
                        snapshot.champ_select = None;
                        snapshot.ready_check = None;
                        snapshot.match_history = None;
                        snapshot.live_game = None;
                        did_auto_accept = false;
                    }
                }

                if let Some(creds) = credentials.as_ref() {
                    if let Some(phase) = runtime.block_on(get_json(
                        &client,
                        creds,
                        "/lol-gameflow/v1/gameflow-phase",
                    )) {
                        snapshot.phase = phase.as_str().unwrap_or("None").to_string();
                    }

                    if refresh_profile.elapsed() >= Duration::from_secs(12) {
                        snapshot.summoner = runtime.block_on(get_json(
                            &client,
                            creds,
                            "/lol-summoner/v1/current-summoner",
                        ));
                        snapshot.ranked = runtime.block_on(get_json(
                            &client,
                            creds,
                            "/lol-ranked/v1/current-ranked-stats",
                        ));
                        refresh_profile = Instant::now();
                    }

                    if refresh_history.elapsed() >= Duration::from_secs(45) {
                        if let Some(puuid) = snapshot.summoner.as_ref()
                            .and_then(|summoner| summoner.get("puuid"))
                            .and_then(Value::as_str)
                        {
                            let endpoint = format!(
                                "/lol-match-history/v1/products/lol/{puuid}/matches?begIndex=0&endIndex=10"
                            );
                            snapshot.match_history = runtime.block_on(get_json(&client, creds, &endpoint));
                        }
                        refresh_history = Instant::now();
                    }

                    if snapshot.phase == "ChampSelect"
                        && refresh_champ_select.elapsed() >= Duration::from_millis(800)
                    {
                        snapshot.champ_select = runtime.block_on(get_json(
                            &client,
                            creds,
                            "/lol-champ-select/v1/session",
                        ));
                        refresh_champ_select = Instant::now();
                    } else if snapshot.phase != "ChampSelect" {
                        snapshot.champ_select = None;
                        refresh_champ_select = Instant::now() - Duration::from_secs(10);
                    }

                    if snapshot.phase == "ReadyCheck"
                        && refresh_ready.elapsed() >= Duration::from_millis(700)
                    {
                        snapshot.ready_check = runtime.block_on(get_json(
                            &client,
                            creds,
                            "/lol-matchmaking/v1/ready-check",
                        ));
                        refresh_ready = Instant::now();
                        let awaiting_response = snapshot
                            .ready_check
                            .as_ref()
                            .map(|data| {
                                data.get("state").and_then(Value::as_str) == Some("InProgress")
                                    && data.get("playerResponse").and_then(Value::as_str)
                                        == Some("None")
                            })
                            .unwrap_or(false);
                        if awaiting_response && auto_accept && !did_auto_accept {
                            did_auto_accept = true;
                            snapshot.notice = Some(runtime.block_on(accept_match(&client, creds)));
                        } else if !awaiting_response {
                            did_auto_accept = false;
                        }
                    } else if snapshot.phase != "ReadyCheck" {
                        snapshot.ready_check = None;
                        refresh_ready = Instant::now() - Duration::from_secs(10);
                        did_auto_accept = false;
                    }
                }

                if refresh_live.elapsed() >= Duration::from_millis(900) {
                    snapshot.live_game = runtime.block_on(get_live_game(&client));
                    refresh_live = Instant::now();
                    if snapshot.live_game.is_some() {
                        snapshot.phase = "InProgress".to_string();
                    }
                }

                snapshot.last_updated = Some(Instant::now());
                let _ = snapshot_tx.send(snapshot.clone());
                ctx.request_repaint();
                thread::sleep(Duration::from_millis(220));
            }
        })
        .expect("failed to start the League client monitor");

    (snapshot_rx, command_tx)
}

async fn get_json(client: &Client, creds: &LcuCredentials, endpoint: &str) -> Option<Value> {
    let url = format!("https://127.0.0.1:{}{}", creds.port, endpoint);
    client
        .get(url)
        .basic_auth("riot", Some(&creds.token))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json::<Value>()
        .await
        .ok()
}

async fn lcu_request(
    client: &Client,
    creds: &LcuCredentials,
    method: Method,
    endpoint: &str,
    body: Option<Value>,
) -> Result<Value, String> {
    let url = format!("https://127.0.0.1:{}{}", creds.port, endpoint);
    let mut request = client
        .request(method, url)
        .basic_auth("riot", Some(&creds.token));
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request.send().await.map_err(|error| error.to_string())?;
    let status = response.status();
    let text = response.text().await.map_err(|error| error.to_string())?;
    if !status.is_success() {
        return Err(format!("League Client returned HTTP {status}: {text}"));
    }
    Ok(serde_json::from_str(&text).unwrap_or(Value::Null))
}

async fn accept_match(client: &Client, creds: &LcuCredentials) -> String {
    match lcu_request(
        client,
        creds,
        Method::POST,
        "/lol-matchmaking/v1/ready-check/accept",
        Some(Value::Object(Default::default())),
    )
    .await
    {
        Ok(_) => "Match accepted".to_string(),
        Err(error) => format!("Could not accept the match: {error}"),
    }
}

async fn import_spells(
    client: &Client,
    creds: &LcuCredentials,
    spell1: i64,
    spell2: i64,
) -> String {
    let body = serde_json::json!({"spell1Id": spell1, "spell2Id": spell2});
    match lcu_request(
        client,
        creds,
        Method::PATCH,
        "/lol-champ-select/v1/session/my-selection",
        Some(body),
    )
    .await
    {
        Ok(_) => "Summoner spells sent to League Client".to_string(),
        Err(error) => format!("Could not set summoner spells: {error}"),
    }
}

async fn import_runes(client: &Client, creds: &LcuCredentials, mut page: Value) -> String {
    let pages_url = format!("https://127.0.0.1:{}/lol-perks/v1/pages", creds.port);
    let pages = match client
        .get(&pages_url)
        .basic_auth("riot", Some(&creds.token))
        .send()
        .await
    {
        Ok(response) => response.json::<Vec<Value>>().await.unwrap_or_default(),
        Err(_) => Vec::new(),
    };
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
        if owned && name.contains("RiftCore") {
            entry.get("id").and_then(Value::as_i64)
        } else {
            None
        }
    });
    if let Some(object) = page.as_object_mut() {
        object.insert("current".to_string(), Value::Bool(true));
        object.remove("isActive");
        object.insert(
            "name".to_string(),
            Value::String("RiftCore Build".to_string()),
        );
    }

    let (result, page_id) = if let Some(id) = target {
        if let Some(object) = page.as_object_mut() {
            object.insert("id".to_string(), Value::from(id));
        }
        match lcu_request(
            client,
            creds,
            Method::PUT,
            &format!("/lol-perks/v1/pages/{id}"),
            Some(page.clone()),
        )
        .await
        {
            Ok(value) => (Ok(value), Some(id)),
            Err(_) => (create_rune_page(client, creds, &mut page).await, None),
        }
    } else {
        (create_rune_page(client, creds, &mut page).await, None)
    };

    match result {
        Ok(created) => {
            let page_id = page_id.or_else(|| created.get("id").and_then(Value::as_i64));
            if let Some(id) = page_id {
                let _ = lcu_request(
                    client,
                    creds,
                    Method::PUT,
                    "/lol-perks/v1/currentpage",
                    Some(Value::from(id)),
                )
                .await;
            }
            "Rune page imported".to_string()
        }
        Err(error) => format!("Could not import rune page: {error}"),
    }
}

async fn export_item_set(client: &Client, creds: &LcuCredentials, item_set: Value) -> String {
    let summoner = match get_json(client, creds, "/lol-summoner/v1/current-summoner").await {
        Some(summoner) => summoner,
        None => return "League Client could not load the current summoner".to_string(),
    };
    let id = summoner
        .get("accountId")
        .or_else(|| summoner.get("summonerId"))
        .and_then(Value::as_u64)
        .map(|id| id.to_string());
    let Some(id) = id else {
        return "League Client did not provide an item set account ID".to_string();
    };
    let endpoint = format!("/lol-item-sets/v1/item-sets/{id}/sets");
    let mut existing = lcu_request(client, creds, Method::GET, &endpoint, None)
        .await
        .unwrap_or_else(|_| serde_json::json!({"itemSets": [], "timestamp": 0}));
    let Some(sets) = existing.get_mut("itemSets").and_then(Value::as_array_mut) else {
        existing = serde_json::json!({"itemSets": [], "timestamp": 0});
        let Some(sets) = existing.get_mut("itemSets").and_then(Value::as_array_mut) else {
            return "Could not prepare an in-game item set".to_string();
        };
        sets.push(item_set);
        return match lcu_request(client, creds, Method::PUT, &endpoint, Some(existing)).await {
            Ok(_) => "In-game item set added. Open the shop with P to use it.".to_string(),
            Err(error) => format!("Could not export item set: {error}"),
        };
    };
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
            .is_some_and(|ids| ids.iter().any(|id| id == &champion_id));
        !(is_riftcore && same_champion)
    });
    sets.push(item_set);
    match lcu_request(client, creds, Method::PUT, &endpoint, Some(existing)).await {
        Ok(_) => "In-game item set added. Open the shop with P to use it.".to_string(),
        Err(error) => format!("Could not export item set: {error}"),
    }
}

async fn validate_riot_key(client: &Client, region: &str, api_key: &str) -> String {
    let Some(region) = valid_platform(region) else {
        return "Choose a supported Riot platform region first".to_string();
    };
    if api_key.trim().is_empty() {
        return "Add a Riot API key in Preferences before testing it".to_string();
    }
    let url = format!("https://{region}.api.riotgames.com/lol/status/v4/platform-data");
    match riot_get(client, &url, api_key).await {
        Ok(_) => format!("Riot API key verified for {}", region.to_ascii_uppercase()),
        Err(error) => error,
    }
}

async fn fetch_latest_release(client: &Client) -> Result<Value, String> {
    client
        .get("https://api.github.com/repos/kotsioskl2/riftcore-companion/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|error| format!("Could not reach GitHub Releases: {error}"))?
        .error_for_status()
        .map_err(|error| format!("GitHub Releases returned an error: {error}"))?
        .json::<Value>()
        .await
        .map_err(|error| format!("GitHub Releases returned unreadable data: {error}"))
}

fn release_asset<'a>(release: &'a Value, name: &str) -> Option<&'a str> {
    release
        .get("assets")?
        .as_array()?
        .iter()
        .find(|asset| asset.get("name").and_then(Value::as_str) == Some(name))?
        .get("browser_download_url")?
        .as_str()
}

async fn download_and_prepare_update(client: &Client, release: &Value) -> Result<(), String> {
    let latest = release
        .get("tag_name")
        .and_then(Value::as_str)
        .ok_or_else(|| "GitHub release is missing its version tag".to_string())?;
    if !is_newer_version(latest, env!("CARGO_PKG_VERSION")) {
        return Err("This release is not newer than the installed version".to_string());
    }
    let download_url = release_asset(release, "RiftCore.exe").ok_or_else(|| {
        "This release has no automatic update package. Download the setup installer from the release page.".to_string()
    })?;
    let parsed_url = reqwest::Url::parse(download_url)
        .map_err(|_| "GitHub provided an invalid update URL".to_string())?;
    if parsed_url.scheme() != "https"
        || parsed_url.host_str() != Some("github.com")
        || !parsed_url
            .path()
            .starts_with("/kotsioskl2/riftcore-companion/releases/download/")
    {
        return Err("GitHub provided an unexpected update URL".to_string());
    }

    #[cfg(not(windows))]
    {
        let _ = (client, download_url);
        return Err("Automatic app updates are supported on Windows only".to_string());
    }

    #[cfg(windows)]
    {
        let response = client
            .get(parsed_url)
            .timeout(Duration::from_secs(300))
            .send()
            .await
            .map_err(|error| format!("Could not download the update: {error}"))?
            .error_for_status()
            .map_err(|error| format!("GitHub rejected the update download: {error}"))?;
        if response
            .content_length()
            .is_some_and(|length| !(1_000_000..=250_000_000).contains(&length))
        {
            return Err("The update package has an unexpected file size".to_string());
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|error| format!("Could not finish downloading the update: {error}"))?;
        if !(1_000_000..=250_000_000).contains(&bytes.len()) {
            return Err("The downloaded update has an unexpected file size".to_string());
        }

        prepare_self_update(&bytes)
    }
}

#[cfg(windows)]
fn prepare_self_update(bytes: &[u8]) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let executable = std::env::current_exe()
        .map_err(|error| format!("Could not locate the running app: {error}"))?;
    let temp_dir = std::env::temp_dir();
    let stem = format!("riftcore-update-{}", std::process::id());
    let downloaded = temp_dir.join(format!("{stem}.exe"));
    let helper = temp_dir.join(format!("{stem}.ps1"));
    std::fs::write(&downloaded, bytes)
        .map_err(|error| format!("Could not save the downloaded update: {error}"))?;

    let quote_ps = |path: &std::path::Path| path.to_string_lossy().replace('\'', "''");
    let script = format!(
        "$ErrorActionPreference = 'Stop'\n$waitForPid = {}\n$source = '{}'\n$target = '{}'\n$backup = $target + '.old'\n$updated = $false\ntry {{\n  Wait-Process -Id $waitForPid -ErrorAction SilentlyContinue\n  Start-Sleep -Milliseconds 400\n  if (Test-Path -LiteralPath $backup) {{ Remove-Item -LiteralPath $backup -Force }}\n  if (Test-Path -LiteralPath $target) {{ Move-Item -LiteralPath $target -Destination $backup -Force }}\n  Move-Item -LiteralPath $source -Destination $target -Force\n  Start-Process -FilePath $target\n  $updated = $true\n}} catch {{ }}\nif (-not $updated) {{\n  if (Test-Path -LiteralPath $backup) {{\n    if (Test-Path -LiteralPath $target) {{ Remove-Item -LiteralPath $target -Force }}\n    Move-Item -LiteralPath $backup -Destination $target -Force\n  }}\n  if (Test-Path -LiteralPath $target) {{ Start-Process -FilePath $target }}\n}} elseif (Test-Path -LiteralPath $backup) {{\n  Remove-Item -LiteralPath $backup -Force -ErrorAction SilentlyContinue\n}}\nif (Test-Path -LiteralPath $source) {{ Remove-Item -LiteralPath $source -Force -ErrorAction SilentlyContinue }}\nRemove-Item -LiteralPath $MyInvocation.MyCommand.Path -Force -ErrorAction SilentlyContinue\n",
        std::process::id(),
        quote_ps(&downloaded),
        quote_ps(&executable),
    );
    if let Err(error) = std::fs::write(&helper, script) {
        let _ = std::fs::remove_file(&downloaded);
        return Err(format!("Could not prepare the update helper: {error}"));
    }
    let spawn_result = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(&helper)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
    if let Err(error) = spawn_result {
        let _ = std::fs::remove_file(&downloaded);
        let _ = std::fs::remove_file(&helper);
        return Err(format!("Could not start the update helper: {error}"));
    }
    Ok(())
}

fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse = |version: &str| {
        version
            .trim_start_matches(['v', 'V'])
            .split(['-', '+'])
            .next()
            .unwrap_or_default()
            .split('.')
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    parse(latest) > parse(current)
}

async fn lookup_riot_account(
    client: &Client,
    riot_id: &str,
    region: &str,
    api_key: &str,
) -> Result<Value, String> {
    let platform = valid_platform(region)
        .ok_or_else(|| "Choose a supported Riot platform region".to_string())?;
    if api_key.trim().is_empty() {
        return Err("Add a Riot API key in Preferences before searching".to_string());
    }
    let trimmed = riot_id.trim();
    let (game_name, tag_line) = trimmed.split_once('#').unwrap_or((trimmed, platform));
    if game_name.trim().is_empty() || tag_line.trim().is_empty() {
        return Err("Enter a Riot ID in the form GameName#TAG".to_string());
    }
    let cluster = platform_cluster(platform);
    let account_url = riot_url(
        &format!("https://{cluster}.api.riotgames.com"),
        &[
            "riot",
            "account",
            "v1",
            "accounts",
            "by-riot-id",
            game_name.trim(),
            tag_line.trim(),
        ],
    )?;
    let account = riot_get(client, &account_url, api_key).await?;
    let puuid = account
        .get("puuid")
        .and_then(Value::as_str)
        .ok_or_else(|| "Riot API returned an account without a PUUID".to_string())?;

    let platform_candidates = platforms_for_cluster(cluster, platform);
    let mut effective_platform = platform.to_string();
    let mut summoner = None;
    let mut last_platform_error = None;
    for candidate in platform_candidates {
        let url = riot_url(
            &format!("https://{candidate}.api.riotgames.com"),
            &["lol", "summoner", "v4", "summoners", "by-puuid", puuid],
        )?;
        match riot_get(client, &url, api_key).await {
            Ok(data) => {
                effective_platform = candidate.to_string();
                summoner = Some(data);
                break;
            }
            Err(error) => {
                let is_auth = error.contains("invalid or expired")
                    || error.contains("forbidden")
                    || error.contains("rate limit");
                last_platform_error = Some(error);
                if is_auth {
                    break;
                }
            }
        }
    }
    let summoner = summoner.ok_or_else(|| last_platform_error.unwrap_or_else(|| "Riot account was found, but its League profile is not available on the selected server".to_string()))?;
    let effective_cluster = platform_cluster(&effective_platform);
    let platform_base = format!("https://{effective_platform}.api.riotgames.com");
    let cluster_base = format!("https://{effective_cluster}.api.riotgames.com");
    let ranked_url = riot_url(
        &platform_base,
        &["lol", "league", "v4", "entries", "by-puuid", puuid],
    )?;
    let mastery_url = riot_url(
        &platform_base,
        &[
            "lol",
            "champion-mastery",
            "v4",
            "champion-masteries",
            "by-puuid",
            puuid,
            "top",
        ],
    )?;
    let match_ids_url =
        format!("{cluster_base}/lol/match/v5/matches/by-puuid/{puuid}/ids?start=0&count=5");
    let (ranked, masteries, match_ids) = tokio::join!(
        riot_get(client, &ranked_url, api_key),
        riot_get(client, &mastery_url, api_key),
        riot_get(client, &match_ids_url, api_key),
    );
    let ranked = ranked.unwrap_or_else(|_| Value::Array(Vec::new()));
    let masteries = masteries.unwrap_or_else(|_| Value::Array(Vec::new()));
    let match_ids = match_ids
        .ok()
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default();
    let match_requests = match_ids.iter().filter_map(Value::as_str).map(|match_id| {
        let url = format!("{cluster_base}/lol/match/v5/matches/{match_id}");
        async move { riot_get(client, &url, api_key).await }
    });
    let matches = futures::future::join_all(match_requests)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "account": account,
        "summoner": summoner,
        "region": effective_platform,
        "ranked": ranked,
        "masteries": masteries,
        "matches": matches
    }))
}

async fn riot_get(client: &Client, url: &str, api_key: &str) -> Result<Value, String> {
    let response = client
        .get(url)
        .header("X-Riot-Token", api_key.trim())
        .send()
        .await
        .map_err(|error| format!("Could not reach Riot API: {error}"))?;
    let status = response.status();
    if status.is_success() {
        return response
            .json::<Value>()
            .await
            .map_err(|error| format!("Riot API returned unreadable data: {error}"));
    }
    Err(match status.as_u16() {
        401 => "Riot API key is invalid or expired".to_string(),
        403 => "Riot API denied this key or endpoint; check the key's permissions".to_string(),
        404 => "Riot ID or League profile was not found in that region".to_string(),
        429 => "Riot API rate limit reached; wait briefly and try again".to_string(),
        code => format!("Riot API returned HTTP {code}"),
    })
}

fn riot_url(base: &str, segments: &[&str]) -> Result<String, String> {
    let mut url = reqwest::Url::parse(base).map_err(|error| error.to_string())?;
    url.path_segments_mut()
        .map_err(|_| "Invalid Riot API URL".to_string())?
        .extend(segments.iter().copied());
    Ok(url.to_string())
}

fn valid_platform(region: &str) -> Option<&'static str> {
    match region.trim().to_ascii_lowercase().as_str() {
        "eun1" => Some("eun1"),
        "euw1" => Some("euw1"),
        "na1" => Some("na1"),
        "kr" => Some("kr"),
        "br1" => Some("br1"),
        "la1" => Some("la1"),
        "la2" => Some("la2"),
        "oc1" => Some("oc1"),
        "tr1" => Some("tr1"),
        "ru" => Some("ru"),
        "jp1" => Some("jp1"),
        "sg2" => Some("sg2"),
        "ph2" => Some("ph2"),
        "tw2" => Some("tw2"),
        "vn2" => Some("vn2"),
        _ => None,
    }
}

fn platform_cluster(platform: &str) -> &'static str {
    match platform {
        "na1" | "br1" | "la1" | "la2" => "americas",
        "kr" | "jp1" => "asia",
        "oc1" | "sg2" | "ph2" | "tw2" | "vn2" => "sea",
        _ => "europe",
    }
}

fn platforms_for_cluster(cluster: &str, preferred: &str) -> Vec<&'static str> {
    let all: &[&str] = match cluster {
        "americas" => &["na1", "br1", "la1", "la2"],
        "asia" => &["kr", "jp1"],
        "sea" => &["oc1", "sg2", "ph2", "tw2", "vn2"],
        _ => &["eun1", "euw1", "tr1", "ru"],
    };
    let mut regions = Vec::with_capacity(all.len());
    if let Some(region) = all.iter().find(|region| **region == preferred) {
        regions.push(*region);
    }
    regions.extend(all.iter().copied().filter(|region| *region != preferred));
    regions
}

async fn create_rune_page(
    client: &Client,
    creds: &LcuCredentials,
    page: &mut Value,
) -> Result<Value, String> {
    if let Some(object) = page.as_object_mut() {
        object.remove("id");
    }
    lcu_request(
        client,
        creds,
        Method::POST,
        "/lol-perks/v1/pages",
        Some(page.clone()),
    )
    .await
}

async fn get_live_game(client: &Client) -> Option<Value> {
    client
        .get("https://127.0.0.1:2999/liveclientdata/allgamedata")
        .timeout(Duration::from_millis(750))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json::<Value>()
        .await
        .ok()
}

fn launch_league() -> String {
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
            return match Command::new(&executable)
                .args([
                    "--launch-product=league_of_legends",
                    "--launch-patchline=live",
                ])
                .spawn()
            {
                Ok(_) => "Launching League of Legends through Riot Client".to_string(),
                Err(error) => format!("Could not launch Riot Client: {error}"),
            };
        }
    }
    "RiotClientServices.exe was not found. Start League from Riot Client or install it in a standard Riot Games folder.".to_string()
}

fn find_league_game_cfg(credentials: Option<&LcuCredentials>) -> Option<PathBuf> {
    let from_client = credentials
        .and_then(|credentials| credentials.install_dir.as_ref())
        .map(|directory| directory.join("Config").join("game.cfg"));
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

fn set_league_window_mode(mode: u8, credentials: Option<&LcuCredentials>) -> String {
    if !matches!(mode, 0 | 2) {
        return "Unsupported League display mode".to_string();
    }
    let Some(path) = find_league_game_cfg(credentials) else {
        return "League game.cfg was not found; launch League once and try again".to_string();
    };
    let mut content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) => return format!("Could not read League game.cfg: {error}"),
    };

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
        if let Err(error) = std::fs::copy(&path, &backup) {
            return format!("Could not back up League game.cfg: {error}");
        }
    }
    match std::fs::write(path, content) {
        Ok(()) => format!(
            "League display mode set to {}. Restart the game to apply it.",
            if mode == 2 {
                "borderless"
            } else {
                "fullscreen"
            }
        ),
        Err(error) => format!("Could not update League game.cfg: {error}"),
    }
}

fn detect_league_client() -> Option<LcuCredentials> {
    let mut candidates = Vec::new();
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
    if let Ok(path) = std::env::var("RIFTCORE_LOCKFILE") {
        candidates.insert(0, PathBuf::from(path));
    }
    if let Some(path) = find_league_client_ux_path() {
        if let Some(parent) = path.parent() {
            candidates.insert(0, parent.join("lockfile"));
        }
    }

    candidates.into_iter().find_map(|path| read_lockfile(&path))
}

fn read_lockfile(path: &Path) -> Option<LcuCredentials> {
    let content = std::fs::read_to_string(path).ok()?;
    let fields: Vec<&str> = content.trim().split(':').collect();
    if fields.len() < 5 || fields[2].is_empty() || fields[3].is_empty() {
        return None;
    }
    if let Ok(pid) = fields[1].parse::<u32>() {
        if !is_process_running(pid) {
            return None;
        }
    }
    Some(LcuCredentials {
        port: fields[2].to_string(),
        token: fields[3].to_string(),
        install_dir: path.parent().map(Path::to_path_buf),
    })
}

fn is_riot_client_open() -> bool {
    let path = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join("Riot Games")
        .join("Riot Client")
        .join("Config")
        .join("lockfile");
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| text.split(':').nth(1)?.parse::<u32>().ok())
        .map(is_process_running)
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_process_running(pid: u32) -> bool {
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
        fn GetExitCodeProcess(process: *mut std::ffi::c_void, code: *mut u32) -> i32;
        fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
    }
    unsafe {
        let process = OpenProcess(0x1000, 0, pid);
        if process.is_null() {
            return false;
        }
        let mut exit_code = 0;
        let active = GetExitCodeProcess(process, &mut exit_code) != 0 && exit_code == 259;
        CloseHandle(process);
        active
    }
}

#[cfg(not(windows))]
fn is_process_running(_pid: u32) -> bool {
    true
}

#[cfg(windows)]
#[repr(C)]
struct ProcessEntry32W {
    size: u32,
    usage: u32,
    process_id: u32,
    default_heap_id: usize,
    module_id: u32,
    threads: u32,
    parent_process_id: u32,
    base_priority: i32,
    flags: u32,
    executable: [u16; 260],
}

#[cfg(windows)]
fn find_league_client_ux_path() -> Option<PathBuf> {
    #[link(name = "kernel32")]
    extern "system" {
        fn CreateToolhelp32Snapshot(flags: u32, process_id: u32) -> *mut std::ffi::c_void;
        fn Process32FirstW(snapshot: *mut std::ffi::c_void, entry: *mut ProcessEntry32W) -> i32;
        fn Process32NextW(snapshot: *mut std::ffi::c_void, entry: *mut ProcessEntry32W) -> i32;
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
        fn QueryFullProcessImageNameW(
            process: *mut std::ffi::c_void,
            flags: u32,
            path: *mut u16,
            size: *mut u32,
        ) -> i32;
        fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
    }
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(0x0000_0002, 0);
        if snapshot.is_null() || snapshot as isize == -1 {
            return None;
        }
        let mut entry: ProcessEntry32W = std::mem::zeroed();
        entry.size = std::mem::size_of::<ProcessEntry32W>() as u32;
        let mut found = None;
        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let name = String::from_utf16_lossy(&entry.executable);
                if name
                    .trim_matches('\0')
                    .eq_ignore_ascii_case("LeagueClientUx.exe")
                {
                    let process = OpenProcess(0x1000, 0, entry.process_id);
                    if !process.is_null() {
                        let mut buffer = [0u16; 1024];
                        let mut size = buffer.len() as u32;
                        if QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut size)
                            != 0
                        {
                            found = Some(PathBuf::from(String::from_utf16_lossy(
                                &buffer[..size as usize],
                            )));
                        }
                        CloseHandle(process);
                    }
                    break;
                }
                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
        found
    }
}

#[cfg(not(windows))]
fn find_league_client_ux_path() -> Option<PathBuf> {
    None
}

#[cfg(windows)]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct WindowBounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[cfg(windows)]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeagueWindowInfo {
    pub hwnd: isize,
    pub process_id: u32,
    pub bounds: [i32; 4],
    pub dpi: u32,
}

#[cfg(windows)]
pub fn league_game_window() -> Option<LeagueWindowInfo> {
    #[link(name = "user32")]
    extern "system" {
        fn EnumWindows(
            callback: unsafe extern "system" fn(*mut std::ffi::c_void, isize) -> i32,
            parameter: isize,
        ) -> i32;
    }

    struct SearchResult {
        window: Option<LeagueWindowInfo>,
        area: i64,
    }

    unsafe extern "system" fn find_game_window(
        hwnd: *mut std::ffi::c_void,
        parameter: isize,
    ) -> i32 {
        #[link(name = "user32")]
        extern "system" {
            fn GetWindowThreadProcessId(hwnd: *mut std::ffi::c_void, process_id: *mut u32) -> u32;
            fn GetClientRect(hwnd: *mut std::ffi::c_void, rect: *mut WindowBounds) -> i32;
            fn ClientToScreen(hwnd: *mut std::ffi::c_void, point: *mut Point) -> i32;
            fn GetDpiForWindow(hwnd: *mut std::ffi::c_void) -> u32;
            fn IsWindowVisible(hwnd: *mut std::ffi::c_void) -> i32;
            fn IsIconic(hwnd: *mut std::ffi::c_void) -> i32;
        }
        #[link(name = "kernel32")]
        extern "system" {
            fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
            fn QueryFullProcessImageNameW(
                process: *mut std::ffi::c_void,
                flags: u32,
                path: *mut u16,
                size: *mut u32,
            ) -> i32;
            fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
        }

        if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
            return 1;
        }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return 1;
        }
        let process = OpenProcess(0x1000, 0, pid);
        if process.is_null() {
            return 1;
        }
        let mut buffer = [0u16; 1024];
        let mut size = buffer.len() as u32;
        let is_league = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut size) != 0
            && Path::new(&String::from_utf16_lossy(&buffer[..size as usize]))
                .file_name()
                .is_some_and(|name| name.eq_ignore_ascii_case("League of Legends.exe"));
        CloseHandle(process);
        if !is_league {
            return 1;
        }
        let mut client = WindowBounds {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if GetClientRect(hwnd, &mut client) == 0 {
            return 1;
        }
        let width = client.right - client.left;
        let height = client.bottom - client.top;
        if width < 800 || height < 500 {
            return 1;
        }
        let mut origin = Point {
            x: client.left,
            y: client.top,
        };
        if ClientToScreen(hwnd, &mut origin) == 0 {
            return 1;
        }

        let area = i64::from(width) * i64::from(height);
        let result = &mut *(parameter as *mut SearchResult);
        if area > result.area {
            result.area = area;
            result.window = Some(LeagueWindowInfo {
                hwnd: hwnd as isize,
                process_id: pid,
                bounds: [origin.x, origin.y, origin.x + width, origin.y + height],
                dpi: GetDpiForWindow(hwnd).max(96),
            });
        }
        1
    }

    let mut result = SearchResult {
        window: None,
        area: 0,
    };
    unsafe {
        EnumWindows(find_game_window, &mut result as *mut SearchResult as isize);
    }
    result.window
}

#[cfg(windows)]
pub fn league_window_is_foreground(window: LeagueWindowInfo) -> bool {
    #[link(name = "user32")]
    extern "system" {
        fn GetForegroundWindow() -> *mut std::ffi::c_void;
        fn GetWindowThreadProcessId(hwnd: *mut std::ffi::c_void, process_id: *mut u32) -> u32;
        fn GetWindowTextW(hwnd: *mut std::ffi::c_void, text: *mut u16, max_count: i32) -> i32;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcessId() -> u32;
    }

    let foreground = unsafe { GetForegroundWindow() };
    if foreground.is_null() {
        return false;
    }

    let mut foreground_pid = 0;
    unsafe { GetWindowThreadProcessId(foreground, &mut foreground_pid) };
    if foreground_pid == window.process_id {
        return true;
    }
    if foreground_pid != unsafe { GetCurrentProcessId() } {
        return false;
    }

    let mut title = [0u16; 128];
    let length = unsafe { GetWindowTextW(foreground, title.as_mut_ptr(), title.len() as i32) };
    length > 0
        && String::from_utf16_lossy(&title[..length as usize])
            .eq_ignore_ascii_case("RiftCore In-Game HUD")
}

#[cfg(not(windows))]
pub fn league_game_window() -> Option<LeagueWindowInfo> {
    None
}

#[cfg(not(windows))]
pub fn league_window_is_foreground(_window: LeagueWindowInfo) -> bool {
    false
}
