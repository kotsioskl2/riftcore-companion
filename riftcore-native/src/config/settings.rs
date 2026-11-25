use crate::platform::security::{
    is_dpapi_protected_value, protect_secret_string, unprotect_secret_string,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use zeroize::Zeroize;

static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetupConflictError {
    AlreadyExists(u32),
}

impl fmt::Display for SetupConflictError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyExists(champion_id) => {
                write!(
                    f,
                    "Champion setup already exists for champion {champion_id}"
                )
            }
        }
    }
}

impl std::error::Error for SetupConflictError {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ChampionSetup {
    pub role: String,
    pub primary_tree: u32,
    pub secondary_tree: u32,
    pub perks: [i64; 9],
    pub secondary_rows: [usize; 2],
    pub spell_one: i64,
    pub spell_two: i64,
}

impl Default for ChampionSetup {
    fn default() -> Self {
        Self {
            role: "MID".to_string(),
            primary_tree: 8100,
            secondary_tree: 8000,
            perks: [8112, 8126, 8140, 8135, 9111, 8014, 5008, 5008, 5001],
            secondary_rows: [2, 3],
            spell_one: 4,
            spell_two: 12,
        }
    }
}

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
    /// Held in memory for active Riot API requests and UI binding; wiped on `Drop`.
    /// Never written to disk in plaintext (persisted exclusively via DPAPI ciphertext).
    pub riot_api_key: String,
    pub hud_gold_lead: bool,
    pub hud_cs_benchmark: bool,
    pub hud_objective_timers: bool,
    pub hud_ally_ult_timers: bool,
    pub hud_enemy_spell_timers: bool,
    pub hud_manual_camp_timers: bool,
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
    pub hud_preview_mode: bool,
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
            hud_enemy_spell_timers: true,
            hud_manual_camp_timers: true,
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
            hud_preview_mode: false,
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

impl AppSettings {
    pub fn set_riot_api_key(&mut self, new_key: String) {
        self.riot_api_key.zeroize();
        self.riot_api_key = new_key;
    }

    pub fn clear_riot_api_key(&mut self) {
        self.riot_api_key.zeroize();
    }

    fn normalize(&mut self) {
        // Desktop preview is a single-session toggle and must not persist across launches.
        self.hud_preview_mode = false;
        self.tft_board.resize(28, String::new());
        self.tft_board.truncate(28);
    }
}

/// Manages persistent application configuration and champion setups in atomic TOML format
/// with DPAPI encryption for the Riot API key and safe recovery from malformed files.
pub struct SettingsStore {
    path: PathBuf,
    pub settings: AppSettings,
    pub champion_setups: BTreeMap<u32, ChampionSetup>,
    pub warnings: Vec<String>,
}

impl Drop for SettingsStore {
    fn drop(&mut self) {
        self.settings.clear_riot_api_key();
    }
}

impl SettingsStore {
    /// Resolves the canonical user config file (`%APPDATA%\RiftCore\settings.toml` on Windows),
    /// refusing to fall back to `current_dir()` so secrets cannot leak to shared directories.
    pub fn default_path() -> Result<PathBuf, String> {
        for var in ["APPDATA", "LOCALAPPDATA", "USERPROFILE", "HOME"] {
            if let Ok(val) = std::env::var(var) {
                let trimmed = val.trim();
                if !trimmed.is_empty() {
                    let base = PathBuf::from(trimmed);
                    let dir = if var == "USERPROFILE" || var == "HOME" {
                        base.join(".config").join("RiftCore")
                    } else {
                        base.join("RiftCore")
                    };
                    return Ok(dir.join("settings.toml"));
                }
            }
        }
        Err("Unable to resolve a private user configuration directory".to_string())
    }

    /// Loads settings from the default user configuration path, recovering gracefully on errors.
    pub fn load_default() -> Self {
        match Self::default_path() {
            Ok(path) => Self::load(path),
            Err(err) => {
                let mut settings = AppSettings::default();
                settings.normalize();
                Self {
                    path: PathBuf::new(),
                    settings,
                    champion_setups: BTreeMap::new(),
                    warnings: vec![err],
                }
            }
        }
    }

