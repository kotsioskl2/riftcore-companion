import React, { useState, useEffect } from 'react';
import {
  Zap,
  Shield,
  Sparkles,
  Check,
  Search,
  ChevronRight,
  ShieldAlert
} from 'lucide-react';
import { ChampSelectSession } from '../types';
import { CHAMPIONS_LIST, getChampionBuild, getChampionIcon, getChampionSplash } from '../services/championData';
import { RUNE_TREES, STAT_SHARDS, SUMMONER_SPELLS } from '../services/runeData';
import { soundService } from '../services/soundService';

interface ChampSelectViewProps {
  session: ChampSelectSession | null;
  onSelectChampion?: (championId: number) => void;
}

export const ChampSelectView: React.FC<ChampSelectViewProps> = ({ session }) => {
  const [selectedChampId, setSelectedChampId] = useState<number>(420);
  const [buildType, setBuildType] = useState<'primary' | 'secondary'>('primary');
  const [searchQuery, setSearchQuery] = useState('');
  const [importStatus, setImportStatus] = useState<'idle' | 'importing' | 'success' | 'error'>('idle');
  const [spellStatus, setSpellStatus] = useState<'idle' | 'importing' | 'success'>('idle');

  useEffect(() => {
    if (session && session.myTeam && session.localPlayerCellId !== undefined) {
      const me = session.myTeam.find((p) => p.cellId === session.localPlayerCellId);
      if (me) {
        const champId = me.championId || me.championPickIntent;
        if (champId && champId > 0) {
          setSelectedChampId(champId);
        }
      }
    }
  }, [session]);

  const champ = CHAMPIONS_LIST.find((c) => c.id === selectedChampId) || CHAMPIONS_LIST[0];
  const build = getChampionBuild(champ.id, champ.name);
  const activeRunes = buildType === 'primary' ? build.runes : build.secondaryRunes;

  const filteredChampions = CHAMPIONS_LIST.filter((c) =>
    c.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const handleImportRunes = async () => {
    setImportStatus('importing');
    try {
      if (window.riftAPI) {
        await window.riftAPI.importRunes({
          name: `⚡ Rift: ${build.championName}`,
          primaryStyleId: activeRunes.primaryStyleId,
          subStyleId: activeRunes.subStyleId,
          selectedPerkIds: activeRunes.selectedPerkIds,
        });
        soundService.playRuneImport();
        setImportStatus('success');
        setTimeout(() => setImportStatus('idle'), 3000);
      } else {
        setTimeout(() => {
          soundService.playRuneImport();
          setImportStatus('success');
          setTimeout(() => setImportStatus('idle'), 3000);
        }, 400);
      }
    } catch (err) {
      console.error('Import runes error:', err);
      setImportStatus('error');
      setTimeout(() => setImportStatus('idle'), 3000);
    }
  };

  const handleImportSpells = async () => {
    setSpellStatus('importing');
    try {
      if (window.riftAPI) {
        await window.riftAPI.importSpells(build.spells[0], build.spells[1]);
        soundService.playRuneImport();
        setSpellStatus('success');
        setTimeout(() => setSpellStatus('idle'), 2500);
      } else {
        setTimeout(() => {
          soundService.playRuneImport();
          setSpellStatus('success');
          setTimeout(() => setSpellStatus('idle'), 2500);
        }, 300);
      }
    } catch {
      setSpellStatus('idle');
    }
  };

  const primaryTree = RUNE_TREES[activeRunes.primaryStyleId];
  const subTree = RUNE_TREES[activeRunes.subStyleId];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Champion Minimalist Hero Header */}
      <div
        className="glass-card"
        style={{
          position: 'relative',
          padding: '24px',
          backgroundImage: `linear-gradient(90deg, rgba(6, 9, 15, 0.96) 0%, rgba(6, 9, 15, 0.8) 55%, rgba(6, 9, 15, 0.4) 100%), url(${getChampionSplash(champ.key)})`,
          backgroundSize: 'cover',
          backgroundPosition: 'center 20%',
          border: '1px solid var(--border-blue)',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', flexWrap: 'wrap', gap: '16px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '18px' }}>
            <img
              src={getChampionIcon(champ.key)}
              alt={champ.name}
              style={{
                width: '74px',
                height: '74px',
                borderRadius: '12px',
                border: '2px solid var(--blue-primary)',
                boxShadow: '0 0 20px var(--blue-glow)',
              }}
            />
            <div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '4px' }}>
                <h1 style={{ fontSize: '24px', color: '#fff', fontWeight: 800, letterSpacing: '0.5px' }}>
                  {build.championName}
                </h1>
                <span className={`tier-badge tier-${build.tier.toLowerCase().replace('+', '-plus')}`}>
                  {build.tier} Tier
                </span>
                <span
                  style={{
                    fontSize: '11px',
                    fontWeight: 700,
                    textTransform: 'uppercase',
                    padding: '3px 8px',
                    background: 'rgba(255, 255, 255, 0.08)',
                    borderRadius: '4px',
                    color: '#94a3b8',
                  }}
                >
                  {build.role}
                </span>
              </div>
              <div style={{ display: 'flex', gap: '18px', fontSize: '13px', color: 'var(--text-secondary)' }}>
                <span>Win Rate: <strong style={{ color: 'var(--blue-primary)' }}>{build.winRate}%</strong></span>
                <span>Pick Rate: <strong style={{ color: '#fff' }}>{build.pickRate}%</strong></span>
                <span>Ban Rate: <strong style={{ color: 'var(--red-primary)' }}>{build.banRate}%</strong></span>
              </div>
            </div>
          </div>

          {/* Quick Actions */}
          <div style={{ display: 'flex', gap: '10px', alignItems: 'center' }}>
            <button
              id="btn-import-runes"
              className="btn-primary"
              onClick={handleImportRunes}
              disabled={importStatus === 'importing'}
              style={{ minWidth: '190px' }}
            >
              {importStatus === 'importing' ? (
                <>Applying Runes...</>
              ) : importStatus === 'success' ? (
                <>
                  <Check size={16} color="#00c8ff" />
                  <span>Runes Injected! (100ms)</span>
                </>
              ) : (
                <>
                  <Zap size={16} />
                  <span>Push Runes to Client</span>
                </>
              )}
            </button>

            <button
              id="btn-import-spells"
              className="btn-secondary"
              onClick={handleImportSpells}
            >
              {spellStatus === 'success' ? (
                <Check size={14} color="var(--blue-primary)" />
              ) : (
                <Sparkles size={14} color="var(--blue-primary)" />
              )}
              <span>Auto Spells</span>
            </button>
          </div>
        </div>

        {/* Minimalist Champion Quick Bar */}
        <div style={{ marginTop: '20px', display: 'flex', alignItems: 'center', gap: '10px' }}>
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              background: 'rgba(0, 0, 0, 0.5)',
              border: '1px solid var(--border-subtle)',
              borderRadius: 'var(--radius-md)',
              padding: '6px 12px',
              width: '220px',
            }}
          >
            <Search size={14} color="#64748b" />
            <input
              type="text"
              placeholder="Search Champion..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
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

          <div style={{ display: 'flex', gap: '6px', overflowX: 'auto', paddingBottom: '2px', flex: 1 }}>
            {filteredChampions.slice(0, 8).map((c) => (
              <button
                key={c.id}
                onClick={() => setSelectedChampId(c.id)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  padding: '4px 10px',
                  borderRadius: '6px',
                  background: selectedChampId === c.id ? 'var(--blue-subtle)' : 'rgba(255, 255, 255, 0.04)',
                  border: selectedChampId === c.id ? '1px solid var(--blue-primary)' : '1px solid transparent',
                  color: selectedChampId === c.id ? 'var(--blue-primary)' : '#94a3b8',
                  fontSize: '11px',
                  fontWeight: 600,
                  cursor: 'pointer',
                  whiteSpace: 'nowrap',
                }}
              >
                <img
                  src={getChampionIcon(c.key)}
                  alt={c.name}
                  style={{ width: '18px', height: '18px', borderRadius: '50%' }}
                />
                <span>{c.name}</span>
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Grid: Runes & Skills on Left, Items & Matchups on Right */}
      <div style={{ display: 'grid', gridTemplateColumns: '1.15fr 0.85fr', gap: '20px' }}>
        {/* LEFT COLUMN */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
          {/* Runes Tree Card */}
          <div className="glass-card">
            <div className="card-header">
              <div className="card-title">
                <Zap size={16} color="var(--blue-primary)" />
                <span>Optimal Runes</span>
              </div>
              <div style={{ display: 'flex', gap: '6px' }}>
                <button
                  className={`btn-secondary ${buildType === 'primary' ? 'active' : ''}`}
                  onClick={() => setBuildType('primary')}
                  style={{ fontSize: '11px', padding: '4px 10px' }}
                >
                  Highest Winrate ({build.winRate}%)
                </button>
                <button
                  className={`btn-secondary ${buildType === 'secondary' ? 'active' : ''}`}
                  onClick={() => setBuildType('secondary')}
                  style={{ fontSize: '11px', padding: '4px 10px' }}
                >
                  Most Popular
                </button>
              </div>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: '1.2fr 1fr', gap: '24px', alignItems: 'start' }}>
              {/* Primary Style */}
              <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '10px', borderBottom: '1px solid var(--border-subtle)', paddingBottom: '8px' }}>
                  <img src={primaryTree?.icon} alt={primaryTree?.name} style={{ width: '26px', height: '26px' }} />
                  <span style={{ fontWeight: 700, color: 'var(--blue-primary)', fontSize: '13px' }}>
                    {primaryTree?.name} (Primary)
                  </span>
                </div>

                {/* Keystone */}
                <div style={{ display: 'flex', alignItems: 'center', gap: '12px', background: 'var(--blue-subtle)', padding: '10px 14px', borderRadius: '10px', border: '1px solid var(--border-blue)' }}>
                  <img
                    src={activeRunes.keystoneIcon}
                    alt={activeRunes.keystoneName}
                    style={{ width: '40px', height: '40px', borderRadius: '50%', boxShadow: '0 0 12px var(--blue-glow)' }}
                  />
                  <div>
                    <div style={{ fontSize: '13px', fontWeight: 700, color: '#fff' }}>
                      {activeRunes.keystoneName}
                    </div>
                    <div style={{ fontSize: '11px', color: '#94a3b8' }}>Keystone Perk</div>
                  </div>
                </div>

                {/* Sub-runes */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
                  {primaryTree?.slots.map((row, idx) => {
                    const selected = row.find((r) => activeRunes.selectedPerkIds.includes(r.id)) || row[0];
                    return (
                      <div
                        key={idx}
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          gap: '10px',
                          padding: '6px 10px',
                          borderRadius: '8px',
                          background: 'rgba(255, 255, 255, 0.02)',
                        }}
                      >
                        <img src={selected.icon} alt={selected.name} style={{ width: '24px', height: '24px', borderRadius: '50%' }} />
                        <div style={{ flex: 1 }}>
                          <div style={{ fontSize: '12px', fontWeight: 600, color: '#fff' }}>{selected.name}</div>
                          <div style={{ fontSize: '10px', color: '#64748b' }}>{selected.desc}</div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>

              {/* Secondary Style & Stat Shards */}
              <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '10px', borderBottom: '1px solid var(--border-subtle)', paddingBottom: '8px' }}>
                  <img src={subTree?.icon} alt={subTree?.name} style={{ width: '24px', height: '24px' }} />
                  <span style={{ fontWeight: 700, color: '#94a3b8', fontSize: '13px' }}>
                    {subTree?.name} (Secondary)
                  </span>
                </div>

                <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
                  {subTree?.slots.flatMap((row) => row).filter((r) => activeRunes.selectedPerkIds.includes(r.id)).map((perk) => (
                    <div
                      key={perk.id}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '10px',
                        padding: '6px 10px',
                        borderRadius: '8px',
                        background: 'rgba(255, 255, 255, 0.02)',
                      }}
                    >
                      <img src={perk.icon} alt={perk.name} style={{ width: '24px', height: '24px', borderRadius: '50%' }} />
                      <div style={{ flex: 1 }}>
                        <div style={{ fontSize: '12px', fontWeight: 600, color: '#fff' }}>{perk.name}</div>
                        <div style={{ fontSize: '10px', color: '#64748b' }}>{perk.desc}</div>
                      </div>
                    </div>
                  ))}
                </div>

                {/* Shards */}
                <div style={{ marginTop: '6px' }}>
                  <div style={{ fontSize: '11px', fontWeight: 700, textTransform: 'uppercase', color: '#64748b', marginBottom: '6px' }}>
                    Stat Shards
                  </div>
                  <div style={{ display: 'flex', gap: '8px' }}>
                    {activeRunes.selectedPerkIds.slice(6, 9).map((shardId, idx) => {
                      const shard = STAT_SHARDS[shardId] || { name: 'Stat Mod', icon: '' };
                      return (
                        <div
                          key={idx}
                          style={{
                            display: 'flex',
                            alignItems: 'center',
                            gap: '6px',
                            background: 'rgba(255, 255, 255, 0.04)',
                            padding: '6px 8px',
                            borderRadius: '6px',
                            border: '1px solid var(--border-subtle)',
                            fontSize: '11px',
                            color: '#cbd5e1',
                          }}
                          title={shard.name}
                        >
                          {shard.icon && (
                            <img src={shard.icon} alt={shard.name} style={{ width: '16px', height: '16px' }} />
                          )}
                          <span>Mod {idx + 1}</span>
                        </div>
                      );
                    })}
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Skill Order Card */}
          <div className="glass-card">
            <div className="card-header">
              <div className="card-title">
                <Shield size={16} color="var(--blue-primary)" />
                <span>Ability Leveling Order</span>
              </div>
              <div
                style={{
                  fontSize: '12px',
                  fontWeight: 800,
                  color: 'var(--blue-primary)',
                  background: 'var(--blue-subtle)',
                  padding: '3px 10px',
                  borderRadius: '6px',
                  border: '1px solid var(--border-blue)',
                }}
              >
                Priority: {build.skillPriority}
              </div>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', overflowX: 'auto' }}>
              {['Q', 'W', 'E', 'R'].map((skillKey) => (
                <div key={skillKey} style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <div
                    style={{
                      width: '26px',
                      height: '26px',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      background: 'rgba(255, 255, 255, 0.05)',
                      borderRadius: '4px',
                      fontSize: '12px',
                      fontWeight: 800,
                      color: 'var(--blue-primary)',
                    }}
                  >
                    {skillKey}
                  </div>
                  <div style={{ display: 'flex', gap: '4px', flex: 1 }}>
                    {build.skillOrder.map((sk, lvlIdx) => {
                      const isUpgraded = sk === skillKey;
                      return (
                        <div
                          key={lvlIdx}
                          style={{
                            width: '24px',
                            height: '24px',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            borderRadius: '4px',
                            fontSize: '10px',
                            fontWeight: 700,
                            background: isUpgraded
                              ? skillKey === 'R'
                                ? 'linear-gradient(135deg, var(--red-primary), var(--red-deep))'
                                : 'linear-gradient(135deg, var(--blue-primary), var(--blue-deep))'
                              : 'rgba(255, 255, 255, 0.03)',
                            color: isUpgraded ? '#fff' : '#475569',
                            border: isUpgraded ? '1px solid rgba(255, 255, 255, 0.2)' : '1px solid transparent',
                          }}
                        >
                          {lvlIdx + 1}
                        </div>
                      );
                    })}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* RIGHT COLUMN */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
          {/* Item Builds Card */}
          <div className="glass-card">
            <div className="card-header">
              <div className="card-title">
                <Sparkles size={16} color="var(--blue-primary)" />
                <span>Item Build & Spells</span>
              </div>
            </div>

            {/* Spells */}
            <div style={{ marginBottom: '16px' }}>
              <div style={{ fontSize: '11px', fontWeight: 700, textTransform: 'uppercase', color: '#64748b', marginBottom: '8px' }}>
                Summoner Spells
              </div>
              <div style={{ display: 'flex', gap: '10px' }}>
                {build.spells.map((sId) => {
                  const sp = SUMMONER_SPELLS[sId];
                  return (
                    <div
                      key={sId}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '8px',
                        background: 'rgba(255, 255, 255, 0.03)',
                        padding: '6px 12px',
                        borderRadius: '8px',
                        border: '1px solid var(--border-subtle)',
                      }}
                    >
                      <img src={sp?.icon} alt={sp?.name} style={{ width: '28px', height: '28px', borderRadius: '6px' }} />
                      <div>
                        <div style={{ fontSize: '12px', fontWeight: 700, color: '#fff' }}>{sp?.name}</div>
                        <div style={{ fontSize: '10px', color: '#64748b' }}>{sp?.cooldown}s CD</div>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>

            {/* Core Items */}
            <div style={{ marginBottom: '16px' }}>
              <div style={{ fontSize: '11px', fontWeight: 700, textTransform: 'uppercase', color: '#64748b', marginBottom: '8px' }}>
                Core Rush
              </div>
              <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                {build.coreItems.map((item, idx) => (
                  <React.Fragment key={item.id}>
                    <div
                      style={{
                        display: 'flex',
                        flexDirection: 'column',
                        alignItems: 'center',
                        gap: '4px',
                        background: 'var(--blue-subtle)',
                        padding: '8px',
                        borderRadius: '8px',
                        border: '1px solid var(--border-blue)',
                        minWidth: '78px',
                        textAlign: 'center',
                      }}
                    >
                      <img
                        src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/item/${item.id}.png`}
                        alt={item.name}
                        style={{ width: '36px', height: '36px', borderRadius: '6px' }}
                        onError={(e: any) => { e.target.style.display = 'none'; }}
                      />
                      <span style={{ fontSize: '10px', fontWeight: 600, color: '#fff' }}>{item.name}</span>
                    </div>
                    {idx < build.coreItems.length - 1 && (
                      <ChevronRight size={14} color="#64748b" />
                    )}
                  </React.Fragment>
                ))}
              </div>
            </div>

            {/* Situational & Boots */}
            <div>
              <div style={{ fontSize: '11px', fontWeight: 700, textTransform: 'uppercase', color: '#64748b', marginBottom: '8px' }}>
                Boots & Situational
              </div>
              <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap' }}>
                <div
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '6px',
                    background: 'rgba(255, 255, 255, 0.03)',
                    padding: '6px 10px',
                    borderRadius: '8px',
                    border: '1px solid var(--border-subtle)',
                  }}
                >
                  <img
                    src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/item/${build.boots.id}.png`}
                    alt={build.boots.name}
                    style={{ width: '22px', height: '22px', borderRadius: '4px' }}
                    onError={(e: any) => { e.target.style.display = 'none'; }}
                  />
                  <span style={{ fontSize: '11px', color: '#cbd5e1' }}>{build.boots.name}</span>
                </div>

                {build.situationalItems.map((item) => (
                  <div
                    key={item.id}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '6px',
                      background: 'rgba(255, 255, 255, 0.03)',
                      padding: '6px 10px',
                      borderRadius: '8px',
                      border: '1px solid var(--border-subtle)',
                    }}
                  >
                    <img
                      src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/item/${item.id}.png`}
                      alt={item.name}
                      style={{ width: '22px', height: '22px', borderRadius: '4px' }}
                      onError={(e: any) => { e.target.style.display = 'none'; }}
                    />
                    <span style={{ fontSize: '11px', color: '#94a3b8' }}>{item.name}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Matchups Card: Red for Counters, Blue for Favored */}
          <div className="glass-card">
            <div className="card-header">
              <div className="card-title">
                <ShieldAlert size={16} color="var(--red-primary)" />
                <span>Matchups & Counters</span>
              </div>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '14px' }}>
              {/* Favored (Blue) */}
              <div>
                <div style={{ fontSize: '11px', fontWeight: 700, color: 'var(--blue-primary)', marginBottom: '8px' }}>
                  ✓ Favored Matchups
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
                  {build.counters.strongAgainst.map((c) => (
                    <div
                      key={c.name}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        background: 'var(--blue-subtle)',
                        padding: '6px 10px',
                        borderRadius: '6px',
                        fontSize: '11px',
                        border: '1px solid var(--border-blue)',
                      }}
                    >
                      <span style={{ fontWeight: 600, color: '#fff' }}>{c.name}</span>
                      <span style={{ color: 'var(--blue-primary)', fontWeight: 700 }}>{c.winRate}%</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Hard Counters (Red) */}
              <div>
                <div style={{ fontSize: '11px', fontWeight: 700, color: 'var(--red-primary)', marginBottom: '8px' }}>
                  ✕ Hard Counters
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
                  {build.counters.weakAgainst.map((c) => (
                    <div
                      key={c.name}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        background: 'var(--red-subtle)',
                        padding: '6px 10px',
                        borderRadius: '6px',
                        fontSize: '11px',
                        border: '1px solid var(--border-red)',
                      }}
                    >
                      <span style={{ fontWeight: 600, color: '#fff' }}>{c.name}</span>
                      <span style={{ color: 'var(--red-primary)', fontWeight: 700 }}>{c.winRate}%</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            {/* Tips */}
            {build.tips && build.tips.length > 0 && (
              <div style={{ marginTop: '14px', paddingTop: '12px', borderTop: '1px solid var(--border-subtle)' }}>
                <div style={{ fontSize: '11px', fontWeight: 700, textTransform: 'uppercase', color: 'var(--blue-primary)', marginBottom: '6px' }}>
                  Strategy Notes
                </div>
                <ul style={{ paddingLeft: '16px', fontSize: '11px', color: '#cbd5e1', display: 'flex', flexDirection: 'column', gap: '4px' }}>
                  {build.tips.map((tip, idx) => (
                    <li key={idx}>{tip}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
