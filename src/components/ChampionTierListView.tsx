import React, { useState } from 'react';
import { Crown, Search, Zap } from 'lucide-react';
import { CHAMPIONS_LIST, getChampionBuild, getChampionIcon } from '../services/championData';

interface ChampionTierListViewProps {
  onSelectChampion: (championId: number) => void;
}

export const ChampionTierListView: React.FC<ChampionTierListViewProps> = ({ onSelectChampion }) => {
  const [activeRole, setActiveRole] = useState<string>('ALL');
  const [search, setSearch] = useState('');
  const [sortBy, setSortBy] = useState<'winRate' | 'pickRate' | 'banRate'>('winRate');

  const roles = ['ALL', 'TOP', 'JUNGLE', 'MID', 'ADC', 'SUPPORT'];

  const champions = CHAMPIONS_LIST.map((c) => {
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
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Header and Controls */}
      <div className="glass-card" style={{ padding: '20px' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '16px' }}>
          <div>
            <h2 style={{ fontSize: '18px', fontWeight: 800, color: '#fff', display: 'flex', alignItems: 'center', gap: '8px' }}>
              <Crown size={20} color="var(--blue-primary)" />
              <span>Patch 16.15 Meta Tier List</span>
            </h2>
            <p style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
              Win rates, pick rates, and optimal rune setups
            </p>
          </div>

          {/* Search bar */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              background: 'rgba(0, 0, 0, 0.5)',
              border: '1px solid var(--border-subtle)',
              borderRadius: 'var(--radius-md)',
              padding: '6px 14px',
              width: '260px',
            }}
          >
            <Search size={14} color="#64748b" />
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

        {/* Lane Filters */}
        <div style={{ display: 'flex', gap: '8px', marginTop: '16px', overflowX: 'auto', paddingBottom: '4px' }}>
          {roles.map((r) => (
            <button
              key={r}
              className={`btn-secondary ${activeRole === r ? 'active' : ''}`}
              onClick={() => setActiveRole(r)}
              style={{
                fontSize: '11px',
                padding: '6px 14px',
              }}
            >
              {r}
            </button>
          ))}
        </div>
      </div>

      {/* Champion Grid */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(260px, 1fr))', gap: '14px' }}>
        {filtered.map((champ) => (
          <div
            key={champ.id}
            className="glass-card"
            style={{
              padding: '14px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              cursor: 'pointer',
              transition: 'all 0.15s ease',
            }}
            onClick={() => onSelectChampion(champ.id)}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <img
                src={getChampionIcon(champ.key)}
                alt={champ.name}
                style={{
                  width: '46px',
                  height: '46px',
                  borderRadius: '10px',
                  border: '1px solid var(--border-subtle)',
                }}
              />
              <div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <span style={{ fontSize: '13px', fontWeight: 700, color: '#fff' }}>{champ.name}</span>
                  <span className={`tier-badge tier-${champ.tier.toLowerCase().replace('+', '-plus')}`}>
                    {champ.tier}
                  </span>
                </div>
                <div style={{ fontSize: '11px', color: '#94a3b8', marginTop: '2px' }}>
                  <span style={{ color: 'var(--blue-primary)', fontWeight: 700 }}>{champ.winRate}% WR</span> • {champ.pickRate}% Pick
                </div>
              </div>
            </div>

            <button
              className="btn-secondary"
              style={{ padding: '6px 10px', fontSize: '11px' }}
              onClick={(e) => {
                e.stopPropagation();
                onSelectChampion(champ.id);
              }}
            >
              <Zap size={13} color="var(--blue-primary)" />
              <span>Build</span>
            </button>
          </div>
        ))}
      </div>
    </div>
  );
};
