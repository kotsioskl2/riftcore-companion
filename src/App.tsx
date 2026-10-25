import React, { useState, useEffect, useCallback, useRef } from 'react';
import { TitleBar } from './components/TitleBar';
import { Sidebar } from './components/Sidebar';
import { ChampSelectView, extractLcuChampionId } from './components/ChampSelectView';
import { DraftSimulatorView } from './components/DraftSimulatorView';
import { ChampionPoolBuilderView } from './components/ChampionPoolBuilderView';
import { GameModesHubView } from './components/GameModesHubView';
import { AramHubView } from './components/AramHubView';
import { LobbyScoutView } from './components/LobbyScoutView';
import { LiveGameOverlay } from './components/LiveGameOverlay';
import { RiftFloatingOverlay } from './components/RiftFloatingOverlay';
import { OverlayErrorBoundary } from './components/OverlayErrorBoundary';
import { SummonerProfileView } from './components/SummonerProfileView';
import { ChampionTierListView } from './components/ChampionTierListView';
import { SettingsModal } from './components/SettingsModal';
import { TftHubView } from './components/TftHubView';
import {
  GameflowPhase,
  SummonerProfile,
  RankedData,
  ChampSelectSession,
  LiveGameData,
  AppSettings
} from './types';
import { soundService } from './services/soundService';
import { CloseConfirmationModal } from './components/CloseConfirmationModal';
import { UpdateNotesModal } from './components/UpdateNotesModal';
import { QuickTourModal } from './components/QuickTourModal';
import { AppUpdateBanner } from './components/AppUpdateBanner';
import { AppUpdateInfo } from './types';
import { APP_VERSION } from './version';
import { Check, Flame, ShieldAlert, Sparkles, Zap } from 'lucide-react';

