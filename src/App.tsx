import React, { useState, useEffect } from 'react';
import { TitleBar } from './components/TitleBar';
import { Sidebar } from './components/Sidebar';
import { ChampSelectView } from './components/ChampSelectView';
import { DraftSimulatorView } from './components/DraftSimulatorView';
import { ChampionPoolBuilderView } from './components/ChampionPoolBuilderView';
import { GameModesHubView } from './components/GameModesHubView';
import { AramHubView } from './components/AramHubView';
import { LobbyScoutView } from './components/LobbyScoutView';
import { LiveGameOverlay } from './components/LiveGameOverlay';
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
import { Check, Flame, ShieldAlert, Sparkles, Zap } from 'lucide-react';

export const App: React.FC = () => {
  const [isConnected, setIsConnected] = useState(false);
  const [isRiotClientReady, setIsRiotClientReady] = useState(false);
  const [gameflowPhase, setGameflowPhase] = useState<GameflowPhase>('None');
  const [activeTab, setActiveTab] = useState<string>('champ-select');
  const [isCompactMode, setIsCompactMode] = useState(false);
  const [isPinned, setIsPinned] = useState(false);

  // League Data
  const [summoner, setSummoner] = useState<SummonerProfile | null>(null);
  const [rankedData, setRankedData] = useState<RankedData | null>(null);
  const [champSelectSession, setChampSelectSession] = useState<ChampSelectSession | null>(null);
  const [liveData, setLiveData] = useState<LiveGameData | null>(null);
  const [readyCheckData, setReadyCheckData] = useState<any>(null);

  // App Settings
  const [settings, setSettings] = useState<AppSettings>({
    autoAccept: true,
    autoImportRunes: false,
    autoImportSpells: false,
    soundAlerts: true,
    theme: 'hextech',
    alwaysOnTop: false,
  });

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
        } else if (res.gameflowPhase === 'InProgress' || res.gameflowPhase === 'GameStart') {
          setActiveTab('live-overlay');
          window.riftAPI?.setAlwaysOnTop?.(true).then((r) => {
            if (r) setIsPinned(r.isPinned);
          });
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

    // 3. Listen to LCU Status
    const unsubLCU = window.riftAPI.onLCUStatusChange((status: any) => {
      setIsConnected(status.connected);
      if (status.riotClient !== undefined) setIsRiotClientReady(status.riotClient);
      if (status.connected) {
        fetchInitialData();
      } else {
        setSummoner(null);
        setRankedData(null);
        setGameflowPhase('None');
      }
    });

    // 4. Listen to Gameflow Phase
    const unsubPhase = window.riftAPI.onGameflowPhase((phase) => {
      setGameflowPhase(phase);
      if (phase === 'ChampSelect') {
        setActiveTab('champ-select');
      } else if (phase === 'InProgress' || phase === 'GameStart') {
        setActiveTab('live-overlay');
        // Auto-enable Always-on-Top so overlay floats over borderless League
        window.riftAPI?.setAlwaysOnTop?.(true).then((r) => {
          if (r) setIsPinned(r.isPinned);
        });
      }
    });

    // 5. Listen to Champ Select Session
    const unsubChamp = window.riftAPI.onChampSelectSession((session) => {
      setChampSelectSession(session);
    });

    // 6. Listen to Ready Check (Queue Pop)
    const unsubReady = window.riftAPI.onReadyCheck((rcData) => {
      setReadyCheckData(rcData);
      if (rcData && rcData.state === 'InProgress' && rcData.playerResponse === 'None') {
        soundService.playQueuePop();
      }
    });

    // 7. Listen to Live Game Update (Direct 2999 poller)
    const unsubLive = window.riftAPI.onLiveGameUpdate((gData) => {
      setLiveData(gData);
      if (gData && gData.gameData) {
        setActiveTab((prev) => {
          if (prev !== 'live-overlay') {
            window.riftAPI?.setAlwaysOnTop?.(true).then((r) => {
              if (r) setIsPinned(r.isPinned);
            });
            return 'live-overlay';
          }
          return prev;
        });
      }
    });

    return () => {
      unsubLCU();
      unsubPhase();
      unsubChamp();
      unsubReady();
      unsubLive();
    };
  }, []);

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

  const handleUpdateSetting = async (key: keyof AppSettings, value: boolean) => {
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
      }
    } else {
      setIsCompactMode(!isCompactMode);
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

  return (
    <div className={`app-container ${isCompactMode ? 'compact-mode' : ''}`}>
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
      />

      {/* Queue Pop Notification Modal */}
      {isQueuePopActive && (
        <div className="queue-pop-modal" id="queue-pop-modal">
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
            <div
              style={{
                width: '40px',
                height: '40px',
                borderRadius: '50%',
                background: 'rgba(37, 211, 102, 0.2)',
                border: '2px solid #25d366',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                animation: 'pulse 1s infinite',
              }}
            >
              <Sparkles size={20} color="#25d366" />
            </div>
            <div>
              <div style={{ fontSize: '15px', fontWeight: 800, color: '#fff', letterSpacing: '0.5px' }}>
                MATCH FOUND!
              </div>
              <div style={{ fontSize: '11px', color: '#94a3b8' }}>
                {settings.autoAccept ? 'Auto-Accepting in progress...' : 'Ready Check Active'}
              </div>
            </div>
          </div>

          <button
            id="btn-modal-accept"
            className="btn-primary btn-accent"
            onClick={handleManualAccept}
            style={{ padding: '8px 24px', fontSize: '13px' }}
          >
            <Check size={16} />
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
            <ChampSelectView session={champSelectSession} />
          )}

          {activeTab === 'draft-simulator' && (
            <DraftSimulatorView
              session={champSelectSession}
              onSelectChampion={(champId) => {
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
            <ChampionPoolBuilderView />
          )}

          {(activeTab === 'game-modes' || activeTab === 'aram') && (
            <GameModesHubView />
          )}

          {activeTab === 'champions' && (
            <ChampionTierListView
              onSelectChampion={(champId) => {
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
