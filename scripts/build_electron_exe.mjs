import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';

const ROOT = path.resolve('.');
const STAGING = path.join(ROOT, 'build_staging');
const TARGET_DIR = path.join(ROOT, 'RiftCore_Rust_Portable', 'overlay');
const DIST_OVERLAY = path.join(ROOT, 'dist_overlay', 'RiftCore_Overlay-win32-x64');

console.log('\n===================================================');
console.log('    RiftCore - Hybrid Overlay Packager');
console.log('===================================================\n');

// 1. Build Vite Frontend
console.log('>>> [1/4] Building Vite Frontend (dist/)...');
execSync('npm run build', { cwd: ROOT, stdio: 'inherit' });

// 2. Assemble Staging Files
console.log('>>> [2/4] Assembling Clean Runtime Staging Area...');
if (fs.existsSync(STAGING)) {
  fs.rmSync(STAGING, { recursive: true, force: true });
}
fs.mkdirSync(STAGING, { recursive: true });

fs.cpSync(path.join(ROOT, 'dist'), path.join(STAGING, 'dist'), { recursive: true });
fs.cpSync(path.join(ROOT, 'electron'), path.join(STAGING, 'electron'), { recursive: true });
if (fs.existsSync(path.join(ROOT, 'public'))) {
  fs.cpSync(path.join(ROOT, 'public'), path.join(STAGING, 'public'), { recursive: true });
}

// Minimal runtime package.json
const rootPkg = JSON.parse(fs.readFileSync(path.join(ROOT, 'package.json'), 'utf8'));
const runtimePkg = {
  name: 'riftcore-overlay',
  version: rootPkg.version,
  main: 'electron/main.js',
  dependencies: {
    ws: rootPkg.dependencies?.ws || rootPkg.devDependencies?.ws || '^8.22.0'
  }
};
fs.writeFileSync(path.join(STAGING, 'package.json'), JSON.stringify(runtimePkg, null, 2), 'utf8');

// Copy production dependencies (ws)
const stagingModules = path.join(STAGING, 'node_modules');
fs.mkdirSync(stagingModules, { recursive: true });
const wsModule = path.join(ROOT, 'node_modules', 'ws');
if (fs.existsSync(wsModule)) {
  fs.cpSync(wsModule, path.join(stagingModules, 'ws'), { recursive: true });
}

// 3. Setup Target Directory & Copy Electron Binaries
console.log('>>> [3/4] Preparing Overlay Distribution Binaries...');
const electronDist = path.join(ROOT, 'node_modules', 'electron', 'dist');

function deployTo(targetPath) {
  if (fs.existsSync(targetPath)) {
    try {
      fs.rmSync(targetPath, { recursive: true, force: true });
    } catch {}
  }
  fs.mkdirSync(targetPath, { recursive: true });

  // Copy Electron runtime
  fs.cpSync(electronDist, targetPath, { recursive: true });

  // Rename electron.exe -> RiftCore_Overlay.exe
  const origExe = path.join(targetPath, 'electron.exe');
  const overlayExe = path.join(targetPath, 'RiftCore_Overlay.exe');
  if (fs.existsSync(origExe)) {
    if (fs.existsSync(overlayExe)) fs.rmSync(overlayExe, { force: true });
    fs.renameSync(origExe, overlayExe);
  }

  // Pack app.asar directly into target/resources/app.asar
  const resourcesDir = path.join(targetPath, 'resources');
  fs.mkdirSync(resourcesDir, { recursive: true });
  const asarTarget = path.join(resourcesDir, 'app.asar');
  execSync(`npx asar pack "${STAGING}" "${asarTarget}"`, { cwd: ROOT, stdio: 'inherit' });
}

deployTo(TARGET_DIR);
deployTo(DIST_OVERLAY);

// 4. Cleanup Staging
fs.rmSync(STAGING, { recursive: true, force: true });

console.log('\n===================================================');
console.log('   HYBRID OVERLAY PACKAGED SUCCESSFULLY!');
console.log(`   Overlay 1: ${path.join(TARGET_DIR, 'RiftCore_Overlay.exe')}`);
console.log(`   Overlay 2: ${path.join(DIST_OVERLAY, 'RiftCore_Overlay.exe')}`);
console.log('===================================================\n');
