import React, { useState, useEffect } from 'react';
import { Minus, Square, X, Pin, LayoutTemplate, RefreshCw, Zap } from 'lucide-react';
import { GameflowPhase } from '../types';
import { patchService, PatchState } from '../services/patchService';

interface TitleBarProps {
  isConnected: boolean;
  isRiotClientReady?: boolean;
  gameflowPhase: GameflowPhase;
  isCompactMode: boolean;
  isPinned: boolean;
  onToggleCompact: () => void;
  onTogglePin: () => void;
  onLaunchLeague?: () => void;
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
}) => {
  const [patchState, setPatchState] = useState<PatchState>(patchService.getPatchState());

  useEffect(() => {
    const unsub = patchService.subscribe((state) => {
      setPatchState(state);
    });
    return unsub;
  }, []);

  const handleControl = (action: 'minimize' | 'maximize' | 'close') => {
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
      <div className="titlebar-brand">
        <div style={{ display: 'flex', gap: '4px' }}>
          <div className="brand-dot-blue" />
          <div className="brand-dot-red" />
        </div>
        <span>RIFTCORE</span>
      </div>

      <div className="titlebar-status">
        <div className={`status-dot ${getStatusDotClass()}`} />
        <span>{getStatusText()}</span>
        {!isConnected && isRiotClientReady && onLaunchLeague ? (
          <button
            onClick={onLaunchLeague}
            className="phase-pill ready-check"
            style={{ cursor: 'pointer', border: '1px solid #00c8ff', background: 'rgba(0, 200, 255, 0.15)' }}
            title="Launch League of Legends via Riot Client"
          >
            ▶ Launch League
          </button>
        ) : (
          <span className={`phase-pill ${phaseInfo.class}`}>{phaseInfo.text}</span>
        )}
      </div>

      <div className="titlebar-controls">
        {/* Dynamic Riot Patch Badge */}
        <button
          onClick={() => patchService.checkAndApplyPatchUpdate()}
          disabled={patchState.isChecking}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '5px',
            padding: '3px 8px',
            borderRadius: '4px',
            background: 'rgba(0, 200, 255, 0.08)',
            border: '1px solid rgba(0, 200, 255, 0.25)',
            fontSize: '11px',
            fontWeight: 700,
            color: 'var(--blue-primary)',
            cursor: 'pointer',
            marginRight: '6px',
            transition: 'all 0.15s ease',
          }}
          title={`Patch ${patchState.version} • Click to check Riot for updates`}
        >
          <RefreshCw size={11} style={{ animation: patchState.isChecking ? 'spin 1s linear infinite' : 'none' }} />
          <span>Patch {patchState.majorMinor}</span>
        </button>

        <button
          id="btn-toggle-pin"
          className={`control-btn ${isPinned ? 'active' : ''}`}
          onClick={onTogglePin}
          title={isPinned ? 'Unpin from Top' : 'Always on Top (Overlay)'}
        >
          <Pin size={13} />
        </button>

        <button
          id="btn-toggle-compact"
          className={`control-btn ${isCompactMode ? 'active' : ''}`}
          onClick={onToggleCompact}
          title={isCompactMode ? 'Full Mode' : 'Compact HUD Mode'}
        >
          <LayoutTemplate size={13} />
        </button>

        <button
          id="btn-minimize"
          className="control-btn"
          onClick={() => handleControl('minimize')}
          title="Minimize"
        >
          <Minus size={14} />
        </button>

        <button
          id="btn-maximize"
          className="control-btn"
          onClick={() => handleControl('maximize')}
          title="Maximize"
        >
          <Square size={12} />
        </button>

        <button
          id="btn-close"
          className="control-btn close"
          onClick={() => handleControl('close')}
          title="Close RiftCore"
        >
          <X size={14} />
        </button>
      </div>
    </header>
  );
};