    /// Loads `settings.toml` from `path`, migrating legacy `settings.json` if present, or
    /// backing up corrupt configuration files and restoring safe defaults.
    pub fn load(path: PathBuf) -> Self {
        let mut store = Self {
            path,
            settings: AppSettings::default(),
            champion_setups: BTreeMap::new(),
            warnings: Vec::new(),
        };

        if store.path.as_os_str().is_empty() {
            store.settings.normalize();
            return store;
        }

        if store.path.exists() {
            match fs::read_to_string(&store.path) {
                Ok(raw) => {
                    let mut migrated_plaintext = false;
                    match parse_settings_toml(&raw, &mut store.warnings) {
                        Ok((mut parsed, setups, had_plaintext_key)) => {
                            parsed.normalize();
                            store.settings = parsed;
                            store.champion_setups = setups;
                            migrated_plaintext = had_plaintext_key;
                        }
                        Err(err) => {
                            let backup = backup_corrupt_file(&store.path);
                            store.warnings.push(match backup {
                                Some(b) => format!(
                                    "Recovered default settings after parse error ({err}); corrupt file moved to {}",
                                    b.display()
                                ),
                                None => format!(
                                    "Recovered default settings after parse error ({err})"
                                ),
                            });
                            store.settings = AppSettings::default();
                            store.settings.normalize();
                        }
                    }
                    if migrated_plaintext {
                        if let Err(err) = store.save() {
                            store.warnings.push(format!(
                                "Loaded legacy plaintext key into memory, but failed to re-encrypt on disk: {err}"
                            ));
                        }
                    }
                }
                Err(err) => {
                    store
                        .warnings
                        .push(format!("Could not read {}: {err}", store.path.display()));
                    store.settings.normalize();
                }
            }
            return store;
        }

        // Migrate from legacy JSON if present in the same directory.
        let legacy_json = store.path.with_file_name("settings.json");
        if legacy_json.exists() {
            match fs::read(&legacy_json) {
                Ok(bytes) => match serde_json::from_slice::<AppSettings>(&bytes) {
                    Ok(mut legacy) => {
                        legacy.normalize();
                        store.settings = legacy;
                        if let Err(err) = store.save() {
                            store.warnings.push(format!(
                                "Migrated legacy settings into memory, but could not save TOML: {err}"
                            ));
                        } else {
                            // Scrub plaintext riotApiKey from the legacy JSON file on disk.
                            scrub_legacy_json_secret(&legacy_json);
                        }
                    }
                    Err(err) => {
                        let _ = backup_corrupt_file(&legacy_json);
                        store.warnings.push(format!(
                            "Could not migrate corrupt legacy settings.json ({err}); using defaults"
                        ));
                    }
                },
                Err(err) => {
                    store.warnings.push(format!(
                        "Could not read legacy settings.json ({err}); using defaults"
                    ));
                }
            }
        }

        store.settings.normalize();
        store
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Atomically persists settings and champion setups to TOML.
    /// Encrypts `riot_api_key` with Windows DPAPI and refuses to serialize plaintext secrets.
    pub fn save(&self) -> Result<(), String> {
        if self.path.as_os_str().is_empty() {
            return Err("No valid settings file path is configured".to_string());
        }
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "Settings path has no parent directory".to_string())?;
        fs::create_dir_all(parent)
            .map_err(|err| format!("Failed creating settings directory: {err}"))?;

        let encrypted_key = protect_secret_string(&self.settings.riot_api_key)?;
        if !self.settings.riot_api_key.is_empty() && !is_dpapi_protected_value(&encrypted_key) {
            return Err("Refusing to persist unencrypted Riot API key".to_string());
        }

        let toml_text =
            serialize_settings_toml(&self.settings, &encrypted_key, &self.champion_setups);
        atomic_write_text(&self.path, toml_text.as_bytes())
    }

    pub fn get_champion_setup(&self, champion_id: u32) -> Option<&ChampionSetup> {
        self.champion_setups.get(&champion_id)
    }

    /// Inserts a champion setup only if no setup is already stored for `champion_id`.
    /// Returns `true` if inserted, `false` if an existing setup was preserved untouched.
    pub fn insert_champion_setup_if_absent(
        &mut self,
        champion_id: u32,
        setup: ChampionSetup,
    ) -> bool {
        if self.champion_setups.contains_key(&champion_id) {
            return false;
        }
        self.champion_setups.insert(champion_id, setup);
        true
    }

    /// Stores a champion setup, rejecting overwrites unless `allow_overwrite` is `true`.
    pub fn save_champion_setup(
        &mut self,
        champion_id: u32,
        setup: ChampionSetup,
        allow_overwrite: bool,
    ) -> Result<(), SetupConflictError> {
        if !allow_overwrite && self.champion_setups.contains_key(&champion_id) {
            return Err(SetupConflictError::AlreadyExists(champion_id));
        }
        self.champion_setups.insert(champion_id, setup);
        Ok(())
    }

    /// Explicitly replaces or inserts a user-edited champion setup.
    pub fn overwrite_champion_setup(&mut self, champion_id: u32, setup: ChampionSetup) {
        self.champion_setups.insert(champion_id, setup);
    }

    pub fn remove_champion_setup(&mut self, champion_id: u32) -> Option<ChampionSetup> {
        self.champion_setups.remove(&champion_id)
    }
}

fn atomic_write_text(target: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = target
        .parent()
        .ok_or_else(|| "Target file has no parent directory".to_string())?;
    let file_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("settings.toml");

    let mut last_err = None;
    let mut created_tmp = None;
    for _ in 0..8 {
        let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!("{file_name}.tmp.{}.{seq}", std::process::id()));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(mut file) => {
                if let Err(err) = file
                    .write_all(bytes)
                    .and_then(|_| file.flush())
                    .and_then(|_| file.sync_all())
                {
                    drop(file);
                    let _ = fs::remove_file(&candidate);
                    return Err(format!("Failed writing temporary settings file: {err}"));
                }
                drop(file);
                created_tmp = Some(candidate);
                break;
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                last_err = Some(err);
                break;
            }
        }
    }

    let tmp_path = match created_tmp {
        Some(p) => p,
        None => {
            return Err(format!(
                "Could not create temporary settings file: {}",
                last_err
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "exhausted candidates".to_string())
            ));
        }
    };

    for attempt in 0..4 {
        match fs::rename(&tmp_path, target) {
            Ok(()) => return Ok(()),
            Err(err) if attempt < 3 && err.kind() == std::io::ErrorKind::PermissionDenied => {
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(err) => {
                let _ = fs::remove_file(&tmp_path);
                return Err(format!(
                    "Failed atomically replacing {}: {err}",
                    target.display()
                ));
            }
        }
    }
    let _ = fs::remove_file(&tmp_path);
    Err("Failed atomically replacing settings file".to_string())
}

