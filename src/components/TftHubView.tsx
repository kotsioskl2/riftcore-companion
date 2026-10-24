import React, { useState } from 'react';
import {
  Crown,
  Sparkles,
  Shield,
  Zap,
  TrendingUp,
  Search,
  ExternalLink,
  Info,
  Grid,
  Filter,
  Layers,
  Award,
  ChevronRight,
  Dice5,
  ShoppingBag
} from 'lucide-react';
import {
  TFT_META_COMPS,
  TFT_COMPONENTS,
  TFT_COMBINED_ITEMS,
  TFT_ROLL_ODDS,
  TFT_AUGMENTS,
  TftComp,
  TftCombinedItem
} from '../services/tftData';
import { patchService } from '../services/patchService';
import { soundService } from '../services/soundService';

export const TftHubView: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'comps' | 'items' | 'odds' | 'augments'>('comps');
  const [selectedCompId, setSelectedCompId] = useState<string>(TFT_META_COMPS[0].id);
  const [itemSearch, setItemSearch] = useState<string>('');
  const [selectedComponentFilter, setSelectedComponentFilter] = useState<string | null>(null);
  const [augmentTierFilter, setAugmentTierFilter] = useState<'All' | 'Silver' | 'Gold' | 'Prismatic'>('All');

  const selectedComp = TFT_META_COMPS.find((c) => c.id === selectedCompId) || TFT_META_COMPS[0];

  // Helper to map cost to color
  const getCostColor = (cost: number) => {
    switch (cost) {
      case 1:
        return '#94a3b8'; // Grey
      case 2:
        return '#22c55e'; // Green
      case 3:
        return '#00d2ff'; // Blue
      case 4:
        return '#c084fc'; // Purple
      case 5:
        return '#facc15'; // Gold
      default:
        return '#fff';
    }
  };

  // Filter combined items
  const filteredItems = TFT_COMBINED_ITEMS.filter((item) => {
    const matchesSearch =
      item.name.toLowerCase().includes(itemSearch.toLowerCase()) ||
      item.category.toLowerCase().includes(itemSearch.toLowerCase()) ||
      item.bestChampions.some((c) => c.toLowerCase().includes(itemSearch.toLowerCase()));

    const matchesComponent = selectedComponentFilter
      ? item.component1 === selectedComponentFilter || item.component2 === selectedComponentFilter
      : true;

    return matchesSearch && matchesComponent;
  });

  // Filter augments
  const filteredAugments = TFT_AUGMENTS.filter((aug) => {
    if (augmentTierFilter === 'All') return true;
    return aug.tier === augmentTierFilter;
  });

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
      {/* Top Banner */}
      <div
        className="glass-card"
        style={{
          padding: '18px 22px',
          background: 'linear-gradient(135deg, rgba(250, 204, 21, 0.08) 0%, rgba(0, 210, 255, 0.06) 100%)',
          border: '1px solid rgba(250, 204, 21, 0.25)',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '14px' }}>
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <div
                style={{
                  width: '32px',
                  height: '32px',
                  borderRadius: '8px',
                  background: 'rgba(250, 204, 21, 0.15)',
                  border: '1px solid rgba(250, 204, 21, 0.4)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                }}
              >
                <Crown size={18} color="#facc15" />
              </div>
              <h1 style={{ fontSize: '18px', fontWeight: 800, color: '#fff', margin: 0 }}>
                TFT Arcane Companion & Meta Hub
              </h1>
              <span
                style={{
                  fontSize: '9.5px',
                  fontWeight: 800,
                  padding: '2px 7px',
                  borderRadius: '4px',
                  background: 'rgba(250, 204, 21, 0.18)',
                  color: '#facc15',
                  border: '1px solid rgba(250, 204, 21, 0.35)',
                }}
              >
                SET 13 ACTIVE
              </span>
            </div>
            <p style={{ fontSize: '12px', color: '#94a3b8', margin: '4px 0 0 0' }}>
              Real-time meta compositions, best-in-slot item recipes, exact hex positioning, and level rolling odds.
            </p>
          </div>

          {/* Sub-Navigation Tabs */}
          <div
            style={{
              display: 'flex',
              background: 'rgba(0, 0, 0, 0.45)',
              padding: '4px',
              borderRadius: '8px',
              border: '1px solid var(--border-subtle)',
              gap: '4px',
            }}
          >
            <button
              onClick={() => {
                setActiveTab('comps');
                soundService.playTimerTick();
              }}
              style={{
                padding: '6px 14px',
                borderRadius: '6px',
                border: 'none',
                background: activeTab === 'comps' ? 'var(--blue-primary)' : 'transparent',
                color: activeTab === 'comps' ? '#000' : '#cbd5e1',
                fontSize: '11.5px',
                fontWeight: 800,
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                transition: 'all 0.15s ease',
              }}
            >
              <Crown size={13} />
              <span>Meta Comps</span>
            </button>

            <button
              onClick={() => {
                setActiveTab('items');
                soundService.playTimerTick();
              }}
              style={{
                padding: '6px 14px',
                borderRadius: '6px',
                border: 'none',
                background: activeTab === 'items' ? 'var(--blue-primary)' : 'transparent',
                color: activeTab === 'items' ? '#000' : '#cbd5e1',
                fontSize: '11.5px',
                fontWeight: 800,
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                transition: 'all 0.15s ease',
              }}
            >
              <ShoppingBag size={13} />
              <span>Item Recipes</span>
            </button>

            <button
              onClick={() => {
                setActiveTab('odds');
                soundService.playTimerTick();
              }}
              style={{
                padding: '6px 14px',
                borderRadius: '6px',
                border: 'none',
                background: activeTab === 'odds' ? 'var(--blue-primary)' : 'transparent',
                color: activeTab === 'odds' ? '#000' : '#cbd5e1',
                fontSize: '11.5px',
                fontWeight: 800,
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                transition: 'all 0.15s ease',
              }}
            >
              <Dice5 size={13} />
              <span>Rolling Odds</span>
            </button>

            <button
              onClick={() => {
                setActiveTab('augments');
                soundService.playTimerTick();
              }}
              style={{
                padding: '6px 14px',
                borderRadius: '6px',
                border: 'none',
                background: activeTab === 'augments' ? 'var(--blue-primary)' : 'transparent',
                color: activeTab === 'augments' ? '#000' : '#cbd5e1',
                fontSize: '11.5px',
                fontWeight: 800,
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                transition: 'all 0.15s ease',
              }}
            >
              <Sparkles size={13} />
              <span>Augments</span>
            </button>
          </div>
        </div>
      </div>

      {/* TAB 1: META COMPOSITIONS */}
      {activeTab === 'comps' && (
        <div style={{ display: 'grid', gridTemplateColumns: '320px 1fr', gap: '16px' }}>
          {/* Left Column: Compositions List */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
            <div style={{ fontSize: '11px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b', letterSpacing: '0.6px', paddingLeft: '4px' }}>
              Tier List S+ & S Tier Comps
            </div>

            {TFT_META_COMPS.map((comp) => {
              const isSelected = comp.id === selectedCompId;
              const isSTier = comp.tier.startsWith('S');
              return (
                <div
                  key={comp.id}
                  onClick={() => {
                    setSelectedCompId(comp.id);
                    soundService.playTimerTick();
                  }}
                  style={{
                    padding: '12px 14px',
                    borderRadius: '8px',
                    background: isSelected ? 'rgba(0, 210, 255, 0.12)' : 'rgba(255, 255, 255, 0.03)',
                    border: isSelected ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                    cursor: 'pointer',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '8px',
                    transition: 'all 0.15s ease',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <span
                        style={{
                          fontSize: '11px',
                          fontWeight: 900,
                          padding: '1px 6px',
                          borderRadius: '4px',
                          background: isSTier ? 'rgba(250, 204, 21, 0.2)' : 'rgba(0, 210, 255, 0.15)',
                          color: isSTier ? '#facc15' : 'var(--blue-primary)',
                          border: `1px solid ${isSTier ? 'rgba(250, 204, 21, 0.4)' : 'rgba(0, 210, 255, 0.3)'}`,
                        }}
                      >
                        {comp.tier}
                      </span>
                      <strong style={{ fontSize: '12.5px', color: '#fff' }}>{comp.name}</strong>
                    </div>
                    <span style={{ fontSize: '10px', color: '#94a3b8', background: 'rgba(255,255,255,0.06)', padding: '1px 5px', borderRadius: '3px' }}>
                      {comp.playstyle}
                    </span>
                  </div>

                  {/* Traits capsules */}
                  <div style={{ display: 'flex', alignItems: 'center', gap: '4px', flexWrap: 'wrap' }}>
                    {comp.traits.map((t) => (
                      <span
                        key={t.name}
                        style={{
                          fontSize: '9.5px',
                          fontWeight: 700,
                          padding: '1px 5px',
                          borderRadius: '3px',
                          background: 'rgba(255, 255, 255, 0.05)',
                          border: '1px solid rgba(255, 255, 255, 0.08)',
                          color: '#cbd5e1',
                        }}
                      >
                        {t.count} {t.name}
                      </span>
                    ))}
                  </div>

                  {/* Carry & Tank preview */}
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', fontSize: '10.5px', color: '#94a3b8', borderTop: '1px solid rgba(255,255,255,0.05)', paddingTop: '6px' }}>
                    <span>Carry: <strong style={{ color: '#00d2ff' }}>{comp.mainCarry.champion}</strong></span>
                    <span>Tank: <strong style={{ color: '#22c55e' }}>{comp.mainTank.champion}</strong></span>
                  </div>
                </div>
              );
            })}
          </div>

          {/* Right Column: Selected Composition Deep Dive */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
            <div className="glass-card" style={{ padding: '20px', border: '1px solid var(--border-blue)' }}>
              {/* Header */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '10px', marginBottom: '14px' }}>
                <div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <span
                      style={{
                        fontSize: '12px',
                        fontWeight: 900,
                        padding: '2px 8px',
                        borderRadius: '4px',
                        background: 'rgba(250, 204, 21, 0.2)',
                        color: '#facc15',
                        border: '1px solid rgba(250, 204, 21, 0.4)',
                      }}
                    >
                      {selectedComp.tier} TIER
                    </span>
                    <h2 style={{ fontSize: '17px', fontWeight: 800, color: '#fff', margin: 0 }}>
                      {selectedComp.name}
                    </h2>
                  </div>
                  <p style={{ fontSize: '12px', color: '#94a3b8', margin: '4px 0 0 0' }}>
                    {selectedComp.tagline}
                  </p>
                </div>

                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <span style={{ fontSize: '11px', color: '#64748b' }}>Playstyle:</span>
                  <span style={{ fontSize: '11px', fontWeight: 800, color: 'var(--blue-primary)', background: 'rgba(0, 210, 255, 0.1)', padding: '2px 8px', borderRadius: '4px', border: '1px solid rgba(0, 210, 255, 0.25)' }}>
                    {selectedComp.playstyle}
                  </span>
                  <span style={{ fontSize: '11px', fontWeight: 800, color: '#10b981', background: 'rgba(16, 185, 129, 0.1)', padding: '2px 8px', borderRadius: '4px', border: '1px solid rgba(16, 185, 129, 0.25)' }}>
                    {selectedComp.difficulty}
                  </span>
                </div>
              </div>

              {/* Roster Badges */}
              <div style={{ marginBottom: '16px' }}>
                <div style={{ fontSize: '10.5px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b', marginBottom: '8px' }}>
                  Full Board Champions & Cost Tier:
                </div>
                <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
                  {selectedComp.champions.map((champ) => {
                    const costColor = getCostColor(champ.cost);
                    const isMainCarry = champ.name === selectedComp.mainCarry.champion;
                    const isMainTank = champ.name === selectedComp.mainTank.champion;

                    return (
                      <div
                        key={champ.name}
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          gap: '6px',
                          padding: '5px 8px',
                          borderRadius: '6px',
                          background: 'rgba(255, 255, 255, 0.04)',
                          border: isMainCarry ? '1px solid #00d2ff' : isMainTank ? '1px solid #22c55e' : `1px solid ${costColor}40`,
                          boxShadow: isMainCarry ? '0 0 8px rgba(0, 210, 255, 0.25)' : 'none',
                        }}
                      >
                        <span
                          style={{
                            fontSize: '9.5px',
                            fontWeight: 900,
                            padding: '1px 4px',
                            borderRadius: '3px',
                            background: `${costColor}20`,
                            color: costColor,
                          }}
                        >
                          {champ.cost}g
                        </span>
                        <span style={{ fontSize: '11.5px', fontWeight: 700, color: '#fff' }}>
                          {champ.name}
                        </span>
                        {isMainCarry && <span style={{ fontSize: '9px', color: '#00d2ff', fontWeight: 800 }}>★CARRY</span>}
                        {isMainTank && <span style={{ fontSize: '9px', color: '#22c55e', fontWeight: 800 }}>★TANK</span>}
                      </div>
                    );
                  })}
                </div>
              </div>

              {/* Carry & Tank Best-in-Slot Items Grid */}
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px', marginBottom: '16px' }}>
                {/* Carry BiS */}
                <div style={{ background: 'rgba(0, 210, 255, 0.04)', border: '1px solid rgba(0, 210, 255, 0.25)', padding: '12px', borderRadius: '8px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' }}>
                    <div style={{ fontSize: '11px', fontWeight: 800, color: '#00d2ff', display: 'flex', alignItems: 'center', gap: '5px' }}>
                      <Crown size={12} color="#00d2ff" />
                      <span>Primary Carry: {selectedComp.mainCarry.champion}</span>
                    </div>
                    <span style={{ fontSize: '9px', fontWeight: 800, color: '#cbd5e1', background: 'rgba(255,255,255,0.06)', padding: '1px 5px', borderRadius: '3px' }}>
                      3x BiS Items
                    </span>
                  </div>
                  <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap' }}>
                    {selectedComp.mainCarry.items.map((itemName) => (
                      <span
                        key={itemName}
                        style={{
                          fontSize: '10.5px',
                          fontWeight: 700,
                          padding: '3px 8px',
                          borderRadius: '4px',
                          background: 'rgba(0, 210, 255, 0.15)',
                          color: '#fff',
                          border: '1px solid rgba(0, 210, 255, 0.35)',
                        }}
                      >
                        ⚔️ {itemName}
                      </span>
                    ))}
                  </div>
                </div>

                {/* Tank BiS */}
                <div style={{ background: 'rgba(16, 185, 129, 0.04)', border: '1px solid rgba(16, 185, 129, 0.25)', padding: '12px', borderRadius: '8px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' }}>
                    <div style={{ fontSize: '11px', fontWeight: 800, color: '#10b981', display: 'flex', alignItems: 'center', gap: '5px' }}>
                      <Shield size={12} color="#10b981" />
                      <span>Main Frontline: {selectedComp.mainTank.champion}</span>
                    </div>
                    <span style={{ fontSize: '9px', fontWeight: 800, color: '#cbd5e1', background: 'rgba(255,255,255,0.06)', padding: '1px 5px', borderRadius: '3px' }}>
                      Defensive Core
                    </span>
                  </div>
                  <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap' }}>
                    {selectedComp.mainTank.items.map((itemName) => (
                      <span
                        key={itemName}
                        style={{
                          fontSize: '10.5px',
                          fontWeight: 700,
                          padding: '3px 8px',
                          borderRadius: '4px',
                          background: 'rgba(16, 185, 129, 0.15)',
                          color: '#fff',
                          border: '1px solid rgba(16, 185, 129, 0.35)',
                        }}
                      >
                        🛡️ {itemName}
                      </span>
                    ))}
                  </div>
                </div>
              </div>

              {/* 4x7 Hex Board Tactical Positioning */}
              <div style={{ marginBottom: '16px' }}>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px' }}>
                  <div style={{ fontSize: '10.5px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b' }}>
                    Hex Board Positioning (Row 1 Frontline ➔ Row 4 Backline):
                  </div>
                  <span style={{ fontSize: '10px', color: '#94a3b8' }}>
                    {selectedComp.positioningTip}
                  </span>
                </div>

                {/* 4 Rows x 7 Cols Board Matrix */}
                <div
                  style={{
                    background: 'rgba(6, 9, 15, 0.95)',
                    border: '1px solid var(--border-subtle)',
                    borderRadius: '8px',
                    padding: '12px',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '6px',
                  }}
                >
                  {[0, 1, 2, 3].map((rowIdx) => (
                    <div
                      key={rowIdx}
                      style={{
                        display: 'grid',
                        gridTemplateColumns: 'repeat(7, 1fr)',
                        gap: '6px',
                        paddingLeft: rowIdx % 2 === 1 ? '16px' : '0', // Stagger hex pattern
                      }}
                    >
                      {[0, 1, 2, 3, 4, 5, 6].map((colIdx) => {
                        const occupant = selectedComp.champions.find(
                          (c) => c.position?.row === rowIdx && c.position?.col === colIdx
                        );

                        const isCarry = occupant && occupant.name === selectedComp.mainCarry.champion;
                        const isTank = occupant && occupant.name === selectedComp.mainTank.champion;

                        return (
                          <div
                            key={colIdx}
                            style={{
                              height: '42px',
                              borderRadius: '6px',
                              background: occupant
                                ? isCarry
                                  ? 'rgba(0, 210, 255, 0.2)'
                                  : isTank
                                  ? 'rgba(16, 185, 129, 0.2)'
                                  : 'rgba(255, 255, 255, 0.08)'
                                : 'rgba(255, 255, 255, 0.02)',
                              border: occupant
                                ? isCarry
                                  ? '1px solid #00d2ff'
                                  : isTank
                                  ? '1px solid #10b981'
                                  : '1px solid rgba(255, 255, 255, 0.15)'
                                : '1px dashed rgba(255, 255, 255, 0.05)',
                              display: 'flex',
                              alignItems: 'center',
                              justifyContent: 'center',
                              flexDirection: 'column',
                              padding: '2px',
                              transition: 'all 0.15s ease',
                            }}
                          >
                            {occupant ? (
                              <>
                                <span style={{ fontSize: '10px', fontWeight: 800, color: '#fff', textAlign: 'center', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', width: '100%' }}>
                                  {occupant.name}
                                </span>
                                <span style={{ fontSize: '8px', color: getCostColor(occupant.cost), fontWeight: 700 }}>
                                  {occupant.cost}g {isCarry ? '👑' : isTank ? '🛡️' : ''}
                                </span>
                              </>
                            ) : (
                              <span style={{ fontSize: '9px', color: '#334155' }}>•</span>
                            )}
                          </div>
                        );
                      })}
                    </div>
                  ))}
                </div>
              </div>

              {/* Leveling & Economy Strategy */}
              <div style={{ background: 'rgba(255, 255, 255, 0.02)', border: '1px solid var(--border-subtle)', borderRadius: '8px', padding: '12px' }}>
                <div style={{ fontSize: '11px', fontWeight: 800, color: '#facc15', marginBottom: '4px', display: 'flex', alignItems: 'center', gap: '5px' }}>
                  <TrendingUp size={12} color="#facc15" />
                  <span>Leveling Curve & Economy Strategy</span>
                </div>
                <div style={{ fontSize: '12px', color: '#cbd5e1', lineHeight: '1.4' }}>
                  {selectedComp.levelingGuide}
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* TAB 2: ITEM RECIPES & MATRIX */}
      {activeTab === 'items' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
          {/* Component Quick Filter Bar */}
          <div className="glass-card" style={{ padding: '14px' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '10px', flexWrap: 'wrap', gap: '8px' }}>
              <div style={{ fontSize: '11px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b' }}>
                Select Base Component to Filter Combined Recipes:
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                <Search size={13} color="#64748b" />
                <input
                  type="text"
                  placeholder="Search combined items or champions..."
                  value={itemSearch}
                  onChange={(e) => setItemSearch(e.target.value)}
                  style={{
                    padding: '4px 8px',
                    borderRadius: '4px',
                    background: 'rgba(0, 0, 0, 0.4)',
                    border: '1px solid var(--border-subtle)',
                    color: '#fff',
                    fontSize: '11px',
                    outline: 'none',
                    width: '220px',
                  }}
                />
              </div>
            </div>

            <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
              <button
                onClick={() => setSelectedComponentFilter(null)}
                style={{
                  padding: '5px 10px',
                  borderRadius: '6px',
                  background: selectedComponentFilter === null ? 'var(--blue-primary)' : 'rgba(255, 255, 255, 0.04)',
                  border: selectedComponentFilter === null ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                  color: selectedComponentFilter === null ? '#000' : '#fff',
                  fontSize: '11px',
                  fontWeight: 800,
                  cursor: 'pointer',
                }}
              >
                All Items ({TFT_COMBINED_ITEMS.length})
              </button>

              {TFT_COMPONENTS.map((comp) => {
                const isSelected = selectedComponentFilter === comp.id;
                return (
                  <button
                    key={comp.id}
                    onClick={() => setSelectedComponentFilter(isSelected ? null : comp.id)}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '5px',
                      padding: '5px 10px',
                      borderRadius: '6px',
                      background: isSelected ? 'rgba(0, 210, 255, 0.2)' : 'rgba(255, 255, 255, 0.03)',
                      border: isSelected ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                      color: isSelected ? 'var(--blue-primary)' : '#cbd5e1',
                      fontSize: '11px',
                      fontWeight: 700,
                      cursor: 'pointer',
                      transition: 'all 0.15s ease',
                    }}
                    title={`${comp.name}: ${comp.stats}`}
                  >
                    <span>{comp.icon}</span>
                    <span>{comp.name}</span>
                  </button>
                );
              })}
            </div>
          </div>

          {/* Combined Items Grid */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: '10px' }}>
            {filteredItems.map((item) => {
              const comp1 = TFT_COMPONENTS.find((c) => c.id === item.component1);
              const comp2 = TFT_COMPONENTS.find((c) => c.id === item.component2);

              return (
                <div
                  key={item.id}
                  className="glass-card"
                  style={{
                    padding: '12px',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '8px',
                    border: '1px solid rgba(255, 255, 255, 0.08)',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <strong style={{ fontSize: '13px', color: '#fff' }}>{item.name}</strong>
                    <span
                      style={{
                        fontSize: '9px',
                        fontWeight: 800,
                        padding: '1px 5px',
                        borderRadius: '3px',
                        background:
                          item.category === 'AD Carry'
                            ? 'rgba(239, 68, 68, 0.15)'
                            : item.category === 'AP Carry'
                            ? 'rgba(0, 210, 255, 0.15)'
                            : 'rgba(16, 185, 129, 0.15)',
                        color:
                          item.category === 'AD Carry'
                            ? '#f87171'
                            : item.category === 'AP Carry'
                            ? '#00d2ff'
                            : '#10b981',
                      }}
                    >
                      {item.category}
                    </span>
                  </div>

                  {/* Recipe Components */}
                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '10px', color: '#94a3b8' }}>
                    <span style={{ background: 'rgba(255, 255, 255, 0.05)', padding: '2px 6px', borderRadius: '4px', border: '1px solid rgba(255,255,255,0.08)' }}>
                      {comp1?.icon} {comp1?.name}
                    </span>
                    <span>+</span>
                    <span style={{ background: 'rgba(255, 255, 255, 0.05)', padding: '2px 6px', borderRadius: '4px', border: '1px solid rgba(255,255,255,0.08)' }}>
                      {comp2?.icon} {comp2?.name}
                    </span>
                  </div>

                  <div style={{ fontSize: '11px', fontWeight: 700, color: '#facc15' }}>
                    {item.stats}
                  </div>

                  <div style={{ fontSize: '11px', color: '#cbd5e1', lineHeight: '1.4' }}>
                    {item.description}
                  </div>

                  {/* Best on */}
                  <div style={{ display: 'flex', alignItems: 'center', gap: '4px', flexWrap: 'wrap', marginTop: 'auto', paddingTop: '6px', borderTop: '1px solid rgba(255,255,255,0.05)' }}>
                    <span style={{ fontSize: '9.5px', color: '#64748b' }}>Best On:</span>
                    {item.bestChampions.map((c) => (
                      <span key={c} style={{ fontSize: '9.5px', fontWeight: 700, color: '#94a3b8', background: 'rgba(255,255,255,0.04)', padding: '1px 5px', borderRadius: '3px' }}>
                        {c}
                      </span>
                    ))}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* TAB 3: LEVEL ROLLING ODDS & XP CURVE */}
      {activeTab === 'odds' && (
        <div className="glass-card" style={{ padding: '20px' }}>
          <div style={{ marginBottom: '14px' }}>
            <h3 style={{ fontSize: '15px', fontWeight: 800, color: '#fff', margin: 0, display: 'flex', alignItems: 'center', gap: '6px' }}>
              <Dice5 size={16} color="var(--blue-primary)" />
              <span>TFT Level Rolling Odds & Shop Probabilities</span>
            </h3>
            <p style={{ fontSize: '12px', color: '#94a3b8', margin: '4px 0 0 0' }}>
              Exact percentage chances of finding 1-cost through 5-cost champions in your shop for each Tactician level.
            </p>
          </div>

          <div style={{ overflowX: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '12px', textAlign: 'center' }}>
              <thead>
                <tr style={{ borderBottom: '1px solid rgba(255, 255, 255, 0.1)', color: '#64748b' }}>
                  <th style={{ padding: '10px', textAlign: 'left' }}>Level</th>
                  <th style={{ padding: '10px' }}>Target Round</th>
                  <th style={{ padding: '10px', color: '#94a3b8' }}>1-Cost</th>
                  <th style={{ padding: '10px', color: '#22c55e' }}>2-Cost</th>
                  <th style={{ padding: '10px', color: '#00d2ff' }}>3-Cost</th>
                  <th style={{ padding: '10px', color: '#c084fc' }}>4-Cost</th>
                  <th style={{ padding: '10px', color: '#facc15' }}>5-Cost</th>
                  <th style={{ padding: '10px', textAlign: 'right' }}>XP to Level</th>
                </tr>
              </thead>
              <tbody>
                {TFT_ROLL_ODDS.map((row) => (
                  <tr
                    key={row.level}
                    style={{
                      borderBottom: '1px solid rgba(255, 255, 255, 0.04)',
                      background: row.level === 8 ? 'rgba(0, 210, 255, 0.06)' : 'transparent',
                    }}
                  >
                    <td style={{ padding: '10px', textAlign: 'left', fontWeight: 800, color: '#fff' }}>
                      Level {row.level} {row.level === 8 && <span style={{ fontSize: '9px', color: '#00d2ff' }}>★POWERSPIKE</span>}
                    </td>
                    <td style={{ padding: '10px', color: '#cbd5e1' }}>{row.recommendedRound}</td>
                    <td style={{ padding: '10px', color: '#94a3b8', fontWeight: 700 }}>{row.cost1}%</td>
                    <td style={{ padding: '10px', color: '#22c55e', fontWeight: 700 }}>{row.cost2}%</td>
                    <td style={{ padding: '10px', color: '#00d2ff', fontWeight: 700 }}>{row.cost3}%</td>
                    <td style={{ padding: '10px', color: '#c084fc', fontWeight: 700 }}>{row.cost4}%</td>
                    <td style={{ padding: '10px', color: '#facc15', fontWeight: 700 }}>{row.cost5}%</td>
                    <td style={{ padding: '10px', textAlign: 'right', color: '#64748b' }}>
                      {row.xpToNext > 0 ? `${row.xpToNext} XP (${row.goldCostEstimate}g)` : 'Max'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* TAB 4: AUGMENTS TIER LIST */}
      {activeTab === 'augments' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          {/* Filter Bar */}
          <div className="glass-card" style={{ padding: '12px', display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span style={{ fontSize: '11px', fontWeight: 700, color: '#64748b' }}>Augment Tier:</span>
            {(['All', 'Silver', 'Gold', 'Prismatic'] as const).map((tier) => (
              <button
                key={tier}
                onClick={() => setAugmentTierFilter(tier)}
                style={{
                  padding: '4px 10px',
                  borderRadius: '6px',
                  background: augmentTierFilter === tier ? 'var(--blue-primary)' : 'rgba(255, 255, 255, 0.04)',
                  border: augmentTierFilter === tier ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                  color: augmentTierFilter === tier ? '#000' : '#fff',
                  fontSize: '11px',
                  fontWeight: 800,
                  cursor: 'pointer',
                }}
              >
                {tier}
              </button>
            ))}
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: '10px' }}>
            {filteredAugments.map((aug) => {
              const tierColor =
                aug.tier === 'Prismatic' ? '#facc15' : aug.tier === 'Gold' ? '#eab308' : '#94a3b8';

              return (
                <div
                  key={aug.id}
                  className="glass-card"
                  style={{
                    padding: '14px',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '8px',
                    border: `1px solid ${tierColor}30`,
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <strong style={{ fontSize: '13px', color: '#fff' }}>{aug.name}</strong>
                    <span
                      style={{
                        fontSize: '9.5px',
                        fontWeight: 900,
                        padding: '1px 6px',
                        borderRadius: '3px',
                        background: `${tierColor}20`,
                        color: tierColor,
                        border: `1px solid ${tierColor}40`,
                      }}
                    >
                      {aug.tier}
                    </span>
                  </div>

                  <div style={{ display: 'flex', alignItems: 'center', gap: '12px', fontSize: '11px' }}>
                    <span style={{ color: '#10b981', fontWeight: 800 }}>★ {aug.winRate}% Win Rate</span>
                    <span style={{ color: '#00d2ff' }}>Avg #{aug.avgPlacement}</span>
                  </div>

                  <p style={{ fontSize: '11.5px', color: '#cbd5e1', margin: 0, lineHeight: '1.4' }}>
                    {aug.description}
                  </p>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
};
