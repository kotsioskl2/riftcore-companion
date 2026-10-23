export type GameflowPhase =
  | 'None'
  | 'Lobby'
  | 'Matchmaking'
  | 'ReadyCheck'
  | 'ChampSelect'
  | 'InProgress'
  | 'WaitingForStats'
  | 'PreEndOfGame'
  | 'EndOfGame';

export interface SummonerProfile {
  accountId: number;
  displayName?: string;
  gameName: string;
  tagLine: string;
  summonerId: number;
  summonerLevel: number;
  profileIconId: number;
  puuid: string;
  percentCompleteForNextLevel: number;
  rerollPoints?: {
    currentPoints: number;
    maxRolls: number;
    numberOfRolls: number;
  };
}

export interface QueueRankedStats {
  queueType: string;
  tier: string;
  division: string;
  leaguePoints: number;
  wins: number;
  losses: number;
  isProvisional: boolean;
  provisionalGamesRemaining?: number;
}

export interface RankedData {
  queues: QueueRankedStats[];
}

export interface RuneSelection {
  primaryStyleId: number;
  primaryStyleName: string;
  subStyleId: number;
  subStyleName: string;
  selectedPerkIds: number[];
  keystoneId: number;
  keystoneName: string;
  keystoneIcon: string;
}

export interface ChampionBuild {
  championId: number;
  championName: string;
  key: string;
  role: 'TOP' | 'JUNGLE' | 'MID' | 'ADC' | 'SUPPORT';
  tier: 'S+' | 'S' | 'A' | 'B';
  winRate: number;
  pickRate: number;
  banRate: number;
  runes: RuneSelection;
  secondaryRunes: RuneSelection;
  spells: [number, number]; // Spell IDs e.g. [4, 12] Flash + Teleport
  skillOrder: string[]; // ['Q', 'E', 'W', 'Q', 'Q', 'R', ...]
  skillPriority: string; // "Q > E > W"
  startingItems: { id: number; name: string }[];
  coreItems: { id: number; name: string }[];
  situationalItems: { id: number; name: string }[];
  boots: { id: number; name: string };
  counters: {
    strongAgainst: { name: string; winRate: number }[];
    weakAgainst: { name: string; winRate: number }[];
  };
  tips: string[];
}

export interface MatchHistoryItem {
  gameId: number;
  gameDuration: number;
  gameCreationDate: string;
  queueId: number;
  gameMode: string;
  championId: number;
  championName?: string;
  win: boolean;
  kills: number;
  deaths: number;
  assists: number;
  cs: number;
  csPerMin: number;
  goldEarned: number;
  damageDealt: number;
  visionScore: number;
  spell1Id: number;
  spell2Id: number;
  items: number[];
  badge?: 'MVP' | 'ACE' | 'CARRY' | 'UNLUCKY';
}

export interface ChampSelectSession {
  gameId?: number;
  myTeam: {
    cellId: number;
    championId: number;
    championPickIntent: number;
    assignedPosition: string;
    spell1Id: number;
    spell2Id: number;
    summonerId: number;
    puuid?: string;
  }[];
  theirTeam: {
    cellId: number;
    championId: number;
    assignedPosition: string;
  }[];
  bans: {
    myTeamBans: number[];
    theirTeamBans: number[];
    numBans: number;
  };
  timer: {
    adjustedTimeLeftInPhase: number;
    phase: string;
    totalTimeInPhase: number;
  };
  localPlayerCellId: number;
}

export interface LiveGamePlayer {
  summonerName: string;
  championName: string;
  team: 'ORDER' | 'CHAOS';
  level: number;
  scores: {
    kills: number;
    deaths: number;
    assists: number;
    creepScore: number;
    wardScore: number;
  };
  items: { itemID: number; count: number; displayName: string }[];
  summonerSpells: {
    summonerSpellOne: { displayName: string; rawDescription: string };
    summonerSpellTwo: { displayName: string; rawDescription: string };
  };
  currentGold?: number;
}

export interface LiveGameData {
  gameData: {
    gameTime: number;
    gameMode: string;
    mapName: string;
  };
  allPlayers: LiveGamePlayer[];
  activePlayer: {
    summonerName: string;
    championStats: {
      currentHealth: number;
      maxHealth: number;
      attackDamage: number;
      abilityPower: number;
      armor: number;
      magicResist: number;
      attackSpeed: number;
      moveSpeed: number;
    };
    currentGold: number;
  };
  events: {
    Events: {
      EventName: string;
      EventTime: number;
      KillerName?: string;
      VictimName?: string;
      DragonType?: string;
    }[];
  };
}

export interface AppSettings {
  autoAccept: boolean;
  autoImportRunes: boolean;
  autoImportSpells: boolean;
  soundAlerts: boolean;
  theme: string;
  alwaysOnTop: boolean;
}

declare global {
  interface Window {
    riftAPI?: {
      isElectron: boolean;
      onLCUStatusChange: (callback: (status: { connected: boolean; credentials?: any }) => void) => () => void;
      onGameflowPhase: (callback: (phase: GameflowPhase) => void) => () => void;
      onChampSelectSession: (callback: (session: ChampSelectSession) => void) => () => void;
      onReadyCheck: (callback: (data: any) => void) => () => void;
      onLiveGameUpdate: (callback: (data: LiveGameData) => void) => () => void;
      lcuRequest: (endpoint: string, method?: string, body?: any) => Promise<any>;
      importRunes: (pageData: any) => Promise<any>;
      importSpells: (spell1Id: number, spell2Id: number) => Promise<any>;
      acceptMatch: () => Promise<any>;
      getLiveClientData: () => Promise<LiveGameData | null>;
      windowControl: (action: 'minimize' | 'maximize' | 'close' | 'toggle-pin' | 'toggle-compact') => Promise<any>;
      getSettings: () => Promise<AppSettings>;
      saveSettings: (settings: Partial<AppSettings>) => Promise<AppSettings>;
      checkClientStatus: () => Promise<{ connected: boolean; gameflowPhase: GameflowPhase; riotClient?: boolean }>;
      launchLeague?: () => Promise<any>;
    };
  }
}
