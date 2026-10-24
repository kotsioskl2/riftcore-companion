// RiftCore ARAM Mayhem & Arena Augment Cards Catalog
// Comprehensive tier rankings, synergies, and instant tactical advice for drafting rounds

export type CardTier = 'Prismatic' | 'Gold' | 'Silver';
export type CardRank = 'S+' | 'S' | 'A';
export type CardMode = 'ARAM' | 'Arena' | 'Both';

export interface MayhemCard {
  id: string;
  name: string;
  tier: CardTier;
  rank: CardRank;
  mode: CardMode;
  archetype: string;
  description: string;
  tip: string;
  color: string;
  tag: 'Crit' | 'Haste' | 'Survivability' | 'Damage' | 'Range' | 'On-Hit' | 'Economy';
}

export const MAYHEM_CARDS: MayhemCard[] = [
  // --- PRISMATIC CARDS (Game-Breaking Meta Augments) ---
  {
    id: 'jeweled-gauntlet',
    name: 'Jeweled Gauntlet',
    tier: 'Prismatic',
    rank: 'S+',
    mode: 'Both',
    archetype: 'Mages & AP Assassins (Ahri, Veigar, Syndra, Hwei, Lux)',
    description: 'Your abilities can critically strike with 140% damage. Gain bonus Critical Strike Chance scaling with Ability Power.',
    tip: 'Must-pick for burst mages. Converts full AP builds into lethal 1-shot crit spells.',
    color: '#c084fc',
    tag: 'Crit',
  },
  {
    id: 'blade-waltz',
    name: 'Blade Waltz',
    tier: 'Prismatic',
    rank: 'S+',
    mode: 'Both',
    archetype: 'Duelists & Skirmishers (Master Yi, Fiora, Yasuo, Bel\'Veth)',
    description: 'Active: Become untargetable while dashing 8 times around target enemies, dealing massive physical and on-hit damage.',
    tip: 'Total invulnerability while shredding multiple enemies. In ARAM choke points, this wins teamfights.',
    color: '#c084fc',
    tag: 'Damage',
  },
  {
    id: 'goliath',
    name: 'Goliath',
    tier: 'Prismatic',
    rank: 'S+',
    mode: 'Both',
    archetype: 'Tanks & Juggernauts (Cho\'Gath, Sion, Tahm Kench, Dr. Mundo)',
    description: 'Gain 35% increased champion size, +800-1000 Maximum Health, and +15% Adaptive Force.',
    tip: 'Massively increases Heartsteel procs and Sunfire radius. Makes frontliners unkillable.',
    color: '#c084fc',
    tag: 'Survivability',
  },
  {
    id: 'symphony-of-war',
    name: 'Symphony of War',
    tier: 'Prismatic',
    rank: 'S+',
    mode: 'Both',
    archetype: 'Hyper-Carries & Marksmen (Jinx, Vayne, Kayle, Twitch)',
    description: 'Gain both Lethal Tempo AND Conqueror keystones simultaneously with no attack speed cap.',
    tip: 'Uncapped attack speed plus true healing. In ARAM teamfights, melts frontlines in seconds.',
    color: '#c084fc',
    tag: 'On-Hit',
  },
  {
    id: 'chain-lightning',
    name: 'Chain Lightning',
    tier: 'Prismatic',
    rank: 'S+',
    mode: 'ARAM',
    archetype: 'Marksmen & Poke Mages (Caitlyn, Ezreal, Ziggs, Varus)',
    description: 'Damaging an enemy champion arcs chain lightning to up to 4 nearby targets, applying spell effects and physical damage.',
    tip: 'Devastating in Howling Abyss single-lane corridors where enemies clump together.',
    color: '#c084fc',
    tag: 'Damage',
  },
  {
    id: 'tap-dancer',
    name: 'Tap Dancer',
    tier: 'Prismatic',
    rank: 'S+',
    mode: 'Both',
    archetype: 'Kiting Marksmen & Bruisers (Vayne, Zeri, Jhin, Lillia)',
    description: 'Basic attacks grant 10 stacking Movement Speed (no cap). Gain Attack Speed equal to 10% of your Movement Speed.',
    tip: 'Unmatched kiting potential. Run circles around skillshot champions while auto-attacking.',
    color: '#c084fc',
    tag: 'Haste',
  },
  {
    id: 'earthwake',
    name: 'Earthwake',
    tier: 'Prismatic',
    rank: 'S+',
    mode: 'Both',
    archetype: 'Mobile Assassins & Divers (Zed, Akali, Lee Sin, Riven, Pyke)',
    description: 'Your dashes, leaps, and blinks leave a seismic path that detonates after 1s dealing massive physical damage.',
    tip: 'Every Snowball dash or gap-closer triggers a lethal explosion under targets.',
    color: '#c084fc',
    tag: 'Damage',
  },
  {
    id: 'eureka',
    name: 'Eureka',
    tier: 'Prismatic',
    rank: 'S+',
    mode: 'Both',
    archetype: 'AP Spellcasters (Karthus, Anivia, Aurelion Sol, Xerath)',
    description: 'Gain Ability Haste equal to 25% of your Ability Power. Spells refund mana on champion hit.',
    tip: 'Turns high AP builds into instant machine-gun spell rotations with 150+ Ability Haste.',
    color: '#c084fc',
    tag: 'Haste',
  },
  {
    id: 'wisdom-of-ages',
    name: 'Wisdom of Ages',
    tier: 'Prismatic',
    rank: 'S',
    mode: 'Both',
    archetype: 'Scaling Champions (Kayle, Kassadin, Vladimir, Smolder)',
    description: 'Gain +1 Champion Level immediately and gain +1 bonus level every 2 combat rounds / 90 seconds.',
    tip: 'Fast-tracks Kayle level 16 ascendancy and Kassadin level 16 riftwalk cooldown.',
    color: '#c084fc',
    tag: 'Economy',
  },
  {
    id: 'center-of-the-universe',
    name: 'Center of the Universe',
    tier: 'Prismatic',
    rank: 'S',
    mode: 'Both',
    archetype: 'Battle Mages & Tanks (Swain, Singed, Amumu, Udyr)',
    description: 'Three celestial stars constantly orbit your champion, expanding with attack range and dealing continuous magic damage.',
    tip: 'Provides free DPS while walking in teamfights; triggers Rylai\'s slow and Liandry\'s burn automatically.',
    color: '#c084fc',
    tag: 'Damage',
  },

  // --- GOLD CARDS (High-Impact Tactical Buffs) ---
  {
    id: 'apex-inventor',
    name: 'Apex Inventor',
    tier: 'Gold',
    rank: 'S+',
    mode: 'Both',
    archetype: 'Item-Proc Users (Aatrox, Volibear, Jax, Leona, Shen)',
    description: 'Gain +75 Item Haste (scaling to 300 Item Haste). Reduces Heartsteel, Sundered Sky, Zhonya\'s, and Eclipse cooldowns to 2.5s.',
    tip: 'Heartsteel triggers multiple times per fight. Sundered Sky crits on every second hit.',
    color: '#f59e0b',
    tag: 'Haste',
  },
  {
    id: 'courage-of-the-colossus',
    name: 'Courage of the Colossus',
    tier: 'Gold',
    rank: 'S+',
    mode: 'Both',
    archetype: 'Engage Tanks & Hard CC (Alistar, Leona, Nautilus, Thresh)',
    description: 'Immobilizing an enemy champion grants a massive decaying shield scaling with 16% Maximum HP.',
    tip: 'Virtually permanent shields when cycling crowd control abilities in teamfights.',
    color: '#f59e0b',
    tag: 'Survivability',
  },
  {
    id: 'vulnerability',
    name: 'Vulnerability',
    tier: 'Gold',
    rank: 'S+',
    mode: 'Both',
    archetype: 'DoT Mages & Burn Champions (Brand, Teemo, Malzahar, Cassiopeia)',
    description: 'Your item damage effects (Liandry\'s, Sunfire) and damage-over-time bleeds/burns can critically strike.',
    tip: 'Brand passive and Liandry\'s melt 40%+ max HP when burning with critical strikes.',
    color: '#f59e0b',
    tag: 'Crit',
  },
  {
    id: 'phenomenal-evil',
    name: 'Phenomenal Evil',
    tier: 'Gold',
    rank: 'S+',
    mode: 'Both',
    archetype: 'Poke & Artillery Mages (Ziggs, Xerath, Lux, Vel\'Koz, Hwei)',
    description: 'Damaging enemy champions with abilities grants +1 Ability Power permanently (1s cooldown per champion).',
    tip: 'In ARAM where enemies are in constant range, easily yields +200-350 free AP over a match.',
    color: '#f59e0b',
    tag: 'Damage',
  },
  {
    id: 'scoped-weapons',
    name: 'Scoped Weapons',
    tier: 'Gold',
    rank: 'S+',
    mode: 'Both',
    archetype: 'Marksmen & Melee Bruisers (Darius, Jax, Caitlyn, Kog\'Maw)',
    description: 'Gain +150 Attack Range if Melee, or +75 Attack Range if Ranged.',
    tip: 'Lets melee bruisers attack from outside turret range; marksmen outrange backlines easily.',
    color: '#f59e0b',
    tag: 'Range',
  },
  {
    id: 'giant-slayer',
    name: 'Giant Slayer',
    tier: 'Gold',
    rank: 'S',
    mode: 'Both',
    archetype: 'ADCs & Assassins vs High-HP Teams',
    description: 'Deal up to 30% increased damage to enemy champions with higher maximum health than you.',
    tip: 'The ultimate counter card against Heartsteel tanks and Colossi in ARAM Mayhem.',
    color: '#f59e0b',
    tag: 'Damage',
  },
  {
    id: 'outlaws-grit',
    name: 'Outlaw\'s Grit',
    tier: 'Gold',
    rank: 'S',
    mode: 'Both',
    archetype: 'Skirmishers & Divers (Riven, Yasuo, Irelia, Bel\'Veth)',
    description: 'Dashing or blinking grants 12-25 Armor and Magic Resist for 4 seconds, stacking up to 5 times.',
    tip: 'Grants over 100 free dual resistances during high-mobility teamfight skirmishes.',
    color: '#f59e0b',
    tag: 'Survivability',
  },
  {
    id: 'with-haste',
    name: 'With Haste',
    tier: 'Gold',
    rank: 'S',
    mode: 'Both',
    archetype: 'Speed & Roaming Champions (Hecarim, Singed, Rammus, Warwick)',
    description: 'Gain Ability Haste equal to 15% of your bonus Movement Speed. Dashing grants a burst of speed.',
    tip: 'Synergizes with Ghost, Snowball, and boots to provide hyper-fast ability uptime.',
    color: '#f59e0b',
    tag: 'Haste',
  },
  {
    id: 'recursion',
    name: 'Recursion',
    tier: 'Gold',
    rank: 'S',
    mode: 'Both',
    archetype: 'Ultimate-Reliant Champions (Malphite, Amumu, Kennen, Orianna)',
    description: 'Casting your Ultimate reduces all basic ability cooldowns by 60% and refunds 25% of your Ultimate cooldown.',
    tip: 'Ensures your game-winning ultimate is available every wave in ARAM Mayhem.',
    color: '#f59e0b',
    tag: 'Haste',
  },

  // --- SILVER CARDS (Utility & Core Engine Enhancements) ---
  {
    id: 'bread-and-butter',
    name: 'Bread and Butter',
    tier: 'Silver',
    rank: 'S',
    mode: 'Both',
    archetype: 'Q-Centric Champions (Blitzcrank, Ezreal, Karthus, Nidalee, Morgana)',
    description: 'Gain +200 Ability Haste on your Q ability.',
    tip: 'Turns Blitzcrank hook, Ezreal Mystic Shot, and Morgana root into sub-2-second spams.',
    color: '#94a3b8',
    tag: 'Haste',
  },
  {
    id: 'bread-and-jam',
    name: 'Bread and Jam',
    tier: 'Silver',
    rank: 'S',
    mode: 'Both',
    archetype: 'W-Centric Champions (Sett, Kog\'Maw, Vladimir, Ahri)',
    description: 'Gain +200 Ability Haste on your W ability.',
    tip: 'Sett Haymaker true damage punch or Vladimir pool available virtually every trade.',
    color: '#94a3b8',
    tag: 'Haste',
  },
  {
    id: 'bread-and-cheese',
    name: 'Bread and Cheese',
    tier: 'Silver',
    rank: 'S',
    mode: 'Both',
    archetype: 'E-Centric Champions (Jax, Fizz, Kayle, Camille)',
    description: 'Gain +200 Ability Haste on your E ability.',
    tip: 'Jax Counter Strike or Fizz Playful/Trickster becomes permanently spammable.',
    color: '#94a3b8',
    tag: 'Haste',
  },
  {
    id: 'erosion',
    name: 'Erosion',
    tier: 'Silver',
    rank: 'S',
    mode: 'Both',
    archetype: 'Fast Attackers & Multi-Hitters (Katarina, Urgot, Ashe, Lucian)',
    description: 'Damaging an enemy shreds 1.5% of their Armor and Magic Resist for 4s, stacking up to 30%.',
    tip: 'Team-wide shred benefit. Great for stripping tanks without needing early Last Whisper.',
    color: '#94a3b8',
    tag: 'Damage',
  },
  {
    id: 'chauffeur',
    name: 'Chauffeur',
    tier: 'Silver',
    rank: 'S',
    mode: 'Both',
    archetype: 'Enchanters & Artillery (Yuumi, Kog\'Maw, Sona, Janna)',
    description: 'You are tethered to your ally and automatically ride with them. Gain massive bonus resistances and move speed.',
    tip: 'Ride a frontline tank (Sion/Malphite) while continuously shielding and shooting from safety.',
    color: '#94a3b8',
    tag: 'Survivability',
  },
  {
    id: 'mind-to-matter',
    name: 'Mind to Matter',
    tier: 'Silver',
    rank: 'S',
    mode: 'Both',
    archetype: 'Mana Stackers (Ryze, Kassadin, Taric, Blitzcrank, Anivia)',
    description: 'Gain Maximum Health equal to 50% of your Maximum Mana.',
    tip: 'Combine with Archangel\'s Staff / Fimbulwinter / Frozen Heart for 1,500+ free bonus HP.',
    color: '#94a3b8',
    tag: 'Survivability',
  },
  {
    id: 'contract-killer',
    name: 'Contract Killer',
    tier: 'Silver',
    rank: 'A',
    mode: 'Both',
    archetype: 'Early Snowball & Pick Champions (Pyke, Talon, LeBlanc)',
    description: 'Each round or fight, a high-value enemy is marked. Taking them down grants 350 bonus team gold.',
    tip: 'Accelerates item spikes and lets you hit core mythic items minutes ahead of opponents.',
    color: '#94a3b8',
    tag: 'Economy',
  },
];

export function searchMayhemCards(
  query: string,
  tierFilter?: CardTier | 'ALL',
  modeFilter?: CardMode | 'ALL'
): MayhemCard[] {
  const q = query.trim().toLowerCase();
  return MAYHEM_CARDS.filter((card) => {
    if (tierFilter && tierFilter !== 'ALL' && card.tier !== tierFilter) return false;
    if (modeFilter && modeFilter !== 'ALL' && card.mode !== 'Both' && card.mode !== modeFilter) return false;
    if (!q) return true;
    return (
      card.name.toLowerCase().includes(q) ||
      card.archetype.toLowerCase().includes(q) ||
      card.description.toLowerCase().includes(q) ||
      card.tag.toLowerCase().includes(q) ||
      card.tip.toLowerCase().includes(q)
    );
  });
}