fn backup_corrupt_file(path: &Path) -> Option<PathBuf> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("settings.toml");
    let backup = path.with_file_name(format!("{file_name}.corrupt.{ts}.{seq}"));
    fs::rename(path, &backup).ok().map(|_| backup)
}

fn scrub_legacy_json_secret(path: &Path) {
    let Ok(bytes) = fs::read(path) else {
        return;
    };
    let Ok(mut val) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return;
    };
    if let Some(obj) = val.as_object_mut() {
        if let Some(existing) = obj.get("riotApiKey").and_then(|v| v.as_str()) {
            if !existing.is_empty() {
                obj.insert(
                    "riotApiKey".to_string(),
                    serde_json::Value::String(String::new()),
                );
                if let Ok(scrubbed) = serde_json::to_vec_pretty(&val) {
                    let _ = atomic_write_text(path, &scrubbed);
                }
            }
        }
    }
}

// ============================================================================
// Self-contained TOML serializer & parser (no external `toml` crate needed)
// ============================================================================

#[derive(Clone, Debug)]
enum TomlVal {
    Str(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    Array(Vec<TomlVal>),
}

impl TomlVal {
    fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }
    fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }
    fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }
    fn as_f32(&self) -> Option<f32> {
        match self {
            Self::Float(f) => Some(*f as f32),
            Self::Int(i) => Some(*i as f32),
            _ => None,
        }
    }
    fn as_array(&self) -> Option<&[TomlVal]> {
        match self {
            Self::Array(arr) => Some(arr.as_slice()),
            _ => None,
        }
    }
}

