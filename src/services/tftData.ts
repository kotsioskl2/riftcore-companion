// Teamfight Tactics (TFT) Meta, Compositions, Items, Rolling Odds & Augments Service
// Current Set 13: Into the Arcane / Meta Companion Engine

export interface TftChampion {
  name: string;
  cost: 1 | 2 | 3 | 4 | 5;
  traits: string[];
  role: 'carry' | 'tank' | 'support' | 'flex';
  recommendedItems?: string[];
  iconUrl?: string;
  position?: { row: number; col: number }; // 0-3 rows, 0-6 cols
}

export interface TftComp {
  id: string;
  name: string;
  tier: 'S+' | 'S' | 'A' | 'B';
  difficulty: 'Easy' | 'Medium' | 'Hard';
  playstyle: 'Fast 8' | 'Slow Roll Lvl 6' | 'Slow Roll Lvl 7' | 'Hyperroll' | 'Fast 9';
  tagline: string;
  traits: { name: string; count: number; activeTier: number }[];
  champions: TftChampion[];
  mainCarry: {
    champion: string;
    items: string[];
  };
  mainTank: {
    champion: string;
    items: string[];
  };
  earlyGame: string[];
  levelingGuide: string;
  positioningTip: string;
}

export interface TftItemComponent {
  id: string;
  name: string;
  icon: string;
  stats: string;
  description: string;
}

export interface TftCombinedItem {
  id: string;
  name: string;
  component1: string;
  component2: string;
  stats: string;
  description: string;
  category: 'AD Carry' | 'AP Carry' | 'Tank' | 'Utility' | 'Emblem' | 'Radiant' | 'Artifact';
  bestChampions: string[];
}

export interface TftRollOdds {
  level: number;
  cost1: number;
  cost2: number;
  cost3: number;
  cost4: number;
  cost5: number;
  recommendedRound: string;
  xpToNext: number;
  goldCostEstimate: number;
}

export interface TftAugment {
  id: string;
  name: string;
  tier: 'Silver' | 'Gold' | 'Prismatic';
  winRate: number;
  avgPlacement: number;
  description: string;
  recommendedComps: string[];
}

// 1. TFT Base Item Components
export const TFT_COMPONENTS: TftItemComponent[] = [
  { id: 'sword', name: 'B.F. Sword', icon: '🗡️', stats: '+10 Attack Damage', description: 'Raw physical damage component.' },
  { id: 'bow', name: 'Recurve Bow', icon: '🏹', stats: '+10% Attack Speed', description: 'Attack speed for DPS carries.' },
  { id: 'rod', name: 'Needlessly Large Rod', icon: '🪄', stats: '+10 Ability Power', description: 'Magic spell amplification component.' },
  { id: 'tear', name: 'Tear of the Goddess', icon: '💧', stats: '+15 Starting Mana', description: 'Faster spell cast frequency.' },
  { id: 'vest', name: 'Chain Vest', icon: '🛡️', stats: '+20 Armor', description: 'Physical defense mitigation.' },
  { id: 'cloak', name: 'Negatron Cloak', icon: '🪖', stats: '+20 Magic Resist', description: 'Magic damage mitigation.' },
  { id: 'belt', name: "Giant's Belt", icon: '🥋', stats: '+150 Maximum Health', description: 'Health buffer for frontliners.' },
  { id: 'gloves', name: 'Sparring Gloves', icon: '🥊', stats: '+20% Critical Strike', description: 'Critical chance and spell crit unlock.' },
  { id: 'spatula', name: 'Spatula', icon: '🍳', stats: 'Special Emblem Recipe', description: 'Crafts powerful trait emblems.' },
];

