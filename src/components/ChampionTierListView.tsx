import React, { useState, useEffect } from 'react';
import {
  Crown,
  Search,
  Zap,
  TrendingUp,
  Filter,
  Sparkles,
  ChevronRight,
  LayoutGrid,
  List,
  Shield,
  Swords,
  X,
  Flame,
  Award
} from 'lucide-react';
import { CHAMPIONS_LIST, getChampionBuild, getChampionIcon } from '../services/championData';
import { patchService, PatchState } from '../services/patchService';

interface ChampionTierListViewProps {
  onSelectChampion: (championId: number) => void;
}

type ViewMode = 'table' | 'brackets';

export const ChampionTierListView: React.FC<ChampionTierListViewProps> = ({ onSelectChampion }) => {
  const [patchState, setPatchState] = useState<PatchState>(patchService.getPatchState());
  const [activeRole, setActiveRole] = useState<string>('ALL');
  const [search, setSearch] = useState('');
  const [sortBy, setSortBy] = useState<'winRate' | 'pickRate' | 'banRate'>('winRate');
  const [viewMode, setViewMode] = useState<ViewMode>('table');

  useEffect(() => {
    return patchService.subscribe((state) => {
      setPatchState(state);
    });
  }, []);

  const roles = [
    { id: 'ALL', label: 'All Roles', icon: '⚡' },
    { id: 'TOP', label: 'Top', icon: '🛡️' },
    { id: 'JUNGLE', label: 'Jungle', icon: '🌲' },
    { id: 'MID', label: 'Mid', icon: '⚔️' },
    { id: 'ADC', label: 'Bot / ADC', icon: '🏹' },
    { id: 'SUPPORT', label: 'Support', icon: '💖' },
  ];

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
      keystoneIcon: build.runes?.keystoneIcon,
      keystoneName: build.runes?.keystoneName,
    };
  });

  const filtered = champions
    .filter((c) => {
      const matchesRole =
        activeRole === 'ALL' ||
        c.role === activeRole ||
        (Array.isArray(c.roles) && c.roles.includes(activeRole));
      const matchesSearch =
        c.name.toLowerCase().includes(search.toLowerCase()) ||
        c.title.toLowerCase().includes(search.toLowerCase());
      return matchesRole && matchesSearch;
    })
    .sort((a, b) => b[sortBy] - a[sortBy]);

  // Group by tier for Brackets view
  const tierGroups = {
    'S+': filtered.filter((c) => c.tier === 'S+'),
    'S': filtered.filter((c) => c.tier === 'S'),
    'A': filtered.filter((c) => c.tier === 'A'),
    'B': filtered.filter((c) => c.tier === 'B'),
  };

  const getTierColor = (tier: string) => {
    switch (tier) {
      case 'S+':
        return { bg: 'rgba(250, 204, 21, 0.15)', text: '#facc15', border: 'rgba(250, 204, 21, 0.4)' };
      case 'S':
        return { bg: 'rgba(0, 200, 255, 0.15)', text: 'var(--blue-primary)', border: 'var(--border-blue)' };
      case 'A':
        return { bg: 'rgba(192, 132, 252, 0.15)', text: '#c084fc', border: 'rgba(192, 132, 252, 0.35)' };
      default:
        return { bg: 'rgba(148, 163, 184, 0.12)', text: '#94a3b8', border: 'rgba(148, 163, 184, 0.25)' };
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
      {/* Header and Controls */}
      <div className="glass-card" style={{ padding: '16px 20px' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '14px' }}>
          <div>
            <h2 style={{ fontSize: '17px', fontWeight: 800, color: '#fff', display: 'flex', alignItems: 'center', gap: '8px', letterSpacing: '0.2px' }}>
              <Crown size={18} color="var(--blue-primary)" />
              <span>Patch {patchState.majorMinor} Meta Tier List</span>
              <span
                style={{
                  fontSize: '10px',
                  fontWeight: 700,
                  padding: '2px 7px',
                  borderRadius: '4px',
                  background: 'rgba(0, 210, 255, 0.1)',
                  color: 'var(--blue-primary)',
                  border: '1px solid var(--border-blue)',
                }}
              >
                {filtered.length} Champions
              </span>
            </h2>
            <p style={{ fontSize: '11.5px', color: '#94a3b8', marginTop: '2px' }}>
              Patch-synchronized rankings, win rates, and optimal keystone setups
            </p>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            {/* View Mode Toggle */}
            <div
              style={{
                display: 'flex',
                background: 'rgba(255, 255, 255, 0.04)',
                border: '1px solid var(--border-subtle)',
                borderRadius: 'var(--radius-sm)',
                padding: '2px',
              }}
            >
              <button
                onClick={() => setViewMode('table')}
                style={{
                  padding: '5px 10px',
                  borderRadius: '4px',
                  border: 'none',
                  background: viewMode === 'table' ? 'var(--blue-primary)' : 'transparent',
                  color: viewMode === 'table' ? '#000' : '#94a3b8',
                  fontSize: '11px',
                  fontWeight: 800,
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '5px',
                  transition: 'all 0.15s ease',
                }}
                title="Table Leaderboard View"
              >
                <List size={13} />
                <span>Leaderboard</span>
              </button>
              <button
                onClick={() => setViewMode('brackets')}
                style={{
                  padding: '5px 10px',
                  borderRadius: '4px',
                  border: 'none',
                  background: viewMode === 'brackets' ? 'var(--blue-primary)' : 'transparent',
                  color: viewMode === 'brackets' ? '#000' : '#94a3b8',
                  fontSize: '11px',
                  fontWeight: 800,
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '5px',
                  transition: 'all 0.15s ease',
                }}
                title="Tier Brackets View (S+, S, A, B)"
              >
                <LayoutGrid size={13} />
                <span>Tier Brackets</span>
              </button>
            </div>

            {/* Search bar */}
            <div className="search-input-box" style={{ width: '220px', position: 'relative' }}>
              <Search size={13} color="#64748b" />
              <input
                type="text"
                placeholder="Filter champion..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                style={{
                  background: 'transparent',
                  border: 'none',
                  color: '#fff',
                  fontSize: '11.5px',
                  outline: 'none',
                  width: '100%',
                }}
              />
              {search && (
                <button
                  onClick={() => setSearch('')}
                  style={{
                    background: 'transparent',
                    border: 'none',
                    color: '#94a3b8',
                    cursor: 'pointer',
                    padding: '2px',
                    display: 'flex',
                  }}
                >
                  <X size={12} />
                </button>
              )}
            </div>
          </div>
        </div>

        {/* Filters and Sorting Row */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            flexWrap: 'wrap',
            gap: '10px',
            marginTop: '12px',
            paddingTop: '12px',
            borderTop: '1px solid rgba(255, 255, 255, 0.06)',
          }}
        >
          {/* Lane Filters */}
          <div style={{ display: 'flex', gap: '5px', overflowX: 'auto', paddingBottom: '2px' }}>
            {roles.map((r) => (
              <button
                key={r.id}
                className={`btn-secondary ${activeRole === r.id ? 'active' : ''}`}
                onClick={() => setActiveRole(r.id)}
                style={{
                  fontSize: '11px',
                  padding: '5px 11px',
                  fontWeight: 700,
                  borderRadius: 'var(--radius-sm)',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '5px',
                  background: activeRole === r.id ? 'rgba(0, 200, 255, 0.18)' : 'rgba(255, 255, 255, 0.03)',
                  border: activeRole === r.id ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                  color: activeRole === r.id ? 'var(--blue-primary)' : '#94a3b8',
                }}
              >
                <span>{r.icon}</span>
                <span>{r.label}</span>
              </button>
            ))}
          </div>

          {/* Sort By Toggles */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '5px' }}>
            <span style={{ fontSize: '10.5px', color: '#64748b', fontWeight: 800, textTransform: 'uppercase', letterSpacing: '0.4px' }}>Sort:</span>
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
                  fontSize: '10.5px',
                  fontWeight: 700,
                  cursor: 'pointer',
                  background: sortBy === sortOption.id ? 'rgba(0, 210, 255, 0.14)' : 'rgba(255, 255, 255, 0.02)',
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

      {/* VIEW MODE 1: PRO LEADERBOARD TABLE (CLEAN, DENSE & UNCLUTTERED) */}
      {viewMode === 'table' && (
        <div className="glass-card" style={{ padding: '0', overflow: 'hidden' }}>
          {/* Table Header */}
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: '50px 2.2fr 85px 100px 105px 95px 95px 120px 85px',
              padding: '10px 16px',
              background: 'rgba(0, 0, 0, 0.25)',
              borderBottom: '1px solid rgba(255, 255, 255, 0.06)',
              fontSize: '10.5px',
              fontWeight: 800,
              textTransform: 'uppercase',
              letterSpacing: '0.5px',
              color: '#64748b',
              alignItems: 'center',
            }}
          >
            <div style={{ textAlign: 'center' }}>#</div>
            <div>Champion</div>
            <div>Tier</div>
            <div>Role</div>
            <div>Win Rate</div>
            <div>Pick Rate</div>
            <div>Ban Rate</div>
            <div>Keystone</div>
            <div style={{ textAlign: 'right' }}>Build</div>
          </div>

          {/* Table Rows */}
          <div style={{ display: 'flex', flexDirection: 'column' }}>
            {filtered.map((champ, idx) => {
              const rank = idx + 1;
              const isTop3 = rank <= 3;
              const rankColor = rank === 1 ? '#f59e0b' : rank === 2 ? '#cbd5e1' : rank === 3 ? '#b45309' : '#64748b';
              const tierStyle = getTierColor(champ.tier);

              return (
                <div
                  key={champ.id}
                  onClick={() => onSelectChampion(Number(champ.id))}
                  style={{
                    display: 'grid',
                    gridTemplateColumns: '50px 2.2fr 85px 100px 105px 95px 95px 120px 85px',
                    padding: '8px 16px',
                    alignItems: 'center',
                    borderBottom: '1px solid rgba(255, 255, 255, 0.03)',
                    cursor: 'pointer',
                    transition: 'background 0.12s ease',
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.background = 'rgba(0, 200, 255, 0.04)';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.background = 'transparent';
                  }}
                >
                  {/* Rank */}
                  <div
                    style={{
                      textAlign: 'center',
                      fontWeight: 900,
                      fontSize: '12px',
                      color: rankColor,
                    }}
                  >
                    #{rank}
                  </div>

                  {/* Champion Avatar & Name */}
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px', minWidth: 0 }}>
                    <img
                      src={getChampionIcon(champ.key)}
                      alt={champ.name}
                      style={{
                        width: '34px',
                        height: '34px',
                        borderRadius: '7px',
                        border: isTop3 ? `1px solid ${rankColor}` : '1px solid var(--border-subtle)',
                        flexShrink: 0,
                      }}
                      onError={(e: any) => { e.target.src = getChampionIcon(420); }}
                    />
                    <div style={{ minWidth: 0, overflow: 'hidden' }}>
                      <div style={{ fontSize: '13px', fontWeight: 800, color: '#fff', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                        {champ.name}
                      </div>
                      <div style={{ fontSize: '10px', color: '#64748b', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                        {champ.title}
                      </div>
                    </div>
                  </div>

                  {/* Tier Badge */}
                  <div>
                    <span
                      style={{
                        fontSize: '10px',
                        fontWeight: 900,
                        padding: '2px 7px',
                        borderRadius: '4px',
                        background: tierStyle.bg,
                        color: tierStyle.text,
                        border: `1px solid ${tierStyle.border}`,
                      }}
                    >
                      {champ.tier}
                    </span>
                  </div>

                  {/* Role */}
                  <div>
                    <span
                      style={{
                        fontSize: '10px',
                        fontWeight: 700,
                        padding: '2px 6px',
                        borderRadius: '3px',
                        background: 'rgba(255, 255, 255, 0.05)',
                        color: '#94a3b8',
                      }}
                    >
                      {champ.role}
                    </span>
                  </div>

                  {/* Win Rate */}
                  <div>
                    <div style={{ fontSize: '12px', fontWeight: 800, color: champ.winRate >= 51.5 ? 'var(--blue-primary)' : '#e2e8f0' }}>
                      {champ.winRate}%
                    </div>
                    {/* Progress indicator */}
                    <div
                      style={{
                        width: '56px',
                        height: '3px',
                        borderRadius: '2px',
                        background: 'rgba(255, 255, 255, 0.08)',
                        marginTop: '3px',
                        overflow: 'hidden',
                      }}
                    >
                      <div
                        style={{
                          width: `${Math.max(0, Math.min(100, (champ.winRate - 46) * 12.5))}%`,
                          height: '100%',
                          background: champ.winRate >= 51.5 ? 'var(--blue-primary)' : '#94a3b8',
                        }}
                      />
                    </div>
                  </div>

                  {/* Pick Rate */}
                  <div style={{ fontSize: '11.5px', color: '#cbd5e1', fontWeight: 600 }}>
                    {champ.pickRate}%
                  </div>

                  {/* Ban Rate */}
                  <div style={{ fontSize: '11.5px', color: '#94a3b8', fontWeight: 600 }}>
                    {champ.banRate}%
                  </div>

                  {/* Keystone Preview */}
                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    {champ.keystoneIcon && (
                      <img
                        src={champ.keystoneIcon}
                        alt="Keystone"
                        style={{ width: '20px', height: '20px', borderRadius: '50%', flexShrink: 0 }}
                      />
                    )}
                    <span style={{ fontSize: '10.5px', color: '#94a3b8', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                      {champ.keystoneName || 'Standard'}
                    </span>
                  </div>

                  {/* Action */}
                  <div style={{ textAlign: 'right' }}>
                    <button
                      className="btn-secondary"
                      style={{ padding: '4px 8px', fontSize: '10.5px' }}
                      onClick={(e) => {
                        e.stopPropagation();
                        onSelectChampion(Number(champ.id));
                      }}
                    >
                      <Zap size={11} color="var(--blue-primary)" />
                      <span>Build</span>
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* VIEW MODE 2: TIER BRACKETS (S+, S, A, B CATEGORIZED) */}
      {viewMode === 'brackets' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {(['S+', 'S', 'A', 'B'] as const).map((t) => {
            const list = tierGroups[t];
            if (list.length === 0) return null;
            const tStyle = getTierColor(t);

            const tierMetaDesc =
              t === 'S+'
                ? '👑 God Tier (Must Pick or Ban • Dominates Current Patch)'
                : t === 'S'
                ? '⭐ Strong Meta (High Winrate & Reliable Blind Picks)'
                : t === 'A'
                ? '⚔️ Balanced Picks (Solid Matchups & Consistent Output)'
                : '🛡️ Situational (Off-Meta or Skill-Specific Matchups)';

            return (
              <div key={t} className="glass-card" style={{ padding: '16px 18px' }}>
                <div
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    marginBottom: '12px',
                    paddingBottom: '8px',
                    borderBottom: '1px solid rgba(255, 255, 255, 0.06)',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <span
                      style={{
                        fontSize: '12px',
                        fontWeight: 900,
                        padding: '2px 8px',
                        borderRadius: '4px',
                        background: tStyle.bg,
                        color: tStyle.text,
                        border: `1px solid ${tStyle.border}`,
                      }}
                    >
                      {t} TIER
                    </span>
                    <span style={{ fontSize: '12px', color: '#94a3b8', fontWeight: 600 }}>
                      {tierMetaDesc}
                    </span>
                  </div>
                  <span style={{ fontSize: '11px', color: '#64748b' }}>
                    {list.length} Champions
                  </span>
                </div>

                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(210px, 1fr))', gap: '8px' }}>
                  {list.map((champ) => (
                    <div
                      key={champ.id}
                      onClick={() => onSelectChampion(Number(champ.id))}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        padding: '8px 10px',
                        borderRadius: '6px',
                        background: 'rgba(255, 255, 255, 0.02)',
                        border: '1px solid rgba(255, 255, 255, 0.05)',
                        cursor: 'pointer',
                        transition: 'all 0.12s ease',
                      }}
                      onMouseEnter={(e) => {
                        e.currentTarget.style.background = 'rgba(0, 200, 255, 0.06)';
                        e.currentTarget.style.borderColor = 'var(--border-blue)';
                      }}
                      onMouseLeave={(e) => {
                        e.currentTarget.style.background = 'rgba(255, 255, 255, 0.02)';
                        e.currentTarget.style.borderColor = 'rgba(255, 255, 255, 0.05)';
                      }}
                    >
                      <div style={{ display: 'flex', alignItems: 'center', gap: '8px', minWidth: 0 }}>
                        <img
                          src={getChampionIcon(champ.key)}
                          alt={champ.name}
                          style={{
                            width: '32px',
                            height: '32px',
                            borderRadius: '6px',
                            border: '1px solid var(--border-subtle)',
                            flexShrink: 0,
                          }}
                          onError={(e: any) => { e.target.src = getChampionIcon(420); }}
                        />
                        <div style={{ minWidth: 0 }}>
                          <div style={{ fontSize: '12px', fontWeight: 800, color: '#fff', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                            {champ.name}
                          </div>
                          <div style={{ fontSize: '10px', color: champ.winRate >= 51.5 ? 'var(--blue-primary)' : '#94a3b8', fontWeight: 700 }}>
                            {champ.winRate}% WR <span style={{ color: '#64748b' }}>• {champ.role}</span>
                          </div>
                        </div>
                      </div>

                      <ChevronRight size={14} color="#64748b" style={{ flexShrink: 0 }} />
                    </div>
                  ))}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};

