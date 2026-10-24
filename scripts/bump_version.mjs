#!/usr/bin/env node
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT = path.resolve(__dirname, '..');

// 1. Read current version from package.json
const pkgPath = path.join(ROOT, 'package.json');
const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
const oldVersion = pkg.version;

// 2. Determine target version
const targetArg = (process.argv[2] || 'patch').trim().toLowerCase();
let newVersion = '';

const semverRegex = /^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?$/;
const match = oldVersion.match(semverRegex);
if (!match) {
  console.error(`[ERROR] Current version "${oldVersion}" is not standard semver!`);
  process.exit(1);
}

let [_, major, minor, patch, prerelease] = match;
let maj = parseInt(major, 10);
let min = parseInt(minor, 10);
let pat = parseInt(patch, 10);

if (targetArg === 'patch') {
  newVersion = `${maj}.${min}.${pat + 1}`;
} else if (targetArg === 'minor') {
  newVersion = `${maj}.${min + 1}.0`;
} else if (targetArg === 'major') {
  newVersion = `${maj + 1}.0.0`;
} else {
  // Direct version passed (e.g. 1.2.0 or v1.2.0)
  const cleanArg = targetArg.replace(/^v/, '');
  if (!semverRegex.test(cleanArg)) {
    console.error(`[ERROR] Invalid version provided: "${targetArg}". Must be semver (e.g. 1.1.3) or "patch", "minor", "major".`);
    process.exit(1);
  }
  newVersion = cleanArg;
}

if (oldVersion === newVersion) {
  console.log(`[INFO] Version is already ${newVersion}. Nothing to update.`);
  process.exit(0);
}

console.log(`\n===================================================`);
console.log(`  Bumping RiftCore Version: v${oldVersion} -> v${newVersion}`);
console.log(`===================================================\n`);

const updatedFiles = [];

// A. Update package.json
pkg.version = newVersion;
fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n', 'utf8');
updatedFiles.push('package.json');

// B. Update src/version.ts (Single source of truth for frontend)
const versionTsPath = path.join(ROOT, 'src', 'version.ts');
const versionTsContent = `/**
 * RiftCore Companion Version - Single Source of Truth
 * Automatically synchronized across backend, installer, and UI by \`npm run bump\`.
 */
export const APP_VERSION = '${newVersion}';
`;
fs.writeFileSync(versionTsPath, versionTsContent, 'utf8');
updatedFiles.push('src/version.ts');

// C. Update riftcore-native/Cargo.toml
const cargoPath = path.join(ROOT, 'riftcore-native', 'Cargo.toml');
if (fs.existsSync(cargoPath)) {
  let cargoContent = fs.readFileSync(cargoPath, 'utf8');
  cargoContent = cargoContent.replace(
    /(\[package\][\s\S]*?version\s*=\s*")[^"]+(")/,
    `$1${newVersion}$2`
  );
  fs.writeFileSync(cargoPath, cargoContent, 'utf8');
  updatedFiles.push('riftcore-native/Cargo.toml');
}

// D. Update installer/installer.nsi
const nsiPath = path.join(ROOT, 'installer', 'installer.nsi');
if (fs.existsSync(nsiPath)) {
  let nsiContent = fs.readFileSync(nsiPath, 'utf8');
  nsiContent = nsiContent.replace(
    /!define\s+PRODUCT_VERSION\s+"[^"]+"/,
    `!define PRODUCT_VERSION "${newVersion}"`
  );
  fs.writeFileSync(nsiPath, nsiContent, 'utf8');
  updatedFiles.push('installer/installer.nsi');
}

// E. Update installer/UPDATE_NOTES.txt header
const notesPath = path.join(ROOT, 'installer', 'UPDATE_NOTES.txt');
if (fs.existsSync(notesPath)) {
  let notesContent = fs.readFileSync(notesPath, 'utf8');
  notesContent = notesContent.replace(
    /RIFTCORE COMPANION - UPDATE v[0-9A-Za-z.-]+/g,
    `RIFTCORE COMPANION - UPDATE v${newVersion}`
  );
  notesContent = notesContent.replace(
    /Hey! Welcome to v[0-9A-Za-z.-]+/g,
    `Hey! Welcome to v${newVersion}`
  );
  fs.writeFileSync(notesPath, notesContent, 'utf8');
  updatedFiles.push('installer/UPDATE_NOTES.txt');
}

// F. Update README.md
const readmePath = path.join(ROOT, 'README.md');
if (fs.existsSync(readmePath)) {
  let readmeContent = fs.readFileSync(readmePath, 'utf8');
  const oldEscaped = oldVersion.replace(/\./g, '\\.');
  const oldRegex = new RegExp(oldEscaped, 'g');
  readmeContent = readmeContent.replace(oldRegex, newVersion);
  fs.writeFileSync(readmePath, readmeContent, 'utf8');
  updatedFiles.push('README.md');
}

// Report
for (const file of updatedFiles) {
  console.log(`  [OK] Synchronized version in ${file}`);
}

console.log(`\n[SUCCESS] RiftCore version successfully bumped to v${newVersion}!`);
console.log(`Next steps:`);
console.log(`  1. Run: npm run build:all        (To compile new exe, installer, and zip)`);
console.log(`  2. Run: npm run release          (To publish v${newVersion} to GitHub Releases)\n`);
