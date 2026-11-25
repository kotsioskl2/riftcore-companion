use crate::platform::security::verify_authenticode;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

pub const DEFAULT_MANIFEST_URL: &str =
    "https://api.github.com/repos/kotsioskl2/riftcore-companion/releases/latest";
pub const RELEASES_FALLBACK_URL: &str =
    "https://github.com/kotsioskl2/riftcore-companion/releases/latest";
const ALLOWED_DOWNLOAD_PREFIX: &str = "/kotsioskl2/riftcore-companion/releases/download/";
const BINARY_ASSET_NAME: &str = "RiftCore.exe";
const CHECKSUMS_ASSET_NAME: &str = "SHA256SUMS.txt";
const MIN_BINARY_BYTES: usize = 1_000_000;
const MAX_BINARY_BYTES: usize = 250_000_000;
const DEFAULT_API_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdaterError {
    InsecureUrl(String),
    NotNewerVersion,
    Network(String),
    InvalidManifest(String),
    MissingChecksumManifest,
    MissingBinaryAsset,
    ChecksumMismatch { expected: String, actual: String },
    AuthenticodeFailed(String),
    SizeOutOfBounds(usize),
    Io(String),
    UnsupportedPlatform,
}

impl fmt::Display for UpdaterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsecureUrl(msg) => write!(f, "Rejected update URL: {msg}"),
            Self::NotNewerVersion => {
                write!(f, "This release is not newer than the installed version")
            }
            Self::Network(msg) => write!(f, "{msg}"),
            Self::InvalidManifest(msg) => write!(f, "Unreadable release manifest: {msg}"),
            Self::MissingChecksumManifest => write!(
                f,
                "Release is missing {CHECKSUMS_ASSET_NAME}; automatic update aborted"
            ),
            Self::MissingBinaryAsset => write!(
                f,
                "This release has no {BINARY_ASSET_NAME} package; open the release page to download"
            ),
            Self::ChecksumMismatch { expected, actual } => write!(
                f,
                "SHA-256 verification failed for update package (expected {expected}, got {actual})"
            ),
            Self::AuthenticodeFailed(msg) => {
                write!(f, "Authenticode verification failed: {msg}")
            }
            Self::SizeOutOfBounds(bytes) => {
                write!(f, "Update package has an unexpected size ({bytes} bytes)")
            }
            Self::Io(msg) => write!(f, "Update staging error: {msg}"),
            Self::UnsupportedPlatform => {
                write!(f, "Automatic app updates are supported on Windows only")
            }
        }
    }
}

impl std::error::Error for UpdaterError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateCheckResult {
    pub release: Value,
    pub latest_tag: String,
    pub is_newer: bool,
    pub has_binary_asset: bool,
    pub has_checksum_asset: bool,
    pub release_page_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StagedUpdate {
    pub version_tag: String,
    pub staged_path: PathBuf,
    pub sha256_hex: String,
}

#[derive(Clone)]
pub struct UpdaterService {
    http: Client,
    manifest_url: String,
}

impl fmt::Debug for UpdaterService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UpdaterService")
            .field("manifest_url", &self.manifest_url)
            .finish()
    }
}

