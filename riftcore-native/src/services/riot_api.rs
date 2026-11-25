use reqwest::header::HeaderValue;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use zeroize::Zeroizing;

pub const DEFAULT_RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(60);
pub const MAX_RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(300);
pub const SUMMONER_CACHE_TTL: Duration = Duration::from_secs(300);
pub const MATCH_CACHE_TTL: Duration = Duration::from_secs(1800);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(4);
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccountCluster {
    Americas,
    Europe,
    Asia,
}

impl AccountCluster {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Americas => "americas",
            Self::Europe => "europe",
            Self::Asia => "asia",
        }
    }

    pub fn host(self) -> String {
        format!("{}.api.riotgames.com", self.as_str())
    }

    pub fn base_url(self) -> String {
        format!("https://{}", self.host())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MatchCluster {
    Americas,
    Europe,
    Asia,
    Sea,
}

impl MatchCluster {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Americas => "americas",
            Self::Europe => "europe",
            Self::Asia => "asia",
            Self::Sea => "sea",
        }
    }

    pub fn host(self) -> String {
        format!("{}.api.riotgames.com", self.as_str())
    }

    pub fn base_url(self) -> String {
        format!("https://{}", self.host())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RiotApiError {
    UnsupportedRegion(String),
    MissingApiKey,
    InvalidApiKeyHeader,
    Unauthorized,
    Forbidden,
    NotFound,
    RateLimited { retry_after: Duration },
    HttpStatus(u16),
    Transport(String),
    InvalidResponse(String),
    InvalidInput(String),
}

impl fmt::Display for RiotApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedRegion(region) => {
                write!(f, "Unsupported Riot platform region: {region}")
            }
            Self::MissingApiKey => {
                write!(f, "Add a Riot API key in Preferences before searching")
            }
            Self::InvalidApiKeyHeader => {
                write!(f, "Riot API key contains invalid header characters")
            }
            Self::Unauthorized => write!(f, "Riot API key is invalid or expired"),
            Self::Forbidden => write!(
                f,
                "Riot API denied this key or endpoint; check the key's permissions"
            ),
            Self::NotFound => write!(f, "Riot ID or League profile was not found in that region"),
            Self::RateLimited { retry_after } => write!(
                f,
                "Riot API rate limit reached; wait {}s and try again",
                retry_after.as_secs()
            ),
            Self::HttpStatus(code) => write!(f, "Riot API returned HTTP {code}"),
            Self::Transport(msg) => write!(f, "Could not reach Riot API: {msg}"),
            Self::InvalidResponse(msg) => write!(f, "Riot API returned unreadable data: {msg}"),
            Self::InvalidInput(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for RiotApiError {}

pub fn valid_platform(region: &str) -> Option<&'static str> {
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
        "me1" => Some("me1"),
        _ => None,
    }
}

/// Platform gateway host prefix for platform-v4 endpoints (`summoner-v4`, `league-v4`,
/// `champion-mastery-v4`, `status-v4`).
/// `me1` and `ph2` route through the `sg2` gateway.
pub fn platform_gateway(region: &str) -> Option<&'static str> {
    match valid_platform(region)? {
        "me1" | "ph2" => Some("sg2"),
        other => Some(other),
    }
}

/// Regional routing cluster for `account-v1`.
/// `oc1`, `sg2`, `vn2`, `tw2`, `me1`, and `ph2` all route to `asia.api.riotgames.com`
/// (account-v1 does not expose `oceania` or `sea` endpoints).
pub fn account_cluster(region: &str) -> Option<AccountCluster> {
    match valid_platform(region)? {
        "na1" | "br1" | "la1" | "la2" => Some(AccountCluster::Americas),
        "euw1" | "eun1" | "tr1" | "ru" => Some(AccountCluster::Europe),
        "kr" | "jp1" | "oc1" | "sg2" | "vn2" | "tw2" | "me1" | "ph2" => {
            Some(AccountCluster::Asia)
        }
        _ => None,
    }
}

