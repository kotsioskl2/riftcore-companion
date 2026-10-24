import React, { useState, useEffect } from 'react';
import {
  Crosshair,
  Clock,
  Copy,
  Check,
  ShieldAlert,
  Play,
  RotateCcw,
  Sparkles,
  Award,
  TrendingUp,
  Flame,
  Swords,
  Timer,
  Volume2,
  VolumeX,
  Compass,
  Info,
  Layers,
  Zap,
  Heart,
  Search,
  Maximize2,
  Minimize2,
  Pin,
  ExternalLink,
  AlertTriangle,
  HelpCircle,
  Eye,
  Monitor,
  Keyboard,
  ShieldCheck,
} from 'lucide-react';
import { LiveGameData, AppSettings } from '../types';
import { soundService } from '../services/soundService';
import { MAYHEM_CARDS, searchMayhemCards, CardTier } from '../services/mayhemCardsData';

interface LiveGameOverlayProps {
  liveData: LiveGameData | null;
  settings?: AppSettings;
  isCompactMode?: boolean;
  isPinned?: boolean;
  onToggleCompact?: () => void;
  onTogglePin?: () => void;
}

type ActiveGameMode = 'CLASSIC' | 'ARAM' | 'CHERRY' | 'URF' | 'ONEFORALL';

interface SpellTracker {
  champion: string;
  role: string;
  spellId: number;
  spellName: string;
  cooldownTotal: number;
  cooldownRemaining: number;
  timerActive: boolean;
  readyGameTime: number;
}

interface RelicTimer {
  id: string;
  name: string;
  side: 'Ally' | 'Enemy';
  respawnTime: number;
  active: boolean;
}

interface AugmentItem {
  name: string;
  tier: 'Prismatic' | 'Gold' | 'Silver';
  rank: 'S+' | 'S' | 'A';
  role: string;
  description: string;
}

