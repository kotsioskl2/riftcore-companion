import React, { useState, useMemo } from 'react';
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
  ShoppingBag,
  Copy,
  Check,
  Flame,
  Star,
  Compass,
  ArrowRight
} from 'lucide-react';
import {
  TFT_META_COMPS,
  TFT_COMPONENTS,
  TFT_COMBINED_ITEMS,
  TFT_ROLL_ODDS,
  TFT_AUGMENTS,
  TftComp,
  TftCombinedItem,
  getTftItemIcon,
  getTftChampionAvatar
} from '../services/tftData';
import { soundService } from '../services/soundService';

export const TftHubView: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'comps' | 'items' | 'odds' | 'augments'>('comps');
  const [selectedCompId, setSelectedCompId] = useState<string>(TFT_META_COMPS[0].id);
  const [compSearchQuery, setCompSearchQuery] = useState<string>('');
  const [compTierFilter, setCompTierFilter] = useState<'ALL' | 'S+' | 'S' | 'A'>('ALL');
  const [copiedCompId, setCopiedCompId] = useState<string | null>(null);

  // Items tab states
  const [itemSearch, setItemSearch] = useState<string>('');
  const [selectedComponentFilter, setSelectedComponentFilter] = useState<string | null>(null);
  const [mixerComp1, setMixerComp1] = useState<string>('sword');
  const [mixerComp2, setMixerComp2] = useState<string>('gloves');

  // Augments tab states
  const [augmentTierFilter, setAugmentTierFilter] = useState<'All' | 'Silver' | 'Gold' | 'Prismatic'>('All');
  const [augmentSearch, setAugmentSearch] = useState<string>('');

  const selectedComp = useMemo(() => {
    return TFT_META_COMPS.find((c) => c.id === selectedCompId) || TFT_META_COMPS[0];
  }, [selectedCompId]);

  // Cost to color theme
  const getCostColor = (cost: number) => {
    switch (cost) {
      case 1:
        return '#94a3b8'; // Grey / 1-cost
      case 2:
        return '#22c55e'; // Green / 2-cost
      case 3:
        return '#00d2ff'; // Hextech Cyan / 3-cost
      case 4:
        return '#c084fc'; // Epic Purple / 4-cost
      case 5:
        return '#facc15'; // Radiant Gold / 5-cost
      default:
        return '#fff';
    }
  };

  const getTierColor = (tier: string) => {
    if (tier === 'S+') return '#facc15';
    if (tier === 'S') return '#00d2ff';
    if (tier === 'A') return '#c084fc';
    return '#94a3b8';
  };

  // Filter comps by tier and search query (matches name, traits, or champion names)
  const filteredComps = useMemo(() => {
    return TFT_META_COMPS.filter((comp) => {
      const matchesTier = compTierFilter === 'ALL' || comp.tier === compTierFilter;
      const q = compSearchQuery.trim().toLowerCase();
      if (!q) return matchesTier;

      const matchesName = comp.name.toLowerCase().includes(q);
      const matchesTraits = comp.traits.some((t) => t.name.toLowerCase().includes(q));
      const matchesChamps = comp.champions.some((c) => c.name.toLowerCase().includes(q));
      return matchesTier && (matchesName || matchesTraits || matchesChamps);
    });
  }, [compTierFilter, compSearchQuery]);

  // Filter combined items
  const filteredItems = useMemo(() => {
    return TFT_COMBINED_ITEMS.filter((item) => {
      const matchesSearch =
        item.name.toLowerCase().includes(itemSearch.toLowerCase()) ||
        item.category.toLowerCase().includes(itemSearch.toLowerCase()) ||
        item.bestChampions.some((c) => c.toLowerCase().includes(itemSearch.toLowerCase()));

      const matchesComponent = selectedComponentFilter
        ? item.component1 === selectedComponentFilter || item.component2 === selectedComponentFilter
        : true;

      return matchesSearch && matchesComponent;
    });
  }, [itemSearch, selectedComponentFilter]);

  // Interactive component mixer result
  const mixedItem = useMemo(() => {
    return (
      TFT_COMBINED_ITEMS.find(
        (it) =>
          (it.component1 === mixerComp1 && it.component2 === mixerComp2) ||
          (it.component1 === mixerComp2 && it.component2 === mixerComp1)
      ) || null
    );
  }, [mixerComp1, mixerComp2]);

  // Filter augments
  const filteredAugments = useMemo(() => {
    return TFT_AUGMENTS.filter((aug) => {
      const matchesTier = augmentTierFilter === 'All' || aug.tier === augmentTierFilter;
      const q = augmentSearch.trim().toLowerCase();
      if (!q) return matchesTier;
      return matchesTier && (aug.name.toLowerCase().includes(q) || aug.description.toLowerCase().includes(q));
    });
  }, [augmentTierFilter, augmentSearch]);

  const handleCopyComp = (comp: TftComp) => {
    const text = `[RiftCore TFT Set 13] ${comp.name} (${comp.tier} Tier, ${comp.playstyle}) | Carry: ${comp.mainCarry.champion} (${comp.mainCarry.items.join(', ')}) | Tank: ${comp.mainTank.champion} (${comp.mainTank.items.join(', ')})`;
    navigator.clipboard.writeText(text);
    setCopiedCompId(comp.id);
    soundService.playAutoAccept();
    setTimeout(() => setCopiedCompId(null), 2500);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', minWidth: 0 }}>
      {/* Top Banner Header */}
      <div
        className="glass-card"
        style={{
          padding: '16px 20px',
          background: 'linear-gradient(135deg, rgba(250, 204, 21, 0.08) 0%, rgba(0, 210, 255, 0.05) 100%)',
          border: '1px solid rgba(250, 204, 21, 0.25)',
          position: 'relative',
          overflow: 'hidden',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '14px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
            <div
              style={{
                width: '40px',
                height: '40px',
                borderRadius: '10px',
                background: 'linear-gradient(135deg, rgba(250, 204, 21, 0.25) 0%, rgba(229, 184, 105, 0.1) 100%)',
                border: '1.5px solid rgba(250, 204, 21, 0.45)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                boxShadow: '0 0 16px rgba(250, 204, 21, 0.25)',
              }}
            >
              <Crown size={22} color="#facc15" />
            </div>
            <div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flexWrap: 'wrap' }}>
                <h1 style={{ fontSize: '18px', fontWeight: 900, color: '#fff', margin: 0, letterSpacing: '0.3px' }}>
                  TFT Set 13 Meta Hub & Tactical Companion
                </h1>
                <span
                  style={{
                    fontSize: '10px',
                    fontWeight: 900,
                    padding: '2px 7px',
                    borderRadius: '4px',
                    background: 'rgba(250, 204, 21, 0.2)',
                    color: '#facc15',
                    border: '1px solid rgba(250, 204, 21, 0.4)',
                    letterSpacing: '0.4px',
                  }}
                >
                  INTO THE ARCANE • SET 13
                </span>
                <span
                  style={{
                    fontSize: '10px',
                    fontWeight: 800,
                    padding: '2px 7px',
                    borderRadius: '4px',
                    background: 'rgba(0, 210, 255, 0.12)',
                    color: 'var(--blue-primary)',
                    border: '1px solid var(--border-blue)',
                  }}
                >
                  PATCH 14.23 LIVE
                </span>
              </div>
              <p style={{ fontSize: '12px', color: '#94a3b8', margin: '3px 0 0 0' }}>
                Verified S-Tier comps, authentic 4×7 hex board positioning, component crafting station, and shop rolling odds.
              </p>
            </div>
          </div>

          {/* Top Navigation Tabs */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '4px',
              background: 'rgba(0, 0, 0, 0.45)',
              padding: '4px',
              borderRadius: '8px',
              border: '1px solid var(--border-subtle)',
            }}
          >
            {[
              { id: 'comps', label: 'Meta Comps', icon: Crown },
              { id: 'items', label: 'Item Recipes', icon: ShoppingBag },
              { id: 'odds', label: 'Rolling Odds', icon: Dice5 },
              { id: 'augments', label: 'Augments', icon: Sparkles },
            ].map((tab) => {
              const Icon = tab.icon;
              const isActive = activeTab === tab.id;
              return (
                <button
                  key={tab.id}
                  onClick={() => {
                    setActiveTab(tab.id as any);
                    soundService.playTimerTick();
                  }}
                  style={{
                    padding: '7px 14px',
                    borderRadius: '6px',
                    border: 'none',
                    background: isActive ? 'var(--blue-primary)' : 'transparent',
                    color: isActive ? '#050811' : '#cbd5e1',
                    fontSize: '12px',
                    fontWeight: 800,
                    cursor: 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '6px',
                    boxShadow: isActive ? '0 0 14px var(--blue-glow)' : 'none',
                    transition: 'all 0.15s ease',
                  }}
                >
                  <Icon size={14} color={isActive ? '#050811' : 'var(--blue-primary)'} />
                  <span>{tab.label}</span>
                </button>
              );
            })}
          </div>
        </div>
      </div>

      {/* ========================================================= */}
      {/* TAB 1: META COMPOSITIONS DEEP DIVE                        */}
      {/* ========================================================= */}
      {activeTab === 'comps' && (
        <div style={{ display: 'grid', gridTemplateColumns: '360px 1fr', gap: '16px', minWidth: 0 }}>
          {/* Left Column: Compositions List */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '10px', minWidth: 0 }}>
            {/* Filter and Search Bar */}
            <div className="glass-card" style={{ padding: '10px 12px', display: 'flex', flexDirection: 'column', gap: '8px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                <Search size={14} color="#64748b" />
                <input
                  type="text"
                  placeholder="Filter comps, traits, or carries..."
                  value={compSearchQuery}
                  onChange={(e) => setCompSearchQuery(e.target.value)}
                  style={{
                    background: 'transparent',
                    border: 'none',
                    outline: 'none',
                    color: '#fff',
                    fontSize: '11.5px',
                    width: '100%',
                  }}
                />
              </div>
              <div style={{ display: 'flex', gap: '4px', alignItems: 'center' }}>
                {(['ALL', 'S+', 'S', 'A'] as const).map((tier) => (
                  <button
                    key={tier}
                    onClick={() => setCompTierFilter(tier)}
                    style={{
                      flex: 1,
                      padding: '3px 0',
                      borderRadius: '4px',
                      fontSize: '10px',
                      fontWeight: 800,
                      cursor: 'pointer',
                      background: compTierFilter === tier ? 'rgba(0, 210, 255, 0.2)' : 'rgba(255, 255, 255, 0.04)',
                      border: compTierFilter === tier ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                      color: compTierFilter === tier ? 'var(--blue-primary)' : '#94a3b8',
                    }}
                  >
                    {tier}
                  </button>
                ))}
              </div>
            </div>

            {/* List of Comps */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', maxHeight: 'calc(100vh - 240px)', overflowY: 'auto', paddingRight: '2px' }}>
              {filteredComps.map((comp) => {
                const isSelected = comp.id === selectedCompId;
                const tierColor = getTierColor(comp.tier);

                return (
                  <div
                    key={comp.id}
                    onClick={() => {
                      setSelectedCompId(comp.id);
                      soundService.playTimerTick();
                    }}
                    style={{
                      padding: '12px 14px',
                      borderRadius: '10px',
                      background: isSelected ? 'linear-gradient(135deg, rgba(0, 210, 255, 0.14) 0%, rgba(10, 16, 28, 0.85) 100%)' : 'rgba(10, 16, 28, 0.65)',
                      border: isSelected ? '1.5px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                      cursor: 'pointer',
                      display: 'flex',
                      flexDirection: 'column',
                      gap: '8px',
                      boxShadow: isSelected ? '0 0 16px var(--blue-glow-subtle)' : 'none',
                      transition: 'all 0.15s ease',
                      position: 'relative',
                    }}
                  >
                    {/* Top Row: Tier, Name, Playstyle */}
                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '8px' }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '8px', minWidth: 0 }}>
                        <span
                          style={{
                            fontSize: '11px',
                            fontWeight: 900,
                            padding: '2px 7px',
                            borderRadius: '4px',
                            background: `${tierColor}25`,
                            color: tierColor,
                            border: `1px solid ${tierColor}50`,
                            flexShrink: 0,
                          }}
                        >
                          {comp.tier}
                        </span>
                        <strong style={{ fontSize: '13px', color: '#fff', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                          {comp.name}
                        </strong>
                      </div>
                      <span style={{ fontSize: '10px', fontWeight: 700, color: '#94a3b8', background: 'rgba(255,255,255,0.06)', padding: '1px 6px', borderRadius: '4px', flexShrink: 0 }}>
                        {comp.playstyle}
                      </span>
                    </div>

                    {/* Stats Strip */}
                    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', fontSize: '10.5px' }}>
                      <span style={{ color: '#10b981', fontWeight: 800 }}>★ {comp.winRate || 54}% WR</span>
                      <span style={{ color: '#00d2ff', fontWeight: 700 }}>• {comp.top4Rate || 78}% Top 4</span>
                      <span style={{ color: '#eab308', fontWeight: 700 }}>• Avg #{comp.avgPlacement || 3.4}</span>
                    </div>

                    {/* Champions Avatar Thumbnail Strip */}
                    <div style={{ display: 'flex', alignItems: 'center', gap: '4px', overflowX: 'hidden' }}>
                      {comp.champions.slice(0, 8).map((champ) => {
                        const isCarry = champ.name === comp.mainCarry.champion;
                        const isTank = champ.name === comp.mainTank.champion;
                        const costColor = getCostColor(champ.cost);
                        const avatarUrl = getTftChampionAvatar(champ.name);

                        return (
                          <div
                            key={champ.name}
                            style={{
                              position: 'relative',
                              width: '28px',
                              height: '28px',
                              borderRadius: '5px',
                              border: `1.5px solid ${isCarry ? '#00d2ff' : isTank ? '#22c55e' : costColor}`,
                              overflow: 'hidden',
                              background: '#050811',
                              flexShrink: 0,
                            }}
                            title={`${champ.name} (${champ.cost}g)`}
                          >
                            <img
                              src={avatarUrl}
                              alt={champ.name}
                              style={{ width: '100%', height: '100%', objectFit: 'cover' }}
                              onError={(e) => {
                                (e.currentTarget as HTMLImageElement).style.display = 'none';
                              }}
                            />
                            {/* Fallback initials if image missing */}
                            <span
                              style={{
                                position: 'absolute',
                                inset: 0,
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'center',
                                fontSize: '9px',
                                fontWeight: 900,
                                color: costColor,
                                zIndex: 0,
                              }}
                            >
                              {champ.name.slice(0, 2)}
                            </span>
                            {isCarry && (
                              <div style={{ position: 'absolute', top: 0, right: 0, background: '#00d2ff', borderRadius: '0 0 0 4px', padding: '0 1px' }}>
                                <Crown size={7} color="#050811" />
                              </div>
                            )}
                            {isTank && (
                              <div style={{ position: 'absolute', top: 0, right: 0, background: '#22c55e', borderRadius: '0 0 0 4px', padding: '0 1px' }}>
                                <Shield size={7} color="#050811" />
                              </div>
                            )}
                          </div>
                        );
                      })}
                    </div>

                    {/* Trait Pills */}
                    <div style={{ display: 'flex', alignItems: 'center', gap: '4px', flexWrap: 'wrap' }}>
                      {comp.traits.map((t) => (
                        <span
                          key={t.name}
                          style={{
                            fontSize: '9.5px',
                            fontWeight: 700,
                            padding: '1px 5px',
                            borderRadius: '3px',
                            background: t.activeTier >= 3 ? 'rgba(250, 204, 21, 0.15)' : 'rgba(255, 255, 255, 0.05)',
                            border: t.activeTier >= 3 ? '1px solid rgba(250, 204, 21, 0.35)' : '1px solid rgba(255, 255, 255, 0.08)',
                            color: t.activeTier >= 3 ? '#facc15' : '#cbd5e1',
                          }}
                        >
                          {t.count} {t.name}
                        </span>
                      ))}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Right Column: Selected Composition Deep Dive */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '14px', minWidth: 0 }}>
            {/* Hero Comp Header */}
            <div className="glass-card" style={{ padding: '18px 22px', border: '1px solid var(--border-blue)' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '12px', marginBottom: '14px' }}>
                <div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <span
                      style={{
                        fontSize: '13px',
                        fontWeight: 900,
                        padding: '2px 8px',
                        borderRadius: '4px',
                        background: `${getTierColor(selectedComp.tier)}20`,
                        color: getTierColor(selectedComp.tier),
                        border: `1px solid ${getTierColor(selectedComp.tier)}40`,
                      }}
                    >
                      {selectedComp.tier} TIER
                    </span>
                    <h2 style={{ fontSize: '19px', fontWeight: 900, color: '#fff', margin: 0, letterSpacing: '0.2px' }}>
                      {selectedComp.name}
                    </h2>
                  </div>
                  <p style={{ fontSize: '12.5px', color: '#94a3b8', margin: '4px 0 0 0' }}>
                    {selectedComp.tagline}
                  </p>
                </div>

                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <button
                    onClick={() => handleCopyComp(selectedComp)}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '5px',
                      padding: '6px 12px',
                      borderRadius: '6px',
                      background: 'rgba(255, 255, 255, 0.06)',
                      border: '1px solid var(--border-subtle)',
                      color: copiedCompId === selectedComp.id ? '#10b981' : '#cbd5e1',
                      fontSize: '11px',
                      fontWeight: 800,
                      cursor: 'pointer',
                      transition: 'all 0.15s ease',
                    }}
                    title="Copy full comp summary to clipboard"
                  >
                    {copiedCompId === selectedComp.id ? <Check size={12} /> : <Copy size={12} />}
                    <span>{copiedCompId === selectedComp.id ? 'Copied!' : 'Copy Comp'}</span>
                  </button>

                  <span style={{ fontSize: '11px', fontWeight: 800, color: 'var(--blue-primary)', background: 'rgba(0, 210, 255, 0.12)', padding: '5px 10px', borderRadius: '6px', border: '1px solid var(--border-blue)' }}>
                    {selectedComp.playstyle}
                  </span>
                  <span style={{ fontSize: '11px', fontWeight: 800, color: '#10b981', background: 'rgba(16, 185, 129, 0.12)', padding: '5px 10px', borderRadius: '6px', border: '1px solid rgba(16, 185, 129, 0.3)' }}>
                    {selectedComp.difficulty}
                  </span>
                </div>
              </div>

              {/* Meta Stats Bar */}
              <div
                style={{
                  display: 'grid',
                  gridTemplateColumns: 'repeat(4, 1fr)',
                  gap: '8px',
                  padding: '10px 14px',
                  borderRadius: '8px',
                  background: 'rgba(0, 0, 0, 0.3)',
                  border: '1px solid var(--border-subtle)',
                }}
              >
                <div>
                  <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700, textTransform: 'uppercase' }}>Win Rate</div>
                  <div style={{ fontSize: '15px', fontWeight: 900, color: '#10b981' }}>{selectedComp.winRate || 55.4}%</div>
                </div>
                <div>
                  <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700, textTransform: 'uppercase' }}>Top 4 Rate</div>
                  <div style={{ fontSize: '15px', fontWeight: 900, color: '#00d2ff' }}>{selectedComp.top4Rate || 79.1}%</div>
                </div>
                <div>
                  <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700, textTransform: 'uppercase' }}>Avg Placement</div>
                  <div style={{ fontSize: '15px', fontWeight: 900, color: '#facc15' }}>#{selectedComp.avgPlacement || 3.38}</div>
                </div>
                <div>
                  <div style={{ fontSize: '10px', color: '#64748b', fontWeight: 700, textTransform: 'uppercase' }}>Optimal Roll</div>
                  <div style={{ fontSize: '14px', fontWeight: 800, color: '#fff' }}>Round 4-2 (Lvl 8)</div>
                </div>
              </div>
            </div>

            {/* Carries & Best-in-Slot (BiS) Showcase */}
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '14px' }}>
              {/* Primary Carry Card */}
              <div
                className="glass-card"
                style={{
                  padding: '14px 16px',
                  border: '1.5px solid rgba(0, 210, 255, 0.35)',
                  background: 'linear-gradient(135deg, rgba(0, 210, 255, 0.08) 0%, rgba(10, 16, 28, 0.8) 100%)',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '10px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <Crown size={15} color="#00d2ff" />
                    <span style={{ fontSize: '13px', fontWeight: 900, color: '#00d2ff' }}>Primary Carry</span>
                  </div>
                  <span style={{ fontSize: '10px', fontWeight: 800, padding: '2px 7px', borderRadius: '4px', background: 'rgba(0, 210, 255, 0.15)', color: '#00d2ff', border: '1px solid rgba(0, 210, 255, 0.3)' }}>
                    3x BiS Required
                  </span>
                </div>

                <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '12px' }}>
                  <div
                    style={{
                      width: '52px',
                      height: '52px',
                      borderRadius: '8px',
                      border: '2px solid #00d2ff',
                      overflow: 'hidden',
                      boxShadow: '0 0 16px rgba(0, 210, 255, 0.3)',
                      flexShrink: 0,
                    }}
                  >
                    <img
                      src={getTftChampionAvatar(selectedComp.mainCarry.champion)}
                      alt={selectedComp.mainCarry.champion}
                      style={{ width: '100%', height: '100%', objectFit: 'cover' }}
                    />
                  </div>
                  <div>
                    <div style={{ fontSize: '15px', fontWeight: 900, color: '#fff' }}>
                      {selectedComp.mainCarry.champion}
                    </div>
                    <div style={{ display: 'flex', gap: '2px', marginTop: '2px' }}>
                      <Star size={11} color="#facc15" fill="#facc15" />
                      <Star size={11} color="#facc15" fill="#facc15" />
                      <Star size={11} color="#64748b" />
                      <span style={{ fontSize: '10px', color: '#94a3b8', marginLeft: '4px' }}>Target: 2-Star</span>
                    </div>
                  </div>
                </div>

                <div style={{ fontSize: '10.5px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b', marginBottom: '6px' }}>
                  Core BiS Items:
                </div>
                <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
                  {selectedComp.mainCarry.items.map((it) => (
                    <div
                      key={it}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '6px',
                        padding: '4px 8px',
                        borderRadius: '6px',
                        background: 'rgba(0, 0, 0, 0.5)',
                        border: '1px solid rgba(0, 210, 255, 0.3)',
                      }}
                    >
                      <img src={getTftItemIcon(it)} alt={it} style={{ width: '22px', height: '22px', borderRadius: '4px' }} />
                      <span style={{ fontSize: '11px', fontWeight: 700, color: '#fff' }}>{it}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Main Frontline Tank Card */}
              <div
                className="glass-card"
                style={{
                  padding: '14px 16px',
                  border: '1.5px solid rgba(34, 197, 94, 0.35)',
                  background: 'linear-gradient(135deg, rgba(34, 197, 94, 0.08) 0%, rgba(10, 16, 28, 0.8) 100%)',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '10px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <Shield size={15} color="#22c55e" />
                    <span style={{ fontSize: '13px', fontWeight: 900, color: '#22c55e' }}>Main Frontline Tank</span>
                  </div>
                  <span style={{ fontSize: '10px', fontWeight: 800, padding: '2px 7px', borderRadius: '4px', background: 'rgba(34, 197, 94, 0.15)', color: '#22c55e', border: '1px solid rgba(34, 197, 94, 0.3)' }}>
                    Impenetrable Wall
                  </span>
                </div>

                <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '12px' }}>
                  <div
                    style={{
                      width: '52px',
                      height: '52px',
                      borderRadius: '8px',
                      border: '2px solid #22c55e',
                      overflow: 'hidden',
                      boxShadow: '0 0 16px rgba(34, 197, 94, 0.3)',
                      flexShrink: 0,
                    }}
                  >
                    <img
                      src={getTftChampionAvatar(selectedComp.mainTank.champion)}
                      alt={selectedComp.mainTank.champion}
                      style={{ width: '100%', height: '100%', objectFit: 'cover' }}
                    />
                  </div>
                  <div>
                    <div style={{ fontSize: '15px', fontWeight: 900, color: '#fff' }}>
                      {selectedComp.mainTank.champion}
                    </div>
                    <div style={{ display: 'flex', gap: '2px', marginTop: '2px' }}>
                      <Star size={11} color="#facc15" fill="#facc15" />
                      <Star size={11} color="#facc15" fill="#facc15" />
                      <Star size={11} color="#64748b" />
                      <span style={{ fontSize: '10px', color: '#94a3b8', marginLeft: '4px' }}>Target: 2-Star</span>
                    </div>
                  </div>
                </div>

                <div style={{ fontSize: '10.5px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b', marginBottom: '6px' }}>
                  Defensive Core Items:
                </div>
                <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
                  {selectedComp.mainTank.items.map((it) => (
                    <div
                      key={it}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '6px',
                        padding: '4px 8px',
                        borderRadius: '6px',
                        background: 'rgba(0, 0, 0, 0.5)',
                        border: '1px solid rgba(34, 197, 94, 0.3)',
                      }}
                    >
                      <img src={getTftItemIcon(it)} alt={it} style={{ width: '22px', height: '22px', borderRadius: '4px' }} />
                      <span style={{ fontSize: '11px', fontWeight: 700, color: '#fff' }}>{it}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            {/* Authentic 4x7 TFT Hex Board */}
            <div className="glass-card" style={{ padding: '16px 20px', border: '1px solid var(--border-blue)' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px', flexWrap: 'wrap', gap: '8px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <Grid size={16} color="var(--blue-primary)" />
                  <span style={{ fontSize: '13px', fontWeight: 900, color: '#fff' }}>
                    Authentic 4×7 Hex Board Positioning
                  </span>
                  <span style={{ fontSize: '10px', color: '#64748b' }}>
                    (Row 1: Frontline ➔ Row 4: Backline)
                  </span>
                </div>
                <span style={{ fontSize: '11px', color: 'var(--blue-primary)', background: 'rgba(0, 210, 255, 0.08)', padding: '3px 9px', borderRadius: '4px', border: '1px solid var(--border-blue)' }}>
                  💡 {selectedComp.positioningTip}
                </span>
              </div>

              {/* Staggered Hex Arena Board */}
              <div
                style={{
                  background: 'radial-gradient(ellipse at center, rgba(15, 23, 42, 0.95) 0%, rgba(5, 8, 17, 0.98) 100%)',
                  border: '1px solid rgba(255, 255, 255, 0.08)',
                  borderRadius: '12px',
                  padding: '16px 20px',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '8px',
                  boxShadow: 'inset 0 0 30px rgba(0, 0, 0, 0.8)',
                }}
              >
                {[0, 1, 2, 3].map((rowIdx) => (
                  <div
                    key={rowIdx}
                    style={{
                      display: 'grid',
                      gridTemplateColumns: 'repeat(7, 1fr)',
                      gap: '8px',
                      paddingLeft: rowIdx % 2 === 1 ? '24px' : '0', // Authentic staggered TFT board pattern
                      paddingRight: rowIdx % 2 === 0 ? '24px' : '0',
                    }}
                  >
                    {[0, 1, 2, 3, 4, 5, 6].map((colIdx) => {
                      const occupant = selectedComp.champions.find(
                        (c) => c.position?.row === rowIdx && c.position?.col === colIdx
                      );

                      const isCarry = occupant && occupant.name === selectedComp.mainCarry.champion;
                      const isTank = occupant && occupant.name === selectedComp.mainTank.champion;
                      const costColor = occupant ? getCostColor(occupant.cost) : '#334155';
                      const avatarUrl = occupant ? getTftChampionAvatar(occupant.name) : '';

                      return (
                        <div
                          key={colIdx}
                          style={{
                            height: '62px',
                            borderRadius: '8px',
                            background: occupant
                              ? isCarry
                                ? 'rgba(0, 210, 255, 0.18)'
                                : isTank
                                ? 'rgba(34, 197, 94, 0.18)'
                                : 'rgba(255, 255, 255, 0.06)'
                              : 'rgba(255, 255, 255, 0.02)',
                            border: occupant
                              ? isCarry
                                ? '2px solid #00d2ff'
                                : isTank
                                ? '2px solid #22c55e'
                                : `1.5px solid ${costColor}`
                              : '1px dashed rgba(255, 255, 255, 0.08)',
                            boxShadow: occupant
                              ? isCarry
                                ? '0 0 16px rgba(0, 210, 255, 0.4)'
                                : isTank
                                ? '0 0 16px rgba(34, 197, 94, 0.4)'
                                : `0 0 8px ${costColor}25`
                              : 'none',
                            position: 'relative',
                            overflow: 'hidden',
                            display: 'flex',
                            flexDirection: 'column',
                            alignItems: 'center',
                            justifyContent: 'space-between',
                            padding: '3px',
                            transition: 'all 0.15s ease',
                          }}
                          title={occupant ? `${occupant.name} (${occupant.cost}g)` : `Hex [R${rowIdx + 1}, C${colIdx + 1}]`}
                        >
                          {occupant ? (
                            <>
                              {/* Background Avatar */}
                              <img
                                src={avatarUrl}
                                alt={occupant.name}
                                style={{
                                  position: 'absolute',
                                  inset: 0,
                                  width: '100%',
                                  height: '100%',
                                  objectFit: 'cover',
                                  opacity: 0.65,
                                  zIndex: 0,
                                }}
                                onError={(e) => {
                                  (e.currentTarget as HTMLImageElement).style.display = 'none';
                                }}
                              />
                              <div
                                style={{
                                  position: 'absolute',
                                  inset: 0,
                                  background: 'linear-gradient(180deg, rgba(0,0,0,0.1) 0%, rgba(5,8,17,0.92) 100%)',
                                  zIndex: 1,
                                }}
                              />

                              {/* Top Bar: Cost & Role Badge */}
                              <div style={{ width: '100%', display: 'flex', justifyContent: 'space-between', alignItems: 'center', zIndex: 2 }}>
                                <span
                                  style={{
                                    fontSize: '8.5px',
                                    fontWeight: 900,
                                    padding: '0 4px',
                                    borderRadius: '3px',
                                    background: costColor,
                                    color: '#050811',
                                  }}
                                >
                                  {occupant.cost}g
                                </span>
                                {isCarry && (
                                  <span style={{ fontSize: '8px', fontWeight: 900, background: '#00d2ff', color: '#000', padding: '0 3px', borderRadius: '3px' }}>
                                    CARRY
                                  </span>
                                )}
                                {isTank && (
                                  <span style={{ fontSize: '8px', fontWeight: 900, background: '#22c55e', color: '#000', padding: '0 3px', borderRadius: '3px' }}>
                                    TANK
                                  </span>
                                )}
                              </div>

                              {/* Champion Name */}
                              <span
                                style={{
                                  fontSize: '10.5px',
                                  fontWeight: 800,
                                  color: '#fff',
                                  zIndex: 2,
                                  textAlign: 'center',
                                  textShadow: '0 1px 3px rgba(0,0,0,0.9)',
                                  whiteSpace: 'nowrap',
                                  overflow: 'hidden',
                                  textOverflow: 'ellipsis',
                                  width: '100%',
                                }}
                              >
                                {occupant.name}
                              </span>

                              {/* Bottom Mini Items (for carry/tank) */}
                              <div style={{ display: 'flex', gap: '2px', zIndex: 2, minHeight: '12px' }}>
                                {isCarry &&
                                  selectedComp.mainCarry.items.slice(0, 3).map((it) => (
                                    <img
                                      key={it}
                                      src={getTftItemIcon(it)}
                                      alt={it}
                                      style={{ width: '12px', height: '12px', borderRadius: '2px', border: '0.5px solid #00d2ff' }}
                                    />
                                  ))}
                                {isTank &&
                                  selectedComp.mainTank.items.slice(0, 3).map((it) => (
                                    <img
                                      key={it}
                                      src={getTftItemIcon(it)}
                                      alt={it}
                                      style={{ width: '12px', height: '12px', borderRadius: '2px', border: '0.5px solid #22c55e' }}
                                    />
                                  ))}
                              </div>
                            </>
                          ) : (
                            <span style={{ fontSize: '10px', color: '#334155', margin: 'auto' }}>•</span>
                          )}
                        </div>
                      );
                    })}
                  </div>
                ))}
              </div>
            </div>

            {/* Stage-by-Stage Leveling & Transition Timeline */}
            <div className="glass-card" style={{ padding: '16px 20px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px', marginBottom: '12px' }}>
                <TrendingUp size={16} color="#facc15" />
                <span style={{ fontSize: '13px', fontWeight: 900, color: '#fff' }}>
                  Stage-by-Stage Leveling Curve & Transition Plan
                </span>
              </div>

              {selectedComp.stageTransitions && (
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '10px', marginBottom: '12px' }}>
                  {selectedComp.stageTransitions.map((step) => (
                    <div
                      key={step.stage}
                      style={{
                        padding: '10px 12px',
                        borderRadius: '8px',
                        background: 'rgba(255, 255, 255, 0.03)',
                        border: '1px solid var(--border-subtle)',
                      }}
                    >
                      <div style={{ fontSize: '11px', fontWeight: 800, color: 'var(--blue-primary)', marginBottom: '4px' }}>
                        {step.stage}
                      </div>
                      <p style={{ fontSize: '11px', color: '#cbd5e1', margin: '0 0 6px 0', lineHeight: '1.4' }}>
                        {step.action}
                      </p>
                      <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                        {step.units.map((u) => (
                          <span
                            key={u}
                            style={{
                              fontSize: '9.5px',
                              fontWeight: 700,
                              padding: '1px 5px',
                              borderRadius: '3px',
                              background: 'rgba(255, 255, 255, 0.05)',
                              color: '#94a3b8',
                            }}
                          >
                            {u}
                          </span>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              )}

              <div style={{ fontSize: '12px', color: '#94a3b8', lineHeight: '1.5', background: 'rgba(0, 0, 0, 0.25)', padding: '8px 12px', borderRadius: '6px' }}>
                <strong style={{ color: '#fff' }}>Economy Tip: </strong>
                {selectedComp.levelingGuide}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ========================================================= */}
      {/* TAB 2: INTERACTIVE ITEM CRAFTING STATION & CHEAT SHEET    */}
      {/* ========================================================= */}
      {activeTab === 'items' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {/* Interactive 2-Component Crafter Mixer */}
          <div
            className="glass-card"
            style={{
              padding: '16px 20px',
              border: '1.5px solid var(--border-blue)',
              background: 'linear-gradient(135deg, rgba(0, 210, 255, 0.08) 0%, rgba(10, 16, 28, 0.9) 100%)',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '12px' }}>
              <Sparkles size={16} color="var(--blue-primary)" />
              <span style={{ fontSize: '14px', fontWeight: 900, color: '#fff' }}>
                Interactive Item Crafting Station (Click Any 2 Base Components)
              </span>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr auto 1fr auto 1.5fr', gap: '14px', alignItems: 'center' }}>
              {/* Component 1 Selector */}
              <div>
                <div style={{ fontSize: '10.5px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b', marginBottom: '6px' }}>
                  Component 1:
                </div>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(5, 1fr)', gap: '6px' }}>
                  {TFT_COMPONENTS.slice(0, 5).map((comp) => {
                    const isSelected = mixerComp1 === comp.id;
                    return (
                      <button
                        key={comp.id}
                        onClick={() => {
                          setMixerComp1(comp.id);
                          soundService.playTimerTick();
                        }}
                        style={{
                          padding: '6px',
                          borderRadius: '6px',
                          background: isSelected ? 'rgba(0, 210, 255, 0.25)' : 'rgba(255, 255, 255, 0.04)',
                          border: isSelected ? '1.5px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                          cursor: 'pointer',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                        }}
                        title={comp.name}
                      >
                        <img src={getTftItemIcon(comp.id)} alt={comp.name} style={{ width: '26px', height: '26px', borderRadius: '4px' }} />
                      </button>
                    );
                  })}
                </div>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '6px', marginTop: '6px' }}>
                  {TFT_COMPONENTS.slice(5).map((comp) => {
                    const isSelected = mixerComp1 === comp.id;
                    return (
                      <button
                        key={comp.id}
                        onClick={() => {
                          setMixerComp1(comp.id);
                          soundService.playTimerTick();
                        }}
                        style={{
                          padding: '6px',
                          borderRadius: '6px',
                          background: isSelected ? 'rgba(0, 210, 255, 0.25)' : 'rgba(255, 255, 255, 0.04)',
                          border: isSelected ? '1.5px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                          cursor: 'pointer',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                        }}
                        title={comp.name}
                      >
                        <img src={getTftItemIcon(comp.id)} alt={comp.name} style={{ width: '26px', height: '26px', borderRadius: '4px' }} />
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* Plus Sign */}
              <div style={{ fontSize: '20px', fontWeight: 900, color: 'var(--blue-primary)' }}>+</div>

              {/* Component 2 Selector */}
              <div>
                <div style={{ fontSize: '10.5px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b', marginBottom: '6px' }}>
                  Component 2:
                </div>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(5, 1fr)', gap: '6px' }}>
                  {TFT_COMPONENTS.slice(0, 5).map((comp) => {
                    const isSelected = mixerComp2 === comp.id;
                    return (
                      <button
                        key={comp.id}
                        onClick={() => {
                          setMixerComp2(comp.id);
                          soundService.playTimerTick();
                        }}
                        style={{
                          padding: '6px',
                          borderRadius: '6px',
                          background: isSelected ? 'rgba(0, 210, 255, 0.25)' : 'rgba(255, 255, 255, 0.04)',
                          border: isSelected ? '1.5px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                          cursor: 'pointer',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                        }}
                        title={comp.name}
                      >
                        <img src={getTftItemIcon(comp.id)} alt={comp.name} style={{ width: '26px', height: '26px', borderRadius: '4px' }} />
                      </button>
                    );
                  })}
                </div>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '6px', marginTop: '6px' }}>
                  {TFT_COMPONENTS.slice(5).map((comp) => {
                    const isSelected = mixerComp2 === comp.id;
                    return (
                      <button
                        key={comp.id}
                        onClick={() => {
                          setMixerComp2(comp.id);
                          soundService.playTimerTick();
                        }}
                        style={{
                          padding: '6px',
                          borderRadius: '6px',
                          background: isSelected ? 'rgba(0, 210, 255, 0.25)' : 'rgba(255, 255, 255, 0.04)',
                          border: isSelected ? '1.5px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                          cursor: 'pointer',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                        }}
                        title={comp.name}
                      >
                        <img src={getTftItemIcon(comp.id)} alt={comp.name} style={{ width: '26px', height: '26px', borderRadius: '4px' }} />
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* Equals Arrow */}
              <div style={{ fontSize: '20px', fontWeight: 900, color: '#facc15' }}>
                <ArrowRight size={24} />
              </div>

              {/* Live Crafted Result Card */}
              <div
                style={{
                  padding: '12px 16px',
                  borderRadius: '10px',
                  background: 'rgba(0, 0, 0, 0.5)',
                  border: '1.5px solid #facc15',
                  boxShadow: '0 0 16px rgba(250, 204, 21, 0.2)',
                }}
              >
                {mixedItem ? (
                  <div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '8px' }}>
                      <img
                        src={getTftItemIcon(mixedItem.name)}
                        alt={mixedItem.name}
                        style={{ width: '38px', height: '38px', borderRadius: '6px', border: '1.5px solid #facc15' }}
                      />
                      <div>
                        <div style={{ fontSize: '14px', fontWeight: 900, color: '#fff' }}>{mixedItem.name}</div>
                        <span
                          style={{
                            fontSize: '9.5px',
                            fontWeight: 800,
                            padding: '1px 6px',
                            borderRadius: '3px',
                            background: 'rgba(250, 204, 21, 0.15)',
                            color: '#facc15',
                          }}
                        >
                          {mixedItem.category}
                        </span>
                      </div>
                    </div>
                    <div style={{ fontSize: '11px', fontWeight: 700, color: '#facc15', marginBottom: '4px' }}>
                      {mixedItem.stats}
                    </div>
                    <div style={{ fontSize: '11px', color: '#cbd5e1', lineHeight: '1.4', marginBottom: '6px' }}>
                      {mixedItem.description}
                    </div>
                    <div style={{ fontSize: '10px', color: '#94a3b8' }}>
                      Best on: <strong style={{ color: '#fff' }}>{mixedItem.bestChampions.join(', ')}</strong>
                    </div>
                  </div>
                ) : (
                  <div style={{ fontSize: '12px', color: '#94a3b8', textAlign: 'center', padding: '10px 0' }}>
                    Select 2 components to craft recipe.
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Component Quick Filter Strip & Search */}
          <div className="glass-card" style={{ padding: '12px 16px', display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '10px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexWrap: 'wrap' }}>
              <button
                onClick={() => setSelectedComponentFilter(null)}
                style={{
                  padding: '5px 12px',
                  borderRadius: '6px',
                  background: selectedComponentFilter === null ? 'var(--blue-primary)' : 'rgba(255, 255, 255, 0.05)',
                  border: selectedComponentFilter === null ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                  color: selectedComponentFilter === null ? '#050811' : '#fff',
                  fontSize: '11px',
                  fontWeight: 800,
                  cursor: 'pointer',
                }}
              >
                All Combined Items ({TFT_COMBINED_ITEMS.length})
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
                      padding: '4px 8px',
                      borderRadius: '6px',
                      background: isSelected ? 'rgba(0, 210, 255, 0.2)' : 'rgba(255, 255, 255, 0.03)',
                      border: isSelected ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                      color: isSelected ? 'var(--blue-primary)' : '#cbd5e1',
                      fontSize: '11px',
                      fontWeight: 700,
                      cursor: 'pointer',
                    }}
                  >
                    <img src={getTftItemIcon(comp.id)} alt={comp.name} style={{ width: '16px', height: '16px', borderRadius: '3px' }} />
                    <span>{comp.name}</span>
                  </button>
                );
              })}
            </div>

            <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
              <Search size={14} color="#64748b" />
              <input
                type="text"
                placeholder="Search items or champions..."
                value={itemSearch}
                onChange={(e) => setItemSearch(e.target.value)}
                style={{
                  padding: '5px 10px',
                  borderRadius: '6px',
                  background: 'rgba(0, 0, 0, 0.4)',
                  border: '1px solid var(--border-subtle)',
                  color: '#fff',
                  fontSize: '11px',
                  outline: 'none',
                  width: '200px',
                }}
              />
            </div>
          </div>

          {/* Combined Items Grid */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(290px, 1fr))', gap: '12px' }}>
            {filteredItems.map((item) => {
              const comp1 = TFT_COMPONENTS.find((c) => c.id === item.component1);
              const comp2 = TFT_COMPONENTS.find((c) => c.id === item.component2);

              return (
                <div
                  key={item.id}
                  className="glass-card"
                  style={{
                    padding: '14px',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '10px',
                    border: '1px solid rgba(255, 255, 255, 0.08)',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <img
                        src={getTftItemIcon(item.name)}
                        alt={item.name}
                        style={{ width: '32px', height: '32px', borderRadius: '6px', border: '1px solid var(--border-subtle)' }}
                      />
                      <strong style={{ fontSize: '13.5px', color: '#fff' }}>{item.name}</strong>
                    </div>
                    <span
                      style={{
                        fontSize: '9.5px',
                        fontWeight: 800,
                        padding: '2px 6px',
                        borderRadius: '4px',
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

                  {/* Recipe Formula */}
                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '10.5px', color: '#94a3b8' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '4px', background: 'rgba(255, 255, 255, 0.05)', padding: '2px 6px', borderRadius: '4px' }}>
                      <img src={getTftItemIcon(comp1?.id || '')} alt="" style={{ width: '14px', height: '14px', borderRadius: '2px' }} />
                      <span>{comp1?.name}</span>
                    </div>
                    <span>+</span>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '4px', background: 'rgba(255, 255, 255, 0.05)', padding: '2px 6px', borderRadius: '4px' }}>
                      <img src={getTftItemIcon(comp2?.id || '')} alt="" style={{ width: '14px', height: '14px', borderRadius: '2px' }} />
                      <span>{comp2?.name}</span>
                    </div>
                  </div>

                  <div style={{ fontSize: '11px', fontWeight: 800, color: '#facc15' }}>
                    {item.stats}
                  </div>

                  <div style={{ fontSize: '11.5px', color: '#cbd5e1', lineHeight: '1.4' }}>
                    {item.description}
                  </div>

                  {/* Best on champions avatars */}
                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px', marginTop: 'auto', paddingTop: '8px', borderTop: '1px solid rgba(255,255,255,0.06)' }}>
                    <span style={{ fontSize: '10px', color: '#64748b' }}>Best On:</span>
                    <div style={{ display: 'flex', gap: '4px' }}>
                      {item.bestChampions.map((c) => (
                        <div
                          key={c}
                          style={{
                            width: '20px',
                            height: '20px',
                            borderRadius: '4px',
                            overflow: 'hidden',
                            border: '1px solid var(--border-subtle)',
                          }}
                          title={c}
                        >
                          <img src={getTftChampionAvatar(c)} alt={c} style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* ========================================================= */}
      {/* TAB 3: LEVEL ROLLING ODDS & LEVELING TIMETABLE            */}
      {/* ========================================================= */}
      {activeTab === 'odds' && (
        <div className="glass-card" style={{ padding: '20px' }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '16px', flexWrap: 'wrap', gap: '10px' }}>
            <div>
              <h3 style={{ fontSize: '16px', fontWeight: 900, color: '#fff', margin: 0, display: 'flex', alignItems: 'center', gap: '8px' }}>
                <Dice5 size={18} color="var(--blue-primary)" />
                <span>TFT Level Rolling Odds & Shop Probability Matrix</span>
              </h3>
              <p style={{ fontSize: '12px', color: '#94a3b8', margin: '4px 0 0 0' }}>
                Exact percentage chances of rolling 1-cost through 5-cost champions per shop slot across all Tactician levels.
              </p>
            </div>
            <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap' }}>
              <span style={{ fontSize: '10.5px', fontWeight: 800, padding: '3px 8px', borderRadius: '4px', background: 'rgba(34, 197, 94, 0.15)', color: '#22c55e', border: '1px solid rgba(34, 197, 94, 0.3)' }}>
                Lvl 5: 1-Cost Reroll (45%)
              </span>
              <span style={{ fontSize: '10.5px', fontWeight: 800, padding: '3px 8px', borderRadius: '4px', background: 'rgba(0, 210, 255, 0.15)', color: '#00d2ff', border: '1px solid var(--border-blue)' }}>
                Lvl 7: 3-Cost Reroll (35%)
              </span>
              <span style={{ fontSize: '10.5px', fontWeight: 800, padding: '3px 8px', borderRadius: '4px', background: 'rgba(192, 132, 252, 0.15)', color: '#c084fc', border: '1px solid rgba(192, 132, 252, 0.3)' }}>
                Lvl 8: 4-Cost Carries (18%)
              </span>
              <span style={{ fontSize: '10.5px', fontWeight: 800, padding: '3px 8px', borderRadius: '4px', background: 'rgba(250, 204, 21, 0.15)', color: '#facc15', border: '1px solid rgba(250, 204, 21, 0.3)' }}>
                Lvl 9: 5-Cost Legendaries (10%)
              </span>
            </div>
          </div>

          <div style={{ overflowX: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '12.5px', textAlign: 'center' }}>
              <thead>
                <tr style={{ borderBottom: '1px solid rgba(255, 255, 255, 0.12)', color: '#64748b' }}>
                  <th style={{ padding: '12px', textAlign: 'left' }}>Level</th>
                  <th style={{ padding: '12px' }}>Standard Round</th>
                  <th style={{ padding: '12px', color: '#94a3b8' }}>1-Cost (%)</th>
                  <th style={{ padding: '12px', color: '#22c55e' }}>2-Cost (%)</th>
                  <th style={{ padding: '12px', color: '#00d2ff' }}>3-Cost (%)</th>
                  <th style={{ padding: '12px', color: '#c084fc' }}>4-Cost (%)</th>
                  <th style={{ padding: '12px', color: '#facc15' }}>5-Cost (%)</th>
                  <th style={{ padding: '12px', textAlign: 'right' }}>XP to Level Up</th>
                </tr>
              </thead>
              <tbody>
                {TFT_ROLL_ODDS.map((row) => (
                  <tr
                    key={row.level}
                    style={{
                      borderBottom: '1px solid rgba(255, 255, 255, 0.05)',
                      background: row.level === 8 ? 'rgba(0, 210, 255, 0.08)' : row.level === 9 ? 'rgba(250, 204, 21, 0.06)' : 'transparent',
                    }}
                  >
                    <td style={{ padding: '12px', textAlign: 'left', fontWeight: 900, color: '#fff' }}>
                      Level {row.level} {row.level === 8 && <span style={{ fontSize: '9px', color: '#00d2ff', marginLeft: '6px' }}>★POWERSPIKE</span>}
                    </td>
                    <td style={{ padding: '12px', color: '#cbd5e1' }}>{row.recommendedRound}</td>
                    <td style={{ padding: '12px', color: '#94a3b8', fontWeight: 800 }}>{row.cost1}%</td>
                    <td style={{ padding: '12px', color: '#22c55e', fontWeight: 800 }}>{row.cost2}%</td>
                    <td style={{ padding: '12px', color: '#00d2ff', fontWeight: 800 }}>{row.cost3}%</td>
                    <td style={{ padding: '12px', color: '#c084fc', fontWeight: 800 }}>{row.cost4}%</td>
                    <td style={{ padding: '12px', color: '#facc15', fontWeight: 800 }}>{row.cost5}%</td>
                    <td style={{ padding: '12px', textAlign: 'right', color: '#94a3b8' }}>
                      {row.xpToNext > 0 ? `${row.xpToNext} XP (${row.goldCostEstimate}g)` : 'Max Cap'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* ========================================================= */}
      {/* TAB 4: AUGMENTS TIER LIST & WINRATES                      */}
      {/* ========================================================= */}
      {activeTab === 'augments' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
          {/* Filter Bar */}
          <div className="glass-card" style={{ padding: '12px 16px', display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '10px' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <span style={{ fontSize: '11px', fontWeight: 700, color: '#64748b' }}>Augment Tier:</span>
              {(['All', 'Silver', 'Gold', 'Prismatic'] as const).map((tier) => (
                <button
                  key={tier}
                  onClick={() => setAugmentTierFilter(tier)}
                  style={{
                    padding: '5px 12px',
                    borderRadius: '6px',
                    background: augmentTierFilter === tier ? 'var(--blue-primary)' : 'rgba(255, 255, 255, 0.04)',
                    border: augmentTierFilter === tier ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                    color: augmentTierFilter === tier ? '#050811' : '#fff',
                    fontSize: '11px',
                    fontWeight: 800,
                    cursor: 'pointer',
                  }}
                >
                  {tier}
                </button>
              ))}
            </div>

            <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
              <Search size={14} color="#64748b" />
              <input
                type="text"
                placeholder="Search augment name or effect..."
                value={augmentSearch}
                onChange={(e) => setAugmentSearch(e.target.value)}
                style={{
                  padding: '5px 10px',
                  borderRadius: '6px',
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

          {/* Augments Grid */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(290px, 1fr))', gap: '12px' }}>
            {filteredAugments.map((aug) => {
              const tierColor =
                aug.tier === 'Prismatic' ? '#facc15' : aug.tier === 'Gold' ? '#eab308' : '#94a3b8';

              return (
                <div
                  key={aug.id}
                  className="glass-card"
                  style={{
                    padding: '16px',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '10px',
                    border: `1.5px solid ${tierColor}40`,
                    background: `linear-gradient(135deg, ${tierColor}08 0%, rgba(10, 16, 28, 0.8) 100%)`,
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <strong style={{ fontSize: '14px', color: '#fff' }}>{aug.name}</strong>
                    <span
                      style={{
                        fontSize: '9.5px',
                        fontWeight: 900,
                        padding: '2px 7px',
                        borderRadius: '4px',
                        background: `${tierColor}20`,
                        color: tierColor,
                        border: `1px solid ${tierColor}50`,
                      }}
                    >
                      {aug.tier}
                    </span>
                  </div>

                  <div style={{ display: 'flex', alignItems: 'center', gap: '14px', fontSize: '11.5px' }}>
                    <span style={{ color: '#10b981', fontWeight: 800 }}>★ {aug.winRate}% Win Rate</span>
                    <span style={{ color: 'var(--blue-primary)', fontWeight: 700 }}>Avg #{aug.avgPlacement}</span>
                  </div>

                  <p style={{ fontSize: '11.5px', color: '#cbd5e1', margin: 0, lineHeight: '1.45' }}>
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
