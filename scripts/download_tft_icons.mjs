import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..');
const dataDir = path.join(repoRoot, 'riftcore-native', 'assets', 'data');
const outputDir = path.join(repoRoot, 'riftcore-native', 'assets', 'tft_icons');
const championData = JSON.parse(await fs.readFile(path.join(dataDir, 'tft-champion.json'), 'utf8'));
const version = championData.version;
const championEntries = Object.entries(championData.data ?? {});
const set = Math.max(...championEntries.map(([key]) => Number(key.match(/TFTSet(\d+)/)?.[1] ?? 0)));
const marker = `TFTSet${set}`;

const sources = [
  {
    kind: 'champion', folder: 'tft-champion', file: 'tft-champion.json',
    include: (key, value) => key.includes(marker) && Number(value.cost ?? 0) > 0,
  },
  {
    kind: 'trait', folder: 'tft-trait', file: 'tft-trait.json',
    include: (key, value) => `${key} ${value.id ?? ''} ${value.image?.full ?? ''}`.includes(`_${set}_`),
  },
  {
    kind: 'item', folder: 'tft-item', file: 'tft-item.json',
    include: key => key.includes(marker),
  },
  {
    kind: 'augment', folder: 'tft-augment', file: 'tft-augments.json',
    include: (_key, value) => (value.id ?? '').includes(`_${set}_`),
  },
];

await fs.mkdir(outputDir, { recursive: true });
const downloads = [];
for (const source of sources) {
  const document = JSON.parse(await fs.readFile(path.join(dataDir, source.file), 'utf8'));
  for (const [key, value] of Object.entries(document.data ?? {})) {
    if (!source.include(key, value)) continue;
    const image = value.image?.full;
    if (!image) continue;
    const safeId = (value.id ?? key).replace(/[^a-z0-9_-]/gi, '_');
    const destination = path.join(outputDir, `${source.kind}_${safeId}.png`);
    downloads.push({ source, image, destination });
  }
}

let next = 0;
let downloaded = 0;
const failed = [];
async function worker() {
  while (next < downloads.length) {
    const current = downloads[next++];
    const url = `https://ddragon.leagueoflegends.com/cdn/${version}/img/${current.source.folder}/${encodeURIComponent(current.image)}`;
    try {
      const response = await fetch(url);
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      await fs.writeFile(current.destination, Buffer.from(await response.arrayBuffer()));
      downloaded++;
    } catch (error) {
      failed.push(`${current.source.kind}:${current.image}: ${error.message}`);
    }
  }
}

await Promise.all(Array.from({ length: 12 }, worker));
console.log(`Downloaded ${downloaded}/${downloads.length} Set ${set} TFT icons from Data Dragon ${version}.`);
if (failed.length) {
  console.warn(`${failed.length} optional icons were unavailable; the app will use text labels for those entries.`);
  for (const line of failed.slice(0, 12)) console.warn(`  ${line}`);
}