// 2. TFT Combined Items Matrix (Recipes)
export const TFT_COMBINED_ITEMS: TftCombinedItem[] = [
  {
    id: 'infinity_edge',
    name: 'Infinity Edge',
    component1: 'sword',
    component2: 'gloves',
    stats: '+35 AD, +35% Crit Chance',
    description: 'Abilities can critically strike. Grants 10% Critical Strike Damage.',
    category: 'AD Carry',
    bestChampions: ['Caitlyn', 'Ezreal', 'Draven', 'Corki'],
  },
  {
    id: 'deathblade',
    name: 'Deathblade',
    component1: 'sword',
    component2: 'sword',
    stats: '+55 Attack Damage',
    description: 'Provides immense bonus Attack Damage to basic attacks and physical abilities.',
    category: 'AD Carry',
    bestChampions: ['Ambessa', 'Caitlyn', 'Draven'],
  },
  {
    id: 'bloodthirster',
    name: 'Bloodthirster',
    component1: 'sword',
    component2: 'cloak',
    stats: '+20 AD, +20 MR, +15 AP',
    description: 'Grants 20% Omnivamp. Once per combat at 40% HP, gain a 25% max HP shield for 5s.',
    category: 'AD Carry',
    bestChampions: ['Ambessa', 'Vi', 'Warwick', 'Sett'],
  },
  {
    id: 'giant_slayer',
    name: 'Giant Slayer',
    component1: 'sword',
    component2: 'bow',
    stats: '+25 AD, +10% AS, +10 AP',
    description: 'Deal 25% more damage against enemies with more than 1,750 max Health.',
    category: 'AD Carry',
    bestChampions: ['Caitlyn', 'Corki', 'Cassiopeia', 'Ezreal'],
  },
  {
    id: 'hextech_gunblade',
    name: 'Hextech Gunblade',
    component1: 'sword',
    component2: 'rod',
    stats: '+20 AD, +20 AP',
    description: 'Grants 20% Omnivamp. Heals the lowest-percent Health ally for the same amount.',
    category: 'Utility',
    bestChampions: ['Silco', 'Cassiopeia', 'Malzahar', 'Heimerdinger'],
  },
  {
    id: 'spear_of_shojin',
    name: 'Spear of Shojin',
    component1: 'sword',
    component2: 'tear',
    stats: '+20 AD, +15 AP, +15 Mana',
    description: 'Attacks grant 5 bonus Mana. Essential for fast ability cycling.',
    category: 'Utility',
    bestChampions: ['Silco', 'Cassiopeia', 'Nami', 'Ezreal'],
  },
  {
    id: 'edge_of_night',
    name: 'Edge of Night',
    component1: 'sword',
    component2: 'vest',
    stats: '+25 AD, +20 Armor',
    description: 'Once per combat at 60% HP, briefly become invulnerable, shed negative effects, and gain 15% AS.',
    category: 'AD Carry',
    bestChampions: ['Ambessa', 'Vi', 'Ekko', 'Draven'],
  },
  {
    id: 'steraks_gage',
    name: "Sterak's Gage",
    component1: 'sword',
    component2: 'belt',
    stats: '+15 AD, +200 HP',
    description: 'Once per combat at 60% HP, gain 25% max HP and 35% AD for the rest of combat.',
    category: 'Tank',
    bestChampions: ['Ambessa', 'Vi', 'Sett', 'Trundle'],
  },
  {
    id: 'guinsoo_rageblade',
    name: "Guinsoo's Rageblade",
    component1: 'bow',
    component2: 'rod',
    stats: '+10% AS, +10 AP',
    description: 'Attacks grant +5% stacking Attack Speed for the rest of combat. Stacks infinitely.',
    category: 'AD Carry',
    bestChampions: ['Corki', 'Ezreal', 'Cassiopeia', 'Zari'],
  },
  {
    id: 'last_whisper',
    name: 'Last Whisper',
    component1: 'bow',
    component2: 'gloves',
    stats: '+15 AD, +25% AS, +20% Crit',
    description: 'Physical damage Sunders target Armor by 30% for 3s. Does not stack.',
    category: 'AD Carry',
    bestChampions: ['Ezreal', 'Caitlyn', 'Corki', 'Draven'],
  },
  {
    id: 'statikk_shiv',
    name: 'Statikk Shiv',
    component1: 'bow',
    component2: 'tear',
    stats: '+15% AS, +15 AP, +15 Mana',
    description: 'Every 3rd attack unleashes chain lightning dealing magic damage and Shredding 30% MR.',
    category: 'Utility',
    bestChampions: ['Ezreal', 'Heimerdinger', 'Silco'],
  },
  {
    id: 'titans_resolve',
    name: "Titan's Resolve",
    component1: 'bow',
    component2: 'vest',
    stats: '+10% AS, +20 Armor',
    description: 'Gain 2% AD and 2 AP when attacking or taking damage (stacks up to 25). At max stacks, gain 20 Armor & MR.',
    category: 'Tank',
    bestChampions: ['Ambessa', 'Vi', 'Sett', 'Blitzcrank'],
  },
  {
    id: 'rabadons_deathcap',
    name: "Rabadon's Deathcap",
    component1: 'rod',
    component2: 'rod',
    stats: '+50 Ability Power',
    description: 'Grants +20% bonus magic damage amplification.',
    category: 'AP Carry',
    bestChampions: ['Silco', 'Cassiopeia', 'Malzahar', 'Heimerdinger'],
  },
  {
    id: 'jeweled_gauntlet',
    name: 'Jeweled Gauntlet',
    component1: 'rod',
    component2: 'gloves',
    stats: '+35 AP, +35% Crit Chance',
    description: 'Abilities can critically strike. Grants 10% Critical Strike Damage.',
    category: 'AP Carry',
    bestChampions: ['Silco', 'Cassiopeia', 'Malzahar', 'Nami'],
  },
  {
    id: 'archangels_staff',
    name: "Archangel's Staff",
    component1: 'rod',
    component2: 'tear',
    stats: '+20 AP, +15 Mana',
    description: 'Combat start: Gain 30 AP every 5 seconds.',
    category: 'AP Carry',
    bestChampions: ['Cassiopeia', 'Malzahar', 'Silco', 'Singed'],
  },
  {
    id: 'morellonomicon',
    name: 'Morellonomicon',
    component1: 'rod',
    component2: 'belt',
    stats: '+25 AP, +150 HP, +10% AS',
    description: 'Magic and True damage Burns targets for 1% max HP per second and inflicts 33% Wound.',
    category: 'Utility',
    bestChampions: ['Heimerdinger', 'Malzahar', 'Silco', 'Singed'],
  },
  {
    id: 'blue_buff',
    name: 'Blue Buff',
    component1: 'tear',
    component2: 'tear',
    stats: '+30 Starting Mana, +10 AP, +10 AD',
    description: 'Reduces max Mana by 10. Scoring a takedown deals 8% more damage for 8s.',
    category: 'AP Carry',
    bestChampions: ['Cassiopeia', 'Malzahar', 'Silco', 'Ezreal'],
  },
  {
    id: 'warmogs_armor',
    name: "Warmog's Armor",
    component1: 'belt',
    component2: 'belt',
    stats: '+800 Maximum Health',
    description: 'Grants an enormous flat Health buffer and +12% maximum Health.',
    category: 'Tank',
    bestChampions: ['Illaoi', 'Sion', 'Blitzcrank', 'Singed'],
  },
  {
    id: 'bramble_vest',
    name: 'Bramble Vest',
    component1: 'vest',
    component2: 'vest',
    stats: '+55 Armor, +100 HP',
    description: 'Reduces critical strike damage taken by 8%. Reflects magic damage when struck.',
    category: 'Tank',
    bestChampions: ['Illaoi', 'Sion', 'Blitzcrank', 'Loris'],
  },
  {
    id: 'dragons_claw',
    name: "Dragon's Claw",
    component1: 'cloak',
    component2: 'cloak',
    stats: '+65 Magic Resist, +100 HP',
    description: 'Every 2 seconds, regenerate 5% of maximum Health. Heavy magic damage shield.',
    category: 'Tank',
    bestChampions: ['Illaoi', 'Sion', 'Loris', 'Renni'],
  },
  {
    id: 'sunfire_cape',
    name: 'Sunfire Cape',
    component1: 'vest',
    component2: 'belt',
    stats: '+20 Armor, +250 HP',
    description: 'Every 2 seconds, burns an enemy within 2 hexes for 1% max HP and inflicts 33% Wound.',
    category: 'Tank',
    bestChampions: ['Illaoi', 'Sion', 'Singed', 'Blitzcrank'],
  },
  {
    id: 'redemption',
    name: 'Redemption',
    component1: 'tear',
    component2: 'belt',
    stats: '+150 HP, +15 Mana',
    description: 'Heals adjacent allies for 15% of missing Health every 5s and reduces AOE damage taken by 10%.',
    category: 'Tank',
    bestChampions: ['Illaoi', 'Blitzcrank', 'Steb', 'Singed'],
  },
  {
    id: 'gargoyle_stoneplate',
    name: 'Gargoyle Stoneplate',
    component1: 'vest',
    component2: 'cloak',
    stats: '+35 Armor, +35 MR, +100 HP',
    description: 'Gain 10 Armor and 10 MR for each enemy targeting the holder.',
    category: 'Tank',
    bestChampions: ['Illaoi', 'Sion', 'Blitzcrank', 'Loris'],
  },
  {
    id: 'quicksilver',
    name: 'Quicksilver',
    component1: 'gloves',
    component2: 'cloak',
    stats: '+30% AS, +20 MR, +20% Crit',
    description: 'Immune to crowd control for the first 14 seconds of combat. Grants +4% stacking AS.',
    category: 'Utility',
    bestChampions: ['Ambessa', 'Vi', 'Warwick', 'Draven'],
  },
];

