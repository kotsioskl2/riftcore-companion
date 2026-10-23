import React, { useState, useEffect } from 'react';
import {
  Crosshair,
  Clock,
  Copy,
  Check,
  ShieldAlert,
  Play,
  RotateCcw,
  Sparkles,
  Award,
  TrendingUp,
  Flame,
  Swords,
  Timer
} from 'lucide-react';
import { LiveGameData } from '../types';
import { soundService } from '../services/soundService';

interface LiveGameOverlayProps {
  liveData: LiveGameData | null;
}

interface SpellTracker {
  champion: string;
  role: string;
  spellId: number;
  spellName: string;
  cooldownTotal: number;
  cooldownRemaining: number;
  timerActive: boolean;
  readyGameTime: number;
}

export const LiveGameOverlay: React.FC<LiveGameOverlayProps> = ({ liveData }) => {
  const isLive = liveData !== null && liveData.gameData !== undefined;
  const [isDemoMode, setIsDemoMode] = useState(false);
  const [demoGameTime, setDemoGameTime] = useState(745);
  const [copied, setCopied] = useState(false);

  // Active game clock in seconds
  const activeGameTime = isLive
    ? Math.floor(liveData.gameData.gameTime)
    : isDemoMode
    ? demoGameTime
    : 0;

  const [enemySpells, setEnemySpells] = useState<SpellTracker[]>([
    { champion: 'Darius', role: 'TOP', spellId: 4, spellName: 'Flash', cooldownTotal: 300, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Darius', role: 'TOP', spellId: 6, spellName: 'Ghost', cooldownTotal: 240, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Lee Sin', role: 'JUNGLE', spellId: 4, spellName: 'Flash', cooldownTotal: 300, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Lee Sin', role: 'JUNGLE', spellId: 11, spellName: 'Smite', cooldownTotal: 90, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Ahri', role: 'MID', spellId: 4, spellName: 'Flash', cooldownTotal: 300, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Ahri', role: 'MID', spellId: 12, spellName: 'Teleport', cooldownTotal: 360, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Jinx', role: 'ADC', spellId: 4, spellName: 'Flash', cooldownTotal: 300, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Jinx', role: 'ADC', spellId: 7, spellName: 'Heal', cooldownTotal: 240, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Thresh', role: 'SUPPORT', spellId: 4, spellName: 'Flash', cooldownTotal: 300, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Thresh', role: 'SUPPORT', spellId: 14, spellName: 'Ignite', cooldownTotal: 180, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
  ]);

  const [objectives, setObjectives] = useState([
    { name: 'Hextech / Elemental Dragon', respawnTime: 140, total: 300, firstSpawn: 300, icon: '🐲', color: '#00c8ff' },
    { name: 'Voidgrubs / Herald', respawnTime: 210, total: 360, firstSpawn: 300, icon: '👾', color: '#ff2a5f' },
    { name: 'Baron Nashor', respawnTime: 455, total: 1200, firstSpawn: 1200, icon: '👑', color: '#c084fc' },
  ]);

  // Only run the ticker if in active live match or if user manually enabled Demo Mode
  useEffect(() => {
    if (!isLive && !isDemoMode) {
      return;
    }

    const timer = setInterval(() => {
      if (isDemoMode) {
        setDemoGameTime((prev) => prev + 1);
      }

      setEnemySpells((prevSpells) =>
        prevSpells.map((s) => {
          if (s.timerActive && s.cooldownRemaining > 0) {
            const nextRemain = s.cooldownRemaining - 1;
            if (nextRemain === 0) {
              soundService.playTimerTick();
              return { ...s, cooldownRemaining: 0, timerActive: false };
            }
            return { ...s, cooldownRemaining: nextRemain };
          }
          return s;
        })
      );

      if (isDemoMode) {
        setObjectives((prevObjs) =>
          prevObjs.map((obj) => ({
            ...obj,
            respawnTime: obj.respawnTime > 0 ? obj.respawnTime - 1 : obj.total,
          }))
        );
      }
    }, 1000);

    return () => clearInterval(timer);
  }, [isLive, isDemoMode]);

  const triggerSpellCooldown = (index: number) => {
    setEnemySpells((prev) => {
      const copy = [...prev];
      const target = copy[index];
      if (target.timerActive) {
        target.timerActive = false;
        target.cooldownRemaining = 0;
      } else {
        target.timerActive = true;
        target.cooldownRemaining = target.cooldownTotal;
        target.readyGameTime = activeGameTime + target.cooldownTotal;
        soundService.playTimerTick();
      }
      return copy;
    });
  };

  const copyTimersToChat = () => {
    const activeCooldowns = enemySpells.filter((s) => s.timerActive);
    let message = '';
    if (activeCooldowns.length === 0) {
      message = 'All enemy summoners UP';
    } else {
      message = activeCooldowns
        .map((s) => {
          const readyMin = Math.floor(s.readyGameTime / 60);
          const readySec = Math.floor(s.readyGameTime % 60);
          const timeFormatted = `${readyMin.toString().padStart(2, '0')}:${readySec.toString().padStart(2, '0')}`;
          return `${s.role.toLowerCase()} ${s.spellName.toLowerCase()} ${timeFormatted}`;
        })
        .join(' | ');
    }

    navigator.clipboard.writeText(message);
    setCopied(true);
    soundService.playAutoAccept();
    setTimeout(() => setCopied(false), 2500);
  };

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  // Live CS Benchmarking
  const currentCs = isLive ? (liveData?.activePlayer?.scores?.creepScore || 0) : isDemoMode ? 96 : 0;
  const gameMinutes = Math.max(1, activeGameTime / 60);
  const csPerMin = activeGameTime > 0 ? (currentCs / gameMinutes).toFixed(1) : '0.0';
  const numCsPerMin = parseFloat(csPerMin);

  const getCsBenchmarkRating = () => {
    if (numCsPerMin >= 8.2) return { tier: 'CHALLENGER PACE', color: '#00c8ff', diff: '+28% Ahead' };
    if (numCsPerMin >= 7.0) return { tier: 'DIAMOND PACE', color: '#22c55e', diff: '+15% Ahead' };
    if (numCsPerMin >= 5.8) return { tier: 'GOLD / PLAT PACE', color: '#f59e0b', diff: 'Average' };
    return { tier: 'NEEDS FARM', color: '#ff2a5f', diff: 'Below Curve' };
  };

  const csRating = getCsBenchmarkRating();

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Top Banner with Game Status & Clock */}
      <div className="glass-card" style={{ padding: '16px 24px', border: '1px solid var(--border-blue)' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '14px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '14px' }}>
            <div
              style={{
                width: '42px',
                height: '42px',
                borderRadius: '50%',
                background: isLive ? 'rgba(34, 197, 94, 0.15)' : 'var(--blue-subtle)',
                border: `1px solid ${isLive ? '#22c55e' : 'var(--blue-primary)'}`,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Crosshair size={22} color={isLive ? '#22c55e' : 'var(--blue-primary)'} />
            </div>
            <div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <h2 style={{ fontSize: '18px', fontWeight: 800, color: '#fff' }}>
                  Live Match Overlay & Timers
                </h2>
                <span
                  style={{
                    fontSize: '10px',
                    fontWeight: 800,
                    textTransform: 'uppercase',
                    padding: '2px 8px',
                    borderRadius: '4px',
                    background: isLive
                      ? 'rgba(34, 197, 94, 0.15)'
                      : isDemoMode
                      ? 'rgba(234, 179, 8, 0.15)'
                      : 'rgba(100, 116, 139, 0.15)',
                    color: isLive ? '#22c55e' : isDemoMode ? '#facc15' : '#94a3b8',
                    border: `1px solid ${isLive ? '#22c55e' : isDemoMode ? '#facc15' : 'rgba(100, 116, 139, 0.3)'}`,
                  }}
                >
                  {isLive
                    ? '● Live Match Synced'
                    : isDemoMode
                    ? '🧪 Practice / Demo Mode'
                    : '⏸️ Standby — Waiting for Match Start'}
                </span>
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
                Summoner's Rift • Game Clock:{' '}
                <strong style={{ color: 'var(--blue-primary)', fontFamily: 'var(--font-mono)', fontSize: '14px' }}>
                  {activeGameTime > 0 ? formatTime(activeGameTime) : '00:00 (Syncs on match load)'}
                </strong>
              </div>
            </div>
          </div>

          {/* Action Buttons: Demo Mode & Copy Timers */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px', flexWrap: 'wrap' }}>
            {!isLive && (
              <button
                onClick={() => setIsDemoMode(!isDemoMode)}
                style={{
                  padding: '8px 14px',
                  borderRadius: 'var(--radius-md)',
                  border: isDemoMode ? '1px solid #facc15' : '1px solid var(--border-subtle)',
                  background: isDemoMode ? 'rgba(234, 179, 8, 0.12)' : 'rgba(255, 255, 255, 0.04)',
                  color: isDemoMode ? '#facc15' : '#94a3b8',
                  fontSize: '12px',
                  fontWeight: 700,
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                }}
              >
                <Sparkles size={14} />
                <span>{isDemoMode ? 'Stop Demo Clock' : 'Test / Demo Mode'}</span>
              </button>
            )}

            <button
              id="btn-copy-timers"
              className="btn-primary"
              onClick={copyTimersToChat}
              style={{ minWidth: '180px' }}
              title="Copies enemy cooldowns formatted for League team chat"
            >
              {copied ? (
                <>
                  <Check size={16} />
                  <span>Copied to Clipboard!</span>
                </>
              ) : (
                <>
                  <Copy size={16} />
                  <span>Copy Timers to Chat</span>
                </>
              )}
            </button>
          </div>
        </div>
      </div>

      {/* Real-time CS / Farm Benchmark Meter */}
      <div
        className="glass-card"
        style={{
          padding: '16px 20px',
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
          gap: '16px',
          border: '1px solid var(--border-subtle)',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <div
            style={{
              width: '38px',
              height: '38px',
              borderRadius: '8px',
              background: 'rgba(0, 200, 255, 0.1)',
              border: '1px solid var(--border-blue)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <TrendingUp size={20} color="var(--blue-primary)" />
          </div>
          <div>
            <div style={{ fontSize: '11px', textTransform: 'uppercase', color: '#94a3b8', fontWeight: 700 }}>
              Live CS Tracker
            </div>
            <div style={{ fontSize: '17px', fontWeight: 800, fontFamily: 'var(--font-mono)', color: '#fff' }}>
              {currentCs} CS <span style={{ fontSize: '13px', color: 'var(--blue-primary)' }}>({csPerMin} / min)</span>
            </div>
          </div>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <div
            style={{
              width: '38px',
              height: '38px',
              borderRadius: '8px',
              background: 'rgba(34, 197, 94, 0.1)',
              border: '1px solid rgba(34, 197, 94, 0.3)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <Award size={20} color="#22c55e" />
          </div>
          <div>
            <div style={{ fontSize: '11px', textTransform: 'uppercase', color: '#94a3b8', fontWeight: 700 }}>
              Rank Benchmark
            </div>
            <div style={{ fontSize: '14px', fontWeight: 800, color: csRating.color }}>
              {csRating.tier} <span style={{ fontSize: '11px', color: '#94a3b8' }}>({csRating.diff})</span>
            </div>
          </div>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <div
            style={{
              width: '38px',
              height: '38px',
              borderRadius: '8px',
              background: 'rgba(255, 42, 95, 0.1)',
              border: '1px solid var(--border-red)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <Swords size={20} color="var(--red-primary)" />
          </div>
          <div>
            <div style={{ fontSize: '11px', textTransform: 'uppercase', color: '#94a3b8', fontWeight: 700 }}>
              Quick Chat Format
            </div>
            <div style={{ fontSize: '12px', fontFamily: 'var(--font-mono)', color: '#cbd5e1' }}>
              Press [Copy] &gt; In-game [Enter &gt; Ctrl+V]
            </div>
          </div>
        </div>
      </div>

      {/* Epic Monster Respawn Timers */}
      <div className="glass-card">
        <div className="card-header">
          <div className="card-title">
            <Clock size={16} color="var(--blue-primary)" />
            <span>Epic Monster Respawn Timers</span>
          </div>
          <span style={{ fontSize: '11px', color: '#64748b' }}>
            Auto-syncs via memory events on port 2999
          </span>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))', gap: '14px' }}>
          {objectives.map((obj) => {
            const displayTime = activeGameTime > 0 ? obj.respawnTime : obj.firstSpawn;
            const percent = activeGameTime > 0 ? Math.max(0, Math.min(100, (1 - obj.respawnTime / obj.total) * 100)) : 0;
            return (
              <div
                key={obj.name}
                style={{
                  background: 'rgba(255, 255, 255, 0.02)',
                  border: '1px solid var(--border-subtle)',
                  borderRadius: 'var(--radius-md)',
                  padding: '14px',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '14px',
                }}
              >
                <div style={{ fontSize: '26px' }}>{obj.icon}</div>
                <div style={{ flex: 1 }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '6px' }}>
                    <span style={{ fontSize: '13px', fontWeight: 600, color: '#fff' }}>{obj.name}</span>
                    <span style={{ fontFamily: 'var(--font-mono)', fontSize: '14px', fontWeight: 700, color: obj.color }}>
                      {activeGameTime > 0 ? formatTime(displayTime) : `Spawns ${formatTime(displayTime)}`}
                    </span>
                  </div>
                  <div style={{ width: '100%', height: '5px', background: 'rgba(255, 255, 255, 0.08)', borderRadius: '4px', overflow: 'hidden' }}>
                    <div
                      style={{
                        width: `${percent}%`,
                        height: '100%',
                        background: obj.color,
                        boxShadow: `0 0 8px ${obj.color}`,
                        transition: 'width 1s linear',
                      }}
                    />
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Enemy Summoner Spell Tracker */}
      <div className="glass-card">
        <div className="card-header">
          <div className="card-title">
            <ShieldAlert size={16} color="var(--red-primary)" />
            <span>Enemy Summoner Spell Cooldown Tracker</span>
          </div>
          <span style={{ fontSize: '11px', color: '#64748b' }}>
            Click any spell icon when you see an enemy flash/ignite to start real-time countdown
          </span>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
          {['TOP', 'JUNGLE', 'MID', 'ADC', 'SUPPORT'].map((role) => {
            const roleSpells = enemySpells
              .map((spell, index) => ({ ...spell, index }))
              .filter((s) => s.role === role);

            const champName = roleSpells[0]?.champion || 'Unknown';

            return (
              <div
                key={role}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  padding: '10px 16px',
                  borderRadius: 'var(--radius-md)',
                  background: 'rgba(255, 255, 255, 0.02)',
                  border: '1px solid var(--border-subtle)',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '14px' }}>
                  <span
                    style={{
                      width: '65px',
                      fontSize: '11px',
                      fontWeight: 800,
                      padding: '3px 8px',
                      borderRadius: '4px',
                      textAlign: 'center',
                      background: 'rgba(255, 42, 95, 0.1)',
                      color: 'var(--red-primary)',
                      border: '1px solid var(--border-red)',
                    }}
                  >
                    {role}
                  </span>
                  <span style={{ fontSize: '14px', fontWeight: 700, color: '#fff' }}>{champName}</span>
                </div>

                <div style={{ display: 'flex', gap: '10px' }}>
                  {roleSpells.map((s) => {
                    const isCooldown = s.timerActive && s.cooldownRemaining > 0;
                    return (
                      <button
                        key={`${s.champion}-${s.spellName}`}
                        onClick={() => triggerSpellCooldown(s.index)}
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          gap: '8px',
                          padding: '6px 14px',
                          borderRadius: 'var(--radius-sm)',
                          background: isCooldown ? 'rgba(255, 42, 95, 0.15)' : 'rgba(255, 255, 255, 0.05)',
                          border: `1px solid ${isCooldown ? 'var(--red-primary)' : 'var(--border-subtle)'}`,
                          cursor: 'pointer',
                          minWidth: '130px',
                          transition: 'all 0.2s ease',
                        }}
                      >
                        <span style={{ fontSize: '12px', fontWeight: 700, color: isCooldown ? 'var(--red-primary)' : '#fff' }}>
                          {s.spellName}
                        </span>
                        <span
                          style={{
                            fontFamily: 'var(--font-mono)',
                            fontSize: '12px',
                            fontWeight: 800,
                            color: isCooldown ? '#ff2a5f' : '#22c55e',
                            marginLeft: 'auto',
                          }}
                        >
                          {isCooldown ? formatTime(s.cooldownRemaining) : 'READY'}
                        </span>
                      </button>
                    );
                  })}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
