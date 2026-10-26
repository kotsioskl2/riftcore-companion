#!/usr/bin/env node
import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT = path.resolve(__dirname, '..');

// 1. Target bump argument (default to 'patch')
const targetArg = (process.argv[2] || 'patch').trim().toLowerCase();

console.log('\n===================================================');
console.log('       RiftCore - 1-Click Automated Release');
console.log('===================================================\n');

function run(cmd, desc) {
  console.log(`\n>>> [STEP] ${desc}...`);
  try {
    execSync(cmd, { cwd: ROOT, stdio: 'inherit' });
  } catch (err) {
    console.error(`\n[ERROR] Step failed: ${desc}`);
    process.exit(1);
  }
}

// Step 1: Bump version across all files (unless 'current' or 'no-bump' requested)
if (targetArg !== 'current' && targetArg !== 'no-bump') {
  run(`node scripts/bump_version.mjs ${targetArg}`, 'Auto-bumping & synchronizing version across all files');
} else {
  console.log('>>> [STEP] Skipping version bump (releasing current version)...');
}

// Read the active version
const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, 'package.json'), 'utf8'));
const version = pkg.version;

// Step 2: Build frontend, native Rust release binary, installer, and zip
run('cmd.exe /c build_all.bat', `Building complete v${version} binary and packages`);

// Step 3: Git stage and commit
run('git add -A', 'Staging release files in git');
try {
  run(`git commit -m "chore(release): v${version}"`, `Committing release v${version}`);
} catch {
  console.log('Working tree already clean, skipping commit.');
}

// Step 4: Git push to main and update git tag
run('git push origin main', 'Pushing release commit to GitHub main');
try {
  run(`git tag -a v${version} -m "Release v${version}" -f`, `Tagging release v${version}`);
  run(`git push origin v${version} -f`, `Pushing git tag v${version} to GitHub`);
} catch (e) {
  console.warn('Warning: Could not push git tag:', e?.message || e);
}

// Step 5: Publish release to GitHub and upload RiftCore.exe
run('powershell -ExecutionPolicy Bypass -File scripts/publish_release.ps1', `Publishing v${version} to GitHub Releases`);

console.log('\n===================================================');
console.log(`  RELEASE v${version} SUCCESSFULLY DEPLOYED!`);
console.log('===================================================');
console.log(`  1. Git commit & push:  Pushed to origin main`);
console.log(`  2. Release Binaries:   Built & packaged in installer_output/`);
console.log(`  3. GitHub Release:     Live at https://github.com/kotsioskl2/riftcore-companion/releases/tag/v${version}`);
console.log(`  4. Auto-Update:        All active RiftCore apps will now prompt and auto-update!`);
console.log('===================================================\n');
