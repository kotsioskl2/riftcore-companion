use crate::domain::live_game::LiveGameEvent;
use crate::domain::protocol::{
    CoreCommand, CoreEvent, CoreNotice, CoreSnapshot, GameflowPhase, KdaSummary,
};
use crate::platform::window::{
    find_league_game_window, is_league_window_foreground, LeagueWindowInfo,
};
use crate::services::lcu::{
    discover_credentials, is_riot_client_open, launch_league, set_league_window_mode, LcuClient,
    LcuCredentials,
};
use crate::services::live_game::{LiveGameClient, LIVE_GAME_POLL_INTERVAL};
use crate::services::riot_api::RiotApiClient;
use crate::services::updater::{spawn_self_update_helper, UpdaterService};
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, watch, Mutex};
use tokio::time::{interval, MissedTickBehavior};

pub const WINDOW_TRACKER_INTERVAL: Duration = Duration::from_millis(250);
const LCU_TICK_INTERVAL: Duration = Duration::from_millis(250);
const LCU_DISCOVERY_INTERVAL: Duration = Duration::from_secs(2);
const LCU_PROFILE_INTERVAL: Duration = Duration::from_secs(12);
const LCU_HISTORY_INTERVAL: Duration = Duration::from_secs(45);
const LCU_CHAMP_SELECT_INTERVAL: Duration = Duration::from_millis(800);
const LCU_READY_CHECK_INTERVAL: Duration = Duration::from_millis(500);

const COMMAND_CHANNEL_CAP: usize = 64;
const SNAPSHOT_CHANNEL_CAP: usize = 32;
const EVENT_CHANNEL_CAP: usize = 128;
const WINDOW_CHANNEL_CAP: usize = 32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowTrackerState {
    pub window: Option<LeagueWindowInfo>,
    pub is_foreground: bool,
}

/// Off-UI 250ms League game window tracker.
/// Runs Win32 window enumeration inside `tokio::task::spawn_blocking` so it never
/// blocks the UI thread or stalls the async runtime.
pub struct WindowTracker;

impl WindowTracker {
    pub fn spawn(
        window_tx: mpsc::Sender<WindowTrackerState>,
        mut shutdown_rx: watch::Receiver<bool>,
        notify_ui: Arc<dyn Fn() + Send + Sync>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = interval(WINDOW_TRACKER_INTERVAL);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
            let mut last_state: Option<WindowTrackerState> = None;

            loop {
                tokio::select! {
                    changed = shutdown_rx.changed() => {
                        if changed.is_err() || *shutdown_rx.borrow() {
                            break;
                        }
                    }
                    _ = ticker.tick() => {
                        let polled = tokio::task::spawn_blocking(|| {
                            let window = find_league_game_window();
                            let is_foreground = window
                                .map(is_league_window_foreground)
                                .unwrap_or(false);
                            WindowTrackerState { window, is_foreground }
                        })
                        .await;

                        if let Ok(state) = polled {
                            if last_state != Some(state) {
                                last_state = Some(state);
                                let _ = window_tx.try_send(state);
                                notify_ui();
                            }
                        }
                    }
                }
            }
        })
    }
}

#[derive(Default)]
struct WorkerShared {
    snapshot: CoreSnapshot,
    lcu: Option<LcuClient>,
    auto_accept: bool,
    did_auto_accept: bool,
    latest_release: Option<Value>,
    last_champ_select: Option<(u32, String)>,
    live_active: bool,
    seen_event_keys: HashSet<u64>,
    last_live_mode: String,
    last_live_map: String,
    last_live_duration: u64,
}

impl WorkerShared {
    fn new(auto_accept: bool) -> Self {
        Self {
            auto_accept,
            ..Default::default()
        }
    }

    fn touch_timestamp(&mut self) {
        self.snapshot.last_updated_epoch_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_millis() as u64);
    }
}

#[derive(Clone)]
struct DispatchContext {
    shared: Arc<Mutex<WorkerShared>>,
    snapshot_tx: mpsc::Sender<CoreSnapshot>,
    event_tx: mpsc::Sender<CoreEvent>,
    notify_ui: Arc<dyn Fn() + Send + Sync>,
    riot_api: RiotApiClient,
    updater: UpdaterService,
    refresh_tx: watch::Sender<u64>,
    shutdown_tx: watch::Sender<bool>,
}

impl DispatchContext {
    async fn publish_snapshot(&self, notice: Option<CoreNotice>) {
        let snap = {
            let mut guard = self.shared.lock().await;
            if notice.is_some() {
                guard.snapshot.notice = notice.clone();
            }
            guard.touch_timestamp();
            guard.snapshot.clone()
        };
        if let Some(n) = notice {
            let _ = self.event_tx.try_send(CoreEvent::Notice(n));
        }
        let _ = self.snapshot_tx.try_send(snap);
        (self.notify_ui)();
    }

