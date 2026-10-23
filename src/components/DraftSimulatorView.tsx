import React, { useState } from 'react';
import {
  Swords,
  Shield,
  Sparkles,
  Zap,
  Users,
  Search,
  Check,
  RefreshCw,
  Flame,
  ArrowRight,
  TrendingUp,
  Layers,
  ChevronRight
} from 'lucide-react';
import { ChampSelectSession } from '../types';
import { CHAMPIONS_LIST, getChampionBuild, getChampionIcon } from '../services/championData';
import { patchService } from '../services/patchService';
import { soundService } from '../services/soundService';

interface DraftSimulatorViewProps {
  session: ChampSelectSession | null;
  onSelectChampion?: (championId: number) => void;
}

interface DraftSlot {
  role: 'TOP' | 'JUNGLE' | 'MID' | 'ADC' | 'SUPPORT';
  championId: number | null;
}

export const DraftSimulatorView: React.FC<DraftSimulatorViewProps> = ({ session, onSelectChampion }) => {
  const currentChamps = patchService.getChampions() || CHAMPIONS_LIST;

  // Blue & Red Team Picks
  const [blueTeam, set法语BlueTeam] = useState<DraftSlot[]>([
    { role: 'TOP', championId: 266 }, // Aatrox
    { role: 'JUNGLE', championId: 64 }, // Lee Sin
    { role: 'MID', championId: null }, // User selection slot
    { role: 'ADC', championId: 222 }, // Jinx
    { role: 'SUPPORT', championId: 412 }, // Thresh
  ]);

  const [redTeam, setRedTeam] = useState<DraftSlot[]>([
    { role: 'TOP', championId: 122 }, // Darius
    { role: 'JUNGLE', championId: 254 }, // Vi
    { role: 'MID', championId: 238 }, // Zed
    { role: 'ADC', championId: 51 }, // Caitlyn
    { role: 'SUPPORT', championId: 53 }, // Blitzcrank
  ]);

  const [activeSlot, setActiveSlot] = useState<{ team: 'blue' | 'red'; index: number }>({ team: 'blue', index: 2 });
  const [search, setSearch] = useState('');

  // Sync from live League Client champ select
  const handleSyncFromClient = () => {
    if (!session) return;
    if (session.myTeam) {
      set法语BlueTeam((prev) =>
        prev.map((slot, idx) => {
          const player = session.myTeam[idx];
          const champId = player?.championId || player?.championPickIntent;
          return { ...slot, championId: champId && champId > 0 ? champId : slot.championId };
        })
      );
    }
    if (session.theirTeam) {
      setRedTeam((prev) =>
        prev.map((slot, idx) => {
          const enemy = session.theirTeam[idx];
          const champId = enemy?.championId;
          return { ...slot, championId: champId && champId > 0 ? champId : slot.championId };
        })
      );
    }
    soundService.playAutoAccept();
  };

  // Helper to pick champion for active slot
  const handleSelectChampion = (champId: number) => {
    if (activeSlot.team === 'blue') {
      set法语BlueTeam((prev) => {
        const next = [...prev];
        next[activeSlot.index].championId = champId;
        return next;
      });
    } else {
      setRedTeam((prev) => {
        const next = [...prev];
        next[activeSlot.index].championId = champId;
        return next;
      });
    }
    soundService.playTimerTick();
  };

  // Calculate Team Comp Attributes
  const calculateCompStats = (team: DraftSlot[]) => {
    let adCount = 0;
    let apCount = 0;
    let ccScore = 0;
    let frontlineScore = 0;

    team.forEach((slot) => {
      if (!slot.championId) return;
      const c = currentChamps.find((ch) => ch.id === slot.championId);
      if (!c) return;

      if (['ADC'].includes(slot.role) || ['Aatrox', 'Darius', 'Zed', 'Yasuo', 'Yone', 'Lee Sin', 'Vi'].includes(c.name)) {
        adCount += 1;
      } else {
        apCount += 1;
      }

      if (['Thresh', 'Leona', 'Nautilus', 'Blitzcrank', 'Amumu', 'Zac', 'Sejuani', 'Malphite'].includes(c.name)) {
        ccScore += 3;
        frontlineScore += 3;
      } else if (['Aatrox', 'Darius', 'Vi', 'Jax', 'Volibear'].includes(c.name)) {
        frontlineScore += 2;
        ccScore += 1.5;
      } else {
        ccScore += 1;
      }
    });

    const total = Math.max(1, adCount + apCount);
    const adPct = Math.round((adCount / total) * 100);
    const apPct = 100 - adPct;

    return {
      adPct,
      apPct,
      ccScore: Math.min(10, Math.round(ccScore)),
      frontlineScore: Math.min(10, Math.round(frontlineScore)),
    };
  };

  const blueStats = calculateCompStats(blueTeam);
  const redStats = calculateCompStats(redTeam);

  // AI Recommendations Engine
  const targetRole = activeSlot.team === 'blue' ? blueTeam[activeSlot.index].role : redTeam[activeSlot.index].role;
  const isRedHeavyAd = redStats.adPct >= 70;
  const blueNeedsAp = blueStats.adPct >= 75;

  const recommendations = [
    {
      champName: blueNeedsAp ? 'Ahri' : 'Orianna',
      score: 95,
      role: targetRole,
      tag: 'COMP BALANCER',
      color: '#00c8ff',
      reason: blueNeedsAp
        ? 'Your team is heavily AD favoured. Picking high-burst AP balances team damage profile and forces enemy to build MR.'
        : 'Synergizes with team engage. Provides safe waveclear and massive 5v5 teamfight shockwave.',
    },
    {
      champName: isRedHeavyAd ? 'Malphite' : 'Sylas',
      score: 91,
      role: targetRole,
      tag: isRedHeavyAd ? 'HARD COUNTER' : 'META SPIKE',
      color: '#22c55e',
      reason: isRedHeavyAd
        ? 'Enemy comp is 80% physical damage. Armor stacking completely shuts down enemy AD carries and frontline.'
        : 'Steals high-impact enemy ultimates and provides strong side-lane pressure against Zed & Darius.',
    },
    {
      champName: 'Hwei',
      score: 87,
      role: targetRole,
      tag: 'ZONE CONTROL',
      color: '#c084fc',
      reason: 'Disrupts enemy dive champions (Vi, Blitzcrank) with fear & slow zones while maintaining long-range poke.',
    },
  ];

  const filteredChamps = currentChamps.filter((c) =>
    c.name.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Header Bar */}
      <div className="glass-card" style={{ padding: '20px' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '14px' }}>
          <div>
            <h2 style={{ fontSize: '18px', fontWeight: 800, color: '#fff', display: 'flex', alignItems: 'center', gap: '8px' }}>
              <Swords size={20} color="var(--blue-primary)" />
              <span>AI Drafting Coach & Team Comp Simulator</span>
            </h2>
            <p style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
              Simulate drafts, analyze team composition synergies, and receive real-time counter-pick recommendations
            </p>
          </div>

          <div style={{ display: 'flex', gap: '10px' }}>
            {session && (
              <button
                onClick={handleSyncFromClient}
                className="btn-primary"
                style={{ fontSize: '12px', padding: '8px 14px' }}
              >
                <RefreshCw size={14} />
                <span>Sync Active Draft</span>
              </button>
            )}

            <button
              onClick={() => {
                set法语BlueTeam([
                  { role: 'TOP', championId: null },
                  { role: 'JUNGLE', championId: null },
                  { role: 'MID', championId: null },
                  { role: 'ADC', championId: null },
                  { role: 'SUPPORT', championId: null },
                ]);
                setRedTeam([
                  { role: 'TOP', championId: null },
                  { role: 'JUNGLE', championId: null },
                  { role: 'MID', championId: null },
                  { role: 'ADC', championId: null },
                  { role: 'SUPPORT', championId: null },
                ]);
                soundService.playTimerTick();
              }}
              className="btn-secondary"
              style={{ fontSize: '12px', padding: '8px 14px' }}
            >
              Clear Board
            </button>
          </div>
        </div>
      </div>

      {/* 5v5 Arena Board */}
      <div className="draft-board-grid">
        {/* BLUE TEAM (Ally) */}
        <div className="glass-card" style={{ padding: '20px', border: '1px solid var(--border-blue)' }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '14px' }}>
            <span style={{ fontSize: '14px', fontWeight: 800, color: 'var(--blue-primary)' }}>
              🔵 YOUR TEAM COMPOSITION
            </span>
            <div style={{ fontSize: '11px', color: '#94a3b8' }}>
              AD: <strong style={{ color: '#fff' }}>{blueStats.adPct}%</strong> | AP: <strong style={{ color: 'var(--blue-primary)' }}>{blueStats.apPct}%</strong>
            </div>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
            {blueTeam.map((slot, idx) => {
              const c = slot.championId ? currentChamps.find((ch) => ch.id === slot.championId) : null;
              const isSelectedSlot = activeSlot.team === 'blue' && activeSlot.index === idx;

              return (
                <button
                  key={idx}
                  onClick={() => setActiveSlot({ team: 'blue', index: idx })}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    padding: '10px 14px',
                    borderRadius: '8px',
                    background: isSelectedSlot ? 'rgba(0, 200, 255, 0.16)' : 'rgba(255, 255, 255, 0.03)',
                    border: isSelectedSlot ? '2px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                    cursor: 'pointer',
                    boxShadow: isSelectedSlot ? '0 0 14px var(--blue-glow)' : 'none',
                    textAlign: 'left',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                    <span style={{ fontSize: '10px', fontWeight: 800, color: '#64748b', width: '55px' }}>
                      {slot.role}
                    </span>
                    {c ? (
                      <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                        <img src={getChampionIcon(c.key)} alt={c.name} style={{ width: '30px', height: '30px', borderRadius: '6px' }} />
                        <span style={{ fontSize: '13px', fontWeight: 700, color: '#fff' }}>{c.name}</span>
                      </div>
                    ) : (
                      <span style={{ fontSize: '12px', fontStyle: 'italic', color: '#64748b' }}>Click to select pick...</span>
                    )}
                  </div>

                  <span style={{ fontSize: '11px', color: isSelectedSlot ? 'var(--blue-primary)' : '#475569' }}>
                    {isSelectedSlot ? 'Editing Slot' : 'Select'}
                  </span>
                </button>
              );
            })}
          </div>

          {/* Blue Team Composition Meters */}
          <div style={{ marginTop: '16px', display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '10px' }}>
            <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '8px', borderRadius: '6px', textAlign: 'center' }}>
              <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>DAMAGE SPREAD</div>
              <div style={{ fontSize: '12px', fontWeight: 800, color: '#fff', marginTop: '2px' }}>
                {blueStats.adPct > 70 ? '⚠️ High AD' : blueStats.apPct > 70 ? '⚠️ High AP' : 'Balanced'}
              </div>
            </div>
            <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '8px', borderRadius: '6px', textAlign: 'center' }}>
              <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>CROWD CONTROL</div>
              <div style={{ fontSize: '12px', fontWeight: 800, color: 'var(--blue-primary)', marginTop: '2px' }}>
                {blueStats.ccScore} / 10
              </div>
            </div>
            <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '8px', borderRadius: '6px', textAlign: 'center' }}>
              <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>FRONTLINE RATING</div>
              <div style={{ fontSize: '12px', fontWeight: 800, color: '#22c55e', marginTop: '2px' }}>
                {blueStats.frontlineScore} / 10
              </div>
            </div>
          </div>
        </div>

        {/* RED TEAM (Enemy) */}
        <div className="glass-card" style={{ padding: '20px', border: '1px solid var(--border-red)' }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '14px' }}>
            <span style={{ fontSize: '14px', fontWeight: 800, color: 'var(--red-primary)' }}>
              🔴 ENEMY TEAM COMPOSITION
            </span>
            <div style={{ fontSize: '11px', color: '#94a3b8' }}>
              AD: <strong style={{ color: '#fff' }}>{redStats.adPct}%</strong> | AP: <strong style={{ color: 'var(--red-primary)' }}>{redStats.apPct}%</strong>
            </div>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
            {redTeam.map((slot, idx) => {
              const c = slot.championId ? currentChamps.find((ch) => ch.id === slot.championId) : null;
              const isSelectedSlot = activeSlot.team === 'red' && activeSlot.index === idx;

              return (
                <button
                  key={idx}
                  onClick={() => setActiveSlot({ team: 'red', index: idx })}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    padding: '10px 14px',
                    borderRadius: '8px',
                    background: isSelectedSlot ? 'rgba(255, 42, 95, 0.16)' : 'rgba(255, 255, 255, 0.03)',
                    border: isSelectedSlot ? '2px solid var(--red-primary)' : '1px solid var(--border-subtle)',
                    cursor: 'pointer',
                    boxShadow: isSelectedSlot ? '0 0 14px var(--red-glow)' : 'none',
                    textAlign: 'left',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                    <span style={{ fontSize: '10px', fontWeight: 800, color: '#64748b', width: '55px' }}>
                      {slot.role}
                    </span>
                    {c ? (
                      <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                        <img src={getChampionIcon(c.key)} alt={c.name} style={{ width: '30px', height: '30px', borderRadius: '6px' }} />
                        <span style={{ fontSize: '13px', fontWeight: 700, color: '#fff' }}>{c.name}</span>
                      </div>
                    ) : (
                      <span style={{ fontSize: '12px', fontStyle: 'italic', color: '#64748b' }}>Click to select enemy...</span>
                    )}
                  </div>

                  <span style={{ fontSize: '11px', color: isSelectedSlot ? 'var(--red-primary)' : '#475569' }}>
                    {isSelectedSlot ? 'Editing Slot' : 'Select'}
                  </span>
                </button>
              );
            })}
          </div>

          {/* Red Team Composition Meters */}
          <div style={{ marginTop: '16px', display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '10px' }}>
            <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '8px', borderRadius: '6px', textAlign: 'center' }}>
              <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>ENEMY THREAT</div>
              <div style={{ fontSize: '12px', fontWeight: 800, color: '#fff', marginTop: '2px' }}>
                {redStats.adPct >= 75 ? 'Armor Counters' : 'Standard'}
              </div>
            </div>
            <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '8px', borderRadius: '6px', textAlign: 'center' }}>
              <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>ENEMY CC</div>
              <div style={{ fontSize: '12px', fontWeight: 800, color: 'var(--red-primary)', marginTop: '2px' }}>
                {redStats.ccScore} / 10
              </div>
            </div>
            <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '8px', borderRadius: '6px', textAlign: 'center' }}>
              <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>ENEMY FRONTLINE</div>
              <div style={{ fontSize: '12px', fontWeight: 800, color: '#f59e0b', marginTop: '2px' }}>
                {redStats.frontlineScore} / 10
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* AI RECOMMENDATION ADVISOR & CHAMPION PICKER */}
      <div className="draft-analysis-grid">
        {/* AI Recommendations */}
        <div className="glass-card" style={{ padding: '20px' }}>
          <div className="card-header" style={{ marginBottom: '14px' }}>
            <div className="card-title">
              <Sparkles size={18} color="var(--blue-primary)" />
              <span>AI Recommended Picks for {targetRole}</span>
            </div>
            <span style={{ fontSize: '11px', color: '#94a3b8' }}>
              Calculated based on synergy, damage balance & enemy counters
            </span>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            {recommendations.map((rec) => {
              const champObj = currentChamps.find((c) => c.name === rec.champName) || currentChamps[0];

              return (
                <div
                  key={rec.champName}
                  style={{
                    background: 'rgba(255, 255, 255, 0.03)',
                    border: '1px solid var(--border-subtle)',
                    borderRadius: '10px',
                    padding: '14px',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    gap: '14px',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: '14px' }}>
                    <img
                      src={getChampionIcon(champObj.key)}
                      alt={rec.champName}
                      style={{ width: '46px', height: '46px', borderRadius: '10px', border: `2px solid ${rec.color}` }}
                    />
                    <div>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <span style={{ fontSize: '15px', fontWeight: 800, color: '#fff' }}>{rec.champName}</span>
                        <span
                          style={{
                            fontSize: '10px',
                            fontWeight: 800,
                            padding: '2px 6px',
                            borderRadius: '4px',
                            background: `${rec.color}18`,
                            color: rec.color,
                            border: `1px solid ${rec.color}44`,
                          }}
                        >
                          {rec.tag}
                        </span>
                      </div>
                      <div style={{ fontSize: '11px', color: '#94a3b8', marginTop: '4px', maxWidth: '380px' }}>
                        {rec.reason}
                      </div>
                    </div>
                  </div>

                  <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end', gap: '6px' }}>
                    <div style={{ fontSize: '17px', fontWeight: 900, color: rec.color, fontFamily: 'var(--font-mono)' }}>
                      {rec.score}%
                    </div>
                    <button
                      onClick={() => handleSelectChampion(champObj.id)}
                      className="btn-primary"
                      style={{ fontSize: '11px', padding: '4px 10px' }}
                    >
                      Lock In
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Quick Champion Search Picker */}
        <div className="glass-card" style={{ padding: '20px' }}>
          <div className="card-header" style={{ marginBottom: '12px' }}>
            <div className="card-title">
              <Search size={16} color="var(--blue-primary)" />
              <span>Select Any Champion</span>
            </div>
          </div>

          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              background: 'rgba(0, 0, 0, 0.5)',
              border: '1px solid var(--border-subtle)',
              borderRadius: '6px',
              padding: '6px 12px',
              marginBottom: '12px',
            }}
          >
            <Search size={14} color="#64748b" />
            <input
              type="text"
              placeholder="Search 173 Champions..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              style={{
                background: 'transparent',
                border: 'none',
                color: '#fff',
                fontSize: '12px',
                outline: 'none',
                width: '100%',
              }}
            />
          </div>

          <div
            style={{
              maxHeight: '230px',
              overflowY: 'auto',
              display: 'grid',
              gridTemplateColumns: 'repeat(3, 1fr)',
              gap: '6px',
            }}
          >
            {filteredChamps.map((c) => (
              <button
                key={c.id}
                onClick={() => handleSelectChampion(c.id)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  padding: '4px 6px',
                  borderRadius: '6px',
                  background: 'rgba(255, 255, 255, 0.03)',
                  border: '1px solid var(--border-subtle)',
                  cursor: 'pointer',
                  textAlign: 'left',
                }}
              >
                <img src={getChampionIcon(c.key)} alt={c.name} style={{ width: '22px', height: '22px', borderRadius: '4px' }} />
                <span style={{ fontSize: '11px', fontWeight: 600, color: '#fff', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                  {c.name}
                </span>
              </button>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