impl UpdaterService {
    pub fn build_http_client() -> Result<Client, UpdaterError> {
        Client::builder()
            .connect_timeout(Duration::from_secs(4))
            .timeout(DEFAULT_API_TIMEOUT)
            .user_agent(concat!("RiftCore Companion/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|error| UpdaterError::Network(error.to_string()))
    }

    pub fn new() -> Result<Self, UpdaterError> {
        Self::with_manifest_url(DEFAULT_MANIFEST_URL)
    }

    pub fn with_manifest_url(manifest_url: &str) -> Result<Self, UpdaterError> {
        validate_https_url(manifest_url)?;
        Ok(Self {
            http: Self::build_http_client()?,
            manifest_url: manifest_url.to_string(),
        })
    }

    pub async fn check_for_updates(&self) -> Result<UpdateCheckResult, UpdaterError> {
        validate_https_url(&self.manifest_url)?;
        let release = self
            .http
            .get(&self.manifest_url)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| UpdaterError::Network(format!("Could not reach GitHub Releases: {e}")))?
            .error_for_status()
            .map_err(|e| UpdaterError::Network(format!("GitHub Releases returned an error: {e}")))?
            .json::<Value>()
            .await
            .map_err(|e| UpdaterError::InvalidManifest(e.to_string()))?;

        let latest_tag = release
            .get("tag_name")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let current = env!("CARGO_PKG_VERSION");
        let is_newer = is_newer_version(&latest_tag, current);
        let has_binary_asset = release_asset_url(&release, BINARY_ASSET_NAME).is_some();
        let has_checksum_asset = release_asset_url(&release, CHECKSUMS_ASSET_NAME).is_some();
        let release_page_url = release
            .get("html_url")
            .and_then(Value::as_str)
            .filter(|url| validate_https_url(url).is_ok())
            .unwrap_or(RELEASES_FALLBACK_URL)
            .to_string();

        Ok(UpdateCheckResult {
            release,
            latest_tag,
            is_newer,
            has_binary_asset,
            has_checksum_asset,
            release_page_url,
        })
    }

    /// Downloads `SHA256SUMS.txt` and `RiftCore.exe` over HTTPS, streams the binary into
    /// `%LOCALAPPDATA%\RiftCore\updates\`, verifies SHA-256 via `ring::digest::SHA256` and
    /// Authenticode via `platform::security::verify_authenticode`.
    /// On verification failure, removes staged artifacts and falls back to opening the
    /// release page in the user's browser.
    pub async fn download_and_verify(&self, release: &Value) -> Result<StagedUpdate, UpdaterError> {
        let release_page_url = release
            .get("html_url")
            .and_then(Value::as_str)
            .filter(|url| validate_https_url(url).is_ok())
            .unwrap_or(RELEASES_FALLBACK_URL)
            .to_string();

        let result = self.download_and_verify_inner(release).await;
        if let Err(ref err) = result {
            if matches!(
                err,
                UpdaterError::MissingChecksumManifest
                    | UpdaterError::MissingBinaryAsset
                    | UpdaterError::ChecksumMismatch { .. }
                    | UpdaterError::AuthenticodeFailed(_)
                    | UpdaterError::InsecureUrl(_)
                    | UpdaterError::SizeOutOfBounds(_)
            ) {
                let _ = open_releases_in_browser(&release_page_url);
            }
        }
        result
    }

    async fn download_and_verify_inner(
        &self,
        release: &Value,
    ) -> Result<StagedUpdate, UpdaterError> {
        let latest = release
            .get("tag_name")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                UpdaterError::InvalidManifest(
                    "GitHub release is missing its version tag".to_string(),
                )
            })?;
        if !is_newer_version(latest, env!("CARGO_PKG_VERSION")) {
            return Err(UpdaterError::NotNewerVersion);
        }

        let binary_url_raw = release_asset_url(release, BINARY_ASSET_NAME)
            .ok_or(UpdaterError::MissingBinaryAsset)?;
        let binary_url = validate_release_asset_url(binary_url_raw)?;

        let sums_url_raw = release_asset_url(release, CHECKSUMS_ASSET_NAME)
            .ok_or(UpdaterError::MissingChecksumManifest)?;
        let sums_url = validate_release_asset_url(sums_url_raw)?;

        #[cfg(not(windows))]
        {
            let _ = (binary_url, sums_url);
            return Err(UpdaterError::UnsupportedPlatform);
        }