/// Regional routing cluster for `match-v5`.
/// SEA/OCE/ME platforms (`oc1`, `sg2`, `vn2`, `tw2`, `me1`, `ph2`) route to `sea.api.riotgames.com`.
pub fn match_cluster(region: &str) -> Option<MatchCluster> {
    match valid_platform(region)? {
        "na1" | "br1" | "la1" | "la2" => Some(MatchCluster::Americas),
        "euw1" | "eun1" | "tr1" | "ru" => Some(MatchCluster::Europe),
        "kr" | "jp1" => Some(MatchCluster::Asia),
        "oc1" | "sg2" | "vn2" | "tw2" | "me1" | "ph2" => Some(MatchCluster::Sea),
        _ => None,
    }
}

pub fn platform_candidates_for_cluster(
    cluster: AccountCluster,
    preferred_region: &str,
) -> Vec<&'static str> {
    let preferred_gateway = platform_gateway(preferred_region).unwrap_or("eun1");
    let pool: &[&'static str] = match cluster {
        AccountCluster::Americas => &["na1", "br1", "la1", "la2"],
        AccountCluster::Europe => &["euw1", "eun1", "tr1", "ru"],
        AccountCluster::Asia => &["kr", "jp1", "oc1", "sg2", "vn2", "tw2"],
    };
    let mut out = Vec::with_capacity(pool.len());
    if let Some(&first) = pool.iter().find(|&&p| p == preferred_gateway) {
        out.push(first);
    }
    out.extend(pool.iter().copied().filter(|&p| p != preferred_gateway));
    out
}

pub fn build_api_key_header(api_key: &str) -> Result<HeaderValue, RiotApiError> {
    let trimmed = Zeroizing::new(api_key.trim().to_string());
    if trimmed.is_empty() {
        return Err(RiotApiError::MissingApiKey);
    }
    let mut value =
        HeaderValue::from_str(trimmed.as_str()).map_err(|_| RiotApiError::InvalidApiKeyHeader)?;
    value.set_sensitive(true);
    Ok(value)
}

pub fn parse_retry_after(header_value: Option<&str>, default_backoff: Duration) -> Duration {
    let parsed = header_value
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .filter(|&secs| secs > 0);
    match parsed {
        Some(secs) => Duration::from_secs(secs).min(MAX_RATE_LIMIT_BACKOFF),
        None => default_backoff,
    }
}

#[derive(Clone, Debug)]
struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

#[derive(Debug)]
pub struct TtlCache<V> {
    entries: HashMap<String, CacheEntry<V>>,
    max_entries: usize,
}

impl<V: Clone> TtlCache<V> {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries: max_entries.max(1),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn get(&mut self, key: &str) -> Option<V> {
        self.get_at(key, Instant::now())
    }

    pub fn get_at(&mut self, key: &str, now: Instant) -> Option<V> {
        let expired = self
            .entries
            .get(key)
            .is_some_and(|entry| now >= entry.expires_at);
        if expired {
            self.entries.remove(key);
            return None;
        }
        self.entries.get(key).map(|entry| entry.value.clone())
    }

    pub fn insert(&mut self, key: String, value: V, ttl: Duration) {
        self.insert_at(key, value, ttl, Instant::now());
    }

    pub fn insert_at(&mut self, key: String, value: V, ttl: Duration, now: Instant) {
        self.entries.retain(|_, entry| entry.expires_at > now);
        if !self.entries.contains_key(&key) && self.entries.len() >= self.max_entries {
            if let Some(oldest_key) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.expires_at)
                .map(|(k, _)| k.clone())
            {
                self.entries.remove(&oldest_key);
            }
        }
        let expires_at = now + ttl;
        self.entries.insert(key, CacheEntry { value, expires_at });
    }
}

#[derive(Clone, Debug)]
pub struct RiotApiConfig {
    pub default_retry_after: Duration,
    pub max_rate_limit_retries: usize,
    pub summoner_ttl: Duration,
    pub match_ttl: Duration,
}

impl Default for RiotApiConfig {
    fn default() -> Self {
        Self {
            default_retry_after: DEFAULT_RATE_LIMIT_BACKOFF,
            max_rate_limit_retries: 1,
            summoner_ttl: SUMMONER_CACHE_TTL,
            match_ttl: MATCH_CACHE_TTL,
        }
    }
}