// 3. TFT Current Top Meta Compositions (S+, S, A Tiers)
export const TFT_META_COMPS: TftComp[] = [
  {
    id: 'rebel_snipers',
    name: '6 Enforcer 2 Sniper Caitlyn',
    tier: 'S+',
    difficulty: 'Medium',
    playstyle: 'Fast 8',
    tagline: 'High burst physical execute with Caitlyn locking down backline carries behind an impenetrable frontline.',
    traits: [
      { name: 'Enforcer', count: 6, activeTier: 3 },
      { name: 'Sniper', count: 2, activeTier: 1 },
      { name: 'Pit Fighter', count: 2, activeTier: 1 },
      { name: 'Bruiser', count: 2, activeTier: 1 },
    ],
    champions: [
      { name: 'Caitlyn', cost: 5, traits: ['Enforcer', 'Sniper'], role: 'carry', position: { row: 3, col: 6 } },
      { name: 'Vi', cost: 4, traits: ['Enforcer', 'Pit Fighter'], role: 'carry', position: { row: 0, col: 2 } },
      { name: 'Illaoi', cost: 4, traits: ['Rebel', 'Bruiser'], role: 'tank', position: { row: 0, col: 3 } },
      { name: 'Loris', cost: 3, traits: ['Enforcer', 'Sentinel'], role: 'tank', position: { row: 0, col: 4 } },
      { name: 'Twisted Fate', cost: 3, traits: ['Enforcer', 'Quickstriker'], role: 'support', position: { row: 3, col: 0 } },
      { name: 'Maddox', cost: 2, traits: ['Enforcer'], role: 'tank', position: { row: 0, col: 1 } },
      { name: 'Steb', cost: 1, traits: ['Enforcer', 'Bruiser'], role: 'tank', position: { row: 0, col: 5 } },
      { name: 'Ezreal', cost: 3, traits: ['Rebel', 'Sniper'], role: 'carry', position: { row: 3, col: 5 } },
    ],
    mainCarry: {
      champion: 'Caitlyn',
      items: ['Infinity Edge', 'Last Whisper', 'Deathblade'],
    },
    mainTank: {
      champion: 'Illaoi',
      items: ["Warmog's Armor", 'Bramble Vest', "Dragon's Claw"],
    },
    earlyGame: ['Steb', 'Maddox', 'Ezreal', 'Draven'],
    levelingGuide: 'Level to 4 at 2-1, Level 5 at 2-5. Save 50 gold and Fast 8 on Round 4-2 to roll for Caitlyn & Vi.',
    positioningTip: 'Place Caitlyn in the far right back corner (Row 3, Col 6) protected by Loris and Illaoi directly in front.',
  },
  {
    id: 'chem_baron_silco',
    name: '4 Chem-Baron 4 Dominance Silco',
    tier: 'S+',
    difficulty: 'Hard',
    playstyle: 'Fast 8',
    tagline: 'Scale immense shimmer power with Silco throwing unstable vials that shred entire clumped teams.',
    traits: [
      { name: 'Chem-Baron', count: 4, activeTier: 2 },
      { name: 'Dominance', count: 4, activeTier: 2 },
      { name: 'Visionary', count: 2, activeTier: 1 },
      { name: 'Bruiser', count: 2, activeTier: 1 },
    ],
    champions: [
      { name: 'Silco', cost: 4, traits: ['Chem-Baron', 'Dominance'], role: 'carry', position: { row: 3, col: 3 } },
      { name: 'Sevika', cost: 5, traits: ['High Roller', 'Pit Fighter'], role: 'carry', position: { row: 1, col: 3 } },
      { name: 'Singed', cost: 1, traits: ['Chem-Baron', 'Sentinel'], role: 'tank', position: { row: 0, col: 2 } },
      { name: 'Renni', cost: 3, traits: ['Chem-Baron', 'Bruiser'], role: 'tank', position: { row: 0, col: 4 } },
      { name: 'Cassiopeia', cost: 3, traits: ['Black Rose', 'Dominance'], role: 'carry', position: { row: 3, col: 4 } },
      { name: 'Blitzcrank', cost: 3, traits: ['Automata', 'Dominance'], role: 'tank', position: { row: 0, col: 3 } },
      { name: 'Smeech', cost: 3, traits: ['Chem-Baron', 'Ambusher'], role: 'flex', position: { row: 1, col: 5 } },
      { name: 'Malzahar', cost: 5, traits: ['Visionary', 'Automata'], role: 'carry', position: { row: 3, col: 2 } },
    ],
    mainCarry: {
      champion: 'Silco',
      items: ['Spear of Shojin', "Rabadon's Deathcap", 'Morellonomicon'],
    },
    mainTank: {
      champion: 'Blitzcrank',
      items: ['Gargoyle Stoneplate', 'Redemption', 'Sunfire Cape'],
    },
    earlyGame: ['Singed', 'Trundle', 'Zyra', 'Cassiopeia'],
    levelingGuide: 'Cash out Shimmer stacks at 300-400 for high-tier radiant chem-gear. Push Level 8 at 4-2 to finalize 2-star Silco.',
    positioningTip: 'Keep Silco center-back to hit both corners with his Shimmer poison vial explosion radius.',
  },
  {
    id: 'black_rose_conqueror',
    name: '5 Black Rose Dominance Cassiopeia',
    tier: 'S',
    difficulty: 'Easy',
    playstyle: 'Slow Roll Lvl 7',
    tagline: 'Summon Sion the Undead Juggernaut in front while Cassiopeia stacks infinite Twin Fang AP.',
    traits: [
      { name: 'Black Rose', count: 5, activeTier: 3 },
      { name: 'Dominance', count: 2, activeTier: 1 },
      { name: 'Visionary', count: 2, activeTier: 1 },
      { name: 'Sorcerer', count: 2, activeTier: 1 },
    ],
    champions: [
      { name: 'Cassiopeia', cost: 3, traits: ['Black Rose', 'Dominance'], role: 'carry', position: { row: 3, col: 0 } },
      { name: 'Elise', cost: 4, traits: ['Black Rose', 'Bruiser'], role: 'tank', position: { row: 0, col: 3 } },
      { name: 'Morgana', cost: 1, traits: ['Black Rose', 'Visionary'], role: 'support', position: { row: 3, col: 1 } },
      { name: 'Vladimir', cost: 2, traits: ['Black Rose', 'Sorcerer'], role: 'tank', position: { row: 0, col: 2 } },
      { name: 'Sion', cost: 4, traits: ['Black Rose Summon', 'Juggernaut'], role: 'tank', position: { row: 0, col: 4 } },
      { name: 'Malzahar', cost: 5, traits: ['Visionary', 'Automata'], role: 'carry', position: { row: 3, col: 6 } },
      { name: 'Nami', cost: 4, traits: ['Emissary', 'Sorcerer'], role: 'support', position: { row: 2, col: 0 } },
    ],
    mainCarry: {
      champion: 'Cassiopeia',
      items: ['Blue Buff', 'Jeweled Gauntlet', "Archangel's Staff"],
    },
    mainTank: {
      champion: 'Elise',
      items: ["Warmog's Armor", 'Bramble Vest', 'Sunfire Cape'],
    },
    earlyGame: ['Morgana', 'Vladimir', 'Zyra', 'Singed'],
    levelingGuide: 'Slow roll at Level 7 above 50 gold until Cassiopeia 3-Star is hit. Then push Level 8 for Malzahar.',
    positioningTip: 'Box Cassiopeia in the bottom-left corner with Morgana and Nami to shield against assassin jumps.',
  },
  {
    id: 'ambessa_conqueror',
    name: '4 Conqueror 4 Pit Fighter Ambessa',
    tier: 'S',
    difficulty: 'Medium',
    playstyle: 'Fast 8',
    tagline: 'Relentless frontline brawler comp with Ambessa executing high-priority targets with true damage leaps.',
    traits: [
      { name: 'Conqueror', count: 4, activeTier: 2 },
      { name: 'Pit Fighter', count: 4, activeTier: 2 },
      { name: 'Form Swapper', count: 2, activeTier: 1 },
    ],
    champions: [
      { name: 'Ambessa', cost: 4, traits: ['Conqueror', 'Emissary'], role: 'carry', position: { row: 1, col: 4 } },
      { name: 'Vi', cost: 4, traits: ['Enforcer', 'Pit Fighter'], role: 'carry', position: { row: 0, col: 2 } },
      { name: 'Darius', cost: 1, traits: ['Conqueror', 'Bruiser'], role: 'tank', position: { row: 0, col: 1 } },
      { name: 'Draven', cost: 2, traits: ['Conqueror', 'Pit Fighter'], role: 'carry', position: { row: 3, col: 6 } },
      { name: 'Sett', cost: 3, traits: ['Pit Fighter', 'Bruiser'], role: 'tank', position: { row: 0, col: 5 } },
      { name: 'Mordekaiser', cost: 5, traits: ['Conqueror', 'Dominance'], role: 'tank', position: { row: 0, col: 3 } },
      { name: 'Renni', cost: 3, traits: ['Chem-Baron', 'Bruiser'], role: 'tank', position: { row: 0, col: 4 } },
    ],
    mainCarry: {
      champion: 'Ambessa',
      items: ['Bloodthirster', "Titan's Resolve", "Sterak's Gage"],
    },
    mainTank: {
      champion: 'Mordekaiser',
      items: ["Warmog's Armor", 'Bramble Vest', "Dragon's Claw"],
    },
    earlyGame: ['Darius', 'Draven', 'Trundle', 'Violet'],
    levelingGuide: 'Streak through Stage 3 with strong 2-star Conquerors. Level to 8 at 4-2 and roll for Ambessa & Mordekaiser.',
    positioningTip: 'Place Ambessa on the 2nd row behind Darius so she does not draw initial aggro before dashing into squishies.',
  },
  {
    id: 'academy_innovators',
    name: '6 Academy 2 Sentinel Heimerdinger',
    tier: 'A',
    difficulty: 'Easy',
    playstyle: 'Fast 8',
    tagline: 'Sponsor high-stat Academy items to supercharge Jayce and Heimerdinger turret barrage.',
    traits: [
      { name: 'Academy', count: 6, activeTier: 3 },
      { name: 'Sentinel', count: 2, activeTier: 1 },
      { name: 'Innovator', count: 2, activeTier: 1 },
    ],
    champions: [
      { name: 'Heimerdinger', cost: 4, traits: ['Academy', 'Visionary'], role: 'carry', position: { row: 3, col: 3 } },
      { name: 'Jayce', cost: 5, traits: ['Academy', 'Form Swapper'], role: 'carry', position: { row: 0, col: 3 } },
      { name: 'Leona', cost: 2, traits: ['Academy', 'Sentinel'], role: 'tank', position: { row: 0, col: 2 } },
      { name: 'Ezreal', cost: 3, traits: ['Academy', 'Rebel'], role: 'carry', position: { row: 3, col: 5 } },
      { name: 'Loris', cost: 3, traits: ['Enforcer', 'Sentinel'], role: 'tank', position: { row: 0, col: 4 } },
      { name: 'Lux', cost: 1, traits: ['Academy', 'Sorcerer'], role: 'support', position: { row: 3, col: 1 } },
    ],
    mainCarry: {
      champion: 'Heimerdinger',
      items: ['Blue Buff', "Rabadon's Deathcap", 'Morellonomicon'],
    },
    mainTank: {
      champion: 'Jayce',
      items: ['Bloodthirster', "Titan's Resolve", "Warmog's Armor"],
    },
    earlyGame: ['Lux', 'Leona', 'Ezreal', 'Steb'],
    levelingGuide: 'Give the Academy sponsored item to Heimerdinger for 500+ extra shield and 40% AP.',
    positioningTip: 'Keep Heimerdinger in the center backrow so his rockets spray across the entire enemy board.',
  },
];

