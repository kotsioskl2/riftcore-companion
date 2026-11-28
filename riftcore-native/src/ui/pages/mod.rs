//! Page router, per-page state structs, and shared champion setup synchronization.

pub mod champ_select;
pub mod champion_pool;
pub mod draft;
pub mod hud_page;
pub mod lobby;
pub mod modes;
pub mod profile;
pub mod settings;
pub mod tft;
pub mod tier_list;

use crate::config::settings::{ChampionSetup, SettingsStore};
use crate::domain::analytics::NormalizedMatchSummary;
use crate::domain::builds;
use crate::domain::champions::{self, Champion, Role};
use crate::domain::live_game::LiveGameState;
use crate::domain::mayhem::{CardMode, CardTier};
use crate::domain::protocol::{CoreCommand, CoreNotice, CoreSnapshot, GameflowPhase};
use crate::domain::runes;
use crate::platform::window::LeagueWindowInfo;
use crate::ui::textures::TextureManager;
use crate::ui::theme::ThemePalette;
use eframe::egui;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Top-level navigation destinations in the main RiftCore window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Page {
    #[default]
    ChampSelect,
    Draft,
    Lobby,
    Hud,
    Tft,
    Modes,
    TierList,
    ChampionPool,
    Profile,
    Settings,
}

/// Settings view sections.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SettingsSection {
    #[default]
    Gameplay,
    Overlay,
    RiotApi,
    App,
}

/// HUD configuration tabs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HudSettingsTab {
    #[default]
    Timers,
    Stats,
    Guides,
    Layout,
}

/// TFT Companion tabs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TftTab {
    #[default]
    Comps,
    Items,
    Odds,
    Augments,
    Board,
    SetData,
}

/// TFT Set Data category filters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TftDataCategory {
    #[default]
    Units,
    Traits,
    Items,
}

/// Tier list sort mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TierSort {
    #[default]
    Name,
    Role,
    Archetype,
}

/// Supported summoner spells for pre-game selection.
pub const SUMMONER_SPELLS: &[(i64, &str)] = &[
    (4, "Flash"),
    (12, "Teleport"),
    (14, "Ignite"),
    (11, "Smite"),
    (32, "Mark / Snowball"),
    (6, "Ghost"),
    (7, "Heal"),
    (3, "Exhaust"),
    (21, "Barrier"),
    (1, "Cleanse"),
];

pub fn spell_name(id: i64) -> &'static str {
    SUMMONER_SPELLS
        .iter()
        .find(|(spell_id, _)| *spell_id == id)
        .map(|(_, name)| *name)
        .unwrap_or("Spell")
}

/// Orders `(first, second)` so Flash (`4`) sits on the user's preferred D/F slot.
pub fn preferred_spell_order(first: i64, second: i64, flash_slot: &str) -> (i64, i64) {
    if first == 4 || second == 4 {
        let other = if first == 4 { second } else { first };
        if flash_slot.eq_ignore_ascii_case("D") {
            (4, other)
        } else {
            (other, 4)
        }
    } else {
        (first, second)
    }
}

#[derive(Clone, Debug)]
pub struct ChampSelectState {
    pub selected_champion: u32,
    pub build_role: Role,
    pub detail_tab: usize,
    pub champion_search: String,
    pub role_filter: Option<Role>,
    pub last_synced_champion: u32,
    pub primary_tree: u32,
    pub secondary_tree: u32,
    pub selected_perks: [i64; 9],
    pub secondary_rune_rows: [usize; 2],
    pub spell_one: i64,
    pub spell_two: i64,
    pub active_spell_slot: usize,
}

