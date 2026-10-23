import { ChampionBuild } from '../types';

export const CHAMPIONS_LIST: { id: number; name: string; key: string; title: string; roles: string[] }[] = [
  { id: 420, name: 'Illaoi', key: 'Illaoi', title: 'The Kraken Priestess', roles: ['TOP'] },
  { id: 266, name: 'Aatrox', key: 'Aatrox', title: 'The Darkin Blade', roles: ['TOP'] },
  { id: 103, name: 'Ahri', key: 'Ahri', title: 'The Nine-Tailed Fox', roles: ['MID'] },
  { id: 84, name: 'Akali', key: 'Akali', title: 'The Rogue Assassin', roles: ['MID', 'TOP'] },
  { id: 222, name: 'Jinx', key: 'Jinx', title: 'The Loose Cannon', roles: ['ADC'] },
  { id: 51, name: 'Caitlyn', key: 'Caitlyn', title: 'The Sheriff of Piltover', roles: ['ADC'] },
  { id: 64, name: 'Lee Sin', key: 'LeeSin', title: 'The Blind Monk', roles: ['JUNGLE'] },
  { id: 157, name: 'Yasuo', key: 'Yasuo', title: 'The Unforgiven', roles: ['MID', 'TOP'] },
  { id: 777, name: 'Yone', key: 'Yone', title: 'The Unforgotten', roles: ['MID', 'TOP'] },
  { id: 412, name: 'Thresh', key: 'Thresh', title: 'The Chain Warden', roles: ['SUPPORT'] },
  { id: 99, name: 'Lux', key: 'Lux', title: 'The Lady of Luminosity', roles: ['SUPPORT', 'MID'] },
  { id: 122, name: 'Darius', key: 'Darius', title: 'The Hand of Noxus', roles: ['TOP'] },
  { id: 67, name: 'Vayne', key: 'Vayne', title: 'The Night Hunter', roles: ['ADC', 'TOP'] },
  { id: 238, name: 'Zed', key: 'Zed', title: 'The Master of Shadows', roles: ['MID'] },
  { id: 81, name: 'Ezreal', key: 'Ezreal', title: 'The Prodigal Explorer', roles: ['ADC'] },
  { id: 145, name: "Kai'Sa", key: 'Kaisa', title: 'Daughter of the Void', roles: ['ADC'] },
  { id: 517, name: 'Sylas', key: 'Sylas', title: 'The Dethroned Mage', roles: ['MID', 'TOP'] },
  { id: 106, name: 'Volibear', key: 'Volibear', title: 'The Relentless Storm', roles: ['TOP', 'JUNGLE'] },
  { id: 24, name: 'Jax', key: 'Jax', title: 'Grandmaster at Arms', roles: ['TOP', 'JUNGLE'] },
  { id: 53, name: 'Blitzcrank', key: 'Blitzcrank', title: 'The Great Steam Golem', roles: ['SUPPORT'] },
  { id: 89, name: 'Leona', key: 'Leona', title: 'The Radiant Dawn', roles: ['SUPPORT'] },
  { id: 111, name: 'Nautilus', key: 'Nautilus', title: 'The Titan of the Depths', roles: ['SUPPORT'] },
  { id: 19, name: 'Warwick', key: 'Warwick', title: 'The Uncaged Wrath of Zaun', roles: ['JUNGLE', 'TOP'] },
  { id: 11, name: 'Master Yi', key: 'MasterYi', title: 'The Wuju Bladesman', roles: ['JUNGLE'] },
  { id: 121, name: "Kha'Zix", key: 'Khazix', title: 'The Voidreaver', roles: ['JUNGLE'] },
  { id: 254, name: 'Vi', key: 'Vi', title: 'The Piltover Enforcer', roles: ['JUNGLE'] },
  { id: 910, name: 'Hwei', key: 'Hwei', title: 'The Visionary', roles: ['MID'] },
  { id: 901, name: 'Smolder', key: 'Smolder', title: 'The Fiery Fledgling', roles: ['ADC', 'MID'] },
  { id: 799, name: 'Ambessa', key: 'Ambessa', title: 'Matriarch of War', roles: ['TOP', 'JUNGLE'] },
  { id: 897, name: "K'Sante", key: 'KSante', title: 'The Pride of Nazumah', roles: ['TOP'] }
];