// 4. TFT Level Rolling Odds & Leveling Timetable
export const TFT_ROLL_ODDS: TftRollOdds[] = [
  { level: 1, cost1: 100, cost2: 0, cost3: 0, cost4: 0, cost5: 0, recommendedRound: 'Stage 1-1', xpToNext: 2, goldCostEstimate: 0 },
  { level: 2, cost1: 100, cost2: 0, cost3: 0, cost4: 0, cost5: 0, recommendedRound: 'Stage 1-3', xpToNext: 2, goldCostEstimate: 0 },
  { level: 3, cost1: 75, cost2: 25, cost3: 0, cost4: 0, cost5: 0, recommendedRound: 'Stage 1-4', xpToNext: 6, goldCostEstimate: 4 },
  { level: 4, cost1: 55, cost2: 30, cost3: 15, cost4: 0, cost5: 0, recommendedRound: 'Stage 2-1', xpToNext: 10, goldCostEstimate: 4 },
  { level: 5, cost1: 45, cost2: 33, cost3: 20, cost4: 2, cost5: 0, recommendedRound: 'Stage 2-5', xpToNext: 20, goldCostEstimate: 12 },
  { level: 6, cost1: 30, cost2: 40, cost3: 25, cost4: 5, cost5: 0, recommendedRound: 'Stage 3-2', xpToNext: 36, goldCostEstimate: 24 },
  { level: 7, cost1: 19, cost2: 35, cost3: 35, cost4: 10, cost5: 1, recommendedRound: 'Stage 4-1', xpToNext: 56, goldCostEstimate: 40 },
  { level: 8, cost1: 18, cost2: 25, cost3: 36, cost4: 18, cost5: 3, recommendedRound: 'Stage 4-2', xpToNext: 80, goldCostEstimate: 56 },
  { level: 9, cost1: 10, cost2: 20, cost3: 25, cost4: 35, cost5: 10, recommendedRound: 'Stage 5-2', xpToNext: 84, goldCostEstimate: 80 },
  { level: 10, cost1: 5, cost2: 10, cost3: 20, cost4: 40, cost5: 25, recommendedRound: 'Stage 6-1', xpToNext: 0, goldCostEstimate: 84 },
];