fn serialize_settings_toml(
    s: &AppSettings,
    encrypted_api_key: &str,
    setups: &BTreeMap<u32, ChampionSetup>,
) -> String {
    let mut out = String::with_capacity(4096);
    out.push_str("# RiftCore Companion Settings (TOML)\n");
    out.push_str("# Riot API key is encrypted with Windows DPAPI (CurrentUser).\n\n");

    push_kv_bool(&mut out, "auto_accept", s.auto_accept);
    push_kv_bool(&mut out, "auto_import_runes", s.auto_import_runes);
    push_kv_bool(&mut out, "auto_import_spells", s.auto_import_spells);
    push_kv_bool(&mut out, "auto_import_item_sets", s.auto_import_item_sets);
    push_kv_str(&mut out, "flash_slot_preference", &s.flash_slot_preference);
    push_kv_bool(&mut out, "sound_alerts", s.sound_alerts);
    push_kv_bool(&mut out, "always_on_top", s.always_on_top);
    push_kv_bool(&mut out, "close_to_tray", s.close_to_tray);
    push_kv_bool(
        &mut out,
        "close_confirmation_seen",
        s.close_confirmation_seen,
    );
    push_kv_str(&mut out, "theme", &s.theme);
    push_kv_str(&mut out, "font_family", &s.font_family);
    push_kv_bool(&mut out, "first_run_tour_seen", s.first_run_tour_seen);
    push_kv_str(&mut out, "last_seen_version", &s.last_seen_version);
    push_kv_str(&mut out, "riot_region", &s.riot_region);
    push_kv_str(&mut out, "riot_api_key_encrypted", encrypted_api_key);
    push_kv_bool(&mut out, "hud_gold_lead", s.hud_gold_lead);
    push_kv_bool(&mut out, "hud_cs_benchmark", s.hud_cs_benchmark);
    push_kv_bool(&mut out, "hud_objective_timers", s.hud_objective_timers);
    push_kv_bool(&mut out, "hud_ally_ult_timers", s.hud_ally_ult_timers);
    push_kv_bool(&mut out, "hud_enemy_spell_timers", s.hud_enemy_spell_timers);
    push_kv_bool(&mut out, "hud_manual_camp_timers", s.hud_manual_camp_timers);
    push_kv_bool(&mut out, "hud_skill_order", s.hud_skill_order);
    push_kv_bool(&mut out, "hud_item_goal", s.hud_item_goal);
    push_kv_bool(&mut out, "hud_trinket_reminder", s.hud_trinket_reminder);
    push_kv_bool(&mut out, "hud_mayhem_cards", s.hud_mayhem_cards);
    push_kv_bool(&mut out, "hud_jungle_pathing", s.hud_jungle_pathing);
    push_kv_bool(&mut out, "hud_map_objective_pins", s.hud_map_objective_pins);
    push_kv_bool(&mut out, "hud_map_route_pins", s.hud_map_route_pins);
    push_kv_bool(&mut out, "hud_map_ward_pins", s.hud_map_ward_pins);
    push_kv_str(&mut out, "jungle_start_buff", &s.jungle_start_buff);
    push_kv_bool(&mut out, "hud_aram_health_timers", s.hud_aram_health_timers);
    push_kv_str(&mut out, "hud_mode", &s.hud_mode);
    push_kv_f32(&mut out, "hud_minimap_scale", s.hud_minimap_scale);
    push_kv_bool(&mut out, "hud_minimap_flip", s.hud_minimap_flip);
    push_kv_f32(&mut out, "hud_minimap_zoom", s.hud_minimap_zoom);
    push_kv_f32(&mut out, "hud_minimap_nudge_x", s.hud_minimap_nudge_x);
    push_kv_f32(&mut out, "hud_minimap_nudge_y", s.hud_minimap_nudge_y);
    push_kv_bool(&mut out, "hud_edit_layout", s.hud_edit_layout);
    push_kv_bool(&mut out, "hud_preview_mode", false);
    push_kv_u32_array(&mut out, "favorite_champions", &s.favorite_champions);
    push_kv_str_array(&mut out, "tft_board", &s.tft_board);

    out.push_str("\n[hud_positions]\n");
    let mut hud_keys: Vec<_> = s.hud_positions.keys().collect();
    hud_keys.sort();
    for key in hud_keys {
        if let Some([x, y]) = s.hud_positions.get(key) {
            out.push_str(&format!(
                "{} = [{}, {}]\n",
                format_toml_key(key),
                format_f32(*x),
                format_f32(*y)
            ));
        }
    }

    out.push_str("\n[champion_pool]\n");
    let mut pool_keys: Vec<_> = s.champion_pool.keys().collect();
    pool_keys.sort();
    for role in pool_keys {
        if let Some(ids) = s.champion_pool.get(role) {
            push_kv_u32_array(&mut out, &format_toml_key(role), ids);
        }
    }

    for (champion_id, setup) in setups {
        out.push_str(&format!("\n[champion_setups.{champion_id}]\n"));
        push_kv_str(&mut out, "role", &setup.role);
        out.push_str(&format!("primary_tree = {}\n", setup.primary_tree));
        out.push_str(&format!("secondary_tree = {}\n", setup.secondary_tree));
        push_kv_i64_array(&mut out, "perks", &setup.perks);
        let rows = [
            setup.secondary_rows[0] as i64,
            setup.secondary_rows[1] as i64,
        ];
        push_kv_i64_array(&mut out, "secondary_rows", &rows);
        out.push_str(&format!("spell_one = {}\n", setup.spell_one));
        out.push_str(&format!("spell_two = {}\n", setup.spell_two));
    }

    out
}

fn push_kv_bool(out: &mut String, key: &str, value: bool) {
    out.push_str(&format!("{key} = {value}\n"));
}

fn push_kv_str(out: &mut String, key: &str, value: &str) {
    out.push_str(&format!("{key} = {}\n", quote_toml_str(value)));
}

fn push_kv_f32(out: &mut String, key: &str, value: f32) {
    out.push_str(&format!("{key} = {}\n", format_f32(value)));
}

fn push_kv_u32_array(out: &mut String, key: &str, values: &[u32]) {
    out.push_str(key);
    out.push_str(" = [");
    for (idx, val) in values.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        out.push_str(&val.to_string());
    }
    out.push_str("]\n");
}

fn push_kv_i64_array(out: &mut String, key: &str, values: &[i64]) {
    out.push_str(key);
    out.push_str(" = [");
    for (idx, val) in values.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        out.push_str(&val.to_string());
    }
    out.push_str("]\n");
}

fn push_kv_str_array(out: &mut String, key: &str, values: &[String]) {
    out.push_str(key);
    out.push_str(" = [");
    for (idx, val) in values.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        out.push_str(&quote_toml_str(val));
    }
    out.push_str("]\n");
}

fn format_f32(value: f32) -> String {
    let safe = if value.is_finite() { value } else { 0.0 };
    let formatted = format!("{safe}");
    if formatted.contains('.') || formatted.contains('e') || formatted.contains('E') {
        formatted
    } else {
        format!("{formatted}.0")
    }
}

fn format_toml_key(key: &str) -> String {
    if !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        key.to_string()
    } else {
        quote_toml_str(key)
    }
}

