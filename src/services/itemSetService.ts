// RiftCore In-Game Shop Item Set Service
// Generates and pushes League of Legends Item Sets directly into the in-game shop (Press 'P' in game)
// Follows Riot Games LCU /lol-item-sets/v1/item-sets API schema

import { ChampionBuild } from '../types/index';

export interface ItemSetBlock {
  type: string;
  items: { id: string; count: number }[];
}

export interface RiotItemSet {
  title: string;
  type: 'custom';
  map: 'any';
  mode: 'any';
  priority: boolean;
  sortrank: number;
  associatedChampions: number[];
  associatedMaps: number[];
  blocks: ItemSetBlock[];
}

export class ItemSetService {
  // Format full in-game shop item set for a specific champion build
  public createItemSetForBuild(build: ChampionBuild, enemyTeamChampions: string[] = []): RiotItemSet {
    const blocks: ItemSetBlock[] = [];

    // 1. Starting Items
    blocks.push({
      type: '⚡ Starting Items & Potions',
      items: build.startingItems.map((it) => ({ id: String(it.id), count: it.id === 2003 ? 2 : 1 })),
    });

    // 2. Early Purchases & Boots
    const earlyItems: { id: string; count: number }[] = [
      { id: String(build.boots.id), count: 1 },
    ];
    // Add Tier 1 boots and component items based on role
    if (build.role === 'ADC') {
      earlyItems.push({ id: '1001', count: 1 }); // Boots of Speed
      earlyItems.push({ id: '1038', count: 1 }); // B.F. Sword
      earlyItems.push({ id: '1037', count: 1 }); // Pickaxe
    } else if (build.role === 'MID') {
      earlyItems.push({ id: '1082', count: 1 }); // Dark Seal
      earlyItems.push({ id: '3802', count: 1 }); // Lost Chapter
      earlyItems.push({ id: '3108', count: 1 }); // Fiendish Codex
    } else if (build.role === 'JUNGLE') {
      earlyItems.push({ id: '1001', count: 1 }); // Boots
      earlyItems.push({ id: '3133', count: 1 }); // Caulfield's
      earlyItems.push({ id: '3067', count: 1 }); // Kindlegem
    } else if (build.role === 'SUPPORT') {
      earlyItems.push({ id: '1001', count: 1 }); // Boots
      earlyItems.push({ id: '3067', count: 1 }); // Kindlegem
      earlyItems.push({ id: '3114', count: 1 }); // Forbidden Idol
    } else {
      earlyItems.push({ id: '1001', count: 1 }); // Boots
      earlyItems.push({ id: '3077', count: 1 }); // Tiamat
      earlyItems.push({ id: '3067', count: 1 }); // Kindlegem
    }

    blocks.push({
      type: '👟 Early Buys & Completed Boots',
      items: earlyItems,
    });

    // 3. Core Item Path (In Optimal Order)
    blocks.push({
      type: '⚔️ Core Mythic & Legendary Build',
      items: build.coreItems.map((it) => ({ id: String(it.id), count: 1 })),
    });

    // 4. Situational & Counter Items (Anti-Heal, Anti-Shield, Anti-Armor, Defensive)
    const situational: { id: string; count: number }[] = build.situationalItems.map((it) => ({
      id: String(it.id),
      count: 1,
    }));

    // Dynamic anti-heal injection
    if (build.role === 'ADC' || build.role === 'TOP') {
      if (!situational.some((i) => i.id === '3033')) situational.push({ id: '3033', count: 1 }); // Mortal Reminder
      if (!situational.some((i) => i.id === '6609')) situational.push({ id: '6609', count: 1 }); // Chempunk Chainsword
    } else if (build.role === 'MID') {
      if (!situational.some((i) => i.id === '3165')) situational.push({ id: '3165', count: 1 }); // Morellonomicon
      if (!situational.some((i) => i.id === '3135')) situational.push({ id: '3135', count: 1 }); // Void Staff
    } else if (build.role === 'SUPPORT') {
      if (!situational.some((i) => i.id === '3075')) situational.push({ id: '3075', count: 1 }); // Thornmail
      if (!situational.some((i) => i.id === '3107')) situational.push({ id: '3107', count: 1 }); // Redemption
    }

    blocks.push({
      type: '🛡️ Situational Counters (Anti-Heal / Defense)',
      items: situational.slice(0, 6),
    });

    // 5. Consumables & Elixirs
    const elixirs: { id: string; count: number }[] = [
      { id: '2055', count: 2 }, // Control Ward
    ];
    if (build.role === 'MID' || (build.role === 'SUPPORT' && build.coreItems.some((i) => i.id === 3089 || i.id === 6653))) {
      elixirs.push({ id: '2139', count: 1 }); // Elixir of Sorcery
    } else if (build.role === 'TOP' || build.role === 'SUPPORT') {
      elixirs.push({ id: '2138', count: 1 }); // Elixir of Iron
      elixirs.push({ id: '2140', count: 1 }); // Elixir of Wrath
    } else {
      elixirs.push({ id: '2140', count: 1 }); // Elixir of Wrath
    }

    blocks.push({
      type: '🧪 Consumables & Elixirs',
      items: elixirs,
    });

    return {
      title: `⚡ RiftCore: ${build.championName} Meta`,
      type: 'custom',
      map: 'any',
      mode: 'any',
      priority: true,
      sortrank: 1,
      associatedChampions: [build.championId],
      associatedMaps: [11, 12], // Summoner's Rift & Howling Abyss
      blocks,
    };
  }