    fn emit_event(&self, event: CoreEvent) {
        let _ = self.event_tx.try_send(event);
        (self.notify_ui)();
    }
}

/// Non-blocking background service coordinator managing LCU polling, 250ms Live Client Data
/// polling, 250ms off-UI WindowTracker, Riot API lookups, and verified self-updates via
/// `tokio::spawn` tasks and `tokio::sync::mpsc` channels.
pub struct BackgroundWorker {
    cmd_tx: mpsc::Sender<CoreCommand>,
    snapshot_rx: mpsc::Receiver<CoreSnapshot>,
    event_rx: mpsc::Receiver<CoreEvent>,
    window_rx: mpsc::Receiver<WindowTrackerState>,
    shutdown_tx: watch::Sender<bool>,
    thread_handle: Option<JoinHandle<()>>,
}

impl BackgroundWorker {
    pub fn start(notify_ui: Arc<dyn Fn() + Send + Sync>) -> Result<Self, std::io::Error> {
        let (cmd_tx, cmd_rx) = mpsc::channel(COMMAND_CHANNEL_CAP);
        let (snapshot_tx, snapshot_rx) = mpsc::channel(SNAPSHOT_CHANNEL_CAP);
        let (event_tx, event_rx) = mpsc::channel(EVENT_CHANNEL_CAP);
        let (window_tx, window_rx) = mpsc::channel(WINDOW_CHANNEL_CAP);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let shutdown_tx_thread = shutdown_tx.clone();

        let thread_handle = std::thread::Builder::new()
            .name("riftcore-background-worker".to_string())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(err) => {
                        let notice = CoreNotice::LcuError(format!(
                            "Unable to start background runtime: {err}"
                        ));
                        let _ = event_tx.try_send(CoreEvent::Notice(notice.clone()));
                        let _ = snapshot_tx.try_send(CoreSnapshot {
                            notice: Some(notice),
                            ..Default::default()
                        });
                        notify_ui();
                        return;
                    }
                };