fn quote_toml_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn parse_settings_toml(
    input: &str,
    warnings: &mut Vec<String>,
) -> Result<(AppSettings, BTreeMap<u32, ChampionSetup>, bool), String> {
    let mut settings = AppSettings::default();
    let mut setups: BTreeMap<u32, ChampionSetup> = BTreeMap::new();
    let mut had_plaintext_key = false;
    let mut seen_encrypted_key = false;
    let mut pool_entries: HashMap<String, Vec<u32>> = HashMap::new();
    let mut saw_pool_section = false;

    let statements = split_toml_statements(input)?;
    let mut current_section = String::new();

    for (line_no, stmt) in statements {
        if stmt.starts_with('[') && stmt.ends_with(']') {
            let inner = stmt[1..stmt.len() - 1].trim();
            if inner.is_empty() || inner.starts_with('[') || inner.ends_with(']') {
                return Err(format!("Invalid TOML table header on line {line_no}"));
            }
            current_section = inner.to_string();
            if current_section == "champion_pool" {
                saw_pool_section = true;
            } else if let Some(id_str) = current_section.strip_prefix("champion_setups.") {
                let id = id_str.trim().parse::<u32>().map_err(|_| {
                    format!("Invalid champion setup ID '{id_str}' on line {line_no}")
                })?;
                setups.entry(id).or_default();
            }
            continue;
        }

        let (raw_key, raw_val) = split_toml_kv(&stmt)
            .ok_or_else(|| format!("Expected key = value on line {line_no}"))?;
        let key =
            parse_toml_key(raw_key).ok_or_else(|| format!("Invalid TOML key on line {line_no}"))?;
        let val = parse_toml_val(raw_val)
            .map_err(|err| format!("Invalid TOML value for '{key}' on line {line_no}: {err}"))?;

        if current_section.is_empty() {
            match key.as_str() {
                "auto_accept" => settings.auto_accept = val.as_bool().ok_or("expected bool")?,
                "auto_import_runes" => {
                    settings.auto_import_runes = val.as_bool().ok_or("expected bool")?
                }
                "auto_import_spells" => {
                    settings.auto_import_spells = val.as_bool().ok_or("expected bool")?
                }
                "auto_import_item_sets" => {
                    settings.auto_import_item_sets = val.as_bool().ok_or("expected bool")?
                }
                "flash_slot_preference" => {
                    settings.flash_slot_preference =
                        val.as_str().ok_or("expected string")?.to_string()
                }
                "sound_alerts" => settings.sound_alerts = val.as_bool().ok_or("expected bool")?,
                "always_on_top" => settings.always_on_top = val.as_bool().ok_or("expected bool")?,
                "close_to_tray" => settings.close_to_tray = val.as_bool().ok_or("expected bool")?,
                "close_confirmation_seen" => {
                    settings.close_confirmation_seen = val.as_bool().ok_or("expected bool")?
                }
                "theme" => settings.theme = val.as_str().ok_or("expected string")?.to_string(),
                "font_family" => {
                    settings.font_family = val.as_str().ok_or("expected string")?.to_string()
                }
                "first_run_tour_seen" => {
                    settings.first_run_tour_seen = val.as_bool().ok_or("expected bool")?
                }
                "last_seen_version" => {
                    settings.last_seen_version = val.as_str().ok_or("expected string")?.to_string()
                }
                "riot_region" => {
                    settings.riot_region = val.as_str().ok_or("expected string")?.to_string()
                }
                "riot_api_key_encrypted" => {
                    seen_encrypted_key = true;
                    let encoded = val.as_str().ok_or("expected string")?;
                    if !encoded.is_empty() {
                        match unprotect_secret_string(encoded) {
                            Ok(decrypted) => settings.set_riot_api_key(decrypted.to_string()),
                            Err(err) => {
                                settings.clear_riot_api_key();
                                warnings.push(format!(
                                    "Could not decrypt stored Riot API key (reset to empty): {err}"
                                ));
                            }
                        }
                    }
                }
                "riot_api_key" => {
                    // Recover legacy plaintext value if hand-edited, and trigger immediate DPAPI re-encryption.
                    let raw_secret = val.as_str().ok_or("expected string")?;
                    if !raw_secret.is_empty() {
                        had_plaintext_key = true;
                        if !seen_encrypted_key {
                            if is_dpapi_protected_value(raw_secret) {
                                if let Ok(decrypted) = unprotect_secret_string(raw_secret) {
                                    settings.set_riot_api_key(decrypted.to_string());
                                }
                            } else {
                                settings.set_riot_api_key(raw_secret.to_string());
                            }
                        }
                    }
                }
                "hud_gold_lead" => settings.hud_gold_lead = val.as_bool().ok_or("expected bool")?,
                "hud_cs_benchmark" => {
                    settings.hud_cs_benchmark = val.as_bool().ok_or("expected bool")?
                }
                "hud_objective_timers" => {
                    settings.hud_objective_timers = val.as_bool().ok_or("expected bool")?
                }
                "hud_ally_ult_timers" => {
                    settings.hud_ally_ult_timers = val.as_bool().ok_or("expected bool")?
                }
                "hud_enemy_spell_timers" => {
                    settings.hud_enemy_spell_timers = val.as_bool().ok_or("expected bool")?
                }
                "hud_manual_camp_timers" => {
                    settings.hud_manual_camp_timers = val.as_bool().ok_or("expected bool")?
                }
                "hud_skill_order" => {
                    settings.hud_skill_order = val.as_bool().ok_or("expected bool")?
                }
                "hud_item_goal" => settings.hud_item_goal = val.as_bool().ok_or("expected bool")?,
                "hud_trinket_reminder" => {
                    settings.hud_trinket_reminder = val.as_bool().ok_or("expected bool")?
                }
                "hud_mayhem_cards" => {
                    settings.hud_mayhem_cards = val.as_bool().ok_or("expected bool")?
                }
                "hud_jungle_pathing" => {
                    settings.hud_jungle_pathing = val.as_bool().ok_or("expected bool")?
                }
                "hud_map_objective_pins" => {
                    settings.hud_map_objective_pins = val.as_bool().ok_or("expected bool")?
                }
                "hud_map_route_pins" => {
                    settings.hud_map_route_pins = val.as_bool().ok_or("expected bool")?
                }
                "hud_map_ward_pins" => {
                    settings.hud_map_ward_pins = val.as_bool().ok_or("expected bool")?
                }
                "jungle_start_buff" => {
                    settings.jungle_start_buff = val.as_str().ok_or("expected string")?.to_string()
                }
                "hud_aram_health_timers" => {
                    settings.hud_aram_health_timers = val.as_bool().ok_or("expected bool")?
                }
                "hud_mode" => {
                    settings.hud_mode = val.as_str().ok_or("expected string")?.to_string()
                }
                "hud_minimap_scale" => {
                    settings.hud_minimap_scale = val.as_f32().ok_or("expected float")?
                }
                "hud_minimap_flip" => {
                    settings.hud_minimap_flip = val.as_bool().ok_or("expected bool")?
                }
                "hud_minimap_zoom" => {
                    settings.hud_minimap_zoom = val.as_f32().ok_or("expected float")?
                }
                "hud_minimap_nudge_x" => {
                    settings.hud_minimap_nudge_x = val.as_f32().ok_or("expected float")?
                }
                "hud_minimap_nudge_y" => {
                    settings.hud_minimap_nudge_y = val.as_f32().ok_or("expected float")?
                }
                "hud_edit_layout" => {
                    settings.hud_edit_layout = val.as_bool().ok_or("expected bool")?
                }
                "hud_preview_mode" => {
                    let _ = val.as_bool();
                    settings.hud_preview_mode = false;
                }
                "favorite_champions" => {
                    let arr = val.as_array().ok_or("expected array")?;
                    let mut ids = Vec::with_capacity(arr.len());
                    for item in arr {
                        let v = item.as_i64().ok_or("expected integer")?;
                        if let Ok(u) = u32::try_from(v) {
                            ids.push(u);
                        }
                    }
                    settings.favorite_champions = ids;
                }
                "tft_board" => {
                    let arr = val.as_array().ok_or("expected array")?;
                    let mut board = Vec::with_capacity(arr.len());
                    for item in arr {
                        board.push(item.as_str().ok_or("expected string")?.to_string());
                    }
                    settings.tft_board = board;
                }
                _ => {}
            }
        } else if current_section == "hud_positions" {
            let arr = val.as_array().ok_or("expected [x, y] array")?;
            if arr.len() >= 2 {
                let x = arr[0].as_f32().ok_or("expected float")?;
                let y = arr[1].as_f32().ok_or("expected float")?;
                settings.hud_positions.insert(key, [x, y]);
            }
        } else if current_section == "champion_pool" {
            let arr = val.as_array().ok_or("expected array")?;
            let mut ids = Vec::with_capacity(arr.len());
            for item in arr {
                let v = item.as_i64().ok_or("expected integer")?;
                if let Ok(u) = u32::try_from(v) {
                    ids.push(u);
                }
            }
            pool_entries.insert(key, ids);
        } else if let Some(id_str) = current_section.strip_prefix("champion_setups.") {
            let id = id_str
                .trim()
                .parse::<u32>()
                .map_err(|_| "invalid setup id")?;
            let setup = setups.entry(id).or_default();
            match key.as_str() {
                "role" => setup.role = val.as_str().ok_or("expected string")?.to_string(),
                "primary_tree" => {
                    let v = val.as_i64().ok_or("expected integer")?;
                    setup.primary_tree = u32::try_from(v).map_err(|_| "invalid tree id")?;
                }
                "secondary_tree" => {
                    let v = val.as_i64().ok_or("expected integer")?;
                    setup.secondary_tree = u32::try_from(v).map_err(|_| "invalid tree id")?;
                }
                "perks" => {
                    let arr = val.as_array().ok_or("expected array")?;
                    for (idx, item) in arr.iter().take(9).enumerate() {
                        setup.perks[idx] = item.as_i64().ok_or("expected integer")?;
                    }
                }
                "secondary_rows" => {
                    let arr = val.as_array().ok_or("expected array")?;
                    for (idx, item) in arr.iter().take(2).enumerate() {
                        let v = item.as_i64().ok_or("expected integer")?;
                        setup.secondary_rows[idx] =
                            usize::try_from(v).map_err(|_| "invalid row index")?;
                    }
                }
                "spell_one" => setup.spell_one = val.as_i64().ok_or("expected integer")?,
                "spell_two" => setup.spell_two = val.as_i64().ok_or("expected integer")?,
                _ => {}
            }
        }
    }

    if saw_pool_section {
        settings.champion_pool = pool_entries;
    }

    Ok((settings, setups, had_plaintext_key))
}

