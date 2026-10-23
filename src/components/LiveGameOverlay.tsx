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
  Timer,
  Volume2,
  VolumeX,
  Compass,
  Info,
  Layers
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

interface JungleCamp {
  name: string;
  icon: string;
  respawnDuration: number;
  remaining: number;
  active: boolean;
}

export const LiveGameOverlay: React.FC<LiveGameOverlayProps> = ({ liveData }) => {
  const isLive = liveData !== null && liveData.gameData !== undefined;
  const [isDemoMode, setIsDemoMode] = useState(false);
  const [demoGameTime, setDemoGameTime] = useState(0);
  const [copied, setCopied] = useState(false);
  const [soundAlerts, setSoundAlerts] = useState(true);
  const [activeTab, setActiveTab] = useState<'epics' | 'jungle'>('epics');

  // Active game clock in seconds
  const activeGameTime = isLive
    ? Math.floor(liveData.gameData.gameTime)
    : isDemoMode
    ? demoGameTime
    : 0;

  // Enemy Summoner Spells
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

  // Epic Objectives
  const [objectives, setObjectives] = useState([
    { id: 'dragon', name: 'Elemental Dragon', respawnTime: 300, total: 300, firstSpawn: 300, icon: '🐲', color: '#00c8ff' },
    { id: 'grubs', name: 'Voidgrubs / Herald', respawnTime: 300, total: 360, firstSpawn: 300, icon: '👾', color: '#ff2a5f' },
    { id: 'baron', name: 'Baron Nashor', respawnTime: 1200, total: 1200, firstSpawn: 1200, icon: '👑', color: '#c084fc' },
  ]);

  // Standard Jungle Camps
  const [jungleCamps, setJungleCamps] = useState<JungleCamp[]>([
    { name: 'Blue Buff (Sentinel)', icon: '🔷', respawnDuration: 300, remaining: 0, active: false },
    { name: 'Red Buff (Brambleback)', icon: '🔴', respawnDuration: 300, remaining: 0, active: false },
    { name: 'Raptors', icon: '🦅', respawnDuration: 135, remaining: 0, active: false },
    { name: 'Murkwolves', icon: '🐺', respawnDuration: 135, remaining: 0, active: false },
    { name: 'Gromp', icon: '🐸', respawnDuration: 135, remaining: 0, active: false },
    { name: 'Ancient Krugs', icon: '🗿', respawnDuration: 135, remaining: 0, active: false },
    { name: 'Rift Scuttler', icon: '🦀', respawnDuration: 150, remaining: 0, active: false },
  ]);

  // Start Demo Mode
  const handleToggleDemoMode = () => {
    if (isDemoMode) {
      // Reset back to Standby
      setIsDemoMode(false);
      setDemoGameTime(0);
      setObjectives([
        { id: 'dragon', name: 'Elemental Dragon', respawnTime: 300, total: 300, firstSpawn: 300, icon: '🐲', color: '#00c8ff' },
        { id: 'grubs', name: 'Voidgrubs / Herald', respawnTime: 300, total: 360, firstSpawn: 300, icon: '👾', color: '#ff2a5f' },
        { id: 'baron', name: 'Baron Nashor', respawnTime: 1200, total: 1200, firstSpawn: 1200, icon: '👑', color: '#c084fc' },
      ]);
      setEnemySpells((prev) => prev.map((s) => ({ ...s, cooldownRemaining: 0, timerActive: false })));
      setJungleCamps((prev) => prev.map((c) => ({ ...c, remaining: 0, active: false })));
    } else {
      // Start simulated game at 12:30 with active objectives
      setIsDemoMode(true);
      setDemoGameTime(750);
      setObjectives([
        { id: 'dragon', name: 'Elemental Dragon', respawnTime: 135, total: 300, firstSpawn: 300, icon: '🐲', color: '#00c8ff' },
        { id: 'grubs', name: 'Voidgrubs / Herald', respawnTime: 210, total: 360, firstSpawn: 300, icon: '👾', color: '#ff2a5f' },
        { id: 'baron', name: 'Baron Nashor', respawnTime: 450, total: 1200, firstSpawn: 1200, icon: '👑', color: '#c084fc' },
      ]);
      soundService.playTimerTick();
    }
  };

  // 1-Second Master Game Timer (Only runs during live match or active demo mode)
  useEffect(() => {
    if (!isLive && !isDemoMode) {
      return; // Do NOT count if in standby!
    }

    const timer = setInterval(() => {
      if (isDemoMode) {
        setDemoGameTime((prev) => prev + 1);
      }

      // Decrement enemy spells
      setEnemySpells((prevSpells) =>
        prevSpells.map((s) => {
          if (s.timerActive && s.cooldownRemaining > 0) {
            const nextRemain = s.cooldownRemaining - 1;
            if (nextRemain === 0) {
              if (soundAlerts) soundService.playTimerTick();
              return { ...s, cooldownRemaining: 0, timerActive: false };
            }
            return { ...s, cooldownRemaining: nextRemain };
          }
          return s;
        })
      );

      // Decrement objectives
      if (isDemoMode) {
        setObjectives((prevObjs) =>
          prevObjs.map((obj) => {
            const nextTime = obj.respawnTime > 0 ? obj.respawnTime - 1 : obj.total;
            if (nextTime === 30 && soundAlerts) {
              soundService.playQueuePop(); // Objective 30s alert
            }
            return { ...obj, respawnTime: nextTime };
          })
        );
      }

      // Decrement jungle camps
      setJungleCamps((prevCamps) =>
        prevCamps.map((camp) => {
          if (camp.active && camp.remaining > 0) {
            const nextRem = camp.remaining - 1;
            if (nextRem === 0) {
              if (soundAlerts) soundService.playTimerTick();
              return { ...camp, remaining: 0, active: false };
            }
            return { ...camp, remaining: nextRem };
          }
          return camp;
        })
      );
    }, 1000);

    return () => clearInterval(timer);
  }, [isLive, isDemoMode, soundAlerts]);

  // Click Enemy Spell to toggle cooldown
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

  // Click Jungle Camp to start respawn timer
  const triggerCampTimer = (index: number) => {
    setJungleCamps((prev) => {
      const copy = [...prev];
      const target = copy[index];
      if (target.active) {
        target.active = false;
        target.remaining = 0;
      } else {
        target.active = true;
        target.remaining = target.respawnDuration;
        soundService.playTimerTick();
      }
      return copy;
    });
  };

  // Copy Formatted Timers to Clipboard for League Team Chat
  const copyTimersToChat = () => {
    const activeCooldowns = enemySpells.filter((s) => s.timerActive);
    let parts: string[] = [];

    if (activeCooldowns.length > 0) {
      const spellStrings = activeCooldowns.map((s) => {
        const readyMin = Math.floor(s.readyGameTime / 60);
        const readySec = Math.floor(s.readyGameTime % 60);
        const timeFormatted = `${readyMin.toString().padStart(2, '0')}:${readySec.toString().padStart(2, '0')}`;
        return `${s.role.toLowerCase()} ${s.spellName.toLowerCase()} ${timeFormatted}`;
      });
      parts.push(spellStrings.join(' | '));
    }

    if (isDemoMode || isLive) {
      const dragonObj = objectives.find((o) => o.id === 'dragon');
      if (dragonObj && dragonObj.respawnTime > 0) {
        const drgReady = activeGameTime + dragonObj.respawnTime;
        parts.push(`drg ${formatTime(drgReady)}`);
      }
    }

    const message = parts.length > 0 ? `[RiftCore] ${parts.join(' | ')}` : '[RiftCore] All enemy summoners & objectives UP';

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
  const currentCs = isLive ? (liveData?.activePlayer?.scores?.creepScore || 0) : isDemoMode ? 104 : 0;
  const gameMinutes = Math.max(1, activeGameTime / 60);
  const csPerMin = activeGameTime > 0 ? (currentCs / gameMinutes).toFixed(1) : '0.0';
  const numCsPerMin = parseFloat(csPerMin);

  const getCsBenchmarkRating = () => {
    if (!isLive && !isDemoMode) return { tier: 'STANDBY', color: '#94a3b8', diff: 'Awaiting Minion Wave' };
    if (numCsPerMin >= 8.4) return { tier: 'CHALLENGER PACE', color: '#00c8ff', diff: '+28% Ahead' };
    if (numCsPerMin >= 7.0) return { tier: 'DIAMOND PACE', color: '#22c55e', diff: '+15% Ahead' };
    if (numCsPerMin >= 5.8) return { tier: 'GOLD / PLAT PACE', color: '#f59e0b', diff: 'Average' };
    return { tier: 'BELOW TARGET', color: '#ff2a5f', diff: 'Focus on Waves' };
  };

  const csRating = getCsBenchmarkRating();

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Top Banner: Status, Game Clock & Controls */}
      <div className="glass-card" style={{ padding: '16px 24px', border: '1px solid var(--border-blue)' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '14px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '14px' }}>
            <div
              style={{
                width: '42px',
                height: '42px',
                borderRadius: '50%',
                background: isLive ? 'rgba(34, 197, 94, 0.15)' : isDemoMode ? 'rgba(234, 179, 8, 0.15)' : 'var(--blue-subtle)',
                border: `1px solid ${isLive ? '#22c55e' : isDemoMode ? '#facc15' : 'var(--blue-primary)'}`,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Crosshair size={22} color={isLive ? '#22c55e' : isDemoMode ? '#facc15' : 'var(--blue-primary)'} />
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
                    ? '🧪 Practice / Simulation Active'
                    : '⏸️ Standby — Awaiting Match'}
                </span>
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
                Summoner's Rift • Game Clock:{' '}
                <strong style={{ color: isLive ? '#22c55e' : isDemoMode ? '#facc15' : 'var(--blue-primary)', fontFamily: 'var(--font-mono)', fontSize: '14px' }}>
                  {activeGameTime > 0 ? formatTime(activeGameTime) : '00:00 (Standby)'}
                </strong>
              </div>
            </div>
          </div>

          {/* Action Buttons: Sound Alert, Demo Simulation, Copy Timers */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px', flexWrap: 'wrap' }}>
            {/* Audio Cue Toggle */}
            <button
              onClick={() => setSoundAlerts(!soundAlerts)}
              style={{
                padding: '8px 12px',
                borderRadius: 'var(--radius-md)',
                border: '1px solid var(--border-subtle)',
                background: soundAlerts ? 'rgba(0, 200, 255, 0.1)' : 'rgba(255, 255, 255, 0.04)',
                color: soundAlerts ? 'var(--blue-primary)' : '#64748b',
                fontSize: '11px',
                fontWeight: 700,
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
              }}
              title="Toggle in-game audio cues for objective spawns"
            >
              {soundAlerts ? <Volume2 size={14} /> : <VolumeX size={14} />}
              <span>{soundAlerts ? 'Audio Alerts: ON' : 'Audio: OFF'}</span>
            </button>

            {/* Test Simulation Toggle */}
            {!isLive && (
              <button
                onClick={handleToggleDemoMode}
                style={{
                  padding: '8px 14px',
                  borderRadius: 'var(--radius-md)',
                  border: isDemoMode ? '1px solid #facc15' : '1px solid var(--border-subtle)',
                  background: isDemoMode ? 'rgba(234, 179, 8, 0.16)' : 'rgba(255, 255, 255, 0.04)',
                  color: isDemoMode ? '#facc15' : '#94a3b8',
                  fontSize: '12px',
                  fontWeight: 700,
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                }}
              >
                {isDemoMode ? <RotateCcw size={14} /> : <Sparkles size={14} />}
                <span>{isDemoMode ? 'Reset to Standby (00:00)' : 'Test Simulation Mode'}</span>
              </button>
            )}

            {/* Copy Timers Button */}
            <button
              id="btn-copy-timers"
              className="btn-primary"
              onClick={copyTimersToChat}
              style={{ minWidth: '180px' }}
              title="Copies cooldowns formatted for League team chat"
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

      {/* Standby Explanation Banner (Only shown when not in a game and not testing) */}
      {!isLive && !isDemoMode && (
        <div
          style={{
            background: 'rgba(0, 200, 255, 0.05)',
            border: '1px solid rgba(0, 200, 255, 0.2)',
            borderRadius: '8px',
            padding: '12px 16px',
            display: 'flex',
            alignItems: 'center',
            gap: '12px',
            fontSize: '12px',
            color: '#94a3b8',
          }}
        >
          <Info size={18} color="var(--blue-primary)" style={{ flexShrink: 0 }} />
          <div>
            <strong style={{ color: '#fff' }}>Automatic Game Sync:</strong> The game clock and objective respawns stay in standby (00:00) until you load into Summoner's Rift. RiftCore connects automatically via Riot's Live Client API (Port 2999) without requiring manual interaction. Want to test timers right now? Click <strong style={{ color: '#facc15' }}>Test Simulation Mode</strong> above.
          </div>
        </div>
      )}

      {/* Demo Mode Alert Banner */}
      {isDemoMode && (
        <div
          style={{
            background: 'rgba(234, 179, 8, 0.08)',
            border: '1px solid rgba(234, 179, 8, 0.3)',
            borderRadius: '8px',
            padding: '12px 16px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            fontSize: '12px',
            color: '#facc15',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Sparkles size={16} />
            <span>
              <strong>Simulation Active:</strong> Timers are currently simulating an active match. Click any enemy spell below to test cooldown tracking.
            </span>
          </div>
          <button
            onClick={handleToggleDemoMode}
            style={{
              background: 'rgba(234, 179, 8, 0.2)',
              border: '1px solid #facc15',
              color: '#fff',
              fontSize: '11px',
              fontWeight: 700,
              padding: '4px 10px',
              borderRadius: '6px',
              cursor: 'pointer',
            }}
          >
            Stop Simulation
          </button>
        </div>
      )}

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
              background: `${csRating.color}18`,
              border: `1px solid ${csRating.color}44`,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <Award size={20} color={csRating.color} />
          </div>
          <div>
            <div style={{ fontSize: '11px', textTransform: 'uppercase', color: '#94a3b8', fontWeight: 700 }}>
              Challenger CS Benchmark
            </div>
            <div style={{ fontSize: '15px', fontWeight: 800, color: csRating.color }}>
              {csRating.tier} <span style={{ fontSize: '12px', color: '#94a3b8' }}>({csRating.diff})</span>
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
            <Compass size={20} color="var(--red-primary)" />
          </div>
          <div>
            <div style={{ fontSize: '11px', textTransform: 'uppercase', color: '#94a3b8', fontWeight: 700 }}>
              Target Minion Curve
            </div>
            <div style={{ fontSize: '13px', fontWeight: 700, color: '#fff' }}>
              10m: <strong style={{ color: 'var(--blue-primary)' }}>80 CS</strong> • 15m: <strong style={{ color: 'var(--blue-primary)' }}>135 CS</strong> • 20m: <strong style={{ color: 'var(--blue-primary)' }}>190 CS</strong>
            </div>
          </div>
        </div>
      </div>

      {/* Objectives Navigation Tabs */}
      <div style={{ display: 'flex', gap: '10px', alignItems: 'center' }}>
        <button
          onClick={() => setActiveTab('epics')}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
            padding: '6px 14px',
            borderRadius: '6px',
            background: activeTab === 'epics' ? 'var(--blue-subtle)' : 'rgba(255, 255, 255, 0.04)',
            border: activeTab === 'epics' ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
            color: activeTab === 'epics' ? 'var(--blue-primary)' : '#94a3b8',
            fontSize: '12px',
            fontWeight: 700,
            cursor: 'pointer',
          }}
        >
          <Timer size={14} />
          <span>Epic Monsters (Dragon, Grubs, Baron)</span>
        </button>

        <button
          onClick={() => setActiveTab('jungle')}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
            padding: '6px 14px',
            borderRadius: '6px',
            background: activeTab === 'jungle' ? 'var(--blue-subtle)' : 'rgba(255, 255, 255, 0.04)',
            border: activeTab === 'jungle' ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
            color: activeTab === 'jungle' ? 'var(--blue-primary)' : '#94a3b8',
            fontSize: '12px',
            fontWeight: 700,
            cursor: 'pointer',
          }}
        >
          <Layers size={14} />
          <span>Jungle Camp Respawns (Buffs & Camps)</span>
        </button>
      </div>

      {/* EPIC OBJECTIVES TAB */}
      {activeTab === 'epics' && (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '16px' }}>
          {objectives.map((obj) => {
            const isStandby = !isLive && !isDemoMode;
            const progress = isStandby ? 0 : Math.max(0, Math.min(100, ((obj.total - obj.respawnTime) / obj.total) * 100));

            return (
              <div
                key={obj.name}
                className="glass-card"
                style={{
                  padding: '16px 20px',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '12px',
                  border: `1px solid ${obj.color}33`,
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                    <span style={{ fontSize: '24px' }}>{obj.icon}</span>
                    <div>
                      <div style={{ fontSize: '14px', fontWeight: 700, color: '#fff' }}>{obj.name}</div>
                      <div style={{ fontSize: '11px', color: '#64748b' }}>
                        {obj.id === 'baron' ? 'Spawns at 20:00' : 'Spawns at 05:00 • 5m Respawn'}
                      </div>
                    </div>
                  </div>
                  <div
                    style={{
                      fontFamily: 'var(--font-mono)',
                      fontSize: '17px',
                      fontWeight: 800,
                      color: isStandby ? '#64748b' : obj.color,
                    }}
                  >
                    {isStandby
                      ? obj.id === 'baron'
                        ? '20:00'
                        : '05:00'
                      : formatTime(obj.respawnTime)}
                  </div>
                </div>

                {/* Progress bar */}
                <div
                  style={{
                    height: '4px',
                    width: '100%',
                    background: 'rgba(255, 255, 255, 0.05)',
                    borderRadius: '2px',
                    overflow: 'hidden',
                  }}
                >
                  <div
                    style={{
                      height: '100%',
                      width: `${progress}%`,
                      background: obj.color,
                      transition: 'width 1s linear',
                    }}
                  />
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* JUNGLE CAMPS TAB */}
      {activeTab === 'jungle' && (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '12px' }}>
          {jungleCamps.map((camp, idx) => (
            <button
              key={camp.name}
              onClick={() => triggerCampTimer(idx)}
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '12px 14px',
                borderRadius: '8px',
                background: camp.active ? 'rgba(0, 200, 255, 0.12)' : 'rgba(255, 255, 255, 0.03)',
                border: camp.active ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                cursor: 'pointer',
                textAlign: 'left',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <span style={{ fontSize: '20px' }}>{camp.icon}</span>
                <div>
                  <div style={{ fontSize: '12px', fontWeight: 700, color: '#fff' }}>{camp.name}</div>
                  <div style={{ fontSize: '10px', color: '#64748b' }}>
                    {camp.active ? 'Respawning...' : 'Click when cleared'}
                  </div>
                </div>
              </div>

              <div
                style={{
                  fontFamily: 'var(--font-mono)',
                  fontSize: '13px',
                  fontWeight: 800,
                  color: camp.active ? 'var(--blue-primary)' : '#22c55e',
                }}
              >
                {camp.active ? formatTime(camp.remaining) : 'READY'}
              </div>
            </button>
          ))}
        </div>
      )}

      {/* ENEMY SUMMONER SPELL TRACKER */}
      <div className="glass-card" style={{ padding: '20px' }}>
        <div className="card-header" style={{ marginBottom: '16px' }}>
          <div className="card-title">
            <ShieldAlert size={18} color="var(--red-primary)" />
            <span>Enemy Summoner Spell Cooldown Tracker</span>
          </div>
          <span style={{ fontSize: '11px', color: '#64748b' }}>
            Click when an enemy uses a spell to start real-time countdown
          </span>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
          {['TOP', 'JUNGLE', 'MID', 'ADC', 'SUPPORT'].map((role) => {
            const roleSpells = enemySpells
              .map((s, idx) => ({ ...s, index: idx }))
              .filter((s) => s.role === role);

            return (
              <div
                key={role}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  padding: '8px 14px',
                  borderRadius: '8px',
                  background: 'rgba(255, 255, 255, 0.02)',
                  border: '1px solid var(--border-subtle)',
                }}
              >
                {/* Role and Champion */}
                <div style={{ display: 'flex', alignItems: 'center', gap: '12px', minWidth: '160px' }}>
                  <span
                    style={{
                      fontSize: '10px',
                      fontWeight: 800,
                      padding: '2px 6px',
                      borderRadius: '4px',
                      background: 'rgba(255, 42, 95, 0.1)',
                      color: 'var(--red-primary)',
                      border: '1px solid var(--border-red)',
                    }}
                  >
                    {role}
                  </span>
                  <span style={{ fontSize: '13px', fontWeight: 700, color: '#fff' }}>
                    {roleSpells[0]?.champion || 'Enemy'}
                  </span>
                </div>

                {/* Spell buttons */}
                <div style={{ display: 'flex', gap: '10px' }}>
                  {roleSpells.map((s) => (
                    <button
                      key={s.index}
                      onClick={() => triggerSpellCooldown(s.index)}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '8px',
                        padding: '6px 12px',
                        borderRadius: '6px',
                        background: s.timerActive ? 'rgba(255, 42, 95, 0.15)' : 'rgba(255, 255, 255, 0.04)',
                        border: s.timerActive ? '1px solid var(--red-primary)' : '1px solid var(--border-subtle)',
                        cursor: 'pointer',
                        minWidth: '105px',
                        transition: 'all 0.15s ease',
                      }}
                    >
                      <img
                        src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/spell/Summoner${s.spellName === 'Ghost' ? 'Haste' : s.spellName === 'Ignite' ? 'Dot' : s.spellName === 'Cleanse' ? 'Boost' : s.spellName}.png`}
                        alt={s.spellName}
                        style={{ width: '22px', height: '22px', borderRadius: '4px' }}
                        onError={(e) => { (e.target as any).style.display = 'none'; }}
                      />
                      <div style={{ textAlign: 'left' }}>
                        <div style={{ fontSize: '11px', fontWeight: 700, color: '#fff' }}>
                          {s.spellName}
                        </div>
                        <div
                          style={{
                            fontSize: '10px',
                            fontFamily: 'var(--font-mono)',
                            color: s.timerActive ? 'var(--red-primary)' : '#22c55e',
                            fontWeight: 700,
                          }}
                        >
                          {s.timerActive ? formatTime(s.cooldownRemaining) : 'READY'}
                        </div>
                      </div>
                    </button>
                  ))}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