impl Default for ChampSelectState {
    fn default() -> Self {
        let mut perks = runes::burst_preset_perks();
        let mut secondary_rows =
            runes::secondary_rows_for_perks(runes::TREE_SORCERY, perks[4], perks[5]);
        runes::normalize_rune_setup(
            runes::TREE_DOMINATION,
            runes::TREE_SORCERY,
            &mut secondary_rows,
            &mut perks,
        );
        Self {
            selected_champion: 103, // Ahri
            build_role: Role::Mid,
            detail_tab: 1,
            champion_search: String::new(),
            role_filter: None,
            last_synced_champion: 0,
            primary_tree: runes::TREE_DOMINATION,
            secondary_tree: runes::TREE_SORCERY,
            selected_perks: perks,
            secondary_rune_rows: secondary_rows,
            spell_one: 14,
            spell_two: 4,
            active_spell_slot: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DraftState {
    pub allies: [u32; 5],
    pub enemies: [u32; 5],
}

impl Default for DraftState {
    fn default() -> Self {
        Self {
            allies: [266, 64, 0, 222, 412],
            enemies: [122, 254, 238, 51, 53],
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct HudPageState {
    pub tab: HudSettingsTab,
    pub aram_relic_seconds: String,
    pub manual_timers: HashMap<String, Instant>,
}

#[derive(Clone, Debug)]
pub struct ModesState {
    pub selected_mode: usize,
    pub search: String,
    pub mayhem_search: String,
    pub mayhem_tier: Option<CardTier>,
    pub mayhem_mode: CardMode,
}

impl Default for ModesState {
    fn default() -> Self {
        Self {
            selected_mode: 0,
            search: String::new(),
            mayhem_search: String::new(),
            mayhem_tier: None,
            mayhem_mode: CardMode::Both,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TftState {
    pub tab: TftTab,
    pub selected_comp: String,
    pub comp_search: String,
    pub item_search: String,
    pub augment_search: String,
    pub data_search: String,
    pub data_category: TftDataCategory,
    pub component_one: String,
    pub component_two: String,
    pub unit_input: String,
    pub selected_cell: Option<usize>,
}

impl Default for TftState {
    fn default() -> Self {
        Self {
            tab: TftTab::Comps,
            selected_comp: String::new(),
            comp_search: String::new(),
            item_search: String::new(),
            augment_search: String::new(),
            data_search: String::new(),
            data_category: TftDataCategory::Units,
            component_one: "sword".to_string(),
            component_two: "gloves".to_string(),
            unit_input: String::new(),
            selected_cell: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TierListState {
    pub role_filter: Option<Role>,
    pub search: String,
    pub view_cards: bool,
    pub sort: TierSort,
}

#[derive(Clone, Debug)]
pub struct ChampionPoolState {
    pub role: Role,
    pub search: String,
    pub assign_slot: Option<usize>,
}

impl Default for ChampionPoolState {
    fn default() -> Self {
        Self {
            role: Role::Mid,
            search: String::new(),
            assign_slot: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProfileState {
    pub riot_id_search: String,
    pub lookup_loading: bool,
    pub trend_window: usize,
    pub selected_analysis_match: Option<NormalizedMatchSummary>,
}

impl Default for ProfileState {
    fn default() -> Self {
        Self {
            riot_id_search: String::new(),
            lookup_loading: false,
            trend_window: 10,
            selected_analysis_match: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SettingsPageState {
    pub section: SettingsSection,
    pub show_api_key: bool,
}

/// Consolidated per-page and shell UI state (no flat 70-field root struct).
#[derive(Debug)]
pub struct AppUiState {
    pub active_page: Page,
    pub champ_select: ChampSelectState,
    pub draft: DraftState,
    pub hud_page: HudPageState,
    pub modes: ModesState,
    pub tft: TftState,
    pub tier_list: TierListState,
    pub champion_pool: ChampionPoolState,
    pub profile: ProfileState,
    pub settings: SettingsPageState,

    pub overlay_mode: bool,
    pub overlay_auto_entered: bool,
    pub overlay_waiting_for_window: bool,
    pub overlay_dismissed_by_user: bool,
    pub overlay_mouse_passthrough: bool,
    pub overlay_target: Option<LeagueWindowInfo>,
    pub overlay_target_is_foreground: bool,
    pub ready_check_alerted: bool,
    pub pinned: bool,

    pub show_quick_tour: bool,
    pub show_whats_new: bool,
    /// In-game HUD cards drawer (Mayhem & Arena augments).
    pub show_mayhem_cards: bool,
    /// Card guide dialog in the main window; separate from the HUD drawer.
    pub show_mayhem_guide: bool,
    pub show_close_confirmation: bool,
    pub force_exit: bool,
    pub update_banner_dismissed: bool,
    pub tray_available: bool,

    pub notice: Option<CoreNotice>,
    pub notice_until: Option<Instant>,
    pub parsed_live_game: Option<LiveGameState>,
}

impl Default for AppUiState {
    fn default() -> Self {
        Self {
            active_page: Page::ChampSelect,
            champ_select: ChampSelectState::default(),
            draft: DraftState::default(),
            hud_page: HudPageState::default(),
            modes: ModesState::default(),
            tft: TftState::default(),
            tier_list: TierListState::default(),
            champion_pool: ChampionPoolState::default(),
            profile: ProfileState::default(),
            settings: SettingsPageState::default(),
            overlay_mode: false,
            overlay_auto_entered: false,
            overlay_waiting_for_window: false,
            overlay_dismissed_by_user: false,
            overlay_mouse_passthrough: false,
            overlay_target: None,
            overlay_target_is_foreground: false,
            ready_check_alerted: false,
            pinned: false,
            show_quick_tour: false,
            show_whats_new: false,
            show_mayhem_cards: false,
            show_mayhem_guide: false,
            show_close_confirmation: false,
            force_exit: false,
            update_banner_dismissed: false,
            tray_available: false,
            notice: None,
            notice_until: None,
            parsed_live_game: None,
        }
    }
}

impl AppUiState {
    pub fn set_notice(&mut self, notice: CoreNotice) {
        self.notice = Some(notice);
        self.notice_until = Some(Instant::now() + Duration::from_secs(5));
    }

    pub fn active_notice(&mut self) -> Option<&CoreNotice> {
        match self.notice_until {
            Some(until) if Instant::now() < until => self.notice.as_ref(),
            _ => {
                self.notice_until = None;
                self.notice = None;
                None
            }
        }
    }
}

/// Deferred side effects collected from pure page/chrome renders and executed by `RiftCoreApp`.
#[derive(Clone, Debug, Default)]
pub struct PendingUiEffects {
    pub save_settings: bool,
    pub apply_theme: bool,
    pub apply_font: bool,
    pub apply_always_on_top: bool,
    pub set_overlay: Option<(bool, bool)>,
    pub set_passthrough: Option<bool>,
    pub open_browser_url: Option<String>,
}

/// Mutable render context passed cleanly to chrome and all 10 page views.
pub struct PageContext<'a> {
    pub snapshot: &'a CoreSnapshot,
    pub store: &'a mut SettingsStore,
    pub palette: &'a ThemePalette,
    pub textures: &'a mut TextureManager,
    pub state: &'a mut AppUiState,
    pub commands: &'a mut Vec<CoreCommand>,
    pub effects: &'a mut PendingUiEffects,
}

impl<'a> PageContext<'a> {
    pub fn send_command(&mut self, cmd: CoreCommand) {
        self.commands.push(cmd);
    }

    pub fn request_save(&mut self) {
        self.effects.save_settings = true;
    }

    pub fn set_notice(&mut self, notice: CoreNotice) {
        self.state.set_notice(notice);
    }

    pub fn open_url(&mut self, url: impl Into<String>) {
        self.effects.open_browser_url = Some(url.into());
    }

    /// Synchronizes champion selection across Champ Select, Tier List, Champion Pool,
    /// Game Modes, and LCU auto-sync so rune/spell state never goes stale or overwrites
    /// another champion's saved setup.
    pub fn select_champion_setup(
        &mut self,
        champion_id: u32,
        requested_role: Option<Role>,
        allow_auto_import: bool,
    ) {
        let Some(champion) = champions::champion_by_id(champion_id) else {
            return;
        };

        select_champion_setup_inner(
            self.snapshot,
            self.store,
            self.state,
            self.commands,
            champion,
            requested_role,
            allow_auto_import,
        );
    }

    /// Navigates to Champion Select for `champion_id` using the canonical setup loader.
    pub fn navigate_to_champion(&mut self, champion_id: u32, requested_role: Option<Role>) {
        self.select_champion_setup(champion_id, requested_role, false);
        self.state.active_page = Page::ChampSelect;
    }

    /// Persists the active Champ Select rune/spell editor values ONLY when edited by the user.
    pub fn persist_active_champion_setup(&mut self) {
        let cs = &self.state.champ_select;
        if cs.selected_champion == 0 {
            return;
        }
        let setup = ChampionSetup {
            role: cs.build_role.as_str().to_string(),
            primary_tree: cs.primary_tree,
            secondary_tree: cs.secondary_tree,
            perks: cs.selected_perks,
            secondary_rows: cs.secondary_rune_rows,
            spell_one: cs.spell_one,
            spell_two: cs.spell_two,
        };
        self.store
            .overwrite_champion_setup(cs.selected_champion, setup);
        self.effects.save_settings = true;
    }
}

/// Pure helper shared by `PageContext` and `RiftCoreApp::logic` so LCU champ-select
/// sync and UI clicks share identical setup resolution.
pub fn select_champion_setup_inner(
    snapshot: &CoreSnapshot,
    store: &SettingsStore,
    state: &mut AppUiState,
    commands: &mut Vec<CoreCommand>,
    champion: &Champion,
    requested_role: Option<Role>,
    allow_auto_import: bool,
) {
    let cs = &mut state.champ_select;
    cs.selected_champion = champion.id;
    cs.last_synced_champion = champion.id;

    let saved = store.get_champion_setup(champion.id);
    let use_saved = saved.filter(|setup| match requested_role {
        Some(role) => Role::from_str_lossy(&setup.role) == Some(role),
        None => true,
    });

    if let Some(setup) = use_saved {
        cs.build_role =
            Role::from_str_lossy(&setup.role).unwrap_or_else(|| champion.primary_role());
        cs.primary_tree = setup.primary_tree;
        cs.secondary_tree = if setup.secondary_tree == setup.primary_tree {
            runes::RUNE_TREE_NAMES
                .iter()
                .find(|(id, _)| *id != setup.primary_tree)
                .map(|(id, _)| *id)
                .unwrap_or(runes::TREE_PRECISION)
        } else {
            setup.secondary_tree
        };
        cs.selected_perks = setup.perks;
        cs.secondary_rune_rows = setup.secondary_rows;
        runes::normalize_rune_setup(
            cs.primary_tree,
            cs.secondary_tree,
            &mut cs.secondary_rune_rows,
            &mut cs.selected_perks,
        );
        cs.spell_one = setup.spell_one;
        cs.spell_two = setup.spell_two;
    } else {
        let profile = builds::build_for_champion(
            champion.id,
            requested_role.unwrap_or_else(|| {
                builds::default_build_for_champion(champion.id)
                    .map(|p| p.default_role)
                    .unwrap_or_else(|| champion.primary_role())
            }),
        )
        .cloned()
        .unwrap_or_else(|| {
            builds::recommend(
                &champion.key,
                &champion.name,
                &champion.roles,
                requested_role,
            )
        });
        cs.build_role = profile.role;
        cs.primary_tree = profile.primary_tree;
        cs.secondary_tree = profile.secondary_tree;
        cs.selected_perks = std::array::from_fn(|index| {
            profile
                .rune_ids
                .get(index)
                .copied()
                .map(i64::from)
                .unwrap_or(runes::SHARD_ADAPTIVE_FORCE)
        });
        cs.secondary_rune_rows = runes::secondary_rows_for_perks(
            cs.secondary_tree,
            cs.selected_perks[4],
            cs.selected_perks[5],
        );
        runes::normalize_rune_setup(
            cs.primary_tree,
            cs.secondary_tree,
            &mut cs.secondary_rune_rows,
            &mut cs.selected_perks,
        );
        (cs.spell_one, cs.spell_two) = preferred_spell_order(
            profile.spell_one,
            profile.spell_two,
            &store.settings.flash_slot_preference,
        );
    }

    if !allow_auto_import || snapshot.phase != GameflowPhase::ChampSelect {
        return;
    }

    let recommendation = builds::build_for_champion(champion.id, cs.build_role)
        .cloned()
        .unwrap_or_else(|| {
            builds::recommend(
                &champion.key,
                &champion.name,
                &champion.roles,
                Some(cs.build_role),
            )
        });

    if store.settings.auto_import_spells {
        commands.push(CoreCommand::ImportSpells {
            spell1: cs.spell_one,
            spell2: cs.spell_two,
        });
    }
    if store.settings.auto_import_runes {
        let page = runes::build_rune_page(
            &format!("RiftCore {}", champion.name),
            cs.primary_tree,
            cs.secondary_tree,
            &cs.selected_perks,
        );
        commands.push(CoreCommand::ImportRunes(page.to_json()));
    }
    if store.settings.auto_import_item_sets {
        let set = builds::champion_item_set_json(champion, &recommendation);
        commands.push(CoreCommand::ExportItemSet(set));
    }
}

/// Dispatches rendering to the currently active page module.
pub fn render_active_page(ctx: &mut PageContext<'_>, ui: &mut egui::Ui) {
    match ctx.state.active_page {
        Page::ChampSelect => champ_select::render(ctx, ui),
        Page::Draft => draft::render(ctx, ui),
        Page::Lobby => lobby::render(ctx, ui),
        Page::Hud => hud_page::render(ctx, ui),
        Page::Tft => tft::render(ctx, ui),
        Page::Modes => modes::render(ctx, ui),
        Page::TierList => tier_list::render(ctx, ui),
        Page::ChampionPool => champion_pool::render(ctx, ui),
        Page::Profile => profile::render(ctx, ui),
        Page::Settings => settings::render(ctx, ui),
    }
}

/// Extracts up to 5 champion IDs from an LCU champ-select `myTeam`/`theirTeam` array.
pub fn session_picks(session: &Value, team_key: &str) -> [u32; 5] {
    let mut picks = [0; 5];
    if let Some(members) = session.get(team_key).and_then(Value::as_array) {
        for (index, member) in members.iter().take(5).enumerate() {
            let champion_id = member
                .get("championId")
                .and_then(Value::as_u64)
                .filter(|&id| id > 0)
                .or_else(|| {
                    member
                        .get("championPickIntent")
                        .and_then(Value::as_u64)
                        .filter(|&id| id > 0)
                })
                .unwrap_or(0) as u32;
            picks[index] = champion_id;
        }
    }
    picks
}

/// Extracts the local player's hovered/locked champion ID from an LCU champ-select session.
pub fn champ_select_local_champion(session: &Value) -> Option<u32> {
    let local_cell = session.get("localPlayerCellId").and_then(Value::as_u64)?;
    let selected_from_member = session
        .get("myTeam")
        .and_then(Value::as_array)
        .and_then(|team| {
            team.iter()
                .find(|member| member.get("cellId").and_then(Value::as_u64) == Some(local_cell))
        })
        .and_then(|member| {
            member
                .get("championId")
                .and_then(Value::as_u64)
                .filter(|champion_id| *champion_id > 0)
                .or_else(|| {
                    member
                        .get("championPickIntent")
                        .and_then(Value::as_u64)
                        .filter(|champion_id| *champion_id > 0)
                })
        });
    if let Some(id) = selected_from_member {
        return Some(id as u32);
    }

    session
        .get("actions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|group| {
            group
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or(std::slice::from_ref(group))
        })
        .find(|action| {
            action.get("actorCellId").and_then(Value::as_u64) == Some(local_cell)
                && action.get("type").and_then(Value::as_str) == Some("pick")
        })
        .and_then(|action| action.get("championId"))
        .and_then(Value::as_u64)
        .filter(|champion_id| *champion_id > 0)
        .map(|champion_id| champion_id as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn switching_champion_via_select_setup_never_corrupts_previous_or_new_setup() {
        let mut store = SettingsStore::load(PathBuf::new());
        // Save a custom Ahri (103) setup
        let ahri_custom = ChampionSetup {
            role: "MID".to_string(),
            primary_tree: runes::TREE_DOMINATION,
            secondary_tree: runes::TREE_PRECISION,
            perks: [8112, 8126, 8140, 8106, 9111, 8014, 5008, 5008, 5001],
            secondary_rows: [1, 3],
            spell_one: 14,
            spell_two: 4,
        };
        store.overwrite_champion_setup(103, ahri_custom.clone());

        let snapshot = CoreSnapshot::default();
        let mut state = AppUiState::default();
        let mut commands = Vec::new();

        // Select Ahri (loads custom setup)
        let ahri = champions::champion_by_id(103).unwrap();
        select_champion_setup_inner(
            &snapshot,
            &store,
            &mut state,
            &mut commands,
            ahri,
            None,
            false,
        );
        assert_eq!(state.champ_select.selected_champion, 103);
        assert_eq!(state.champ_select.secondary_tree, runes::TREE_PRECISION);

        // Navigate to Udyr (77) in Jungle without mutating Ahri or writing Ahri runes onto Udyr
        let udyr = champions::champion_by_id(77).unwrap();
        select_champion_setup_inner(
            &snapshot,
            &store,
            &mut state,
            &mut commands,
            udyr,
            Some(Role::Jungle),
            false,
        );
        assert_eq!(state.champ_select.selected_champion, 77);
        assert_eq!(state.champ_select.build_role, Role::Jungle);
        assert_eq!(state.champ_select.spell_two, 4); // Flash on F by default
        assert_eq!(state.champ_select.spell_one, 11); // Smite on D
        assert!(store.get_champion_setup(77).is_none());
        assert_eq!(store.get_champion_setup(103), Some(&ahri_custom));
    }
}