                runtime.block_on(async move {
                    run_supervisor(
                        cmd_rx,
                        snapshot_tx,
                        event_tx,
                        window_tx,
                        shutdown_tx_thread,
                        shutdown_rx,
                        notify_ui,
                    )
                    .await;
                });
            })?;

        Ok(Self {
            cmd_tx,
            snapshot_rx,
            event_rx,
            window_rx,
            shutdown_tx,
            thread_handle: Some(thread_handle),
        })
    }

    pub fn start_headless() -> Result<Self, std::io::Error> {
        Self::start(Arc::new(|| {}))
    }

    pub fn command_sender(&self) -> mpsc::Sender<CoreCommand> {
        self.cmd_tx.clone()
    }

    /// Sends a command to the background worker without blocking the UI thread.
    pub fn try_send_command(&self, command: CoreCommand) -> Result<(), mpsc::error::TrySendError<CoreCommand>> {
        self.cmd_tx.try_send(command)
    }

    /// Non-blocking drain of the next published snapshot.
    pub fn try_recv_snapshot(&mut self) -> Option<CoreSnapshot> {
        self.snapshot_rx.try_recv().ok()
    }

    /// Non-blocking drain of the next discrete domain event.
    pub fn try_recv_event(&mut self) -> Option<CoreEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Non-blocking drain of the next `WindowTrackerState` update.
    pub fn try_recv_window(&mut self) -> Option<WindowTrackerState> {
        self.window_rx.try_recv().ok()
    }

    pub fn shutdown(&mut self) {
        let _ = self.shutdown_tx.send(true);
        let _ = self.cmd_tx.try_send(CoreCommand::Shutdown);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for BackgroundWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

async fn run_supervisor(
    cmd_rx: mpsc::Receiver<CoreCommand>,
    snapshot_tx: mpsc::Sender<CoreSnapshot>,
    event_tx: mpsc::Sender<CoreEvent>,
    window_tx: mpsc::Sender<WindowTrackerState>,
    shutdown_tx: watch::Sender<bool>,
    mut shutdown_rx: watch::Receiver<bool>,
    notify_ui: Arc<dyn Fn() + Send + Sync>,
) {
    let shared = Arc::new(Mutex::new(WorkerShared::new(true)));
    let (refresh_tx, refresh_rx) = watch::channel(0u64);

    let riot_api = RiotApiClient::new().unwrap_or_else(|_| {
        RiotApiClient::with_http_and_config(reqwest::Client::new(), Default::default())
    });
    let updater = UpdaterService::new().unwrap_or_else(|_| {
        UpdaterService::with_manifest_url(crate::services::updater::DEFAULT_MANIFEST_URL)
            .expect("static HTTPS URL is valid")
    });
    let live_client = LiveGameClient::new().unwrap_or_else(|_| {
        LiveGameClient::with_http_and_endpoint(
            reqwest::Client::new(),
            crate::services::live_game::LIVE_GAME_ENDPOINT,
        )
    });

    let ctx = DispatchContext {
        shared,
        snapshot_tx,
        event_tx,
        notify_ui: Arc::clone(&notify_ui),
        riot_api,
        updater,
        refresh_tx,
        shutdown_tx: shutdown_tx.clone(),
    };

    let cmd_task = tokio::spawn(run_command_loop(cmd_rx, ctx.clone(), shutdown_rx.clone()));
    let lcu_task = tokio::spawn(run_lcu_loop(ctx.clone(), refresh_rx, shutdown_rx.clone()));
    let live_task = tokio::spawn(run_live_game_loop(
        ctx.clone(),
        live_client,
        shutdown_rx.clone(),
    ));
    let window_task = WindowTracker::spawn(window_tx, shutdown_rx.clone(), notify_ui);

    while !*shutdown_rx.borrow() {
        if shutdown_rx.changed().await.is_err() {
            break;
        }
    }

    let _ = tokio::join!(cmd_task, lcu_task, live_task, window_task);
}

async fn run_command_loop(
    mut cmd_rx: mpsc::Receiver<CoreCommand>,
    ctx: DispatchContext,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    loop {
        let cmd = tokio::select! {
            changed = shutdown_rx.changed() => {
                if changed.is_err() || *shutdown_rx.borrow() {
                    break;
                }
                continue;
            }
            maybe_cmd = cmd_rx.recv() => {
                match maybe_cmd {
                    Some(cmd) => cmd,
                    None => break,
                }
            }
        };

        match cmd {
            CoreCommand::Shutdown => {
                let _ = ctx.shutdown_tx.send(true);
                break;
            }
            CoreCommand::SetAutoAccept(enabled) => {
                ctx.shared.lock().await.auto_accept = enabled;
            }
            CoreCommand::SetEnabled(enabled) => {
                {
                    let mut guard = ctx.shared.lock().await;
                    guard.snapshot.overlay_enabled = enabled;
                }
                ctx.publish_snapshot(None).await;
            }
            CoreCommand::SetHudMode(_)
            | CoreCommand::SetMousePassthrough(_)
            | CoreCommand::UpdateTarget { .. } => {
                // Handled primarily by overlay controller; accepted cleanly here.
            }
            CoreCommand::Refresh => {
                let next = *ctx.refresh_tx.borrow() + 1;
                let _ = ctx.refresh_tx.send(next);
            }
            CoreCommand::AcceptMatch => {
                let c = ctx.clone();
                tokio::spawn(async move {
                    let client = c.shared.lock().await.lcu.clone();
                    let notice = match client {
                        Some(lcu) => match lcu.accept_match().await {
                            Ok(()) => CoreNotice::Success("Match accepted".to_string()),
                            Err(err) => CoreNotice::LcuError(format!("Could not accept the match: {err}")),
                        },
                        None => CoreNotice::Warning("League Client is not connected".to_string()),
                    };
                    c.publish_snapshot(Some(notice)).await;
                });
            }
            CoreCommand::ImportSpells { spell1, spell2 } => {
                let c = ctx.clone();
                tokio::spawn(async move {
                    let client = c.shared.lock().await.lcu.clone();
                    let notice = match client {
                        Some(lcu) => match lcu.import_spells(spell1, spell2).await {
                            Ok(()) => CoreNotice::Success("Summoner spells sent to League Client".to_string()),
                            Err(err) => CoreNotice::LcuError(format!("Could not set summoner spells: {err}")),
                        },
                        None => CoreNotice::Warning("League Client is not connected".to_string()),
                    };
                    c.publish_snapshot(Some(notice)).await;
                });
            }
            CoreCommand::ImportRunes(page) => {
                let c = ctx.clone();
                tokio::spawn(async move {
                    let client = c.shared.lock().await.lcu.clone();
                    let notice = match client {
                        Some(lcu) => match lcu.import_runes(page).await {
                            Ok(msg) => CoreNotice::Success(msg),
                            Err(err) => CoreNotice::LcuError(format!("Could not import rune page: {err}")),
                        },
                        None => CoreNotice::Warning("League Client is not connected".to_string()),
                    };
                    c.publish_snapshot(Some(notice)).await;
                });
            }
            CoreCommand::ExportItemSet(item_set) => {
                let c = ctx.clone();
                tokio::spawn(async move {
                    let (client, cached_summoner) = {
                        let guard = c.shared.lock().await;
                        (guard.lcu.clone(), guard.snapshot.summoner.clone())
                    };
                    let notice = match client {
                        Some(lcu) => match lcu.export_item_set(item_set, cached_summoner.as_ref()).await {
                            Ok(msg) => CoreNotice::Success(msg),
                            Err(err) => CoreNotice::LcuError(format!("Could not export item set: {err}")),
                        },
                        None => CoreNotice::Warning("League Client is not connected".to_string()),
                    };
                    c.publish_snapshot(Some(notice)).await;
                });
            }
            CoreCommand::LookupRiotId {
                riot_id,
                region,
                api_key,
            } => {
                let c = ctx.clone();
                tokio::spawn(async move {
                    {
                        let mut guard = c.shared.lock().await;
                        guard.snapshot.riot_lookup = None;
                    }
                    c.publish_snapshot(Some(CoreNotice::Info(format!(
                        "Looking up {riot_id} through Riot API…"
                    ))))
                    .await;

                    match c.riot_api.lookup_account(&riot_id, &region, &api_key).await {
                        Ok(data) => {
                            {
                                let mut guard = c.shared.lock().await;
                                guard.snapshot.riot_lookup = Some(data);
                            }
                            c.emit_event(CoreEvent::RiotLookupCompleted {
                                riot_id,
                                ok: true,
                            });
                            c.publish_snapshot(Some(CoreNotice::Success(
                                "Riot account lookup complete".to_string(),
                            )))
                            .await;
                        }
                        Err(err) => {
                            let msg = err.to_string();
                            {
                                let mut guard = c.shared.lock().await;
                                guard.snapshot.riot_lookup = Some(serde_json::json!({ "error": msg }));
                            }
                            c.emit_event(CoreEvent::RiotLookupCompleted {
                                riot_id,
                                ok: false,
                            });
                            c.publish_snapshot(Some(CoreNotice::RiotApiError(msg)))
                                .await;
                        }
                    }
                });
            }
            CoreCommand::ValidateRiotApiKey { region, api_key } => {
                let c = ctx.clone();
                tokio::spawn(async move {
                    let status = match c.riot_api.validate_api_key(&region, &api_key).await {
                        Ok(msg) => msg,
                        Err(err) => err.to_string(),
                    };
                    {
                        let mut guard = c.shared.lock().await;
                        guard.snapshot.riot_api_status = Some(status);
                    }
                    c.publish_snapshot(None).await;
                });
            }
            CoreCommand::CheckUpdates => {
                let c = ctx.clone();
                tokio::spawn(async move {
                    {
                        let mut guard = c.shared.lock().await;
                        guard.snapshot.update_status = Some("Checking GitHub Releases…".to_string());
                    }
                    c.publish_snapshot(None).await;

                    match c.updater.check_for_updates().await {
                        Ok(check) => {
                            let current = env!("CARGO_PKG_VERSION");
                            let status = if check.is_newer {
                                if check.has_binary_asset && check.has_checksum_asset {
                                    format!("RiftCore {} is available", check.latest_tag)
                                } else {
                                    format!(
                                        "RiftCore {} is available; download setup from the release page",
                                        check.latest_tag
                                    )
                                }
                            } else {
                                format!("You have the latest release ({current})")
                            };
                            let notice = if check.is_newer {
                                Some(CoreNotice::UpdateAvailable {
                                    version: check.latest_tag.clone(),
                                    downloadable: check.has_binary_asset && check.has_checksum_asset,
                                })
                            } else {
                                None
                            };
                            {
                                let mut guard = c.shared.lock().await;
                                guard.latest_release = Some(check.release.clone());
                                guard.snapshot.latest_release = Some(check.release);
                                guard.snapshot.update_status = Some(status);
                            }
                            c.publish_snapshot(notice).await;
                        }
                        Err(err) => {
                            let msg = err.to_string();
                            {
                                let mut guard = c.shared.lock().await;
                                guard.snapshot.update_status = Some(msg.clone());
                                guard.snapshot.latest_release =
                                    Some(serde_json::json!({ "error": msg }));
                            }
                            c.publish_snapshot(None).await;
                        }
                    }
                });
            }
            CoreCommand::InstallUpdate => {
                let c = ctx.clone();
                tokio::spawn(async move {
                    let release = c.shared.lock().await.latest_release.clone();
                    {
                        let mut guard = c.shared.lock().await;
                        guard.snapshot.update_status =
                            Some("Downloading and verifying update package…".to_string());
                    }
                    c.publish_snapshot(None).await;

                    let Some(release) = release else {
                        let mut guard = c.shared.lock().await;
                        guard.snapshot.update_status =
                            Some("Check for updates before installing".to_string());
                        drop(guard);
                        c.publish_snapshot(None).await;
                        return;
                    };

                    match c.updater.download_and_verify(&release).await {
                        Ok(staged) => {
                            let helper_res = tokio::task::spawn_blocking(move || {
                                spawn_self_update_helper(&staged.staged_path)
                            })
                            .await;
                            let status = match helper_res {
                                Ok(Ok(())) => "Installing update and restarting…".to_string(),
                                Ok(Err(err)) => err.to_string(),
                                Err(err) => format!("Update helper task failed: {err}"),
                            };
                            {
                                let mut guard = c.shared.lock().await;
                                guard.snapshot.update_status = Some(status);
                            }
                            c.publish_snapshot(None).await;
                        }
                        Err(err) => {
                            let msg = err.to_string();
                            {
                                let mut guard = c.shared.lock().await;
                                guard.snapshot.update_status = Some(msg.clone());
                            }
                            c.publish_snapshot(Some(CoreNotice::Warning(msg))).await;
                        }
                    }
                });
            }
            CoreCommand::LaunchLeague => {
                let c = ctx.clone();
                tokio::spawn(async move {
                    let res = tokio::task::spawn_blocking(launch_league).await;
                    let notice = match res {
                        Ok(Ok(msg)) => CoreNotice::Success(msg),
                        Ok(Err(err)) => CoreNotice::Warning(err.to_string()),
                        Err(err) => CoreNotice::Warning(format!("Launch task failed: {err}")),
                    };
                    c.publish_snapshot(Some(notice)).await;
                });
            }
            CoreCommand::SetLeagueWindowMode(mode) => {
                let c = ctx.clone();
                tokio::spawn(async move {
                    let install_dir = c
                        .shared
                        .lock()
                        .await
                        .lcu
                        .as_ref()
                        .and_then(|lcu| lcu.credentials().install_dir.clone());
                    let res = tokio::task::spawn_blocking(move || {
                        set_league_window_mode(mode, install_dir.as_deref())
                    })
                    .await;
                    let notice = match res {
                        Ok(Ok(msg)) => CoreNotice::Success(msg),
                        Ok(Err(err)) => CoreNotice::Warning(err.to_string()),
                        Err(err) => CoreNotice::Warning(format!("Display update failed: {err}")),
                    };
                    c.publish_snapshot(Some(notice)).await;
                });
            }
        }
    }
}

async fn run_lcu_loop(
    ctx: DispatchContext,
    mut refresh_rx: watch::Receiver<u64>,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    let mut ticker = interval(LCU_TICK_INTERVAL);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let shared_http = LcuClient::build_http_client().unwrap_or_default();
    let mut last_discovery = Instant::now() - LCU_DISCOVERY_INTERVAL;
    let mut last_profile = Instant::now() - LCU_PROFILE_INTERVAL;
    let mut last_history = Instant::now() - LCU_HISTORY_INTERVAL;
    let mut last_champ_select = Instant::now() - LCU_CHAMP_SELECT_INTERVAL;
    let mut last_ready_check = Instant::now() - LCU_READY_CHECK_INTERVAL;
    let mut cached_creds: Option<LcuCredentials> = None;
    let mut cached_client: Option<LcuClient> = None;

    loop {
        tokio::select! {
            changed = shutdown_rx.changed() => {
                if changed.is_err() || *shutdown_rx.borrow() {
                    break;
                }
            }
            changed = refresh_rx.changed() => {
                if changed.is_ok() {
                    last_discovery = Instant::now() - LCU_DISCOVERY_INTERVAL;
                    last_profile = Instant::now() - LCU_PROFILE_INTERVAL;
                    last_history = Instant::now() - LCU_HISTORY_INTERVAL;
                }
            }
            _ = ticker.tick() => {}
        }

        if *shutdown_rx.borrow() {
            break;
        }

        let mut dirty = false;

        if last_discovery.elapsed() >= LCU_DISCOVERY_INTERVAL {
            let discovered = tokio::task::spawn_blocking(|| {
                (discover_credentials(), is_riot_client_open())
            })
            .await;
            last_discovery = Instant::now();

            if let Ok((new_creds, riot_open)) = discovered {
                let was_connected = cached_creds.is_some();
                let creds_changed = match (&cached_creds, &new_creds) {
                    (Some(a), Some(b)) => {
                        a.port != b.port || a.pid != b.pid || a.password() != b.password()
                    }
                    (None, None) => false,
                    _ => true,
                };

                if creds_changed {
                    cached_creds = new_creds.clone();
                    cached_client = new_creds
                        .as_ref()
                        .map(|c| LcuClient::with_http_client(shared_http.clone(), c.clone()));

                    let mut guard = ctx.shared.lock().await;
                    guard.lcu = cached_client.clone();
                    guard.snapshot.connected = cached_client.is_some();
                    if cached_client.is_none() {
                        let prev_phase = guard.snapshot.phase;
                        guard.snapshot.phase = GameflowPhase::None;
                        guard.snapshot.phase_raw = "None".to_string();
                        guard.snapshot.summoner = None;
                        guard.snapshot.ranked = None;
                        guard.snapshot.match_history = None;
                        guard.snapshot.champ_select = None;
                        guard.snapshot.ready_check = None;
                        guard.snapshot.summoner_name.clear();
                        guard.snapshot.summoner_tag.clear();
                        guard.snapshot.summoner_puuid.clear();
                        guard.did_auto_accept = false;
                        guard.last_champ_select = None;
                        drop(guard);
                        if was_connected {
                            ctx.emit_event(CoreEvent::ClientDisconnected);
                            if prev_phase != GameflowPhase::None {
                                ctx.emit_event(CoreEvent::PhaseChanged {
                                    from: prev_phase,
                                    to: GameflowPhase::None,
                                });
                            }
                        }
                    } else {
                        drop(guard);
                        // Reset profile/history timers immediately on reconnect.
                        last_profile = Instant::now() - LCU_PROFILE_INTERVAL;
                        last_history = Instant::now() - LCU_HISTORY_INTERVAL;
                        if !was_connected {
                            ctx.emit_event(CoreEvent::ClientConnected);
                        }
                    }
                    dirty = true;
                }

                let mut guard = ctx.shared.lock().await;
                if guard.snapshot.riot_client_open != riot_open {
                    guard.snapshot.riot_client_open = riot_open;
                    dirty = true;
                }
            }
        }

        let Some(ref lcu) = cached_client else {
            if dirty {
                ctx.publish_snapshot(None).await;
            }
            continue;
        };

        if let Ok(phase_str) = lcu.gameflow_phase().await {
            let new_phase = GameflowPhase::from_lcu(&phase_str);
            let mut guard = ctx.shared.lock().await;
            let live_override = guard.live_active;
            if !live_override && guard.snapshot.phase != new_phase {
                let prev = guard.snapshot.phase;
                guard.snapshot.phase = new_phase;
                guard.snapshot.phase_raw = phase_str;
                drop(guard);
                ctx.emit_event(CoreEvent::PhaseChanged {
                    from: prev,
                    to: new_phase,
                });
                dirty = true;
            } else if !live_override && guard.snapshot.phase_raw != phase_str {
                guard.snapshot.phase_raw = phase_str;
                dirty = true;
            }
        }

        if last_profile.elapsed() >= LCU_PROFILE_INTERVAL {
            let (summoner_res, ranked_res) =
                tokio::join!(lcu.current_summoner(), lcu.ranked_stats());
            last_profile = Instant::now();

            let mut guard = ctx.shared.lock().await;
            if let Ok(summoner) = summoner_res {
                let (name, tag, puuid) = extract_summoner_identity(&summoner);
                guard.snapshot.summoner_name = name;
                guard.snapshot.summoner_tag = tag;
                guard.snapshot.summoner_puuid = puuid;
                guard.snapshot.summoner = Some(summoner);
                dirty = true;
            }
            if let Ok(ranked) = ranked_res {
                guard.snapshot.ranked = Some(ranked);
                dirty = true;
            }
        }

        if last_history.elapsed() >= LCU_HISTORY_INTERVAL {
            let puuid = ctx.shared.lock().await.snapshot.summoner_puuid.clone();
            if !puuid.is_empty() {
                last_history = Instant::now();
                if let Ok(history) = lcu.match_history(&puuid, 0, 10).await {
                    let count = history
                        .get("games")
                        .and_then(|g| g.get("games"))
                        .and_then(Value::as_array)
                        .map(|a| a.len())
                        .unwrap_or(0);
                    ctx.shared.lock().await.snapshot.match_history = Some(history);
                    ctx.emit_event(CoreEvent::MatchHistoryUpdated { count });
                    dirty = true;
                }
            }
        }

        let current_phase = ctx.shared.lock().await.snapshot.phase;
        if current_phase == GameflowPhase::ChampSelect {
            if last_champ_select.elapsed() >= LCU_CHAMP_SELECT_INTERVAL {
                last_champ_select = Instant::now();
                if let Ok(maybe_session) = lcu.champ_select_session().await {
                    let mut guard = ctx.shared.lock().await;
                    let next_key = maybe_session.as_ref().map(extract_champ_select_local);
                    if guard.snapshot.champ_select != maybe_session {
                        guard.snapshot.champ_select = maybe_session;
                        dirty = true;
                    }
                    if let Some((champ_id, role)) = next_key {
                        if guard.last_champ_select.as_ref() != Some(&(champ_id, role.clone())) {
                            guard.last_champ_select = Some((champ_id, role.clone()));
                            guard.snapshot.champion_id = champ_id;
                            drop(guard);
                            ctx.emit_event(CoreEvent::ChampSelectChanged {
                                local_champion_id: champ_id,
                                assigned_role: role,
                            });
                        }
                    }
                }
            }
        } else {
            let mut guard = ctx.shared.lock().await;
            if guard.snapshot.champ_select.is_some() || guard.last_champ_select.is_some() {
                guard.snapshot.champ_select = None;
                guard.last_champ_select = None;
                dirty = true;
            }
            last_champ_select = Instant::now() - LCU_CHAMP_SELECT_INTERVAL;
        }

        if current_phase == GameflowPhase::ReadyCheck {
            if last_ready_check.elapsed() >= LCU_READY_CHECK_INTERVAL {
                last_ready_check = Instant::now();
                if let Ok(maybe_rc) = lcu.ready_check().await {
                    let awaiting = maybe_rc.as_ref().is_some_and(|data| {
                        data.get("state").and_then(Value::as_str) == Some("InProgress")
                            && data.get("playerResponse").and_then(Value::as_str) == Some("None")
                    });
                    let mut should_auto_accept = false;
                    let mut popped_now = false;
                    {
                        let mut guard = ctx.shared.lock().await;
                        if guard.snapshot.ready_check != maybe_rc {
                            guard.snapshot.ready_check = maybe_rc;
                            dirty = true;
                        }
                        if awaiting && !guard.did_auto_accept {
                            popped_now = true;
                            if guard.auto_accept {
                                guard.did_auto_accept = true;
                                should_auto_accept = true;
                            }
                        } else if !awaiting {
                            guard.did_auto_accept = false;
                        }
                    }
                    if popped_now {
                        ctx.emit_event(CoreEvent::ReadyCheckPopped);
                    }
                    if should_auto_accept {
                        let lcu_clone = lcu.clone();
                        let ctx_clone = ctx.clone();
                        tokio::spawn(async move {
                            let notice = match lcu_clone.accept_match().await {
                                Ok(()) => CoreNotice::Success("Match accepted".to_string()),
                                Err(err) => CoreNotice::LcuError(format!(
                                    "Could not accept the match: {err}"
                                )),
                            };
                            ctx_clone.publish_snapshot(Some(notice)).await;
                        });
                    }
                }
            }
        } else {
            let mut guard = ctx.shared.lock().await;
            if guard.snapshot.ready_check.is_some() || guard.did_auto_accept {
                guard.snapshot.ready_check = None;
                guard.did_auto_accept = false;
                dirty = true;
            }
            last_ready_check = Instant::now() - LCU_READY_CHECK_INTERVAL;
        }

        if dirty {
            ctx.publish_snapshot(None).await;
        }
    }
}

async fn run_live_game_loop(
    ctx: DispatchContext,
    live_client: LiveGameClient,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    let mut ticker = interval(LIVE_GAME_POLL_INTERVAL);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            changed = shutdown_rx.changed() => {
                if changed.is_err() || *shutdown_rx.borrow() {
                    break;
                }
            }
            _ = ticker.tick() => {}
        }

        if *shutdown_rx.borrow() {
            break;
        }

        match live_client.poll().await {
            Ok(Some(poll)) => {
                let mut events_to_emit = Vec::new();
                {
                    let mut guard = ctx.shared.lock().await;
                    let was_active = guard.live_active;
                    guard.live_active = true;
                    guard.last_live_mode = poll.state.game_mode.clone();
                    guard.last_live_map = poll.state.map_name.clone();
                    guard.last_live_duration = poll.state.game_time_secs.max(0.0) as u64;

                    let prev_phase = guard.snapshot.phase;
                    guard.snapshot.phase = GameflowPhase::InProgress;
                    guard.snapshot.phase_raw = "InProgress".to_string();
                    guard.snapshot.champion_name = poll.state.active_player.champion_name.clone();
                    guard.snapshot.kda = KdaSummary {
                        kills: poll.state.active_player.scores.kills,
                        deaths: poll.state.active_player.scores.deaths,
                        assists: poll.state.active_player.scores.assists,
                    };
                    guard.snapshot.gold = poll.state.active_player.current_gold;
                    guard.snapshot.cs = poll.state.active_player.scores.creep_score;
                    guard.snapshot.game_time_secs = poll.state.game_time_secs;
                    guard.snapshot.game_mode = poll.state.game_mode.clone();
                    guard.snapshot.live_game = Some(poll.raw);

                    if !was_active {
                        if prev_phase != GameflowPhase::InProgress {
                            events_to_emit.push(CoreEvent::PhaseChanged {
                                from: prev_phase,
                                to: GameflowPhase::InProgress,
                            });
                        }
                        events_to_emit.push(CoreEvent::GameStarted {
                            game_mode: poll.state.game_mode.clone(),
                            map_name: poll.state.map_name.clone(),
                        });
                    }

                    for (idx, ev) in poll.state.events.iter().enumerate() {
                        let key = ev.event_id.unwrap_or(1_000_000 + idx as u64);
                        if guard.seen_event_keys.insert(key) {
                            if let Some(domain_ev) = map_live_event(ev) {
                                events_to_emit.push(domain_ev);
                            }
                        }
                    }
                }
                for ev in events_to_emit {
                    ctx.emit_event(ev);
                }
                ctx.publish_snapshot(None).await;
            }
            Ok(None) => {
                let ended_duration = {
                    let mut guard = ctx.shared.lock().await;
                    if !guard.live_active {
                        None
                    } else {
                        let duration = guard.last_live_duration;
                        guard.live_active = false;
                        guard.seen_event_keys.clear();
                        guard.snapshot.live_game = None;
                        guard.snapshot.champion_name.clear();
                        guard.snapshot.kda = KdaSummary::default();
                        guard.snapshot.gold = 0.0;
                        guard.snapshot.cs = 0;
                        guard.snapshot.game_time_secs = 0.0;
                        guard.snapshot.game_mode.clear();
                        if guard.lcu.is_none() {
                            guard.snapshot.phase = GameflowPhase::None;
                            guard.snapshot.phase_raw = "None".to_string();
                        }
                        Some(duration)
                    }
                };
                if let Some(duration_secs) = ended_duration {
                    ctx.emit_event(CoreEvent::GameEnded {
                        win: None,
                        duration_secs,
                    });
                    ctx.publish_snapshot(None).await;
                }
            }
            Err(_) => {
                // Transient transport error during active game poll; retry on next 250ms tick.
            }
        }
    }
}