#[derive(Clone)]
pub struct RiotApiClient {
    http: Client,
    config: RiotApiConfig,
    summoner_cache: Arc<Mutex<TtlCache<Value>>>,
    match_cache: Arc<Mutex<TtlCache<Value>>>,
    rate_limited_until: Arc<Mutex<Option<Instant>>>,
}

impl fmt::Debug for RiotApiClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RiotApiClient")
            .field("config", &self.config)
            .finish()
    }
}

impl RiotApiClient {
    pub fn build_http_client() -> Result<Client, RiotApiError> {
        Client::builder()
            .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
            .timeout(DEFAULT_REQUEST_TIMEOUT)
            .user_agent(concat!("RiftCore Companion/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|error| RiotApiError::Transport(error.to_string()))
    }

    pub fn new() -> Result<Self, RiotApiError> {
        Ok(Self::with_http_and_config(
            Self::build_http_client()?,
            RiotApiConfig::default(),
        ))
    }

    pub fn with_http_and_config(http: Client, config: RiotApiConfig) -> Self {
        Self {
            http,
            config,
            summoner_cache: Arc::new(Mutex::new(TtlCache::new(128))),
            match_cache: Arc::new(Mutex::new(TtlCache::new(256))),
            rate_limited_until: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn clear_caches(&self) {
        self.summoner_cache.lock().await.clear();
        self.match_cache.lock().await.clear();
    }

    async fn wait_for_rate_limit(&self) {
        let sleep_dur = {
            let mut guard = self.rate_limited_until.lock().await;
            match *guard {
                Some(until) => {
                    let now = Instant::now();
                    if until > now {
                        Some(until - now)
                    } else {
                        *guard = None;
                        None
                    }
                }
                None => None,
            }
        };
        if let Some(dur) = sleep_dur {
            tokio::time::sleep(dur).await;
        }
    }

    async fn record_rate_limit(&self, retry_after: Duration) {
        let mut guard = self.rate_limited_until.lock().await;
        let next = Instant::now() + retry_after;
        *guard = Some(match *guard {
            Some(existing) if existing > next => existing,
            _ => next,
        });
    }

    pub async fn get_json(&self, url: &str, api_key: &str) -> Result<Value, RiotApiError> {
        let token_header = build_api_key_header(api_key)?;
        let mut attempts = 0usize;

        loop {
            self.wait_for_rate_limit().await;

            let response = self
                .http
                .get(url)
                .header("X-Riot-Token", token_header.clone())
                .send()
                .await
                .map_err(|error| RiotApiError::Transport(error.to_string()))?;

            let status = response.status();
            if status.is_success() {
                return response
                    .json::<Value>()
                    .await
                    .map_err(|error| RiotApiError::InvalidResponse(error.to_string()));
            }

            if status == StatusCode::TOO_MANY_REQUESTS {
                let retry_header = response
                    .headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok());
                let wait = parse_retry_after(retry_header, self.config.default_retry_after);
                self.record_rate_limit(wait).await;

                if attempts < self.config.max_rate_limit_retries {
                    attempts += 1;
                    tokio::time::sleep(wait).await;
                    continue;
                }
                return Err(RiotApiError::RateLimited { retry_after: wait });
            }

            return Err(match status.as_u16() {
                401 => RiotApiError::Unauthorized,
                403 => RiotApiError::Forbidden,
                404 => RiotApiError::NotFound,
                code => RiotApiError::HttpStatus(code),
            });
        }
    }

    async fn get_cached_summoner(
        &self,
        cache_key: &str,
        url: &str,
        api_key: &str,
    ) -> Result<Value, RiotApiError> {
        if let Some(hit) = self.summoner_cache.lock().await.get(cache_key) {
            return Ok(hit);
        }
        let fresh = self.get_json(url, api_key).await?;
        self.summoner_cache.lock().await.insert(
            cache_key.to_string(),
            fresh.clone(),
            self.config.summoner_ttl,
        );
        Ok(fresh)
    }

    async fn get_cached_match(
        &self,
        cache_key: &str,
        url: &str,
        api_key: &str,
    ) -> Result<Value, RiotApiError> {
        if let Some(hit) = self.match_cache.lock().await.get(cache_key) {
            return Ok(hit);
        }
        let fresh = self.get_json(url, api_key).await?;
        self.match_cache.lock().await.insert(
            cache_key.to_string(),
            fresh.clone(),
            self.config.match_ttl,
        );
        Ok(fresh)
    }

    pub async fn validate_api_key(&self, region: &str, api_key: &str) -> Result<String, RiotApiError> {
        let Some(gateway) = platform_gateway(region) else {
            return Err(RiotApiError::UnsupportedRegion(region.to_string()));
        };
        let url = format!("https://{gateway}.api.riotgames.com/lol/status/v4/platform-data");
        self.get_json(&url, api_key).await?;
        Ok(format!(
            "Riot API key verified for {}",
            gateway.to_ascii_uppercase()
        ))
    }

    pub async fn lookup_account(
        &self,
        riot_id: &str,
        region: &str,
        api_key: &str,
    ) -> Result<Value, RiotApiError> {
        let Some(selected_platform) = valid_platform(region) else {
            return Err(RiotApiError::UnsupportedRegion(region.to_string()));
        };
        let Some(acct_cluster) = account_cluster(selected_platform) else {
            return Err(RiotApiError::UnsupportedRegion(region.to_string()));
        };
        if api_key.trim().is_empty() {
            return Err(RiotApiError::MissingApiKey);
        }

        let trimmed = riot_id.trim();
        let (game_name, tag_line) = trimmed
            .split_once('#')
            .unwrap_or((trimmed, selected_platform));
        if game_name.trim().is_empty() || tag_line.trim().is_empty() {
            return Err(RiotApiError::InvalidInput(
                "Enter a Riot ID in the form GameName#TAG".to_string(),
            ));
        }

        let account_url = riot_url(
            &acct_cluster.base_url(),
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
        let account_cache_key = format!(
            "acct:{}:{}#{}",
            acct_cluster.as_str(),
            game_name.trim().to_ascii_lowercase(),
            tag_line.trim().to_ascii_lowercase()
        );
        let account = self
            .get_cached_summoner(&account_cache_key, &account_url, api_key)
            .await?;

        let puuid = account
            .get("puuid")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                RiotApiError::InvalidResponse(
                    "Riot API returned an account without a PUUID".to_string(),
                )
            })?;

        let candidates = platform_candidates_for_cluster(acct_cluster, selected_platform);
        let mut effective_gateway = platform_gateway(selected_platform)
            .unwrap_or(selected_platform)
            .to_string();
        let mut summoner = None;
        let mut last_error = None;

        for candidate in candidates {
            let sum_url = riot_url(
                &format!("https://{candidate}.api.riotgames.com"),
                &["lol", "summoner", "v4", "summoners", "by-puuid", puuid],
            )?;
            let sum_cache_key = format!("sum:{candidate}:{puuid}");
            match self.get_cached_summoner(&sum_cache_key, &sum_url, api_key).await {
                Ok(data) => {
                    effective_gateway = candidate.to_string();
                    summoner = Some(data);
                    break;
                }
                Err(error) => {
                    let abort = matches!(
                        error,
                        RiotApiError::Unauthorized
                            | RiotApiError::Forbidden
                            | RiotApiError::RateLimited { .. }
                            | RiotApiError::MissingApiKey
                            | RiotApiError::InvalidApiKeyHeader
                    );
                    last_error = Some(error);
                    if abort {
                        break;
                    }
                }
            }
        }

        let summoner = summoner.ok_or_else(|| {
            last_error.unwrap_or(RiotApiError::NotFound)
        })?;

        let effective_match_cluster = match_cluster(&effective_gateway)
            .unwrap_or(MatchCluster::Europe);
        let platform_base = format!("https://{effective_gateway}.api.riotgames.com");
        let match_base = effective_match_cluster.base_url();

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
        let match_ids_url = format!(
            "{match_base}/lol/match/v5/matches/by-puuid/{puuid}/ids?start=0&count=5"
        );

        let ranked_key = format!("ranked:{effective_gateway}:{puuid}");
        let mastery_key = format!("mastery:{effective_gateway}:{puuid}");
        let ids_key = format!("mids:{}:{puuid}", effective_match_cluster.as_str());

        let (ranked_res, mastery_res, ids_res) = tokio::join!(
            self.get_cached_summoner(&ranked_key, &ranked_url, api_key),
            self.get_cached_summoner(&mastery_key, &mastery_url, api_key),
            self.get_cached_summoner(&ids_key, &match_ids_url, api_key),
        );

        for res in [&ranked_res, &mastery_res, &ids_res] {
            if let Err(err @ (RiotApiError::RateLimited { .. }
            | RiotApiError::Unauthorized
            | RiotApiError::Forbidden)) = res
            {
                return Err(err.clone());
            }
        }

        let ranked = ranked_res.unwrap_or_else(|_| Value::Array(Vec::new()));
        let masteries = mastery_res.unwrap_or_else(|_| Value::Array(Vec::new()));
        let match_ids = ids_res
            .ok()
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();

        // Fetch match payloads sequentially/boundedly rather than firing a 5-way burst,
        // respecting the match cache and 429 backoff.
        let mut matches = Vec::with_capacity(match_ids.len());
        for match_id in match_ids.iter().filter_map(Value::as_str) {
            let match_url = riot_url(
                &match_base,
                &["lol", "match", "v5", "matches", match_id],
            )?;
            let match_key = format!("match:{}:{match_id}", effective_match_cluster.as_str());
            match self.get_cached_match(&match_key, &match_url, api_key).await {
                Ok(m) => matches.push(m),
                Err(RiotApiError::RateLimited { retry_after }) => {
                    return Err(RiotApiError::RateLimited { retry_after });
                }
                Err(_) => {}
            }
        }

        Ok(serde_json::json!({
            "account": account,
            "summoner": summoner,
            "region": effective_gateway,
            "ranked": ranked,
            "masteries": masteries,
            "matches": matches,
        }))
    }
}

pub fn riot_url(base: &str, segments: &[&str]) -> Result<String, RiotApiError> {
    let mut url = reqwest::Url::parse(base)
        .map_err(|error| RiotApiError::InvalidInput(error.to_string()))?;
    url.path_segments_mut()
        .map_err(|_| RiotApiError::InvalidInput("Invalid Riot API base URL".to_string()))?
        .extend(segments.iter().copied());
    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn routing_account_cluster_all_regions() {
        // Americas
        for r in ["na1", "br1", "la1", "la2", "NA1"] {
            assert_eq!(account_cluster(r), Some(AccountCluster::Americas), "region {r}");
            assert_eq!(
                account_cluster(r).unwrap().host(),
                "americas.api.riotgames.com"
            );
        }
        // Europe
        for r in ["euw1", "eun1", "tr1", "ru", "EUW1"] {
            assert_eq!(account_cluster(r), Some(AccountCluster::Europe), "region {r}");
            assert_eq!(
                account_cluster(r).unwrap().host(),
                "europe.api.riotgames.com"
            );
        }
        // Asia + OCE + SEA + ME/PH -> MUST route to asia.api.riotgames.com for account-v1
        for r in ["kr", "jp1", "oc1", "sg2", "vn2", "tw2", "me1", "ph2"] {
            assert_eq!(account_cluster(r), Some(AccountCluster::Asia), "region {r}");
            assert_eq!(
                account_cluster(r).unwrap().host(),
                "asia.api.riotgames.com"
            );
        }
        assert_eq!(account_cluster("invalid"), None);
    }

    #[test]
    fn routing_match_cluster_all_regions() {
        for r in ["na1", "br1", "la1", "la2"] {
            assert_eq!(match_cluster(r), Some(MatchCluster::Americas), "region {r}");
            assert_eq!(
                match_cluster(r).unwrap().host(),
                "americas.api.riotgames.com"
            );
        }
        for r in ["euw1", "eun1", "tr1", "ru"] {
            assert_eq!(match_cluster(r), Some(MatchCluster::Europe), "region {r}");
            assert_eq!(
                match_cluster(r).unwrap().host(),
                "europe.api.riotgames.com"
            );
        }
        for r in ["kr", "jp1"] {
            assert_eq!(match_cluster(r), Some(MatchCluster::Asia), "region {r}");
            assert_eq!(match_cluster(r).unwrap().host(), "asia.api.riotgames.com");
        }
        for r in ["oc1", "sg2", "vn2", "tw2", "me1", "ph2"] {
            assert_eq!(match_cluster(r), Some(MatchCluster::Sea), "region {r}");
            assert_eq!(match_cluster(r).unwrap().host(), "sea.api.riotgames.com");
        }
        assert_eq!(match_cluster(""), None);
    }

    #[test]
    fn routing_platform_gateway_maps_me1_and_ph2_to_sg2() {
        assert_eq!(platform_gateway("me1"), Some("sg2"));
        assert_eq!(platform_gateway("ph2"), Some("sg2"));
        assert_eq!(platform_gateway(" PH2 "), Some("sg2"));
        assert_eq!(platform_gateway("oc1"), Some("oc1"));
        assert_eq!(platform_gateway("sg2"), Some("sg2"));
        assert_eq!(platform_gateway("eun1"), Some("eun1"));
    }

    #[test]
    fn api_key_header_is_marked_sensitive() {
        let header = build_api_key_header("  RGAPI-test-secret-1234  ").expect("valid header");
        assert!(header.is_sensitive());
        assert_eq!(header.to_str().unwrap(), "RGAPI-test-secret-1234");
        assert!(matches!(build_api_key_header("   "), Err(RiotApiError::MissingApiKey)));
    }

    #[test]
    fn ttl_cache_expires_and_evicts_entries() {
        let start = Instant::now();
        let mut cache = TtlCache::new(2);

        cache.insert_at("k1".to_string(), 10u32, Duration::from_secs(30), start);
        cache.insert_at("k2".to_string(), 20u32, Duration::from_secs(60), start);

        assert_eq!(cache.get_at("k1", start + Duration::from_secs(29)), Some(10));
        // Exactly at expires_at -> expired
        assert_eq!(cache.get_at("k1", start + Duration::from_secs(30)), None);
        assert_eq!(cache.get_at("k2", start + Duration::from_secs(30)), Some(20));

        // Capacity eviction removes earliest-expiring entry
        cache.clear();
        cache.insert_at("a".to_string(), 1, Duration::from_secs(10), start);
        cache.insert_at("b".to_string(), 2, Duration::from_secs(50), start);
        cache.insert_at("c".to_string(), 3, Duration::from_secs(40), start);
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get_at("a", start), None);
        assert_eq!(cache.get_at("b", start), Some(2));
        assert_eq!(cache.get_at("c", start), Some(3));
    }

    #[test]
    fn parse_retry_after_uses_header_or_60s_default() {
        assert_eq!(
            parse_retry_after(Some("15"), DEFAULT_RATE_LIMIT_BACKOFF),
            Duration::from_secs(15)
        );
        assert_eq!(
            parse_retry_after(None, DEFAULT_RATE_LIMIT_BACKOFF),
            Duration::from_secs(60)
        );
        assert_eq!(
            parse_retry_after(Some("invalid"), DEFAULT_RATE_LIMIT_BACKOFF),
            Duration::from_secs(60)
        );
        assert_eq!(
            parse_retry_after(Some("0"), DEFAULT_RATE_LIMIT_BACKOFF),
            Duration::from_secs(60)
        );
    }

    #[test]
    fn rate_limit_429_waits_retry_after_and_retries() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let addr = listener.local_addr().unwrap();
        let hits = Arc::new(AtomicUsize::new(0));
        let hits_srv = Arc::clone(&hits);

        let handle = std::thread::spawn(move || {
            for idx in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf);
                hits_srv.fetch_add(1, Ordering::SeqCst);
                if idx == 0 {
                    let resp = "HTTP/1.1 429 Too Many Requests\r\nRetry-After: 1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                    stream.write_all(resp.as_bytes()).unwrap();
                } else {
                    let body = r#"{"ok":true}"#;
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    stream.write_all(resp.as_bytes()).unwrap();
                }
            }
        });

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let client = RiotApiClient::with_http_and_config(
            RiotApiClient::build_http_client().unwrap(),
            RiotApiConfig {
                default_retry_after: Duration::from_millis(50),
                max_rate_limit_retries: 1,
                ..Default::default()
            },
        );
        let url = format!("http://{addr}/lol/status/v4/platform-data");
        let result = rt.block_on(client.get_json(&url, "RGAPI-test-key")).unwrap();
        assert_eq!(result["ok"], Value::Bool(true));
        assert_eq!(hits.load(Ordering::SeqCst), 2);
        handle.join().unwrap();
    }
}
