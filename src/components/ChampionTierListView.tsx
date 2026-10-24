import React, { useState, useEffect } from 'react';
import { Crown, Search, Zap, RefreshCw, TrendingUp, Filter, Sparkles, ChevronRight } from 'lucide-react';
import { CHAMPIONS_LIST, getChampionBuild, getChampionIcon } from '../services/championData';
import { patchService, PatchState } from '../services/patchService';

interface ChampionTierListViewProps {
  onSelectChampion: (championId: number) => void;
}

export const ChampionTierListView: React.FC<ChampionTierListViewProps> = ({ onSelectChampion }) => {
  const [patchState, setPatchState] = useState<PatchState>(patchService.getPatchState());
  const [activeRole, setActiveRole] = useState<string>('ALL');
  const [search, setSearch] = useState('');
  const [sortBy, setSortBy] = useState<'winRate' | 'pickRate' | 'banRate'>('winRate');

  useEffect(() => {
    return patchService.subscribe((state) => {
      setPatchState(state);
    });
  }, []);

  const roles = ['ALL', 'TOP', 'JUNGLE', 'MID', 'ADC', 'SUPPORT'];

  const currentChamps = patchService.getChampions() || CHAMPIONS_LIST;

  const champions = currentChamps.map((c) => {
    const build = getChampionBuild(c.id, c.name);
    return {
      ...c,
      tier: build.tier,
      winRate: build.winRate,
      pickRate: build.pickRate,
      banRate: build.banRate,
      role: build.role,
    };
  });

  const filtered = champions
    .filter((c) => {
      const matchesRole = activeRole === 'ALL' || c.roles.includes(activeRole) || c.role === activeRole;
      const matchesSearch = c.name.toLowerCase().includes(search.toLowerCase());
      return matchesRole && matchesSearch;
    })
    .sort((a, b) => b[sortBy] - a[sortBy]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '18px' }}>
      {/* Header and Controls */}
      <div className="glass-card" style={{ padding: '20px 24px' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '16px' }}>
          <div>
            <h2 style={{ fontSize: '18px', fontWeight: 800, color: '#fff', display: 'flex', alignItems: 'center', gap: '8px', letterSpacing: '0.2px' }}>
              <Crown size={19} color="var(--blue-primary)" />
              <span>Patch {patchState.majorMinor} Meta Tier List</span>
              <span
                style={{
                  fontSize: '10.5px',
                  fontWeight: 700,
                  padding: '2px 8px',
                  borderRadius: '4px',
                  background: 'rgba(0, 210, 255, 0.1)',
                  color: 'var(--blue-primary)',
                  border: '1px solid var(--border-blue)',
                }}
              >
                {patchState.totalChampions} Champions • Auto-Synced
              </span>
            </h2>
            <p style={{ fontSize: '12px', color: '#94a3b8', marginTop: '3px' }}>
              Real-time patch win rates, ban statistics, and high-elo optimal rune pages
            </p>
          </div>

          {/* Search bar */}
          <div className="search-input-box" style={{ width: '260px' }}>
            <Search size={13} color="#64748b" />
            <input
              type="text"
              placeholder="Search Champion..."
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
        </div>

        {/* Filters and Sorting Row */}
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '12px', marginTop: '16px', paddingTop: '14px', borderTop: '1px solid var(--border-subtle)' }}>
          {/* Lane Filters */}
          <div style={{ display: 'flex', gap: '6px', overflowX: 'auto', paddingBottom: '2px' }}>
            {roles.map((r) => (
              <button
                key={r}
                className={`btn-secondary ${activeRole === r ? 'active' : ''}`}
                onClick={() => setActiveRole(r)}
                style={{
                  fontSize: '11px',
                  padding: '5px 12px',
                  fontWeight: 700,
                  borderRadius: 'var(--radius-sm)',
                }}
              >
                {r}
              </button>
            ))}
          </div>

          {/* Sort By Toggles */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            <span style={{ fontSize: '11px', color: '#64748b', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.4px' }}>Sort:</span>
            {[
              { id: 'winRate', label: 'Win Rate' },
              { id: 'pickRate', label: 'Pick Rate' },
              { id: 'banRate', label: 'Ban Rate' },
            ].map((sortOption) => (
              <button
                key={sortOption.id}
                onClick={() => setSortBy(sortOption.id as any)}
                style={{
                  padding: '4px 9px',
                  borderRadius: 'var(--radius-sm)',
                  fontSize: '11px',
                  fontWeight: 700,
                  cursor: 'pointer',
                  background: sortBy === sortOption.id ? 'rgba(0, 210, 255, 0.14)' : 'rgba(255, 255, 255, 0.03)',
                  border: sortBy === sortOption.id ? '1px solid var(--blue-primary)' : '1px solid transparent',
                  color: sortBy === sortOption.id ? 'var(--blue-primary)' : '#94a3b8',
                  transition: 'all 0.15s ease',
                }}
              >
                {sortOption.label}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Champion Grid */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(270px, 1fr))', gap: '12px' }}>
        {filtered.map((champ, idx) => {
          const rank = idx + 1;
          const isTop3 = rank <= 3;
          const rankColor = rank === 1 ? '#f59e0b' : rank === 2 ? '#cbd5e1' : rank === 3 ? '#b45309' : '#64748b';

          return (
            <div
              key={champ.id}
              className="glass-card"
              style={{
                padding: '12px 14px',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                cursor: 'pointer',
                borderRadius: 'var(--radius-md)',
              }}
              onClick={() => onSelectChampion(Number(champ.id))}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '10px', minWidth: 0 }}>
                {/* Rank number */}
                <div
                  className="tabular-nums"
                  style={{
                    width: '20px',
                    fontSize: '11px',
                    fontWeight: 900,
                    color: rankColor,
                    textAlign: 'center',
                    flexShrink: 0,
                  }}
                >
                  #{rank}
                </div>

                {/* Champion Avatar */}
                <div style={{ position: 'relative', flexShrink: 0 }}>
                  <img
                    src={getChampionIcon(champ.key)}
                    alt={champ.name}
                    style={{
                      width: '42px',
                      height: '42px',
                      borderRadius: '9px',
                      border: isTop3 ? `1px solid ${rankColor}` : '1px solid var(--border-subtle)',
                      boxShadow: isTop3 ? `0 0 10px ${rankColor}40` : 'none',
                      display: 'block',
                    }}
                  />
                  <span
                    style={{
                      position: 'absolute',
                      bottom: '-3px',
                      right: '-3px',
                      fontSize: '8px',
                      fontWeight: 900,
                      padding: '1px 4px',
                      borderRadius: '3px',
                      background: 'rgba(6, 10, 18, 0.95)',
                      color: champ.tier.includes('S') ? 'var(--blue-primary)' : '#cbd5e1',
                      border: '1px solid rgba(255, 255, 255, 0.15)',
                    }}
                  >
                    {champ.tier}
                  </span>
                </div>

                <div style={{ minWidth: 0 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                    <span style={{ fontSize: '13px', fontWeight: 700, color: '#fff' }}>{champ.name}</span>
                    <span
                      style={{
                        fontSize: '9.5px',
                        fontWeight: 700,
                        padding: '1px 5px',
                        borderRadius: '3px',
                        background: 'rgba(255, 255, 255, 0.05)',
                        color: '#94a3b8',
                      }}
                    >
                      {champ.role}
                    </span>
                  </div>
                  <div className="tabular-nums" style={{ fontSize: '11px', color: '#94a3b8', marginTop: '2px', display: 'flex', gap: '6px' }}>
                    <span style={{ color: champ.winRate >= 51 ? 'var(--blue-primary)' : '#cbd5e1', fontWeight: 700 }}>
                      {champ.winRate}% WR
                    </span>
                    <span>•</span>
                    <span>{champ.pickRate}% Pick</span>
                  </div>
                </div>
              </div>

              <button
                className="btn-secondary"
                style={{ padding: '5px 9px', fontSize: '11px', flexShrink: 0 }}
                onClick={(e) => {
                  e.stopPropagation();
                  onSelectChampion(Number(champ.id));
                }}
              >
                <Zap size={12} color="var(--blue-primary)" />
                <span>Build</span>
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );
};