fn extract_summoner_identity(summoner: &Value) -> (String, String, String) {
    let name = summoner
        .get("gameName")
        .or_else(|| summoner.get("displayName"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let tag = summoner
        .get("tagLine")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let puuid = summoner
        .get("puuid")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    (name, tag, puuid)
}

fn extract_champ_select_local(session: &Value) -> (u32, String) {
    let Some(local_cell) = session.get("localPlayerCellId").and_then(Value::as_u64) else {
        return (0, String::new());
    };
    if let Some(team) = session.get("myTeam").and_then(Value::as_array) {
        if let Some(member) = team
            .iter()
            .find(|m| m.get("cellId").and_then(Value::as_u64) == Some(local_cell))
        {
            let champ_id = member
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
            let role = member
                .get("assignedPosition")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_ascii_uppercase();
            if champ_id > 0 || !role.is_empty() {
                return (champ_id, role);
            }
        }
    }
    let action_champ = session
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
        .filter(|id| *id > 0)
        .unwrap_or(0) as u32;
    (action_champ, String::new())
}

fn map_live_event(ev: &LiveGameEvent) -> Option<CoreEvent> {
    match ev.event_name.as_str() {
        "ChampionKill" => Some(CoreEvent::PlayerKill {
            killer: ev.killer_name.clone().unwrap_or_default(),
            victim: ev.victim_name.clone().unwrap_or_default(),
            assisters: ev.assisters.clone(),
            game_time_secs: ev.event_time,
        }),
        "DragonKill" | "BaronKill" | "HeraldKill" | "HordeKill" => {
            Some(CoreEvent::ObjectiveTaken {
                objective: ev.event_name.clone(),
                dragon_type: ev.dragon_type.clone(),
                killer: ev.killer_name.clone().unwrap_or_default(),
                game_time_secs: ev.event_time,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_worker_handles_nonblocking_commands_and_shuts_down_cleanly() {
        let mut worker = BackgroundWorker::start_headless().expect("worker starts");
        worker
            .try_send_command(CoreCommand::SetAutoAccept(false))
            .expect("send SetAutoAccept");
        worker
            .try_send_command(CoreCommand::SetEnabled(true))
            .expect("send SetEnabled");

        let start = Instant::now();
        let mut got_enabled_snapshot = false;
        while start.elapsed() < Duration::from_secs(2) {
            if let Some(snap) = worker.try_recv_snapshot() {
                if snap.overlay_enabled {
                    got_enabled_snapshot = true;
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(15));
        }
        assert!(got_enabled_snapshot);
        worker.shutdown();
    }

    #[test]
    fn window_tracker_emits_initial_state_and_stops_on_shutdown() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let (tx, mut rx) = mpsc::channel(8);
            let (shutdown_tx, shutdown_rx) = watch::channel(false);
            let handle = WindowTracker::spawn(tx, shutdown_rx, Arc::new(|| {}));

            let state = tokio::time::timeout(Duration::from_secs(2), rx.recv())
                .await
                .expect("window tracker emits within 2s")
                .expect("channel open");
            let _ = state.is_foreground;

            let _ = shutdown_tx.send(true);
            tokio::time::timeout(Duration::from_secs(1), handle)
                .await
                .expect("window tracker shuts down promptly")
                .expect("task clean");
        });
    }
}