fn split_toml_statements(input: &str) -> Result<Vec<(usize, String)>, String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut start_line = 1usize;
    let mut line_no = 1usize;
    let mut bracket_depth = 0usize;
    let mut in_basic_str = false;
    let mut in_lit_str = false;
    let mut escaped = false;
    let mut in_comment = false;

    for ch in input.chars() {
        if in_comment {
            if ch == '\n' {
                in_comment = false;
                if bracket_depth == 0 {
                    let trimmed = current.trim();
                    if !trimmed.is_empty() {
                        statements.push((start_line, trimmed.to_string()));
                    }
                    current.clear();
                } else {
                    current.push(' ');
                }
                line_no += 1;
            }
            continue;
        }

        if in_basic_str {
            if ch == '\n' {
                return Err(format!("Unterminated string literal on line {line_no}"));
            }
            current.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_basic_str = false;
            }
            continue;
        }

        if in_lit_str {
            if ch == '\n' {
                return Err(format!("Unterminated literal string on line {line_no}"));
            }
            current.push(ch);
            if ch == '\'' {
                in_lit_str = false;
            }
            continue;
        }

        match ch {
            '#' => in_comment = true,
            '"' => {
                if current.trim().is_empty() {
                    start_line = line_no;
                }
                in_basic_str = true;
                current.push(ch);
            }
            '\'' => {
                if current.trim().is_empty() {
                    start_line = line_no;
                }
                in_lit_str = true;
                current.push(ch);
            }
            '[' => {
                if current.trim().is_empty() {
                    start_line = line_no;
                }
                bracket_depth += 1;
                current.push(ch);
            }
            ']' => {
                if bracket_depth == 0 {
                    return Err(format!("Unexpected ']' on line {line_no}"));
                }
                bracket_depth -= 1;
                current.push(ch);
            }
            '\n' => {
                if bracket_depth == 0 {
                    let trimmed = current.trim();
                    if !trimmed.is_empty() {
                        statements.push((start_line, trimmed.to_string()));
                    }
                    current.clear();
                } else {
                    current.push(' ');
                }
                line_no += 1;
            }
            '\r' => {}
            _ => {
                if current.trim().is_empty() && !ch.is_whitespace() {
                    start_line = line_no;
                }
                current.push(ch);
            }
        }
    }

    if in_basic_str || in_lit_str {
        return Err(format!("Unterminated string literal on line {line_no}"));
    }
    if bracket_depth != 0 {
        return Err(format!(
            "Unclosed '[' bracket starting on line {start_line}"
        ));
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        statements.push((start_line, trimmed.to_string()));
    }
    Ok(statements)
}

