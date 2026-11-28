//! Background loading-screen scouting through the Riot API (the user's own developer key).
//!
//! [`spawn_scout`] looks players up one at a time on its own thread, so the League Client
//! and Live Client polling never wait on it, and reports each finished card as it goes.
//! Per player: `account-v1` (Riot ID → PUUID), `league-v4` entries, `champion-mastery-v4`
//! for the locked-in champion and a few recent `match-v5` matches — about 8 requests, which
//! keeps a 9-player scout inside a development key's 100 requests per 2 minutes. Rate-limit
//! answers wait for `Retry-After` and try again.

use crate::domain::scouting::{self, ScoutCard, ScoutTarget};
use crate::services::riot_api::{
    account_cluster, build_api_key_header, match_cluster, parse_retry_after, platform_gateway,
    riot_url,
};
use reqwest::header::RETRY_AFTER;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

/// Progress of a scout: the cards so far and a status line for the HUD.
#[derive(Clone, Debug, PartialEq)]
pub struct ScoutUpdate {
    /// [`scouting::roster_key`] of the match being scouted.
    pub roster: String,
    pub cards: Vec<ScoutCard>,
    pub status: Option<String>,
    pub done: bool,
}

const MAX_RETRIES: u32 = 3;
/// Longest single wait for a rate limit before giving up on that request.
const MAX_RETRY_WAIT: Duration = Duration::from_secs(30);

/// Starts scouting `targets` on a background thread; `on_update` runs after every update is
/// sent (use it to wake the UI). Dropping the receiver stops the scout after the current
/// player.
pub fn spawn_scout(
    targets: Vec<ScoutTarget>,
    region: String,
    api_key: String,
    on_update: impl Fn() + Send + 'static,
) -> Receiver<ScoutUpdate> {
    let (tx, rx) = mpsc::channel();
    let roster = scouting::roster_key(&targets);
    let spawned = std::thread::Builder::new()
        .name("riftcore-scout".to_string())
        .spawn({
            let tx = tx.clone();
            let roster = roster.clone();
            move || {
                let send = |cards: &[ScoutCard], status: Option<String>, done: bool| {
                    let ok = tx
                        .send(ScoutUpdate {
                            roster: roster.clone(),
                            cards: cards.to_vec(),
                            status,
                            done,
                        })
                        .is_ok();
                    on_update();
                    ok
                };
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(err) => {
                        send(&[], Some(format!("Scouting unavailable: {err}")), true);
                        return;
                    }
                };
                let client = match Client::builder()
                    .connect_timeout(Duration::from_secs(4))
                    .timeout(Duration::from_secs(8))
                    .user_agent(format!("RiftCore Companion/{}", env!("CARGO_PKG_VERSION")))
                    .build()
                {
                    Ok(client) => client,
                    Err(err) => {
                        send(&[], Some(format!("Scouting unavailable: {err}")), true);
                        return;
                    }
                };
                let total = targets.len();
                let mut cards = Vec::with_capacity(total);
                if !send(&cards, Some(format!("Scouting 0 / {total}")), false) {
                    return;
                }
                let mut auth_error = None;
                for target in &targets {
                    let card = match auth_error.as_ref() {
                        Some(err) => ScoutCard::failed(target, err),
                        None => match runtime
                            .block_on(scout_player(&client, target, &region, &api_key))
                        {
                            Ok(card) => card,
                            Err(err) => {
                                // A bad key or region fails every player the same way.
                                if err.fatal {
                                    auth_error = Some(err.message.clone());
                                }
                                ScoutCard::failed(target, err.message)
                            }
                        },
                    };
                    cards.push(card);
                    let status = format!("Scouting {} / {total}", cards.len());
                    if !send(&cards, Some(status), false) {
                        return;
                    }
                }
                let status = auth_error.or_else(|| {
                    let failed = cards.iter().filter(|c| c.error.is_some()).count();
                    (failed > 0)
                        .then(|| format!("{failed} of {total} players could not be scouted"))
                });
                send(&cards, status, true);
            }
        });
    if let Err(err) = spawned {
        let _ = tx.send(ScoutUpdate {
            roster,
            cards: Vec::new(),
            status: Some(format!("Scouting unavailable: {err}")),
            done: true,
        });
    }
    rx
}

#[derive(Debug)]
struct ScoutError {
    message: String,
    /// The key, region or permissions are wrong: every other player would fail too.
    fatal: bool,
}

impl ScoutError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            fatal: false,
        }
    }
}