        #[cfg(windows)]
        {
            let sums_text = self
                .http
                .get(sums_url)
                .send()
                .await
                .map_err(|e| {
                    UpdaterError::Network(format!("Could not download {CHECKSUMS_ASSET_NAME}: {e}"))
                })?
                .error_for_status()
                .map_err(|e| {
                    UpdaterError::Network(format!("GitHub rejected {CHECKSUMS_ASSET_NAME}: {e}"))
                })?
                .text()
                .await
                .map_err(|e| {
                    UpdaterError::Network(format!("Failed reading {CHECKSUMS_ASSET_NAME}: {e}"))
                })?;

            let checksums = parse_sha256sums(&sums_text);
            let expected_hex = checksums
                .get(&BINARY_ASSET_NAME.to_ascii_lowercase())
                .cloned()
                .ok_or(UpdaterError::MissingChecksumManifest)?;

            let staging_dir = updates_staging_dir()?;
            std::fs::create_dir_all(&staging_dir).map_err(|e| {
                UpdaterError::Io(format!("Could not create staging directory: {e}"))
            })?;

            let safe_tag: String = latest
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                        c
                    } else {
                        '_'
                    }
                })
                .collect();
            let part_path =
                staging_dir.join(format!("RiftCore-{safe_tag}-{}.part", std::process::id()));
            let final_path =
                staging_dir.join(format!("RiftCore-{safe_tag}-{}.exe", std::process::id()));

            let mut response = self
                .http
                .get(binary_url)
                .timeout(DEFAULT_DOWNLOAD_TIMEOUT)
                .send()
                .await
                .map_err(|e| UpdaterError::Network(format!("Could not download the update: {e}")))?
                .error_for_status()
                .map_err(|e| {
                    UpdaterError::Network(format!("GitHub rejected the update download: {e}"))
                })?;

            if let Some(len) = response.content_length() {
                let len_usize = usize::try_from(len).unwrap_or(usize::MAX);
                if !(MIN_BINARY_BYTES..=MAX_BINARY_BYTES).contains(&len_usize) {
                    return Err(UpdaterError::SizeOutOfBounds(len_usize));
                }
            }

            let mut file = std::fs::File::create(&part_path)
                .map_err(|e| UpdaterError::Io(format!("Could not create staged file: {e}")))?;
            let mut hasher = ring::digest::Context::new(&ring::digest::SHA256);
            let mut total_bytes: usize = 0;

            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|e| UpdaterError::Network(format!("Download interrupted: {e}")))?
            {
                total_bytes = total_bytes.saturating_add(chunk.len());
                if total_bytes > MAX_BINARY_BYTES {
                    drop(file);
                    let _ = std::fs::remove_file(&part_path);
                    return Err(UpdaterError::SizeOutOfBounds(total_bytes));
                }
                hasher.update(&chunk);
                if let Err(e) = file.write_all(&chunk) {
                    drop(file);
                    let _ = std::fs::remove_file(&part_path);
                    return Err(UpdaterError::Io(format!("Failed writing staged file: {e}")));
                }
            }

            if !(MIN_BINARY_BYTES..=MAX_BINARY_BYTES).contains(&total_bytes) {
                drop(file);
                let _ = std::fs::remove_file(&part_path);
                return Err(UpdaterError::SizeOutOfBounds(total_bytes));
            }
            if let Err(e) = file.flush() {
                drop(file);
                let _ = std::fs::remove_file(&part_path);
                return Err(UpdaterError::Io(format!(
                    "Failed flushing staged file: {e}"
                )));
            }
            drop(file);

            let actual_digest = hasher.finish();
            let actual_hex = encode_hex(actual_digest.as_ref());
            if !verify_sha256_digest_bytes(actual_digest.as_ref(), &expected_hex) {
                let _ = std::fs::remove_file(&part_path);
                return Err(UpdaterError::ChecksumMismatch {
                    expected: expected_hex,
                    actual: actual_hex,
                });
            }

            if final_path.exists() {
                let _ = std::fs::remove_file(&final_path);
            }
            if let Err(e) = std::fs::rename(&part_path, &final_path) {
                let _ = std::fs::remove_file(&part_path);
                return Err(UpdaterError::Io(format!(
                    "Could not finalize staged EXE: {e}"
                )));
            }

            if let Err(auth_err) = verify_authenticode(&final_path) {
                let _ = std::fs::remove_file(&final_path);
                return Err(UpdaterError::AuthenticodeFailed(auth_err));
            }

            Ok(StagedUpdate {
                version_tag: latest.to_string(),
                staged_path: final_path,
                sha256_hex: actual_hex,
            })
        }
    }
}

pub fn validate_https_url(raw: &str) -> Result<reqwest::Url, UpdaterError> {
    let url = reqwest::Url::parse(raw.trim())
        .map_err(|_| UpdaterError::InsecureUrl("invalid URL".to_string()))?;
    if url.scheme() != "https" {
        return Err(UpdaterError::InsecureUrl(
            "only HTTPS update URLs are permitted".to_string(),
        ));
    }
    if url.host_str().is_none() {
        return Err(UpdaterError::InsecureUrl("missing host".to_string()));
    }
    Ok(url)
}

