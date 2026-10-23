// Riot Games Official Web API Service for RiftCore Companion
// Supports Regional clusters (americas, europe, asia, sea) and Platform routing (eun1, euw1, na1, kr, etc.)

export interface RiotAccount {
  puuid: string;
  gameName: string;
  tagLine: string;
}

export interface RiotSummoner {
  id: string;
  accountId: string;
  puuid: string;
  profileIconId: number;
  revisionDate: number;
  summonerLevel: number;
}

export interface RiotLeagueEntry {
  queueType: string;
  tier: string;
  rank: string;
  leaguePoints: number;
  wins: number;
  losses: number;
  veteran: boolean;
  inactive: boolean;
  freshBlood: boolean;
  hotStreak: boolean;
}

export interface RiotChampionMastery {
  championId: number;
  championLevel: number;
  championPoints: number;
  lastPlayTime: number;
  championPointsSinceLastLevel?: number;
  championPointsUntilNextLevel?: number;
  tokensEarned?: number;
}

export const RIOT_REGIONS = [
  { id: 'eun1', label: 'Europe Nordic & East (EUN1)', cluster: 'europe' },
  { id: 'euw1', label: 'Europe West (EUW1)', cluster: 'europe' },
  { id: 'na1', label: 'North America (NA1)', cluster: 'americas' },
  { id: 'kr', label: 'Korea (KR)', cluster: 'asia' },
  { id: 'br1', label: 'Brazil (BR1)', cluster: 'americas' },
  { id: 'la1', label: 'Latin America North (LA1)', cluster: 'americas' },
  { id: 'la2', label: 'Latin America South (LA2)', cluster: 'americas' },
  { id: 'oc1', label: 'Oceania (OC1)', cluster: 'sea' },
  { id: 'tr1', label: 'Turkey (TR1)', cluster: 'europe' },
  { id: 'ru', label: 'Russia (RU)', cluster: 'europe' },
  { id: 'jp1', label: 'Japan (JP1)', cluster: 'asia' },
];

class RiotApiService {
  private getClusterForPlatform(platform: string): string {
    const found = RIOT_REGIONS.find((r) => r.id === platform.toLowerCase());
    return found ? found.cluster : 'europe';
  }

  private async makeRequest(url: string, apiKey: string): Promise<{ status: number; data: any }> {
    if (window.riftAPI && window.riftAPI.riotApiRequest) {
      try {
        const res = await window.riftAPI.riotApiRequest(url, apiKey);
        return res;
      } catch (err: any) {
        return { status: 500, data: { error: err.message } };
      }
    }

    // Direct fetch fallback
    try {
      const res = await fetch(url, {
        headers: {
          'X-Riot-Token': apiKey,
        },
      });
      const data = await res.json().catch(() => ({}));
      return { status: res.status, data };
    } catch (err: any) {
      return { status: 500, data: { error: err.message } };
    }
  }

  // Validate API Key by checking platform status or testing a ping
  public async validateKey(apiKey: string, platform: string = 'eun1'): Promise<{ valid: boolean; message: string; details?: any }> {
    if (!apiKey || !apiKey.trim()) {
      return { valid: false, message: 'API Key is empty. Please enter your RGAPI key.' };
    }

    const cleanKey = apiKey.trim();
    if (!cleanKey.startsWith('RGAPI-')) {
      return { valid: false, message: 'Invalid format. Riot API keys start with "RGAPI-".' };
    }

    const url = `https://${platform}.api.riotgames.com/lol/status/v4/platform-data`;
    const res = await this.makeRequest(url, cleanKey);

    if (res.status === 200) {
      return {
        valid: true,
        message: '🟢 API Key Verified & Active! Connected to Riot Games Web API.',
        details: res.data,
      };
    } else if (res.status === 401 || res.status === 403) {
      return {
        valid: false,
        message: '🔴 Invalid or Expired API Key (Error 403). Please generate a new key on developer.riotgames.com.',
      };
    } else if (res.status === 429) {
      return {
        valid: false,
        message: '⚠️ Rate limit exceeded (Error 429). Please wait a moment and try again.',
      };
    } else {
      return {
        valid: false,
        message: `HTTP Error ${res.status}: Failed connecting to Riot API.`,
      };
    }
  }

  // Look up Riot Account by GameName + TagLine (e.g. Faker#KR1)
  public async getAccountByRiotId(
    gameName: string,
    tagLine: string,
    apiKey: string,
    platform: string = 'eun1'
  ): Promise<RiotAccount | null> {
    const cluster = this.getClusterForPlatform(platform);
    const encName = encodeURIComponent(gameName.trim());
    const encTag = encodeURIComponent(tagLine.trim());
    const url = `https://${cluster}.api.riotgames.com/riot/account/v1/accounts/by-riot-id/${encName}/${encTag}`;

    const res = await this.makeRequest(url, apiKey);
    if (res.status === 200 && res.data?.puuid) {
      return res.data as RiotAccount;
    }
    return null;
  }

  // Look up Summoner profile by PUUID
  public async getSummonerByPuuid(
    puuid: string,
    apiKey: string,
    platform: string = 'eun1'
  ): Promise<RiotSummoner | null> {
    const url = `https://${platform}.api.riotgames.com/lol/summoner/v4/summoners/by-puuid/${puuid}`;
    const res = await this.makeRequest(url, apiKey);
    if (res.status === 200 && res.data?.id) {
      return res.data as RiotSummoner;
    }
    return null;
  }

  // Look up Ranked stats (tier, division, lp, wins/losses)
  public async getRankedEntries(
    summonerId: string,
    apiKey: string,
    platform: string = 'eun1'
  ): Promise<RiotLeagueEntry[]> {
    const url = `https://${platform}.api.riotgames.com/lol/league/v4/entries/by-summoner/${summonerId}`;
    const res = await this.makeRequest(url, apiKey);
    if (res.status === 200 && Array.isArray(res.data)) {
      return res.data as RiotLeagueEntry[];
    }
    return [];
  }

  // Look up recent match IDs
  public async getRecentMatchIds(
    puuid: string,
    apiKey: string,
    platform: string = 'eun1',
    count: number = 10
  ): Promise<string[]> {
    const cluster = this.getClusterForPlatform(platform);
    const url = `https://${cluster}.api.riotgames.com/lol/match/v5/matches/by-puuid/${puuid}/ids?start=0&count=${count}`;
    const res = await this.makeRequest(url, apiKey);
    if (res.status === 200 && Array.isArray(res.data)) {
      return res.data as string[];
    }
    return [];
  }

  // Look up Top Champion Masteries
  public async getTopChampionMasteries(
    puuid: string,
    apiKey: string,
    platform: string = 'eun1',
    count: number = 5
  ): Promise<RiotChampionMastery[]> {
    const url = `https://${platform}.api.riotgames.com/lol/champion-mastery/v4/champion-masteries/by-puuid/${puuid}/top?count=${count}`;
    const res = await this.makeRequest(url, apiKey);
    if (res.status === 200 && Array.isArray(res.data)) {
      return res.data as RiotChampionMastery[];
    }
    return [];
  }

  // Look up Single Match Details
  public async getMatchDetails(
    matchId: string,
    apiKey: string,
    platform: string = 'eun1'
  ): Promise<any | null> {
    const cluster = this.getClusterForPlatform(platform);
    const url = `https://${cluster}.api.riotgames.com/lol/match/v5/matches/${matchId}`;
    const res = await this.makeRequest(url, apiKey);
    if (res.status === 200 && res.data) {
      return res.data;
    }
    return null;
  }
}

export const riotApiService = new RiotApiService();