/// GET `url`; `Ok(None)` for 404. Waits out 429 answers (up to [`MAX_RETRIES`] times).
async fn get(client: &Client, url: &str, api_key: &str) -> Result<Option<Value>, ScoutError> {
    let header = build_api_key_header(api_key).map_err(|err| ScoutError {
        message: err.to_string(),
        fatal: true,
    })?;
    let mut attempt = 0;
    loop {
        let response = client
            .get(url)
            .header("X-Riot-Token", header.clone())
            .send()
            .await
            .map_err(|err| ScoutError::new(format!("Could not reach Riot API: {err}")))?;
        match response.status() {
            status if status.is_success() => {
                return response
                    .json::<Value>()
                    .await
                    .map(Some)
                    .map_err(|err| ScoutError::new(format!("Unreadable Riot API data: {err}")));
            }
            StatusCode::NOT_FOUND => return Ok(None),
            StatusCode::TOO_MANY_REQUESTS if attempt < MAX_RETRIES => {
                attempt += 1;
                let wait = parse_retry_after(
                    response
                        .headers()
                        .get(RETRY_AFTER)
                        .and_then(|value| value.to_str().ok()),
                    Duration::from_secs(10),
                )
                .min(MAX_RETRY_WAIT);
                tokio::time::sleep(wait).await;
            }
            StatusCode::TOO_MANY_REQUESTS => {
                return Err(ScoutError::new("Riot API rate limit reached"));
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                return Err(ScoutError {
                    message: "Riot API key is invalid or expired".to_string(),
                    fatal: true,
                });
            }
            status => {
                return Err(ScoutError::new(format!(
                    "Riot API returned HTTP {}",
                    status.as_u16()
                )))
            }
        }
    }
}

fn url(base: &str, segments: &[&str]) -> Result<String, ScoutError> {
    riot_url(base, segments).map_err(|err| ScoutError::new(err.to_string()))
}

async fn scout_player(
    client: &Client,
    target: &ScoutTarget,
    region: &str,
    api_key: &str,
) -> Result<ScoutCard, ScoutError> {
    let fatal = |message: &str| ScoutError {
        message: message.to_string(),
        fatal: true,
    };
    let platform = platform_gateway(region)
        .ok_or_else(|| fatal("Choose your Riot region in Settings to scout players"))?;
    let account_base = account_cluster(region)
        .ok_or_else(|| fatal("Unsupported Riot region"))?
        .base_url();
    let match_base = match_cluster(region)
        .ok_or_else(|| fatal("Unsupported Riot region"))?
        .base_url();
    let platform_base = format!("https://{platform}.api.riotgames.com");

    let (game_name, tag_line) = target
        .riot_id
        .split_once('#')
        .ok_or_else(|| ScoutError::new("No Riot ID"))?;
    let account_url = url(
        &account_base,
        &[
            "riot",
            "account",
            "v1",
            "accounts",
            "by-riot-id",
            game_name,
            tag_line,
        ],
    )?;
    let account = get(client, &account_url, api_key)
        .await?
        .ok_or_else(|| ScoutError::new("Riot ID not found"))?;
    let puuid = account
        .get("puuid")
        .and_then(Value::as_str)
        .ok_or_else(|| ScoutError::new("Riot account without a PUUID"))?
        .to_string();

    let ranked_url = url(
        &platform_base,
        &["lol", "league", "v4", "entries", "by-puuid", &puuid],
    )?;
    let champion_id = target.champion_id.to_string();
    let mastery_url = url(
        &platform_base,
        &[
            "lol",
            "champion-mastery",
            "v4",
            "champion-masteries",
            "by-puuid",
            &puuid,
            "by-champion",
            &champion_id,
        ],
    )?;
    let ids_url = format!(
        "{}?start=0&count={}",
        url(
            &match_base,
            &["lol", "match", "v5", "matches", "by-puuid", &puuid, "ids"],
        )?,
        scouting::RECENT_MATCHES
    );
    let mastery_lookup = async {
        if target.champion_id == 0 {
            return Err(ScoutError::new("unknown champion"));
        }
        get(client, &mastery_url, api_key).await
    };
    let (ranked, mastery, ids) = tokio::join!(
        get(client, &ranked_url, api_key),
        mastery_lookup,
        get(client, &ids_url, api_key),
    );
    // A fatal answer (bad key) on any lookup stops the whole scout.
    for result in [&ranked, &ids] {
        if let Err(err) = result {
            if err.fatal {
                return Err(ScoutError {
                    message: err.message.clone(),
                    fatal: true,
                });
            }
        }
    }
    let ranked = ranked.ok().flatten().unwrap_or(Value::Array(Vec::new()));
    // 404 = never played the champion; any other failure = unknown.
    let mastery = match mastery {
        Ok(found) => Some(found.unwrap_or(Value::Null)),
        Err(_) => None,
    };
    let ids: Vec<String> = ids
        .ok()
        .flatten()
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();
    let match_base = match_base.as_str();
    let match_lookups = ids.iter().map(|id| async move {
        let match_url = url(match_base, &["lol", "match", "v5", "matches", id])?;
        get(client, &match_url, api_key).await
    });
    let matches: Vec<Value> = futures::future::join_all(match_lookups)
        .await
        .into_iter()
        .filter_map(|result| result.ok().flatten())
        .collect();

    Ok(scouting::build_card(
        target,
        &puuid,
        &ranked,
        mastery.as_ref(),
        &matches,
    ))
}
