import React, { useState, useEffect } from 'react';
import {
  Crosshair,
  Clock,
  Copy,
  Check,
  ShieldAlert,
  Play,
  RotateCcw
} from 'lucide-react';
import { LiveGameData } from '../types';
import { SUMMONER_SPELLS } from '../services/runeData';
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
  const [gameTime, setGameTime] = useState(745);
  const [copied, setCopied] = useState(false);

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
    { name: 'Hextech Dragon', respawnTime: 140, total: 300, icon: '🐲', color: '#00c8ff' },
    { name: 'Voidgrubs / Herald', respawnTime: 210, total: 360, icon: '👾', color: '#ff2a5f' },
    { name: 'Baron Nashor', respawnTime: 455, total: 1200, icon: '👑', color: '#00c8ff' },
  ]);

  useEffect(() => {
    const timer = setInterval(() => {
      setGameTime((prev) => prev + 1);

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

      setObjectives((prevObjs) =>
        prevObjs.map((obj) => ({
          ...obj,
          respawnTime: obj.respawnTime > 0 ? obj.respawnTime - 1 : obj.total,
        }))
      );
    }, 1000);

    return () => clearInterval(timer);
  }, []);

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
        target.readyGameTime = gameTime + target.cooldownTotal;
        soundService.playTimerTick();
      }
      return copy;
    });
  };

  const copyTimersToChat = () => {
    const activeCooldowns = enemySpells.filter((s) => s.timerActive);
    let message = '';
    if (activeCooldowns.length === 0) {
      message = 'All enemy spells up!';
    } else {
      message = activeCooldowns
        .map((s) => {
          const readyMin = Math.floor(s.readyGameTime / 60);
          const readySec = Math.floor(s.readyGameTime % 60);
          const timeFormatted = `${readyMin.toString().padStart(2, '0')}:${readySec.toString().padStart(2, '0')}`;
          return `${s.role} ${s.spellName} [${timeFormatted}]`;
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

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Live Match Top Banner */}
      <div className="glass-card" style={{ padding: '16px 24px', border: '1px solid var(--border-blue)' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '12px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '14px' }}>
            <div
              style={{
                width: '42px',
                height: '42px',
                borderRadius: '50%',
                background: 'var(--blue-subtle)',
                border: '1px solid var(--blue-primary)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <Crosshair size={22} color="var(--blue-primary)" />
            </div>
            <div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <h2 style={{ fontSize: '18px', fontWeight: 800, color: '#fff' }}>
                  Live Match Companion
                </h2>
                <span
                  style={{
                    fontSize: '10px',
                    fontWeight: 700,
                    textTransform: 'uppercase',
                    padding: '2px 8px',
                    borderRadius: '4px',
                    background: liveData ? 'var(--blue-subtle)' : 'var(--red-subtle)',
                    color: liveData ? 'var(--blue-primary)' : 'var(--red-primary)',
                    border: liveData ? '1px solid var(--blue-primary)' : '1px solid var(--red-primary)',
                  }}
                >
                  {liveData ? 'Live Match Synced' : 'Companion Active'}
                </span>
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8' }}>
                Summoner's Rift • Game Clock: <strong style={{ color: 'var(--blue-primary)', fontFamily: 'var(--font-mono)' }}>{formatTime(gameTime)}</strong>
              </div>
            </div>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <button
              id="btn-copy-timers"
              className="btn-primary"
              onClick={copyTimersToChat}
              style={{ minWidth: '180px' }}
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

      {/* Objective Respawn Timers */}
      <div className="glass-card">
        <div className="card-header">
          <div className="card-title">
            <Clock size={16} color="var(--blue-primary)" />
            <span>Epic Monster Respawn Timers</span>
          </div>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '14px' }}>
          {objectives.map((obj) => {
            const percent = Math.max(0, Math.min(100, (1 - obj.respawnTime / obj.total) * 100));
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
                <div style={{ fontSize: '24px' }}>{obj.icon}</div>
                <div style={{ flex: 1 }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '6px' }}>
                    <span style={{ fontSize: '13px', fontWeight: 600, color: '#fff' }}>{obj.name}</span>
                    <span style={{ fontFamily: 'var(--font-mono)', fontSize: '13px', fontWeight: 700, color: obj.color }}>
                      {formatTime(obj.respawnTime)}
                    </span>
                  </div>
                  <div style={{ width: '100%', height: '4px', background: 'rgba(255, 255, 255, 0.08)', borderRadius: '4px', overflow: 'hidden' }}>
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
          <span style={{ fontSize: '11px', color: '#94a3b8' }}>
            Click when an enemy uses a spell to start real-time countdown
          </span>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
          {['TOP', 'JUNGLE', 'MID', 'ADC', 'SUPPORT'].map((role) => {
            const roleSpells = enemySpells.filter((s) => s.role === role);
            const champName = roleSpells[0]?.champion || 'Enemy';

            return (
              <div
                key={role}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  background: 'rgba(255, 255, 255, 0.02)',
                  border: '1px solid var(--border-subtle)',
                  borderRadius: 'var(--radius-md)',
                  padding: '10px 16px',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '14px', minWidth: '160px' }}>
                  <span
                    style={{
                      fontSize: '11px',
                      fontWeight: 800,
                      padding: '2px 8px',
                      borderRadius: '4px',
                      background: 'var(--red-subtle)',
                      color: 'var(--red-primary)',
                      border: '1px solid var(--border-red)',
                      width: '60px',
                      textAlign: 'center',
                    }}
                  >
                    {role}
                  </span>
                  <span style={{ fontSize: '13px', fontWeight: 600, color: '#fff' }}>{champName}</span>
                </div>

                <div style={{ display: 'flex', gap: '14px' }}>
                  {roleSpells.map((spell) => {
                    const globalIdx = enemySpells.findIndex(
                      (s) => s.role === spell.role && s.spellId === spell.spellId
                    );
                    const sp = SUMMONER_SPELLS[spell.spellId];
                    return (
                      <button
                        key={spell.spellId}
                        id={`btn-track-spell-${role.toLowerCase()}-${spell.spellName.toLowerCase()}`}
                        onClick={() => triggerSpellCooldown(globalIdx)}
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          gap: '10px',
                          background: spell.timerActive ? 'var(--red-subtle)' : 'rgba(255, 255, 255, 0.04)',
                          border: spell.timerActive ? '1px solid var(--red-primary)' : '1px solid var(--border-subtle)',
                          padding: '6px 14px',
                          borderRadius: '8px',
                          cursor: 'pointer',
                          minWidth: '140px',
                          transition: 'all 0.15s ease',
                        }}
                      >
                        <div style={{ position: 'relative' }}>
                          <img
                            src={sp?.icon}
                            alt={spell.spellName}
                            style={{
                              width: '28px',
                              height: '28px',
                              borderRadius: '6px',
                              filter: spell.timerActive ? 'grayscale(0.8)' : 'none',
                            }}
                          />
                          {spell.timerActive && (
                            <div
                              style={{
                                position: 'absolute',
                                top: 0,
                                left: 0,
                                right: 0,
                                bottom: 0,
                                background: 'rgba(0, 0, 0, 0.5)',
                                borderRadius: '6px',
                              }}
                            />
                          )}
                        </div>

                        <div style={{ textAlign: 'left' }}>
                          <div style={{ fontSize: '12px', fontWeight: 700, color: '#fff' }}>
                            {spell.spellName}
                          </div>
                          <div
                            style={{
                              fontFamily: 'var(--font-mono)',
                              fontSize: '11px',
                              fontWeight: 700,
                              color: spell.timerActive ? 'var(--red-primary)' : 'var(--blue-primary)',
                            }}
                          >
                            {spell.timerActive ? `${spell.cooldownRemaining}s` : 'READY'}
                          </div>
                        </div>
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
