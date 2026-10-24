import React, { useState, useEffect } from 'react';
import { LiveGameData, AppSettings } from '../types';
import {
  Maximize2,
  Minus,
  Pin,
  Zap,
  ShoppingBag,
  Swords,
  Timer,
  TrendingUp,
  Copy,
  Check,
  ChevronRight,
  ChevronDown,
  Eye,
  PanelTop,
  Columns,
  Sparkles,
  X,
} from 'lucide-react';
import { CHAMPIONS_LIST, getChampionBuild, getChampionIcon } from '../services/championData';
import { soundService } from '../services/soundService';
import { itemSetService } from '../services/itemSetService';
import { MAYHEM_CARDS, searchMayhemCards, CardTier } from '../services/mayhemCardsData';

interface RiftFloatingOverlayProps {
  liveData: LiveGameData | null;
  settings?: AppSettings;
  onExpand: () => void;
  onMinimize?: () => void;
}

type OverlayTab = 'skills' | 'build' | 'cards' | 'matchup' | 'stats' | 'timers';
type OverlayHudMode = 'integrated' | 'card' | 'bar' | 'pill';

export const RiftFloatingOverlay: React.FC<RiftFloatingOverlayProps> = ({
  liveData,
  settings,
  onExpand,
  onMinimize,
}) => {
  const isLive = liveData !== null && liveData.gameData !== undefined;
  const [hudMode, setHudMode] = useState<OverlayHudMode>('integrated');
  const [activeTab, setActiveTab] = useState<OverlayTab>('skills');
  const [isPreviewMode, setIsPreviewMode] = useState<boolean>(!isLive);
  const showInGameHud = isLive || isPreviewMode;
  const isDemoMode = !isLive && isPreviewMode;
  const [demoClock, setDemoClock] = useState<number>(765); // 12:45
  const [isPinned, setIsPinned] = useState(true);
  const [copiedChat, setCopiedChat] = useState(false);
  const [shopExportStatus, setShopExportStatus] = useState<'idle' | 'success' | 'error'>('idle');
  const [overlayOpacity, setOverlayOpacity] = useState<number>(0.92);
  const [showCardsSheet, setShowCardsSheet] = useState<boolean>(false);
  const [hudCardSearch, setHudCardSearch] = useState<string>('');
  const [hudCardTierFilter, setHudCardTierFilter] = useState<CardTier | 'ALL'>('ALL');

  // Auto-expand to full desktop app if outside an active match and not in preview mode
  useEffect(() => {
    if (!showInGameHud && !isPreviewMode) {
      onExpand();
    }
  }, [showInGameHud, isPreviewMode, onExpand]);

  // Keyboard shortcut listener: ESC returns to full app
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        if (isPreviewMode) {
          setIsPreviewMode(false);
        } else {
          onExpand();
        }
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isPreviewMode, onExpand]);

  // Helper to dynamically resize the native window
  const resizeHud = (w: number, h: number) => {
    if (window.riftAPI?.resizeWindow) {
      window.riftAPI.resizeWindow(w, h);
    } else if ((window as any).ipc) {
      (window as any).ipc.postMessage(JSON.stringify({ action: 'resize-window', width: w, height: h }));
    }
  };

  const handleSwitchToIntegrated = () => {
    setHudMode('integrated');
    if (window.riftAPI?.setHudMode) {
      window.riftAPI.setHudMode('integrated');
    } else if ((window as any).ipc) {
      (window as any).ipc.postMessage(JSON.stringify({ action: 'set-hud-mode', mode: 'integrated' }));
    }
  };

  const handleSwitchToCard = () => {
    setHudMode('card');
    if (window.riftAPI?.setHudMode) {
      window.riftAPI.setHudMode('card');
    } else if ((window as any).ipc) {
      (window as any).ipc.postMessage(JSON.stringify({ action: 'set-hud-mode', mode: 'card' }));
    } else {
      resizeHud(340, 270);
    }
  };

  const handleSwitchToBar = () => {
    setHudMode('bar');
    if (window.riftAPI?.setHudMode) {
      window.riftAPI.setHudMode('bar');
    } else if ((window as any).ipc) {
      (window as any).ipc.postMessage(JSON.stringify({ action: 'set-hud-mode', mode: 'bar' }));
    } else {
      resizeHud(620, 42);
      if (window.riftAPI?.snapOverlay) {
        window.riftAPI.snapOverlay('top-center', 620, 42);
      }
    }
  };

  const handleCollapseToPill = () => {
    setHudMode('pill');
    resizeHud(280, 38);
  };

  const handleCycleOpacity = () => {
    const nextOp = overlayOpacity === 0.92 ? 0.72 : overlayOpacity === 0.72 ? 0.50 : overlayOpacity === 0.50 ? 0.98 : 0.92;
    setOverlayOpacity(nextOp);
  };

  // Ensure integrated mode activates native fullscreen transparent click-through
  useEffect(() => {
    if (hudMode === 'integrated') {
      if (window.riftAPI?.setHudMode) {
        window.riftAPI.setHudMode('integrated');
      } else if ((window as any).ipc) {
        (window as any).ipc.postMessage(JSON.stringify({ action: 'set-hud-mode', mode: 'integrated' }));
      }
    }
  }, [hudMode]);

  // Clock tick in demo mode
  useEffect(() => {
    if (!isDemoMode || isLive) return;
    const timer = setInterval(() => {
      setDemoClock((prev) => prev + 1);
    }, 1000);
    return () => clearInterval(timer);
  }, [isDemoMode, isLive]);

  // Live or Demo values
  const gameTime = isLive
    ? Math.floor(liveData.gameData.gameTime)
    : isDemoMode
    ? demoClock
    : 0;

  const gameMinutes = Math.max(1, gameTime / 60);

  // Active Player info
  const activeSumm = liveData?.activePlayer?.summonerName?.toLowerCase() || '';
  const myPlayer = liveData?.allPlayers?.find(
    (p) => p.summonerName.toLowerCase() === activeSumm || p.championName.toLowerCase() === activeSumm
  ) || liveData?.allPlayers?.[0];

  const myTeam = myPlayer?.team || 'ORDER';

  // My Champion Identity
  const rawChampName = isLive ? (myPlayer?.championName || 'Ahri') : 'Illaoi';
  const myChampObj = CHAMPIONS_LIST.find((c) => c.name.toLowerCase() === rawChampName.toLowerCase()) || CHAMPIONS_LIST[0];
  const myChampBuild = getChampionBuild(myChampObj.id, myChampObj.name);

  // Detect Enemy Lane Opponent
  const enemyTeam = myTeam === 'ORDER' ? 'CHAOS' : 'ORDER';
  const enemyPlayers = isLive
    ? (liveData?.allPlayers?.filter((p) => p.team === enemyTeam) || [])
    : [];

  const laneOpponent = isLive
    ? (enemyPlayers[0] || { championName: myChampBuild.counters.weakAgainst[0]?.name || 'Darius' })
    : { championName: 'Darius' };

  const opponentChampObj = CHAMPIONS_LIST.find((c) => c.name.toLowerCase() === laneOpponent.championName.toLowerCase()) || {
    id: 122,
    name: 'Darius',
    key: 'Darius',
    roles: ['TOP'],
  };
  const opponentBuild = getChampionBuild(opponentChampObj.id, opponentChampObj.name);

  // Stats calculation
  const currentCs = isLive
    ? (liveData?.activePlayer?.scores?.creepScore ?? myPlayer?.scores?.creepScore ?? 0)
    : isDemoMode
    ? Math.floor((demoClock / 60) * 8.2)
    : 0;

  const csm = (currentCs / gameMinutes).toFixed(1);
  const benchmarkCSM = 7.5;
  const csmDelta = (parseFloat(csm) - benchmarkCSM).toFixed(1);
  const isCsmAhead = parseFloat(csmDelta) >= 0;

  // Opponent CS & delta
  const opponentCs = isLive
    ? ((laneOpponent as any)?.scores?.creepScore ?? 0)
    : isDemoMode
    ? Math.max(0, Math.floor(currentCs - 14))
    : 0;
  const csDiffVsOpponent = currentCs - opponentCs;

  const myLevel = isLive
    ? (liveData?.activePlayer?.level || myPlayer?.level || 1)
    : isDemoMode
    ? Math.min(18, Math.max(1, Math.floor(demoClock / 85)))
    : 1;

  const currentGold = isLive
    ? (liveData?.activePlayer?.currentGold ?? (myPlayer?.currentGold || 0))
    : isDemoMode
    ? 2450
    : 500;

  const teamKills = isLive
    ? (liveData?.allPlayers?.filter((p) => p.team === myTeam)?.reduce((sum, p) => sum + (p.scores?.kills || 0), 0) || 0)
    : 16;
  const myKills = isLive ? (myPlayer?.scores?.kills || 0) : 4;
  const myDeaths = isLive ? (myPlayer?.scores?.deaths || 0) : 1;
  const myAssists = isLive ? (myPlayer?.scores?.assists || 0) : 5;
  const kp = teamKills > 0 ? Math.round(((myKills + myAssists) / teamKills) * 100) : 56;
  const kdaRatio = myDeaths > 0 ? ((myKills + myAssists) / myDeaths).toFixed(1) : (myKills + myAssists).toFixed(1);
  const gpm = Math.floor(((currentGold + currentCs * 21 + myKills * 300) / gameMinutes) || 410);

  // Gold calculation for scoreboard / tab section
  const calcPlayerGold = (p: any) => {
    if (!p) return 0;
    const csGold = (p.scores?.creepScore || 0) * 21;
    const killGold = (p.scores?.kills || 0) * 300;
    const assistGold = (p.scores?.assists || 0) * 100;
    const passiveGold = Math.floor(gameMinutes * 125);
    return 500 + csGold + killGold + assistGold + passiveGold;
  };

  const myTeamTotalGold = isLive && liveData?.allPlayers?.length
    ? liveData.allPlayers.filter((p) => p.team === myTeam).reduce((sum, p) => sum + calcPlayerGold(p), 0)
    : Math.floor(21400 + (gameMinutes * 650));

  const enemyTeamTotalGold = isLive && liveData?.allPlayers?.length
    ? liveData.allPlayers.filter((p) => p.team === enemyTeam).reduce((sum, p) => sum + calcPlayerGold(p), 0)
    : Math.floor(19600 + (gameMinutes * 600));

  const teamGoldDiff = myTeamTotalGold - enemyTeamTotalGold;
  const isTeamGoldAhead = teamGoldDiff >= 0;

  const myLaneGold = isLive ? calcPlayerGold(myPlayer) : Math.floor(currentGold + currentCs * 21 + myKills * 300);
  const enemyLaneGold = isLive ? calcPlayerGold(laneOpponent) : Math.max(500, myLaneGold - 450);
  const laneGoldDiff = myLaneGold - enemyLaneGold;
  const isLaneGoldAhead = laneGoldDiff >= 0;

  // Next skill to level up calculation
  const nextSkillIndex = Math.min(17, myLevel);
  const nextSkillKey = myChampBuild.skillOrder[nextSkillIndex] || myChampBuild.skillOrder[0] || 'Q';

  // Current item target & components
  const targetCoreItem = myChampBuild.coreItems[1] || myChampBuild.coreItems[0] || { id: 3071, name: 'Black Cleaver' };
  const targetCost = 3100;
  const goldRemaining = Math.max(0, targetCost - currentGold);

  // Summoner Spell Timers (Interactive Click-to-Track)
  const [enemySpells, setEnemySpells] = useState([
    { role: 'TOP', champ: opponentChampObj.name, name: 'Flash', total: 300, remaining: 0, active: false, icon: 'https://ddragon.leagueoflegends.com/cdn/14.23.1/img/spell/SummonerFlash.png' },
    { role: 'TOP', champ: opponentChampObj.name, name: 'Ghost', total: 240, remaining: 0, active: false, icon: 'https://ddragon.leagueoflegends.com/cdn/14.23.1/img/spell/SummonerHaste.png' },
    { role: 'MID', champ: 'Mid Lane', name: 'Flash', total: 300, remaining: 0, active: false, icon: 'https://ddragon.leagueoflegends.com/cdn/14.23.1/img/spell/SummonerFlash.png' },
    { role: 'ADC', champ: 'ADC', name: 'Flash', total: 300, remaining: 0, active: false, icon: 'https://ddragon.leagueoflegends.com/cdn/14.23.1/img/spell/SummonerFlash.png' },
    { role: 'SUP', champ: 'Support', name: 'Ignite', total: 180, remaining: 0, active: false, icon: 'https://ddragon.leagueoflegends.com/cdn/14.23.1/img/spell/SummonerDot.png' },
  ]);

  // Separate camp timers positioned directly on the in-game minimap
  const [campTimers, setCampTimers] = useState([
    { id: 'dragon', name: 'Drake', icon: '🐲', total: 300, remaining: 42, color: '#38bdf8', mapPos: { bottom: '72px', right: '78px' } },
    { id: 'baron', name: 'Baron', icon: '👑', total: 1200, remaining: Math.max(0, 1200 - gameTime), color: '#c084fc', mapPos: { bottom: '162px', right: '166px' } },
    { id: 'grubs', name: 'Grubs', icon: '👾', total: 360, remaining: 0, color: '#f43f5e', mapPos: { bottom: '168px', right: '176px' } },
    { id: 'blue', name: 'Blue', icon: '🔵', total: 300, remaining: 105, color: '#3b82f6', mapPos: { bottom: '110px', right: '160px' } },
    { id: 'red', name: 'Red', icon: '🔴', total: 300, remaining: 130, color: '#ef4444', mapPos: { bottom: '60px', right: '124px' } },
  ]);

  // Objective Timers
  const [objectives, setObjectives] = useState([
    { name: 'Dragon', icon: '🐲', total: 300, remaining: 0, color: 'var(--blue-primary)' },
    { name: 'Voidgrubs', icon: '👾', total: 360, remaining: 0, color: 'var(--red-primary)' },
    { name: 'Baron', icon: '👑', total: 1200, remaining: 0, color: '#c084fc' },
  ]);

  // Dynamic in-game objective & camp timers from live telemetry
  useEffect(() => {
    if (!isLive || !liveData?.gameData) return;
    const gTime = Math.floor(liveData.gameData.gameTime);
    const events = liveData.events?.Events || [];

    // Dragon calculation
    let nextDragonTime = 300;
    const dragonKills = events.filter((e) => e.EventName === 'DragonKill');
    if (dragonKills.length > 0) {
      const lastKill = dragonKills[dragonKills.length - 1];
      const cooldown = lastKill.DragonType === 'Elder' ? 360 : 300;
      nextDragonTime = Math.floor(lastKill.EventTime) + cooldown;
    }
    const dragonRemaining = Math.max(0, nextDragonTime - gTime);

    // Baron calculation
    let nextBaronTime = 1200;
    const baronKills = events.filter((e) => e.EventName === 'BaronKill');
    if (baronKills.length > 0) {
      const lastBaron = baronKills[baronKills.length - 1];
      nextBaronTime = Math.floor(lastBaron.EventTime) + 360;
    }
    const baronRemaining = Math.max(0, nextBaronTime - gTime);

    // Voidgrubs / Herald calculation
    let grubsRemaining = 0;
    if (gTime < 300) {
      grubsRemaining = 300 - gTime;
    } else if (gTime < 840) {
      const hordeKills = events.filter((e) => e.EventName === 'HordeKill');
      grubsRemaining = hordeKills.length >= 6 ? Math.max(0, 840 - gTime) : 0;
    } else if (gTime < 1185) {
      const heraldKills = events.filter((e) => e.EventName === 'HeraldKill');
      grubsRemaining = heraldKills.length > 0 ? Math.max(0, 1200 - gTime) : 0;
    } else {
      grubsRemaining = Math.max(0, 1200 - gTime);
    }

    // Buffs (first spawn at 1:30 = 90s)
    const blueRemaining = gTime < 90 ? 90 - gTime : 0;
    const redRemaining = gTime < 90 ? 90 - gTime : 0;

    setCampTimers([
      { id: 'dragon', name: 'Drake', icon: '🐲', total: 300, remaining: dragonRemaining, color: '#38bdf8', mapPos: { bottom: '72px', right: '78px' } },
      { id: 'baron', name: 'Baron', icon: '👑', total: 1200, remaining: baronRemaining, color: '#c084fc', mapPos: { bottom: '162px', right: '166px' } },
      { id: 'grubs', name: 'Grubs', icon: '👾', total: 360, remaining: grubsRemaining, color: '#f43f5e', mapPos: { bottom: '168px', right: '176px' } },
      { id: 'blue', name: 'Blue', icon: '🔵', total: 300, remaining: blueRemaining, color: '#3b82f6', mapPos: { bottom: '110px', right: '160px' } },
      { id: 'red', name: 'Red', icon: '🔴', total: 300, remaining: redRemaining, color: '#ef4444', mapPos: { bottom: '60px', right: '124px' } },
    ]);

    setObjectives([
      { name: 'Dragon', icon: '🐲', total: 300, remaining: dragonRemaining, color: 'var(--blue-primary)' },
      { name: 'Voidgrubs', icon: '👾', total: 360, remaining: grubsRemaining, color: 'var(--red-primary)' },
      { name: 'Baron', icon: '👑', total: 1200, remaining: baronRemaining, color: '#c084fc' },
    ]);
  }, [liveData, isLive]);

  // Countdown ticker
  useEffect(() => {
    const ticker = setInterval(() => {
      setCampTimers((prev) =>
        prev.map((c) => (c.remaining > 0 ? { ...c, remaining: c.remaining - 1 } : c))
      );
      setEnemySpells((prev) =>
        prev.map((s) => {
          if (s.active && s.remaining > 0) {
            return { ...s, remaining: s.remaining - 1 };
          } else if (s.active && s.remaining <= 1) {
            soundService.playAutoAccept();
            return { ...s, active: false, remaining: 0 };
          }
          return s;
        })
      );
      setObjectives((prev) =>
        prev.map((obj) => {
          if (obj.remaining > 0) {
            return { ...obj, remaining: obj.remaining - 1 };
          }
          return obj;
        })
      );
    }, 1000);
    return () => clearInterval(ticker);
  }, []);

  const handleTriggerSpell = (idx: number) => {
    setEnemySpells((prev) => {
      const copy = [...prev];
      if (copy[idx].active) {
        copy[idx].active = false;
        copy[idx].remaining = 0;
      } else {
        copy[idx].active = true;
        copy[idx].remaining = copy[idx].total;
        soundService.playTimerTick();
      }
      return copy;
    });
  };

  const handleCopyTimers = () => {
    const active = enemySpells.filter((s) => s.active);
    let msg = '';
    if (active.length > 0) {
      msg = `[RiftCore] ` + active.map((s) => `${s.role} ${s.name} ready in ${s.remaining}s`).join(' | ');
    } else {
      msg = `[RiftCore] All enemy summoner spells UP • Next Dragon in ${objectives[0].remaining}s`;
    }
    navigator.clipboard.writeText(msg);
    setCopiedChat(true);
    soundService.playAutoAccept();
    setTimeout(() => setCopiedChat(false), 2500);
  };

  const handleExportShop = async () => {
    try {
      const res = await itemSetService.exportItemSetToClient(myChampBuild);
      if (res && res.success) {
        setShopExportStatus('success');
        soundService.playRuneImport();
        setTimeout(() => setShopExportStatus('idle'), 3000);
      } else {
        setShopExportStatus('error');
        setTimeout(() => setShopExportStatus('idle'), 3000);
      }
    } catch {
      setShopExportStatus('error');
      setTimeout(() => setShopExportStatus('idle'), 3000);
    }
  };

  const handleTogglePin = async () => {
    const next = !isPinned;
    setIsPinned(next);
    if (window.riftAPI?.setAlwaysOnTop) {
      await window.riftAPI.setAlwaysOnTop(next);
    } else if (window.riftAPI?.windowControl) {
      await window.riftAPI.windowControl('toggle-pin');
    }
  };

  // Drag handler for window
  const handleStartDrag = () => {
    if (window.riftAPI?.windowControl) {
      (window.riftAPI.windowControl as any)('drag-window');
    } else if ((window as any).ipc) {
      (window as any).ipc.postMessage(JSON.stringify({ action: 'drag-window' }));
    }
  };

  const formatMinSec = (sec: number) => {
    const m = Math.floor(sec / 60);
    const s = sec % 60;
    return `${m}:${s < 10 ? '0' : ''}${s}`;
  };

  return (
    <div
      style={{
        width: '100vw',
        height: '100vh',
        background: 'transparent',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
        boxSizing: 'border-box',
        padding: 0,
        margin: 0,
      }}
    >
      {/* MODE 0: TRUE IN-GAME INTEGRATED HUD */}
      {hudMode === 'integrated' && (
        <>
          {/* Live / Preview Fullscreen HUD Layer */}
          <div
            style={{
              position: 'relative',
              width: '100vw',
              height: '100vh',
              background: 'transparent',
              pointerEvents: 'none',
              userSelect: 'none',
              overflow: 'hidden',
            }}
          >
            {/* Top Indicator in Preview Mode */}
            {isPreviewMode && (
              <div
                style={{
                  position: 'absolute',
                  top: '52px',
                  left: '50%',
                  transform: 'translateX(-50%)',
                  pointerEvents: 'auto',
                  zIndex: 99999,
                  display: 'flex',
                  alignItems: 'center',
                  gap: '10px',
                  padding: '5px 14px',
                  borderRadius: '20px',
                  background: 'rgba(5, 10, 20, 0.92)',
                  border: '1px solid var(--blue-primary)',
                  boxShadow: '0 4px 20px rgba(0, 210, 255, 0.3)',
                }}
              >
                <Eye size={13} color="var(--blue-primary)" />
                <span style={{ fontSize: '11px', fontWeight: 800, color: '#fff' }}>
                  HUD Layout Preview (Transparent Over League)
                </span>
                <button
                  onClick={() => {
                    setIsPreviewMode(false);
                    onExpand();
                  }}
                  style={{
                    padding: '3px 9px',
                    borderRadius: '12px',
                    background: 'rgba(255, 42, 95, 0.2)',
                    border: '1px solid var(--red-primary)',
                    color: '#fff',
                    fontSize: '10.5px',
                    fontWeight: 800,
                    cursor: 'pointer',
                  }}
                >
                  Exit Preview
                </button>
              </div>
            )}
              {/* 1. GOLD DIFFERENCE IN TAB SECTION (Top Center Scoreboard) */}
              {settings?.hudGoldLead !== false && (
                <div
                  style={{
                    position: 'absolute',
                    top: '8px',
                    left: '50%',
                    transform: 'translateX(-50%)',
                    pointerEvents: 'auto',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '10px',
                    background: 'rgba(5, 10, 20, 0.92)',
                    backdropFilter: 'blur(16px)',
                    WebkitBackdropFilter: 'blur(16px)',
                    padding: '5px 14px',
                    borderRadius: '24px',
                    border: `1.5px solid ${isTeamGoldAhead ? 'rgba(16, 185, 129, 0.55)' : 'rgba(239, 68, 68, 0.55)'}`,
                    boxShadow: `0 6px 24px rgba(0,0,0,0.85), 0 0 16px ${isTeamGoldAhead ? 'rgba(16, 185, 129, 0.25)' : 'rgba(239, 68, 68, 0.25)'}`,
                    zIndex: 9999,
                  }}
                >
                  {/* Team Gold Difference */}
                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span style={{ fontSize: '13px' }}>💰</span>
                    <div style={{ display: 'flex', flexDirection: 'column' }}>
                      <span style={{ fontSize: '8.5px', color: '#94a3b8', fontWeight: 800, textTransform: 'uppercase', letterSpacing: '0.5px' }}>
                        Team Gold
                      </span>
                      <span
                        style={{
                          fontSize: '12.5px',
                          fontWeight: 900,
                          color: isTeamGoldAhead ? '#10b981' : '#f43f5e',
                          textShadow: `0 0 8px ${isTeamGoldAhead ? 'rgba(16,185,129,0.5)' : 'rgba(244,63,94,0.5)'}`,
                        }}
                      >
                        {isTeamGoldAhead ? `+${(teamGoldDiff / 1000).toFixed(1)}k Lead` : `-${(Math.abs(teamGoldDiff) / 1000).toFixed(1)}k Deficit`}
                      </span>
                    </div>
                  </div>

                  {/* Split Visual Ratio Bar */}
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '2px', width: '80px' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '8px', fontWeight: 800 }}>
                      <span style={{ color: '#38bdf8' }}>{(myTeamTotalGold / 1000).toFixed(1)}k</span>
                      <span style={{ color: '#f43f5e' }}>{(enemyTeamTotalGold / 1000).toFixed(1)}k</span>
                    </div>
                    <div style={{ width: '100%', height: '4px', background: 'rgba(255,255,255,0.1)', borderRadius: '2px', overflow: 'hidden', display: 'flex' }}>
                      <div
                        style={{
                          width: `${Math.min(75, Math.max(25, (myTeamTotalGold / (myTeamTotalGold + enemyTeamTotalGold)) * 100))}%`,
                          background: isTeamGoldAhead ? '#10b981' : '#38bdf8',
                          height: '100%',
                        }}
                      />
                      <div style={{ flex: 1, background: '#f43f5e', height: '100%' }} />
                    </div>
                  </div>

                  <div style={{ width: '1px', height: '18px', background: 'rgba(255,255,255,0.15)' }} />

                  {/* Lane Matchup Gold Difference */}
                  <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <span style={{ fontSize: '11px' }}>⚔️</span>
                    <div style={{ display: 'flex', flexDirection: 'column' }}>
                      <span style={{ fontSize: '8.5px', color: '#94a3b8', fontWeight: 800, textTransform: 'uppercase', letterSpacing: '0.5px' }}>
                        Lane vs {laneOpponent.championName}
                      </span>
                      <span
                        style={{
                          fontSize: '12px',
                          fontWeight: 900,
                          color: isLaneGoldAhead ? '#38bdf8' : '#fb923c',
                        }}
                      >
                        {isLaneGoldAhead ? `+${laneGoldDiff}g` : `-${Math.abs(laneGoldDiff)}g`}
                      </span>
                    </div>
                  </div>

                  {/* Return / Exit Controls */}
                  <div style={{ display: 'flex', alignItems: 'center', gap: '4px', marginLeft: '6px', borderLeft: '1px solid rgba(255,255,255,0.15)', paddingLeft: '8px' }}>
                    {/* Mayhem & Arena Cards Drawer Toggle */}
                    {settings?.hudMayhemCards !== false && (
                      <button
                        onClick={() => setShowCardsSheet(!showCardsSheet)}
                        style={{
                          background: showCardsSheet ? 'rgba(192, 132, 252, 0.35)' : 'rgba(192, 132, 252, 0.15)',
                          border: '1px solid rgba(192, 132, 252, 0.5)',
                          color: '#c084fc',
                          fontSize: '9px',
                          fontWeight: 800,
                          padding: '2px 8px',
                          borderRadius: '10px',
                          cursor: 'pointer',
                          display: 'flex',
                          alignItems: 'center',
                          gap: '4px',
                        }}
                        title="Toggle ARAM Mayhem & Arena Cards Drawer"
                      >
                        <Sparkles size={9} />
                        <span>Cards</span>
                      </button>
                    )}

                    {isPreviewMode && (
                      <button
                        onClick={() => setIsPreviewMode(false)}
                        style={{
                          background: 'rgba(239, 68, 68, 0.15)',
                          border: '1px solid rgba(239, 68, 68, 0.4)',
                          color: '#f87171',
                          fontSize: '9px',
                          fontWeight: 800,
                          padding: '2px 7px',
                          borderRadius: '10px',
                          cursor: 'pointer',
                        }}
                        title="Exit Preview Mode"
                      >
                        Exit Preview
                      </button>
                    )}

                    <button
                      onClick={() => {
                        setIsPreviewMode(false);
                        onExpand();
                      }}
                      style={{
                        background: 'rgba(0, 210, 255, 0.15)',
                        border: '1px solid var(--border-blue)',
                        color: 'var(--blue-primary)',
                        fontSize: '9px',
                        fontWeight: 800,
                        padding: '2px 7px',
                        borderRadius: '10px',
                        cursor: 'pointer',
                        display: 'flex',
                        alignItems: 'center',
                        gap: '3px',
                      }}
                      title="Return to Full Desktop App"
                    >
                      <Maximize2 size={9} />
                      <span>Full App</span>
                    </button>

                    <button
                      onClick={() => {
                        setIsPreviewMode(false);
                        if (window.riftAPI?.setCompactHud) {
                          window.riftAPI.setCompactHud(false);
                        }
                        onExpand();
                      }}
                      style={{
                        background: 'rgba(239, 68, 68, 0.18)',
                        border: '1px solid rgba(239, 68, 68, 0.5)',
                        color: '#ff4d6d',
                        fontSize: '9px',
                        fontWeight: 800,
                        padding: '2px 7px',
                        borderRadius: '10px',
                        cursor: 'pointer',
                        display: 'flex',
                        alignItems: 'center',
                        gap: '3px',
                      }}
                      title="Close Overlay & Return to App"
                    >
                      <X size={9} />
                      <span>Close</span>
                    </button>
                  </div>
                </div>
              )}

              {/* 1b. FLOATING IN-GAME CARDS DRAWER (When toggled during Mayhem or Arena rounds) */}
              {settings?.hudMayhemCards !== false && showCardsSheet && (
                <div
                  style={{
                    position: 'absolute',
                    top: '48px',
                    left: '50%',
                    transform: 'translateX(-50%)',
                    width: '380px',
                    maxHeight: '340px',
                    background: 'rgba(6, 11, 22, 0.96)',
                    backdropFilter: 'blur(24px)',
                    WebkitBackdropFilter: 'blur(24px)',
                    border: '1px solid rgba(192, 132, 252, 0.4)',
                    boxShadow: '0 12px 36px rgba(0,0,0,0.9), 0 0 20px rgba(192, 132, 252, 0.2)',
                    borderRadius: '12px',
                    padding: '12px',
                    pointerEvents: 'auto',
                    zIndex: 10000,
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '8px',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                      <Sparkles size={13} color="#c084fc" />
                      <span style={{ fontSize: '12px', fontWeight: 800, color: '#fff' }}>Mayhem & Arena Cards Advice</span>
                    </div>
                    <div style={{ display: 'flex', gap: '3px' }}>
                      {(['ALL', 'Prismatic', 'Gold', 'Silver'] as const).map((tier) => (
                        <button
                          key={tier}
                          onClick={() => setHudCardTierFilter(tier)}
                          style={{
                            fontSize: '8px',
                            fontWeight: 800,
                            padding: '1px 5px',
                            borderRadius: '3px',
                            background: hudCardTierFilter === tier ? 'rgba(255,255,255,0.18)' : 'rgba(255,255,255,0.04)',
                            border: hudCardTierFilter === tier ? '1px solid rgba(255,255,255,0.4)' : '1px solid transparent',
                            color: tier === 'Prismatic' ? '#c084fc' : tier === 'Gold' ? '#f59e0b' : tier === 'Silver' ? '#94a3b8' : '#fff',
                            cursor: 'pointer',
                          }}
                        >
                          {tier === 'Prismatic' ? 'Prism' : tier}
                        </button>
                      ))}
                    </div>
                  </div>

                  <input
                    type="text"
                    placeholder="Search Mayhem cards (e.g. Crit, Haste, Bleed)..."
                    value={hudCardSearch}
                    onChange={(e) => setHudCardSearch(e.target.value)}
                    style={{
                      padding: '4px 8px',
                      borderRadius: '5px',
                      background: 'rgba(0,0,0,0.5)',
                      border: '1px solid var(--border-subtle)',
                      color: '#fff',
                      fontSize: '11px',
                      width: '100%',
                      boxSizing: 'border-box',
                    }}
                  />

                  <div style={{ overflowY: 'auto', maxHeight: '220px', display: 'flex', flexDirection: 'column', gap: '5px', paddingRight: '2px' }}>
                    {searchMayhemCards(hudCardSearch, hudCardTierFilter, 'Both').map((card) => (
                      <div
                        key={card.id}
                        style={{
                          background: 'rgba(255,255,255,0.03)',
                          border: `1px solid ${card.color}33`,
                          borderLeft: `3px solid ${card.color}`,
                          borderRadius: '5px',
                          padding: '6px 8px',
                        }}
                      >
                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '2px' }}>
                          <div style={{ display: 'flex', alignItems: 'center', gap: '5px' }}>
                            <span style={{ fontSize: '11px', fontWeight: 800, color: '#fff' }}>{card.name}</span>
                            <span style={{ fontSize: '8px', fontWeight: 800, padding: '0 4px', borderRadius: '2px', background: `${card.color}20`, color: card.color }}>
                              {card.tier}
                            </span>
                            <span style={{ fontSize: '8px', fontWeight: 900, padding: '0 4px', borderRadius: '2px', background: card.rank === 'S+' ? 'rgba(239,68,68,0.25)' : 'rgba(34,197,94,0.25)', color: card.rank === 'S+' ? '#ef4444' : '#22c55e' }}>
                              {card.rank}
                            </span>
                          </div>
                          <span style={{ fontSize: '8px', color: '#64748b' }}>{card.tag}</span>
                        </div>
                        <div style={{ fontSize: '9.5px', color: '#cbd5e1', lineHeight: '1.3' }}>{card.description}</div>
                        <div style={{ fontSize: '8.5px', color: '#94a3b8', marginTop: '2px' }}>
                          <span style={{ color: 'var(--blue-primary)', fontWeight: 700 }}>Best:</span> {card.archetype}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* 2. SMALL CS SCORE UP (Top Right, beside game clock) */}
              {settings?.hudCsBenchmark !== false && (
                <div
                  style={{
                    position: 'absolute',
                    top: '8px',
                    right: '270px',
                    pointerEvents: 'none',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '6px',
                    background: 'rgba(5, 10, 20, 0.90)',
                    backdropFilter: 'blur(14px)',
                    WebkitBackdropFilter: 'blur(14px)',
                    border: '1px solid rgba(0, 210, 255, 0.4)',
                    padding: '4px 10px',
                    borderRadius: '8px',
                    boxShadow: '0 4px 16px rgba(0,0,0,0.8), 0 0 10px rgba(0,210,255,0.15)',
                  }}
                >
                  <span style={{ fontSize: '12px', fontWeight: 900, color: '#f8fafc' }}>
                    🎯 {currentCs} CS
                  </span>
                  <span style={{ fontSize: '10px', color: 'var(--blue-primary)', fontWeight: 800 }}>
                    {csm}/m
                  </span>
                  <span
                    style={{
                      fontSize: '9px',
                      fontWeight: 900,
                      padding: '1px 5px',
                      borderRadius: '4px',
                      background: csDiffVsOpponent >= 0 ? 'rgba(16, 185, 129, 0.2)' : 'rgba(239, 68, 68, 0.2)',
                      color: csDiffVsOpponent >= 0 ? '#10b981' : '#f43f5e',
                      border: `1px solid ${csDiffVsOpponent >= 0 ? 'rgba(16, 185, 129, 0.45)' : 'rgba(239, 68, 68, 0.45)'}`,
                    }}
                  >
                    {csDiffVsOpponent >= 0 ? `▲ +${csDiffVsOpponent}` : `▼ ${csDiffVsOpponent}`} vs {laneOpponent.championName}
                  </span>
                </div>
              )}

              {/* 3. IN-MAP OBJECTIVE TIMERS (Pinned directly on minimap camp spawn pits) */}
              {settings?.hudObjectiveTimers !== false && (
                <>
                  {campTimers.map((camp) => {
                    // Only show countdown while dead/spawning (remaining > 0)
                    if (camp.remaining <= 0) return null;
                    return (
                      <div
                        key={camp.id}
                        style={{
                          position: 'absolute',
                          bottom: camp.mapPos.bottom,
                          right: camp.mapPos.right,
                          transform: 'translate(50%, 50%)',
                          pointerEvents: 'none',
                          zIndex: 9999,
                          display: 'flex',
                          alignItems: 'center',
                          gap: '3px',
                          background: 'rgba(5, 10, 20, 0.94)',
                          backdropFilter: 'blur(8px)',
                          WebkitBackdropFilter: 'blur(8px)',
                          border: `1.5px solid ${camp.remaining <= 30 ? '#f59e0b' : camp.color}`,
                          boxShadow: `0 2px 10px rgba(0,0,0,0.9), 0 0 ${camp.remaining <= 30 ? '12px #f59e0b' : '8px ' + camp.color}`,
                          borderRadius: '12px',
                          padding: '2px 6px',
                          animation: camp.remaining <= 15 ? 'pulse-blue 1.2s infinite' : 'none',
                        }}
                      >
                        <span style={{ fontSize: '10px' }}>{camp.icon}</span>
                        <span
                          className="tabular-nums"
                          style={{
                            fontSize: '10px',
                            fontWeight: 900,
                            color: camp.remaining <= 30 ? '#f59e0b' : camp.color,
                            letterSpacing: '0.2px',
                          }}
                        >
                          {camp.remaining > 60 ? formatMinSec(camp.remaining) : `${camp.remaining}s`}
                        </span>
                      </div>
                    );
                  })}
                </>
              )}

              {/* 4. SKILL BAR LEVEL-UP ANCHOR (Hovering directly over ability bar at bottom center) */}
              {settings?.hudSkillOrder !== false && (
                <div
                  style={{
                    position: 'absolute',
                    bottom: '95px',
                    left: '50%',
                    transform: 'translateX(-50%)',
                    pointerEvents: 'none',
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    animation: 'pulse-blue 2s infinite',
                  }}
                >
                  <div
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: '8px',
                      background: 'linear-gradient(135deg, rgba(8, 15, 30, 0.95) 0%, rgba(14, 25, 45, 0.95) 100%)',
                      border: '1.5px solid var(--blue-primary)',
                      boxShadow: '0 0 20px rgba(0, 210, 255, 0.4), inset 0 0 12px rgba(0, 210, 255, 0.2)',
                      padding: '5px 12px',
                      borderRadius: '8px',
                    }}
                  >
                    <div style={{ fontSize: '11px', fontWeight: 900, color: '#fff', display: 'flex', alignItems: 'center', gap: '6px' }}>
                      <span style={{ color: '#94a3b8', fontSize: '9.5px', textTransform: 'uppercase' }}>Next Level {Math.min(18, myLevel + 1)}:</span>
                      <span style={{ color: '#000', background: 'var(--blue-primary)', padding: '1px 7px', borderRadius: '4px', fontWeight: 900, fontSize: '12px', boxShadow: '0 0 10px var(--blue-glow)' }}>
                        {nextSkillKey}
                      </span>
                      <span style={{ color: 'var(--blue-primary)', fontWeight: 800 }}>Upgrade {nextSkillKey}</span>
                    </div>
                    <div style={{ height: '12px', width: '1px', background: 'rgba(255,255,255,0.2)', margin: '0 3px' }} />
                    <div style={{ fontSize: '9.5px', color: '#94a3b8', display: 'flex', alignItems: 'center', gap: '4px' }}>
                      <span>Max:</span>
                      <span style={{ color: '#cbd5e1', fontWeight: 800 }}>{myChampBuild.skillPriority}</span>
                    </div>
                  </div>

                  {/* Glowing downward pointer arrow */}
                  <div
                    style={{
                      width: 0,
                      height: 0,
                      borderLeft: '6px solid transparent',
                      borderRight: '6px solid transparent',
                      borderTop: '6px solid var(--blue-primary)',
                      filter: 'drop-shadow(0 2px 4px rgba(0,210,255,0.6))',
                      marginTop: '-1px',
                    }}
                  />
                </div>
              )}

              {/* 5. ITEM GOAL & BUY ALERT (Beside skill upgrade bar) */}
              {settings?.hudItemGoal !== false && (
                <div
                  style={{
                    position: 'absolute',
                    bottom: '95px',
                    left: 'calc(50% + 145px)',
                    pointerEvents: 'none',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '6px',
                    background: 'rgba(5, 10, 20, 0.90)',
                    backdropFilter: 'blur(14px)',
                    WebkitBackdropFilter: 'blur(14px)',
                    border: `1px solid ${goldRemaining === 0 ? '#10b981' : 'rgba(0, 210, 255, 0.4)'}`,
                    padding: '4px 9px',
                    borderRadius: '8px',
                    boxShadow: '0 4px 16px rgba(0,0,0,0.8)',
                  }}
                >
                  <img
                    src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/item/${targetCoreItem.id}.png`}
                    alt={targetCoreItem.name}
                    style={{ width: '20px', height: '20px', borderRadius: '4px', border: '1px solid #10b981' }}
                    onError={(e) => { (e.target as any).style.display = 'none'; }}
                  />
                  <div style={{ display: 'flex', flexDirection: 'column' }}>
                    <span style={{ fontSize: '8px', fontWeight: 800, color: '#94a3b8', textTransform: 'uppercase' }}>Target Goal</span>
                    <span style={{ fontSize: '10px', fontWeight: 900, color: goldRemaining === 0 ? '#10b981' : '#f8fafc' }}>
                      {goldRemaining === 0 ? '✓ Ready to Buy!' : `Need ${goldRemaining}g`}
                    </span>
                  </div>
                </div>
              )}

              {/* 6. ENEMY SPELL TRACKER DOCK (Pinned to left side) */}
              {settings?.hudSpellTimers !== false && (
                <div
                  style={{
                    position: 'absolute',
                    top: '28%',
                    left: '14px',
                    pointerEvents: 'auto',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '4px',
                    background: 'rgba(5, 10, 20, 0.88)',
                    backdropFilter: 'blur(14px)',
                    WebkitBackdropFilter: 'blur(14px)',
                    border: '1px solid rgba(255, 255, 255, 0.08)',
                    padding: '6px 8px',
                    borderRadius: '10px',
                    boxShadow: '0 6px 20px rgba(0,0,0,0.8)',
                    zIndex: 9998,
                  }}
                >
                  <div style={{ fontSize: '8.5px', fontWeight: 900, color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '2px' }}>
                    Enemy Spells
                  </div>
                  {enemySpells.slice(0, 4).map((sp, idx) => (
                    <button
                      key={idx}
                      onClick={() => handleTriggerSpell(idx)}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        gap: '6px',
                        padding: '3px 6px',
                        borderRadius: '4px',
                        background: sp.active ? 'rgba(255, 42, 95, 0.2)' : 'rgba(255, 255, 255, 0.04)',
                        border: sp.active ? '1px solid var(--red-primary)' : '1px solid transparent',
                        cursor: 'pointer',
                      }}
                      title={`Click to track ${sp.role} ${sp.name}`}
                    >
                      <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                        <span style={{ fontSize: '8px', fontWeight: 800, color: 'var(--blue-primary)', width: '20px', textAlign: 'left' }}>
                          {sp.role}
                        </span>
                        <img src={sp.icon} alt={sp.name} style={{ width: '14px', height: '14px', borderRadius: '2px' }} />
                      </div>
                      <span className="tabular-nums" style={{ fontSize: '9px', fontWeight: 800, color: sp.active ? 'var(--red-primary)' : '#10b981' }}>
                        {sp.active ? `${sp.remaining}s` : 'UP'}
                      </span>
                    </button>
                  ))}
                </div>
              )}
            </div>
        </>
      )}

      {/* MODE 1: ULTRA-SLIM HORIZONTAL IN-GAME HUD BAR (Takes 0 vertical space) */}
      {hudMode === 'bar' && (
        <div
          className="app-drag-region"
          onMouseDown={handleStartDrag}
          style={{
            width: '100%',
            height: '100%',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '0 10px',
            background: `rgba(6, 11, 20, ${overlayOpacity})`,
            backdropFilter: 'blur(20px) saturate(180%)',
            WebkitBackdropFilter: 'blur(20px) saturate(180%)',
            border: '1px solid rgba(0, 210, 255, 0.4)',
            boxShadow: '0 6px 24px rgba(0, 0, 0, 0.9), 0 0 16px rgba(0, 210, 255, 0.2)',
            color: '#f8fafc',
            overflow: 'hidden',
            boxSizing: 'border-box',
            cursor: 'grab',
            userSelect: 'none',
          }}
        >
          {/* Champ & Level */}
          <div className="app-no-drag" style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <div style={{ position: 'relative' }}>
              <img
                src={getChampionIcon(myChampObj.key)}
                alt={myChampObj.name}
                style={{ width: '26px', height: '26px', borderRadius: '5px', border: '1px solid var(--blue-primary)' }}
              />
              <span style={{ position: 'absolute', bottom: '-3px', right: '-3px', fontSize: '8px', fontWeight: 900, background: '#000', color: 'var(--blue-primary)', padding: '0 3px', borderRadius: '3px' }}>
                Lv{myLevel}
              </span>
            </div>
            <div style={{ fontSize: '11px', fontWeight: 800, color: '#fff' }}>
              {myChampObj.name}
            </div>
          </div>

          {/* Next Skill Upgrade Box */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '5px', background: 'rgba(0, 210, 255, 0.12)', padding: '2px 8px', borderRadius: '4px', border: '1px solid rgba(0, 210, 255, 0.35)' }}>
            <span style={{ fontSize: '9px', fontWeight: 800, color: '#94a3b8', textTransform: 'uppercase' }}>Next:</span>
            <span style={{ fontSize: '11px', fontWeight: 900, color: '#000', background: 'var(--blue-primary)', padding: '0 5px', borderRadius: '3px', boxShadow: '0 0 8px var(--blue-glow)' }}>
              {nextSkillKey}
            </span>
            <span style={{ fontSize: '10px', color: '#cbd5e1' }}>(Lv{Math.min(18, myLevel + 1)})</span>
          </div>

          {/* Max Priority */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '3px', fontSize: '10px', color: '#94a3b8' }}>
            <span style={{ fontWeight: 700 }}>Max:</span>
            <span style={{ fontWeight: 800, color: 'var(--blue-primary)' }}>{myChampBuild.skillPriority}</span>
          </div>

          {/* Item Target */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            <img
              src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/item/${targetCoreItem.id}.png`}
              alt={targetCoreItem.name}
              style={{ width: '22px', height: '22px', borderRadius: '3px', border: '1px solid #10b981' }}
              onError={(e) => { (e.target as any).style.display = 'none'; }}
            />
            <span style={{ fontSize: '10px', fontWeight: 700, color: goldRemaining === 0 ? '#10b981' : '#cbd5e1' }}>
              {goldRemaining === 0 ? '✓ Ready!' : `Need ${goldRemaining}g`}
            </span>
          </div>

          {/* CS / Min */}
          <div style={{ fontSize: '10px', fontWeight: 700, color: '#cbd5e1' }}>
            <span>{currentCs} CS</span> <span style={{ color: 'var(--blue-primary)' }}>({csm}/m)</span>
          </div>

          {/* Next Epic Objective */}
          <div style={{ fontSize: '10px', fontWeight: 800, color: objectives[0].remaining > 0 ? objectives[0].color : '#10b981', display: 'flex', alignItems: 'center', gap: '3px' }}>
            <span>{objectives[0].icon}</span>
            <span>{objectives[0].remaining > 0 ? `${objectives[0].remaining}s` : 'UP'}</span>
          </div>

          {/* Quick HUD Controls */}
          <div className="app-no-drag" style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
            <button
              onClick={handleSwitchToCard}
              title="Switch to Tactical Card Mode"
              style={{
                background: 'rgba(0, 210, 255, 0.15)',
                border: '1px solid rgba(0, 210, 255, 0.35)',
                color: 'var(--blue-primary)',
                cursor: 'pointer',
                padding: '2px 7px',
                borderRadius: '4px',
                fontSize: '9.5px',
                fontWeight: 800,
                display: 'flex',
                alignItems: 'center',
                gap: '3px',
              }}
            >
              <Columns size={11} />
              <span>Card</span>
            </button>
            <button
              onClick={handleCycleOpacity}
              title={`Overlay Opacity: ${Math.round(overlayOpacity * 100)}%`}
              style={{ background: 'transparent', border: 'none', color: '#94a3b8', cursor: 'pointer', padding: '3px' }}
            >
              <Eye size={12} />
            </button>
            <button
              onClick={handleCollapseToPill}
              title="Fold to Mini Pill"
              style={{ background: 'transparent', border: 'none', color: '#94a3b8', cursor: 'pointer', padding: '3px' }}
            >
              <ChevronDown size={14} />
            </button>
            <button
              onClick={onExpand}
              title="Expand to Full RiftCore App"
              style={{ background: 'transparent', border: 'none', color: '#94a3b8', cursor: 'pointer', padding: '3px' }}
            >
              <Maximize2 size={12} />
            </button>
          </div>
        </div>
      )}

      {/* MODE 2: MINI PILL SUMMARY (Minimalist pill) */}
      {hudMode === 'pill' && (
        <div
          className="app-drag-region"
          onMouseDown={handleStartDrag}
          style={{
            width: '100%',
            height: '100%',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '0 10px',
            background: `rgba(6, 11, 20, ${overlayOpacity})`,
            backdropFilter: 'blur(20px) saturate(180%)',
            WebkitBackdropFilter: 'blur(20px) saturate(180%)',
            border: '1px solid rgba(0, 210, 255, 0.4)',
            borderRadius: '20px',
            boxShadow: '0 6px 20px rgba(0, 0, 0, 0.85)',
            color: '#f8fafc',
            cursor: 'grab',
            userSelect: 'none',
            boxSizing: 'border-box',
          }}
        >
          <div className="app-no-drag" style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <img
              src={getChampionIcon(myChampObj.key)}
              alt={myChampObj.name}
              style={{ width: '22px', height: '22px', borderRadius: '50%', border: '1px solid var(--blue-primary)' }}
            />
            <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
              <span style={{ fontSize: '9px', color: '#94a3b8' }}>Next:</span>
              <span style={{ fontWeight: 900, color: '#000', background: 'var(--blue-primary)', padding: '0 5px', borderRadius: '3px', fontSize: '10.5px' }}>
                {nextSkillKey}
              </span>
            </div>
            <span style={{ fontSize: '10px', color: '#94a3b8' }}>CS {currentCs} ({csm}/m)</span>
          </div>

          <div className="app-no-drag" style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
            <button
              onClick={handleSwitchToCard}
              title="Unfold to Tactical Card"
              style={{
                background: 'rgba(0, 210, 255, 0.15)',
                border: '1px solid rgba(0, 210, 255, 0.3)',
                color: 'var(--blue-primary)',
                cursor: 'pointer',
                padding: '2px 7px',
                borderRadius: '10px',
                fontSize: '9px',
                fontWeight: 800,
              }}
            >
              ▲ Card
            </button>
            <button
              onClick={handleSwitchToBar}
              title="Unfold to Top HUD Bar"
              style={{
                background: 'rgba(255, 255, 255, 0.05)',
                border: '1px solid var(--border-subtle)',
                color: '#cbd5e1',
                cursor: 'pointer',
                padding: '2px 7px',
                borderRadius: '10px',
                fontSize: '9px',
                fontWeight: 800,
              }}
            >
              HUD Bar
            </button>
          </div>
        </div>
      )}

      {/* MODE 3: COMPACT TACTICAL CARD (340px × 270px, 0 empty dead space) */}
      {hudMode === 'card' && (
        <div
          className="glass-card"
          style={{
            width: '100%',
            height: '100%',
            background: `rgba(8, 13, 24, ${overlayOpacity})`,
            backdropFilter: 'blur(24px) saturate(180%)',
            WebkitBackdropFilter: 'blur(24px) saturate(180%)',
            border: '1px solid rgba(0, 210, 255, 0.35)',
            borderTop: '1px solid rgba(255, 255, 255, 0.25)',
            boxShadow: '0 16px 48px -4px rgba(0, 0, 0, 0.85), 0 0 24px rgba(0, 210, 255, 0.15)',
            display: 'flex',
            flexDirection: 'column',
            color: '#f8fafc',
            overflow: 'hidden',
            padding: 0,
            boxSizing: 'border-box',
          }}
        >
          {/* DRAGGABLE OVERLAY HEADER BAR */}
          <div
            className="app-drag-region"
            onMouseDown={handleStartDrag}
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              padding: '6px 10px',
              background: 'linear-gradient(90deg, rgba(14, 22, 38, 0.95) 0%, rgba(8, 13, 24, 0.95) 100%)',
              borderBottom: '1px solid var(--border-subtle)',
              cursor: 'grab',
              userSelect: 'none',
            }}
          >
            {/* Matchup Identity */}
            <div className="app-no-drag" style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <div style={{ position: 'relative' }}>
                <img
                  src={getChampionIcon(myChampObj.key)}
                  alt={myChampObj.name}
                  style={{ width: '26px', height: '26px', borderRadius: '5px', border: '1px solid var(--blue-primary)' }}
                />
                <span
                  style={{
                    position: 'absolute',
                    bottom: '-3px',
                    right: '-3px',
                    fontSize: '8px',
                    fontWeight: 900,
                    background: '#000',
                    color: 'var(--blue-primary)',
                    padding: '0 3px',
                    borderRadius: '3px',
                  }}
                >
                  Lv{myLevel}
                </span>
              </div>

              <div style={{ fontSize: '11px', fontWeight: 800, color: '#fff', display: 'flex', alignItems: 'center', gap: '4px' }}>
                <span>{myChampObj.name}</span>
                <span style={{ fontSize: '9px', color: '#64748b' }}>vs</span>
                <span style={{ color: 'var(--red-primary)' }}>{opponentChampObj.name}</span>
              </div>

              <span
                className="tabular-nums"
                style={{
                  fontSize: '9.5px',
                  fontWeight: 800,
                  color: 'var(--blue-primary)',
                  background: 'rgba(0, 210, 255, 0.1)',
                  padding: '1px 5px',
                  borderRadius: '4px',
                  border: '1px solid rgba(0, 210, 255, 0.25)',
                }}
              >
                {formatMinSec(gameTime)}
              </span>
            </div>

            {/* Action Controls */}
            <div className="app-no-drag" style={{ display: 'flex', alignItems: 'center', gap: '3px' }}>
              {/* Switch to True In-Game HUD */}
              <button
                onClick={handleSwitchToIntegrated}
                title="Switch to True In-Game HUD (Anchored directly over League elements)"
                style={{
                  background: 'rgba(0, 210, 255, 0.18)',
                  border: '1px solid var(--border-blue)',
                  color: 'var(--blue-primary)',
                  cursor: 'pointer',
                  padding: '2px 6px',
                  borderRadius: '4px',
                  fontSize: '9px',
                  fontWeight: 800,
                  display: 'flex',
                  alignItems: 'center',
                  gap: '3px',
                }}
              >
                <Sparkles size={10} />
                <span>In-Game HUD</span>
              </button>

              {/* Switch to Top HUD Bar */}
              <button
                onClick={handleSwitchToBar}
                title="Switch to Ultra-Slim Top HUD Bar"
                style={{
                  background: 'rgba(0, 210, 255, 0.12)',
                  border: '1px solid rgba(0, 210, 255, 0.25)',
                  color: 'var(--blue-primary)',
                  cursor: 'pointer',
                  padding: '2px 5px',
                  borderRadius: '4px',
                  fontSize: '9px',
                  fontWeight: 800,
                  display: 'flex',
                  alignItems: 'center',
                  gap: '2px',
                }}
              >
                <PanelTop size={11} />
                <span>Bar</span>
              </button>

              {/* Opacity Cycle */}
              <button
                onClick={handleCycleOpacity}
                style={{
                  background: 'transparent',
                  border: 'none',
                  color: overlayOpacity < 0.9 ? 'var(--blue-primary)' : '#94a3b8',
                  cursor: 'pointer',
                  padding: '3px 4px',
                  borderRadius: '4px',
                }}
                title={`Overlay Opacity: ${Math.round(overlayOpacity * 100)}% (Click to toggle)`}
              >
                <Eye size={12} />
              </button>

              {/* Collapse to Pill */}
              <button
                onClick={handleCollapseToPill}
                style={{
                  background: 'transparent',
                  border: 'none',
                  color: '#94a3b8',
                  cursor: 'pointer',
                  padding: '3px 4px',
                  borderRadius: '4px',
                }}
                title="Fold to Mini Pill"
              >
                <ChevronDown size={14} />
              </button>

              {/* Demo Mode Toggle */}
              <button
                onClick={() => setIsPreviewMode(!isPreviewMode)}
                style={{
                  fontSize: '8.5px',
                  fontWeight: 800,
                  padding: '2px 4px',
                  borderRadius: '4px',
                  background: isDemoMode ? 'rgba(245, 158, 11, 0.2)' : 'rgba(255, 255, 255, 0.05)',
                  border: isDemoMode ? '1px solid #f59e0b' : '1px solid var(--border-subtle)',
                  color: isDemoMode ? '#f59e0b' : '#94a3b8',
                  cursor: 'pointer',
                }}
                title="Toggle Live Demo Simulation"
              >
                {isDemoMode ? '⚡ Demo' : 'Live'}
              </button>

              {/* Pin Toggle */}
              <button
                onClick={handleTogglePin}
                title={isPinned ? 'Always on Top (Active)' : 'Pin on Top'}
                style={{
                  background: isPinned ? 'rgba(0, 210, 255, 0.18)' : 'transparent',
                  border: 'none',
                  color: isPinned ? 'var(--blue-primary)' : '#94a3b8',
                  cursor: 'pointer',
                  padding: '3px 4px',
                  borderRadius: '4px',
                }}
              >
                <Pin size={12} />
              </button>

              {/* Expand to desktop / Minimize */}
              <button
                onClick={onExpand}
                title="Expand to Full RiftCore App"
                style={{ background: 'transparent', border: 'none', color: '#94a3b8', cursor: 'pointer', padding: '3px 4px' }}
              >
                <Maximize2 size={12} />
              </button>
              <button
                onClick={onMinimize}
                title="Minimize HUD"
                style={{ background: 'transparent', border: 'none', color: '#94a3b8', cursor: 'pointer', padding: '3px 4px' }}
              >
                <Minus size={13} />
              </button>
            </div>
          </div>

          {/* OVERLAY TABS BAR */}
          <div
            style={{
              display: 'flex',
              background: 'rgba(5, 8, 15, 0.95)',
              borderBottom: '1px solid var(--border-subtle)',
            }}
          >
            {[
              { id: 'skills', label: 'Skills', icon: Zap },
              { id: 'build', label: 'Build', icon: ShoppingBag },
              { id: 'cards', label: 'Cards', icon: Sparkles },
              { id: 'matchup', label: 'Matchup', icon: Swords },
              { id: 'stats', label: 'CS / Gold', icon: TrendingUp },
              { id: 'timers', label: 'Timers', icon: Timer },
            ].map((t) => {
              const Icon = t.icon;
              const isActive = activeTab === t.id;
              return (
                <button
                  key={t.id}
                  onClick={() => setActiveTab(t.id as OverlayTab)}
                  style={{
                    flex: 1,
                    padding: '6px 0',
                    fontSize: '9.5px',
                    fontWeight: 800,
                    textTransform: 'uppercase',
                    letterSpacing: '0.4px',
                    background: isActive ? 'rgba(0, 210, 255, 0.12)' : 'transparent',
                    color: isActive ? 'var(--blue-primary)' : '#64748b',
                    border: 'none',
                    borderBottom: isActive ? '2px solid var(--blue-primary)' : '2px solid transparent',
                    cursor: 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    gap: '4px',
                    transition: 'all 0.15s ease',
                  }}
                >
                  <Icon size={11} />
                  <span>{t.label}</span>
                </button>
              );
            })}
          </div>

          {/* TAB 1: SKILLS & LEVEL UP */}
          {activeTab === 'skills' && (
            <div style={{ flex: 1, padding: '10px 12px', display: 'flex', flexDirection: 'column', gap: '8px', overflowY: 'auto' }}>
              {/* Next Skill Upgrade Banner */}
              <div
                style={{
                  padding: '8px 10px',
                  borderRadius: '7px',
                  background: 'linear-gradient(135deg, rgba(0, 210, 255, 0.15) 0%, rgba(14, 165, 233, 0.08) 100%)',
                  border: '1px solid var(--blue-primary)',
                  boxShadow: '0 0 12px var(--blue-glow-subtle)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                }}
              >
                <div>
                  <div style={{ fontSize: '8.5px', fontWeight: 800, textTransform: 'uppercase', color: '#94a3b8', letterSpacing: '0.5px' }}>
                    Next Skill to Level Up:
                  </div>
                  <div style={{ fontSize: '12.5px', fontWeight: 900, color: '#fff', marginTop: '1px' }}>
                    Level {Math.min(18, myLevel + 1)} → <span style={{ color: 'var(--blue-primary)' }}>Upgrade {nextSkillKey}</span>
                  </div>
                </div>
                <div
                  style={{
                    width: '30px',
                    height: '30px',
                    borderRadius: '7px',
                    background: 'var(--blue-primary)',
                    color: '#000',
                    fontSize: '15px',
                    fontWeight: 900,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    boxShadow: '0 0 10px var(--blue-glow)',
                  }}
                >
                  {nextSkillKey}
                </div>
              </div>

              {/* Skill Max Priority */}
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '5px 8px', background: 'rgba(255, 255, 255, 0.03)', borderRadius: '5px', border: '1px solid var(--border-subtle)' }}>
                <span style={{ fontSize: '10px', color: '#94a3b8', fontWeight: 700 }}>Skill Max Priority:</span>
                <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                  {myChampBuild.skillPriority.split('>').map((s, idx, arr) => {
                    const key = s.trim();
                    return (
                      <React.Fragment key={key + idx}>
                        <span style={{ fontSize: '10.5px', fontWeight: 900, color: 'var(--blue-primary)', background: 'rgba(0, 210, 255, 0.1)', padding: '1px 5px', borderRadius: '3px', border: '1px solid rgba(0, 210, 255, 0.3)' }}>
                          {key}
                        </span>
                        {idx < arr.length - 1 && <ChevronRight size={10} color="#64748b" />}
                      </React.Fragment>
                    );
                  })}
                </div>
              </div>

              {/* Level 1-18 Skill Progression */}
              <div>
                <div style={{ fontSize: '9px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b', marginBottom: '4px' }}>
                  Level 1–18 Skill Progression:
                </div>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(9, 1fr)', gap: '3px' }}>
                  {myChampBuild.skillOrder.slice(0, 18).map((sk, idx) => {
                    const lvl = idx + 1;
                    const isCurrent = lvl === myLevel;
                    const isNext = lvl === myLevel + 1;
                    const isPast = lvl <= myLevel;

                    return (
                      <div
                        key={lvl}
                        style={{
                          padding: '3px 1px',
                          borderRadius: '4px',
                          textAlign: 'center',
                          background: isCurrent
                            ? 'var(--blue-primary)'
                            : isNext
                            ? 'rgba(0, 210, 255, 0.2)'
                            : isPast
                            ? 'rgba(255, 255, 255, 0.08)'
                            : 'rgba(255, 255, 255, 0.02)',
                          border: isCurrent
                            ? '1px solid #fff'
                            : isNext
                            ? '1px solid var(--blue-primary)'
                            : '1px solid rgba(255, 255, 255, 0.05)',
                          color: isCurrent ? '#000' : isPast ? '#fff' : '#64748b',
                        }}
                      >
                        <div style={{ fontSize: '7.5px', fontWeight: 800 }}>{lvl}</div>
                        <div style={{ fontSize: '9.5px', fontWeight: 900 }}>{sk}</div>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>
          )}

          {/* TAB 2: BUILD & ITEMS */}
          {activeTab === 'build' && (
            <div style={{ flex: 1, padding: '10px 12px', display: 'flex', flexDirection: 'column', gap: '8px', overflowY: 'auto' }}>
              {/* Next Item Target */}
              <div
                style={{
                  padding: '8px 10px',
                  borderRadius: '7px',
                  background: 'linear-gradient(135deg, rgba(16, 185, 129, 0.12) 0%, rgba(6, 10, 18, 0.8) 100%)',
                  border: '1px solid rgba(16, 185, 129, 0.4)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <img
                    src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/item/${targetCoreItem.id}.png`}
                    alt={targetCoreItem.name}
                    style={{ width: '32px', height: '32px', borderRadius: '5px', border: '1px solid #10b981' }}
                    onError={(e) => { (e.target as any).style.display = 'none'; }}
                  />
                  <div>
                    <div style={{ fontSize: '8.5px', fontWeight: 800, textTransform: 'uppercase', color: '#10b981' }}>
                      Next Item Target:
                    </div>
                    <div style={{ fontSize: '11.5px', fontWeight: 800, color: '#fff' }}>
                      {targetCoreItem.name}
                    </div>
                    <div className="tabular-nums" style={{ fontSize: '9.5px', color: '#94a3b8' }}>
                      {goldRemaining === 0 ? '✓ Ready in Base!' : `Need ${goldRemaining}g (${currentGold}/${targetCost}g)`}
                    </div>
                  </div>
                </div>

                <button
                  onClick={handleExportShop}
                  style={{
                    padding: '4px 8px',
                    borderRadius: '5px',
                    background: shopExportStatus === 'success' ? 'rgba(16, 185, 129, 0.2)' : 'rgba(0, 210, 255, 0.15)',
                    border: shopExportStatus === 'success' ? '1px solid #10b981' : '1px solid var(--border-blue)',
                    color: shopExportStatus === 'success' ? '#10b981' : 'var(--blue-primary)',
                    fontSize: '9.5px',
                    fontWeight: 800,
                    cursor: 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '4px',
                  }}
                  title="Push Item Set into LoL in-game shop (Press P in game)"
                >
                  {shopExportStatus === 'success' ? <Check size={10} /> : <Zap size={10} />}
                  <span>{shopExportStatus === 'success' ? 'Synced' : 'Push Shop'}</span>
                </button>
              </div>

              {/* Core Items Chain */}
              <div>
                <div style={{ fontSize: '9px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b', marginBottom: '4px' }}>
                  Core Rush Build Path:
                </div>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '6px' }}>
                  {myChampBuild.coreItems.slice(0, 3).map((item, idx) => (
                    <div
                      key={item.id}
                      style={{
                        background: 'rgba(255, 255, 255, 0.03)',
                        padding: '5px',
                        borderRadius: '5px',
                        border: '1px solid var(--border-subtle)',
                        display: 'flex',
                        flexDirection: 'column',
                        alignItems: 'center',
                        textAlign: 'center',
                        gap: '3px',
                      }}
                    >
                      <div style={{ position: 'relative' }}>
                        <img
                          src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/item/${item.id}.png`}
                          alt={item.name}
                          style={{ width: '26px', height: '26px', borderRadius: '4px' }}
                          onError={(e) => { (e.target as any).style.display = 'none'; }}
                        />
                        <span
                          style={{
                            position: 'absolute',
                            bottom: '-3px',
                            right: '-3px',
                            background: 'var(--blue-primary)',
                            color: '#000',
                            fontSize: '7.5px',
                            fontWeight: 900,
                            borderRadius: '50%',
                            width: '11px',
                            height: '11px',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                          }}
                        >
                          {idx + 1}
                        </span>
                      </div>
                      <span style={{ fontSize: '9px', fontWeight: 700, color: '#cbd5e1', lineHeight: '1.2' }}>
                        {item.name}
                      </span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Starting & Boots */}
              <div style={{ display: 'flex', gap: '6px' }}>
                <div style={{ flex: 1 }}>
                  <div style={{ fontSize: '8.5px', fontWeight: 800, color: '#64748b', textTransform: 'uppercase', marginBottom: '3px' }}>
                    Starting
                  </div>
                  <div style={{ display: 'flex', gap: '4px' }}>
                    {myChampBuild.startingItems.map((item) => (
                      <div key={item.id} style={{ display: 'flex', alignItems: 'center', gap: '3px', background: 'rgba(255,255,255,0.03)', padding: '3px 5px', borderRadius: '4px', border: '1px solid var(--border-subtle)', fontSize: '9.5px' }}>
                        <img src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/item/${item.id}.png`} alt={item.name} style={{ width: '16px', height: '16px', borderRadius: '3px' }} onError={(e) => { (e.target as any).style.display = 'none'; }} />
                        <span style={{ color: '#cbd5e1' }}>{item.name}</span>
                      </div>
                    ))}
                  </div>
                </div>

                <div>
                  <div style={{ fontSize: '8.5px', fontWeight: 800, color: '#64748b', textTransform: 'uppercase', marginBottom: '3px' }}>
                    Boots
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '3px', background: 'rgba(255,255,255,0.03)', padding: '3px 5px', borderRadius: '4px', border: '1px solid var(--border-subtle)', fontSize: '9.5px' }}>
                    <img src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/item/${myChampBuild.boots.id}.png`} alt={myChampBuild.boots.name} style={{ width: '16px', height: '16px', borderRadius: '3px' }} onError={(e) => { (e.target as any).style.display = 'none'; }} />
                    <span style={{ color: '#cbd5e1' }}>{myChampBuild.boots.name}</span>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* TAB: MAYHEM & ARENA CARDS ADVICE */}
          {activeTab === 'cards' && (
            <div style={{ flex: 1, padding: '8px 10px', display: 'flex', flexDirection: 'column', gap: '6px', overflow: 'hidden' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '6px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                  <Sparkles size={11} color="#c084fc" />
                  <span style={{ fontSize: '10px', fontWeight: 800, color: '#fff' }}>Mayhem Cards</span>
                </div>
                <div style={{ display: 'flex', gap: '2px' }}>
                  {(['ALL', 'Prismatic', 'Gold', 'Silver'] as const).map((tier) => (
                    <button
                      key={tier}
                      onClick={() => setHudCardTierFilter(tier)}
                      style={{
                        fontSize: '7.5px',
                        fontWeight: 800,
                        padding: '1px 5px',
                        borderRadius: '3px',
                        background: hudCardTierFilter === tier ? 'rgba(255,255,255,0.18)' : 'rgba(255,255,255,0.03)',
                        border: hudCardTierFilter === tier ? '1px solid rgba(255,255,255,0.3)' : '1px solid transparent',
                        color: tier === 'Prismatic' ? '#c084fc' : tier === 'Gold' ? '#f59e0b' : tier === 'Silver' ? '#94a3b8' : '#fff',
                        cursor: 'pointer',
                      }}
                    >
                      {tier === 'Prismatic' ? 'Prism' : tier}
                    </button>
                  ))}
                </div>
              </div>

              {/* Compact Search */}
              <input
                type="text"
                placeholder="Search Mayhem cards (Crit, Haste, Bleed)..."
                value={hudCardSearch}
                onChange={(e) => setHudCardSearch(e.target.value)}
                style={{
                  padding: '3px 8px',
                  borderRadius: '4px',
                  background: 'rgba(0,0,0,0.4)',
                  border: '1px solid var(--border-subtle)',
                  color: '#fff',
                  fontSize: '10px',
                  width: '100%',
                  boxSizing: 'border-box',
                }}
              />

              {/* Card List */}
              <div style={{ flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '5px', paddingRight: '2px' }}>
                {searchMayhemCards(hudCardSearch, hudCardTierFilter, 'Both').map((card) => (
                  <div
                    key={card.id}
                    style={{
                      background: 'rgba(255,255,255,0.02)',
                      border: `1px solid ${card.color}33`,
                      borderLeft: `3px solid ${card.color}`,
                      borderRadius: '5px',
                      padding: '5px 8px',
                      display: 'flex',
                      flexDirection: 'column',
                      gap: '2px',
                    }}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                        <span style={{ fontSize: '10.5px', fontWeight: 800, color: '#fff' }}>{card.name}</span>
                        <span
                          style={{
                            fontSize: '7.5px',
                            fontWeight: 800,
                            padding: '0 4px',
                            borderRadius: '2px',
                            background: `${card.color}20`,
                            color: card.color,
                          }}
                        >
                          {card.tier}
                        </span>
                        <span
                          style={{
                            fontSize: '7.5px',
                            fontWeight: 900,
                            padding: '0 4px',
                            borderRadius: '2px',
                            background: card.rank === 'S+' ? 'rgba(239,68,68,0.25)' : 'rgba(34,197,94,0.25)',
                            color: card.rank === 'S+' ? '#ef4444' : '#22c55e',
                          }}
                        >
                          {card.rank}
                        </span>
                      </div>
                      <span style={{ fontSize: '7.5px', color: '#64748b' }}>{card.tag}</span>
                    </div>
                    <div style={{ fontSize: '9px', color: '#cbd5e1', lineHeight: '1.25' }}>
                      {card.description}
                    </div>
                    <div style={{ fontSize: '8px', color: '#94a3b8' }}>
                      <span style={{ color: 'var(--blue-primary)', fontWeight: 700 }}>Best:</span> {card.archetype}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* TAB 3: LANE MATCHUP */}
          {activeTab === 'matchup' && (
            <div style={{ flex: 1, padding: '10px 12px', display: 'flex', flexDirection: 'column', gap: '8px', overflowY: 'auto' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '8px 10px', background: 'rgba(255, 42, 95, 0.08)', borderRadius: '6px', border: '1px solid rgba(255, 42, 95, 0.25)' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <img
                    src={getChampionIcon(opponentChampObj.key)}
                    alt={opponentChampObj.name}
                    style={{ width: '32px', height: '32px', borderRadius: '6px', border: '1px solid var(--red-primary)' }}
                  />
                  <div>
                    <div style={{ fontSize: '11.5px', fontWeight: 800, color: '#fff' }}>
                      vs {opponentChampObj.name} ({opponentBuild.role})
                    </div>
                    <div style={{ fontSize: '9.5px', color: '#94a3b8' }}>
                      Winrate: <strong style={{ color: 'var(--blue-primary)' }}>52.8% Favored</strong>
                    </div>
                  </div>
                </div>
                <span className="tier-badge tier-s" style={{ fontSize: '9px', padding: '2px 6px' }}>S Tier</span>
              </div>

              <div>
                <div style={{ fontSize: '9px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b', marginBottom: '4px' }}>
                  Key Strategy:
                </div>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                  {myChampBuild.tips.slice(0, 2).map((tip, idx) => (
                    <div
                      key={idx}
                      style={{
                        display: 'flex',
                        gap: '6px',
                        padding: '6px 8px',
                        borderRadius: '5px',
                        background: 'rgba(255, 255, 255, 0.03)',
                        border: '1px solid var(--border-subtle)',
                        fontSize: '10px',
                        color: '#cbd5e1',
                        lineHeight: '1.35',
                      }}
                    >
                      <span style={{ color: 'var(--blue-primary)', fontWeight: 900 }}>•</span>
                      <span>{tip}</span>
                    </div>
                  ))}
                </div>
              </div>

              <div style={{ padding: '6px 8px', borderRadius: '5px', background: 'rgba(245, 158, 11, 0.08)', border: '1px solid rgba(245, 158, 11, 0.25)', fontSize: '9.5px', color: '#f59e0b' }}>
                ⚠️ <strong>Power Spike:</strong> {opponentChampObj.name} hits massive kill threat at Level 6.
              </div>
            </div>
          )}

          {/* TAB 4: CS & GOLD BENCHMARKS */}
          {activeTab === 'stats' && (
            <div style={{ flex: 1, padding: '10px 12px', display: 'flex', flexDirection: 'column', gap: '6px', overflowY: 'auto' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '6px 8px', background: 'rgba(255,255,255,0.03)', borderRadius: '5px', border: '1px solid var(--border-subtle)' }}>
                <div>
                  <div style={{ fontSize: '9px', fontWeight: 800, color: '#94a3b8' }}>CREEP SCORE (CS)</div>
                  <div className="tabular-nums" style={{ fontSize: '13px', fontWeight: 900, color: '#fff' }}>
                    {currentCs} CS <span style={{ fontSize: '10px', color: 'var(--blue-primary)' }}>({csm} /min)</span>
                  </div>
                </div>
                <div style={{ textAlign: 'right' }}>
                  <span style={{ fontSize: '10px', fontWeight: 800, color: isCsmAhead ? '#10b981' : '#ff2a5f' }}>
                    {isCsmAhead ? `+${csmDelta} Ahead` : `${csmDelta} Behind`}
                  </span>
                  <div style={{ fontSize: '8.5px', color: '#64748b' }}>Goal: 7.5/m</div>
                </div>
              </div>

              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '6px 8px', background: 'rgba(255,255,255,0.03)', borderRadius: '5px', border: '1px solid var(--border-subtle)' }}>
                <div>
                  <div style={{ fontSize: '9px', fontWeight: 800, color: '#94a3b8' }}>GOLD / MIN (GPM)</div>
                  <div className="tabular-nums" style={{ fontSize: '13px', fontWeight: 900, color: '#fff' }}>
                    {gpm} GPM
                  </div>
                </div>
                <div style={{ textAlign: 'right' }}>
                  <span style={{ fontSize: '10px', fontWeight: 800, color: 'var(--blue-primary)' }}>
                    {currentGold}g Gold
                  </span>
                  <div style={{ fontSize: '8.5px', color: '#64748b' }}>Avg: 390</div>
                </div>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '6px' }}>
                <div style={{ padding: '6px 8px', background: 'rgba(255,255,255,0.03)', borderRadius: '5px', border: '1px solid var(--border-subtle)' }}>
                  <div style={{ fontSize: '9px', fontWeight: 800, color: '#94a3b8' }}>KDA RATIO</div>
                  <div className="tabular-nums" style={{ fontSize: '12px', fontWeight: 900, color: '#fff' }}>
                    {kdaRatio}:1 <span style={{ fontSize: '9px', color: '#94a3b8' }}>({myKills}/{myDeaths}/{myAssists})</span>
                  </div>
                </div>

                <div style={{ padding: '6px 8px', background: 'rgba(255,255,255,0.03)', borderRadius: '5px', border: '1px solid var(--border-subtle)' }}>
                  <div style={{ fontSize: '9px', fontWeight: 800, color: '#94a3b8' }}>KILL PARTICIPATION</div>
                  <div className="tabular-nums" style={{ fontSize: '12px', fontWeight: 900, color: '#10b981' }}>
                    {kp}% <span style={{ fontSize: '9px', color: '#94a3b8' }}>({teamKills} total)</span>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* TAB 5: SPELLS & OBJECTIVES */}
          {activeTab === 'timers' && (
            <div style={{ flex: 1, padding: '10px 12px', display: 'flex', flexDirection: 'column', gap: '6px', overflowY: 'auto' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <span style={{ fontSize: '9px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b' }}>
                  Enemy Spells (Click to track):
                </span>
                <button
                  onClick={handleCopyTimers}
                  style={{
                    fontSize: '9px',
                    fontWeight: 800,
                    padding: '2px 6px',
                    borderRadius: '4px',
                    background: 'rgba(255, 255, 255, 0.05)',
                    border: '1px solid var(--border-subtle)',
                    color: copiedChat ? '#10b981' : 'var(--blue-primary)',
                    cursor: 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '3px',
                  }}
                >
                  {copiedChat ? <Check size={9} /> : <Copy size={9} />}
                  <span>{copiedChat ? 'Copied!' : 'Copy'}</span>
                </button>
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                {enemySpells.slice(0, 3).map((sp, idx) => (
                  <button
                    key={idx}
                    onClick={() => handleTriggerSpell(idx)}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      padding: '4px 8px',
                      borderRadius: '5px',
                      background: sp.active ? 'rgba(255, 42, 95, 0.14)' : 'rgba(255, 255, 255, 0.03)',
                      border: sp.active ? '1px solid var(--red-primary)' : '1px solid var(--border-subtle)',
                      cursor: 'pointer',
                      transition: 'all 0.15s ease',
                    }}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                      <span style={{ fontSize: '9px', fontWeight: 800, color: 'var(--blue-primary)', width: '24px', textAlign: 'left' }}>
                        {sp.role}
                      </span>
                      <img src={sp.icon} alt={sp.name} style={{ width: '16px', height: '16px', borderRadius: '3px' }} />
                      <span style={{ fontSize: '10px', fontWeight: 700, color: '#fff' }}>
                        {sp.name}
                      </span>
                    </div>

                    <div className="tabular-nums" style={{ fontSize: '10px', fontWeight: 800, color: sp.active ? 'var(--red-primary)' : '#10b981' }}>
                      {sp.active ? `${sp.remaining}s` : 'READY'}
                    </div>
                  </button>
                ))}
              </div>

              <div>
                <div style={{ fontSize: '9px', fontWeight: 800, textTransform: 'uppercase', color: '#64748b', marginBottom: '3px' }}>
                  Objectives:
                </div>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '4px' }}>
                  {objectives.map((obj) => (
                    <div
                      key={obj.name}
                      style={{
                        padding: '4px',
                        borderRadius: '4px',
                        background: 'rgba(255, 255, 255, 0.03)',
                        border: '1px solid var(--border-subtle)',
                        textAlign: 'center',
                      }}
                    >
                      <div style={{ fontSize: '11px' }}>{obj.icon}</div>
                      <div style={{ fontSize: '8.5px', fontWeight: 800, color: '#cbd5e1' }}>{obj.name}</div>
                      <div className="tabular-nums" style={{ fontSize: '9.5px', fontWeight: 800, color: obj.remaining > 0 ? obj.color : '#10b981' }}>
                        {obj.remaining > 0 ? `${obj.remaining}s` : 'UP'}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
