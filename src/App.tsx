import React, { useState, useEffect } from 'react';
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
import { SummonerProfileView } from './components/SummonerProfileView';
import { ChampionTierListView } from './components/ChampionTierListView';
import { SettingsModal } from './components/SettingsModal';
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
import { Check, Flame, ShieldAlert, Sparkles, Zap } from 'lucide-react';

export const App: React.FC = () => {
  const [isConnected, setIsConnected] = useState(false);
  const [isRiotClientReady, setIsRiotClientReady] = useState(false);
  const [gameflowPhase, setGameflowPhase] = useState<GameflowPhase>('None');
  const [activeTab, setActiveTab] = useState<string>('champ-select');
  const [isCompactMode, setIsCompactMode] = useState(false);
  const [isPinned, setIsPinned] = useState(false);
  const [showCloseDialog, setShowCloseDialog] = useState(false);
  const [showUpdateNotes, setShowUpdateNotes] = useState(false);

  // League Data
  const [summoner, setSummoner] = useState<SummonerProfile | null>(null);
  const [rankedData, setRankedData] = useState<RankedData | null>(null);
  const [champSelectSession, setChampSelectSession] = useState<ChampSelectSession | null>(null);
  const [activeChampionId, setActiveChampionId] = useState<number>(420);
  const [liveData, setLiveData] = useState<LiveGameData | null>(null);
  const [readyCheckData, setReadyCheckData] = useState<any>(null);

  // App Settings
  const [settings, setSettings] = useState<AppSettings>({
    autoAccept: true,
    autoImportRunes: false,
    autoImportSpells: false,
    autoImportItemSets: true,
    soundAlerts: true,
    theme: 'hextech',
    fontFamily: 'plus-jakarta',
    alwaysOnTop: false,
    closeToTray: true,
    riotApiKey: 'RGAPI-184ff7f6-57c0-4ae5-a5ce-722df7c55cc9',
    riotRegion: 'eun1',
  });

  // Sync font preference to HTML root
  useEffect(() => {
    const font = settings.fontFamily || 'plus-jakarta';
    document.documentElement.setAttribute('data-font', font);
  }, [settings.fontFamily]);

  // Initialize and register electron event listeners
  useEffect(() => {
    if (!window.riftAPI) {
      console.warn('Running outside Electron environment');
      return;
    }

    // 1. Initial status check
    window.riftAPI.checkClientStatus().then((res) => {
      setIsConnected(res.connected);
      if (res.riotClient !== undefined) setIsRiotClientReady(res.riotClient);
      if (res.gameflowPhase) {
        setGameflowPhase(res.gameflowPhase);
        if (res.gameflowPhase === 'ChampSelect') {
          setActiveTab('champ-select');
        } else {
          setChampSelectSession(null);
          if (res.gameflowPhase === 'InProgress' || res.gameflowPhase === 'GameStart') {
            setIsCompactMode(true);
            window.riftAPI?.setCompactHud?.(true);
            window.riftAPI?.setAlwaysOnTop?.(true).then((r) => {
              if (r) setIsPinned(r.isPinned);
            });
          }
        }
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

    // Check if app was updated to v1.1.0
    try {
      const seenVer = localStorage.getItem('riftcore_seen_version');
      if (seenVer !== '1.1.0') {
        setShowUpdateNotes(true);
        localStorage.setItem('riftcore_seen_version', '1.1.0');
      }
    } catch {}

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
          setActiveTab('live-overlay');
          setReadyCheckData(null);
          setIsCompactMode(true);
          window.riftAPI?.setCompactHud?.(true);
          window.riftAPI?.setAlwaysOnTop?.(true).then((r) => {
            if (r) setIsPinned(r.isPinned);
          });
        } else if (phase === 'EndOfGame' || phase === 'None' || phase === 'Lobby') {
          setIsCompactMode(false);
          window.riftAPI?.setCompactHud?.(false);
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
        setIsCompactMode(true);
        window.riftAPI?.setCompactHud?.(true);
        window.riftAPI?.setAlwaysOnTop?.(true).then((r) => {
          if (r) setIsPinned(r.isPinned);
        });
      }
    });

    const handleCompactEvent = (e: any) => {
      const isCompact = Boolean(e.detail);
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
  }, []);

  // Direct In-Game Live Client Data poller (port 2999)
  useEffect(() => {
    let isSubscribed = true;
    const pollLiveData = async () => {
      if (!window.riftAPI?.getLiveClientData) return;
      try {
        const data = await window.riftAPI.getLiveClientData();
        if (isSubscribed && data && data.gameData) {
          setLiveData(data);
          setIsCompactMode(true);
          window.riftAPI?.setCompactHud?.(true);
          window.riftAPI?.setAlwaysOnTop?.(true).then((r) => {
            if (r) setIsPinned(r.isPinned);
          });
        }
      } catch {}
    };

    pollLiveData();
    const interval = setInterval(pollLiveData, 1000);
    return () => {
      isSubscribed = false;
      clearInterval(interval);
    };
  }, []);

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
        setIsCompactMode(res.isCompactMode);
        setIsPinned(res.isPinned);
        if (res.isCompactMode) {
          setActiveTab('live-overlay');
        }
      }
    } else {
      const next = !isCompactMode;
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
    } else {
      document.body.classList.remove('compact-mode');
      document.documentElement.classList.remove('compact-mode');
    }
  }, [isCompactMode]);

  const isMatchLive = gameflowPhase === 'InProgress' || gameflowPhase === 'GameStart' || Boolean(liveData?.gameData);

  // Auto-revert compact mode if outside an active League match (never trap the user in standby)
  useEffect(() => {
    if (isCompactMode && !isMatchLive) {
      setIsCompactMode(false);
      window.riftAPI?.setCompactHud?.(false);
    }
  }, [isCompactMode, isMatchLive]);

  // When in Compact In-Game Overlay Mode AND a match is actively live: Render Pure Floating Tactical HUD Widget
  if (isCompactMode && isMatchLive) {
    return (
      <div className="app-container compact-mode" style={{ background: 'transparent' }}>
        <RiftFloatingOverlay
          liveData={liveData}
          onExpand={handleToggleCompact}
          onMinimize={() => window.riftAPI?.windowControl('minimize')}
        />
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
            />
          )}
        </main>
      </div>
    </div>
  );
};
