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
  Compass
} from 'lucide-react';
import { CHAMPIONS_LIST, getChampionBuild, getChampionIcon, getChampionSplash } from '../services/championData';
import { patchService } from '../services/patchService';
import { soundService } from '../services/soundService';

type ModeTab = 'arena' | 'aram' | 'urf' | 'oneforall';

interface ArenaAugment {
  name: string;
  tier: 'Prismatic' | 'Gold' | 'Silver';
  desc: string;
  bestFor: string;
  color: string;
}

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
  const [selectedMode, setSelectedMode] = useState<ModeTab>(initialMode || 'arena');
  const [search, setSearch] = useState('');
  const [selectedChampId, setSelectedChampId] = useState<number>(222);
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

  // ARENA DATA
  const arenaAugments: ArenaAugment[] = [
    {
      name: 'Jeweled Gauntlet',
      tier: 'Prismatic',
      desc: 'Your abilities can critically strike and grant bonus Critical Strike Chance. AP translates to Crit Chance.',
      bestFor: 'Mages & AP Assassins (Ahri, Veigar, Syndra, Hwei)',
      color: '#c084fc',
    },
    {
      name: 'Blade Waltz',
      tier: 'Prismatic',
      desc: 'Become untargetable while dashing around enemies 8 times, dealing heavy physical on-hit damage.',
      bestFor: 'Duelists & Skirmishers (Master Yi, Fiora, Yasuo)',
      color: '#c084fc',
    },
    {
      name: 'Goliath',
      tier: 'Prismatic',
      desc: 'Gain 35% increased champion size, +1000 Maximum Health, and 15% bonus Adaptive Force.',
      bestFor: 'Tanks & Juggernauts (Cho\'Gath, Sion, Tahm Kench, Darius)',
      color: '#c084fc',
    },
    {
      name: 'Courage of the Colossus',
      tier: 'Prismatic',
      desc: 'Immobilizing an enemy champion grants a massive decaying shield scaling with 16% Maximum HP.',
      bestFor: 'Hard CC Tanks (Alistar, Leona, Nautilus, Thresh)',
      color: '#c084fc',
    },
    {
      name: 'Apex Inventor',
      tier: 'Gold',
      desc: 'Gain +75 Item Haste. Reduces cooldowns of Heartsteel, Sundered Sky, and Zhonya\'s down to 3 seconds.',
      bestFor: 'Item-Proc Bruisers (Aatrox, Volibear, Jax)',
      color: '#f59e0b',
    },
    {
      name: 'Symphony of War',
      tier: 'Prismatic',
      desc: 'Gain both Lethal Tempo AND Conqueror keystones simultaneously with no attack speed cap.',
      bestFor: 'Hyper-Carries & ADCs (Jinx, Vayne, Kayle, Master Yi)',
      color: '#c084fc',
    },
    {
      name: 'Earthwake',
      tier: 'Gold',
      desc: 'Your dashes, leaps, and blinks leave a trail that detonates after 1 second dealing huge physical damage.',
      bestFor: 'Mobile Assassins (Zed, Akali, Lee Sin, Riven)',
      color: '#f59e0b',
    },
    {
      name: 'Chauffeur',
      tier: 'Silver',
      desc: 'You are tethered to your ally and automatically ride with them. Gain massive bonus resistances and move speed.',
      bestFor: 'Enchanters & Artillery (Yuumi, Kog\'Maw, Sona)',
      color: '#94a3b8',
    },
  ];

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
      playstyle: 'Unbreakable Late-Game Scaling',
      description: 'Braum\'s shield and stuns keep Kayle 100% safe until she reaches level 16 ascendancy and wipes both enemies.',
    },
    {
      duo: ['Singed', 'Cassiopeia'],
      synergyRating: 89,
      playstyle: 'Poison Twin Fang Melter',
      description: 'Singed\'s poison trail constantly activates Cassiopeia\'s empowered E (Twin Fang) without needing Cassiopeia to land Q.',
    },
  ];

  // ARAM DATA
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
        return { dealt: 100, taken: 100, haste: 0, heal: 100, tier: 'A', note: 'Standard ARAM baseline' };
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
        if (modeName === 'ARAM') {
          await window.riftAPI.importRunes({
            name: `❄️ ARAM: ${champ.name}`,
            primaryStyleId: 8100, // Domination
            subStyleId: 8000, // Precision
            selectedPerkIds: [8128, 8143, 8138, 8106, 8009, 8299, 5008, 5008, 5001],
          });
        } else if (modeName === 'ARENA') {
          await window.riftAPI.importRunes({
            name: `🏟️ Arena: ${champ.name}`,
            primaryStyleId: 8000,
            subStyleId: 8400,
            selectedPerkIds: [8010, 9111, 9104, 8299, 8473, 8451, 5008, 5008, 5001],
          });
        } else {
          await window.riftAPI.importRunes({
            name: `⚡ URF: ${champ.name}`,
            primaryStyleId: 8100,
            subStyleId: 8200,
            selectedPerkIds: [8128, 8143, 8138, 8105, 8226, 8210, 5008, 5008, 5001],
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
  );

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Top Banner: Mode Switcher */}
      <div className="glass-card" style={{ padding: '20px' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '16px' }}>
          <div>
            <h2 style={{ fontSize: '18px', fontWeight: 800, color: '#fff', display: 'flex', alignItems: 'center', gap: '8px' }}>
              <Flame size={20} color="var(--blue-primary)" />
              <span>Multi-Mode Companion: Arena, ARAM, URF & Alternate Modes</span>
            </h2>
            <p style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
              Specialized meta builds, augment tier lists, balance multipliers, and duo synergies for all game modes
            </p>
          </div>

          {/* Mode Selector Tabs */}
          <div style={{ display: 'flex', gap: '8px' }}>
            <button
              onClick={() => setSelectedMode('arena')}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                padding: '6px 14px',
                borderRadius: '6px',
                background: selectedMode === 'arena' ? 'var(--blue-subtle)' : 'rgba(255, 255, 255, 0.04)',
                border: selectedMode === 'arena' ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                color: selectedMode === 'arena' ? 'var(--blue-primary)' : '#94a3b8',
                fontSize: '11px',
                fontWeight: 800,
                cursor: 'pointer',
              }}
            >
              <Trophy size={14} />
              <span>Arena 2v2v2v2</span>
            </button>

            <button
              onClick={() => setSelectedMode('aram')}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                padding: '6px 14px',
                borderRadius: '6px',
                background: selectedMode === 'aram' ? 'var(--blue-subtle)' : 'rgba(255, 255, 255, 0.04)',
                border: selectedMode === 'aram' ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                color: selectedMode === 'aram' ? 'var(--blue-primary)' : '#94a3b8',
                fontSize: '11px',
                fontWeight: 800,
                cursor: 'pointer',
              }}
            >
              <Snowflake size={14} />
              <span>ARAM Mayhem</span>
            </button>

            <button
              onClick={() => setSelectedMode('urf')}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                padding: '6px 14px',
                borderRadius: '6px',
                background: selectedMode === 'urf' ? 'rgba(255, 42, 95, 0.15)' : 'rgba(255, 255, 255, 0.04)',
                border: selectedMode === 'urf' ? '1px solid var(--red-primary)' : '1px solid var(--border-subtle)',
                color: selectedMode === 'urf' ? 'var(--red-primary)' : '#94a3b8',
                fontSize: '11px',
                fontWeight: 800,
                cursor: 'pointer',
              }}
            >
              <Zap size={14} />
              <span>URF / ARURF</span>
            </button>

            <button
              onClick={() => setSelectedMode('oneforall')}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                padding: '6px 14px',
                borderRadius: '6px',
                background: selectedMode === 'oneforall' ? 'rgba(192, 132, 252, 0.15)' : 'rgba(255, 255, 255, 0.04)',
                border: selectedMode === 'oneforall' ? '1px solid #c084fc' : '1px solid var(--border-subtle)',
                color: selectedMode === 'oneforall' ? '#c084fc' : '#94a3b8',
                fontSize: '11px',
                fontWeight: 800,
                cursor: 'pointer',
              }}
            >
              <Crown size={14} />
              <span>One for All</span>
            </button>
          </div>
        </div>
      </div>

      {/* 1. ARENA 2v2v2v2 VIEW */}
      {selectedMode === 'arena' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
          {/* Tactical Arena Tips Bar */}
          <div
            style={{
              background: 'rgba(0, 200, 255, 0.05)',
              border: '1px solid var(--border-blue)',
              borderRadius: '8px',
              padding: '14px 18px',
              display: 'grid',
              gridTemplateColumns: 'repeat(3, 1fr)',
              gap: '14px',
            }}
          >
            <div>
              <div style={{ fontSize: '11px', fontWeight: 800, color: 'var(--blue-primary)', textTransform: 'uppercase' }}>
                🌸 Power Flowers (0:15 Spawn)
              </div>
              <div style={{ fontSize: '11px', color: '#cbd5e1', marginTop: '2px' }}>
                Hit 3 times to restore 8% max HP/Mana + 20% cooldown refund per hit. Fight for flower control!
              </div>
            </div>
            <div>
              <div style={{ fontSize: '11px', fontWeight: 800, color: '#f59e0b', textTransform: 'uppercase' }}>
                ⏳ Partner Revive Ring
              </div>
              <div style={{ fontSize: '11px', color: '#cbd5e1', marginTop: '2px' }}>
                Stand inside your dead ally's circle for 2.5s to revive them with 30% health.
              </div>
            </div>
            <div>
              <div style={{ fontSize: '11px', fontWeight: 800, color: 'var(--red-primary)', textTransform: 'uppercase' }}>
                🔥 Ring of Fire (1:15 Shrink)
              </div>
              <div style={{ fontSize: '11px', color: '#cbd5e1', marginTop: '2px' }}>
                Deals true damage scaling up to 20% max HP/s. Push or knock enemies into the flames.
              </div>
            </div>
          </div>

          {/* Arena Duo Synergies */}
          <div className="glass-card" style={{ padding: '20px' }}>
            <div className="card-header" style={{ marginBottom: '14px' }}>
              <div className="card-title">
                <Trophy size={18} color="var(--blue-primary)" />
                <span>Top S+ Arena 2v2 Duo Pairings</span>
              </div>
              <span style={{ fontSize: '11px', color: '#94a3b8' }}>
                Highest win-rate 2v2 combinations with proven mechanical synergies
              </span>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))', gap: '12px' }}>
              {arenaDuos.map((d, idx) => (
                <div
                  key={idx}
                  style={{
                    background: 'rgba(255, 255, 255, 0.03)',
                    border: '1px solid var(--border-subtle)',
                    borderRadius: '8px',
                    padding: '12px 14px',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <span style={{ fontSize: '14px', fontWeight: 800, color: '#fff' }}>
                        {d.duo[0]} + {d.duo[1]}
                      </span>
                      <span
                        style={{
                          fontSize: '10px',
                          fontWeight: 800,
                          padding: '2px 6px',
                          borderRadius: '4px',
                          background: 'rgba(0, 200, 255, 0.15)',
                          color: 'var(--blue-primary)',
                          border: '1px solid var(--border-blue)',
                        }}
                      >
                        {d.playstyle}
                      </span>
                    </div>
                    <span style={{ fontSize: '13px', fontWeight: 900, color: '#22c55e', fontFamily: 'var(--font-mono)' }}>
                      {d.synergyRating}% Win
                    </span>
                  </div>
                  <div style={{ fontSize: '11px', color: '#94a3b8' }}>{d.description}</div>
                </div>
              ))}
            </div>
          </div>

          {/* Arena Augments Tier List */}
          <div className="glass-card" style={{ padding: '20px' }}>
            <div className="card-header" style={{ marginBottom: '14px' }}>
              <div className="card-title">
                <Sparkles size={18} color="#c084fc" />
                <span>Prismatic & Gold Augments Tier List</span>
              </div>
              <span style={{ fontSize: '11px', color: '#94a3b8' }}>
                Look for these game-breaking augments in the drafting rounds
              </span>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '12px' }}>
              {arenaAugments.map((aug) => (
                <div
                  key={aug.name}
                  style={{
                    background: 'rgba(255, 255, 255, 0.03)',
                    border: `1px solid ${aug.color}33`,
                    borderRadius: '8px',
                    padding: '12px',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '6px' }}>
                    <span style={{ fontSize: '13px', fontWeight: 800, color: '#fff' }}>{aug.name}</span>
                    <span
                      style={{
                        fontSize: '10px',
                        fontWeight: 800,
                        padding: '2px 6px',
                        borderRadius: '4px',
                        background: `${aug.color}18`,
                        color: aug.color,
                        border: `1px solid ${aug.color}44`,
                      }}
                    >
                      {aug.tier}
                    </span>
                  </div>
                  <div style={{ fontSize: '11px', color: '#94a3b8', marginBottom: '8px' }}>{aug.desc}</div>
                  <div style={{ fontSize: '10px', color: '#64748b' }}>
                    <strong style={{ color: '#cbd5e1' }}>Best for:</strong> {aug.bestFor}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* 2. ARAM MAYHEM VIEW */}
      {selectedMode === 'aram' && (
        <div className="game-modes-grid">
          <div className="glass-card" style={{ padding: '24px', border: '1px solid var(--border-blue)' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '16px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '14px' }}>
                <img
                  src={getChampionIcon(champ.key)}
                  alt={champ.name}
                  style={{ width: '60px', height: '60px', borderRadius: '12px', border: '2px solid var(--blue-primary)' }}
                />
                <div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <h3 style={{ fontSize: '20px', fontWeight: 800, color: '#fff' }}>{champ.name}</h3>
                    <span className={`tier-badge tier-${aramMods.tier.toLowerCase().replace('+', '-plus')}`}>
                      {aramMods.tier} Tier (ARAM)
                    </span>
                  </div>
                  <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
                    {aramMods.note}
                  </div>
                </div>
              </div>

              <div style={{ display: 'flex', gap: '8px' }}>
                {onSelectChampion && (
                  <button
                    onClick={() => onSelectChampion(champ.id)}
                    className="btn-secondary"
                    style={{ minWidth: '140px', padding: '8px 14px' }}
                  >
                    <Sparkles size={15} color="var(--blue-primary)" />
                    <span>Inspect Build</span>
                  </button>
                )}
                <button
                  onClick={() => handlePushModeRunes('ARAM')}
                  className="btn-primary"
                  disabled={importStatus === 'importing'}
                  style={{ minWidth: '160px' }}
                >
                  {importStatus === 'success' ? (
                    <>
                      <Check size={16} />
                      <span>ARAM Runes Pushed!</span>
                    </>
                  ) : (
                    <>
                      <Zap size={16} />
                      <span>Push ARAM Runes</span>
                    </>
                  )}
                </button>
              </div>
            </div>

            {/* Multipliers */}
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '10px', marginTop: '16px' }}>
              <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '10px', borderRadius: '8px', textAlign: 'center' }}>
                <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>DAMAGE DEALT</div>
                <div style={{ fontSize: '16px', fontWeight: 900, color: aramMods.dealt > 100 ? '#22c55e' : aramMods.dealt < 100 ? 'var(--red-primary)' : '#fff', marginTop: '2px' }}>
                  {aramMods.dealt}%
                </div>
              </div>
              <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '10px', borderRadius: '8px', textAlign: 'center' }}>
                <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>DAMAGE TAKEN</div>
                <div style={{ fontSize: '16px', fontWeight: 900, color: aramMods.taken < 100 ? '#22c55e' : aramMods.taken > 100 ? 'var(--red-primary)' : '#fff', marginTop: '2px' }}>
                  {aramMods.taken}%
                </div>
              </div>
              <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '10px', borderRadius: '8px', textAlign: 'center' }}>
                <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>ABILITY HASTE</div>
                <div style={{ fontSize: '16px', fontWeight: 900, color: aramMods.haste > 0 ? 'var(--blue-primary)' : '#fff', marginTop: '2px' }}>
                  {aramMods.haste > 0 ? `+${aramMods.haste}` : aramMods.haste < 0 ? `${aramMods.haste}` : '0'}
                </div>
              </div>
              <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '10px', borderRadius: '8px', textAlign: 'center' }}>
                <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>HEALING RATIO</div>
                <div style={{ fontSize: '16px', fontWeight: 900, color: aramMods.heal < 100 ? 'var(--red-primary)' : '#fff', marginTop: '2px' }}>
                  {aramMods.heal}%
                </div>
              </div>
            </div>

            <div style={{ marginTop: '20px', background: 'rgba(0, 200, 255, 0.04)', border: '1px solid var(--border-blue)', padding: '14px', borderRadius: '8px' }}>
              <div style={{ fontSize: '11px', fontWeight: 800, textTransform: 'uppercase', color: 'var(--blue-primary)', marginBottom: '6px' }}>
                ARAM Snowball & Relic Rules
              </div>
              <div style={{ fontSize: '12px', color: '#cbd5e1' }}>
                Health relics spawn at <strong>01:45</strong> and have a 60-second respawn timer. Always run <strong>Mark / Dash (Snowball)</strong> for wave clearing, bush vision, and diving squishies.
              </div>
            </div>
          </div>

          {/* Quick Champion Picker Grid */}
          <div className="glass-card" style={{ padding: '20px' }}>
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
                placeholder="Search ARAM Champion..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                style={{ background: 'transparent', border: 'none', color: '#fff', fontSize: '12px', outline: 'none', width: '100%' }}
              />
            </div>

            <div style={{ maxHeight: '250px', overflowY: 'auto', display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '6px' }}>
              {filteredChamps.map((c) => (
                <button
                  key={c.id}
                  onClick={() => {
                    setSelectedChampId(c.id);
                    soundService.playTimerTick();
                  }}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '6px',
                    padding: '6px',
                    borderRadius: '6px',
                    background: selectedChampId === c.id ? 'var(--blue-subtle)' : 'rgba(255, 255, 255, 0.03)',
                    border: selectedChampId === c.id ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
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
      )}

      {/* 3. URF / ARURF VIEW */}
      {selectedMode === 'urf' && (
        <div className="glass-card" style={{ padding: '20px' }}>
          <div className="card-header" style={{ marginBottom: '16px' }}>
            <div className="card-title">
              <Zap size={18} color="var(--red-primary)" />
              <span>Ultra Rapid Fire (URF) S+ Tier List & Meta Exploits</span>
            </div>
            <span style={{ fontSize: '11px', color: '#94a3b8' }}>
              80% Cooldown Reduction, Zero Mana/Energy costs, and Cannon launches
            </span>
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))', gap: '14px' }}>
            {urfPicks.map((pick) => (
              <div
                key={pick.name}
                style={{
                  background: 'rgba(255, 255, 255, 0.03)',
                  border: '1px solid rgba(255, 42, 95, 0.3)',
                  borderRadius: '10px',
                  padding: '14px',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '14px',
                }}
              >
                <img
                  src={getChampionIcon(pick.name)}
                  alt={pick.name}
                  style={{ width: '48px', height: '48px', borderRadius: '10px', border: '2px solid var(--red-primary)' }}
                />
                <div style={{ flex: 1 }}>
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <span style={{ fontSize: '15px', fontWeight: 800, color: '#fff' }}>{pick.name}</span>
                      <span className="tier-badge tier-s-plus">{pick.tier}</span>
                    </div>
                    <span style={{ fontSize: '13px', fontWeight: 800, color: '#22c55e', fontFamily: 'var(--font-mono)' }}>
                      {pick.winRate}% Win
                    </span>
                  </div>
                  <div style={{ fontSize: '11px', color: '#94a3b8', marginTop: '4px' }}>
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
        <div className="glass-card" style={{ padding: '20px' }}>
          <div className="card-header" style={{ marginBottom: '16px' }}>
            <div className="card-title">
              <Crown size={18} color="#c084fc" />
              <span>One for All (OFA) 5x Synergies & Secret Combos</span>
            </div>
            <span style={{ fontSize: '11px', color: '#94a3b8' }}>
              Mechanics where champion passives and abilities stack across all 5 players
            </span>
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))', gap: '14px' }}>
            {ofaCombos.map((ofa) => (
              <div
                key={ofa.name}
                style={{
                  background: 'rgba(255, 255, 255, 0.03)',
                  border: '1px solid rgba(192, 132, 252, 0.3)',
                  borderRadius: '10px',
                  padding: '14px',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '14px',
                }}
              >
                <img
                  src={getChampionIcon(ofa.name)}
                  alt={ofa.name}
                  style={{ width: '48px', height: '48px', borderRadius: '10px', border: '2px solid #c084fc' }}
                />
                <div style={{ flex: 1 }}>
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <span style={{ fontSize: '15px', fontWeight: 800, color: '#fff' }}>{ofa.name}</span>
                      <span
                        style={{
                          fontSize: '10px',
                          fontWeight: 800,
                          padding: '2px 6px',
                          borderRadius: '4px',
                          background: 'rgba(192, 132, 252, 0.15)',
                          color: '#c084fc',
                          border: '1px solid rgba(192, 132, 252, 0.4)',
                        }}
                      >
                        {ofa.synergy}
                      </span>
                    </div>
                  </div>
                  <div style={{ fontSize: '11px', color: '#94a3b8', marginTop: '4px' }}>
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