fn split_toml_kv(stmt: &str) -> Option<(&str, &str)> {
    let mut in_basic = false;
    let mut in_lit = false;
    let mut escaped = false;
    for (idx, ch) in stmt.char_indices() {
        if in_basic {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_basic = false;
            }
            continue;
        }
        if in_lit {
            if ch == '\'' {
                in_lit = false;
            }
            continue;
        }
        match ch {
            '"' => in_basic = true,
            '\'' => in_lit = true,
            '=' => return Some((stmt[..idx].trim(), stmt[idx + 1..].trim())),
            _ => {}
        }
    }
    None
}

fn parse_toml_key(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with('"') || trimmed.starts_with('\'') {
        parse_toml_string(trimmed).ok()
    } else if trimmed
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn parse_toml_val(raw: &str) -> Result<TomlVal, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("empty value".to_string());
    }
    if trimmed == "true" {
        return Ok(TomlVal::Bool(true));
    }
    if trimmed == "false" {
        return Ok(TomlVal::Bool(false));
    }
    if trimmed.starts_with('"') || trimmed.starts_with('\'') {
        return parse_toml_string(trimmed).map(TomlVal::Str);
    }
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = trimmed[1..trimmed.len() - 1].trim();
        if inner.is_empty() {
            return Ok(TomlVal::Array(Vec::new()));
        }
        let items = split_toml_array_items(inner)?;
        let mut parsed = Vec::with_capacity(items.len());
        for item in items {
            parsed.push(parse_toml_val(item)?);
        }
        return Ok(TomlVal::Array(parsed));
    }

    let cleaned = trimmed.replace('_', "");
    if cleaned.contains('.') || cleaned.contains('e') || cleaned.contains('E') {
        let f = cleaned
            .parse::<f64>()
            .map_err(|_| format!("invalid float '{trimmed}'"))?;
        if !f.is_finite() {
            return Err("non-finite float".to_string());
        }
        return Ok(TomlVal::Float(f));
    }

    cleaned
        .parse::<i64>()
        .map(TomlVal::Int)
        .map_err(|_| format!("unrecognized token '{trimmed}'"))
}

