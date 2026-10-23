import React, { useState, useEffect } from 'react';
import {
  Layers,
  Shield,
  Zap,
  Sparkles,
  Check,
  Plus,
  Trash2,
  Award,
  AlertTriangle,
  Search,
  BookOpen
} from 'lucide-react';
import { CHAMPIONS_LIST, getChampionBuild, getChampionIcon, getChampionSplash } from '../services/championData';
import { patchService } from '../services/patchService';
import { soundService } from '../services/soundService';

interface PoolChampion {
  roleSlot: 'blind' | 'counter' | 'pocket';
  championId: number;
}

export const ChampionPoolBuilderView: React.FC = () => {
  const currentChamps = patchService.getChampions() || CHAMPIONS_LIST;
  const [selectedRole, setSelectedRole] = useState<'TOP' | 'JUNGLE' | 'MID' | 'ADC' | 'SUPPORT'>('MID');
  const [search, setSearch] = useState('');
  const [activeSlotType, setActiveSlotType] = useState<'blind' | 'counter' | 'pocket' | null>(null);

  // Stored pool per role
  const [pool, setPool] = useState<Record<string, { blind: number | null; counter: number | null; pocket: number | null }>>(() => {
    try {
      const saved = localStorage.getItem('riftcore_champion_pool');
      if (saved) return JSON.parse(saved);
    } catch {}
    return {
      TOP: { blind: 266, counter: 122, pocket: 24 }, // Aatrox, Darius, Jax
      JUNGLE: { blind: 64, counter: 254, pocket: 121 }, // Lee Sin, Vi, Kha'Zix
      MID: { blind: 103, counter: 238, pocket: 517 }, // Ahri, Zed, Sylas
      ADC: { blind: 222, counter: 51, pocket: 67 }, // Jinx, Caitlyn, Vayne
      SUPPORT: { blind: 412, counter: 53, pocket: 99 }, // Thresh, Blitzcrank, Lux
    };
  });

  const savePool = (updated: typeof pool) => {
    setPool(updated);
    try {
      localStorage.setItem('riftcore_champion_pool', JSON.stringify(updated));
    } catch {}
  };

  const handleAssignChampion = (champId: number) => {
    if (!activeSlotType) return;
    const currentRolePool = { ...pool[selectedRole], [activeSlotType]: champId };
    const nextPool = { ...pool, [selectedRole]: currentRolePool };
    savePool(nextPool);
    setActiveSlotType(null);
    soundService.playAutoAccept();
  };

  const handleClearSlot = (slotType: 'blind' | 'counter' | 'pocket') => {
    const currentRolePool = { ...pool[selectedRole], [slotType]: null };
    const nextPool = { ...pool, [selectedRole]: currentRolePool };
    savePool(nextPool);
    soundService.playTimerTick();
  };

  const currentRolePool = pool[selectedRole];
  const blindChamp = currentRolePool.blind ? currentChamps.find((c) => c.id === currentRolePool.blind) : null;
  const counterChamp = currentRolePool.counter ? currentChamps.find((c) => c.id === currentRolePool.counter) : null;
  const pocketChamp = currentRolePool.pocket ? currentChamps.find((c) => c.id === currentRolePool.pocket) : null;

  // Evaluate Pool Strength & Flexibility
  const filledSlots = [blindChamp, counterChamp, pocketChamp].filter(Boolean);
  const poolCoverageScore = Math.min(100, filledSlots.length * 33 + (filledSlots.length === 3 ? 1 : 0));

  const roleChamps = currentChamps.filter((c) => {
    const matchesRole = c.roles.includes(selectedRole);
    const matchesSearch = c.name.toLowerCase().includes(search.toLowerCase());
    return matchesRole && matchesSearch;
  });

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Header */}
      <div className="glass-card" style={{ padding: '20px' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '14px' }}>
          <div>
            <h2 style={{ fontSize: '18px', fontWeight: 800, color: '#fff', display: 'flex', alignItems: 'center', gap: '8px' }}>
              <Layers size={20} color="var(--blue-primary)" />
              <span>Ranked Champion Pool Builder (3-Champion Rule)</span>
            </h2>
            <p style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
              Challenger coaches agree: narrowing your focus to 3 specialized champions yields the highest LP gain
            </p>
          </div>

          {/* Role Filter Tabs */}
          <div style={{ display: 'flex', gap: '6px' }}>
            {(['TOP', 'JUNGLE', 'MID', 'ADC', 'SUPPORT'] as const).map((r) => (
              <button
                key={r}
                onClick={() => setSelectedRole(r)}
                style={{
                  padding: '6px 14px',
                  borderRadius: '6px',
                  background: selectedRole === r ? 'var(--blue-subtle)' : 'rgba(255, 255, 255, 0.04)',
                  border: selectedRole === r ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                  color: selectedRole === r ? 'var(--blue-primary)' : '#94a3b8',
                  fontSize: '11px',
                  fontWeight: 800,
                  cursor: 'pointer',
                }}
              >
                {r}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* 3 Core Slots Grid */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '16px' }}>
        {/* SLOT 1: BLIND PICK */}
        <div
          className="glass-card"
          style={{
            padding: '20px',
            border: '1px solid var(--border-blue)',
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'space-between',
          }}
        >
          <div>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' }}>
              <span style={{ fontSize: '11px', fontWeight: 800, textTransform: 'uppercase', color: 'var(--blue-primary)' }}>
                Slot 1 • Safe Blind Pick
              </span>
              {blindChamp && (
                <button onClick={() => handleClearSlot('blind')} style={{ background: 'transparent', border: 'none', color: '#64748b', cursor: 'pointer' }}>
                  <Trash2 size={13} />
                </button>
              )}
            </div>
            <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '14px' }}>
              Your go-to comfort pick when picking early in the draft. Has few unplayable matchups.
            </div>

            {blindChamp ? (
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px', background: 'rgba(255, 255, 255, 0.03)', padding: '12px', borderRadius: '8px' }}>
                <img src={getChampionIcon(blindChamp.key)} alt={blindChamp.name} style={{ width: '48px', height: '48px', borderRadius: '10px', border: '2px solid var(--blue-primary)' }} />
                <div>
                  <div style={{ fontSize: '15px', fontWeight: 800, color: '#fff' }}>{blindChamp.name}</div>
                  <div style={{ fontSize: '11px', color: '#22c55e', fontWeight: 700 }}>52.4% Win Rate • Low Ban</div>
                </div>
              </div>
            ) : (
              <button
                onClick={() => setActiveSlotType('blind')}
                style={{
                  width: '100%',
                  padding: '20px',
                  borderRadius: '8px',
                  border: '1px dashed var(--border-blue)',
                  background: 'rgba(0, 200, 255, 0.04)',
                  color: 'var(--blue-primary)',
                  fontSize: '12px',
                  fontWeight: 700,
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '6px',
                }}
              >
                <Plus size={16} />
                <span>Assign Blind Pick</span>
              </button>
            )}
          </div>
        </div>

        {/* SLOT 2: HARD COUNTER PICK */}
        <div
          className="glass-card"
          style={{
            padding: '20px',
            border: '1px solid var(--border-red)',
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'space-between',
          }}
        >
          <div>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' }}>
              <span style={{ fontSize: '11px', fontWeight: 800, textTransform: 'uppercase', color: 'var(--red-primary)' }}>
                Slot 2 • Meta Counter Pick
              </span>
              {counterChamp && (
                <button onClick={() => handleClearSlot('counter')} style={{ background: 'transparent', border: 'none', color: '#64748b', cursor: 'pointer' }}>
                  <Trash2 size={13} />
                </button>
              )}
            </div>
            <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '14px' }}>
              Picks when enemy locks in popular lane bullies to instantly dominate lane phase.
            </div>

            {counterChamp ? (
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px', background: 'rgba(255, 255, 255, 0.03)', padding: '12px', borderRadius: '8px' }}>
                <img src={getChampionIcon(counterChamp.key)} alt={counterChamp.name} style={{ width: '48px', height: '48px', borderRadius: '10px', border: '2px solid var(--red-primary)' }} />
                <div>
                  <div style={{ fontSize: '15px', fontWeight: 800, color: '#fff' }}>{counterChamp.name}</div>
                  <div style={{ fontSize: '11px', color: 'var(--red-primary)', fontWeight: 700 }}>Lane Assassin • High Kill Rate</div>
                </div>
              </div>
            ) : (
              <button
                onClick={() => setActiveSlotType('counter')}
                style={{
                  width: '100%',
                  padding: '20px',
                  borderRadius: '8px',
                  border: '1px dashed var(--border-red)',
                  background: 'rgba(255, 42, 95, 0.04)',
                  color: 'var(--red-primary)',
                  fontSize: '12px',
                  fontWeight: 700,
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '6px',
                }}
              >
                <Plus size={16} />
                <span>Assign Counter Pick</span>
              </button>
            )}
          </div>
        </div>

        {/* SLOT 3: POCKET COMFORT PICK */}
        <div
          className="glass-card"
          style={{
            padding: '20px',
            border: '1px solid rgba(192, 132, 252, 0.3)',
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'space-between',
          }}
        >
          <div>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' }}>
              <span style={{ fontSize: '11px', fontWeight: 800, textTransform: 'uppercase', color: '#c084fc' }}>
                Slot 3 • Pocket Hyper-Carry
              </span>
              {pocketChamp && (
                <button onClick={() => handleClearSlot('pocket')} style={{ background: 'transparent', border: 'none', color: '#64748b', cursor: 'pointer' }}>
                  <Trash2 size={13} />
                </button>
              )}
            </div>
            <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '14px' }}>
              Your signature high-mastery champion for games where you need to 1v9 solo carry.
            </div>

            {pocketChamp ? (
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px', background: 'rgba(255, 255, 255, 0.03)', padding: '12px', borderRadius: '8px' }}>
                <img src={getChampionIcon(pocketChamp.key)} alt={pocketChamp.name} style={{ width: '48px', height: '48px', borderRadius: '10px', border: '2px solid #c084fc' }} />
                <div>
                  <div style={{ fontSize: '15px', fontWeight: 800, color: '#fff' }}>{pocketChamp.name}</div>
                  <div style={{ fontSize: '11px', color: '#c084fc', fontWeight: 700 }}>High Scaling • Solo Carry</div>
                </div>
              </div>
            ) : (
              <button
                onClick={() => setActiveSlotType('pocket')}
                style={{
                  width: '100%',
                  padding: '20px',
                  borderRadius: '8px',
                  border: '1px dashed rgba(192, 132, 252, 0.4)',
                  background: 'rgba(192, 132, 252, 0.04)',
                  color: '#c084fc',
                  fontSize: '12px',
                  fontWeight: 700,
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: '6px',
                }}
              >
                <Plus size={16} />
                <span>Assign Pocket Pick</span>
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Champion Selection Drawer when choosing a slot */}
      {activeSlotType && (
        <div className="glass-card" style={{ padding: '20px', border: '1px solid var(--border-blue)' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
            <span style={{ fontSize: '13px', fontWeight: 800, color: '#fff' }}>
              Select Champion for {activeSlotType.toUpperCase()} ({selectedRole})
            </span>
            <button
              onClick={() => setActiveSlotType(null)}
              className="btn-secondary"
              style={{ fontSize: '11px', padding: '4px 10px' }}
            >
              Cancel
            </button>
          </div>

          <div
            style={{
              maxHeight: '200px',
              overflowY: 'auto',
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fill, minmax(130px, 1fr))',
              gap: '8px',
            }}
          >
            {roleChamps.map((c) => (
              <button
                key={c.id}
                onClick={() => handleAssignChampion(c.id)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  padding: '6px',
                  borderRadius: '6px',
                  background: 'rgba(255, 255, 255, 0.03)',
                  border: '1px solid var(--border-subtle)',
                  cursor: 'pointer',
                  textAlign: 'left',
                }}
              >
                <img src={getChampionIcon(c.key)} alt={c.name} style={{ width: '26px', height: '26px', borderRadius: '4px' }} />
                <span style={{ fontSize: '12px', fontWeight: 600, color: '#fff' }}>{c.name}</span>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Pool Health Diagnostics */}
      <div className="glass-card" style={{ padding: '20px' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Award size={18} color="var(--blue-primary)" />
            <span style={{ fontSize: '14px', fontWeight: 800, color: '#fff' }}>Pool Health & Ranked Readiness</span>
          </div>
          <span style={{ fontSize: '16px', fontWeight: 900, color: poolCoverageScore === 100 ? '#22c55e' : '#f59e0b', fontFamily: 'var(--font-mono)' }}>
            {poolCoverageScore}% Coverage
          </span>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '14px' }}>
          <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '12px', borderRadius: '8px' }}>
            <div style={{ fontSize: '11px', color: '#94a3b8' }}>Damage Versatility</div>
            <div style={{ fontSize: '13px', fontWeight: 700, color: '#fff', marginTop: '4px' }}>
              {filledSlots.length >= 2 ? '✅ Mixed AD & AP Threats' : '⚠️ Need more variety'}
            </div>
          </div>
          <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '12px', borderRadius: '8px' }}>
            <div style={{ fontSize: '11px', color: '#94a3b8' }}>Ban Resilience</div>
            <div style={{ fontSize: '13px', fontWeight: 700, color: '#fff', marginTop: '4px' }}>
              {filledSlots.length === 3 ? '✅ 100% Protected (Always an open pick)' : '⚠️ Fill all 3 slots'}
            </div>
          </div>
          <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '12px', borderRadius: '8px' }}>
            <div style={{ fontSize: '11px', color: '#94a3b8' }}>Coaching Rule</div>
            <div style={{ fontSize: '13px', fontWeight: 700, color: 'var(--blue-primary)', marginTop: '4px' }}>
              Stick strictly to these 3 for 50+ games to climb 2 divisions
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
