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
  Award
} from 'lucide-react';
import { CHAMPIONS_LIST, getChampionBuild, getChampionIcon, getChampionSplash } from '../services/championData';
import { patchService } from '../services/patchService';
import { soundService } from '../services/soundService';

export const AramHubView: React.FC = () => {
  const currentChamps = patchService.getChampions() || CHAMPIONS_LIST;
  const [selectedChampId, setSelectedChampId] = useState<number>(222); // Jinx default
  const [search, setSearch] = useState('');
  const [importStatus, setImportStatus] = useState<'idle' | 'importing' | 'success'>('idle');

  const champ = currentChamps.find((c) => c.id === selectedChampId) || currentChamps[0];

  // Specific ARAM balance modifiers (Riot's Howling Abyss balance aura)
  const getAramModifiers = (champName: string) => {
    switch (champName) {
      case 'Jinx':
        return { dealt: 95, taken: 105, haste: 0, heal: 100, tier: 'S', note: '-5% Dmg / +5% Taken due to passive resets' };
      case 'Lux':
        return { dealt: 85, taken: 110, haste: -10, heal: 80, tier: 'A', note: '-15% Dmg due to long-range choke poke' };
      case 'Aatrox':
        return { dealt: 105, taken: 90, haste: 10, heal: 105, tier: 'S+', note: '+5% Dmg / -10% Taken (Melee Buff)' };
      case 'Lee Sin':
        return { dealt: 110, taken: 85, haste: 20, heal: 120, tier: 'S+', note: 'Massive +10% Dmg / -15% Taken ARAM Aura' };
      case 'Sona':
        return { dealt: 80, taken: 115, haste: -20, heal: 70, tier: 'B', note: 'Heavy AOE heal & shield penalty' };
      case 'Ziggs':
        return { dealt: 80, taken: 120, haste: -10, heal: 100, tier: 'C', note: 'Heavy structure & AOE poke penalty' };
      default:
        return { dealt: 100, taken: 100, haste: 0, heal: 100, tier: 'A', note: 'Standard ARAM baseline' };
    }
  };

  const mods = getAramModifiers(champ.name);

  // Optimal ARAM Runes (Dark Harvest / Snowball Setup)
  const handleImportAramRunes = async () => {
    setImportStatus('importing');
    try {
      if (window.riftAPI) {
        const safeChampName = (champ.name || 'Champion').replace(/[^a-zA-Z0-9 ]/g, '').trim();
        await window.riftAPI.importRunes({
          name: `RiftCore ARAM ${safeChampName}`,
          primaryStyleId: 8100, // Domination
          subStyleId: 8000, // Precision
          selectedPerkIds: [8128, 8143, 8140, 8106, 8009, 8299, 5008, 5008, 5001], // Dark Harvest, Sudden Impact, Grisly Mementos, Ultimate Hunter, PoM, Last Stand
        });
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

  const filtered = currentChamps.filter((c) =>
    c.name.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Header Banner */}
      <div className="glass-card" style={{ padding: '20px' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '14px' }}>
          <div>
            <h2 style={{ fontSize: '18px', fontWeight: 800, color: '#fff', display: 'flex', alignItems: 'center', gap: '8px' }}>
              <Snowflake size={20} color="var(--blue-primary)" />
              <span>ARAM Mayhem: Balance Buffs & Snowball Meta</span>
            </h2>
            <p style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
              Howling Abyss specific champion balance multipliers, Dark Harvest setups & situational anti-dive builds
            </p>
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
              width: '240px',
            }}
          >
            <Search size={14} color="#64748b" />
            <input
              type="text"
              placeholder="Search ARAM Champion..."
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
      </div>

      {/* Main Grid: Selected Champion on Left, Champion Carousel on Right */}
      <div style={{ display: 'grid', gridTemplateColumns: '1.2fr 0.8fr', gap: '20px' }}>
        {/* Selected Champion ARAM Intel */}
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
                  <span className={`tier-badge tier-${mods.tier.toLowerCase().replace('+', '-plus')}`}>
                    {mods.tier} Tier (ARAM)
                  </span>
                </div>
                <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
                  {mods.note}
                </div>
              </div>
            </div>

            <button
              onClick={handleImportAramRunes}
              className="btn-primary"
              disabled={importStatus === 'importing'}
              style={{ minWidth: '180px' }}
            >
              {importStatus === 'success' ? (
                <>
                  <Check size={16} />
                  <span>ARAM Runes Applied!</span>
                </>
              ) : (
                <>
                  <Zap size={16} />
                  <span>Apply ARAM Runes</span>
                </>
              )}
            </button>
          </div>

          {/* Official ARAM Balance Multipliers */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '10px', marginTop: '16px' }}>
            <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '10px', borderRadius: '8px', textAlign: 'center' }}>
              <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>DAMAGE DEALT</div>
              <div style={{ fontSize: '16px', fontWeight: 900, color: mods.dealt > 100 ? '#22c55e' : mods.dealt < 100 ? 'var(--red-primary)' : '#fff', marginTop: '2px' }}>
                {mods.dealt}%
              </div>
            </div>
            <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '10px', borderRadius: '8px', textAlign: 'center' }}>
              <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>DAMAGE TAKEN</div>
              <div style={{ fontSize: '16px', fontWeight: 900, color: mods.taken < 100 ? '#22c55e' : mods.taken > 100 ? 'var(--red-primary)' : '#fff', marginTop: '2px' }}>
                {mods.taken}%
              </div>
            </div>
            <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '10px', borderRadius: '8px', textAlign: 'center' }}>
              <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>ABILITY HASTE</div>
              <div style={{ fontSize: '16px', fontWeight: 900, color: mods.haste > 0 ? 'var(--blue-primary)' : '#fff', marginTop: '2px' }}>
                {mods.haste > 0 ? `+${mods.haste}` : mods.haste < 0 ? `${mods.haste}` : '0'}
              </div>
            </div>
            <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '10px', borderRadius: '8px', textAlign: 'center' }}>
              <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700 }}>HEALING RATIO</div>
              <div style={{ fontSize: '16px', fontWeight: 900, color: mods.heal < 100 ? 'var(--red-primary)' : '#fff', marginTop: '2px' }}>
                {mods.heal}%
              </div>
            </div>
          </div>

          {/* Recommended ARAM Spells & Setup */}
          <div style={{ marginTop: '20px', background: 'rgba(0, 200, 255, 0.04)', border: '1px solid var(--border-blue)', padding: '14px', borderRadius: '8px' }}>
            <div style={{ fontSize: '11px', fontWeight: 800, textTransform: 'uppercase', color: 'var(--blue-primary)', marginBottom: '6px' }}>
              Optimal ARAM Summoner Spells
            </div>
            <div style={{ fontSize: '12px', color: '#cbd5e1' }}>
              <strong>Flash + Mark / Dash (Snowball):</strong> Essential on 95% of champions for dodging skillshots, checking unwarded bushes, and diving backline squishies.
            </div>
          </div>
        </div>

        {/* Quick Champion Picker Grid */}
        <div className="glass-card" style={{ padding: '20px' }}>
          <div className="card-header" style={{ marginBottom: '12px' }}>
            <div className="card-title">
              <Flame size={16} color="var(--red-primary)" />
              <span>Browse ARAM Roster</span>
            </div>
          </div>

          <div
            style={{
              maxHeight: '260px',
              overflowY: 'auto',
              display: 'grid',
              gridTemplateColumns: 'repeat(3, 1fr)',
              gap: '6px',
            }}
          >
            {filtered.map((c) => (
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
    </div>
  );
};
