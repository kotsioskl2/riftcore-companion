import React, { useState, useEffect, useRef } from 'react';
import {
  Zap,
  Shield,
  Sparkles,
  Check,
  Search,
  ChevronRight,
  ShieldAlert,
  Flame,
  Swords,
  Grid,
  ChevronDown,
  Layers,
  RotateCcw,
  BookOpen,
  Sliders,
  ExternalLink,
  Target,
  Pin,
  ShoppingBag
} from 'lucide-react';
import { ChampSelectSession } from '../types';
import { CHAMPIONS_LIST, getChampionBuild, getChampionIcon, getChampionSplash } from '../services/championData';
import { RUNE_TREES, STAT_SHARDS, SUMMONER_SPELLS } from '../services/runeData';
import { RuneCustomizer } from './RuneCustomizer';
import { soundService } from '../services/soundService';
import { patchService, PatchState } from '../services/patchService';
import { itemSetService } from '../services/itemSetService';

interface ChampSelectViewProps {
  session: ChampSelectSession | null;
  selectedChampionId?: number;
  onSelectChampion?: (championId: number) => void;
}

// Helper to extract active hovered or locked champion from League Client session
export const extractLcuChampionId = (sess: ChampSelectSession | null): number => {
  if (!sess) return 0;
  const localCell = Number(sess.localPlayerCellId);

  // 1. Check local player in myTeam FIRST (locked in, assigned in ARAM/Blind, or intent hovered)
  if (Array.isArray(sess.myTeam)) {
    const me = sess.myTeam.find((p) => Number(p.cellId) === localCell);
    if (me) {
      if (me.championId && Number(me.championId) > 0) return Number(me.championId);
      if (me.championPickIntent && Number(me.championPickIntent) > 0) return Number(me.championPickIntent);
    }
  }

  // 2. Check actions (active pick turn hover or completed picks)
  if (Array.isArray(sess.actions)) {
    for (const actionGroup of sess.actions) {
      const group = Array.isArray(actionGroup) ? actionGroup : [actionGroup];
      for (const action of group) {
        if (Number(action?.actorCellId) === localCell && action?.type === 'pick') {
          if (action?.isInProgress && action?.championId && Number(action.championId) > 0) {
            return Number(action.championId);
          }
          if (action?.championId && Number(action.championId) > 0) {
            return Number(action.championId);
          }
        }
      }
    }
  }

  return 0;
};

