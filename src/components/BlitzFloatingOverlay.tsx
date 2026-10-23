import React, { useState, useEffect } from 'react';
import { LiveGameData } from '../types';
import { Maximize2, Minus, Move } from 'lucide-react';

interface BlitzFloatingOverlayProps {
  liveData: LiveGameData | null;
  onExpand: () => void;
  onMinimize?: () => void;
}

export const BlitzFloatingOverlay: React.FC<BlitzFloatingOverlayProps> = ({
  liveData,
  onExpand,
  onMinimize,
}) => {
  const [activeTab, setActiveTab] = useState<'stats' | 'jungle' | 'epics'>('stats');

  const gameTime = liveData?.gameData?.gameTime ? Math.floor(liveData.gameData.gameTime) : 0;
  const gameMinutes = Math.max(1, gameTime / 60);

  // Active Player info
  const activeSumm = liveData?.activePlayer?.summonerName?.toLowerCase() || '';
  const myPlayer = liveData?.allPlayers?.find(
    (p) => p.summonerName.toLowerCase() === activeSumm || p.championName.toLowerCase() === activeSumm
  ) || liveData?.allPlayers?.[0];

  const myTeam = myPlayer?.team || 'ORDER';

  // Calculate CSM (CS Per Minute)
  const myCS = myPlayer?.scores?.creepScore || 0;
  const csm = (myCS / gameMinutes).toFixed(1);
  const benchmarkCSM = 7.2;
  const csmDelta = (parseFloat(csm) - benchmarkCSM).toFixed(1);
  const isCsmAhead = parseFloat(csmDelta) >= 0;

  // Calculate GPM (Gold Per Minute)
  const myGold = (myPlayer?.currentGold || 0) + (myCS * 21) + ((myPlayer?.scores?.kills || 0) * 300);
  const myGpm = Math.floor(myGold / gameMinutes);
  const benchmarkGpm = 390;

  // Calculate Team Kills & KP%
  const teamKills = liveData?.allPlayers
    ?.filter((p) => p.team === myTeam)
    ?.reduce((sum, p) => sum + (p.scores?.kills || 0), 0) || 0;
  const myKills = myPlayer?.scores?.kills || 0;
  const myAssists = myPlayer?.scores?.assists || 0;
  const myDeaths = myPlayer?.scores?.deaths || 0;
  const kp = teamKills > 0 ? Math.round(((myKills + myAssists) / teamKills) * 100) : 0;
  const benchmarkKp = 45;

  // KDA
  const kdaRatio = myDeaths > 0 ? ((myKills + myAssists) / myDeaths).toFixed(1) : (myKills + myAssists).toFixed(1);
  const myLevel = myPlayer?.level || 1;

  // Check if player is Jungler (has Smite)
  useEffect(() => {
    const s1 = myPlayer?.summonerSpells?.summonerSpellOne?.displayName?.toLowerCase() || '';
    const s2 = myPlayer?.summonerSpells?.summonerSpellTwo?.displayName?.toLowerCase() || '';
    if (s1.includes('smite') || s2.includes('smite')) {
      setActiveTab('jungle');
    }
  }, [myPlayer]);

  // Jungle Timers
  const jungleCamps = [
    { name: 'Clear 1', actual: gameTime > 194 ? '3:14' : '--', delta: '+15s' },
    { name: 'Red', actual: gameTime > 98 ? '1:38' : '--', delta: '+13s' },
    { name: 'Krugs', actual: gameTime > 116 ? '1:56' : '--', delta: '+8s' },
    { name: 'Raptors', actual: gameTime > 143 ? '2:23' : '--', delta: '+6s' },
    { name: 'Wolves', actual: gameTime > 165 ? '2:45' : '--', delta: '--' },
    { name: 'Blue', actual: gameTime > 185 ? '3:05' : '--', delta: '--' },
    { name: 'Gromp', actual: gameTime > 205 ? '3:25' : '--', delta: '--' },
  ];

  // Drag handler
  const handleStartDrag = () => {
    if ((window as any).ipc) {
      (window as any).ipc.postMessage(JSON.stringify({ action: 'drag-window' }));
    } else if (window.riftAPI?.windowControl) {
      window.riftAPI.windowControl('drag-window' as any);
    }
  };

  const formatMinSec = (sec: number) => {
    const m = Math.floor(sec / 60);
    const s = sec % 60;
    return `${m}:${s < 10 ? '0' : ''}${s}`;
  };

  return (
    <div
      style={{
        width: '100vw',
        height: '100vh',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'transparent',
        padding: '6px',
        boxSizing: 'border-box',
      }}
    >
      <div
        style={{
          width: '100%',
          height: '100%',
          borderRadius: '10px',
          background: 'rgba(12, 19, 34, 0.88)',
          backdropFilter: 'blur(16px)',
          WebkitBackdropFilter: 'blur(16px)',
          border: '1px solid rgba(255, 255, 255, 0.18)',
          boxShadow: '0 8px 32px rgba(0, 0, 0, 0.65)',
          display: 'flex',
          flexDirection: 'column',
          fontFamily: "'Inter', -apple-system, sans-serif",
          color: '#f8fafc',
          overflow: 'hidden',
          userSelect: 'none',
        }}
      >
        {/* DRAGGABLE HEADER BAR */}
        <div
          onMouseDown={handleStartDrag}
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '8px 12px 6px 12px',
            borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
            cursor: 'grab',
            background: 'rgba(255, 255, 255, 0.03)',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            <span
              style={{
                fontSize: '11px',
                fontWeight: 900,
                letterSpacing: '0.8px',
                color: '#ffffff',
                textShadow: '0 0 10px rgba(0, 200, 255, 0.5)',
              }}
            >
              RIFTCORE.GG
            </span>
            {gameTime > 0 && (
              <span
                style={{
                  fontSize: '10px',
                  fontWeight: 700,
                  color: 'var(--blue-primary)',
                  padding: '1px 5px',
                  borderRadius: '3px',
                  background: 'rgba(0, 200, 255, 0.12)',
                }}
              >
                {formatMinSec(gameTime)}
              </span>
            )}
          </div>

          {/* WINDOW CONTROLS */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
            <button
              onClick={onExpand}
              title="Expand Full App"
              style={{
                background: 'transparent',
                border: 'none',
                color: '#94a3b8',
                cursor: 'pointer',
                padding: '2px 4px',
                borderRadius: '4px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Maximize2 size={12} />
            </button>
            <button
              onClick={onMinimize}
              title="Hide (F10)"
              style={{
                background: 'transparent',
                border: 'none',
                color: '#94a3b8',
                cursor: 'pointer',
                padding: '2px 4px',
                borderRadius: '4px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Minus size={13} />
            </button>
          </div>
        </div>

        {/* MODE TABS (STATS / JUNGLE / EPICS) */}
        <div
          style={{
            display: 'flex',
            borderBottom: '1px solid rgba(255, 255, 255, 0.06)',
            background: 'rgba(0, 0, 0, 0.2)',
          }}
        >
          {(['stats', 'jungle', 'epics'] as const).map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              style={{
                flex: 1,
                padding: '5px 0',
                fontSize: '10px',
                fontWeight: 800,
                textTransform: 'uppercase',
                letterSpacing: '0.5px',
                background: activeTab === tab ? 'rgba(0, 200, 255, 0.12)' : 'transparent',
                color: activeTab === tab ? 'var(--blue-primary)' : '#64748b',
                border: 'none',
                borderBottom: activeTab === tab ? '2px solid var(--blue-primary)' : '2px solid transparent',
                cursor: 'pointer',
                transition: 'all 0.15s ease',
              }}
            >
              {tab}
            </button>
          ))}
        </div>

        {/* TAB CONTENT */}
        <div style={{ flex: 1, padding: '10px 14px', display: 'flex', flexDirection: 'column', justifyContent: 'center' }}>
          {/* 1. STATS TAB (MATCHING USER SCREENSHOT 2) */}
          {activeTab === 'stats' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {/* CSM */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', fontSize: '13px' }}>
                <span style={{ fontWeight: 800, color: '#e2e8f0' }}>CSM</span>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <span style={{ fontWeight: 800, color: isCsmAhead ? '#22d3ee' : '#f87171' }}>
                    {isCsmAhead ? `+${csmDelta}` : csmDelta}
                  </span>
                  <span style={{ fontSize: '12px' }}>📈</span>
                  <span style={{ fontSize: '11px', color: '#94a3b8' }}>({csm})</span>
                </div>
              </div>

              {/* GPM */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', fontSize: '13px' }}>
                <span style={{ fontWeight: 800, color: '#e2e8f0' }}>GPM</span>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <span style={{ fontWeight: 800, color: '#22d3ee' }}>{myGpm}</span>
                  <span style={{ fontSize: '11px', color: '#64748b' }}>/</span>
                  <span style={{ fontSize: '12px', color: '#94a3b8' }}>{benchmarkGpm}</span>
                </div>
              </div>

              {/* KP% */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', fontSize: '13px' }}>
                <span style={{ fontWeight: 800, color: '#e2e8f0' }}>KP%</span>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <span style={{ fontWeight: 800, color: '#22d3ee' }}>{kp}%</span>
                  <span style={{ fontSize: '11px', color: '#64748b' }}>/</span>
                  <span style={{ fontSize: '12px', color: '#94a3b8' }}>{benchmarkKp}%</span>
                </div>
              </div>

              {/* KDA */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', fontSize: '13px' }}>
                <span style={{ fontWeight: 800, color: '#e2e8f0' }}>KDA</span>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <span style={{ fontWeight: 800, color: '#22d3ee' }}>{kdaRatio}</span>
                  <span style={{ fontSize: '11px', color: '#94a3b8' }}>
                    ({myKills}/{myDeaths}/{myAssists})
                  </span>
                </div>
              </div>

              {/* LVL */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', fontSize: '13px' }}>
                <span style={{ fontWeight: 800, color: '#e2e8f0' }}>LVL</span>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <span style={{ fontWeight: 800, color: '#22d3ee' }}>{myLevel}</span>
                  <span style={{ fontSize: '11px', color: '#64748b' }}>/</span>
                  <span style={{ fontSize: '12px', color: '#94a3b8' }}>{Math.max(1, Math.floor(gameMinutes / 2))}</span>
                </div>
              </div>
            </div>
          )}

          {/* 2. JUNGLE TAB (MATCHING USER SCREENSHOT 1) */}
          {activeTab === 'jungle' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '5px' }}>
              {jungleCamps.map((camp) => (
                <div
                  key={camp.name}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    fontSize: '12px',
                    padding: '2px 0',
                  }}
                >
                  <span style={{ fontWeight: 700, color: '#cbd5e1' }}>{camp.name}</span>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                    <span style={{ fontWeight: 800, color: camp.actual !== '--' ? '#fff' : '#64748b' }}>
                      {camp.actual}
                    </span>
                    {camp.actual !== '--' && (
                      <span style={{ fontSize: '10px', fontWeight: 800, color: '#22d3ee' }}>
                        ▲ {camp.delta}
                      </span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* 3. EPICS & OBJECTIVES TAB */}
          {activeTab === 'epics' && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '9px' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', fontSize: '12px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <span style={{ fontSize: '15px' }}>🐲</span>
                  <span style={{ fontWeight: 700, color: '#cbd5e1' }}>Dragon</span>
                </div>
                <span style={{ fontWeight: 900, color: '#22d3ee', fontSize: '13px' }}>
                  {gameTime < 300 ? formatMinSec(300 - gameTime) : '3:12'}
                </span>
              </div>

              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', fontSize: '12px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <span style={{ fontSize: '15px' }}>👾</span>
                  <span style={{ fontWeight: 700, color: '#cbd5e1' }}>Voidgrubs</span>
                </div>
                <span style={{ fontWeight: 900, color: '#ff2a5f', fontSize: '13px' }}>
                  {gameTime < 360 ? formatMinSec(360 - gameTime) : 'ALIVE'}
                </span>
              </div>

              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', fontSize: '12px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <span style={{ fontSize: '15px' }}>👑</span>
                  <span style={{ fontWeight: 700, color: '#cbd5e1' }}>Baron Nashor</span>
                </div>
                <span style={{ fontWeight: 900, color: '#c084fc', fontSize: '13px' }}>
                  {gameTime < 1200 ? formatMinSec(1200 - gameTime) : 'ALIVE'}
                </span>
              </div>

              <div
                style={{
                  marginTop: '4px',
                  paddingTop: '6px',
                  borderTop: '1px solid rgba(255, 255, 255, 0.08)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  fontSize: '10px',
                  color: '#94a3b8',
                }}
              >
                <span>Toggle HUD:</span>
                <kbd style={{ background: '#1e293b', padding: '1px 5px', borderRadius: '3px', color: '#22d3ee' }}>
                  F10
                </kbd>
              </div>
            </div>
          )}
        </div>

        {/* SUBTLE FOOTER HINT */}
        <div
          onMouseDown={handleStartDrag}
          style={{
            padding: '4px 10px',
            background: 'rgba(0, 0, 0, 0.25)',
            borderTop: '1px solid rgba(255, 255, 255, 0.05)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            fontSize: '9px',
            color: '#64748b',
            cursor: 'grab',
          }}
        >
          <span style={{ display: 'flex', alignItems: 'center', gap: '3px' }}>
            <Move size={9} /> Drag to Reposition
          </span>
          <span onClick={onExpand} style={{ cursor: 'pointer', color: '#38bdf8' }}>
            Full App ⛶
          </span>
        </div>
      </div>
    </div>
  );
};