pub fn validate_release_asset_url(raw: &str) -> Result<reqwest::Url, UpdaterError> {
    let url = validate_https_url(raw)?;
    if url.host_str() != Some("github.com") || !url.path().starts_with(ALLOWED_DOWNLOAD_PREFIX) {
        return Err(UpdaterError::InsecureUrl(
            "unexpected GitHub release download host or path".to_string(),
        ));
    }
    Ok(url)
}

pub fn release_asset_url<'a>(release: &'a Value, name: &str) -> Option<&'a str> {
    release
        .get("assets")?
        .as_array()?
        .iter()
        .find(|asset| {
            asset
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|n| n.eq_ignore_ascii_case(name))
        })?
        .get("browser_download_url")?
        .as_str()
}

/// Parses a standard `sha256sum` / `SHA256SUMS.txt` file into lowercase `filename -> sha256_hex`.
pub fn parse_sha256sums(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.splitn(2, |c: char| c.is_ascii_whitespace());
        let Some(hash_raw) = parts.next() else {
            continue;
        };
        let Some(rest) = parts.next() else {
            continue;
        };
        let hash = hash_raw.trim().to_ascii_lowercase();
        if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            continue;
        }
        let filename = rest
            .trim()
            .trim_start_matches('*')
            .trim_start_matches("./")
            .trim_start_matches(".\\");
        if let Some(base) = Path::new(filename).file_name().and_then(|s| s.to_str()) {
            map.insert(base.to_ascii_lowercase(), hash);
        }
    }
    map
}

pub fn verify_sha256_bytes_with_ring(data: &[u8], expected_hex: &str) -> bool {
    let digest = ring::digest::digest(&ring::digest::SHA256, data);
    verify_sha256_digest_bytes(digest.as_ref(), expected_hex)
}

fn verify_sha256_digest_bytes(digest_bytes: &[u8], expected_hex: &str) -> bool {
    let Some(expected_bytes) = decode_sha256_hex(expected_hex) else {
        return false;
    };
    if digest_bytes.len() != 32 {
        return false;
    }
    let mut diff = 0u8;
    for (&a, &b) in digest_bytes.iter().zip(expected_bytes.iter()) {
        diff |= a ^ b;
    }
    diff == 0
}

fn decode_sha256_hex(hex: &str) -> Option<[u8; 32]> {
    let trimmed = hex.trim();
    if trimmed.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (idx, chunk) in trimmed.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let hi = hex_nibble(chunk[0])?;
        let lo = hex_nibble(chunk[1])?;
        out[idx] = (hi << 4) | lo;
    }
    Some(out)
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(LUT[(b >> 4) as usize] as char);
        out.push(LUT[(b & 0x0f) as usize] as char);
    }
    out
}

/// Resolves the update staging directory under `%LOCALAPPDATA%\RiftCore\updates`
/// (never staging onto `F:`).
pub fn updates_staging_dir() -> Result<PathBuf, UpdaterError> {
    let env_local = std::env::var("LOCALAPPDATA")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let env_profile = std::env::var("USERPROFILE")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    resolve_staging_dir_from(env_local.as_deref(), env_profile.as_deref())
}

pub fn resolve_staging_dir_from(
    local_app_data: Option<&str>,
    user_profile: Option<&str>,
) -> Result<PathBuf, UpdaterError> {
    if let Some(local) = local_app_data.map(str::trim).filter(|s| !s.is_empty()) {
        return Ok(PathBuf::from(local).join("RiftCore").join("updates"));
    }
    if let Some(profile) = user_profile.map(str::trim).filter(|s| !s.is_empty()) {
        return Ok(PathBuf::from(profile)
            .join("AppData")
            .join("Local")
            .join("RiftCore")
            .join("updates"));
    }
    let fallback = std::env::temp_dir().join("RiftCore").join("updates");
    let first_component = fallback.to_string_lossy();
    if first_component.to_ascii_uppercase().starts_with("F:") {
        return Err(UpdaterError::Io(
            "Refusing to stage updates onto F: drive when LOCALAPPDATA is unset".to_string(),
        ));
    }
    Ok(fallback)
}

pub fn open_releases_in_browser(url: &str) -> Result<(), UpdaterError> {
    let parsed = validate_https_url(url)?;
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        Command::new("rundll32.exe")
            .args(["url.dll,FileProtocolHandler", parsed.as_str()])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| UpdaterError::Io(format!("Could not open browser: {e}")))?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = parsed;
        Err(UpdaterError::UnsupportedPlatform)
    }
}