export const CHAMPION_BUILDS: Record<number, ChampionBuild> = {
  // Illaoi (420)
  420: {
    championId: 420,
    championName: 'Illaoi',
    key: 'Illaoi',
    role: 'TOP',
    tier: 'S',
    winRate: 52.4,
    pickRate: 4.8,
    banRate: 6.2,
    runes: {
      primaryStyleId: 8400,
      primaryStyleName: 'Resolve',
      subStyleId: 8000,
      subStyleName: 'Precision',
      selectedPerkIds: [8437, 8446, 8473, 8451, 8009, 8299, 5008, 5008, 5001],
      keystoneId: 8437,
      keystoneName: 'Grasp of the Undying',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/GraspOfTheUndying/GraspOfTheUndying.png'
    },
    secondaryRunes: {
      primaryStyleId: 8000,
      primaryStyleName: 'Precision',
      subStyleId: 8400,
      subStyleName: 'Resolve',
      selectedPerkIds: [8010, 8009, 9105, 8299, 8446, 8473, 5008, 5008, 5001],
      keystoneId: 8010,
      keystoneName: 'Conqueror',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/Conqueror/Conqueror.png'
    },
    spells: [4, 12], // Flash, TP
    skillOrder: ['E', 'Q', 'W', 'E', 'E', 'R', 'E', 'Q', 'E', 'Q', 'R', 'Q', 'Q', 'W', 'W', 'R', 'W', 'W'],
    skillPriority: 'E > Q > W',
    startingItems: [
      { id: 1054, name: "Doran's Shield" },
      { id: 2003, name: 'Health Potion' }
    ],
    coreItems: [
      { id: 3071, name: 'Black Cleaver' },
      { id: 6631, name: 'Stridebreaker' },
      { id: 3053, name: "Sterak's Gage" }
    ],
    situationalItems: [
      { id: 3068, name: 'Sunfire Aegis' },
      { id: 4401, name: 'Force of Nature' },
      { id: 3075, name: 'Thornmail' }
    ],
    boots: { id: 3047, name: 'Plated Steelcaps' },
    counters: {
      strongAgainst: [
        { name: 'Sion', winRate: 56.8 },
        { name: 'Tahm Kench', winRate: 55.4 },
        { name: 'Nasus', winRate: 54.9 }
      ],
      weakAgainst: [
        { name: 'Kayle', winRate: 44.5 },
        { name: 'Vayne', winRate: 45.2 },
        { name: 'Yorick', winRate: 46.1 }
      ]
    },
    tips: [
      'Land Test of Spirit (E) before casting Leap of Faith (R) for an extra tentacle spawn.',
      'Fight near walls where your tentacles can slam multiple times.',
      'Never trade without at least 1-2 active tentacles nearby.'
    ]
  },

  // Aatrox (266)
  266: {
    championId: 266,
    championName: 'Aatrox',
    key: 'Aatrox',
    role: 'TOP',
    tier: 'S+',
    winRate: 51.2,
    pickRate: 11.4,
    banRate: 8.5,
    runes: {
      primaryStyleId: 8000,
      primaryStyleName: 'Precision',
      subStyleId: 8400,
      subStyleName: 'Resolve',
      selectedPerkIds: [8010, 9111, 9105, 8299, 8444, 8451, 5008, 5008, 5001],
      keystoneId: 8010,
      keystoneName: 'Conqueror',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/Conqueror/Conqueror.png'
    },
    secondaryRunes: {
      primaryStyleId: 8100,
      primaryStyleName: 'Domination',
      subStyleId: 8000,
      subStyleName: 'Precision',
      selectedPerkIds: [8128, 8143, 8138, 8135, 9111, 8299, 5008, 5008, 5001],
      keystoneId: 8128,
      keystoneName: 'Dark Harvest',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/DarkHarvest/DarkHarvest.png'
    },
    spells: [4, 12],
    skillOrder: ['Q', 'E', 'W', 'Q', 'Q', 'R', 'Q', 'E', 'Q', 'E', 'R', 'E', 'E', 'W', 'W', 'R', 'W', 'W'],
    skillPriority: 'Q > E > W',
    startingItems: [
      { id: 1055, name: "Doran's Blade" },
      { id: 2003, name: 'Health Potion' }
    ],
    coreItems: [
      { id: 6692, name: 'Eclipse' },
      { id: 6694, name: 'Sundered Sky' },
      { id: 3071, name: 'Black Cleaver' }
    ],
    situationalItems: [
      { id: 3053, name: "Sterak's Gage" },
      { id: 6695, name: "Serpents Fang" },
      { id: 3026, name: 'Guardian Angel' }
    ],
    boots: { id: 3047, name: 'Plated Steelcaps' },
    counters: {
      strongAgainst: [
        { name: 'Sion', winRate: 54.3 },
        { name: 'Gangplank', winRate: 53.8 },
        { name: 'Kennen', winRate: 53.2 }
      ],
      weakAgainst: [
        { name: 'Irelia', winRate: 46.2 },
        { name: 'Fiora', winRate: 46.7 },
        { name: 'Kled', winRate: 47.1 }
      ]
    },
    tips: [
      'Use E (Umbral Dash) during Q animation to adjust the sweet spot landing position.',
      'World Ender (R) resets duration on champion takedowns and amplifies all self-healing.'
    ]
  },

  // Jinx (222)
  222: {
    championId: 222,
    championName: 'Jinx',
    key: 'Jinx',
    role: 'ADC',
    tier: 'S+',
    winRate: 52.8,
    pickRate: 19.3,
    banRate: 7.1,
    runes: {
      primaryStyleId: 8000,
      primaryStyleName: 'Precision',
      subStyleId: 8200,
      subStyleName: 'Sorcery',
      selectedPerkIds: [8004, 9101, 9104, 8014, 8210, 8236, 5005, 5008, 5001],
      keystoneId: 8004,
      keystoneName: 'Lethal Tempo',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/LethalTempo/LethalTempoTemp.png'
    },
    secondaryRunes: {
      primaryStyleId: 8000,
      primaryStyleName: 'Precision',
      subStyleId: 8300,
      subStyleName: 'Inspiration',
      selectedPerkIds: [8008, 9101, 9104, 8017, 8304, 8345, 5005, 5008, 5001],
      keystoneId: 8008,
      keystoneName: 'Fleet Footwork',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/FleetFootwork/FleetFootwork.png'
    },
    spells: [4, 7], // Flash, Heal
    skillOrder: ['Q', 'W', 'E', 'Q', 'Q', 'R', 'Q', 'W', 'Q', 'W', 'R', 'W', 'W', 'E', 'E', 'R', 'E', 'E'],
    skillPriority: 'Q > W > E',
    startingItems: [
      { id: 1055, name: "Doran's Blade" },
      { id: 2003, name: 'Health Potion' }
    ],
    coreItems: [
      { id: 3031, name: 'Infinity Edge' },
      { id: 3094, name: 'Rapid Firecannon' },
      { id: 3085, name: "Runaan's Hurricane" }
    ],
    situationalItems: [
      { id: 3036, name: "Lord Dominik's Regards" },
      { id: 3072, name: 'Bloodthirster' },
      { id: 3026, name: 'Guardian Angel' }
    ],
    boots: { id: 3006, name: "Berserker's Greaves" },
    counters: {
      strongAgainst: [
        { name: 'Aphelios', winRate: 54.1 },
        { name: 'Zeri', winRate: 53.6 },
        { name: 'KaiSa', winRate: 52.9 }
      ],
      weakAgainst: [
        { name: 'Twitch', winRate: 46.4 },
        { name: 'Draven', winRate: 47.0 },
        { name: 'Samira', winRate: 47.8 }
      ]
    },
    tips: [
      'Switch to Fishbones (Rockets) for long range poke and teamfight AoE, use Pow-Pow (Minigun) for towers and single-target DPS.',
      'Place Flame Chompers (E) behind enemy champions or in choke points to self-peel.'
    ]
  },

  // Ahri (103)
  103: {
    championId: 103,
    championName: 'Ahri',
    key: 'Ahri',
    role: 'MID',
    tier: 'S',
    winRate: 51.5,
    pickRate: 10.2,
    banRate: 4.5,
    runes: {
      primaryStyleId: 8100,
      primaryStyleName: 'Domination',
      subStyleId: 8200,
      subStyleName: 'Sorcery',
      selectedPerkIds: [8112, 8139, 8138, 8106, 8226, 8210, 5008, 5008, 5001],
      keystoneId: 8112,
      keystoneName: 'Electrocute',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/Electrocute/Electrocute.png'
    },
    secondaryRunes: {
      primaryStyleId: 8300,
      primaryStyleName: 'Inspiration',
      subStyleId: 8100,
      subStyleName: 'Domination',
      selectedPerkIds: [8351, 8304, 8345, 8347, 8139, 8106, 5008, 5008, 5001],
      keystoneId: 8351,
      keystoneName: 'Glacial Augment',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Inspiration/GlacialAugment/GlacialAugment.png'
    },
    spells: [4, 12],
    skillOrder: ['Q', 'W', 'E', 'Q', 'Q', 'R', 'Q', 'W', 'Q', 'W', 'R', 'W', 'W', 'E', 'E', 'R', 'E', 'E'],
    skillPriority: 'Q > W > E',
    startingItems: [
      { id: 1056, name: "Doran's Ring" },
      { id: 2003, name: 'Health Potion' }
    ],
    coreItems: [
      { id: 6653, name: "Liandry's Torment" },
      { id: 4645, name: 'Shadowflame' },
      { id: 3089, name: "Rabadon's Deathcap" }
    ],
    situationalItems: [
      { id: 3157, name: "Zhonya's Hourglass" },
      { id: 3135, name: 'Void Staff' },
      { id: 3102, name: "Banshee's Veil" }
    ],
    boots: { id: 3020, name: "Sorcerer's Shoes" },
    counters: {
      strongAgainst: [
        { name: 'Kassadin', winRate: 53.7 },
        { name: 'Azir', winRate: 53.1 },
        { name: 'Orianna', winRate: 52.4 }
      ],
      weakAgainst: [
        { name: 'Tristana', winRate: 46.8 },
        { name: 'Akshan', winRate: 47.3 },
        { name: 'Naafiri', winRate: 47.9 }
      ]
    },
    tips: [
      'Charm (E) cancels dashes and amplifies follow-up magic damage burst.',
      'Spirit Rush (R) grants up to 3 dashes and gains an additional recast on takedowns.'
    ]
  },

  // Lee Sin (64)
  64: {
    championId: 64,
    championName: 'Lee Sin',
    key: 'LeeSin',
    role: 'JUNGLE',
    tier: 'S',
    winRate: 49.8,
    pickRate: 15.6,
    banRate: 9.3,
    runes: {
      primaryStyleId: 8000,
      primaryStyleName: 'Precision',
      subStyleId: 8300,
      subStyleName: 'Inspiration',
      selectedPerkIds: [8010, 9111, 9104, 8014, 8304, 8347, 5008, 5008, 5001],
      keystoneId: 8010,
      keystoneName: 'Conqueror',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/Conqueror/Conqueror.png'
    },
    secondaryRunes: {
      primaryStyleId: 8100,
      primaryStyleName: 'Domination',
      subStyleId: 8000,
      subStyleName: 'Precision',
      selectedPerkIds: [8112, 8143, 8138, 8105, 9111, 8014, 5008, 5008, 5001],
      keystoneId: 8112,
      keystoneName: 'Electrocute',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/Electrocute/Electrocute.png'
    },
    spells: [4, 11], // Flash, Smite
    skillOrder: ['Q', 'W', 'E', 'Q', 'Q', 'R', 'Q', 'W', 'Q', 'W', 'R', 'W', 'W', 'E', 'E', 'R', 'E', 'E'],
    skillPriority: 'Q > W > E',
    startingItems: [
      { id: 1102, name: 'Gustwalker Hatchling' },
      { id: 2003, name: 'Health Potion' }
    ],
    coreItems: [
      { id: 6692, name: 'Eclipse' },
      { id: 6694, name: 'Sundered Sky' },
      { id: 3071, name: 'Black Cleaver' }
    ],
    situationalItems: [
      { id: 3053, name: "Sterak's Gage" },
      { id: 3026, name: 'Guardian Angel' },
      { id: 6333, name: "Death's Dance" }
    ],
    boots: { id: 3047, name: 'Plated Steelcaps' },
    counters: {
      strongAgainst: [
        { name: 'Nidalee', winRate: 53.4 },
        { name: 'Karthus', winRate: 52.9 },
        { name: 'Graves', winRate: 52.0 }
      ],
      weakAgainst: [
        { name: 'Warwick', winRate: 46.1 },
        { name: 'Poppy', winRate: 46.8 },
        { name: 'Udyr', winRate: 47.3 }
      ]
    },
    tips: [
      'Insec Kick combo: Q1 -> Q2 -> Ward Hop (W) behind target -> Dragon Rage (R) towards your team.',
      'Always weave basic attacks between spell casts to utilize Flurry energy restoration.'
    ]
  },

  // Thresh (412)
  412: {
    championId: 412,
    championName: 'Thresh',
    key: 'Thresh',
    role: 'SUPPORT',
    tier: 'S+',
    winRate: 51.6,
    pickRate: 14.2,
    banRate: 5.7,
    runes: {
      primaryStyleId: 8300,
      primaryStyleName: 'Inspiration',
      subStyleId: 8400,
      subStyleName: 'Resolve',
      selectedPerkIds: [8351, 8306, 8345, 8347, 8473, 8453, 5007, 5008, 5001],
      keystoneId: 8351,
      keystoneName: 'Glacial Augment',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Inspiration/GlacialAugment/GlacialAugment.png'
    },
    secondaryRunes: {
      primaryStyleId: 8400,
      primaryStyleName: 'Resolve',
      subStyleId: 8300,
      subStyleName: 'Inspiration',
      selectedPerkIds: [8439, 8463, 8473, 8453, 8345, 8347, 5007, 5008, 5001],
      keystoneId: 8439,
      keystoneName: 'Aftershock',
      keystoneIcon: 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Resolve/VeteranAftershock/VeteranAftershock.png'
    },
    spells: [4, 14], // Flash, Ignite
    skillOrder: ['Q', 'E', 'W', 'Q', 'Q', 'R', 'Q', 'W', 'Q', 'W', 'R', 'W', 'W', 'E', 'E', 'R', 'E', 'E'],
    skillPriority: 'Q > W > E',
    startingItems: [
      { id: 3865, name: 'World Atlas' },
      { id: 2003, name: 'Health Potion' }
    ],
    coreItems: [
      { id: 3190, name: 'Locket of the Iron Solari' },
      { id: 3050, name: "Zeke's Convergence" },
      { id: 3109, name: "Knight's Vow" }
    ],
    situationalItems: [
      { id: 3107, name: 'Redemption' },
      { id: 3001, name: 'Abyssal Mask' },
      { id: 3075, name: 'Thornmail' }
    ],
    boots: { id: 3117, name: 'Mobility Boots' },
    counters: {
      strongAgainst: [
        { name: 'Yuumi', winRate: 54.8 },
        { name: 'Leona', winRate: 53.2 },
        { name: 'Nautilus', winRate: 52.5 }
      ],
      weakAgainst: [
        { name: 'Morgana', winRate: 46.3 },
        { name: 'Zyra', winRate: 47.1 },
        { name: 'Brand', winRate: 47.6 }
      ]
    },
    tips: [
      'Flay (E) before Death Sentence (Q) for a guaranteed hook against mobile targets.',
      'Lantern (W) collects souls automatically from distance and can save overextended allies.'
    ]
  }
};

