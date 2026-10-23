const fs = require('fs');
const champions = JSON.parse(fs.readFileSync('all_champions.json', 'utf8'));
let ts = fs.readFileSync('src/services/championData.ts', 'utf8');

// Ensure import of patchService
if (!ts.includes("import { patchService } from './patchService';")) {
  ts = "import { patchService } from './patchService';\n" + ts;
}

// Replace CHAMPIONS_LIST with full 173 champions list
const newChampionsListStr = 'export const CHAMPIONS_LIST: { id: number; name: string; key: string; title: string; roles: string[] }[] = ' + JSON.stringify(champions, null, 2) + ';';

const listRegex = /export const CHAMPIONS_LIST: [^=]+=\s*\[[\s\S]*?\n\];/;
ts = ts.replace(listRegex, newChampionsListStr);

// Wire getChampionIcon and getChampionSplash to patchService
const iconFunc = `export function getChampionIcon(championIdOrKey: number | string): string {
  if (typeof championIdOrKey === 'number') {
    const list = patchService.getChampions() || CHAMPIONS_LIST;
    const found = list.find((c) => c.id === championIdOrKey);
    if (found) {
      return patchService.getChampionIconUrl(found.key);
    }
    return \`https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/\${championIdOrKey}.png\`;
  }
  return patchService.getChampionIconUrl(championIdOrKey);
}

export function getChampionSplash(championKey: string): string {
  return patchService.getChampionSplashUrl(championKey);
}`;

const iconRegex = /export function getChampionIcon[\s\S]*?export function getChampionSplash[\s\S]*?\n\}/;
ts = ts.replace(iconRegex, iconFunc);

fs.writeFileSync('src/services/championData.ts', ts, 'utf8');
console.log('championData.ts updated with patchService and', champions.length, 'champions!');
