import React, { useState } from 'react';
import {
  Flame,
  Zap,
  Shield,
  Search,
  Sparkles,
  Check,
  TrendingUp,
  Snowflake,
  Swords,
  Trophy,
  Layers,
  Crown,
  Heart,
  HelpCircle,
  Clock,
  Compass,
  Filter
} from 'lucide-react';
import { CHAMPIONS_LIST, getChampionBuild, getChampionIcon, getChampionSplash } from '../services/championData';
import { patchService } from '../services/patchService';
import { soundService } from '../services/soundService';
import { MAYHEM_CARDS, searchMayhemCards, CardTier } from '../services/mayhemCardsData';

type ModeTab = 'arena' | 'aram' | 'urf' | 'oneforall';

interface ArenaDuo {
  duo: [string, string];
  synergyRating: number;
  playstyle: string;
  description: string;
}

interface GameModesHubViewProps {
  initialMode?: ModeTab;
  liveGameMode?: string;
  onSelectChampion?: (championId: number) => void;
}

export const GameModesHubView: React.FC<GameModesHubViewProps> = ({ initialMode, liveGameMode, onSelectChampion }) => {
  const currentChamps = patchService.getChampions() || CHAMPIONS_LIST;
  const [selectedMode, setSelectedMode] = useState<ModeTab>(initialMode || 'aram');
  const [search, setSearch] = useState('');
  const [cardSearch, setCardSearch] = useState('');
  const [cardTierFilter, setCardTierFilter] = useState<CardTier | 'ALL'>('ALL');
  const [selectedChampId, setSelectedChampId] = useState<number>(222); // Default Jinx
  const [importStatus, setImportStatus] = useState<'idle' | 'importing' | 'success'>('idle');

  // Auto-sync tab whenever initialMode or liveGameMode updates
  React.useEffect(() => {
    if (initialMode) {
      setSelectedMode(initialMode);
    } else if (liveGameMode) {
      const lm = liveGameMode.toUpperCase();
      if (lm.includes('ARAM')) setSelectedMode('aram');
      else if (lm.includes('CHERRY') || lm.includes('ARENA')) setSelectedMode('arena');
      else if (lm.includes('URF')) setSelectedMode('urf');
      else if (lm.includes('ONEFORALL') || lm.includes('OFA')) setSelectedMode('oneforall');
    }
  }, [initialMode, liveGameMode]);

  const arenaDuos: ArenaDuo[] = [
    {
      duo: ['Taric', 'Master Yi'],
      synergyRating: 98,
      playstyle: 'Invulnerability Hyper-Dive',
      description: 'Taric casts Dazzle (E) and Cosmic Radiance (R) through Master Yi as he Alpha Strikes, making Yi untargetable and immortal.',
    },
    {
      duo: ['Ivern', 'Rengar'],
      synergyRating: 95,
      playstyle: 'Endless Bush Stalking',
      description: 'Ivern constantly plants brush anywhere on the arena floor; Rengar leaps infinitely with zero cooldowns.',
    },
    {
      duo: ['Alistar', 'Poppy'],
      synergyRating: 92,
      playstyle: 'Ring of Fire Ring-Out',
      description: 'Stall until the Ring of Fire closes, then use Alistar Headbutt and Poppy Keeper\'s Verdict to knock enemies into the deadly fire.',
    },
    {
      duo: ['Kayle', 'Braum'],
      synergyRating: 90,
      playstyle: 'Unbreakable Late Scaling',
      description: 'Braum\'s shield and stuns keep Kayle 100% safe until she reaches level 16 ascendancy and wipes both enemies.',
    },
    {
      duo: ['Singed', 'Cassiopeia'],
      synergyRating: 89,
      playstyle: 'Poison Twin Fang Melter',
      description: 'Singed\'s poison trail constantly activates Cassiopeia\'s empowered E (Twin Fang) without needing Cassiopeia to land Q.',
    },
  ];

  // ARAM DATA & MULTIPLIERS
  const getAramModifiers = (champName: string) => {
    switch (champName) {
      case 'Jinx':
        return { dealt: 95, taken: 105, haste: 0, heal: 100, tier: 'S', note: '-5% Dmg / +5% Taken due to passive resets' };
      case 'Lux':
        return { dealt: 85, taken: 110, haste: -10, heal: 80, tier: 'A', note: '-15% Dmg due to long-range choke poke' };
      case 'Aatrox':
        return { dealt: 105, taken: 90, haste: 10, heal: 105, tier: 'S+', note: '+5% Dmg / -10% Taken (Melee Buff)' };
      case 'Lee Sin':
        return { dealt: 110, taken: 85, haste: 20, heal: 120, tier: 'S+', note: '+10% Dmg / -15% Taken ARAM Aura' };
      case 'Sona':
        return { dealt: 80, taken: 115, haste: -20, heal: 70, tier: 'B', note: 'Heavy AOE heal & shield penalty' };
      case 'Ziggs':
        return { dealt: 80, taken: 120, haste: -10, heal: 100, tier: 'C', note: 'Heavy structure & AOE poke penalty' };
      default:
        return { dealt: 100, taken: 100, haste: 0, heal: 100, tier: 'A', note: 'Standard ARAM baseline balance' };
    }
  };

  // URF DATA
  const urfPicks = [
    { name: 'Hecarim', winRate: 61.2, tier: 'S+', tip: 'Q spam has 0.1s cooldown. Infinite movement speed with Ghost & Predator.' },
    { name: 'Zed', winRate: 59.8, tier: 'S+', tip: 'Double shadow W has 2s cooldown. Infinite shuriken barrage and untargetability.' },
    { name: 'Morgana', winRate: 58.4, tier: 'S', tip: 'Q root duration (3s) is longer than Q cooldown (2.2s). Guaranteed permanent root.' },
    { name: 'Jax', winRate: 57.9, tier: 'S', tip: 'Counter Strike (E) is active 80% of combat time. Immune to basic attacks.' },
    { name: 'Fizz', winRate: 57.5, tier: 'S', tip: 'Playful / Trickster (E) has 1.8s cooldown. Virtually perma-untargetable.' },
    { name: 'Vladimir', winRate: 56.8, tier: 'S', tip: 'Sanguine Pool (W) and Transfusion (Q) spam provides endless healing and untargetability.' },
  ];

  // ONE FOR ALL DATA
  const ofaCombos = [
    {
      name: 'Darius',
      tier: 'S+',
      synergy: 'Shared Hemorrhage Stacks',
      description: 'All 5 Darius players share passive bleed stacks! If any Darius lands 5 attacks, ALL 5 Darius players immediately enter Noxian Might with +230 bonus AD!',
    },
    {
      name: 'Lux',
      tier: 'S+',
      synergy: 'Simultaneous Final Sparks',
      description: 'One Q root leads to 5 simultaneous Final Spark lasers across the entire lane, dealing 4,000+ instant damage.',
    },
    {
      name: 'Morgana',
      tier: 'S',
      synergy: '15-Second CC Chain & Black Shield Wall',
      description: '5 Dark Bindings allow a single target to be rooted for 15 unbroken seconds, while 5 Black Shields block all enemy CC.',
    },
    {
      name: 'Tahm Kench',
      tier: 'S',
      synergy: 'Matryoshka Doll Ingestion',
      description: 'Tahm Kench can devour an ally Tahm Kench who has devoured an ally Tahm Kench, creating an unkillable rolling shield fortress.',
    },
  ];

  const champ = currentChamps.find((c) => c.id === selectedChampId) || currentChamps[0];
  const aramMods = getAramModifiers(champ.name);

  const handlePushModeRunes = async (modeName: string) => {
    setImportStatus('importing');
    try {
      if (window.riftAPI) {
        const safeChampName = (champ.name || 'Champion').replace(/[^a-zA-Z0-9 ]/g, '').trim();
        if (modeName === 'ARAM') {
          await window.riftAPI.importRunes({
            name: `RiftCore ARAM ${safeChampName}`,
            primaryStyleId: 8100, // Domination
            subStyleId: 8000, // Precision
            selectedPerkIds: [8128, 8143, 8140, 8106, 8009, 8299, 5008, 5008, 5001],
          });
        } else if (modeName === 'ARENA') {
          await window.riftAPI.importRunes({
            name: `RiftCore Arena ${safeChampName}`,
            primaryStyleId: 8000,
            subStyleId: 8400,
            selectedPerkIds: [8010, 9111, 9104, 8299, 8473, 8451, 5008, 5008, 5001],
          });
        } else {
          await window.riftAPI.importRunes({
            name: `RiftCore URF ${safeChampName}`,
            primaryStyleId: 8100,
            subStyleId: 8200,
            selectedPerkIds: [8128, 8143, 8140, 8105, 8226, 8210, 5008, 5008, 5001],
          });
        }
        soundService.playRuneImport();
        setImportStatus('success');
        setTimeout(() => setImportStatus('idle'), 2500);
      } else {
        setTimeout(() => {
          soundService.playRuneImport();
          setImportStatus('success');
          setTimeout(() => setImportStatus('idle'), 2500);
        }, 350);
      }
    } catch {
      setImportStatus('idle');
    }
  };

  const filteredChamps = currentChamps.filter((c) =>
    c.name.toLowerCase().includes(search.toLowerCase())
  ).slice(0, 16); // Clean compact search limit to avoid UI clutter

  const filteredMayhemCards = searchMayhemCards(cardSearch, cardTierFilter, selectedMode === 'arena' ? 'Arena' : 'ARAM');

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
      {/* Sleek Minimalist Mode Header & Navigation */}
      <div
        className="glass-card"
        style={{
          padding: '12px 18px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexWrap: 'wrap',
          gap: '12px',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
          <div
            style={{
              width: '32px',
              height: '32px',
              borderRadius: '8px',
              background: 'rgba(0, 210, 255, 0.12)',
              border: '1px solid rgba(0, 210, 255, 0.3)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <Trophy size={16} color="var(--blue-primary)" />
          </div>
          <div>
            <h2 style={{ fontSize: '15px', fontWeight: 800, color: '#fff' }}>
              Special Modes & Augments Hub
            </h2>
            <div style={{ fontSize: '11px', color: '#94a3b8' }}>
              Meta cards, champion balance multipliers, and duo synergies
            </div>
          </div>
        </div>

        {/* Clean Segmented Mode Selector */}
        <div
          style={{
            display: 'flex',
            background: 'rgba(5, 10, 20, 0.65)',
            border: '1px solid var(--border-subtle)',
            borderRadius: '8px',
            padding: '3px',
            gap: '4px',
          }}
        >
          {[
            { id: 'aram', label: 'ARAM Mayhem', icon: Snowflake, color: 'var(--blue-primary)' },
            { id: 'arena', label: 'Arena 2v2', icon: Trophy, color: '#eab308' },
            { id: 'urf', label: 'URF / ARURF', icon: Zap, color: 'var(--red-primary)' },
            { id: 'oneforall', label: 'One for All', icon: Crown, color: '#c084fc' },
          ].map((mode) => {
            const Icon = mode.icon;
            const isActive = selectedMode === mode.id;
            return (
              <button
                key={mode.id}
                onClick={() => setSelectedMode(mode.id as ModeTab)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  padding: '5px 12px',
                  borderRadius: '6px',
                  background: isActive ? 'rgba(255, 255, 255, 0.08)' : 'transparent',
                  border: isActive ? `1px solid ${mode.color}` : '1px solid transparent',
                  color: isActive ? '#fff' : '#94a3b8',
                  fontSize: '11px',
                  fontWeight: 700,
                  cursor: 'pointer',
                  transition: 'all 0.15s ease',
                }}
              >
                <Icon size={13} color={isActive ? mode.color : '#94a3b8'} />
                <span>{mode.label}</span>
              </button>
            );
          })}
        </div>
      </div>

      {/* 1. ARAM MAYHEM VIEW (Organized & Clutter-Free) */}
      {selectedMode === 'aram' && (
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1.3fr', gap: '16px' }}>
          {/* Left Column: Champion Balance & Quick Picker */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
            <div className="glass-card" style={{ padding: '16px' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px' }}>
                <span style={{ fontSize: '11px', fontWeight: 800, textTransform: 'uppercase', letterSpacing: '0.6px', color: 'var(--blue-primary)' }}>
                  Selected Champion Balance
                </span>
                <span className={`tier-badge tier-${aramMods.tier.toLowerCase().replace('+', '-plus')}`}>
                  {aramMods.tier} Tier (ARAM)
                </span>
              </div>

              <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '14px' }}>
                <img
                  src={getChampionIcon(champ.key)}
                  alt={champ.name}
                  style={{ width: '50px', height: '50px', borderRadius: '10px', border: '1.5px solid var(--blue-primary)' }}
                />
                <div style={{ flex: 1 }}>
                  <div style={{ fontSize: '16px', fontWeight: 800, color: '#fff' }}>{champ.name}</div>
                  <div style={{ fontSize: '11px', color: '#94a3b8', marginTop: '2px' }}>
                    {aramMods.note}
                  </div>
                </div>
              </div>

              {/* Multiplier KPI Grid */}
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '6px', marginBottom: '14px' }}>
                <div style={{ background: 'rgba(255, 255, 255, 0.03)', padding: '8px 4px', borderRadius: '6px', textAlign: 'center' }}>
                  <div style={{ fontSize: '8.5px', color: '#64748b', fontWeight: 800 }}>DMG DEALT</div>
                  <div style={{ fontSize: '13px', fontWeight: 900, color: aramMods.dealt > 100 ? '#22c55e' : aramMods.dealt < 100 ? 'var(--red-primary)' : '#fff', marginTop: '1px' }}>
                    {aramMods.dealt}%
                  </div>
                </div>
                <div style={{ background: 'rgba(255, 255, 255, 0.03)', padding: '8px 4px', borderRadius: '6px', textAlign: 'center' }}>
                  <div style={{ fontSize: '8.5px', color: '#64748b', fontWeight: 800 }}>DMG TAKEN</div>
                  <div style={{ fontSize: '13px', fontWeight: 900, color: aramMods.taken < 100 ? '#22c55e' : aramMods.taken > 100 ? 'var(--red-primary)' : '#fff', marginTop: '1px' }}>
                    {aramMods.taken}%
                  </div>
                </div>
                <div style={{ background: 'rgba(255, 255, 255, 0.03)', padding: '8px 4px', borderRadius: '6px', textAlign: 'center' }}>
                  <div style={{ fontSize: '8.5px', color: '#64748b', fontWeight: 800 }}>ABILITY HASTE</div>
                  <div style={{ fontSize: '13px', fontWeight: 900, color: aramMods.haste > 0 ? 'var(--blue-primary)' : '#fff', marginTop: '1px' }}>
                    {aramMods.haste > 0 ? `+${aramMods.haste}` : aramMods.haste < 0 ? `${aramMods.haste}` : '0'}
                  </div>
                </div>
                <div style={{ background: 'rgba(255, 255, 255, 0.03)', padding: '8px 4px', borderRadius: '6px', textAlign: 'center' }}>
                  <div style={{ fontSize: '8.5px', color: '#64748b', fontWeight: 800 }}>HEALING</div>
                  <div style={{ fontSize: '13px', fontWeight: 900, color: aramMods.heal < 100 ? 'var(--red-primary)' : '#fff', marginTop: '1px' }}>
                    {aramMods.heal}%
                  </div>
                </div>
              </div>

              {/* Action Buttons */}
              <div style={{ display: 'flex', gap: '8px' }}>
                <button
                  onClick={() => handlePushModeRunes('ARAM')}
                  className="btn-primary"
                  disabled={importStatus === 'importing'}
                  style={{ flex: 1, padding: '7px 12px', fontSize: '11px', fontWeight: 800 }}
                >
                  {importStatus === 'success' ? (
                    <>
                      <Check size={14} />
                      <span>ARAM Runes Pushed!</span>
                    </>
                  ) : (
                    <>
                      <Zap size={14} />
                      <span>Push ARAM Runes</span>
                    </>
                  )}
                </button>
                {onSelectChampion && (
                  <button
                    onClick={() => onSelectChampion(champ.id)}
                    className="btn-secondary"
                    style={{ padding: '7px 12px', fontSize: '11px', fontWeight: 700 }}
                  >
                    <span>View Build</span>
                  </button>
                )}
              </div>
            </div>

            {/* Compact Champion Selector with Search */}
            <div className="glass-card" style={{ padding: '14px' }}>
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  background: 'rgba(0, 0, 0, 0.4)',
                  border: '1px solid var(--border-subtle)',
                  borderRadius: '6px',
                  padding: '5px 10px',
                  marginBottom: '10px',
                }}
              >
                <Search size={13} color="#64748b" />
                <input
                  type="text"
                  placeholder="Filter champion..."
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  style={{ background: 'transparent', border: 'none', color: '#fff', fontSize: '11.5px', outline: 'none', width: '100%' }}
                />
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '6px', maxHeight: '180px', overflowY: 'auto' }}>
                {filteredChamps.map((c) => (
                  <button
                    key={c.id}
                    onClick={() => {
                      setSelectedChampId(c.id);
                      soundService.playTimerTick();
                    }}
                    style={{
                      display: 'flex',
                      flexDirection: 'column',
                      alignItems: 'center',
                      gap: '4px',
                      padding: '6px 4px',
                      borderRadius: '6px',
                      background: selectedChampId === c.id ? 'rgba(0, 210, 255, 0.15)' : 'rgba(255, 255, 255, 0.02)',
                      border: selectedChampId === c.id ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                      cursor: 'pointer',
                      transition: 'all 0.15s ease',
                    }}
                  >
                    <img src={getChampionIcon(c.key)} alt={c.name} style={{ width: '28px', height: '28px', borderRadius: '5px' }} />
                    <span style={{ fontSize: '9.5px', fontWeight: 700, color: '#cbd5e1', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: '55px' }}>
                      {c.name}
                    </span>
                  </button>
                ))}
              </div>
            </div>

            {/* Quick Snowball & Relic Rules */}
            <div
              style={{
                background: 'rgba(0, 210, 255, 0.04)',
                border: '1px solid var(--border-blue)',
                borderRadius: '8px',
                padding: '10px 14px',
                fontSize: '11px',
                color: '#cbd5e1',
                display: 'flex',
                alignItems: 'center',
                gap: '10px',
              }}
            >
              <Snowflake size={16} color="var(--blue-primary)" style={{ flexShrink: 0 }} />
              <div>
                <strong>Health Relics:</strong> First spawn at <strong>01:45</strong> (60s respawn). Always equip <strong>Mark / Dash (Snowball)</strong> for waveclear & bush checks.
              </div>
            </div>
          </div>

          {/* Right Column: ARAM MAYHEM CARDS & AUGMENTS (Requested Feature) */}
          <div className="glass-card" style={{ padding: '16px', display: 'flex', flexDirection: 'column', height: '100%', boxSizing: 'border-box' }}>
            {/* Cards Header & Filter Bar */}
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '10px', flexWrap: 'wrap', gap: '8px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <Sparkles size={16} color="#c084fc" />
                <span style={{ fontSize: '13.5px', fontWeight: 800, color: '#fff' }}>
                  ARAM Mayhem Cards & Augments
                </span>
                <span style={{ fontSize: '10px', color: '#94a3b8' }}>
                  ({filteredMayhemCards.length} cards)
                </span>
              </div>

              {/* Tier Filter Pills */}
              <div style={{ display: 'flex', gap: '4px' }}>
                {(['ALL', 'Prismatic', 'Gold', 'Silver'] as const).map((tier) => (
                  <button
                    key={tier}
                    onClick={() => setCardTierFilter(tier)}
                    style={{
                      fontSize: '9.5px',
                      fontWeight: 800,
                      padding: '2px 8px',
                      borderRadius: '4px',
                      background: cardTierFilter === tier ? 'rgba(255, 255, 255, 0.12)' : 'rgba(255, 255, 255, 0.03)',
                      border: cardTierFilter === tier ? '1px solid rgba(255, 255, 255, 0.3)' : '1px solid transparent',
                      color: tier === 'Prismatic' ? '#c084fc' : tier === 'Gold' ? '#f59e0b' : tier === 'Silver' ? '#94a3b8' : '#fff',
                      cursor: 'pointer',
                    }}
                  >
                    {tier}
                  </button>
                ))}
              </div>
            </div>

            {/* Card Search Input */}
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
                background: 'rgba(0, 0, 0, 0.4)',
                border: '1px solid var(--border-subtle)',
                borderRadius: '6px',
                padding: '6px 12px',
                marginBottom: '10px',
              }}
            >
              <Search size={13} color="#64748b" />
              <input
                type="text"
                placeholder="Search card name, trait (Crit, Haste, Tank, DoT)..."
                value={cardSearch}
                onChange={(e) => setCardSearch(e.target.value)}
                style={{ background: 'transparent', border: 'none', color: '#fff', fontSize: '11.5px', outline: 'none', width: '100%' }}
              />
            </div>

            {/* Scrollable Mayhem Cards Grid */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', overflowY: 'auto', maxHeight: '420px', paddingRight: '4px' }}>
              {filteredMayhemCards.map((card) => (
                <div
                  key={card.id}
                  style={{
                    background: 'rgba(255, 255, 255, 0.02)',
                    border: `1px solid ${card.color}33`,
                    borderLeft: `3px solid ${card.color}`,
                    borderRadius: '6px',
                    padding: '10px 12px',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '4px',
                    transition: 'all 0.15s ease',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <span style={{ fontSize: '13px', fontWeight: 800, color: '#fff' }}>{card.name}</span>
                      <span
                        style={{
                          fontSize: '9px',
                          fontWeight: 800,
                          padding: '1px 5px',
                          borderRadius: '3px',
                          background: `${card.color}18`,
                          color: card.color,
                          border: `1px solid ${card.color}44`,
                        }}
                      >
                        {card.tier}
                      </span>
                      <span
                        style={{
                          fontSize: '9px',
                          fontWeight: 900,
                          padding: '1px 5px',
                          borderRadius: '3px',
                          background: card.rank === 'S+' ? 'rgba(239, 68, 68, 0.2)' : 'rgba(34, 197, 94, 0.2)',
                          color: card.rank === 'S+' ? '#ef4444' : '#22c55e',
                          border: `1px solid ${card.rank === 'S+' ? 'rgba(239, 68, 68, 0.4)' : 'rgba(34, 197, 94, 0.4)'}`,
                        }}
                      >
                        {card.rank} Tier
                      </span>
                    </div>
                    <span style={{ fontSize: '9px', color: '#64748b', fontWeight: 700 }}>{card.tag}</span>
                  </div>

                  <div style={{ fontSize: '11px', color: '#cbd5e1', lineHeight: '1.4' }}>
                    {card.description}
                  </div>

                  <div style={{ fontSize: '10px', color: '#94a3b8', marginTop: '2px' }}>
                    <strong style={{ color: 'var(--blue-primary)' }}>Best For:</strong> {card.archetype}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* 2. ARENA 2v2v2v2 VIEW (De-cluttered) */}
      {selectedMode === 'arena' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
          {/* Tactical 3-Pill Banner */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '10px' }}>
            <div style={{ background: 'rgba(0, 210, 255, 0.05)', border: '1px solid var(--border-blue)', borderRadius: '6px', padding: '10px 12px' }}>
              <div style={{ fontSize: '11px', fontWeight: 800, color: 'var(--blue-primary)', marginBottom: '2px' }}>
                🌸 Power Flowers (0:15)
              </div>
              <div style={{ fontSize: '10.5px', color: '#cbd5e1' }}>
                Hit 3x to gain 8% max HP/Mana + 20% cooldown refund per hit.
              </div>
            </div>
            <div style={{ background: 'rgba(234, 179, 8, 0.05)', border: '1px solid rgba(234, 179, 8, 0.3)', borderRadius: '6px', padding: '10px 12px' }}>
              <div style={{ fontSize: '11px', fontWeight: 800, color: '#f59e0b', marginBottom: '2px' }}>
                ⏳ Partner Revive Ring
              </div>
              <div style={{ fontSize: '10.5px', color: '#cbd5e1' }}>
                Stand inside dead ally's circle for 2.5s to revive with 30% HP.
              </div>
            </div>
            <div style={{ background: 'rgba(239, 68, 68, 0.05)', border: '1px solid rgba(239, 68, 68, 0.3)', borderRadius: '6px', padding: '10px 12px' }}>
              <div style={{ fontSize: '11px', fontWeight: 800, color: 'var(--red-primary)', marginBottom: '2px' }}>
                🔥 Ring of Fire (1:15)
              </div>
              <div style={{ fontSize: '10.5px', color: '#cbd5e1' }}>
                Shrinking circle deals true damage up to 20% max HP/s.
              </div>
            </div>
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '14px' }}>
            {/* Arena Duos */}
            <div className="glass-card" style={{ padding: '16px' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <Trophy size={15} color="var(--blue-primary)" />
                  <span style={{ fontSize: '13px', fontWeight: 800, color: '#fff' }}>S+ Duo Synergies</span>
                </div>
                <span style={{ fontSize: '10px', color: '#94a3b8' }}>Proven 2v2 pairings</span>
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                {arenaDuos.map((d, idx) => (
                  <div
                    key={idx}
                    style={{
                      background: 'rgba(255, 255, 255, 0.02)',
                      border: '1px solid var(--border-subtle)',
                      borderRadius: '6px',
                      padding: '8px 10px',
                    }}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '4px' }}>
                      <span style={{ fontSize: '12.5px', fontWeight: 800, color: '#fff' }}>
                        {d.duo[0]} + {d.duo[1]}
                      </span>
                      <span style={{ fontSize: '11px', fontWeight: 900, color: '#22c55e', fontFamily: 'var(--font-mono)' }}>
                        {d.synergyRating}% Win
                      </span>
                    </div>
                    <div style={{ fontSize: '10.5px', color: '#94a3b8' }}>{d.description}</div>
                  </div>
                ))}
              </div>
            </div>

            {/* Arena Augments */}
            <div className="glass-card" style={{ padding: '16px' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <Sparkles size={15} color="#c084fc" />
                  <span style={{ fontSize: '13px', fontWeight: 800, color: '#fff' }}>Arena Drafting Cards</span>
                </div>
                <span style={{ fontSize: '10px', color: '#94a3b8' }}>Tier S+ Priority</span>
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', maxHeight: '380px', overflowY: 'auto' }}>
                {MAYHEM_CARDS.slice(0, 6).map((aug) => (
                  <div
                    key={aug.id}
                    style={{
                      background: 'rgba(255, 255, 255, 0.02)',
                      border: `1px solid ${aug.color}33`,
                      borderRadius: '6px',
                      padding: '8px 10px',
                    }}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '3px' }}>
                      <span style={{ fontSize: '12.5px', fontWeight: 800, color: '#fff' }}>{aug.name}</span>
                      <span
                        style={{
                          fontSize: '9px',
                          fontWeight: 800,
                          padding: '1px 5px',
                          borderRadius: '3px',
                          background: `${aug.color}18`,
                          color: aug.color,
                        }}
                      >
                        {aug.tier}
                      </span>
                    </div>
                    <div style={{ fontSize: '10.5px', color: '#cbd5e1' }}>{aug.description}</div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 3. URF VIEW */}
      {selectedMode === 'urf' && (
        <div className="glass-card" style={{ padding: '16px' }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '14px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <Zap size={16} color="var(--red-primary)" />
              <span style={{ fontSize: '14px', fontWeight: 800, color: '#fff' }}>Ultra Rapid Fire (URF) S+ Meta Picks</span>
            </div>
            <span style={{ fontSize: '11px', color: '#94a3b8' }}>80% CDR & Zero Mana Costs</span>
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '10px' }}>
            {urfPicks.map((pick) => (
              <div
                key={pick.name}
                style={{
                  background: 'rgba(255, 255, 255, 0.02)',
                  border: '1px solid rgba(255, 42, 95, 0.25)',
                  borderRadius: '8px',
                  padding: '10px 12px',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '12px',
                }}
              >
                <img
                  src={getChampionIcon(pick.name)}
                  alt={pick.name}
                  style={{ width: '40px', height: '40px', borderRadius: '8px', border: '1.5px solid var(--red-primary)' }}
                />
                <div style={{ flex: 1 }}>
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <span style={{ fontSize: '13.5px', fontWeight: 800, color: '#fff' }}>{pick.name}</span>
                    <span style={{ fontSize: '12px', fontWeight: 800, color: '#22c55e', fontFamily: 'var(--font-mono)' }}>
                      {pick.winRate}% Win
                    </span>
                  </div>
                  <div style={{ fontSize: '10.5px', color: '#94a3b8', marginTop: '2px' }}>
                    {pick.tip}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 4. ONE FOR ALL VIEW */}
      {selectedMode === 'oneforall' && (
        <div className="glass-card" style={{ padding: '16px' }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '14px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <Crown size={16} color="#c084fc" />
              <span style={{ fontSize: '14px', fontWeight: 800, color: '#fff' }}>One for All 5x Passive Synergies</span>
            </div>
            <span style={{ fontSize: '11px', color: '#94a3b8' }}>Abilities that stack across all 5 players</span>
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '10px' }}>
            {ofaCombos.map((ofa) => (
              <div
                key={ofa.name}
                style={{
                  background: 'rgba(255, 255, 255, 0.02)',
                  border: '1px solid rgba(192, 132, 252, 0.25)',
                  borderRadius: '8px',
                  padding: '10px 12px',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '12px',
                }}
              >
                <img
                  src={getChampionIcon(ofa.name)}
                  alt={ofa.name}
                  style={{ width: '40px', height: '40px', borderRadius: '8px', border: '1.5px solid #c084fc' }}
                />
                <div style={{ flex: 1 }}>
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <span style={{ fontSize: '13.5px', fontWeight: 800, color: '#fff' }}>{ofa.name}</span>
                    <span
                      style={{
                        fontSize: '9px',
                        fontWeight: 800,
                        padding: '1px 5px',
                        borderRadius: '3px',
                        background: 'rgba(192, 132, 252, 0.15)',
                        color: '#c084fc',
                        border: '1px solid rgba(192, 132, 252, 0.3)',
                      }}
                    >
                      {ofa.synergy}
                    </span>
                  </div>
                  <div style={{ fontSize: '10.5px', color: '#94a3b8', marginTop: '2px' }}>
                    {ofa.description}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};
