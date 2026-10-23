import React from 'react';
import { RUNE_TREES, STAT_SHARDS, STAT_SHARD_ROWS, RuneTree } from '../services/runeData';
import { Sparkles, Shield, Flame, Zap } from 'lucide-react';

interface RuneCustomizerProps {
  primaryStyleId: number;
  subStyleId: number;
  selectedPerkIds: number[];
  onChangeRunes: (newPerks: number[], primaryStyleId: number, subStyleId: number) => void;
  activePreset?: 'meta' | 'burst' | 'sustain' | 'custom';
  onSelectPreset?: (preset: 'meta' | 'burst' | 'sustain') => void;
}

export const RuneCustomizer: React.FC<RuneCustomizerProps> = ({
  primaryStyleId,
  subStyleId,
  selectedPerkIds,
  onChangeRunes,
  activePreset = 'custom',
  onSelectPreset,
}) => {
  const primaryTree = RUNE_TREES[primaryStyleId] || RUNE_TREES[8000];
  const subTree = RUNE_TREES[subStyleId] || RUNE_TREES[8400];

  const keystoneId = selectedPerkIds[0] || primaryTree.keystones[0]?.id;
  const primarySlotIds = [
    selectedPerkIds[1] || primaryTree.slots[0]?.[0]?.id,
    selectedPerkIds[2] || primaryTree.slots[1]?.[0]?.id,
    selectedPerkIds[3] || primaryTree.slots[2]?.[0]?.id,
  ];

  // Secondary perks (2 perks)
  const secondarySlotIds = [
    selectedPerkIds[4] || subTree.slots[0]?.[0]?.id,
    selectedPerkIds[5] || subTree.slots[1]?.[0]?.id,
  ];

  // Shards (3 perks)
  const shardIds = [
    selectedPerkIds[6] || 5008,
    selectedPerkIds[7] || 5008,
    selectedPerkIds[8] || 5001,
  ];

  // Helper to trigger update
  const emitUpdate = (
    newPrimaryStyleId: number,
    newSubStyleId: number,
    newKeystoneId: number,
    newPrimarySlots: number[],
    newSecondarySlots: number[],
    newShards: number[]
  ) => {
    const fullPerks = [
      newKeystoneId,
      ...newPrimarySlots,
      ...newSecondarySlots,
      ...newShards,
    ];
    onChangeRunes(fullPerks, newPrimaryStyleId, newSubStyleId);
  };

  // Switch Primary Tree
  const handleSelectPrimaryTree = (styleId: number) => {
    if (styleId === primaryStyleId) return;
    const newTree = RUNE_TREES[styleId];
    let nextSubStyleId = subStyleId;
    if (nextSubStyleId === styleId) {
      // Pick another sub style
      const candidate = Object.keys(RUNE_TREES).map(Number).find((id) => id !== styleId);
      nextSubStyleId = candidate || 8400;
    }
    const nextSubTree = RUNE_TREES[nextSubStyleId];
    const newKeystone = newTree.keystones[0].id;
    const newPrimarySlots = [
      newTree.slots[0][0].id,
      newTree.slots[1][0].id,
      newTree.slots[2][0].id,
    ];
    const newSecondarySlots = [
      nextSubTree.slots[0][0].id,
      nextSubTree.slots[1][0].id,
    ];
    emitUpdate(styleId, nextSubStyleId, newKeystone, newPrimarySlots, newSecondarySlots, shardIds);
  };

  // Switch Secondary Tree
  const handleSelectSecondaryTree = (styleId: number) => {
    if (styleId === subStyleId || styleId === primaryStyleId) return;
    const nextSubTree = RUNE_TREES[styleId];
    const newSecondarySlots = [
      nextSubTree.slots[0][0].id,
      nextSubTree.slots[1][0].id,
    ];
    emitUpdate(primaryStyleId, styleId, keystoneId, primarySlotIds, newSecondarySlots, shardIds);
  };

  // Switch Keystone
  const handleSelectKeystone = (kId: number) => {
    emitUpdate(primaryStyleId, subStyleId, kId, primarySlotIds, secondarySlotIds, shardIds);
  };

  // Switch Primary Minor Slot (row: 0, 1, 2)
  const handleSelectPrimarySlot = (rowIndex: number, perkId: number) => {
    const nextSlots = [...primarySlotIds];
    nextSlots[rowIndex] = perkId;
    emitUpdate(primaryStyleId, subStyleId, keystoneId, nextSlots, secondarySlotIds, shardIds);
  };

  // Switch Secondary Perk
  const handleToggleSecondaryPerk = (perkId: number) => {
    if (secondarySlotIds.includes(perkId)) return;
    // Replace the oldest or second slot
    const nextSec = [secondarySlotIds[1], perkId];
    emitUpdate(primaryStyleId, subStyleId, keystoneId, primarySlotIds, nextSec, shardIds);
  };

  // Switch Shard (row: 0, 1, 2)
  const handleSelectShard = (rowIndex: number, shardId: number) => {
    const nextShards = [...shardIds];
    nextShards[rowIndex] = shardId;
    emitUpdate(primaryStyleId, subStyleId, keystoneId, primarySlotIds, secondarySlotIds, nextShards);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
      {/* Preset Switcher Bar */}
      {onSelectPreset && (
        <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap', alignItems: 'center' }}>
          <span style={{ fontSize: '11px', fontWeight: 800, textTransform: 'uppercase', color: '#94a3b8' }}>
            Rune Build:
          </span>
          <button
            onClick={() => onSelectPreset('meta')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '4px 12px',
              borderRadius: '6px',
              background: activePreset === 'meta' ? 'var(--blue-subtle)' : 'rgba(255, 255, 255, 0.04)',
              border: activePreset === 'meta' ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
              color: activePreset === 'meta' ? 'var(--blue-primary)' : '#94a3b8',
              fontSize: '11px',
              fontWeight: 700,
              cursor: 'pointer',
            }}
          >
            <Flame size={13} />
            <span>Highest Win Rate (Meta)</span>
          </button>

          <button
            onClick={() => onSelectPreset('burst')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '4px 12px',
              borderRadius: '6px',
              background: activePreset === 'burst' ? 'rgba(255, 42, 95, 0.15)' : 'rgba(255, 255, 255, 0.04)',
              border: activePreset === 'burst' ? '1px solid var(--red-primary)' : '1px solid var(--border-subtle)',
              color: activePreset === 'burst' ? 'var(--red-primary)' : '#94a3b8',
              fontSize: '11px',
              fontWeight: 700,
              cursor: 'pointer',
            }}
          >
            <Zap size={13} />
            <span>Lethality & Burst</span>
          </button>

          <button
            onClick={() => onSelectPreset('sustain')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '4px 12px',
              borderRadius: '6px',
              background: activePreset === 'sustain' ? 'rgba(34, 197, 94, 0.15)' : 'rgba(255, 255, 255, 0.04)',
              border: activePreset === 'sustain' ? '1px solid #22c55e' : '1px solid var(--border-subtle)',
              color: activePreset === 'sustain' ? '#22c55e' : '#94a3b8',
              fontSize: '11px',
              fontWeight: 700,
              cursor: 'pointer',
            }}
          >
            <Shield size={13} />
            <span>Anti-Tank / Sustain</span>
          </button>

          {activePreset === 'custom' && (
            <span
              style={{
                fontSize: '11px',
                fontWeight: 700,
                color: 'var(--blue-primary)',
                padding: '3px 8px',
                borderRadius: '4px',
                background: 'rgba(0, 200, 255, 0.1)',
                border: '1px solid var(--border-blue)',
              }}
            >
              ✏️ Custom Tuned
            </span>
          )}
        </div>
      )}

      {/* Primary & Secondary Dual Columns */}
      <div className="rune-dual-columns">
        {/* PRIMARY STYLE */}
        <div
          style={{
            background: 'rgba(6, 9, 15, 0.65)',
            border: '1px solid var(--border-subtle)',
            borderRadius: '10px',
            padding: '14px',
            display: 'flex',
            flexDirection: 'column',
            gap: '14px',
          }}
        >
          {/* Primary Tree Selector Tabs */}
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', borderBottom: '1px solid var(--border-subtle)', paddingBottom: '10px' }}>
            <span style={{ fontSize: '12px', fontWeight: 800, textTransform: 'uppercase', color: primaryTree.color }}>
              Primary: {primaryTree.name}
            </span>
            <div style={{ display: 'flex', gap: '6px' }}>
              {Object.values(RUNE_TREES).map((tree) => {
                const isSelected = tree.id === primaryStyleId;
                return (
                  <button
                    key={tree.id}
                    onClick={() => handleSelectPrimaryTree(tree.id)}
                    title={tree.name}
                    style={{
                      width: '30px',
                      height: '30px',
                      borderRadius: '50%',
                      background: isSelected ? 'rgba(255, 255, 255, 0.15)' : 'rgba(255, 255, 255, 0.03)',
                      border: isSelected ? `2px solid ${tree.color}` : '1px solid transparent',
                      cursor: 'pointer',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      transition: 'all 0.15s ease',
                      boxShadow: isSelected ? `0 0 10px ${tree.color}55` : 'none',
                    }}
                  >
                    <img
                      src={tree.icon}
                      alt={tree.name}
                      style={{ width: '18px', height: '18px' }}
                      onError={(e) => {
                        (e.target as HTMLElement).style.display = 'none';
                      }}
                    />
                  </button>
                );
              })}
            </div>
          </div>

          {/* Keystones Row */}
          <div>
            <div style={{ fontSize: '11px', fontWeight: 700, color: '#64748b', textTransform: 'uppercase', marginBottom: '8px' }}>
              Keystone Perk (Click to switch)
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(130px, 1fr))', gap: '8px' }}>
              {primaryTree.keystones.map((k) => {
                const isSelected = k.id === keystoneId;
                return (
                  <button
                    key={k.id}
                    onClick={() => handleSelectKeystone(k.id)}
                    title={`${k.name}: ${k.desc}`}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '8px',
                      padding: '8px 10px',
                      borderRadius: '8px',
                      background: isSelected ? 'rgba(0, 200, 255, 0.12)' : 'rgba(255, 255, 255, 0.03)',
                      border: isSelected ? '2px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                      cursor: 'pointer',
                      transition: 'all 0.15s ease',
                      boxShadow: isSelected ? '0 0 14px var(--blue-glow)' : 'none',
                      opacity: isSelected ? 1 : 0.65,
                      minWidth: 0,
                    }}
                  >
                    <img
                      src={k.icon}
                      alt={k.name}
                      style={{
                        width: '32px',
                        height: '32px',
                        borderRadius: '50%',
                        flexShrink: 0,
                        transform: isSelected ? 'scale(1.08)' : 'scale(1)',
                        transition: 'transform 0.15s ease',
                      }}
                    />
                    <div style={{ textAlign: 'left', minWidth: 0, overflow: 'hidden' }}>
                      <div style={{ fontSize: '12px', fontWeight: 700, color: isSelected ? '#fff' : '#94a3b8', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                        {k.name}
                      </div>
                      <div style={{ fontSize: '10px', color: '#64748b', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                        {k.desc}
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>
          </div>

          {/* Primary Minor Slots (3 rows) */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {primaryTree.slots.map((row, rowIdx) => {
              const activeSlotPerkId = primarySlotIds[rowIdx];
              return (
                <div
                  key={rowIdx}
                  style={{
                    background: 'rgba(255, 255, 255, 0.02)',
                    borderRadius: '8px',
                    padding: '8px 10px',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '10px',
                  }}
                >
                  <div style={{ fontSize: '10px', fontWeight: 800, color: '#64748b', textTransform: 'uppercase', width: '45px', flexShrink: 0 }}>
                    Slot {rowIdx + 1}
                  </div>
                  <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '8px', flex: 1 }}>
                    {row.map((perk) => {
                      const isSelected = perk.id === activeSlotPerkId;
                      return (
                        <button
                          key={perk.id}
                          onClick={() => handleSelectPrimarySlot(rowIdx, perk.id)}
                          title={`${perk.name}: ${perk.desc}`}
                          style={{
                            display: 'flex',
                            alignItems: 'center',
                            gap: '6px',
                            padding: '6px 8px',
                            borderRadius: '6px',
                            background: isSelected ? 'rgba(0, 200, 255, 0.14)' : 'rgba(255, 255, 255, 0.02)',
                            border: isSelected ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                            cursor: 'pointer',
                            opacity: isSelected ? 1 : 0.6,
                            transition: 'all 0.15s ease',
                            minWidth: 0,
                          }}
                        >
                          <img src={perk.icon} alt={perk.name} style={{ width: '22px', height: '22px', borderRadius: '50%', flexShrink: 0 }} />
                          <span style={{
                            fontSize: '11px',
                            fontWeight: 600,
                            color: isSelected ? '#fff' : '#94a3b8',
                            whiteSpace: 'nowrap',
                            overflow: 'hidden',
                            textOverflow: 'ellipsis',
                            textAlign: 'left',
                          }}>
                            {perk.name}
                          </span>
                        </button>
                      );
                    })}
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* SECONDARY STYLE & SHARDS */}
        <div
          style={{
            background: 'rgba(6, 9, 15, 0.65)',
            border: '1px solid var(--border-subtle)',
            borderRadius: '10px',
            padding: '14px',
            display: 'flex',
            flexDirection: 'column',
            gap: '14px',
          }}
        >
          {/* Secondary Tree Selector Tabs */}
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', borderBottom: '1px solid var(--border-subtle)', paddingBottom: '10px' }}>
            <span style={{ fontSize: '12px', fontWeight: 800, textTransform: 'uppercase', color: subTree.color }}>
              Secondary: {subTree.name}
            </span>
            <div style={{ display: 'flex', gap: '6px' }}>
              {Object.values(RUNE_TREES)
                .filter((t) => t.id !== primaryStyleId)
                .map((tree) => {
                  const isSelected = tree.id === subStyleId;
                  return (
                    <button
                      key={tree.id}
                      onClick={() => handleSelectSecondaryTree(tree.id)}
                      title={tree.name}
                      style={{
                        width: '28px',
                        height: '28px',
                        borderRadius: '50%',
                        background: isSelected ? 'rgba(255, 255, 255, 0.15)' : 'rgba(255, 255, 255, 0.03)',
                        border: isSelected ? `2px solid ${tree.color}` : '1px solid transparent',
                        cursor: 'pointer',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        transition: 'all 0.15s ease',
                        boxShadow: isSelected ? `0 0 10px ${tree.color}55` : 'none',
                      }}
                    >
                      <img
                        src={tree.icon}
                        alt={tree.name}
                        style={{ width: '16px', height: '16px' }}
                        onError={(e) => {
                          (e.target as HTMLElement).style.display = 'none';
                        }}
                      />
                    </button>
                  );
                })}
            </div>
          </div>

          {/* Secondary Perks (Click 2 to select) */}
          <div>
            <div style={{ fontSize: '11px', fontWeight: 700, color: '#64748b', textTransform: 'uppercase', marginBottom: '8px' }}>
              Choose 2 Secondary Perks
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {subTree.slots.map((row, rowIdx) => (
                <div key={rowIdx} style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '8px' }}>
                  {row.map((perk) => {
                    const isSelected = secondarySlotIds.includes(perk.id);
                    return (
                      <button
                        key={perk.id}
                        onClick={() => handleToggleSecondaryPerk(perk.id)}
                        title={`${perk.name}: ${perk.desc}`}
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          gap: '6px',
                          padding: '6px 8px',
                          borderRadius: '6px',
                          background: isSelected ? 'rgba(34, 197, 94, 0.15)' : 'rgba(255, 255, 255, 0.02)',
                          border: isSelected ? '1px solid #22c55e' : '1px solid var(--border-subtle)',
                          cursor: 'pointer',
                          opacity: isSelected ? 1 : 0.6,
                          transition: 'all 0.15s ease',
                          minWidth: 0,
                        }}
                      >
                        <img src={perk.icon} alt={perk.name} style={{ width: '18px', height: '18px', borderRadius: '50%', flexShrink: 0 }} />
                        <span
                          style={{
                            fontSize: '11px',
                            fontWeight: 600,
                            color: isSelected ? '#fff' : '#94a3b8',
                            overflow: 'hidden',
                            textOverflow: 'ellipsis',
                            whiteSpace: 'nowrap',
                            textAlign: 'left',
                          }}
                        >
                          {perk.name}
                        </span>
                      </button>
                    );
                  })}
                </div>
              ))}
            </div>
          </div>

          {/* STAT SHARDS (3 Rows) */}
          <div style={{ marginTop: '4px' }}>
            <div style={{ fontSize: '11px', fontWeight: 700, color: '#64748b', textTransform: 'uppercase', marginBottom: '8px' }}>
              Stat Shards (Click to switch)
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {STAT_SHARD_ROWS.map((shardRow, rowIdx) => {
                const activeShard = shardIds[rowIdx];
                return (
                  <div key={rowIdx} style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '8px' }}>
                    {shardRow.map((sId) => {
                      const shardInfo = STAT_SHARDS[sId];
                      const isSelected = sId === activeShard;
                      return (
                        <button
                          key={sId}
                          onClick={() => handleSelectShard(rowIdx, sId)}
                          title={shardInfo.name}
                          style={{
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            gap: '6px',
                            padding: '6px 8px',
                            borderRadius: '6px',
                            background: isSelected ? 'rgba(0, 200, 255, 0.14)' : 'rgba(255, 255, 255, 0.02)',
                            border: isSelected ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                            cursor: 'pointer',
                            opacity: isSelected ? 1 : 0.6,
                            transition: 'all 0.15s ease',
                            minWidth: 0,
                          }}
                        >
                          {shardInfo.icon && (
                            <img src={shardInfo.icon} alt={shardInfo.name} style={{ width: '15px', height: '15px', flexShrink: 0 }} />
                          )}
                          <span style={{ fontSize: '10px', fontWeight: 700, color: isSelected ? '#fff' : '#94a3b8', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                            {shardInfo.name.includes('+9 AD/AP')
                              ? '+9 AD/AP'
                              : shardInfo.name.includes('+10% AS')
                              ? '+10% AS'
                              : shardInfo.name.includes('+8 AH')
                              ? '+8 AH'
                              : shardInfo.name.includes('+2% MS')
                              ? '+2% MS'
                              : shardInfo.name.includes('+10-180 HP')
                              ? '+10-180 HP'
                              : shardInfo.name.includes('+65 HP')
                              ? '+65 HP'
                              : shardInfo.name.includes('Tenacity')
                              ? 'Tenacity'
                              : shardInfo.name.split('(')[1]?.replace(')', '') || shardInfo.name.slice(0, 8)}
                          </span>
                        </button>
                      );
                    })}
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