#[cfg(windows)]
pub fn spawn_self_update_helper(staged_exe: &Path) -> Result<(), UpdaterError> {
    use std::os::windows::process::CommandExt;

    if !staged_exe.is_file() {
        return Err(UpdaterError::Io(
            "Staged update executable does not exist".to_string(),
        ));
    }
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let executable = std::env::current_exe()
        .map_err(|e| UpdaterError::Io(format!("Could not locate the running app: {e}")))?;
    let staging_dir = updates_staging_dir()?;
    std::fs::create_dir_all(&staging_dir)
        .map_err(|e| UpdaterError::Io(format!("Could not prepare staging directory: {e}")))?;

    let stem = format!("riftcore-apply-{}", std::process::id());
    let helper = staging_dir.join(format!("{stem}.ps1"));
    let log_path = staging_dir.join(format!("{stem}.log"));

    let quote_ps = |path: &Path| path.to_string_lossy().replace('\'', "''");
    let script = format!(
        "$ErrorActionPreference = 'Stop'\n$waitForPid = {}\n$source = '{}'\n$target = '{}'\n$backup = $target + '.old'\n$logPath = '{}'\n$updated = $false\ntry {{\n  Wait-Process -Id $waitForPid -ErrorAction SilentlyContinue\n  Start-Sleep -Milliseconds 400\n  if (Test-Path -LiteralPath $backup) {{ Remove-Item -LiteralPath $backup -Force }}\n  if (Test-Path -LiteralPath $target) {{ Move-Item -LiteralPath $target -Destination $backup -Force }}\n  Move-Item -LiteralPath $source -Destination $target -Force\n  Start-Process -FilePath $target\n  $updated = $true\n}} catch {{\n  $_ | Out-File -LiteralPath $logPath -Encoding utf8 -Force\n}}\nif (-not $updated) {{\n  if (Test-Path -LiteralPath $backup) {{\n    if (Test-Path -LiteralPath $target) {{ Remove-Item -LiteralPath $target -Force }}\n    Move-Item -LiteralPath $backup -Destination $target -Force\n  }}\n  if (Test-Path -LiteralPath $target) {{ Start-Process -FilePath $target }}\n}} elseif (Test-Path -LiteralPath $backup) {{\n  Remove-Item -LiteralPath $backup -Force -ErrorAction SilentlyContinue\n}}\nRemove-Item -LiteralPath $MyInvocation.MyCommand.Path -Force -ErrorAction SilentlyContinue\n",
        std::process::id(),
        quote_ps(staged_exe),
        quote_ps(&executable),
        quote_ps(&log_path),
    );
    std::fs::write(&helper, script)
        .map_err(|e| UpdaterError::Io(format!("Could not write update helper: {e}")))?;

    if let Err(err) = Command::new("powershell.exe")
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
        .spawn()
    {
        let _ = std::fs::remove_file(&helper);
        return Err(UpdaterError::Io(format!(
            "Could not start update helper: {err}"
        )));
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn spawn_self_update_helper(_staged_exe: &Path) -> Result<(), UpdaterError> {
    Err(UpdaterError::UnsupportedPlatform)
}

pub fn is_newer_version(latest: &str, current: &str) -> bool {
    #[derive(Eq, PartialEq, Ord, PartialOrd)]
    struct SemVerKey {
        major: u64,
        minor: u64,
        patch: u64,
        is_stable: bool,
        pre: String,
    }

    let parse = |raw: &str| {
        let stripped = raw.trim().trim_start_matches(['v', 'V']);
        let (core, pre) = stripped
            .split_once('-')
            .map(|(c, p)| (c, p.split('+').next().unwrap_or_default()))
            .unwrap_or_else(|| (stripped.split('+').next().unwrap_or_default(), ""));
        let mut nums = core.split('.').map(|part| part.parse::<u64>().unwrap_or(0));
        SemVerKey {
            major: nums.next().unwrap_or(0),
            minor: nums.next().unwrap_or(0),
            patch: nums.next().unwrap_or(0),
            is_stable: pre.is_empty(),
            pre: pre.to_ascii_lowercase(),
        }
    };

    parse(latest) > parse(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::security::sha256_hex;

    #[test]
    fn sha256_ring_verification_accepts_valid_and_rejects_mismatch() {
        let payload = b"riftcore-signed-binary-contents";
        let digest_hex = sha256_hex(payload);

        assert!(verify_sha256_bytes_with_ring(payload, &digest_hex));
        assert!(verify_sha256_bytes_with_ring(
            payload,
            &digest_hex.to_ascii_uppercase()
        ));
        // Fail-closed on tampered payload, wrong hex, or malformed hex
        assert!(!verify_sha256_bytes_with_ring(b"tampered", &digest_hex));
        assert!(!verify_sha256_bytes_with_ring(payload, &"0".repeat(64)));
        assert!(!verify_sha256_bytes_with_ring(payload, "not-a-sha256"));
    }

    #[test]
    fn parses_sha256sums_manifest_strictly() {
        let valid_hex = "a".repeat(64);
        let other_hex = "b".repeat(64);
        let sums = format!(
            "# release checksums\n{other_hex}  RiftCore-Setup-2.2.4.exe\n{valid_hex} *./RiftCore.exe\nbadhex RiftCore.exe\n"
        );
        let parsed = parse_sha256sums(&sums);
        assert_eq!(parsed.get("riftcore.exe"), Some(&valid_hex));
        assert_eq!(parsed.get("riftcore-setup-2.2.4.exe"), Some(&other_hex));
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn rejects_non_https_and_untrusted_release_urls() {
        assert!(validate_https_url("http://api.github.com/repos/x/y").is_err());
        assert!(validate_https_url("file:///C:/temp/RiftCore.exe").is_err());
        assert!(validate_https_url("https://api.github.com/repos/x/y").is_ok());

        assert!(validate_release_asset_url(
            "https://github.com/kotsioskl2/riftcore-companion/releases/download/v2.2.4/RiftCore.exe"
        )
        .is_ok());
        assert!(validate_release_asset_url(
            "https://evil.example.com/kotsioskl2/riftcore-companion/releases/download/v2.2.4/RiftCore.exe"
        )
        .is_err());
        assert!(validate_release_asset_url(
            "http://github.com/kotsioskl2/riftcore-companion/releases/download/v2.2.4/RiftCore.exe"
        )
        .is_err());
    }

    #[test]
    fn staging_dir_uses_localappdata_never_f_drive() {
        let dir = resolve_staging_dir_from(Some("C:\\Users\\Pilot\\AppData\\Local"), None)
            .expect("valid LOCALAPPDATA staging path");
        assert_eq!(
            dir,
            PathBuf::from("C:\\Users\\Pilot\\AppData\\Local\\RiftCore\\updates")
        );
        assert!(!dir.to_string_lossy().to_ascii_uppercase().starts_with("F:"));
    }

    #[test]
    fn semver_ordering_handles_prereleases_and_missing_patch() {
        assert!(is_newer_version("v2.2.4", "2.2.3"));
        assert!(!is_newer_version("v2.2.3", "2.2.3"));
        assert!(!is_newer_version("v2.2.3-rc.1", "2.2.3"));
        assert!(is_newer_version("2.2.3", "2.2.3-rc.1"));
        assert!(!is_newer_version("2.2", "2.2.0"));
    }

    #[test]
    fn authenticode_fails_closed_on_unsigned_staged_exe() {
        let tmp =
            std::env::temp_dir().join(format!("riftcore-updater-test-{}.exe", std::process::id()));
        std::fs::write(&tmp, b"MZfake-unsigned-binary").unwrap();
        let res = verify_authenticode(&tmp);
        let _ = std::fs::remove_file(&tmp);
        assert!(res.is_err());
    }

    #[test]
    fn missing_sha256sums_asset_fails_closed() {
        let service = UpdaterService::new().unwrap();
        let release = serde_json::json!({
            "tag_name": "v99.0.0",
            "html_url": "https://github.com/kotsioskl2/riftcore-companion/releases/tag/v99.0.0",
            "assets": [{
                "name": "RiftCore.exe",
                "browser_download_url": "https://github.com/kotsioskl2/riftcore-companion/releases/download/v99.0.0/RiftCore.exe"
            }]
        });
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let err = rt
            .block_on(service.download_and_verify_inner(&release))
            .expect_err("missing SHA256SUMS.txt must fail closed");
        assert_eq!(err, UpdaterError::MissingChecksumManifest);
    }
}