  // Push Item Set directly into League of Legends Shop via LCU API
  public async exportItemSetToClient(build: ChampionBuild, enemyChampions: string[] = []): Promise<{ success: boolean; message: string }> {
    const itemSet = this.createItemSetForBuild(build, enemyChampions);

    // 1. If running under RiftCore Native or Electron with dedicated export API
    if (window.riftAPI?.exportItemSet) {
      try {
        await window.riftAPI.exportItemSet(itemSet);
        return {
          success: true,
          message: `✓ In-Game Shop Item Set created for ${build.championName}! Press 'P' in game to open shop.`,
        };
      } catch (err: any) {
        console.warn('[ItemSetService] exportItemSet failed, attempting direct LCU fallback...', err);
      }
    }

    // 2. Direct LCU API Fallback via window.riftAPI.lcuRequest
    if (window.riftAPI?.lcuRequest) {
      try {
        // Fetch current summoner to get summonerId
        const summoner = await window.riftAPI.lcuRequest('/lol-summoner/v1/current-summoner');
        if (!summoner || !summoner.summonerId) {
          throw new Error('League Client not logged in or summoner unavailable.');
        }

        const summonerId = summoner.summonerId;

        // Fetch existing sets
        let existingData: { itemSets: any[]; timestamp?: number } = { itemSets: [], timestamp: Date.now() };
        try {
          const res = await window.riftAPI.lcuRequest(`/lol-item-sets/v1/item-sets/${summonerId}/sets`);
          if (res && Array.isArray(res.itemSets)) {
            existingData = res;
          }
        } catch {}

        // Remove old RiftCore item set for this champion
        const targetChampId = build.championId;
        existingData.itemSets = existingData.itemSets.filter((s: any) => {
          const isRiftTitle = typeof s.title === 'string' && s.title.startsWith('⚡ RiftCore:');
          const hasThisChamp = Array.isArray(s.associatedChampions) && s.associatedChampions.includes(targetChampId);
          return !(isRiftTitle && hasThisChamp);
        });

        // Add fresh item set
        existingData.itemSets.push(itemSet);

        // Put back to LCU
        await window.riftAPI.lcuRequest(`/lol-item-sets/v1/item-sets/${summonerId}/sets`, 'PUT', existingData);

        return {
          success: true,
          message: `✓ In-Game Shop Item Set created for ${build.championName}! Press 'P' in game to open shop.`,
        };
      } catch (err: any) {
        return {
          success: false,
          message: `Could not export item set to League Client: ${err.message || 'LCU request failed'}`,
        };
      }
    }

    return {
      success: false,
      message: 'League of Legends Client is not running. Please open League Client first.',
    };
  }
}

export const itemSetService = new ItemSetService();