fn split_toml_array_items(inner: &str) -> Result<Vec<&str>, String> {
    let mut items = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    let mut in_basic = false;
    let mut in_lit = false;
    let mut escaped = false;

    for (idx, ch) in inner.char_indices() {
        if in_basic {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_basic = false;
            }
            continue;
        }
        if in_lit {
            if ch == '\'' {
                in_lit = false;
            }
            continue;
        }
        match ch {
            '"' => in_basic = true,
            '\'' => in_lit = true,
            '[' => depth += 1,
            ']' => {
                if depth == 0 {
                    return Err("unbalanced ']' inside array".to_string());
                }
                depth -= 1;
            }
            ',' if depth == 0 => {
                let piece = inner[start..idx].trim();
                if piece.is_empty() {
                    return Err("empty element in array".to_string());
                }
                items.push(piece);
                start = idx + 1;
            }
            _ => {}
        }
    }
    let tail = inner[start..].trim();
    if !tail.is_empty() {
        items.push(tail);
    }
    Ok(items)
}

fn parse_toml_string(raw: &str) -> Result<String, String> {
    if let Some(rest) = raw.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
        return Ok(rest.to_string());
    }
    let Some(inner) = raw.strip_prefix('"').and_then(|s| s.strip_suffix('"')) else {
        return Err("malformed string quotes".to_string());
    };
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        let esc = chars.next().ok_or("trailing backslash in string")?;
        match esc {
            '"' => out.push('"'),
            '\\' => out.push('\\'),
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            't' => out.push('\t'),
            'b' => out.push('\u{0008}'),
            'f' => out.push('\u{000C}'),
            'u' => out.push(parse_unicode_escape(&mut chars, 4)?),
            'U' => out.push(parse_unicode_escape(&mut chars, 8)?),
            other => return Err(format!("unsupported escape \\{other}")),
        }
    }
    Ok(out)
}

fn parse_unicode_escape(chars: &mut std::str::Chars<'_>, digits: usize) -> Result<char, String> {
    let mut hex = String::with_capacity(digits);
    for _ in 0..digits {
        hex.push(chars.next().ok_or("incomplete unicode escape")?);
    }
    let code = u32::from_str_radix(&hex, 16).map_err(|_| "invalid unicode hex")?;
    char::from_u32(code).ok_or_else(|| "invalid unicode scalar".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_config_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "riftcore-settings-test-{}-{}",
            std::process::id(),
            TEMP_SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        dir.join(name)
    }

    #[cfg(windows)]
    #[test]
    fn settings_store_atomic_toml_roundtrip_and_dpapi_protection() {
        let path = temp_config_path("settings.toml");
        let mut store = SettingsStore::load(path.clone());
        store
            .settings
            .set_riot_api_key("RGAPI-unit-test-secret-9999".to_string());
        store.settings.theme = "void".to_string();
        store
            .settings
            .hud_positions
            .insert("scoreboard".to_string(), [120.5, 42.0]);
        store.overwrite_champion_setup(
            103,
            ChampionSetup {
                role: "MID".to_string(),
                primary_tree: 8100,
                secondary_tree: 8200,
                perks: [8112, 8126, 8140, 8135, 8226, 8210, 5008, 5008, 5001],
                secondary_rows: [1, 2],
                spell_one: 4,
                spell_two: 14,
            },
        );
        store.save().expect("save should succeed");

        let disk_contents = fs::read_to_string(&path).expect("read saved toml");
        assert!(!disk_contents.contains("RGAPI-unit-test-secret-9999"));
        assert!(disk_contents.contains("riot_api_key_encrypted = \"dpapi:v1:"));

        let loaded = SettingsStore::load(path.clone());
        assert_eq!(loaded.settings.riot_api_key, "RGAPI-unit-test-secret-9999");
        assert_eq!(loaded.settings.theme, "void");
        assert_eq!(
            loaded.settings.hud_positions.get("scoreboard"),
            Some(&[120.5, 42.0])
        );
        assert_eq!(
            loaded.get_champion_setup(103).map(|s| s.spell_two),
            Some(14)
        );

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn champion_setup_never_silently_overwrites() {
        let path = temp_config_path("setups.toml");
        let mut store = SettingsStore::load(path.clone());
        let first = ChampionSetup {
            role: "TOP".to_string(),
            spell_two: 12,
            ..Default::default()
        };
        let second = ChampionSetup {
            role: "MID".to_string(),
            spell_two: 14,
            ..Default::default()
        };
        assert!(store.insert_champion_setup_if_absent(266, first.clone()));
        assert!(!store.insert_champion_setup_if_absent(266, second.clone()));
        assert_eq!(
            store.save_champion_setup(266, second.clone(), false),
            Err(SetupConflictError::AlreadyExists(266))
        );
        assert_eq!(store.get_champion_setup(266), Some(&first));

        store
            .save_champion_setup(266, second.clone(), true)
            .expect("explicit overwrite should succeed");
        assert_eq!(store.get_champion_setup(266), Some(&second));
    }

    #[test]
    fn corrupt_toml_is_backed_up_and_recovers_defaults() {
        let path = temp_config_path("corrupt.toml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "theme = \"unterminated\nauto_accept = maybe").unwrap();

        let store = SettingsStore::load(path.clone());
        assert!(!store.warnings.is_empty());
        assert_eq!(store.settings.theme, "hextech");
        assert!(!path.exists());

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }
}
