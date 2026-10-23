// Auto-Patch Synchronization Service for League of Legends (Data Dragon & LCU)

export interface ChampionSummary {
  id: number;
  name: string;
  key: string;
  title: string;
  roles: string[];
}

export interface PatchState {
  version: string;
  majorMinor: string;
  lastChecked: number;
  isChecking: boolean;
  isUpToDate: boolean;
  totalChampions: number;
}

const STORAGE_KEY_VERSION = 'riftcore_patch_version';
const STORAGE_KEY_CHAMPS = 'riftcore_cached_champions';
const DEFAULT_VERSION = '16.21.1';

class PatchService {
  private currentVersion: string = DEFAULT_VERSION;
  private cachedChampions: ChampionSummary[] | null = null;
  private isChecking = false;
  private listeners: ((state: PatchState) => void)[] = [];
  private syncIntervalId: any = null;

  constructor() {
    // 1. Read persisted patch version
    try {
      const savedVersion = localStorage.getItem(STORAGE_KEY_VERSION);
      if (savedVersion) {
        this.currentVersion = savedVersion;
      }
      const savedChamps = localStorage.getItem(STORAGE_KEY_CHAMPS);
      if (savedChamps) {
        this.cachedChampions = JSON.parse(savedChamps);
      }
    } catch {}

    // 2. Initial silent background check on startup
    setTimeout(() => {
      this.checkAndApplyPatchUpdate().catch(() => {});
    }, 1500);

    // 3. Periodic background check every 30 minutes
    this.startBackgroundSync(30 * 60 * 1000);
  }

  public getVersion(): string {
    return this.currentVersion;
  }

  public getMajorMinor(): string {
    const parts = this.currentVersion.split('.');
    return parts.length >= 2 ? `${parts[0]}.${parts[1]}` : this.currentVersion;
  }

  public getChampions(): ChampionSummary[] | null {
    return this.cachedChampions;
  }

  public getPatchState(): PatchState {
    return {
      version: this.currentVersion,
      majorMinor: this.getMajorMinor(),
      lastChecked: Date.now(),
      isChecking: this.isChecking,
      isUpToDate: true,
      totalChampions: this.cachedChampions?.length || 173,
    };
  }

  public subscribe(listener: (state: PatchState) => void): () => void {
    this.listeners.push(listener);
    listener(this.getPatchState());
    return () => {
      this.listeners = this.listeners.filter((l) => l !== listener);
    };
  }

  private notify() {
    const state = this.getPatchState();
    this.listeners.forEach((l) => l(state));
  }

  // Check Riot's Data Dragon versions.json and automatically download newly added champions
  public async checkAndApplyPatchUpdate(): Promise<{ updated: boolean; version: string; championCount: number }> {
    if (this.isChecking) {
      return { updated: false, version: this.currentVersion, championCount: this.cachedChampions?.length || 173 };
    }

    this.isChecking = true;
    this.notify();

    try {
      console.log('[PatchService] Checking latest Riot Data Dragon patch...');
      const vRes = await fetch('https://ddragon.leagueoflegends.com/api/versions.json', { cache: 'no-cache' });
      if (!vRes.ok) throw new Error(`HTTP error fetching versions: ${vRes.status}`);

      const versions: string[] = await vRes.json();
      if (!Array.isArray(versions) || versions.length === 0) {
        throw new Error('Invalid versions payload received');
      }

      const latestVersion = versions[0];
      const isNewVersion = latestVersion !== this.currentVersion || !this.cachedChampions;

      if (isNewVersion) {
        console.log(`[PatchService] New patch version discovered: ${latestVersion} (Previous: ${this.currentVersion}). Downloading champion manifest...`);
        const cRes = await fetch(`https://ddragon.leagueoflegends.com/cdn/${latestVersion}/data/en_US/champion.json`);
        if (!cRes.ok) throw new Error(`Failed to download champions for patch ${latestVersion}`);

        const champData = await cRes.json();
        const roleMap: Record<string, string[]> = {
          Fighter: ['TOP', 'JUNGLE'],
          Mage: ['MID', 'SUPPORT'],
          Assassin: ['MID', 'JUNGLE'],
          Marksman: ['ADC'],
          Tank: ['TOP', 'SUPPORT'],
          Support: ['SUPPORT'],
        };

        const parsedChampions: ChampionSummary[] = Object.values(champData.data).map((c: any) => {
          const primaryTag = c.tags?.[0] || 'Fighter';
          let roles = roleMap[primaryTag] ? [...roleMap[primaryTag]] : ['MID'];
          if (c.tags?.includes('Marksman') && !roles.includes('ADC')) roles = ['ADC', ...roles];
          if (c.tags?.includes('Support') && !roles.includes('SUPPORT')) roles = [...roles, 'SUPPORT'];
          return {
            id: parseInt(c.key, 10),
            name: c.name,
            key: c.id,
            title: c.title,
            roles: Array.from(new Set(roles)),
          };
        }).sort((a, b) => a.name.localeCompare(b.name));

        this.currentVersion = latestVersion;
        this.cachedChampions = parsedChampions;

        try {
          localStorage.setItem(STORAGE_KEY_VERSION, latestVersion);
          localStorage.setItem(STORAGE_KEY_CHAMPS, JSON.stringify(parsedChampions));
        } catch {}

        console.log(`[PatchService] Successfully synced Patch ${latestVersion} with ${parsedChampions.length} champions!`);
        this.isChecking = false;
        this.notify();
        return { updated: true, version: latestVersion, championCount: parsedChampions.length };
      }

      this.isChecking = false;
      this.notify();
      return { updated: false, version: this.currentVersion, championCount: this.cachedChampions?.length || 173 };
    } catch (err) {
      console.warn('[PatchService] Failed to check for patch update, keeping current version:', err);
      this.isChecking = false;
      this.notify();
      return { updated: false, version: this.currentVersion, championCount: this.cachedChampions?.length || 173 };
    }
  }

  public startBackgroundSync(intervalMs: number) {
    if (this.syncIntervalId) {
      clearInterval(this.syncIntervalId);
    }
    this.syncIntervalId = setInterval(() => {
      this.checkAndApplyPatchUpdate().catch(() => {});
    }, intervalMs);
  }

  // Dynamic Image URL Helpers using the active patch version
  public getChampionIconUrl(championKey: string): string {
    return `https://ddragon.leagueoflegends.com/cdn/${this.currentVersion}/img/champion/${championKey}.png`;
  }

  public getItemIconUrl(itemId: number): string {
    return `https://ddragon.leagueoflegends.com/cdn/${this.currentVersion}/img/item/${itemId}.png`;
  }

  public getSpellIconUrl(spellName: string): string {
    const formatted = spellName === 'Ghost' ? 'Haste' : spellName === 'Ignite' ? 'Dot' : spellName === 'Cleanse' ? 'Boost' : spellName;
    return `https://ddragon.leagueoflegends.com/cdn/${this.currentVersion}/img/spell/Summoner${formatted}.png`;
  }

  public getChampionSplashUrl(championKey: string): string {
    return `https://ddragon.leagueoflegends.com/cdn/img/champion/splash/${championKey}_0.jpg`;
  }
}

export const patchService = new PatchService();
