import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';

const ROOT = path.resolve('.');
const STAGING = path.join(ROOT, 'build_staging');
const TEMP_BASE = 'F:\\temp';
if (!fs.existsSync(TEMP_BASE)) fs.mkdirSync(TEMP_BASE, { recursive: true });

// Always terminate any old instances before packaging to avoid file locks
try {
  execSync('taskkill /F /IM RiftCore.exe /IM RiftCore_Overlay.exe /IM focus_watcher.exe 2>nul', { stdio: 'ignore' });
} catch {}

const BUILD_ID = `rc_pkg_${Date.now()}`;
const PKG_OUT = path.join(TEMP_BASE, BUILD_ID);
const PACKAGED_APP = path.join(PKG_OUT, 'RiftCore-win32-x64');
const PORTABLE_DIR = path.join(ROOT, 'RiftCore_Rust_Portable');

console.log('\n===================================================');
console.log('    RiftCore - Unified Application Packager');
console.log('===================================================\n');

// 1. Build Vite Frontend
console.log('>>> [1/4] Building Vite Frontend (dist/)...');
execSync('npm run build', { cwd: ROOT, stdio: 'inherit' });

// 2. Ensure Focus Watcher is Compiled
console.log('>>> [2/4] Verifying Alt-Tab Focus Watcher Engine...');
const focusSource = path.join(ROOT, 'electron', 'bin', 'focus_watcher.c');
const focusExe = path.join(ROOT, 'electron', 'bin', 'focus_watcher.exe');
if (fs.existsSync(focusSource)) {
  try {
    const gccPath = 'F:\\w64devkit\\bin\\gcc.exe';
    const compiler = fs.existsSync(gccPath) ? gccPath : 'gcc';
    execSync(`"${compiler}" -O2 -s -o "${focusExe}" "${focusSource}" -luser32`, { cwd: ROOT, stdio: 'ignore' });
    console.log('    Focus Watcher compiled successfully.');
  } catch (e) {
    if (fs.existsSync(focusExe)) {
      console.log('    Using pre-compiled Focus Watcher binary.');
    } else {
      console.warn('    Warning: gcc not found, could not compile focus watcher.');
    }
  }
}

// 3. Assemble Lightweight Staging Area (< 15 MB)
console.log('>>> [3/4] Assembling Clean Application Staging Area...');
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
  name: 'riftcore-companion',
  productName: 'RiftCore Companion',
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

// 4. Package Unified Application via electron-packager
console.log('>>> [4/4] Packaging Unified Standalone Application...');
fs.mkdirSync(PKG_OUT, { recursive: true });

const packagerArgs = [
  `npx electron-packager "${STAGING}" RiftCore`,
  '--platform=win32',
  '--arch=x64',
  `--out="${PKG_OUT}"`,
  '--overwrite',
  `--icon="${path.join(ROOT, 'dist', 'icon.ico')}"`,
  '--prune=false',
  '--win32metadata.ProductName="RiftCore Companion"',
  '--win32metadata.FileDescription="RiftCore Companion"',
  '--win32metadata.CompanyName="RiftCore Team"',
  '--win32metadata.InternalName="RiftCore"',
  '--win32metadata.OriginalFilename="RiftCore.exe"',
].join(' ');

execSync(packagerArgs, { cwd: ROOT, stdio: 'inherit' });

// Deploy focus_watcher.exe to packaged directory
if (fs.existsSync(focusExe)) {
  try {
    fs.copyFileSync(focusExe, path.join(PACKAGED_APP, 'focus_watcher.exe'));
    const resDir = path.join(PACKAGED_APP, 'resources');
    if (fs.existsSync(resDir)) {
      fs.copyFileSync(focusExe, path.join(resDir, 'focus_watcher.exe'));
    }
  } catch {}
}

// 5. Populate Portable Directory
if (!fs.existsSync(PORTABLE_DIR)) {
  fs.mkdirSync(PORTABLE_DIR, { recursive: true });
}

function copyDirSafe(src, dest) {
  if (!fs.existsSync(dest)) fs.mkdirSync(dest, { recursive: true });
  const entries = fs.readdirSync(src, { withFileTypes: true });
  for (const entry of entries) {
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);
    if (entry.isDirectory()) {
      copyDirSafe(srcPath, destPath);
    } else {
      try {
        fs.copyFileSync(srcPath, destPath);
      } catch (err) {
        try {
          fs.unlinkSync(destPath);
          fs.copyFileSync(srcPath, destPath);
        } catch {}
      }
    }
  }
}

copyDirSafe(PACKAGED_APP, PORTABLE_DIR);

// Ensure icon.ico and LICENSE exist in portable root
const iconSource = path.join(ROOT, 'public', 'icon.ico');
if (fs.existsSync(iconSource)) {
  try { fs.copyFileSync(iconSource, path.join(PORTABLE_DIR, 'icon.ico')); } catch {}
}
const licenseSource = path.join(ROOT, 'LICENSE');
if (fs.existsSync(licenseSource)) {
  try { fs.copyFileSync(licenseSource, path.join(PORTABLE_DIR, 'LICENSE')); } catch {}
}

// Backward-compatible overlay executable copy
const overlayDir = path.join(PORTABLE_DIR, 'overlay');
if (!fs.existsSync(overlayDir)) {
  fs.mkdirSync(overlayDir, { recursive: true });
}
const portableExe = path.join(PORTABLE_DIR, 'RiftCore.exe');
const overlayExe = path.join(overlayDir, 'RiftCore_Overlay.exe');
if (fs.existsSync(portableExe)) {
  try {
    fs.copyFileSync(portableExe, overlayExe);
  } catch {}
}

// Clean up temporary build folder and staging
try { fs.rmSync(STAGING, { recursive: true, force: true }); } catch {}
try { fs.rmSync(PKG_OUT, { recursive: true, force: true }); } catch {}

console.log('\n===================================================');
console.log('   UNIFIED RIFTCORE PACKAGED SUCCESSFULLY!');
console.log(`   Standalone App: ${path.join(PORTABLE_DIR, 'RiftCore.exe')}`);
console.log(`   Portable Dir:   ${PORTABLE_DIR}`);
console.log('===================================================\n');