// Fallback build generator for any champion in the game
export function getChampionBuild(championId: number, championName?: string): ChampionBuild {
  if (CHAMPION_BUILDS[championId]) {
    return CHAMPION_BUILDS[championId];
  }

  // Find info from list or use defaults
  const champ = CHAMPIONS_LIST.find(c => c.id === championId) || {
    id: championId,
    name: championName || 'Champion',
    key: championName || 'Champion',
    roles: ['MID']
  };

  const role = (champ.roles[0] || 'MID') as ChampionBuild['role'];

  // Smart defaults based on role
  let primaryStyleId = 8000;
  let primaryStyleName = 'Precision';
  let subStyleId = 8400;
  let subStyleName = 'Resolve';
  let selectedPerkIds = [8010, 9111, 9104, 8299, 8473, 8451, 5008, 5008, 5001];
  let keystoneId = 8010;
  let keystoneName = 'Conqueror';
  let keystoneIcon = 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/Conqueror/Conqueror.png';
  let spells: [number, number] = [4, 12];
  let boots = { id: 3047, name: 'Plated Steelcaps' };

  if (role === 'ADC') {
    primaryStyleId = 8000;
    primaryStyleName = 'Precision';
    subStyleId = 8200;
    subStyleName = 'Sorcery';
    selectedPerkIds = [8004, 9101, 9104, 8014, 8210, 8236, 5005, 5008, 5001];
    keystoneId = 8004;
    keystoneName = 'Lethal Tempo';
    keystoneIcon = 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Precision/LethalTempo/LethalTempoTemp.png';
    spells = [4, 7]; // Flash, Heal
    boots = { id: 3006, name: "Berserker's Greaves" };
  } else if (role === 'SUPPORT') {
    primaryStyleId = 8300;
    primaryStyleName = 'Inspiration';
    subStyleId = 8400;
    subStyleName = 'Resolve';
    selectedPerkIds = [8351, 8304, 8345, 8347, 8473, 8453, 5007, 5008, 5001];
    keystoneId = 8351;
    keystoneName = 'Glacial Augment';
    keystoneIcon = 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Inspiration/GlacialAugment/GlacialAugment.png';
    spells = [4, 14]; // Flash, Ignite
    boots = { id: 3117, name: 'Mobility Boots' };
  } else if (role === 'JUNGLE') {
    spells = [4, 11]; // Flash, Smite
  } else if (role === 'MID') {
    primaryStyleId = 8100;
    primaryStyleName = 'Domination';
    subStyleId = 8200;
    subStyleName = 'Sorcery';
    selectedPerkIds = [8112, 8139, 8138, 8106, 8226, 8210, 5008, 5008, 5001];
    keystoneId = 8112;
    keystoneName = 'Electrocute';
    keystoneIcon = 'https://ddragon.leagueoflegends.com/cdn/img/perk-images/Styles/Domination/Electrocute/Electrocute.png';
    boots = { id: 3020, name: "Sorcerer's Shoes" };
  }

  return {
    championId: champ.id,
    championName: champ.name,
    key: champ.key,
    role,
    tier: 'S',
    winRate: 51.4,
    pickRate: 6.8,
    banRate: 4.2,
    runes: {
      primaryStyleId,
      primaryStyleName,
      subStyleId,
      subStyleName,
      selectedPerkIds,
      keystoneId,
      keystoneName,
      keystoneIcon
    },
    secondaryRunes: {
      primaryStyleId,
      primaryStyleName,
      subStyleId,
      subStyleName,
      selectedPerkIds,
      keystoneId,
      keystoneName,
      keystoneIcon
    },
    spells,
    skillOrder: ['Q', 'E', 'W', 'Q', 'Q', 'R', 'Q', 'E', 'Q', 'E', 'R', 'E', 'E', 'W', 'W', 'R', 'W', 'W'],
    skillPriority: 'Q > E > W',
    startingItems: [
      { id: 1055, name: "Doran's Blade" },
      { id: 2003, name: 'Health Potion' }
    ],
    coreItems: [
      { id: 6692, name: 'Eclipse' },
      { id: 6694, name: 'Sundered Sky' },
      { id: 3071, name: 'Black Cleaver' }
    ],
    situationalItems: [
      { id: 3053, name: "Sterak's Gage" },
      { id: 3026, name: 'Guardian Angel' },
      { id: 3157, name: "Zhonya's Hourglass" }
    ],
    boots,
    counters: {
      strongAgainst: [
        { name: 'Sion', winRate: 54.2 },
        { name: 'Kennen', winRate: 53.6 }
      ],
      weakAgainst: [
        { name: 'Fiora', winRate: 46.5 },
        { name: 'Irelia', winRate: 47.1 }
      ]
    },
    tips: [
      'Focus on your core item spikes to dominate teamfights.',
      'Track enemy cooldowns before initiating trades.'
    ]
  };
}

export function getChampionIcon(championIdOrKey: number | string): string {
  if (typeof championIdOrKey === 'number') {
    const found = CHAMPIONS_LIST.find(c => c.id === championIdOrKey);
    if (found) {
      return `https://ddragon.leagueoflegends.com/cdn/14.23.1/img/champion/${found.key}.png`;
    }
    return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/${championIdOrKey}.png`;
  }
  return `https://ddragon.leagueoflegends.com/cdn/14.23.1/img/champion/${championIdOrKey}.png`;
}

export function getChampionSplash(championKey: string): string {
  return `https://ddragon.leagueoflegends.com/cdn/img/champion/splash/${championKey}_0.jpg`;
}
