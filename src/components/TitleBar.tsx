import React, { useState, useEffect } from 'react';
import { Minus, Square, X, Pin, RefreshCw, Zap, Sparkles, Compass } from 'lucide-react';
import { GameflowPhase } from '../types';
import { patchService, PatchState } from '../services/patchService';
import { APP_VERSION } from '../version';

interface TitleBarProps {
  isConnected: boolean;
  isRiotClientReady?: boolean;
  gameflowPhase: GameflowPhase;
  isCompactMode: boolean;
  isPinned: boolean;
  onToggleCompact: () => void;
  onTogglePin: () => void;
  onLaunchLeague?: () => void;
  onCloseClick?: () => void;
  onOpenUpdateNotes?: () => void;
  onOpenQuickTour?: () => void;
}

export const TitleBar: React.FC<TitleBarProps> = ({
  isConnected,
  isRiotClientReady,
  gameflowPhase,
  isCompactMode,
  isPinned,
  onToggleCompact,
  onTogglePin,
  onLaunchLeague,
  onCloseClick,
  onOpenUpdateNotes,
  onOpenQuickTour,
}) => {
  const [patchState, setPatchState] = useState<PatchState>(patchService.getPatchState());

  useEffect(() => {
    const unsub = patchService.subscribe((state) => {
      setPatchState(state);
    });
    return unsub;
  }, []);

  const handleControl = (action: 'minimize' | 'maximize' | 'close') => {
    if (action === 'close' && onCloseClick) {
      onCloseClick();
      return;
    }
    if (window.riftAPI) {
      window.riftAPI.windowControl(action);
    }
  };

  const getPhaseDisplay = (phase: GameflowPhase) => {
    switch (phase) {
      case 'Matchmaking':
        return { text: 'In Queue', class: 'queue' };
      case 'ReadyCheck':
        return { text: 'Queue Pop!', class: 'ready-check' };
      case 'ChampSelect':
        return { text: 'Champ Select', class: 'champ-select' };
      case 'InProgress':
        return { text: 'Live Match', class: 'in-game' };
      case 'Lobby':
        return { text: 'Lobby', class: 'lobby' };
      case 'EndOfGame':
      case 'WaitingForStats':
        return { text: 'Post Game', class: 'post-game' };
      default:
        if (isConnected) return { text: 'Client Idle', class: 'idle' };
        if (isRiotClientReady) return { text: 'Launch League', class: 'ready-check' };
        return { text: 'Client Offline', class: 'offline' };
    }
  };

  const phaseInfo = getPhaseDisplay(gameflowPhase);

  const getStatusText = () => {
    if (isConnected) return 'League Connected';
    if (isRiotClientReady) return 'Riot Client Ready';
    return 'Waiting for League Client...';
  };

  const getStatusDotClass = () => {
    if (isConnected) return 'online';
    if (isRiotClientReady) return 'ready';
    return '';
  };

  return (
    <header className="titlebar" id="app-titlebar">
      <div className="titlebar-brand" style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        <img
          src="/logo.png"
          alt="RiftCore"
          style={{
            width: '20px',
            height: '20px',
            borderRadius: '4px',
            boxShadow: '0 0 10px rgba(0, 210, 255, 0.45)',
          }}
          onError={(e) => {
            (e.target as HTMLElement).style.display = 'none';
          }}
        />
        <span
          style={{
            background: 'linear-gradient(90deg, #ffffff 0%, #d8f5ff 55%, #00d2ff 100%)',
            WebkitBackgroundClip: 'text',
            WebkitTextFillColor: 'transparent',
            fontWeight: 900,
            fontSize: '12.5px',
            letterSpacing: '1.8px',
          }}
        >
          RIFTCORE
        </span>
        <span
          style={{
            fontSize: '8.5px',
            fontWeight: 800,
            letterSpacing: '0.6px',
            padding: '1px 5px',
            borderRadius: '3px',
            background: 'rgba(0, 210, 255, 0.1)',
            border: '1px solid rgba(0, 210, 255, 0.25)',
            color: 'var(--blue-primary)',
          }}
        >
          PRO
        </span>
      </div>

      <div className="titlebar-status">
        <div className="status-dot-container">
          <div className={`status-dot ${getStatusDotClass()}`} />
          <div className="status-ring" />
        </div>
        <span className="titlebar-status-text tabular-nums">{getStatusText()}</span>
        {!isConnected && isRiotClientReady && onLaunchLeague ? (
          <button
            onClick={onLaunchLeague}
            className="phase-pill ready-check"
            style={{ cursor: 'pointer', border: '1px solid #00d2ff', background: 'rgba(0, 210, 255, 0.15)' }}
            title="Launch League of Legends via Riot Client"
          >
            ▶ Launch League
          </button>
        ) : (
          <span className={`phase-pill ${phaseInfo.class}`}>{phaseInfo.text}</span>
        )}
      </div>

      <div className="titlebar-controls">
        {/* Quick Tour Button */}
        {onOpenQuickTour && (
          <button
            onClick={onOpenQuickTour}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '4px',
              padding: '2.5px 8px',
              borderRadius: 'var(--radius-xs)',
              background: 'rgba(255, 170, 0, 0.12)',
              border: '1px solid rgba(255, 170, 0, 0.35)',
              fontSize: '10px',
              fontWeight: 800,
              color: '#ffbe3b',
              cursor: 'pointer',
              marginRight: '6px',
              transition: 'all 0.18s ease',
            }}
            title="Open Interactive Quick Tour & Keybindings"
          >
            <Compass size={11} color="#ffbe3b" />
            <span>Tour</span>
          </button>
        )}

        {/* What's New / Update Notes Button */}
        {onOpenUpdateNotes && (
          <button
            onClick={onOpenUpdateNotes}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '4px',
              padding: '2.5px 8px',
              borderRadius: 'var(--radius-xs)',
              background: 'rgba(0, 210, 255, 0.12)',
              border: '1px solid rgba(0, 210, 255, 0.35)',
              fontSize: '10px',
              fontWeight: 800,
              color: '#fff',
              cursor: 'pointer',
              marginRight: '6px',
              transition: 'all 0.18s ease',
            }}
            title={`View What's New & v${APP_VERSION} Update Notes`}
          >
            <Sparkles size={10} color="var(--blue-primary)" />
            <span>What's New</span>
          </button>
        )}

        {/* Dynamic Riot Patch Badge */}
        <button
          onClick={() => patchService.checkAndApplyPatchUpdate()}
          disabled={patchState.isChecking}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '5px',
            padding: '2.5px 8px',
            borderRadius: 'var(--radius-xs)',
            background: 'rgba(0, 210, 255, 0.08)',
            border: '1px solid rgba(0, 210, 255, 0.22)',
            fontSize: '10.5px',
            fontWeight: 700,
            color: 'var(--blue-primary)',
            cursor: 'pointer',
            marginRight: '6px',
            transition: 'all 0.18s ease',
          }}
          title={`Patch ${patchState.version} • Click to check Riot for updates`}
        >
          <RefreshCw size={10} style={{ animation: patchState.isChecking ? 'spin 1s linear infinite' : 'none' }} />
          <span className="tabular-nums">Patch {patchState.majorMinor}</span>
        </button>

        <button
          id="btn-toggle-pin"
          className={`control-btn ${isPinned ? 'active' : ''}`}
          onClick={onTogglePin}
          title={isPinned ? 'Unpin from Top' : 'Always on Top'}
        >
          <Pin size={12.5} />
        </button>

        <button
          id="btn-minimize"
          className="control-btn"
          onClick={() => handleControl('minimize')}
          title="Minimize"
        >
          <Minus size={13} />
        </button>

        <button
          id="btn-maximize"
          className="control-btn"
          onClick={() => handleControl('maximize')}
          title="Maximize"
        >
          <Square size={11} />
        </button>

        <button
          id="btn-close"
          className="control-btn close"
          onClick={() => handleControl('close')}
          title="Close RiftCore"
        >
          <X size={13} />
        </button>
      </div>
    </header>
  );
};