export const LiveGameOverlay: React.FC<LiveGameOverlayProps> = ({
  liveData,
  settings,
  isCompactMode = false,
  isPinned = false,
  onToggleCompact,
  onTogglePin,
}) => {
  const isLive = liveData !== null && liveData.gameData !== undefined;
  const [isDemoMode, setIsDemoMode] = useState(false);
  const [demoGameTime, setDemoGameTime] = useState(0);
  const [copied, setCopied] = useState(false);
  const [soundAlerts, setSoundAlerts] = useState(true);
  const [showFullscreenGuide, setShowFullscreenGuide] = useState(false);

  // Active Game Mode
  const [activeMode, setActiveMode] = useState<ActiveGameMode>('CLASSIC');
  const [augmentSearch, setAugmentSearch] = useState('');
  const [auraSearch, setAuraSearch] = useState('');
  const [aramCardSearch, setAramCardSearch] = useState('');
  const [aramCardTierFilter, setAramCardTierFilter] = useState<CardTier | 'ALL'>('ALL');
  const [srActiveTab, setSrActiveTab] = useState<'epics' | 'jungle'>('epics');
  const [jungleStart, setJungleStart] = useState<'bot' | 'top' | 'solo'>('bot');

  // Detect mode from live client data
  useEffect(() => {
    if (liveData?.gameData?.gameMode) {
      const modeStr = liveData.gameData.gameMode.toUpperCase();
      if (modeStr.includes('ARAM')) {
        setActiveMode('ARAM');
      } else if (modeStr.includes('CHERRY') || modeStr.includes('ARENA')) {
        setActiveMode('CHERRY');
      } else if (modeStr.includes('URF')) {
        setActiveMode('URF');
      } else if (modeStr.includes('ONEFORALL') || modeStr.includes('OFA')) {
        setActiveMode('ONEFORALL');
      } else {
        setActiveMode('CLASSIC');
      }
    }
  }, [liveData]);

  // Active game clock in seconds
  const activeGameTime = isLive
    ? Math.floor(liveData.gameData.gameTime)
    : isDemoMode
    ? demoGameTime
    : 0;

  // 1. Summoner's Rift Spells
  const [srEnemySpells, setSrEnemySpells] = useState<SpellTracker[]>([
    { champion: 'Darius', role: 'TOP', spellId: 4, spellName: 'Flash', cooldownTotal: 300, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Darius', role: 'TOP', spellId: 6, spellName: 'Ghost', cooldownTotal: 240, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Lee Sin', role: 'JUNGLE', spellId: 4, spellName: 'Flash', cooldownTotal: 300, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Lee Sin', role: 'JUNGLE', spellId: 11, spellName: 'Smite', cooldownTotal: 90, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Ahri', role: 'MID', spellId: 4, spellName: 'Flash', cooldownTotal: 300, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Ahri', role: 'MID', spellId: 12, spellName: 'Teleport', cooldownTotal: 360, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Jinx', role: 'ADC', spellId: 4, spellName: 'Flash', cooldownTotal: 300, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Jinx', role: 'ADC', spellId: 7, spellName: 'Heal', cooldownTotal: 240, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Thresh', role: 'SUPPORT', spellId: 4, spellName: 'Flash', cooldownTotal: 300, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
    { champion: 'Thresh', role: 'SUPPORT', spellId: 14, spellName: 'Ignite', cooldownTotal: 180, cooldownRemaining: 0, timerActive: false, readyGameTime: 0 },
  ]);

  // 1. Summoner's Rift Epics
  const [srObjectives, setSrObjectives] = useState([
    { id: 'dragon', name: 'Elemental Dragon', respawnTime: 300, total: 300, icon: '🐲', color: '#00c8ff' },
    { id: 'grubs', name: 'Voidgrubs / Herald', respawnTime: 300, total: 360, icon: '👾', color: '#ff2a5f' },
    { id: 'baron', name: 'Baron Nashor', respawnTime: 1200, total: 1200, icon: '👑', color: '#c084fc' },
  ]);

  // 1. Summoner's Rift Camps
  const [srJungleCamps, setSrJungleCamps] = useState([
    { name: 'Blue Buff (Sentinel)', icon: '🔷', respawnDuration: 300, remaining: 0, active: false },
    { name: 'Red Buff (Brambleback)', icon: '🔴', respawnDuration: 300, remaining: 0, active: false },
    { name: 'Raptors', icon: '🦅', respawnDuration: 135, remaining: 0, active: false },
    { name: 'Murkwolves', icon: '🐺', respawnDuration: 135, remaining: 0, active: false },
    { name: 'Gromp', icon: '🐸', respawnDuration: 135, remaining: 0, active: false },
    { name: 'Ancient Krugs', icon: '🗿', respawnDuration: 135, remaining: 0, active: false },
    { name: 'Rift Scuttler', icon: '🦀', respawnDuration: 150, remaining: 0, active: false },
  ]);

  // 2. ARAM Relics & Spells
  const [aramRelics, setAramRelics] = useState<RelicTimer[]>([
    { id: 'ally-outer', name: 'Ally Outer Shrine', side: 'Ally', respawnTime: 0, active: false },
    { id: 'ally-inner', name: 'Ally Inner Shrine', side: 'Ally', respawnTime: 0, active: false },
    { id: 'enemy-inner', name: 'Enemy Inner Shrine', side: 'Enemy', respawnTime: 0, active: false },
    { id: 'enemy-outer', name: 'Enemy Outer Shrine', side: 'Enemy', respawnTime: 0, active: false },
  ]);

  const [aramInhibitor, setAramInhibitor] = useState<{ remaining: number; active: boolean }>({
    remaining: 0,
    active: false,
  });

  const [aramSpells, setAramSpells] = useState([
    { champion: 'Enemy 1', spell: 'Snowball (Mark)', total: 80, remaining: 0, active: false },
    { champion: 'Enemy 1', spell: 'Flash (ARAM Haste)', total: 180, remaining: 0, active: false },
    { champion: 'Enemy 2', spell: 'Snowball (Mark)', total: 80, remaining: 0, active: false },
    { champion: 'Enemy 2', spell: 'Flash (ARAM Haste)', total: 180, remaining: 0, active: false },
    { champion: 'Enemy 3', spell: 'Clarity', total: 144, remaining: 0, active: false },
    { champion: 'Enemy 3', spell: 'Flash (ARAM Haste)', total: 180, remaining: 0, active: false },
    { champion: 'Enemy 4', spell: 'Exhaust', total: 126, remaining: 0, active: false },
    { champion: 'Enemy 5', spell: 'Ghost', total: 144, remaining: 0, active: false },
  ]);

  // 3. Arena Power Flower & Round
  const [arenaFlower, setArenaFlower] = useState({ remaining: 0, active: false });
  const [arenaRingOfFire, setArenaRingOfFire] = useState({ remaining: 50, active: false });

  // 4. URF Objectives & Flash
  const [urfFlashTimers, setUrfFlashTimers] = useState([
    { player: 'Enemy Top', remaining: 0, active: false },
    { player: 'Enemy Jg', remaining: 0, active: false },
    { player: 'Enemy Mid', remaining: 0, active: false },
    { player: 'Enemy Adc', remaining: 0, active: false },
    { player: 'Enemy Sup', remaining: 0, active: false },
  ]);

  // ARAM Balance Auras Database
  const aramAuras = [
    { champ: 'Akali', dealt: '+10%', taken: '-10%', note: '+20% Tenacity (Hyper Carry)' },
    { champ: 'Bard', dealt: '+15%', taken: '-15%', note: 'Full AP Meeps Nuker' },
    { champ: 'LeBlanc', dealt: '+10%', taken: '-10%', note: 'High burst execute' },
    { champ: 'Qiyana', dealt: '+10%', taken: '-10%', note: '+20% Tenacity' },
    { champ: 'Lee Sin', dealt: '+5%', taken: '-5%', note: '+20% Shielding' },
    { champ: 'Nidalee', dealt: '+10%', taken: '-10%', note: 'Max range Javelins buffed' },
    { champ: 'Sylas', dealt: '+5%', taken: '-5%', note: 'Healing buffed' },
    { champ: 'Ziggs', dealt: '-15%', taken: '+15%', note: '-20 Ability Haste (Siege Nerf)' },
    { champ: 'Ashe', dealt: '-18%', taken: '+10%', note: 'Volley cooldown heavily nerfed' },
    { champ: 'Sona', dealt: '-10%', taken: '+10%', note: '-20% Healing & Shielding' },
    { champ: 'Seraphine', dealt: '-10%', taken: '+10%', note: '-20% Shielding' },
    { champ: 'Teemo', dealt: '-10%', taken: '+10%', note: 'Mushroom damage reduced' },
    { champ: 'Jinx', dealt: '-10%', taken: '+10%', note: 'Rocket poke reduced' },
    { champ: 'Lux', dealt: '-10%', taken: '+10%', note: 'Laser cooldown adjusted' },
  ];

  // Arena Augments Database
  const arenaAugments: AugmentItem[] = [
    { name: 'Earthwake', tier: 'Prismatic', rank: 'S+', role: 'Bruisers / Assassins', description: 'Your dashes and blinks leave a path that detonates after 1s for massive physical damage.' },
    { name: 'Jeweled Gauntlet', tier: 'Prismatic', rank: 'S+', role: 'Mages / Assassins', description: 'Your abilities can critically strike with 140% damage. Gain +20% Critical Strike Chance.' },
    { name: 'Blade Waltz', tier: 'Prismatic', rank: 'S+', role: 'Fighters / ADCs', description: 'Active: Become untargetable and dash 8 times around target enemies, dealing on-hit damage.' },
    { name: 'Goliath', tier: 'Prismatic', rank: 'S+', role: 'Tanks / Colossi', description: 'Gain massive size, 800 Health, and 15% Adaptive Force.' },
    { name: 'Courage of the Colossus', tier: 'Gold', rank: 'S+', role: 'Tanks / Engagers', description: 'Immobilizing an enemy champion grants a decaying shield equal to 100-300 + 10% max HP.' },
    { name: 'Apex Inventor', tier: 'Gold', rank: 'S+', role: 'Item Builders', description: 'Gain 300 Item Haste. Heartsteel, Eclipse, Zhonya, and Sundered Sky trigger constantly.' },
    { name: 'Vulnerability', tier: 'Gold', rank: 'S+', role: 'DoT Mages / ADCs', description: 'Your item effects and damage-over-time effects can critically strike.' },
    { name: 'Bread and Butter', tier: 'Silver', rank: 'S', role: 'Q-Spammers', description: 'Gain 200 Ability Haste on your Q ability.' },
    { name: 'Bread and Jam', tier: 'Silver', rank: 'S', role: 'W-Spammers', description: 'Gain 200 Ability Haste on your W ability.' },
    { name: 'Bread and Cheese', tier: 'Silver', rank: 'S', role: 'E-Spammers', description: 'Gain 200 Ability Haste on your E ability.' },
    { name: 'Slow Cooker', tier: 'Gold', rank: 'S', role: 'Tanks', description: 'Every second, apply a stack of burn dealing 1.5% max HP damage. Stacks up to 10 times.' },
  ];

  // Demo Simulation Toggle
  const handleToggleDemoMode = () => {
    if (isDemoMode) {
      setIsDemoMode(false);
      setDemoGameTime(0);
      setSrObjectives([
        { id: 'dragon', name: 'Elemental Dragon', respawnTime: 300, total: 300, icon: '🐲', color: '#00c8ff' },
        { id: 'grubs', name: 'Voidgrubs / Herald', respawnTime: 300, total: 360, icon: '👾', color: '#ff2a5f' },
        { id: 'baron', name: 'Baron Nashor', respawnTime: 1200, total: 1200, icon: '👑', color: '#c084fc' },
      ]);
      setSrEnemySpells((prev) => prev.map((s) => ({ ...s, cooldownRemaining: 0, timerActive: false })));
      setAramRelics((prev) => prev.map((r) => ({ ...r, respawnTime: 0, active: false })));
      setAramInhibitor({ remaining: 0, active: false });
      setArenaFlower({ remaining: 0, active: false });
    } else {
      setIsDemoMode(true);
      setDemoGameTime(750);
      setSrObjectives([
        { id: 'dragon', name: 'Elemental Dragon', respawnTime: 135, total: 300, icon: '🐲', color: '#00c8ff' },
        { id: 'grubs', name: 'Voidgrubs / Herald', respawnTime: 210, total: 360, icon: '👾', color: '#ff2a5f' },
        { id: 'baron', name: 'Baron Nashor', respawnTime: 450, total: 1200, icon: '👑', color: '#c084fc' },
      ]);
      setAramRelics([
        { id: 'ally-outer', name: 'Ally Outer Shrine', side: 'Ally', respawnTime: 24, active: true },
        { id: 'ally-inner', name: 'Ally Inner Shrine', side: 'Ally', respawnTime: 0, active: false },
        { id: 'enemy-inner', name: 'Enemy Inner Shrine', side: 'Enemy', respawnTime: 42, active: true },
        { id: 'enemy-outer', name: 'Enemy Outer Shrine', side: 'Enemy', respawnTime: 0, active: false },
      ]);
      setArenaFlower({ remaining: 18, active: true });
      soundService.playTimerTick();
    }
  };

  // 1-Second Master Game Clock
  useEffect(() => {
    if (!isLive && !isDemoMode) return;

    const timer = setInterval(() => {
      if (isDemoMode) {
        setDemoGameTime((prev) => prev + 1);
      }

      // Decrement SR Spells
      setSrEnemySpells((prev) =>
        prev.map((s) => {
          if (s.timerActive && s.cooldownRemaining > 0) {
            const nextRemain = s.cooldownRemaining - 1;
            if (nextRemain === 0) {
              if (soundAlerts) soundService.playTimerTick();
              return { ...s, cooldownRemaining: 0, timerActive: false };
            }
            return { ...s, cooldownRemaining: nextRemain };
          }
          return s;
        })
      );

      // Decrement SR Objectives in demo
      if (isDemoMode) {
        setSrObjectives((prev) =>
          prev.map((obj) => {
            const nextTime = obj.respawnTime > 0 ? obj.respawnTime - 1 : obj.total;
            if (nextTime === 30 && soundAlerts) soundService.playQueuePop();
            return { ...obj, respawnTime: nextTime };
          })
        );
      }

      // Decrement ARAM Relics
      setAramRelics((prev) =>
        prev.map((relic) => {
          if (relic.active && relic.respawnTime > 0) {
            const nextTime = relic.respawnTime - 1;
            if (nextTime === 5 && soundAlerts) soundService.playTimerTick();
            return {
              ...relic,
              respawnTime: nextTime,
              active: nextTime > 0,
            };
          }
          return relic;
        })
      );

      // Decrement ARAM Inhibitor
      setAramInhibitor((prev) => {
        if (prev.active && prev.remaining > 0) {
          const nextRemain = prev.remaining - 1;
          if (nextRemain === 0 && soundAlerts) soundService.playQueuePop();
          return { remaining: nextRemain, active: nextRemain > 0 };
        }
        return prev;
      });

      // Decrement ARAM Spells
      setAramSpells((prev) =>
        prev.map((s) => {
          if (s.active && s.remaining > 0) {
            const next = s.remaining - 1;
            return { ...s, remaining: next, active: next > 0 };
          }
          return s;
        })
      );

      // Decrement Arena Flower
      setArenaFlower((prev) => {
        if (prev.active && prev.remaining > 0) {
          const next = prev.remaining - 1;
          return { remaining: next, active: next > 0 };
        }
        return prev;
      });

      // Decrement URF Flashes
      setUrfFlashTimers((prev) =>
        prev.map((f) => {
          if (f.active && f.remaining > 0) {
            const next = f.remaining - 1;
            return { ...f, remaining: next, active: next > 0 };
          }
          return f;
        })
      );
    }, 1000);

    return () => clearInterval(timer);
  }, [isLive, isDemoMode, soundAlerts]);

  // Live Game Objectives & Camps calculation from real in-game telemetry
  useEffect(() => {
    if (!isLive || !liveData?.gameData) return;
    const gTime = Math.floor(liveData.gameData.gameTime);
    const events = liveData.events?.Events || [];

    // 1. Dragon calculation
    // First dragon spawns at 5:00 (300s). Respawn is 300s (or 360s for Elder).
    let nextDragonTime = 300;
    const dragonKills = events.filter((e) => e.EventName === 'DragonKill');
    if (dragonKills.length > 0) {
      const lastKill = dragonKills[dragonKills.length - 1];
      const cooldown = lastKill.DragonType === 'Elder' ? 360 : 300;
      nextDragonTime = Math.floor(lastKill.EventTime) + cooldown;
    }
    const dragonRemaining = Math.max(0, nextDragonTime - gTime);

    // 2. Voidgrubs / Rift Herald / Baron Nashor
    let grubsHeraldName = 'Voidgrubs';
    let grubsHeraldRemaining = 0;
    let grubsHeraldTotal = 300;

    if (gTime < 300) {
      grubsHeraldName = 'Voidgrubs';
      grubsHeraldRemaining = 300 - gTime;
      grubsHeraldTotal = 300;
    } else if (gTime < 840) {
      grubsHeraldName = 'Voidgrubs';
      const hordeKills = events.filter((e) => e.EventName === 'HordeKill');
      if (hordeKills.length >= 6) {
        grubsHeraldName = 'Rift Herald (14:00)';
        grubsHeraldRemaining = Math.max(0, 840 - gTime);
        grubsHeraldTotal = 840;
      } else {
        grubsHeraldRemaining = 0; // ALIVE
      }
    } else if (gTime < 1185) {
      const heraldKills = events.filter((e) => e.EventName === 'HeraldKill');
      if (heraldKills.length > 0) {
        grubsHeraldName = 'Baron Nashor (20:00)';
        grubsHeraldRemaining = Math.max(0, 1200 - gTime);
        grubsHeraldTotal = 1200;
      } else {
        grubsHeraldName = 'Rift Herald';
        grubsHeraldRemaining = 0; // ALIVE
      }
    } else {
      grubsHeraldName = 'Baron Nashor';
      grubsHeraldRemaining = Math.max(0, 1200 - gTime);
      grubsHeraldTotal = 1200;
    }

    // Baron Nashor calculation
    let nextBaronTime = 1200;
    const baronKills = events.filter((e) => e.EventName === 'BaronKill');
    if (baronKills.length > 0) {
      const lastBaron = baronKills[baronKills.length - 1];
      nextBaronTime = Math.floor(lastBaron.EventTime) + 360;
    }
    const baronRemaining = Math.max(0, nextBaronTime - gTime);

    setSrObjectives([
      { id: 'dragon', name: 'Elemental Dragon', respawnTime: dragonRemaining, total: 300, icon: '🐲', color: '#00c8ff' },
      { id: 'grubs', name: grubsHeraldName, respawnTime: grubsHeraldRemaining, total: grubsHeraldTotal, icon: '👾', color: '#ff2a5f' },
      { id: 'baron', name: 'Baron Nashor', respawnTime: baronRemaining, total: 360, icon: '👑', color: '#c084fc' },
    ]);

    // Live jungle camp first spawn countdowns (Red/Blue 90s, Scuttler 210s)
    setSrJungleCamps((prev) =>
      prev.map((camp) => {
        if (camp.name.includes('Blue Buff') || camp.name.includes('Red Buff')) {
          if (gTime < 90) {
            return { ...camp, remaining: 90 - gTime, active: true };
          } else if (camp.active && camp.remaining === 90 - gTime) {
            return { ...camp, remaining: 0, active: false };
          }
        } else if (camp.name.includes('Scuttler')) {
          if (gTime < 210) {
            return { ...camp, remaining: 210 - gTime, active: true };
          } else if (camp.active && camp.remaining === 210 - gTime) {
            return { ...camp, remaining: 0, active: false };
          }
        }
        return camp;
      })
    );
  }, [liveData, isLive]);

  // Dynamically populate enemy summoner spells from live game data
  useEffect(() => {
    if (!liveData || !liveData.allPlayers || liveData.allPlayers.length === 0) return;

    const myName = liveData.activePlayer?.summonerName;
    const me = liveData.allPlayers.find((p) => p.summonerName === myName);
    const myTeam = me?.team || 'ORDER';
    const enemyTeam = myTeam === 'ORDER' ? 'CHAOS' : 'ORDER';

    const enemyPlayers = liveData.allPlayers.filter((p) => p.team === enemyTeam);
    if (enemyPlayers.length === 0) return;

    const roles = ['TOP', 'JUNGLE', 'MID', 'ADC', 'SUPPORT'];

    const getCd = (spell: string) => {
      const s = spell.toLowerCase();
      if (s.includes('flash')) return 300;
      if (s.includes('teleport')) return 360;
      if (s.includes('ghost')) return 240;
      if (s.includes('heal')) return 240;
      if (s.includes('barrier')) return 180;
      if (s.includes('exhaust')) return 210;
      if (s.includes('ignite')) return 180;
      if (s.includes('cleanse')) return 210;
      if (s.includes('smite')) return 90;
      return 300;
    };

    const getSpellId = (spell: string) => {
      const s = spell.toLowerCase();
      if (s.includes('flash')) return 4;
      if (s.includes('teleport')) return 12;
      if (s.includes('ghost')) return 6;
      if (s.includes('heal')) return 7;
      if (s.includes('barrier')) return 21;
      if (s.includes('exhaust')) return 3;
      if (s.includes('ignite')) return 14;
      if (s.includes('cleanse')) return 1;
      if (s.includes('smite')) return 11;
      return 4;
    };

    const newEnemySpells: SpellTracker[] = [];
    enemyPlayers.forEach((p, idx) => {
      const role = roles[idx] || 'FLEX';
      const champ = p.championName || `Enemy ${idx + 1}`;

      const s1Name = p.summonerSpells?.summonerSpellOne?.displayName || 'Flash';
      const s2Name = p.summonerSpells?.summonerSpellTwo?.displayName || 'Ignite';

      newEnemySpells.push({
        champion: champ,
        role,
        spellId: getSpellId(s1Name),
        spellName: s1Name,
        cooldownTotal: getCd(s1Name),
        cooldownRemaining: 0,
        timerActive: false,
        readyGameTime: 0,
      });

      newEnemySpells.push({
        champion: champ,
        role,
        spellId: getSpellId(s2Name),
        spellName: s2Name,
        cooldownTotal: getCd(s2Name),
        cooldownRemaining: 0,
        timerActive: false,
        readyGameTime: 0,
      });
    });

    if (newEnemySpells.length > 0) {
      setSrEnemySpells((prev) => {
        return newEnemySpells.map((ns) => {
          const match = prev.find((ps) => ps.champion === ns.champion && ps.spellName === ns.spellName);
          if (match && match.timerActive) {
            return {
              ...ns,
              timerActive: match.timerActive,
              cooldownRemaining: match.cooldownRemaining,
              readyGameTime: match.readyGameTime,
            };
          }
          return ns;
        });
      });
    }
  }, [liveData]);

  // Actions
  const triggerSrSpell = (index: number) => {
    setSrEnemySpells((prev) => {
      const copy = [...prev];
      const target = copy[index];
      if (target.timerActive) {
        target.timerActive = false;
        target.cooldownRemaining = 0;
      } else {
        target.timerActive = true;
        target.cooldownRemaining = target.cooldownTotal;
        target.readyGameTime = activeGameTime + target.cooldownTotal;
        soundService.playTimerTick();
      }
      return copy;
    });
  };

  const triggerAramRelic = (id: string) => {
    setAramRelics((prev) =>
      prev.map((r) => {
        if (r.id === id) {
          if (r.active) return { ...r, active: false, respawnTime: 0 };
          soundService.playTimerTick();
          return { ...r, active: true, respawnTime: 60 };
        }
        return r;
      })
    );
  };

  const triggerAramInhibitor = () => {
    if (aramInhibitor.active) {
      setAramInhibitor({ remaining: 0, active: false });
    } else {
      setAramInhibitor({ remaining: 300, active: true });
      soundService.playTimerTick();
    }
  };

  const triggerAramSpell = (idx: number) => {
    setAramSpells((prev) => {
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

  const triggerArenaFlower = () => {
    if (arenaFlower.active) {
      setArenaFlower({ remaining: 0, active: false });
    } else {
      setArenaFlower({ remaining: 45, active: true });
      soundService.playTimerTick();
    }
  };

  const triggerUrfFlash = (idx: number) => {
    setUrfFlashTimers((prev) => {
      const copy = [...prev];
      if (copy[idx].active) {
        copy[idx].active = false;
        copy[idx].remaining = 0;
      } else {
        copy[idx].active = true;
        copy[idx].remaining = 60;
        soundService.playTimerTick();
      }
      return copy;
    });
  };

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  // Copy chat message
  const copyTimersToChat = () => {
    let msg = '';
    if (activeMode === 'CLASSIC') {
      const activeCooldowns = srEnemySpells.filter((s) => s.timerActive);
      const parts: string[] = [];
      activeCooldowns.forEach((s) => {
        parts.push(`${s.role.toLowerCase()} ${s.spellName.toLowerCase()} ${formatTime(s.readyGameTime)}`);
      });
      const drg = srObjectives.find((o) => o.id === 'dragon');
      if (drg && drg.respawnTime > 0) parts.push(`drg ${formatTime(activeGameTime + drg.respawnTime)}`);
      msg = parts.length > 0 ? `[RiftCore] ${parts.join(' | ')}` : '[RiftCore] All enemy summoners & objectives UP';
    } else if (activeMode === 'ARAM') {
      const parts: string[] = [];
      if (aramInhibitor.active) parts.push(`Inh ready in ${formatTime(aramInhibitor.remaining)}`);
      const activeRelics = aramRelics.filter((r) => r.active);
      if (activeRelics.length > 0) {
        parts.push(`Relic up in ${Math.min(...activeRelics.map((r) => r.respawnTime))}s`);
      }
      msg = parts.length > 0 ? `[RiftCore ARAM] ${parts.join(' | ')}` : '[RiftCore ARAM] Relics UP';
    } else if (activeMode === 'CHERRY') {
      msg = arenaFlower.active
        ? `[RiftCore Arena] Flower regrows in ${arenaFlower.remaining}s • Ring closes at 50s`
        : '[RiftCore Arena] Power Flowers UP • Grab 3 hits for shield & 20% CDR';
    } else {
      msg = `[RiftCore ${activeMode}] Mode active`;
    }

    navigator.clipboard.writeText(msg);
    setCopied(true);
    soundService.playAutoAccept();
    setTimeout(() => setCopied(false), 2500);
  };

  // CS Benchmark
  const activePlayerInList = liveData?.allPlayers?.find(
    (p) => p.summonerName === liveData?.activePlayer?.summonerName
  );
  const currentCs = isLive
    ? (liveData?.activePlayer?.scores?.creepScore ?? activePlayerInList?.scores?.creepScore ?? 0)
    : isDemoMode ? 104 : 0;
  const gameMinutes = Math.max(1, activeGameTime / 60);
  const csPerMin = activeGameTime > 0 ? (currentCs / gameMinutes).toFixed(1) : '0.0';
  const numCsPerMin = parseFloat(csPerMin);

  const getCsBenchmarkRating = () => {
    if (!isLive && !isDemoMode) return { tier: 'STANDBY', color: '#94a3b8', diff: 'Awaiting Wave' };
    if (numCsPerMin >= 8.4) return { tier: 'CHALLENGER PACE', color: '#00c8ff', diff: '+28% Ahead' };
    if (numCsPerMin >= 7.0) return { tier: 'DIAMOND PACE', color: '#22c55e', diff: '+15% Ahead' };
    if (numCsPerMin >= 5.8) return { tier: 'GOLD / PLAT PACE', color: '#f59e0b', diff: 'Average' };
    return { tier: 'BELOW TARGET', color: '#ff2a5f', diff: 'Focus on Waves' };
  };

  const csRating = getCsBenchmarkRating();

  const getModeTitle = () => {
    switch (activeMode) {
      case 'ARAM':
        return 'Howling Abyss (ARAM Mayhem)';
      case 'CHERRY':
        return 'Arena 2v2v2v2 (Rings of Wrath)';
      case 'URF':
        return 'Ultra Rapid Fire (URF / ARURF)';
      case 'ONEFORALL':
        return 'One for All (5x Synergy)';
      default:
        return "Summoner's Rift (5v5 Classic / Ranked)";
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', minWidth: 0, width: '100%', maxWidth: '100%', boxSizing: 'border-box' }}>

      {/* IN-GAME OVERLAY STATUS & FULLSCREEN ENGINE BANNER */}
      <div
        className="glass-card"
        style={{
          padding: '12px 18px',
          border: isPinned ? '1px solid rgba(0, 200, 255, 0.4)' : '1px solid rgba(255, 170, 0, 0.4)',
          background: isPinned ? 'rgba(0, 200, 255, 0.05)' : 'rgba(255, 170, 0, 0.05)',
          minWidth: 0,
          width: '100%',
          boxSizing: 'border-box',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '10px', minWidth: 0 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px', minWidth: 0 }}>
            <div
              style={{
                width: '32px',
                height: '32px',
                borderRadius: '8px',
                background: isPinned ? 'rgba(0, 200, 255, 0.15)' : 'rgba(0, 255, 170, 0.15)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                flexShrink: 0,
              }}
            >
              {isPinned ? <Pin size={16} color="var(--blue-primary)" /> : <TrendingUp size={16} color="#00ffaa" />}
            </div>
            <div style={{ minWidth: 0 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flexWrap: 'wrap' }}>
                <span style={{ fontSize: '13px', fontWeight: 800, color: '#fff' }}>
                  In-Game HUD Overlay
                </span>
                <span
                  style={{
                    fontSize: '10.5px',
                    fontWeight: 800,
                    padding: '2px 8px',
                    borderRadius: '4px',
                    background: isPinned ? 'rgba(0, 200, 255, 0.15)' : 'rgba(0, 255, 170, 0.15)',
                    color: isPinned ? 'var(--blue-primary)' : '#00ffaa',
                    border: `1px solid ${isPinned ? 'rgba(0, 200, 255, 0.3)' : 'rgba(0, 255, 170, 0.3)'}`,
                  }}
                >
                  {isPinned ? '🟢 OVERLAY PINNED' : '⚡ FULLSCREEN READY'}
                </span>
                <span
                  style={{
                    fontSize: '10px',
                    fontWeight: 700,
                    padding: '2px 6px',
                    borderRadius: '4px',
                    background: 'rgba(255, 255, 255, 0.06)',
                    border: '1px solid rgba(255, 255, 255, 0.1)',
                    color: '#94a3b8',
                  }}
                >
                  Toggle: <kbd style={{ color: '#00c8ff' }}>Ctrl + Space</kbd>
                </span>
              </div>
              <div style={{ fontSize: '11px', color: '#64748b', marginTop: '2px' }}>
                Ultra-smooth in-game HUD • 100% Ban Safe & Riot Approved
              </div>
            </div>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flexWrap: 'wrap' }}>
            {onTogglePin && (
              <button
                onClick={onTogglePin}
                style={{
                  padding: '6px 12px',
                  borderRadius: '6px',
                  border: isPinned ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                  background: isPinned ? 'rgba(0, 200, 255, 0.15)' : 'rgba(255, 255, 255, 0.05)',
                  color: isPinned ? 'var(--blue-primary)' : '#fff',
                  fontSize: '11px',
                  fontWeight: 700,
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '5px',
                }}
              >
                <Pin size={12} />
                <span>{isPinned ? 'Overlay Pinned' : 'Pin In-Game'}</span>
              </button>
            )}

            {isLive && onToggleCompact ? (
              <button
                onClick={onToggleCompact}
                style={{
                  padding: '6px 14px',
                  borderRadius: '6px',
                  border: '1px solid var(--blue-primary)',
                  background: isCompactMode ? 'var(--blue-primary)' : 'linear-gradient(135deg, rgba(0, 210, 255, 0.25) 0%, rgba(14, 165, 233, 0.15) 100%)',
                  color: isCompactMode ? '#000' : '#fff',
                  fontSize: '11px',
                  fontWeight: 800,
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  boxShadow: '0 0 12px rgba(0, 210, 255, 0.25)',
                }}
                title="Toggle floating tactical HUD directly on top of League (Alt+G)"
              >
                {isCompactMode ? <Maximize2 size={13} /> : <Zap size={13} color="var(--blue-primary)" />}
                <span>{isCompactMode ? 'Expand Desktop View' : '✨ Toggle In-Game HUD (Alt+G)'}</span>
              </button>
            ) : (
              <button
                onClick={handleToggleDemoMode}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  padding: '6px 13px',
                  borderRadius: '6px',
                  background: isDemoMode ? 'rgba(234, 179, 8, 0.15)' : 'rgba(0, 210, 255, 0.12)',
                  border: isDemoMode ? '1px solid #facc15' : '1px solid var(--blue-primary)',
                  fontSize: '11px',
                  fontWeight: 800,
                  color: isDemoMode ? '#facc15' : 'var(--blue-primary)',
                  cursor: 'pointer',
                  boxShadow: '0 0 12px rgba(0, 210, 255, 0.2)',
                  transition: 'all 0.15s ease',
                }}
                title="Tactical HUD automatically anchors over League when match starts. Click to test live simulation!"
              >
                {isDemoMode ? <RotateCcw size={12} color="#facc15" /> : <Eye size={12} color="var(--blue-primary)" />}
                <span>{isDemoMode ? 'Stop Simulation' : '👁️ Preview HUD (Simulation)'}</span>
              </button>
            )}

            <button
              onClick={() => setShowFullscreenGuide(!showFullscreenGuide)}
              style={{
                padding: '6px 10px',
                borderRadius: '6px',
                border: showFullscreenGuide ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                background: showFullscreenGuide ? 'rgba(0, 210, 255, 0.12)' : 'rgba(255, 255, 255, 0.04)',
                color: showFullscreenGuide ? 'var(--blue-primary)' : '#94a3b8',
                fontSize: '11px',
                fontWeight: 700,
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: '5px',
                transition: 'all 0.15s ease',
              }}
            >
              <HelpCircle size={13} />
              <span>Overlay Guide</span>
            </button>
          </div>
        </div>

        {/* Expandable Fullscreen HUD Guide (Sleek 4-Card Feature Grid) */}
        {showFullscreenGuide && (
          <div
            style={{
              marginTop: '14px',
              paddingTop: '14px',
              borderTop: '1px solid rgba(255, 255, 255, 0.08)',
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
              gap: '10px',
            }}
          >
            {/* Card 1: True Fullscreen */}
            <div
              style={{
                background: 'rgba(0, 210, 255, 0.04)',
                border: '1px solid rgba(0, 210, 255, 0.22)',
                borderRadius: '8px',
                padding: '9px 12px',
                display: 'flex',
                flexDirection: 'column',
                gap: '8px',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <Monitor size={14} color="#00d2ff" />
                  <span style={{ fontSize: '11px', fontWeight: 800, color: '#fff' }}>True Fullscreen</span>
                </div>
                <span style={{ fontSize: '9px', fontWeight: 900, padding: '1px 6px', borderRadius: '3px', background: 'rgba(0, 210, 255, 0.15)', color: '#00d2ff', border: '1px solid rgba(0, 210, 255, 0.3)' }}>
                  ACTIVE
                </span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '5px', flexWrap: 'wrap' }}>
                <span style={{ fontSize: '9.5px', fontWeight: 800, padding: '2px 7px', borderRadius: '4px', background: 'rgba(0, 210, 255, 0.1)', color: '#00d2ff', border: '1px solid rgba(0, 210, 255, 0.2)' }}>
                  ⚡ Ultra Smooth
                </span>
                <span style={{ fontSize: '9.5px', fontWeight: 800, padding: '2px 7px', borderRadius: '4px', background: 'rgba(16, 185, 129, 0.1)', color: '#10b981', border: '1px solid rgba(16, 185, 129, 0.2)' }}>
                  🚀 0 FPS Drop
                </span>
                <span style={{ fontSize: '9.5px', fontWeight: 800, padding: '2px 7px', borderRadius: '4px', background: 'rgba(255, 255, 255, 0.05)', color: '#cbd5e1', border: '1px solid rgba(255, 255, 255, 0.08)' }}>
                  All Resolutions
                </span>
              </div>
            </div>

            {/* Card 2: Hotkeys */}
            <div
              style={{
                background: 'rgba(250, 204, 21, 0.04)',
                border: '1px solid rgba(250, 204, 21, 0.22)',
                borderRadius: '8px',
                padding: '9px 12px',
                display: 'flex',
                flexDirection: 'column',
                gap: '8px',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <Keyboard size={14} color="#facc15" />
                  <span style={{ fontSize: '11px', fontWeight: 800, color: '#fff' }}>Quick Hotkeys</span>
                </div>
                <span style={{ fontSize: '9px', fontWeight: 900, padding: '1px 6px', borderRadius: '3px', background: 'rgba(250, 204, 21, 0.15)', color: '#facc15', border: '1px solid rgba(250, 204, 21, 0.3)' }}>
                  INSTANT
                </span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '5px', flexWrap: 'wrap' }}>
                <kbd style={{ background: '#1e293b', border: '1px solid rgba(250,204,21,0.35)', padding: '2px 7px', borderRadius: '4px', color: '#facc15', fontSize: '10px', fontFamily: 'monospace', fontWeight: 800 }}>Ctrl + Space</kbd>
                <kbd style={{ background: '#1e293b', border: '1px solid rgba(255,255,255,0.15)', padding: '2px 7px', borderRadius: '4px', color: '#cbd5e1', fontSize: '10px', fontFamily: 'monospace', fontWeight: 700 }}>Alt + ~</kbd>
                <kbd style={{ background: '#1e293b', border: '1px solid rgba(255,255,255,0.15)', padding: '2px 7px', borderRadius: '4px', color: '#cbd5e1', fontSize: '10px', fontFamily: 'monospace', fontWeight: 700 }}>Alt + X</kbd>
              </div>
            </div>

            {/* Card 3: Mini HUD */}
            <div
              style={{
                background: 'rgba(16, 185, 129, 0.04)',
                border: '1px solid rgba(16, 185, 129, 0.22)',
                borderRadius: '8px',
                padding: '9px 12px',
                display: 'flex',
                flexDirection: 'column',
                gap: '8px',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <Maximize2 size={14} color="#10b981" />
                  <span style={{ fontSize: '11px', fontWeight: 800, color: '#fff' }}>Mini HUD Mode</span>
                </div>
                <kbd style={{ background: '#1e293b', border: '1px solid rgba(16,185,129,0.35)', padding: '1px 6px', borderRadius: '4px', color: '#10b981', fontSize: '10px', fontFamily: 'monospace', fontWeight: 800 }}>
                  Alt + G
                </kbd>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '5px', flexWrap: 'wrap' }}>
                <span style={{ fontSize: '9.5px', fontWeight: 800, padding: '2px 7px', borderRadius: '4px', background: 'rgba(16, 185, 129, 0.1)', color: '#10b981', border: '1px solid rgba(16, 185, 129, 0.2)' }}>
                  📊 CS Pacing
                </span>
                <span style={{ fontSize: '9.5px', fontWeight: 800, padding: '2px 7px', borderRadius: '4px', background: 'rgba(255, 255, 255, 0.05)', color: '#cbd5e1', border: '1px solid rgba(255, 255, 255, 0.08)' }}>
                  💰 Gold Lead
                </span>
                <span style={{ fontSize: '9.5px', fontWeight: 800, padding: '2px 7px', borderRadius: '4px', background: 'rgba(255, 255, 255, 0.05)', color: '#cbd5e1', border: '1px solid rgba(255, 255, 255, 0.08)' }}>
                  🐉 Pit Timers
                </span>
              </div>
            </div>

            {/* Card 4: Vanguard Safe */}
            <div
              style={{
                background: 'rgba(168, 85, 247, 0.04)',
                border: '1px solid rgba(168, 85, 247, 0.22)',
                borderRadius: '8px',
                padding: '9px 12px',
                display: 'flex',
                flexDirection: 'column',
                gap: '8px',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <ShieldCheck size={14} color="#c084fc" />
                  <span style={{ fontSize: '11px', fontWeight: 800, color: '#fff' }}>Anti-Cheat Safe</span>
                </div>
                <span style={{ fontSize: '9px', fontWeight: 900, padding: '1px 6px', borderRadius: '3px', background: 'rgba(168, 85, 247, 0.15)', color: '#c084fc', border: '1px solid rgba(168, 85, 247, 0.3)' }}>
                  RIOT APPROVED
                </span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '5px', flexWrap: 'wrap' }}>
                <span style={{ fontSize: '9.5px', fontWeight: 800, padding: '2px 7px', borderRadius: '4px', background: 'rgba(16, 185, 129, 0.1)', color: '#10b981', border: '1px solid rgba(16, 185, 129, 0.2)' }}>
                  🔒 100% Ban-Free
                </span>
                <span style={{ fontSize: '9.5px', fontWeight: 800, padding: '2px 7px', borderRadius: '4px', background: 'rgba(168, 85, 247, 0.1)', color: '#c084fc', border: '1px solid rgba(168, 85, 247, 0.2)' }}>
                  ✓ Official Game API
                </span>
                <span style={{ fontSize: '9.5px', fontWeight: 800, padding: '2px 7px', borderRadius: '4px', background: 'rgba(255, 255, 255, 0.05)', color: '#cbd5e1', border: '1px solid rgba(255, 255, 255, 0.08)' }}>
                  🛡️ No Cheats / Mods
                </span>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* TOP CONTROLS & MODE SELECTOR */}
      <div className="glass-card" style={{ padding: '14px 20px', border: '1px solid var(--border-blue)', minWidth: 0, width: '100%', boxSizing: 'border-box' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '12px', minWidth: 0, width: '100%' }}>
          {/* Mode Selector Tabs */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px', flexWrap: 'wrap', minWidth: 0 }}>
            {(
              [
                { mode: 'CLASSIC', label: "Summoner's Rift", icon: '⚔️' },
                { mode: 'ARAM', label: 'ARAM Mayhem', icon: '❄️' },
                { mode: 'CHERRY', label: 'Arena 2v2v2v2', icon: '🥊' },
                { mode: 'URF', label: 'URF / ARURF', icon: '⚡' },
                { mode: 'ONEFORALL', label: 'One for All', icon: '👥' },
              ] as const
            ).map((item) => (
              <button
                key={item.mode}
                onClick={() => setActiveMode(item.mode)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  padding: '6px 12px',
                  borderRadius: '6px',
                  fontSize: '12px',
                  fontWeight: activeMode === item.mode ? 800 : 600,
                  cursor: 'pointer',
                  border:
                    activeMode === item.mode
                      ? '1px solid var(--blue-primary)'
                      : '1px solid var(--border-subtle)',
                  background:
                    activeMode === item.mode
                      ? 'rgba(0, 200, 255, 0.15)'
                      : 'rgba(255, 255, 255, 0.03)',
                  color: activeMode === item.mode ? '#fff' : '#94a3b8',
                  boxShadow:
                    activeMode === item.mode
                      ? '0 0 10px rgba(0, 200, 255, 0.25)'
                      : 'none',
                  transition: 'all 0.15s ease',
                }}
              >
                <span>{item.icon}</span>
                <span>{item.label}</span>
              </button>
            ))}
          </div>

          {/* Clock & Action buttons */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px', flexWrap: 'wrap' }}>
            {/* Clock display */}
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
                padding: '5px 12px',
                borderRadius: '6px',
                background: 'rgba(0, 0, 0, 0.4)',
                border: '1px solid var(--border-subtle)',
              }}
            >
              <Clock size={14} color="var(--blue-primary)" />
              <span style={{ fontSize: '11px', color: '#94a3b8' }}>Game Clock:</span>
              <strong
                className="tabular-nums"
                style={{
                  fontSize: '15px',
                  fontWeight: 800,
                  fontFamily: 'var(--font-timer)',
                  letterSpacing: '0.8px',
                  color: isLive ? '#22c55e' : isDemoMode ? '#facc15' : 'var(--blue-primary)',
                }}
              >
                {activeGameTime > 0 ? formatTime(activeGameTime) : '00:00 (Standby)'}
              </strong>
            </div>

            {/* Audio Toggle */}
            <button
              onClick={() => setSoundAlerts(!soundAlerts)}
              style={{
                padding: '6px 10px',
                borderRadius: '6px',
                border: '1px solid var(--border-subtle)',
                background: soundAlerts ? 'rgba(0, 200, 255, 0.1)' : 'rgba(255, 255, 255, 0.04)',
                color: soundAlerts ? 'var(--blue-primary)' : '#64748b',
                fontSize: '11px',
                fontWeight: 700,
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: '5px',
              }}
              title="Toggle in-game audio cues for objective spawns"
            >
              {soundAlerts ? <Volume2 size={13} /> : <VolumeX size={13} />}
              <span>{soundAlerts ? 'Audio' : 'Muted'}</span>
            </button>

            {/* Test Simulation Mode Button */}
            {!isLive && (
              <button
                onClick={handleToggleDemoMode}
                style={{
                  padding: '6px 12px',
                  borderRadius: '6px',
                  border: isDemoMode ? '1px solid #facc15' : '1px solid var(--border-subtle)',
                  background: isDemoMode ? 'rgba(234, 179, 8, 0.15)' : 'rgba(255, 255, 255, 0.04)',
                  color: isDemoMode ? '#facc15' : '#94a3b8',
                  fontSize: '11px',
                  fontWeight: 700,
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '5px',
                }}
              >
                {isDemoMode ? <RotateCcw size={12} /> : <Play size={12} />}
                <span>{isDemoMode ? 'Stop Simulation' : '🧪 Test Mode'}</span>
              </button>
            )}

            {/* 1-Click Copy Timers to Chat */}
            <button
              onClick={copyTimersToChat}
              className="btn-primary btn-accent"
              style={{ padding: '6px 14px', fontSize: '11px', fontWeight: 800 }}
              title="Copy active enemy cooldowns and objectives to clipboard for team chat"
            >
              {copied ? <Check size={13} /> : <Copy size={13} />}
              <span>{copied ? 'COPIED TO CHAT!' : 'COPY TIMERS'}</span>
            </button>
          </div>
        </div>
      </div>

      {/* ======================================================== */}
      {/* MODE 1: SUMMONER'S RIFT (CLASSIC)                        */}
      {/* ======================================================== */}
      {activeMode === 'CLASSIC' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {/* CS Benchmark & Pace */}
          {settings?.hudCsBenchmark !== false && (
            <div className="glass-card" style={{ padding: '14px 20px', minWidth: 0, width: '100%', boxSizing: 'border-box' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '12px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                  <TrendingUp size={18} color="var(--blue-primary)" />
                  <span style={{ fontSize: '13px', fontWeight: 800, color: '#fff' }}>
                    Live CS Performance & Farm Pace
                  </span>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                  <span style={{ fontSize: '13px', color: '#94a3b8' }}>
                    CS: <strong style={{ color: '#fff' }}>{currentCs}</strong> ({csPerMin} CS/min)
                  </span>
                  <span
                    style={{
                      fontSize: '11px',
                      fontWeight: 800,
                      padding: '3px 10px',
                      borderRadius: '4px',
                      background: 'rgba(0, 200, 255, 0.1)',
                      color: csRating.color,
                      border: `1px solid ${csRating.color}`,
                    }}
                  >
                    {csRating.tier}
                  </span>
                </div>
              </div>
            </div>
          )}

          {/* EARLY JUNGLE LEASH & PATH PREDICTOR */}
          {settings?.hudJunglePathing !== false && (
            <div
              className="glass-card"
              style={{
                padding: '16px 20px',
                border: '1px solid rgba(0, 200, 255, 0.3)',
                background: 'linear-gradient(90deg, rgba(0, 200, 255, 0.05) 0%, rgba(11, 18, 30, 0.7) 100%)',
                minWidth: 0,
                width: '100%',
                boxSizing: 'border-box',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px', flexWrap: 'wrap', gap: '8px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <Compass size={18} color="var(--blue-primary)" />
                  <span style={{ fontSize: '13px', fontWeight: 800, color: '#fff' }}>
                    Early Game Jungle Leash & Route Predictor
                  </span>
                  <span
                    style={{
                      fontSize: '10px',
                      fontWeight: 800,
                      padding: '2px 8px',
                      borderRadius: '4px',
                      background: 'rgba(0, 200, 255, 0.15)',
                      color: 'var(--blue-primary)',
                      border: '1px solid var(--border-blue)',
                    }}
                  >
                    PRO SCOUT
                  </span>
                </div>

                {/* Leash Selector */}
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <span style={{ fontSize: '11px', color: '#94a3b8', fontWeight: 600 }}>Observed Start:</span>
                  <button
                    onClick={() => setJungleStart('bot')}
                    style={{
                      padding: '3px 8px',
                      borderRadius: '4px',
                      fontSize: '11px',
                      fontWeight: 800,
                      cursor: 'pointer',
                      background: jungleStart === 'bot' ? 'var(--blue-primary)' : 'rgba(255,255,255,0.05)',
                      color: jungleStart === 'bot' ? '#000' : '#cbd5e1',
                      border: jungleStart === 'bot' ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                    }}
                  >
                    Bot Leash (90%)
                  </button>
                  <button
                    onClick={() => setJungleStart('top')}
                    style={{
                      padding: '3px 8px',
                      borderRadius: '4px',
                      fontSize: '11px',
                      fontWeight: 800,
                      cursor: 'pointer',
                      background: jungleStart === 'top' ? 'var(--blue-primary)' : 'rgba(255,255,255,0.05)',
                      color: jungleStart === 'top' ? '#000' : '#cbd5e1',
                      border: jungleStart === 'top' ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                    }}
                  >
                    Top Leash
                  </button>
                  <button
                    onClick={() => setJungleStart('solo')}
                    style={{
                      padding: '3px 8px',
                      borderRadius: '4px',
                      fontSize: '11px',
                      fontWeight: 800,
                      cursor: 'pointer',
                      background: jungleStart === 'solo' ? 'var(--blue-primary)' : 'rgba(255,255,255,0.05)',
                      color: jungleStart === 'solo' ? '#000' : '#cbd5e1',
                      border: jungleStart === 'solo' ? '1px solid var(--blue-primary)' : '1px solid var(--border-subtle)',
                    }}
                  >
                    Solo / Raptors
                  </button>
                </div>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))', gap: '12px' }}>
                {/* Predicted Path Step-by-Step */}
                <div style={{ background: 'rgba(255, 255, 255, 0.02)', padding: '10px 14px', borderRadius: '8px', border: '1px solid var(--border-subtle)' }}>
                  <div style={{ fontSize: '11px', fontWeight: 800, color: '#94a3b8', textTransform: 'uppercase', marginBottom: '6px' }}>
                    Predicted Route (Level 1 to 4)
                  </div>
                  <div style={{ fontSize: '12px', fontWeight: 700, color: '#fff', display: 'flex', alignItems: 'center', gap: '6px', flexWrap: 'wrap' }}>
                    {jungleStart === 'bot' ? (
                      <>
                        <span style={{ color: '#00c8ff' }}>Bot Buff (1:30)</span>
                        <span>➔</span>
                        <span>Gromp / Wolves</span>
                        <span>➔</span>
                        <span style={{ color: '#f59e0b' }}>Top Buff (2:45)</span>
                        <span>➔</span>
                        <span style={{ color: '#ff2a5f' }}>Top Scuttle (3:30)</span>
                      </>
                    ) : jungleStart === 'top' ? (
                      <>
                        <span style={{ color: '#00c8ff' }}>Top Buff (1:30)</span>
                        <span>➔</span>
                        <span>Wolves / Raptors</span>
                        <span>➔</span>
                        <span style={{ color: '#f59e0b' }}>Bot Buff (2:45)</span>
                        <span>➔</span>
                        <span style={{ color: '#ff2a5f' }}>Bot Scuttle (3:30)</span>
                      </>
                    ) : (
                      <>
                        <span style={{ color: '#00c8ff' }}>Raptors / Wolves (1:30)</span>
                        <span>➔</span>
                        <span>Full Clear to Opposite Side</span>
                        <span>➔</span>
                        <span style={{ color: '#ff2a5f' }}>Scuttle Contest (3:30)</span>
                      </>
                    )}
                  </div>
                </div>

                {/* Gank Warning & Scuttle Countdown */}
                <div
                  style={{
                    background: jungleStart === 'bot' ? 'rgba(255, 42, 95, 0.08)' : 'rgba(0, 200, 255, 0.08)',
                    padding: '10px 14px',
                    borderRadius: '8px',
                    border: `1px solid ${jungleStart === 'bot' ? 'rgba(255, 42, 95, 0.3)' : 'rgba(0, 200, 255, 0.3)'}`,
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '4px' }}>
                    <div style={{ fontSize: '11px', fontWeight: 800, color: jungleStart === 'bot' ? 'var(--red-primary)' : 'var(--blue-primary)', textTransform: 'uppercase' }}>
                      {jungleStart === 'bot' ? '⚠️ High Alert: Top & Mid Lanes' : '⚠️ High Alert: Bot & Mid Lanes'}
                    </div>
                    <div
                      className="tabular-nums"
                      style={{ fontSize: '12px', fontFamily: 'var(--font-timer)', fontWeight: 800, letterSpacing: '0.6px', color: '#facc15' }}
                    >
                      {activeGameTime < 210 ? `Scuttle in ${formatTime(210 - activeGameTime)}` : 'Scuttles Active'}
                    </div>
                  </div>
                  <div style={{ fontSize: '11px', color: '#cbd5e1' }}>
                    {jungleStart === 'bot'
                      ? 'Enemy jungler will reach Top River around 3:15–3:30. Top laners: ward river bush at 2:45 and play toward safe side!'
                      : 'Enemy jungler will reach Bot River around 3:15–3:30. Bot laners: avoid overextending after 3:10 and watch for gank!'}
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Epics & Camps */}
          {settings?.hudObjectiveTimers !== false && (
            <div className="glass-card" style={{ padding: '16px', minWidth: 0, width: '100%', boxSizing: 'border-box' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '14px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <Award size={16} color="var(--blue-primary)" />
                  <span style={{ fontSize: '14px', fontWeight: 800, color: '#fff' }}>
                    Objective Spawn Timers
                  </span>
                </div>
                <div style={{ display: 'flex', gap: '6px' }}>
                  <button
                    onClick={() => setSrActiveTab('epics')}
                    style={{
                      padding: '4px 10px',
                      borderRadius: '4px',
                      fontSize: '11px',
                      fontWeight: 700,
                      cursor: 'pointer',
                      background: srActiveTab === 'epics' ? 'var(--blue-primary)' : 'rgba(255,255,255,0.04)',
                      color: srActiveTab === 'epics' ? '#000' : '#94a3b8',
                      border: '1px solid var(--border-subtle)',
                    }}
                  >
                    Epic Monsters
                  </button>
                  <button
                    onClick={() => setSrActiveTab('jungle')}
                    style={{
                      padding: '4px 10px',
                      borderRadius: '4px',
                      fontSize: '11px',
                      fontWeight: 700,
                      cursor: 'pointer',
                      background: srActiveTab === 'jungle' ? 'var(--blue-primary)' : 'rgba(255,255,255,0.04)',
                      color: srActiveTab === 'jungle' ? '#000' : '#94a3b8',
                      border: '1px solid var(--border-subtle)',
                    }}
                  >
                    Jungle Camps
                  </button>
                </div>
              </div>

              {srActiveTab === 'epics' ? (
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '12px' }}>
                  {srObjectives.map((obj) => (
                    <div
                      key={obj.id}
                      style={{
                        padding: '12px 14px',
                        borderRadius: '8px',
                        background: 'rgba(255, 255, 255, 0.02)',
                        border: `1px solid ${obj.color}40`,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                      }}
                    >
                      <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                        <span style={{ fontSize: '24px' }}>{obj.icon}</span>
                        <div>
                          <div style={{ fontSize: '12px', fontWeight: 800, color: '#fff' }}>{obj.name}</div>
                          <div style={{ fontSize: '10px', color: '#94a3b8' }}>Respawn: {formatTime(obj.total)}</div>
                        </div>
                      </div>
                      <div style={{ textAlign: 'right' }}>
                        <div
                          className="tabular-nums"
                          style={{
                            fontSize: '17px',
                            fontWeight: 800,
                            fontFamily: 'var(--font-timer)',
                            letterSpacing: '0.8px',
                            color: obj.respawnTime > 0 ? obj.color : '#22c55e',
                          }}
                        >
                          {obj.respawnTime > 0 ? formatTime(obj.respawnTime) : 'ALIVE'}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: '8px' }}>
                  {srJungleCamps.map((camp, idx) => (
                    <div
                      key={camp.name}
                      onClick={() => {
                        setSrJungleCamps((prev) => {
                          const copy = [...prev];
                          copy[idx].active = !copy[idx].active;
                          copy[idx].remaining = copy[idx].active ? copy[idx].respawnDuration : 0;
                          return copy;
                        });
                      }}
                      style={{
                        padding: '8px 12px',
                        borderRadius: '6px',
                        background: camp.active ? 'rgba(255, 42, 95, 0.1)' : 'rgba(255, 255, 255, 0.02)',
                        border: camp.active ? '1px solid var(--red-primary)' : '1px solid var(--border-subtle)',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        cursor: 'pointer',
                      }}
                    >
                      <span style={{ fontSize: '12px', color: '#fff' }}>
                        {camp.icon} {camp.name}
                      </span>
                      <span
                        style={{
                          fontSize: '11px',
                          fontFamily: 'var(--font-mono)',
                          fontWeight: 800,
                          color: camp.active ? 'var(--red-primary)' : '#22c55e',
                        }}
                      >
                        {camp.active ? formatTime(camp.remaining) : 'READY'}
                      </span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Enemy Summoner Spells */}
          {settings?.hudSpellTimers !== false && (
            <div className="glass-card" style={{ padding: '16px', minWidth: 0, width: '100%', boxSizing: 'border-box' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <ShieldAlert size={16} color="var(--red-primary)" />
                  <span style={{ fontSize: '14px', fontWeight: 800, color: '#fff' }}>
                    Enemy Summoner Spell Timers (Click to Start Countdown)
                  </span>
                </div>
                <span style={{ fontSize: '11px', color: '#64748b' }}>
                  Formatted ready times for League team chat
                </span>
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                {['TOP', 'JUNGLE', 'MID', 'ADC', 'SUPPORT'].map((role) => {
                  const roleSpells = srEnemySpells
                    .map((s, idx) => ({ ...s, index: idx }))
                    .filter((s) => s.role === role);

                  return (
                    <div
                      key={role}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        padding: '8px 14px',
                        borderRadius: '6px',
                        background: 'rgba(255, 255, 255, 0.02)',
                        border: '1px solid var(--border-subtle)',
                      }}
                    >
                      <div style={{ display: 'flex', alignItems: 'center', gap: '10px', minWidth: '85px' }}>
                        <span
                          style={{
                            fontSize: '10px',
                            fontWeight: 800,
                            padding: '2px 6px',
                            borderRadius: '4px',
                            background: 'rgba(255, 42, 95, 0.1)',
                            color: 'var(--red-primary)',
                            border: '1px solid var(--border-red)',
                          }}
                        >
                          {role}
                        </span>
                        <span style={{ fontSize: '12px', fontWeight: 700, color: '#fff' }}>
                          {roleSpells[0]?.champion || 'Enemy'}
                        </span>
                      </div>

                      <div style={{ display: 'flex', gap: '8px' }}>
                        {roleSpells.map((s) => (
                          <button
                            key={s.index}
                            onClick={() => triggerSrSpell(s.index)}
                            style={{
                              display: 'flex',
                              alignItems: 'center',
                              gap: '6px',
                              padding: '5px 10px',
                              borderRadius: '4px',
                              background: s.timerActive ? 'rgba(255, 42, 95, 0.15)' : 'rgba(255, 255, 255, 0.04)',
                              border: s.timerActive ? '1px solid var(--red-primary)' : '1px solid var(--border-subtle)',
                              cursor: 'pointer',
                              minWidth: '95px',
                            }}
                          >
                            <img
                              src={`https://ddragon.leagueoflegends.com/cdn/14.23.1/img/spell/Summoner${s.spellName === 'Ghost' ? 'Haste' : s.spellName === 'Ignite' ? 'Dot' : s.spellName === 'Cleanse' ? 'Boost' : s.spellName}.png`}
                              alt={s.spellName}
                              style={{ width: '18px', height: '18px', borderRadius: '3px' }}
                              onError={(e) => { (e.target as any).style.display = 'none'; }}
                            />
                            <div style={{ textAlign: 'left' }}>
                              <div style={{ fontSize: '10px', fontWeight: 700, color: '#fff' }}>{s.spellName}</div>
                              <div
                                style={{
                                  fontSize: '10px',
                                  fontFamily: 'var(--font-mono)',
                                  color: s.timerActive ? 'var(--red-primary)' : '#22c55e',
                                  fontWeight: 700,
                                }}
                              >
                                {s.timerActive ? formatTime(s.cooldownRemaining) : 'READY'}
                              </div>
                            </div>
                          </button>
                        ))}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}
        </div>
      )}

      {/* ======================================================== */}
      {/* MODE 2: ARAM MAYHEM (HOWLING ABYSS)                      */}
      {/* ======================================================== */}
      {activeMode === 'ARAM' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {/* Health Relics & Inhibitor */}
          <div className="glass-card" style={{ padding: '16px' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '14px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <Heart size={18} color="#22c55e" />
                <span style={{ fontSize: '14px', fontWeight: 800, color: '#fff' }}>
                  Health Relics (Shrines) & Inhibitor Timers
                </span>
              </div>
              <span style={{ fontSize: '11px', color: '#94a3b8' }}>
                Shrines respawn in 60s • Inhibitor in 5:00
              </span>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '10px', marginBottom: '12px' }}>
              {aramRelics.map((relic) => (
                <button
                  key={relic.id}
                  onClick={() => triggerAramRelic(relic.id)}
                  style={{
                    padding: '12px 14px',
                    borderRadius: '8px',
                    background: relic.active ? 'rgba(255, 42, 95, 0.12)' : 'rgba(34, 197, 94, 0.08)',
                    border: relic.active ? '1px solid var(--red-primary)' : '1px solid rgba(34, 197, 94, 0.3)',
                    cursor: 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                  }}
                >
                  <div style={{ textAlign: 'left' }}>
                    <div style={{ fontSize: '12px', fontWeight: 700, color: '#fff' }}>{relic.name}</div>
                    <div style={{ fontSize: '10px', color: '#94a3b8' }}>{relic.side} Bridge Zone</div>
                  </div>
                  <div
                    style={{
                      fontSize: '15px',
                      fontWeight: 800,
                      fontFamily: 'var(--font-mono)',
                      color: relic.active ? 'var(--red-primary)' : '#22c55e',
                    }}
                  >
                    {relic.active ? `${relic.respawnTime}s` : 'UP'}
                  </div>
                </button>
              ))}
            </div>

            {/* Inhibitor Tracker */}
            <div
              onClick={triggerAramInhibitor}
              style={{
                padding: '12px 16px',
                borderRadius: '8px',
                background: aramInhibitor.active ? 'rgba(255, 42, 95, 0.15)' : 'rgba(0, 200, 255, 0.06)',
                border: aramInhibitor.active ? '1px solid var(--red-primary)' : '1px solid var(--border-blue)',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                <Flame size={20} color={aramInhibitor.active ? 'var(--red-primary)' : 'var(--blue-primary)'} />
                <div>
                  <div style={{ fontSize: '13px', fontWeight: 800, color: '#fff' }}>Inhibitor Respawn Tracker</div>
                  <div style={{ fontSize: '10px', color: '#94a3b8' }}>Click when an inhibitor is destroyed (5:00 timer)</div>
                </div>
              </div>
              <div
                style={{
                  fontSize: '16px',
                  fontWeight: 800,
                  fontFamily: 'var(--font-mono)',
                  color: aramInhibitor.active ? 'var(--red-primary)' : '#22c55e',
                }}
              >
                {aramInhibitor.active ? formatTime(aramInhibitor.remaining) : 'INHIBITOR UP'}
              </div>
            </div>
          </div>

          {/* ARAM Summoner Spells (Haste Adjusted) */}
          <div className="glass-card" style={{ padding: '16px' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <Zap size={16} color="#00c8ff" />
                <span style={{ fontSize: '14px', fontWeight: 800, color: '#fff' }}>
                  ARAM Summoner Spells (Snowball & 70 Haste Flash)
                </span>
              </div>
              <span style={{ fontSize: '10px', color: '#64748b' }}>ARAM grants +70 Summoner Spell Haste</span>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '8px' }}>
              {aramSpells.map((s, idx) => (
                <button
                  key={idx}
                  onClick={() => triggerAramSpell(idx)}
                  style={{
                    padding: '8px 12px',
                    borderRadius: '6px',
                    background: s.active ? 'rgba(255, 42, 95, 0.12)' : 'rgba(255, 255, 255, 0.03)',
                    border: s.active ? '1px solid var(--red-primary)' : '1px solid var(--border-subtle)',
                    cursor: 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                  }}
                >
                  <div style={{ textAlign: 'left' }}>
                    <div style={{ fontSize: '11px', fontWeight: 700, color: '#fff' }}>{s.spell}</div>
                    <div style={{ fontSize: '10px', color: '#94a3b8' }}>{s.champion}</div>
                  </div>
                  <span
                    style={{
                      fontSize: '12px',
                      fontWeight: 800,
                      fontFamily: 'var(--font-mono)',
                      color: s.active ? 'var(--red-primary)' : '#22c55e',
                    }}
                  >
                    {s.active ? `${s.remaining}s` : 'READY'}
                  </span>
                </button>
              ))}
            </div>
          </div>

          {/* Riot ARAM Balance Auras (Buffs / Nerfs) */}
          <div className="glass-card" style={{ padding: '16px' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px', flexWrap: 'wrap', gap: '8px' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <Sparkles size={16} color="var(--blue-primary)" />
                <span style={{ fontSize: '14px', fontWeight: 800, color: '#fff' }}>
                  Riot ARAM Balance Multipliers (Hidden Buffs / Nerfs)
                </span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                <Search size={13} color="#94a3b8" />
                <input
                  type="text"
                  placeholder="Filter champion..."
                  value={auraSearch}
                  onChange={(e) => setAuraSearch(e.target.value)}
                  style={{
                    padding: '4px 8px',
                    borderRadius: '4px',
                    background: 'rgba(0, 0, 0, 0.3)',
                    border: '1px solid var(--border-subtle)',
                    color: '#fff',
                    fontSize: '11px',
                    width: '130px',
                  }}
                />
              </div>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '8px', maxHeight: '240px', overflowY: 'auto' }}>
              {aramAuras
                .filter((a) => a.champ.toLowerCase().includes(auraSearch.toLowerCase()))
                .map((a) => (
                  <div
                    key={a.champ}
                    style={{
                      padding: '8px 12px',
                      borderRadius: '6px',
                      background: 'rgba(255, 255, 255, 0.02)',
                      border: '1px solid var(--border-subtle)',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                    }}
                  >
                    <div>
                      <strong style={{ fontSize: '12px', color: '#fff' }}>{a.champ}</strong>
                      <div style={{ fontSize: '10px', color: '#94a3b8' }}>{a.note}</div>
                    </div>
                    <div style={{ display: 'flex', gap: '6px', fontSize: '11px', fontWeight: 800 }}>
                      <span style={{ color: a.dealt.startsWith('+') ? '#22c55e' : '#ff2a5f' }}>
                        Dmg: {a.dealt}
                      </span>
                      <span style={{ color: a.taken.startsWith('-') ? '#22c55e' : '#ff2a5f' }}>
                        Taken: {a.taken}
                      </span>
                    </div>
                  </div>
                ))}
            </div>
          </div>

          {/* ARAM Mayhem Augment Cards & In-Round Selector */}
          {settings?.hudMayhemCards !== false && (
            <div className="glass-card" style={{ padding: '16px', minWidth: 0, width: '100%', boxSizing: 'border-box' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px', flexWrap: 'wrap', gap: '8px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <Sparkles size={16} color="#c084fc" />
                  <span style={{ fontSize: '14px', fontWeight: 800, color: '#fff' }}>
                    ARAM Mayhem Cards & Instant Tier Advice
                  </span>
                  <span style={{ fontSize: '10px', color: '#94a3b8' }}>
                    (Draft Rounds)
                  </span>
                </div>

                {/* Filters */}
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <div style={{ display: 'flex', gap: '3px' }}>
                    {(['ALL', 'Prismatic', 'Gold', 'Silver'] as const).map((tier) => (
                      <button
                        key={tier}
                        onClick={() => setAramCardTierFilter(tier)}
                        style={{
                          fontSize: '9px',
                          fontWeight: 800,
                          padding: '2px 7px',
                          borderRadius: '4px',
                          background: aramCardTierFilter === tier ? 'rgba(255, 255, 255, 0.14)' : 'rgba(255, 255, 255, 0.03)',
                          border: aramCardTierFilter === tier ? '1px solid rgba(255, 255, 255, 0.3)' : '1px solid transparent',
                          color: tier === 'Prismatic' ? '#c084fc' : tier === 'Gold' ? '#f59e0b' : tier === 'Silver' ? '#94a3b8' : '#fff',
                          cursor: 'pointer',
                        }}
                      >
                        {tier}
                      </button>
                    ))}
                  </div>

                  <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                    <Search size={12} color="#94a3b8" />
                    <input
                      type="text"
                      placeholder="Search Mayhem card..."
                      value={aramCardSearch}
                      onChange={(e) => setAramCardSearch(e.target.value)}
                      style={{
                        padding: '4px 8px',
                        borderRadius: '4px',
                        background: 'rgba(0, 0, 0, 0.35)',
                        border: '1px solid var(--border-subtle)',
                        color: '#fff',
                        fontSize: '11px',
                        width: '140px',
                      }}
                    />
                  </div>
                </div>
              </div>

              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))', gap: '8px', maxHeight: '280px', overflowY: 'auto' }}>
                {searchMayhemCards(aramCardSearch, aramCardTierFilter, 'ARAM').map((card) => (
                  <div
                    key={card.id}
                    style={{
                      background: 'rgba(255, 255, 255, 0.02)',
                      border: `1px solid ${card.color}33`,
                      borderLeft: `3px solid ${card.color}`,
                      borderRadius: '6px',
                      padding: '8px 10px',
                      display: 'flex',
                      flexDirection: 'column',
                      gap: '3px',
                    }}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                        <span style={{ fontSize: '12px', fontWeight: 800, color: '#fff' }}>{card.name}</span>
                        <span
                          style={{
                            fontSize: '8.5px',
                            fontWeight: 800,
                            padding: '1px 5px',
                            borderRadius: '3px',
                            background: `${card.color}18`,
                            color: card.color,
                          }}
                        >
                          {card.tier}
                        </span>
                        <span
                          style={{
                            fontSize: '8.5px',
                            fontWeight: 900,
                            padding: '1px 5px',
                            borderRadius: '3px',
                            background: card.rank === 'S+' ? 'rgba(239, 68, 68, 0.2)' : 'rgba(34, 197, 94, 0.2)',
                            color: card.rank === 'S+' ? '#ef4444' : '#22c55e',
                          }}
                        >
                          {card.rank} Tier
                        </span>
                      </div>
                      <span style={{ fontSize: '8.5px', color: '#64748b', fontWeight: 700 }}>{card.tag}</span>
                    </div>
                    <div style={{ fontSize: '10.5px', color: '#cbd5e1', lineHeight: '1.35' }}>
                      {card.description}
                    </div>
                    <div style={{ fontSize: '9.5px', color: '#94a3b8' }}>
                      <strong style={{ color: 'var(--blue-primary)' }}>Best For:</strong> {card.archetype}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* ======================================================== */}
      {/* MODE 3: ARENA 2v2v2v2 (CHERRY)                           */}
      {/* ======================================================== */}
      {activeMode === 'CHERRY' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {/* Power Flower & Ring of Fire */}
          <div className="glass-card" style={{ padding: '16px' }}>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '12px' }}>
              {/* Power Flower Plant */}
              <div
                onClick={triggerArenaFlower}
                style={{
                  padding: '12px 16px',
                  borderRadius: '8px',
                  background: arenaFlower.active ? 'rgba(255, 42, 95, 0.12)' : 'rgba(34, 197, 94, 0.1)',
                  border: arenaFlower.active ? '1px solid var(--red-primary)' : '1px solid #22c55e',
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                }}
              >
                <div>
                  <div style={{ fontSize: '13px', fontWeight: 800, color: '#fff' }}>🌸 Power Flower Plant</div>
                  <div style={{ fontSize: '10px', color: '#94a3b8' }}>3 hits: 20% HP/Shield & 20% CDR refund</div>
                </div>
                <div
                  style={{
                    fontSize: '16px',
                    fontWeight: 800,
                    fontFamily: 'var(--font-mono)',
                    color: arenaFlower.active ? 'var(--red-primary)' : '#22c55e',
                  }}
                >
                  {arenaFlower.active ? `${arenaFlower.remaining}s` : 'READY'}
                </div>
              </div>

              {/* Ring of Fire */}
              <div
                style={{
                  padding: '12px 16px',
                  borderRadius: '8px',
                  background: 'rgba(255, 170, 0, 0.1)',
                  border: '1px solid rgba(255, 170, 0, 0.3)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                }}
              >
                <div>
                  <div style={{ fontSize: '13px', fontWeight: 800, color: '#fff' }}>🔥 Ring of Fire (Sudden Death)</div>
                  <div style={{ fontSize: '10px', color: '#94a3b8' }}>Starts constricting inward at 50s</div>
                </div>
                <span style={{ fontSize: '11px', fontWeight: 800, color: '#fbbf24' }}>
                  BURN DAMAGE
                </span>
              </div>
            </div>
          </div>

          {/* Cameo Hazard Guide */}
          <div className="glass-card" style={{ padding: '14px 18px' }}>
            <div style={{ fontSize: '13px', fontWeight: 800, color: '#fff', marginBottom: '8px' }}>
              ⚠️ Round Cameo Event Hazards
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '8px' }}>
              <div style={{ padding: '8px', borderRadius: '4px', background: 'rgba(255,255,255,0.02)', fontSize: '11px' }}>
                <strong style={{ color: '#fbbf24' }}>Sett:</strong> Takes damage then punches W cone. Bait enemies into it!
              </div>
              <div style={{ padding: '8px', borderRadius: '4px', background: 'rgba(255,255,255,0.02)', fontSize: '11px' }}>
                <strong style={{ color: '#00c8ff' }}>Pyke:</strong> Dashes and stuns lowest HP targets. Side-step sideways!
              </div>
              <div style={{ padding: '8px', borderRadius: '4px', background: 'rgba(255,255,255,0.02)', fontSize: '11px' }}>
                <strong style={{ color: '#22c55e' }}>Thresh:</strong> Throws lanterns across ring. Click to escape or reposition!
              </div>
              <div style={{ padding: '8px', borderRadius: '4px', background: 'rgba(255,255,255,0.02)', fontSize: '11px' }}>
                <strong style={{ color: '#ff2a5f' }}>Jhin:</strong> Snipes execute 4th shot. Block for low HP teammate!
              </div>
            </div>
          </div>

          {/* In-Round Augment Search & Tier List */}
          {settings?.hudMayhemCards !== false && (
            <div className="glass-card" style={{ padding: '16px', minWidth: 0, width: '100%', boxSizing: 'border-box' }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '12px', flexWrap: 'wrap', gap: '8px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <Sparkles size={16} color="var(--blue-primary)" />
                  <span style={{ fontSize: '14px', fontWeight: 800, color: '#fff' }}>
                    Arena Augment Companion & Instant Tier Advice
                  </span>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <Search size={13} color="#94a3b8" />
                  <input
                    type="text"
                    placeholder="Search augment name..."
                    value={augmentSearch}
                    onChange={(e) => setAugmentSearch(e.target.value)}
                    style={{
                      padding: '4px 10px',
                      borderRadius: '4px',
                      background: 'rgba(0,0,0,0.3)',
                      border: '1px solid var(--border-subtle)',
                      color: '#fff',
                      fontSize: '11px',
                      width: '160px',
                    }}
                  />
                </div>
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', maxHeight: '300px', overflowY: 'auto' }}>
                {arenaAugments
                  .filter((aug) => aug.name.toLowerCase().includes(augmentSearch.toLowerCase()) || aug.role.toLowerCase().includes(augmentSearch.toLowerCase()))
                  .map((aug) => (
                    <div
                      key={aug.name}
                      style={{
                        padding: '10px 14px',
                        borderRadius: '6px',
                        background: 'rgba(255, 255, 255, 0.02)',
                        border: '1px solid var(--border-subtle)',
                        display: 'flex',
                        alignItems: 'flex-start',
                        justifyContent: 'space-between',
                        gap: '12px',
                      }}
                    >
                      <div>
                        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                          <span style={{ fontSize: '13px', fontWeight: 800, color: '#fff' }}>{aug.name}</span>
                          <span
                            style={{
                              fontSize: '9px',
                              fontWeight: 800,
                              padding: '1px 6px',
                              borderRadius: '3px',
                              background:
                                aug.tier === 'Prismatic'
                                  ? 'rgba(192, 132, 252, 0.2)'
                                  : aug.tier === 'Gold'
                                  ? 'rgba(250, 204, 21, 0.2)'
                                  : 'rgba(148, 163, 184, 0.2)',
                              color:
                                aug.tier === 'Prismatic'
                                  ? '#c084fc'
                                  : aug.tier === 'Gold'
                                  ? '#facc15'
                                  : '#94a3b8',
                              border: `1px solid ${
                                aug.tier === 'Prismatic'
                                  ? '#c084fc'
                                  : aug.tier === 'Gold'
                                  ? '#facc15'
                                  : '#94a3b8'
                              }`,
                            }}
                          >
                            {aug.tier}
                          </span>
                          <span style={{ fontSize: '10px', color: '#94a3b8' }}>Best on: {aug.role}</span>
                        </div>
                        <div style={{ fontSize: '11px', color: '#cbd5e1', marginTop: '4px', lineHeight: '1.4' }}>
                          {aug.description}
                        </div>
                      </div>
                      <span
                        style={{
                          fontSize: '12px',
                          fontWeight: 900,
                          color: 'var(--blue-primary)',
                          padding: '2px 8px',
                          borderRadius: '4px',
                          background: 'rgba(0, 200, 255, 0.1)',
                          border: '1px solid var(--border-blue)',
                        }}
                      >
                        {aug.rank}
                      </span>
                    </div>
                  ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* ======================================================== */}
      {/* MODE 4: URF / ARURF                                      */}
      {/* ======================================================== */}
      {activeMode === 'URF' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {/* URF Buff Dashboard */}
          <div className="glass-card" style={{ padding: '16px' }}>
            <div style={{ fontSize: '14px', fontWeight: 800, color: '#fff', marginBottom: '12px' }}>
              ⚡ 80% CDR Ultra Rapid Fire Buff Matrix
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: '8px' }}>
              <div style={{ padding: '8px 12px', borderRadius: '4px', background: 'rgba(0,200,255,0.06)', border: '1px solid var(--border-blue)' }}>
                <div style={{ fontSize: '10px', color: '#94a3b8' }}>Ability Haste</div>
                <div style={{ fontSize: '14px', fontWeight: 800, color: '#00c8ff' }}>300 AH (80% CDR)</div>
              </div>
              <div style={{ padding: '8px 12px', borderRadius: '4px', background: 'rgba(0,200,255,0.06)', border: '1px solid var(--border-blue)' }}>
                <div style={{ fontSize: '10px', color: '#94a3b8' }}>Mana & Energy</div>
                <div style={{ fontSize: '14px', fontWeight: 800, color: '#22c55e' }}>100% Free / Zero Cost</div>
              </div>
              <div style={{ padding: '8px 12px', borderRadius: '4px', background: 'rgba(0,200,255,0.06)', border: '1px solid var(--border-blue)' }}>
                <div style={{ fontSize: '10px', color: '#94a3b8' }}>Tenacity</div>
                <div style={{ fontSize: '14px', fontWeight: 800, color: '#facc15' }}>+25% Base Tenacity</div>
              </div>
              <div style={{ padding: '8px 12px', borderRadius: '4px', background: 'rgba(0,200,255,0.06)', border: '1px solid var(--border-blue)' }}>
                <div style={{ fontSize: '10px', color: '#94a3b8' }}>Movement Speed</div>
                <div style={{ fontSize: '14px', fontWeight: 800, color: '#c084fc' }}>+60 Bonus Flat MS</div>
              </div>
            </div>
          </div>

          {/* 60s URF Flashes */}
          <div className="glass-card" style={{ padding: '16px' }}>
            <div style={{ fontSize: '14px', fontWeight: 800, color: '#fff', marginBottom: '12px' }}>
              ⚡ 60-Second URF Enemy Flash Trackers
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: '8px' }}>
              {urfFlashTimers.map((f, idx) => (
                <button
                  key={idx}
                  onClick={() => triggerUrfFlash(idx)}
                  style={{
                    padding: '8px 12px',
                    borderRadius: '6px',
                    background: f.active ? 'rgba(255, 42, 95, 0.15)' : 'rgba(255, 255, 255, 0.03)',
                    border: f.active ? '1px solid var(--red-primary)' : '1px solid var(--border-subtle)',
                    cursor: 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                  }}
                >
                  <span style={{ fontSize: '11px', fontWeight: 700, color: '#fff' }}>{f.player}</span>
                  <span
                    style={{
                      fontSize: '12px',
                      fontWeight: 800,
                      fontFamily: 'var(--font-mono)',
                      color: f.active ? 'var(--red-primary)' : '#22c55e',
                    }}
                  >
                    {f.active ? `${f.remaining}s` : 'FLASH READY'}
                  </span>
                </button>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* ======================================================== */}
      {/* MODE 5: ONE FOR ALL                                      */}
      {/* ======================================================== */}
      {activeMode === 'ONEFORALL' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {/* 5x Synergy Stacking Rules */}
          <div className="glass-card" style={{ padding: '16px' }}>
            <div style={{ fontSize: '14px', fontWeight: 800, color: '#fff', marginBottom: '12px' }}>
              👥 5x Champion Passive Stacking Synergies
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(260px, 1fr))', gap: '10px' }}>
              <div style={{ padding: '10px', borderRadius: '6px', background: 'rgba(255,255,255,0.02)', border: '1px solid var(--border-subtle)' }}>
                <strong style={{ color: '#ff2a5f' }}>🩸 Darius (Shared Bleed):</strong>
                <div style={{ fontSize: '11px', color: '#cbd5e1', marginTop: '4px' }}>
                  Hemorrhage bleed stacks are shared across all 5 Dariuses! 1 auto-attack from each triggers Noxian Might for all 5 instantly.
                </div>
              </div>
              <div style={{ padding: '10px', borderRadius: '6px', background: 'rgba(255,255,255,0.02)', border: '1px solid var(--border-subtle)' }}>
                <strong style={{ color: '#00c8ff' }}>🐸 Tahm Kench (Shared Stacks):</strong>
                <div style={{ fontSize: '11px', color: '#cbd5e1', marginTop: '4px' }}>
                  3 stacks of An Acquired Taste are shared. Any frog on your team can Devour the stunned enemy champion!
                </div>
              </div>
              <div style={{ padding: '10px', borderRadius: '6px', background: 'rgba(255,255,255,0.02)', border: '1px solid var(--border-subtle)' }}>
                <strong style={{ color: '#fbbf24' }}>🔥 Brand (Triple Explosion):</strong>
                <div style={{ fontSize: '11px', color: '#cbd5e1', marginTop: '4px' }}>
                  Blaze passive stacks trigger explosive chain reactions on targets hit by multiple Brands simultaneously.
                </div>
              </div>
              <div style={{ padding: '10px', borderRadius: '6px', background: 'rgba(255,255,255,0.02)', border: '1px solid var(--border-subtle)' }}>
                <strong style={{ color: '#c084fc' }}>🔮 Morgana (15s Root Chain):</strong>
                <div style={{ fontSize: '11px', color: '#cbd5e1', marginTop: '4px' }}>
                  Chain Dark Binding Q snares (3s each) into overlapping W Tormented Shadow pools for inescapable 100-to-0 kills.
                </div>
              </div>
            </div>
          </div>

          {/* Anti-Stacking Item Counter-Builds */}
          <div className="glass-card" style={{ padding: '16px' }}>
            <div style={{ fontSize: '14px', fontWeight: 800, color: '#fff', marginBottom: '12px' }}>
              🛡️ Mirror Matchup Counter-Itemization
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '10px' }}>
              <div style={{ padding: '10px', borderRadius: '6px', background: 'rgba(0, 200, 255, 0.05)', border: '1px solid var(--border-blue)' }}>
                <strong style={{ color: '#00c8ff' }}>Against 5x AP Mages:</strong>
                <div style={{ fontSize: '11px', color: '#cbd5e1', marginTop: '4px' }}>
                  Rush <strong>Kaenic Rookern</strong> (20% max HP magic shield) + <strong>Force of Nature</strong> (+70 MR, 10% MS) + <strong>Mercury Treads</strong>. Makes you completely unkillable.
                </div>
              </div>
              <div style={{ padding: '10px', borderRadius: '6px', background: 'rgba(255, 42, 95, 0.05)', border: '1px solid var(--border-red)' }}>
                <strong style={{ color: '#ff2a5f' }}>Against 5x AD / Marksmen:</strong>
                <div style={{ fontSize: '11px', color: '#cbd5e1', marginTop: '4px' }}>
                  Rush <strong>Frozen Heart</strong> (-20% AS aura) + <strong>Randuin's Omen</strong> (-30% Crit Damage) + <strong>Plated Steelcaps</strong> (-12% basic attack damage).
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