export const App: React.FC = () => {
  const [isConnected, setIsConnected] = useState(false);
  const [isRiotClientReady, setIsRiotClientReady] = useState(false);
  const [gameflowPhase, setGameflowPhase] = useState<GameflowPhase>('None');
  const [activeTab, setActiveTab] = useState<string>('champ-select');
  const [isCompactMode, setIsCompactMode] = useState(false);
  const compactModeRef = useRef(false);
  const [isPinned, setIsPinned] = useState(false);
  const [showCloseDialog, setShowCloseDialog] = useState(false);
  const [showUpdateNotes, setShowUpdateNotes] = useState(false);
  const [showQuickTour, setShowQuickTour] = useState(false);
  const [appUpdate, setAppUpdate] = useState<AppUpdateInfo | null>(null);

  // League Data
  const [summoner, setSummoner] = useState<SummonerProfile | null>(null);
  const [rankedData, setRankedData] = useState<RankedData | null>(null);
  const [champSelectSession, setChampSelectSession] = useState<ChampSelectSession | null>(null);
  const [activeChampionId, setActiveChampionId] = useState<number>(420);
  const [liveData, setLiveData] = useState<LiveGameData | null>(null);
  const [readyCheckData, setReadyCheckData] = useState<any>(null);

  const enterCompactMode = useCallback(() => {
    if (compactModeRef.current) return;
    compactModeRef.current = true;
    setIsCompactMode(true);
    setActiveTab('live-overlay');
    void window.riftAPI?.setCompactHud?.(true);
    window.riftAPI?.setAlwaysOnTop?.(true)?.then((r) => {
      if (r) setIsPinned(r.isPinned);
    });
  }, []);

  const exitCompactMode = useCallback(() => {
    const wasCompact = compactModeRef.current;
    compactModeRef.current = false;
    setIsCompactMode(false);
    if (wasCompact) void window.riftAPI?.setCompactHud?.(false);
  }, []);

  // App Settings
  const [settings, setSettings] = useState<AppSettings>({
    autoAccept: true,
    autoImportRunes: true,
    autoImportSpells: true,
    flashSlotPreference: 'F',
    autoImportItemSets: true,
    soundAlerts: true,
    theme: 'hextech',
    fontFamily: 'plus-jakarta',
    alwaysOnTop: false,
    closeToTray: true,
    riotApiKey: 'RGAPI-184ff7f6-57c0-4ae5-a5ce-722df7c55cc9',
    riotRegion: 'eun1',
    hudGoldLead: true,
    hudCsBenchmark: true,
    hudObjectiveTimers: true,
    hudSpellTimers: true,
    hudUltimateTimers: true,
    hudSkillOrder: true,
    hudItemGoal: true,
    hudTrinketReminder: true,
    hudMayhemCards: true,
    hudJunglePathing: true,
    hudAramHealthTimers: true,
  });

  // Sync font preference to HTML root
  useEffect(() => {
    const font = settings.fontFamily || 'plus-jakarta';
    document.documentElement.setAttribute('data-font', font);
  }, [settings.fontFamily]);

  // Initialize and register native event listeners
  useEffect(() => {
    if (!window.riftAPI) {
      console.warn('Running outside native desktop environment');
      return;
    }

    // 1. Initial status check
    window.riftAPI.checkClientStatus().then((res) => {
      setIsConnected(res.connected);
      if (res.riotClient !== undefined) setIsRiotClientReady(res.riotClient);
      if (res.connected && res.gameflowPhase && res.gameflowPhase !== 'None') {
        setGameflowPhase(res.gameflowPhase);
        if (res.gameflowPhase === 'ChampSelect') {
          setActiveTab('champ-select');
        } else {
          setChampSelectSession(null);
          if (res.gameflowPhase === 'InProgress' || res.gameflowPhase === 'GameStart') {
            enterCompactMode();
          }
        }
      } else {
        setGameflowPhase('None');
        setChampSelectSession(null);
        compactModeRef.current = false;
        setIsCompactMode(false);
      }
      if (res.connected) fetchInitialData();
    });

    // 2. Initial settings
    window.riftAPI.getSettings().then((s) => {
      if (s) {
        setSettings(s);
        soundService.setEnabled(s.soundAlerts);
      }
    });

    // Check if app was updated
    try {
      const seenVer = localStorage.getItem('riftcore_seen_version');
      if (seenVer !== APP_VERSION) {
        setShowUpdateNotes(true);
        localStorage.setItem('riftcore_seen_version', APP_VERSION);
      } else {
        const tourSeen = localStorage.getItem('riftcore_tour_seen');
        if (!tourSeen) {
          setShowQuickTour(true);
        }
      }
    } catch {}

    // Check for App Updates via GitHub Releases
    if (window.riftAPI?.checkForAppUpdate) {
      window.riftAPI.checkForAppUpdate()
        .then((info) => {
          if (info && info.hasUpdate) {
            console.log('[RiftCore] New update available on GitHub:', info);
            setAppUpdate(info);
          }
        })
        .catch((e) => {
          console.warn('[RiftCore] Initial update check error:', e);
        });
    }

    // 3. Listen to LCU Status
    const unsubLCU = window.riftAPI.onLCUStatusChange((status: any) => {
      setIsConnected(status.connected);
      if (status.riotClient !== undefined) setIsRiotClientReady(status.riotClient);
      if (status.connected) {
        fetchInitialData();
      } else {
        setSummoner(null);
        setRankedData(null);
        setChampSelectSession(null);
        setGameflowPhase('None');
      }
    });

    // 4. Listen to Gameflow Phase
    const unsubPhase = window.riftAPI.onGameflowPhase((phase) => {
      setGameflowPhase(phase);
      if (phase === 'ChampSelect') {
        setActiveTab('champ-select');
        setReadyCheckData(null);
      } else {
        setChampSelectSession(null);
        if (phase === 'ReadyCheck') {
          soundService.playQueuePop();
          if (settings.autoAccept && window.riftAPI?.acceptMatch) {
            console.log('[RiftCore] ReadyCheck phase detected! Auto-accepting match...');
            setTimeout(() => {
              window.riftAPI?.acceptMatch().then(() => {
                soundService.playAutoAccept();
              });
            }, 600);
          }
        } else if (phase === 'InProgress' || phase === 'GameStart') {
          setReadyCheckData(null);
          enterCompactMode();
        } else if (phase === 'EndOfGame' || phase === 'None' || phase === 'Lobby') {
          exitCompactMode();
        }
      }
    });

    // 5. Listen to Champ Select Session
    const unsubChamp = window.riftAPI.onChampSelectSession((session) => {
      setChampSelectSession(session);
      if (session) {
        const champId = extractLcuChampionId(session);
        if (champId > 0) {
          setActiveChampionId(champId);
        }
      }
    });

    // 6. Listen to Ready Check (Queue Pop)
    const unsubReady = window.riftAPI.onReadyCheck((rcData) => {
      setReadyCheckData(rcData);
      if (rcData && rcData.state === 'InProgress' && rcData.playerResponse === 'None') {
        soundService.playQueuePop();
        if (settings.autoAccept && window.riftAPI?.acceptMatch) {
          console.log('[RiftCore] ReadyCheck state in progress! Auto-accepting match...');
          setTimeout(() => {
            window.riftAPI?.acceptMatch().then(() => {
              soundService.playAutoAccept();
            });
          }, 500);
        }
      }
    });

    // 7. Listen to Live Game Update (Direct 2999 poller)
    const unsubLive = window.riftAPI.onLiveGameUpdate((gData) => {
      setLiveData(gData);
      if (gData && gData.gameData) {
        enterCompactMode();
      }
    });

    const handleCompactEvent = (e: any) => {
      const isCompact = Boolean(e.detail);
      compactModeRef.current = isCompact;
      setIsCompactMode(isCompact);
      if (isCompact) {
        setActiveTab('live-overlay');
      }
    };
    window.addEventListener('rift-compact-hud', handleCompactEvent);

    return () => {
      unsubLCU();
      unsubPhase();
      unsubChamp();
      unsubReady();
      unsubLive();
      window.removeEventListener('rift-compact-hud', handleCompactEvent);
    };
  }, [enterCompactMode, exitCompactMode]);

  // Direct In-Game Live Client Data poller (port 2999)
  useEffect(() => {
    let isSubscribed = true;
    const pollLiveData = async () => {
      if (!window.riftAPI?.getLiveClientData) return;
      try {
        const data = await window.riftAPI.getLiveClientData();
        if (isSubscribed && data && data.gameData) {
          setLiveData(data);
          enterCompactMode();
        }
      } catch {}
    };

    pollLiveData();
    const interval = setInterval(pollLiveData, 1000);
    return () => {
      isSubscribed = false;
      clearInterval(interval);
    };
  }, [enterCompactMode]);

  // Active session poller during ChampSelect phase (guarantees session freshness even if websocket dropped)
  useEffect(() => {
    if (gameflowPhase !== 'ChampSelect' || !window.riftAPI) {
      if (gameflowPhase !== 'ChampSelect') setChampSelectSession(null);
      return;
    }

    let isSubscribed = true;
    const pollSession = async () => {
      if (!window.riftAPI) return;
      try {
        const sess = await window.riftAPI.lcuRequest('/lol-champ-select/v1/session');
        if (isSubscribed && sess && !sess.errorCode) {
          setChampSelectSession(sess);
          const champId = extractLcuChampionId(sess);
          if (champId > 0) {
            setActiveChampionId(champId);
          }
        }
      } catch {}
    };

    pollSession();
    const interval = setInterval(pollSession, 1000);
    return () => {
      isSubscribed = false;
      clearInterval(interval);
    };
  }, [gameflowPhase]);

  const fetchInitialData = async () => {
    if (!window.riftAPI) return;
    try {
      const summ = await window.riftAPI.lcuRequest('/lol-summoner/v1/current-summoner');
      setSummoner(summ);

      const ranked = await window.riftAPI.lcuRequest('/lol-ranked/v1/current-ranked-stats');
      setRankedData(ranked);
    } catch (err) {
      console.error('Failed to fetch initial summoner/ranked data:', err);
    }
  };

  const handleCheckForUpdate = useCallback(async () => {
    if (!window.riftAPI?.checkForAppUpdate) return null;
    try {
      const info = await window.riftAPI.checkForAppUpdate();
      if (info) {
        setAppUpdate(info.hasUpdate ? info : null);
      }
      return info;
    } catch (e) {
      console.warn('[RiftCore] Check for update failed:', e);
      return null;
    }
  }, []);

  const handleUpdateSetting = async (key: keyof AppSettings, value: any) => {
    const updated = { ...settings, [key]: value };
    setSettings(updated);
    if (window.riftAPI) {
      await window.riftAPI.saveSettings(updated);
    }
  };

  const handleToggleCompact = async () => {
    if (window.riftAPI) {
      const res = await window.riftAPI.windowControl('toggle-compact');
      if (res) {
        compactModeRef.current = res.isCompactMode;
        setIsCompactMode(res.isCompactMode);
        setIsPinned(res.isPinned);
        if (res.isCompactMode) {
          setActiveTab('live-overlay');
        }
      }
    } else {
      const next = !isCompactMode;
      compactModeRef.current = next;
      setIsCompactMode(next);
      if (next) {
        setActiveTab('live-overlay');
      }
    }
  };

  const handleTogglePin = async () => {
    if (window.riftAPI) {
      const res = await window.riftAPI.windowControl('toggle-pin');
      if (res) {
        setIsPinned(res.isPinned);
      }
    } else {
      setIsPinned(!isPinned);
    }
  };

  const handleManualAccept = async () => {
    if (window.riftAPI) {
      await window.riftAPI.acceptMatch();
      soundService.playAutoAccept();
    }
    setReadyCheckData(null);
  };

  const isQueuePopActive =
    readyCheckData &&
    readyCheckData.state === 'InProgress' &&
    readyCheckData.playerResponse === 'None';

  const handleLaunchLeague = () => {
    if (window.riftAPI?.launchLeague) {
      window.riftAPI.launchLeague();
    }
  };

  const handleTitleBarClose = () => {
    if (settings.closeToTray === false) {
      window.riftAPI?.windowControl('exit-app');
      return;
    }
    const isDismissed = localStorage.getItem('riftcore_tray_hint_dismissed') === 'true';
    if (isDismissed) {
      window.riftAPI?.windowControl('close');
    } else {
      setShowCloseDialog(true);
    }
  };

  const handleConfirmMinimizeToTray = (remember: boolean) => {
    if (remember) {
      localStorage.setItem('riftcore_tray_hint_dismissed', 'true');
    }
    handleUpdateSetting('closeToTray', true);
    setShowCloseDialog(false);
    window.riftAPI?.windowControl('close');
  };

  const handleConfirmExitApp = () => {
    setShowCloseDialog(false);
    window.riftAPI?.windowControl('exit-app');
  };

  // Sync compact mode class to document body for transparency
  useEffect(() => {
    if (isCompactMode) {
      document.body.classList.add('compact-mode');
      document.documentElement.classList.add('compact-mode');
      document.body.style.backgroundColor = 'transparent';
      document.body.style.background = 'transparent';
      document.documentElement.style.backgroundColor = 'transparent';
      document.documentElement.style.background = 'transparent';
    } else {
      document.body.classList.remove('compact-mode');
      document.documentElement.classList.remove('compact-mode');
      document.body.style.backgroundColor = '';
      document.body.style.background = '';
      document.documentElement.style.backgroundColor = '';
      document.documentElement.style.background = '';
    }
  }, [isCompactMode]);

  const isMatchLive = gameflowPhase === 'InProgress' || gameflowPhase === 'GameStart' || Boolean(liveData?.gameData);

  // Auto-revert compact mode if outside an active League match (never trap the user in standby)
  useEffect(() => {
    if (isCompactMode && !isMatchLive) {
      exitCompactMode();
    }
  }, [isCompactMode, isMatchLive, exitCompactMode]);

  // When in Compact In-Game Overlay Mode AND a match is actively live: Render Pure Floating Tactical HUD Widget
  if (isCompactMode && isMatchLive) {
    return (
      <div className="app-container compact-mode" style={{ background: 'transparent' }}>
        <OverlayErrorBoundary onExpand={handleToggleCompact}>
          <RiftFloatingOverlay
            liveData={liveData}
            settings={settings}
            onExpand={handleToggleCompact}
            onMinimize={() => window.riftAPI?.windowControl('minimize')}
          />
        </OverlayErrorBoundary>
      </div>
    );
  }

  return (
    <div className="app-container">
      {/* Titlebar */}
      <TitleBar
        isConnected={isConnected}
        isRiotClientReady={isRiotClientReady}
        gameflowPhase={gameflowPhase}
        isCompactMode={isCompactMode}
        isPinned={isPinned}
        onToggleCompact={handleToggleCompact}
        onTogglePin={handleTogglePin}
        onLaunchLeague={handleLaunchLeague}
        onCloseClick={handleTitleBarClose}
        onOpenUpdateNotes={() => setShowUpdateNotes(true)}
        onOpenQuickTour={() => setShowQuickTour(true)}
      />

      {/* In-App Auto-Update Banner */}
      <AppUpdateBanner
        updateInfo={appUpdate}
        onDismiss={() => setAppUpdate(null)}
      />

      {/* Quick Tour Modal */}
      <QuickTourModal
        isOpen={showQuickTour}
        onClose={() => setShowQuickTour(false)}
      />

      {/* Update Notes / What's New Modal */}
      <UpdateNotesModal
        isOpen={showUpdateNotes}
        onClose={() => setShowUpdateNotes(false)}
      />

      {/* Close Confirmation Modal */}
      <CloseConfirmationModal
        isOpen={showCloseDialog}
        onMinimizeToTray={handleConfirmMinimizeToTray}
        onExitApp={handleConfirmExitApp}
        onCancel={() => setShowCloseDialog(false)}
      />

      {/* Queue Pop Notification Modal */}
      {isQueuePopActive && (
        <div className="queue-pop-modal" id="queue-pop-modal">
          <div style={{ display: 'flex', alignItems: 'center', gap: '14px' }}>
            <div
              style={{
                width: '42px',
                height: '42px',
                borderRadius: '50%',
                background: 'rgba(16, 185, 129, 0.15)',
                border: '2px solid #10b981',
                boxShadow: '0 0 16px rgba(16, 185, 129, 0.4)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                animation: 'pulse-blue 1.2s infinite',
              }}
            >
              <Sparkles size={20} color="#10b981" />
            </div>
            <div>
              <div style={{ fontSize: '15px', fontWeight: 900, color: '#fff', letterSpacing: '0.8px' }}>
                MATCH FOUND!
              </div>
              <div style={{ fontSize: '11px', color: '#94a3b8', marginTop: '1px' }}>
                {settings.autoAccept ? 'Auto-Accepting in progress...' : 'Ready Check Active'}
              </div>
            </div>
          </div>

          <button
            id="btn-modal-accept"
            className="btn-primary btn-accent"
            onClick={handleManualAccept}
            style={{ padding: '8px 22px', fontSize: '12px', fontWeight: 800, letterSpacing: '0.5px' }}
          >
            <Check size={15} />
            <span>ACCEPT MATCH</span>
          </button>
        </div>
      )}

      {/* Main App Content Layout */}
      <div className="app-content">
        {!isCompactMode && (
          <Sidebar
            activeTab={activeTab}
            setActiveTab={setActiveTab}
            settings={settings}
            onUpdateSetting={handleUpdateSetting}
          />
        )}

        {/* Viewport Content */}
        <main className="main-viewport" id="app-viewport">
          {activeTab === 'champ-select' && (
            <ChampSelectView
              session={champSelectSession}
              selectedChampionId={activeChampionId}
              onSelectChampion={setActiveChampionId}
              settings={settings}
              onUpdateSetting={handleUpdateSetting}
            />
          )}

          {activeTab === 'draft-simulator' && (
            <DraftSimulatorView
              session={champSelectSession}
              onSelectChampion={(champId) => {
                setActiveChampionId(champId);
                setActiveTab('champ-select');
              }}
            />
          )}

          {activeTab === 'lobby-scout' && (
            <LobbyScoutView
              session={champSelectSession}
              summoner={summoner}
              rankedData={rankedData}
            />
          )}

          {activeTab === 'live-overlay' && (
            <LiveGameOverlay
              liveData={liveData}
              settings={settings}
              isCompactMode={isCompactMode}
              isPinned={isPinned}
              onToggleCompact={handleToggleCompact}
              onTogglePin={handleTogglePin}
            />
          )}

          {activeTab === 'champion-pool' && (
            <ChampionPoolBuilderView
              onSelectChampion={(champId) => {
                setActiveChampionId(champId);
                setActiveTab('champ-select');
              }}
            />
          )}

          {(activeTab === 'game-modes' || activeTab === 'aram') && (
            <GameModesHubView
              initialMode={activeTab === 'aram' ? 'aram' : undefined}
              liveGameMode={liveData?.gameData?.gameMode}
              onSelectChampion={(champId) => {
                setActiveChampionId(champId);
                setActiveTab('champ-select');
              }}
            />
          )}

          {activeTab === 'tft-hub' && (
            <TftHubView />
          )}

          {activeTab === 'champions' && (
            <ChampionTierListView
              onSelectChampion={(champId) => {
                setActiveChampionId(champId);
                setActiveTab('champ-select');
              }}
            />
          )}

          {activeTab === 'profile' && (
            <SummonerProfileView
              summoner={summoner}
              rankedData={rankedData}
              onRefresh={fetchInitialData}
            />
          )}

          {activeTab === 'settings' && (
            <SettingsModal
              settings={settings}
              onUpdateSetting={handleUpdateSetting}
              isConnected={isConnected}
              onReconnect={fetchInitialData}
              appUpdate={appUpdate}
              onCheckForUpdate={handleCheckForUpdate}
            />
          )}
        </main>
      </div>
    </div>
  );
};