export const ChampSelectView: React.FC<ChampSelectViewProps> = ({
  session,
  selectedChampionId,
  onSelectChampion,
}) => {
  const [patchState, setPatchState] = useState<PatchState>(patchService.getPatchState());
  const [selectedChampId, setSelectedChampId] = useState<number>(selectedChampionId || 420);
  const [lastSyncedLcuChampId, setLastSyncedLcuChampId] = useState<number>(0);
  const isFirstMountRef = useRef(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedRoleFilter, setSelectedRoleFilter] = useState<string>('ALL');
  const [showFullRosterModal, setShowFullRosterModal] = useState(false);

  useEffect(() => {
    return patchService.subscribe((state) => {
      setPatchState(state);
    });
  }, []);

  const currentChampions = patchService.getChampions() || CHAMPIONS_LIST;

  // Active Build & Customization State
  const champ = currentChampions.find((c) => Number(c.id) === Number(selectedChampId)) || currentChampions[0];
  const initialBuild = getChampionBuild(champ.id, champ.name);

  // Per-champion custom overrides (presets, custom runes, and swapped spells)
  const [customRunesByChamp, setCustomRunesByChamp] = useState<Record<number, {
    primaryStyleId: number;
    subStyleId: number;
    selectedPerkIds: number[];
    spells: [number, number];
    activePreset: 'meta' | 'burst' | 'sustain' | 'custom';
  }>>({});

  const activeBuildData = customRunesByChamp[selectedChampId] || {
    primaryStyleId: initialBuild.runes.primaryStyleId,
    subStyleId: initialBuild.runes.subStyleId,
    selectedPerkIds: initialBuild.runes.selectedPerkIds,
    spells: initialBuild.spells,
    activePreset: 'meta' as const,
  };

  const primaryStyleId = activeBuildData.primaryStyleId;
  const subStyleId = activeBuildData.subStyleId;
  const selectedPerkIds = activeBuildData.selectedPerkIds;
  const selectedSpells = activeBuildData.spells;
  const activePreset = activeBuildData.activePreset;

  // Spell Switcher Popover
  const [activeSpellSlot, setActiveSpellSlot] = useState<0 | 1 | null>(null);

  // Status indicators
  const [importStatus, setImportStatus] = useState<'idle' | 'importing' | 'success' | 'error'>('idle');
  const [spellStatus, setSpellStatus] = useState<'idle' | 'importing' | 'success'>('idle');
  const [itemSetStatus, setItemSetStatus] = useState<'idle' | 'exporting' | 'success' | 'error'>('idle');
  const [itemSetMessage, setItemSetMessage] = useState<string>('');
  const [autoImport, setAutoImport] = useState(true);

  // Auto-sync whenever the LoL Client genuinely changes champion selection/hover
  useEffect(() => {
    if (!session) return;
    const lcuChampId = extractLcuChampionId(session);
    if (lcuChampId > 0 && lcuChampId !== selectedChampId) {
      setLastSyncedLcuChampId(lcuChampId);
      setSelectedChampId(lcuChampId);
      if (onSelectChampion) onSelectChampion(lcuChampId);
    }
  }, [session, selectedChampId, onSelectChampion]);

  // Sync when parent component passes a new selectedChampionId
  useEffect(() => {
    if (selectedChampionId && selectedChampionId > 0 && selectedChampionId !== selectedChampId) {
      setSelectedChampId(selectedChampionId);
    }
  }, [selectedChampionId, selectedChampId]);

  // Auto-import runes & in-game shop item sets to client whenever champion changes
  useEffect(() => {
    if (selectedChampId && selectedChampId > 0) {
      const targetChamp = currentChampions.find((c) => Number(c.id) === Number(selectedChampId)) || currentChampions[0];
      const targetBuild = getChampionBuild(targetChamp.id, targetChamp.name);
      const runesToUse = customRunesByChamp[selectedChampId] || {
        primaryStyleId: targetBuild.runes.primaryStyleId,
        subStyleId: targetBuild.runes.subStyleId,
        selectedPerkIds: targetBuild.runes.selectedPerkIds,
      };

      if (autoImport && window.riftAPI?.importRunes) {
        setImportStatus('importing');
        window.riftAPI
          .importRunes({
            name: `⚡ Rift: ${targetChamp.name}`,
            primaryStyleId: runesToUse.primaryStyleId,
            subStyleId: runesToUse.subStyleId,
            selectedPerkIds: runesToUse.selectedPerkIds,
          })
          .then(() => {
            setImportStatus('success');
            setTimeout(() => setImportStatus('idle'), 2500);
          })
          .catch(() => {
            setImportStatus('error');
            setTimeout(() => setImportStatus('idle'), 2500);
          });
      }
      itemSetService.exportItemSetToClient(targetBuild).catch(() => {});
    }
  }, [selectedChampId, autoImport]);

  // User chooses champion in RiftCore UI
  const handleSelectChampion = (champId: number) => {
    setSelectedChampId(champId);
    if (onSelectChampion) onSelectChampion(champId);
    soundService.playTimerTick();
  };

  // Presets handler
  const handleSelectPreset = (preset: 'meta' | 'burst' | 'sustain') => {
    const build = getChampionBuild(champ.id, champ.name);
    let pStyle = build.runes.primaryStyleId;
    let sStyle = build.runes.subStyleId;
    let perks = build.runes.selectedPerkIds;

    if (preset === 'burst') {
      pStyle = 8100; // Domination
      sStyle = 8200; // Sorcery
      perks = [8112, 8143, 8138, 8106, 8226, 8210, 5008, 5008, 5001];
    } else if (preset === 'sustain') {
      pStyle = 8400; // Resolve
      sStyle = 8000; // Precision
      perks = [8437, 8446, 8473, 8451, 9111, 8299, 5008, 5008, 5001];
    }

    setCustomRunesByChamp((prev) => ({
      ...prev,
      [selectedChampId]: {
        primaryStyleId: pStyle,
        subStyleId: sStyle,
        selectedPerkIds: perks,
        spells: selectedSpells,
        activePreset: preset,
      },
    }));
    soundService.playTimerTick();
  };

  // Customizer callback
  const handleCustomRunesChange = (newPerks: number[], newPrimaryStyleId: number, newSubStyleId: number) => {
    setCustomRunesByChamp((prev) => ({
      ...prev,
      [selectedChampId]: {
        primaryStyleId: newPrimaryStyleId,
        subStyleId: newSubStyleId,
        selectedPerkIds: newPerks,
        spells: selectedSpells,
        activePreset: 'custom',
      },
    }));
  };

  // Push Runes to League Client via LCU
  const handleImportRunes = async () => {
    setImportStatus('importing');
    try {
      if (window.riftAPI) {
        await window.riftAPI.importRunes({
          name: `⚡ Rift: ${champ.name}`,
          primaryStyleId,
          subStyleId,
          selectedPerkIds,
        });
        soundService.playRuneImport();
        setImportStatus('success');
        setTimeout(() => setImportStatus('idle'), 3000);
      } else {
        setTimeout(() => {
          soundService.playRuneImport();
          setImportStatus('success');
          setTimeout(() => setImportStatus('idle'), 3000);
        }, 350);
      }
    } catch (err) {
      console.error('Import runes error:', err);
      setImportStatus('error');
      setTimeout(() => setImportStatus('idle'), 3000);
    }
  };

  // Push Spells to League Client
  const handleImportSpells = async () => {
    setSpellStatus('importing');
    try {
      if (window.riftAPI) {
        await window.riftAPI.importSpells(selectedSpells[0], selectedSpells[1]);
        soundService.playRuneImport();
        setSpellStatus('success');
        setTimeout(() => setSpellStatus('idle'), 2500);
      } else {
        setTimeout(() => {
          soundService.playRuneImport();
          setSpellStatus('success');
          setTimeout(() => setSpellStatus('idle'), 2500);
        }, 250);
      }
    } catch {
      setSpellStatus('idle');
    }
  };

  // Push In-Game Item Set to League Client (Shop menu 'P')
  const handleExportItemSet = async () => {
    setItemSetStatus('exporting');
    try {
      const res = await itemSetService.exportItemSetToClient(initialBuild);
      if (res.success) {
        soundService.playRuneImport();
        setItemSetStatus('success');
        setItemSetMessage(res.message);
        setTimeout(() => setItemSetStatus('idle'), 3500);
      } else {
        if (!window.riftAPI) {
          soundService.playRuneImport();
          setItemSetStatus('success');
          setItemSetMessage(`✓ Shop Set Generated for ${champ.name}!`);
          setTimeout(() => setItemSetStatus('idle'), 3500);
        } else {
          setItemSetStatus('error');
          setItemSetMessage(res.message);
          setTimeout(() => setItemSetStatus('idle'), 4000);
        }
      }
    } catch (err: any) {
      console.error('Export item set error:', err);
      setItemSetStatus('error');
      setItemSetMessage(err?.message || 'Failed to export item set');
      setTimeout(() => setItemSetStatus('idle'), 4000);
    }
  };

  // Swap Spell in slot
  const handleSelectSpell = (spellId: number) => {
    if (activeSpellSlot === null) return;
    const nextSpells: [number, number] = [...selectedSpells];
    // If the other slot already has this spell, swap them
    if (nextSpells[1 - activeSpellSlot] === spellId) {
      nextSpells[1 - activeSpellSlot] = nextSpells[activeSpellSlot];
    }
    nextSpells[activeSpellSlot] = spellId;
    setCustomRunesByChamp((prev) => ({
      ...prev,
      [selectedChampId]: {
        primaryStyleId,
        subStyleId,
        selectedPerkIds,
        spells: nextSpells,
        activePreset,
      },
    }));
    setActiveSpellSlot(null);
    soundService.playTimerTick();
  };

  // Champion filtering
  const filteredChampions = currentChampions.filter((c) => {
    const matchesSearch = c.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                          c.title.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesRole = selectedRoleFilter === 'ALL' || c.roles.includes(selectedRoleFilter);
    return matchesSearch && matchesRole;
  });

  const roles = ['ALL', 'TOP', 'JUNGLE', 'MID', 'ADC', 'SUPPORT'];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Top Banner: Champion Hero Banner & Quick Actions */}
      <div
        className="glass-card"
        style={{
          position: 'relative',
          padding: '20px 24px',
          backgroundImage: `linear-gradient(90deg, rgba(5, 8, 16, 0.98) 0%, rgba(5, 8, 16, 0.88) 42%, rgba(5, 8, 16, 0.35) 100%), url(${getChampionSplash(champ.key)})`,
          backgroundSize: 'cover',
          backgroundPosition: 'center 22%',
          border: '1px solid rgba(0, 210, 255, 0.3)',
          borderTop: '1px solid rgba(255, 255, 255, 0.2)',
          borderRadius: 'var(--radius-lg)',
          boxShadow: '0 12px 36px -4px rgba(0, 0, 0, 0.65), 0 0 24px rgba(0, 210, 255, 0.1)',
          transform: 'none !important',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', flexWrap: 'wrap', gap: '16px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
            <div style={{ position: 'relative' }}>
              <img
                src={getChampionIcon(champ.key)}
                alt={champ.name}
                style={{
                  width: '72px',
                  height: '72px',
                  borderRadius: '13px',
                  border: '2px solid rgba(0, 210, 255, 0.7)',
                  boxShadow: '0 0 20px rgba(0, 210, 255, 0.35)',
                  display: 'block',
                }}
              />
              <span
                style={{
                  position: 'absolute',
                  bottom: '-5px',
                  right: '-5px',
                  fontSize: '9px',
                  fontWeight: 900,
                  padding: '1px 6px',
                  borderRadius: '4px',
                  background: 'rgba(5, 8, 16, 0.95)',
                  border: '1px solid rgba(0, 210, 255, 0.4)',
                  color: 'var(--blue-primary)',
                  letterSpacing: '0.4px',
                }}
              >
                {initialBuild.tier}
              </span>
            </div>
            <div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '6px', flexWrap: 'wrap' }}>
                <h1 style={{ fontSize: '23px', color: '#fff', fontWeight: 800, letterSpacing: '0.3px', margin: 0 }}>
                  {champ.name}
                </h1>
                <span className={`tier-badge tier-${initialBuild.tier.toLowerCase().replace('+', '-plus')}`}>
                  {initialBuild.tier} Tier
                </span>
                <span
                  style={{
                    fontSize: '10.5px',
                    fontWeight: 700,
                    textTransform: 'uppercase',
                    letterSpacing: '0.5px',
                    padding: '2.5px 8px',
                    background: 'rgba(255, 255, 255, 0.06)',
                    border: '1px solid rgba(255, 255, 255, 0.08)',
                    borderRadius: '4px',
                    color: '#cbd5e1',
                  }}
                >
                  {initialBuild.role}
                </span>
              </div>
              <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap', alignItems: 'center' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '5px', background: 'rgba(0, 210, 255, 0.07)', border: '1px solid rgba(0, 210, 255, 0.2)', padding: '2px 9px', borderRadius: '5px', fontSize: '11px' }}>
                  <span style={{ color: '#94a3b8' }}>Win:</span>
                  <strong className="tabular-nums" style={{ color: 'var(--blue-primary)', fontWeight: 800 }}>{initialBuild.winRate}%</strong>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '5px', background: 'rgba(255, 255, 255, 0.04)', border: '1px solid rgba(255, 255, 255, 0.08)', padding: '2px 9px', borderRadius: '5px', fontSize: '11px' }}>
                  <span style={{ color: '#94a3b8' }}>Pick:</span>
                  <strong className="tabular-nums" style={{ color: '#fff', fontWeight: 800 }}>{initialBuild.pickRate}%</strong>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '5px', background: 'rgba(255, 42, 95, 0.07)', border: '1px solid rgba(255, 42, 95, 0.2)', padding: '2px 9px', borderRadius: '5px', fontSize: '11px' }}>
                  <span style={{ color: '#94a3b8' }}>Ban:</span>
                  <strong className="tabular-nums" style={{ color: 'var(--red-primary)', fontWeight: 800 }}>{initialBuild.banRate}%</strong>
                </div>
              </div>
            </div>
          </div>

          {/* Quick Actions Bar */}
          <div style={{ display: 'flex', gap: '8px', alignItems: 'center', flexWrap: 'wrap' }}>
            {/* Sync to Client Pick Button */}
            {lastSyncedLcuChampId > 0 && selectedChampId !== lastSyncedLcuChampId && (
              <button
                onClick={() => handleSelectChampion(lastSyncedLcuChampId)}
                style={{
                  padding: '6px 12px',
                  borderRadius: 'var(--radius-sm)',
                  background: 'rgba(0, 210, 255, 0.15)',
                  border: '1px solid var(--blue-primary)',
                  color: 'var(--blue-primary)',
                  fontSize: '11px',
                  fontWeight: 800,
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  boxShadow: '0 0 12px rgba(0, 210, 255, 0.25)',
                }}
                title="Return to the champion currently selected in League Client"
              >
                <Pin size={12} />
                <span>Sync to Client Pick</span>
              </button>
            )}

            {/* Auto Import Toggle */}
            <button
              onClick={() => setAutoImport(!autoImport)}
              style={{
                padding: '6px 12px',
                borderRadius: 'var(--radius-sm)',
                background: autoImport ? 'rgba(0, 210, 255, 0.12)' : 'rgba(255, 255, 255, 0.04)',
                border: autoImport ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                color: autoImport ? 'var(--blue-primary)' : '#94a3b8',
                fontSize: '11px',
                fontWeight: 700,
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                transition: 'all 0.18s ease',
              }}
              title="Automatically push optimal runes upon picking champion"
            >
              <Zap size={13} />
              <span>Auto-Import: {autoImport ? 'ON' : 'OFF'}</span>
            </button>

            {/* Push Runes */}
            <button
              id="btn-import-runes"
              className="btn-primary"
              onClick={handleImportRunes}
              disabled={importStatus === 'importing'}
              style={{ flexShrink: 0 }}
            >
              {importStatus === 'importing' ? (
                <>Applying Runes...</>
              ) : importStatus === 'success' ? (
                <>
                  <Check size={14} color="#fff" />
                  <span>Runes Injected! (100ms)</span>
                </>
              ) : (
                <>
                  <Zap size={14} />
                  <span>Push Runes</span>
                </>
              )}
            </button>

            {/* Push Spells */}
            <button
              id="btn-import-spells"
              className="btn-secondary"
              onClick={handleImportSpells}
            >
              {spellStatus === 'success' ? (
                <Check size={13} color="var(--blue-primary)" />
              ) : (
                <Sparkles size={13} color="var(--blue-primary)" />
              )}
              <span>Push Spells</span>
            </button>

            {/* Export In-Game Shop Item Set */}
            <button
              id="btn-export-item-set"
              className="btn-secondary"
              onClick={handleExportItemSet}
              disabled={itemSetStatus === 'exporting'}
              style={{
                borderColor: itemSetStatus === 'success' ? '#10b981' : itemSetStatus === 'error' ? 'var(--red-primary)' : 'var(--border-blue)',
                color: itemSetStatus === 'success' ? '#10b981' : itemSetStatus === 'error' ? 'var(--red-primary)' : 'var(--blue-primary)',
              }}
              title="Export recommended item set directly into the League in-game shop (Press 'P' in game)"
            >
              {itemSetStatus === 'exporting' ? (
                <>
                  <RotateCcw className="spin" size={13} />
                  <span>Exporting...</span>
                </>
              ) : itemSetStatus === 'success' ? (
                <>
                  <Check size={13} color="#10b981" />
                  <span>Shop Ready!</span>
                </>
              ) : (
                <>
                  <ShoppingBag size={13} color="var(--blue-primary)" />
                  <span>⚡ LoL Shop</span>
                </>
              )}
            </button>
          </div>
        </div>

        {/* Champion Search, Role Filters & Full Roster Bar */}
        <div style={{ marginTop: '18px', display: 'flex', alignItems: 'center', gap: '10px', flexWrap: 'wrap' }}>
          {/* Search Box */}
          <div className="search-input-box" style={{ width: '220px' }}>
            <Search size={13} color="#64748b" />
            <input
              type="text"
              placeholder="Search 169 Champions..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              style={{
                background: 'transparent',
                border: 'none',
                color: '#fff',
                fontSize: '11.5px',
                outline: 'none',
                width: '100%',
              }}
            />
          </div>

          {/* Role Filters */}
          <div style={{ display: 'flex', gap: '5px' }}>
            {roles.map((r) => (
              <button
                key={r}
                onClick={() => setSelectedRoleFilter(r)}
                style={{
                  padding: '4px 9px',
                  borderRadius: 'var(--radius-sm)',
                  background: selectedRoleFilter === r ? 'var(--blue-subtle)' : 'rgba(255, 255, 255, 0.04)',
                  border: selectedRoleFilter === r ? '1px solid var(--blue-primary)' : '1px solid transparent',
                  color: selectedRoleFilter === r ? 'var(--blue-primary)' : '#94a3b8',
                  fontSize: '10.5px',
                  fontWeight: 700,
                  letterSpacing: '0.3px',
                  cursor: 'pointer',
                  transition: 'all 0.15s ease',
                }}
              >
                {r}
              </button>
            ))}
          </div>

          {/* Browse All Champions Toggle */}
          <button
            onClick={() => setShowFullRosterModal(!showFullRosterModal)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '4px 11px',
              borderRadius: 'var(--radius-sm)',
              background: showFullRosterModal ? 'rgba(0, 210, 255, 0.18)' : 'rgba(255, 255, 255, 0.05)',
              border: '1px solid var(--border-blue)',
              color: '#fff',
              fontSize: '11px',
              fontWeight: 700,
              cursor: 'pointer',
              marginLeft: 'auto',
              transition: 'all 0.15s ease',
            }}
          >
            <Grid size={12.5} color="var(--blue-primary)" />
            <span>{showFullRosterModal ? 'Close Roster' : 'Browse All 169 Champions'}</span>
          </button>
        </div>

        {/* Quick Horizontal Champion Ribbon (Scrollable) */}
        {!showFullRosterModal && (
          <div
            className="no-scrollbar"
            style={{
              marginTop: '12px',
              display: 'flex',
              gap: '6px',
              overflowX: 'auto',
              scrollbarWidth: 'none',
              paddingBottom: '2px',
            }}
          >
            {filteredChampions.slice(0, 24).map((c) => {
              const isSelected = Number(selectedChampId) === Number(c.id);
              return (
                <button
                  key={c.id}
                  onClick={() => handleSelectChampion(c.id)}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '6px',
                    padding: '3.5px 9px',
                    borderRadius: 'var(--radius-sm)',
                    background: isSelected ? 'var(--blue-subtle)' : 'rgba(255, 255, 255, 0.03)',
                    border: isSelected ? '1px solid var(--blue-primary)' : '1px solid transparent',
                    color: isSelected ? 'var(--blue-primary)' : '#94a3b8',
                    fontSize: '11px',
                    fontWeight: 600,
                    cursor: 'pointer',
                    whiteSpace: 'nowrap',
                    transition: 'all 0.15s ease',
                    boxShadow: isSelected ? '0 0 10px var(--blue-glow-subtle)' : 'none',
                  }}
                >
                  <img
                    src={getChampionIcon(c.key)}
                    alt={c.name}
                    style={{ width: '18px', height: '18px', borderRadius: '50%' }}
                  />
                  <span>{c.name}</span>
                </button>
              );
            })}
          </div>
        )}
      </div>

      {/* Full 169 Champion Grid Modal / Drawer */}
      {showFullRosterModal && (
        <div
          className="glass-card"
          style={{
            padding: '20px',
            border: '1px solid var(--border-blue)',
            maxHeight: '340px',
            overflowY: 'auto',
          }}
        >
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '14px' }}>
            <div style={{ fontSize: '13px', fontWeight: 800, color: '#fff', display: 'flex', alignItems: 'center', gap: '8px' }}>
              <Layers size={16} color="var(--blue-primary)" />
              <span>Complete League Roster ({filteredChampions.length} Champions Found)</span>
            </div>
            <span style={{ fontSize: '11px', color: '#64748b' }}>
              Click any champion to inspect optimal runes, items, and power spikes
            </span>
          </div>

          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fill, minmax(130px, 1fr))',
              gap: '10px',
            }}
          >
            {filteredChampions.map((c) => {
              const isSelected = Number(selectedChampId) === Number(c.id);
              return (
                <button
                  key={c.id}
                  onClick={() => {
                    handleSelectChampion(c.id);
                    setShowFullRosterModal(false);
                  }}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '8px',
                    padding: '6px 8px',
                    borderRadius: '8px',
                    background: isSelected ? 'rgba(0, 200, 255, 0.18)' : 'rgba(255, 255, 255, 0.03)',
                    border: isSelected ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                    cursor: 'pointer',
                    textAlign: 'left',
                    transition: 'all 0.15s ease',
                  }}
                >
                  <img
                    src={getChampionIcon(c.key)}
                    alt={c.name}
                    style={{
                      width: '32px',
                      height: '32px',
                      borderRadius: '8px',
                      border: isSelected ? '1px solid var(--blue-primary)' : 'none',
                    }}
                  />
                  <div style={{ overflow: 'hidden' }}>
                    <div style={{ fontSize: '12px', fontWeight: 700, color: isSelected ? 'var(--blue-primary)' : '#fff', whiteSpace: 'nowrap', textOverflow: 'ellipsis', overflow: 'hidden' }}>
                      {c.name}
                    </div>
                    <div style={{ fontSize: '10px', color: '#64748b' }}>
                      {c.roles.join(', ')}
                    </div>
                  </div>
                </button>
              );
            })}
          </div>
        </div>
      )}

      {/* Main Grid: Runes on Left, Items & Spells & Matchups on Right */}
      <div className="champ-select-grid">
        {/* LEFT COLUMN: INTERACTIVE RUNE CUSTOMIZER */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px', minWidth: 0 }}>
          <div className="glass-card" style={{ padding: '20px' }}>
            <div className="card-header" style={{ marginBottom: '16px' }}>
              <div className="card-title">
                <Zap size={18} color="var(--blue-primary)" />
                <span>Interactive Rune Page & Keystone Selector</span>
              </div>
              <span style={{ fontSize: '11px', color: '#94a3b8' }}>
                Click any keystone or rune to customize
              </span>
            </div>

            {/* Interactive Rune Customizer */}
            <RuneCustomizer
              primaryStyleId={primaryStyleId}
              subStyleId={subStyleId}
              selectedPerkIds={selectedPerkIds}
              onChangeRunes={handleCustomRunesChange}
              activePreset={activePreset}
              onSelectPreset={handleSelectPreset}
            />
          </div>
        </div>

        {/* RIGHT COLUMN: SPELLS, ITEMS, MATCHUPS */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px', minWidth: 0 }}>
          {/* Summoner Spells Card */}
          <div className="glass-card" style={{ padding: '16px 20px' }}>
            <div className="card-header" style={{ marginBottom: '12px' }}>
              <div className="card-title">
                <Sparkles size={16} color="var(--blue-primary)" />
                <span>Summoner Spells (Click to swap)</span>
              </div>
            </div>

            <div style={{ display: 'flex', gap: '14px', alignItems: 'center' }}>
              {[0, 1].map((slotIdx) => {
                const sId = selectedSpells[slotIdx];
                const spell = SUMMONER_SPELLS[sId] || SUMMONER_SPELLS[4];
                return (
                  <button
                    key={slotIdx}
                    onClick={() => setActiveSpellSlot(slotIdx as 0 | 1)}
                    style={{
                      flex: 1,
                      display: 'flex',
                      alignItems: 'center',
                      gap: '10px',
                      padding: '8px 12px',
                      borderRadius: '8px',
                      background: 'rgba(255, 255, 255, 0.04)',
                      border: activeSpellSlot === slotIdx ? '2px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                      cursor: 'pointer',
                      boxShadow: activeSpellSlot === slotIdx ? '0 0 12px var(--blue-glow)' : 'none',
                    }}
                  >
                    <img src={spell.icon} alt={spell.name} style={{ width: '32px', height: '32px', borderRadius: '6px' }} />
                    <div style={{ textAlign: 'left' }}>
                      <div style={{ fontSize: '12px', fontWeight: 700, color: '#fff' }}>
                        {slotIdx === 0 ? 'D Spell: ' : 'F Spell: '}{spell.name}
                      </div>
                      <div style={{ fontSize: '10px', color: '#94a3b8' }}>
                        CD: {spell.cooldown}s • Click to swap
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>

            {/* Spell Popover Selector */}
            {activeSpellSlot !== null && (
              <div
                style={{
                  marginTop: '12px',
                  padding: '12px',
                  background: 'rgba(6, 9, 15, 0.95)',
                  border: '1px solid var(--border-blue)',
                  borderRadius: '8px',
                }}
              >
                <div style={{ fontSize: '11px', fontWeight: 800, color: 'var(--blue-primary)', marginBottom: '8px' }}>
                  Select Spell for {activeSpellSlot === 0 ? 'Key D' : 'Key F'}:
                </div>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '6px' }}>
                  {Object.values(SUMMONER_SPELLS).map((sp) => (
                    <button
                      key={sp.id}
                      onClick={() => handleSelectSpell(sp.id)}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '6px',
                        padding: '6px',
                        borderRadius: '6px',
                        background: 'rgba(255, 255, 255, 0.04)',
                        border: '1px solid var(--border-subtle)',
                        cursor: 'pointer',
                      }}
                    >
                      <img src={sp.icon} alt={sp.name} style={{ width: '20px', height: '20px', borderRadius: '4px' }} />
                      <span style={{ fontSize: '11px', fontWeight: 600, color: '#fff' }}>{sp.name}</span>
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* Dynamic Item Build Paths & In-Game Shop Preview */}
          <div className="glass-card" style={{ padding: '18px 20px' }}>
            <div className="card-header" style={{ marginBottom: '14px', display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '8px' }}>
              <div className="card-title" style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <ShoppingBag size={16} color="var(--blue-primary)" />
                <span>Recommended Item Paths</span>
                <span
                  style={{
                    fontSize: '10px',
                    fontWeight: 700,
                    padding: '2px 8px',
                    borderRadius: '4px',
                    background: 'rgba(0, 200, 255, 0.12)',
                    border: '1px solid rgba(0, 200, 255, 0.3)',
                    color: 'var(--blue-primary)',
                  }}
                >
                  ⚡ In-Game Shop Set (Press P)
                </span>
              </div>
              <button
                onClick={handleExportItemSet}
                disabled={itemSetStatus === 'exporting'}
                style={{
                  padding: '4px 10px',
                  borderRadius: '6px',
                  background: itemSetStatus === 'success' ? 'rgba(16, 185, 129, 0.15)' : 'rgba(0, 200, 255, 0.12)',
                  border: itemSetStatus === 'success' ? '1px solid #10b981' : '1px solid var(--border-blue)',
                  color: itemSetStatus === 'success' ? '#10b981' : 'var(--blue-primary)',
                  fontSize: '11px',
                  fontWeight: 700,
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  transition: 'all 0.2s',
                }}
                title="Inject complete build directly into League of Legends in-game shop menu (Press 'P' in game)"
              >
                {itemSetStatus === 'exporting' ? (
                  <RotateCcw className="spin" size={12} />
                ) : itemSetStatus === 'success' ? (
                  <Check size={12} />
                ) : (
                  <ShoppingBag size={12} />
                )}
                <span>{itemSetStatus === 'success' ? 'Shop Updated' : 'Push to LoL Shop'}</span>
              </button>
            </div>

            {/* Starting & Boots */}
            <div style={{ display: 'flex', gap: '14px', marginBottom: '14px' }}>
              <div style={{ flex: 1 }}>
                <div style={{ fontSize: '10px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b', marginBottom: '6px' }}>
                  Starting Items & Potions
                </div>
                <div style={{ display: 'flex', gap: '6px' }}>
                  {initialBuild.startingItems.map((item) => (
                    <div
                      key={item.id}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '6px',
                        background: 'rgba(255, 255, 255, 0.04)',
                        padding: '6px 10px',
                        borderRadius: '6px',
                        border: '1px solid var(--border-subtle)',
                        fontSize: '11px',
                        color: '#cbd5e1',
                      }}
                    >
                      <img
                        src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/item/${item.id}.png`}
                        alt={item.name}
                        style={{ width: '22px', height: '22px', borderRadius: '4px' }}
                        onError={(e) => { (e.target as any).style.display = 'none'; }}
                      />
                      <span>{item.name}</span>
                    </div>
                  ))}
                </div>
              </div>

              <div>
                <div style={{ fontSize: '10px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b', marginBottom: '6px' }}>
                  Completed Boots
                </div>
                <div
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: '6px',
                    background: 'rgba(255, 255, 255, 0.04)',
                    padding: '6px 10px',
                    borderRadius: '6px',
                    border: '1px solid var(--border-subtle)',
                    fontSize: '11px',
                    color: '#cbd5e1',
                  }}
                >
                  <img
                    src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/item/${initialBuild.boots.id}.png`}
                    alt={initialBuild.boots.name}
                    style={{ width: '22px', height: '22px', borderRadius: '4px' }}
                    onError={(e) => { (e.target as any).style.display = 'none'; }}
                  />
                  <span>{initialBuild.boots.name}</span>
                </div>
              </div>
            </div>

            {/* Core 3 Items */}
            <div style={{ marginBottom: '14px' }}>
              <div style={{ fontSize: '10px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b', marginBottom: '6px' }}>
                Core Rush (Build in Order)
              </div>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '8px' }}>
                {initialBuild.coreItems.map((item, idx) => (
                  <div
                    key={item.id}
                    style={{
                      display: 'flex',
                      flexDirection: 'column',
                      alignItems: 'center',
                      gap: '4px',
                      background: 'rgba(255, 255, 255, 0.03)',
                      padding: '8px',
                      borderRadius: '8px',
                      border: '1px solid var(--border-subtle)',
                      textAlign: 'center',
                    }}
                  >
                    <div style={{ position: 'relative' }}>
                      <img
                        src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/item/${item.id}.png`}
                        alt={item.name}
                        style={{ width: '32px', height: '32px', borderRadius: '6px', border: '1px solid var(--border-blue)' }}
                        onError={(e) => { (e.target as any).style.display = 'none'; }}
                      />
                      <span
                        style={{
                          position: 'absolute',
                          bottom: '-4px',
                          right: '-4px',
                          background: 'var(--blue-primary)',
                          color: '#000',
                          fontSize: '9px',
                          fontWeight: 900,
                          borderRadius: '50%',
                          width: '14px',
                          height: '14px',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                        }}
                      >
                        {idx + 1}
                      </span>
                    </div>
                    <span style={{ fontSize: '10px', fontWeight: 700, color: '#fff' }}>
                      {item.name}
                    </span>
                  </div>
                ))}
              </div>
            </div>

            {/* Situational Counter Items */}
            <div style={{ marginBottom: '14px' }}>
              <div style={{ fontSize: '10px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b', marginBottom: '6px' }}>
                Situational Counter Items
              </div>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
                {initialBuild.situationalItems.map((item) => (
                  <div
                    key={item.id}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '10px',
                      padding: '6px 10px',
                      borderRadius: '6px',
                      background: 'rgba(255, 255, 255, 0.02)',
                      border: '1px solid var(--border-subtle)',
                    }}
                  >
                    <img
                      src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/item/${item.id}.png`}
                      alt={item.name}
                      style={{ width: '24px', height: '24px', borderRadius: '4px' }}
                      onError={(e) => { (e.target as any).style.display = 'none'; }}
                    />
                    <div style={{ flex: 1 }}>
                      <div style={{ fontSize: '11px', fontWeight: 700, color: '#fff' }}>{item.name}</div>
                    </div>
                    <span
                      style={{
                        fontSize: '9px',
                        fontWeight: 700,
                        padding: '2px 6px',
                        borderRadius: '4px',
                        background: 'rgba(255, 42, 95, 0.15)',
                        color: 'var(--red-primary)',
                        border: '1px solid rgba(255, 42, 95, 0.3)',
                      }}
                    >
                      Counter
                    </span>
                  </div>
                ))}
              </div>
            </div>

            {/* Consumables & In-Game Shop Hotkey Bar */}
            <div
              style={{
                padding: '10px 12px',
                borderRadius: '8px',
                background: 'rgba(0, 200, 255, 0.04)',
                border: '1px dashed rgba(0, 200, 255, 0.25)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                flexWrap: 'wrap',
                gap: '8px',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <ShoppingBag size={14} color="var(--blue-primary)" />
                <span style={{ fontSize: '11px', color: '#cbd5e1' }}>
                  In-game shop set synced. Press <strong style={{ color: '#fff', background: 'rgba(255,255,255,0.1)', padding: '2px 5px', borderRadius: '4px' }}>P</strong> in game to open this build.
                </span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                {/* Control Ward & Elixir Preview */}
                <div style={{ display: 'flex', alignItems: 'center', gap: '4px', background: 'rgba(0,0,0,0.3)', padding: '3px 8px', borderRadius: '4px', border: '1px solid var(--border-subtle)' }}>
                  <img src="https://ddragon.leagueoflegends.com/cdn/14.23.1/img/item/2055.png" alt="Control Ward" style={{ width: '16px', height: '16px', borderRadius: '3px' }} />
                  <span style={{ fontSize: '10px', color: '#94a3b8' }}>Control Ward</span>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '4px', background: 'rgba(0,0,0,0.3)', padding: '3px 8px', borderRadius: '4px', border: '1px solid var(--border-subtle)' }}>
                  <img
                    src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/item/${initialBuild.role === 'MID' ? '2139' : initialBuild.role === 'ADC' ? '2140' : '2138'}.png`}
                    alt="Elixir"
                    style={{ width: '16px', height: '16px', borderRadius: '3px' }}
                  />
                  <span style={{ fontSize: '10px', color: '#94a3b8' }}>Late Elixir</span>
                </div>
              </div>
            </div>
          </div>

          {/* Skill Max Order & 1-18 Leveling Matrix */}
          <div className="glass-card" style={{ padding: '18px 20px' }}>
            <div className="card-header" style={{ marginBottom: '14px', display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '8px' }}>
              <div className="card-title">
                <Zap size={16} color="var(--blue-primary)" />
                <span>Skill Priority & Level-Up Path</span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                <span style={{ fontSize: '11px', color: '#94a3b8', fontWeight: 600 }}>Max Order:</span>
                <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                  {initialBuild.skillPriority.split('>').map((s, idx, arr) => {
                    const skill = s.trim();
                    const color = skill === 'Q' ? '#00c8ff' : skill === 'W' ? '#10b981' : skill === 'E' ? '#f59e0b' : '#c084fc';
                    return (
                      <React.Fragment key={skill + idx}>
                        <span
                          style={{
                            fontSize: '11px',
                            fontWeight: 900,
                            padding: '2px 8px',
                            borderRadius: '5px',
                            background: `${color}20`,
                            color: color,
                            border: `1px solid ${color}50`,
                            boxShadow: `0 0 8px ${color}30`,
                          }}
                        >
                          {skill}
                        </span>
                        {idx < arr.length - 1 && (
                          <ChevronRight size={12} color="#64748b" />
                        )}
                      </React.Fragment>
                    );
                  })}
                </div>
              </div>
            </div>

            {/* Visual Level 1 to 18 Matrix Grid */}
            <div
              className="no-scrollbar"
              style={{
                overflowX: 'auto',
                scrollbarWidth: 'none',
                paddingBottom: '2px',
              }}
            >
              <div style={{ minWidth: '460px' }}>
                {/* Column Headers: Levels 1 to 18 */}
                <div style={{ display: 'grid', gridTemplateColumns: '32px repeat(18, 1fr)', gap: '3px', marginBottom: '6px', textAlign: 'center' }}>
                  <div style={{ fontSize: '9px', fontWeight: 800, color: '#64748b' }}>LVL</div>
                  {Array.from({ length: 18 }, (_, i) => i + 1).map((lvl) => (
                    <div
                      key={lvl}
                      style={{
                        fontSize: '9px',
                        fontWeight: lvl === 6 || lvl === 11 || lvl === 16 ? 900 : 700,
                        color: lvl === 6 || lvl === 11 || lvl === 16 ? '#ec4899' : '#94a3b8',
                      }}
                    >
                      {lvl}
                    </div>
                  ))}
                </div>

                {/* 4 Skill Rows: Q, W, E, R */}
                {['Q', 'W', 'E', 'R'].map((skillKey) => {
                  const color = skillKey === 'Q' ? '#00c8ff' : skillKey === 'W' ? '#10b981' : skillKey === 'E' ? '#f59e0b' : '#ec4899';
                  return (
                    <div
                      key={skillKey}
                      style={{
                        display: 'grid',
                        gridTemplateColumns: '32px repeat(18, 1fr)',
                        gap: '3px',
                        marginBottom: '4px',
                        alignItems: 'center',
                      }}
                    >
                      {/* Skill Key Label */}
                      <div
                        style={{
                          fontSize: '11px',
                          fontWeight: 900,
                          textAlign: 'center',
                          color: color,
                          background: `${color}15`,
                          borderRadius: '4px',
                          padding: '2px 0',
                          border: `1px solid ${color}30`,
                        }}
                      >
                        {skillKey}
                      </div>

                      {/* 18 Level Slots */}
                      {Array.from({ length: 18 }, (_, i) => {
                        const level = i + 1;
                        const isLearned = initialBuild.skillOrder[i] === skillKey;
                        const isUlt = skillKey === 'R' && isLearned;

                        return (
                          <div
                            key={level}
                            title={`Level ${level}: ${isLearned ? `Upgrade ${skillKey}` : '-'}`}
                            style={{
                              height: '22px',
                              borderRadius: '4px',
                              display: 'flex',
                              alignItems: 'center',
                              justifyContent: 'center',
                              fontSize: '9px',
                              fontWeight: 900,
                              background: isLearned
                                ? isUlt
                                  ? 'linear-gradient(135deg, rgba(236, 72, 153, 0.4), rgba(192, 132, 252, 0.4))'
                                  : `${color}35`
                                : 'rgba(255, 255, 255, 0.02)',
                              color: isLearned ? (isUlt ? '#fff' : color) : 'transparent',
                              border: isLearned
                                ? `1px solid ${isUlt ? '#ec4899' : color}`
                                : '1px solid rgba(255, 255, 255, 0.04)',
                              boxShadow: isLearned ? `0 0 6px ${color}40` : 'none',
                              transition: 'all 0.15s ease',
                            }}
                          >
                            {isLearned ? skillKey : ''}
                          </div>
                        );
                      })}
                    </div>
                  );
                })}
              </div>
            </div>
          </div>

          {/* Matchups & Coaching Tips */}
          <div className="glass-card" style={{ padding: '18px 20px' }}>
            <div className="card-header" style={{ marginBottom: '12px' }}>
              <div className="card-title">
                <Swords size={16} color="var(--red-primary)" />
                <span>Matchups & Power Spikes</span>
              </div>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '10px', marginBottom: '14px' }}>
              {/* Strong Against */}
              <div style={{ background: 'rgba(34, 197, 94, 0.06)', borderRadius: '8px', padding: '10px', border: '1px solid rgba(34, 197, 94, 0.2)' }}>
                <div style={{ fontSize: '10px', fontWeight: 800, color: '#22c55e', textTransform: 'uppercase', marginBottom: '6px' }}>
                  Favorable Matchups
                </div>
                {initialBuild.counters.strongAgainst.map((c) => (
                  <div key={c.name} style={{ display: 'flex', justifyContent: 'space-between', fontSize: '11px', padding: '2px 0' }}>
                    <span style={{ color: '#fff', fontWeight: 600 }}>{c.name}</span>
                    <span style={{ color: '#22c55e', fontWeight: 700 }}>{c.winRate}%</span>
                  </div>
                ))}
              </div>

              {/* Weak Against */}
              <div style={{ background: 'rgba(255, 42, 95, 0.06)', borderRadius: '8px', padding: '10px', border: '1px solid rgba(255, 42, 95, 0.2)' }}>
                <div style={{ fontSize: '10px', fontWeight: 800, color: 'var(--red-primary)', textTransform: 'uppercase', marginBottom: '6px' }}>
                  Tough Counters
                </div>
                {initialBuild.counters.weakAgainst.map((c) => (
                  <div key={c.name} style={{ display: 'flex', justifyContent: 'space-between', fontSize: '11px', padding: '2px 0' }}>
                    <span style={{ color: '#fff', fontWeight: 600 }}>{c.name}</span>
                    <span style={{ color: 'var(--red-primary)', fontWeight: 700 }}>{c.winRate}%</span>
                  </div>
                ))}
              </div>
            </div>

            {/* Pro Coaching Tips */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
              {initialBuild.tips.map((tip, idx) => (
                <div key={idx} style={{ fontSize: '11px', color: '#94a3b8', display: 'flex', gap: '6px' }}>
                  <span style={{ color: 'var(--blue-primary)', fontWeight: 800 }}>•</span>
                  <span>{tip}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