// 5. TFT Augments Tier List
export const TFT_AUGMENTS: TftAugment[] = [
  { id: 'enforcer_crest', name: 'Enforcer Crest', tier: 'Gold', winRate: 18.4, avgPlacement: 3.82, description: 'Gain an Enforcer Emblem and a Steb.', recommendedComps: ['rebel_snipers'] },
  { id: 'chem_baron_crown', name: 'Chem-Baron Crown', tier: 'Prismatic', winRate: 22.1, avgPlacement: 3.41, description: 'Gain a Chem-Baron Emblem, a Bloodthirster, and a Silco.', recommendedComps: ['chem_baron_silco'] },
  { id: 'tiny_titans', name: 'Tiny Titans', tier: 'Prismatic', winRate: 19.8, avgPlacement: 3.65, description: 'Gain 30 Tactician Health and heal 1 Health every round.', recommendedComps: ['rebel_snipers', 'chem_baron_silco'] },
  { id: 'pandoras_items', name: "Pandora's Items", tier: 'Silver', winRate: 14.2, avgPlacement: 4.12, description: 'At the start of each round, items on your bench transform randomly.', recommendedComps: ['rebel_snipers', 'black_rose_conqueror'] },
  { id: 'clear_mind', name: 'Clear Mind', tier: 'Silver', winRate: 15.6, avgPlacement: 4.05, description: 'If you have no champions on your bench at the end of a round, gain 3 XP.', recommendedComps: ['rebel_snipers', 'ambessa_conqueror'] },
  { id: 'cybernetic_implants', name: 'Cybernetic Implants', tier: 'Gold', winRate: 16.5, avgPlacement: 4.10, description: 'Your champions holding an item gain 120 Health and 12% Attack Damage.', recommendedComps: ['ambessa_conqueror', 'academy_innovators'] },
  { id: 'lotus_blossom', name: 'Jeweled Lotus II', tier: 'Gold', winRate: 17.2, avgPlacement: 3.95, description: 'Magic and True damage from your units can critically strike with 15% bonus Crit.', recommendedComps: ['black_rose_conqueror', 'chem_baron_silco'] },
  { id: 'hedge_fund', name: 'Hedge Fund', tier: 'Prismatic', winRate: 18.0, avgPlacement: 3.88, description: 'Gain 22 gold. Your maximum interest is increased to 10 gold.', recommendedComps: ['rebel_snipers', 'chem_baron_silco'] },
];
