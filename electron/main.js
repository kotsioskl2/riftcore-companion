const { app, BrowserWindow, ipcMain, shell, globalShortcut, screen, Tray, Menu } = require('electron');
const path = require('path');
const fs = require('fs');
const https = require('https');
const { exec, spawn } = require('child_process');
const WebSocket = require('ws');

// Set Application Identity for Windows Task Manager & Shell grouping
app.setName('RiftCore Companion');
if (process.platform === 'win32') {
  app.setAppUserModelId('com.riftcore.companion');
}

// Disable TLS verification for local self-signed Riot certificates
process.env["NODE_TLS_REJECT_UNAUTHORIZED"] = "0";

process.on('uncaughtException', (err) => {
  console.error('[RiftCore FATAL uncaughtException]:', err);
});

process.on('unhandledRejection', (reason) => {
  console.error('[RiftCore FATAL unhandledRejection]:', reason);
});

const httpsAgent = new https.Agent({ rejectUnauthorized: false });

const isOverlayOnly = process.argv.includes('--overlay') || process.execPath.toLowerCase().includes('overlay');

let mainWindow = null;
let overlayWindow = null;
let tray = null;
let isCompactMode = false;
let isPinned = false;
let currentHudMode = 'integrated';
let overlayClickThrough = true;

// Alt-Tab Auto-Hide & Focus Tracking State
let isLeagueInForeground = false;
let focusWatcherProc = null;

function getFocusWatcherPath() {
  const userPath = path.join(app.getPath('userData'), 'focus_watcher.exe');
  const bundledPath = path.join(__dirname, 'bin', 'focus_watcher.exe');
  const resPath = path.join(process.resourcesPath || '', 'focus_watcher.exe');
  const besideExePath = path.join(path.dirname(process.execPath), 'focus_watcher.exe');

  if (fs.existsSync(besideExePath)) return besideExePath;
  if (fs.existsSync(resPath)) return resPath;
  if (fs.existsSync(bundledPath)) {
    try {
      if (!fs.existsSync(userPath) || fs.statSync(bundledPath).mtimeMs > fs.statSync(userPath).mtimeMs) {
        fs.copyFileSync(bundledPath, userPath);
      }
      return userPath;
    } catch {
      return bundledPath;
    }
  }
  if (fs.existsSync(userPath)) return userPath;
  return null;
}

function startFocusWatcher() {
  if (process.platform !== 'win32') return;
  if (focusWatcherProc) return;

  const watcherExe = getFocusWatcherPath();
  if (!watcherExe || !fs.existsSync(watcherExe)) {
    console.warn('[RiftCore] focus_watcher.exe not found, falling back to manual overlay hotkey');
    return;
  }

  try {
    focusWatcherProc = spawn(watcherExe, [process.pid.toString()], {
      windowsHide: true,
      stdio: ['ignore', 'pipe', 'ignore'],
    });

    focusWatcherProc.stdout.on('data', (chunk) => {
      const text = chunk.toString();
      if (text.includes('LEAGUE_FOCUSED')) {
        isLeagueInForeground = true;
        // League gained focus! Automatically restore overlay if in match
        const shouldShow = gameflowPhase === 'InProgress' || gameflowPhase === 'GameStart' || isOverlayOnly || lastLiveData !== null;
        if (shouldShow && overlayWindow && !overlayWindow.isDestroyed()) {
          overlayWindow.showInactive();
          overlayWindow.setAlwaysOnTop(true, 'screen-saver', 1);
          overlayWindow.moveTop();
          startTopmostWatcher();
        }
      } else if (text.includes('LEAGUE_BLURRED')) {
        isLeagueInForeground = false;
        stopTopmostWatcher();
        // User Alt-Tabbed out of League of Legends! Automatically hide overlay immediately!
        if (overlayWindow && !overlayWindow.isDestroyed() && overlayWindow.isVisible()) {
          overlayWindow.hide();
        }
      }
    });

    focusWatcherProc.on('exit', () => {
      focusWatcherProc = null;
      stopTopmostWatcher();
    });
  } catch (err) {
    console.warn('[RiftCore] Error starting focus watcher:', err);
  }
}

function stopFocusWatcher() {
  stopTopmostWatcher();
  if (focusWatcherProc) {
    try { focusWatcherProc.kill(); } catch {}
    focusWatcherProc = null;
  }
}

let topmostReinforceTimer = null;

function startTopmostWatcher() {
  if (topmostReinforceTimer) return;
  topmostReinforceTimer = setInterval(() => {
    if (isLeagueInForeground && overlayWindow && !overlayWindow.isDestroyed() && overlayWindow.isVisible()) {
      overlayWindow.setAlwaysOnTop(true, 'screen-saver', 1);
      overlayWindow.moveTop();
    }
  }, 2000);
}

function stopTopmostWatcher() {
  if (topmostReinforceTimer) {
    clearInterval(topmostReinforceTimer);
    topmostReinforceTimer = null;
  }
}

// -------------------------------------------------------------
// DYNAMIC INTERACTIVE REGION HIT-TESTING (WINDOWS CLICK-THROUGH)
// -------------------------------------------------------------
let interactiveRegions = [];
let mousePollingTimer = null;
let isCurrentlyInteractive = false;
let isLayoutEditMode = false;

function startMouseWatcher() {
  if (mousePollingTimer) return;
  mousePollingTimer = setInterval(() => {
    if (!overlayWindow || overlayWindow.isDestroyed() || !overlayWindow.isVisible()) {
      return;
    }
    if (isLayoutEditMode) {
      if (!isCurrentlyInteractive) {
        isCurrentlyInteractive = true;
        overlayWindow.setIgnoreMouseEvents(false);
      }
      return;
    }
    const point = screen.getCursorScreenPoint();
    const bounds = overlayWindow.getBounds();
    let hit = false;
    for (const r of interactiveRegions) {
      const rx = bounds.x + r.x;
      const ry = bounds.y + r.y;
      if (
        point.x >= rx &&
        point.x <= rx + r.width &&
        point.y >= ry &&
        point.y <= ry + r.height
      ) {
        hit = true;
        break;
      }
    }
    if (hit !== isCurrentlyInteractive) {
      isCurrentlyInteractive = hit;
      if (hit) {
        overlayWindow.setIgnoreMouseEvents(false);
      } else {
        overlayWindow.setIgnoreMouseEvents(true, { forward: true });
      }
    }
  }, 40);
}

function stopMouseWatcher() {
  if (mousePollingTimer) {
    clearInterval(mousePollingTimer);
    mousePollingTimer = null;
  }
}

// LCU State
let lcuCredentials = null;
let lcuWebSocket = null;
let gameflowPhase = 'None';
let champSelectSession = null;
let liveGameInterval = null;
let lastLiveData = null;

// Settings path
const settingsPath = path.join(app.getPath('userData'), 'riftcore_settings.json');
let appSettings = {
  autoAccept: true,
  autoImportRunes: true,
  autoImportSpells: true,
  soundAlerts: true,
  theme: 'hextech',
  alwaysOnTop: false,
  closeToTray: true,
  hudGoldLead: true,
  hudCsBenchmark: true,
  hudObjectiveTimers: true,
  hudSpellTimers: true,
  hudUltimateTimers: true,
  hudSkillOrder: true,
  hudItemGoal: true,
  hudTrinketReminder: true,
  hudMayhemCards: true,
  hudJunglePathing: true,
  hudAramHealthTimers: true,
  hudMinimapScale: 2.16,
  hudMinimapFlip: false,
};

function loadSettings() {
  try {
    if (fs.existsSync(settingsPath)) {
      const data = fs.readFileSync(settingsPath, 'utf8');
      appSettings = { ...appSettings, ...JSON.parse(data) };
    }
  } catch (err) {
    console.error('[RiftCore] Error loading settings:', err);
  }
}

function saveSettings(newSettings) {
  try {
    appSettings = { ...appSettings, ...newSettings };
    fs.writeFileSync(settingsPath, JSON.stringify(appSettings, null, 2));
  } catch (err) {
    console.error('[RiftCore] Error saving settings:', err);
  }
}

loadSettings();

function findLeagueGameCfg() {
  const candidatePaths = [
    'C:\\Riot Games\\League of Legends\\Config\\game.cfg',
    'D:\\Riot Games\\League of Legends\\Config\\game.cfg',
    'E:\\Riot Games\\League of Legends\\Config\\game.cfg',
    'F:\\Riot Games\\League of Legends\\Config\\game.cfg',
    'C:\\Riot Games\\League of Legends (PBE)\\Config\\game.cfg',
    'D:\\Riot Games\\League of Legends (PBE)\\Config\\game.cfg',
    'F:\\Riot Games\\League of Legends (PBE)\\Config\\game.cfg',
  ];

  if (lcuCredentials && lcuCredentials.installDir) {
    candidatePaths.unshift(path.join(lcuCredentials.installDir, 'Config', 'game.cfg'));
  }

  for (const p of candidatePaths) {
    if (fs.existsSync(p)) {
      return p;
    }
  }
  return null;
}

function parseLeagueGameCfg() {
  const cfgPath = findLeagueGameCfg();
  if (!cfgPath) return { found: false, windowMode: 0, minimapScale: 2.16, flipMinimap: false, path: '' };

  try {
    const content = fs.readFileSync(cfgPath, 'utf8');
    let windowMode = 0;
    let minimapScale = 2.16;
    let flipMinimap = false;

    for (const line of content.split('\n')) {
      const trimmed = line.trim();
      if (trimmed.startsWith('WindowMode=')) {
        windowMode = parseInt(trimmed.replace('WindowMode=', ''), 10) || 0;
      } else if (trimmed.startsWith('MinimapScale=')) {
        minimapScale = parseFloat(trimmed.replace('MinimapScale=', '')) || 2.16;
      } else if (trimmed.startsWith('FlipMiniMap=')) {
        flipMinimap = trimmed.replace('FlipMiniMap=', '') === '1';
      }
    }
    return { found: true, windowMode, minimapScale, flipMinimap, path: cfgPath };
  } catch {
    return { found: false, windowMode: 0, minimapScale: 2.16, flipMinimap: false, path: cfgPath };
  }
}

// -------------------------------------------------------------
// WINDOW MANAGEMENT: MAIN APP & DEDICATED TRANSPARENT OVERLAY
// -------------------------------------------------------------

function createMainWindow() {
  mainWindow = new BrowserWindow({
    width: 1240,
    height: 820,
    minWidth: 900,
    minHeight: 600,
    frame: false,
    transparent: true,
    backgroundColor: '#00000000',
    hasShadow: true,
    icon: path.join(app.getAppPath(), 'dist/icon.ico'),
    webPreferences: {
      preload: path.join(app.getAppPath(), 'electron/preload.js'),
      nodeIntegration: false,
      contextIsolation: true,
      webSecurity: false,
    }
  });

  const isDev = process.env.NODE_ENV !== 'production' && !app.isPackaged && process.env.VITE_DEV_SERVER_URL;
  const indexHtml = path.join(app.getAppPath(), 'dist/index.html');
  if (isDev) {
    mainWindow.loadURL(process.env.VITE_DEV_SERVER_URL);
  } else {
    mainWindow.loadFile(indexHtml);
  }

  mainWindow.webContents.on('did-fail-load', (e, code, desc, url) => {
    console.error('[RiftCore] mainWindow did-fail-load:', code, desc, url);
  });

  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    if (url.startsWith('http://') || url.startsWith('https://')) {
      shell.openExternal(url);
    }
    return { action: 'deny' };
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
    if (overlayWindow && !overlayWindow.isDestroyed()) {
      overlayWindow.close();
    }
    if (liveGameInterval) clearInterval(liveGameInterval);
    if (lcuWebSocket) lcuWebSocket.close();
  });
}

function createOverlayWindow() {
  if (overlayWindow && !overlayWindow.isDestroyed()) {
    return overlayWindow;
  }

  const primaryDisplay = screen.getPrimaryDisplay();
  const { width, height } = primaryDisplay.bounds;

  overlayWindow = new BrowserWindow({
    x: 0,
    y: 0,
    width,
    height,
    frame: false,
    transparent: true,
    backgroundColor: '#00000000',
    alwaysOnTop: true,
    skipTaskbar: true,
    focusable: false,
    hasShadow: false,
    show: false,
    webPreferences: {
      preload: path.join(app.getAppPath(), 'electron/preload.js'),
      nodeIntegration: false,
      contextIsolation: true,
      webSecurity: false,
    }
  });

  overlayWindow.setAlwaysOnTop(true, 'screen-saver', 1);
  if (overlayClickThrough) {
    overlayWindow.setIgnoreMouseEvents(true, { forward: true });
  } else {
    overlayWindow.setIgnoreMouseEvents(false);
  }

  const isDev = process.env.NODE_ENV !== 'production' && !app.isPackaged && process.env.VITE_DEV_SERVER_URL;
  const indexHtml = path.join(app.getAppPath(), 'dist/index.html');
  if (isDev) {
    overlayWindow.loadURL(`${process.env.VITE_DEV_SERVER_URL}#overlay`);
  } else {
    overlayWindow.loadFile(indexHtml, { hash: 'overlay' });
  }

  overlayWindow.on('closed', () => {
    overlayWindow = null;
  });

  return overlayWindow;
}

function showOverlay() {
  const win = overlayWindow || createOverlayWindow();
  if (win && !win.isDestroyed()) {
    const primaryDisplay = screen.getPrimaryDisplay();
    const { x, y, width, height } = primaryDisplay.bounds;
    win.setBounds({ x, y, width, height });
    win.setAlwaysOnTop(true, 'screen-saver', 1);
    win.moveTop();
    try {
      win.setVisibleOnAllWorkspaces(true, { visibleOnFullScreen: true });
    } catch {}
    if (overlayClickThrough && !isLayoutEditMode) {
      win.setIgnoreMouseEvents(true, { forward: true });
    } else {
      win.setIgnoreMouseEvents(false);
    }
    win.showInactive();
    startMouseWatcher();
    if (isLeagueInForeground) {
      startTopmostWatcher();
    }
  }
}

function hideOverlay() {
  stopMouseWatcher();
  stopTopmostWatcher();
  if (isOverlayOnly) {
    app.quit();
    return;
  }
  if (overlayWindow && !overlayWindow.isDestroyed()) {
    overlayWindow.hide();
  }
}

function notifyAllRenderers(channel, payload) {
  if (mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.webContents.send(channel, payload);
  }
  if (overlayWindow && !overlayWindow.isDestroyed()) {
    overlayWindow.webContents.send(channel, payload);
  }
}

// -------------------------------------------------------------
// LCU DETECTION & WEBSOCKET ENGINE
// -------------------------------------------------------------

function getAuthHeader(credentials) {
  if (!credentials) return '';
  const token = Buffer.from(`riot:${credentials.password}`).toString('base64');
  return `Basic ${token}`;
}

async function requestLCU(endpoint, method = 'GET', body = null) {
  if (!lcuCredentials) {
    throw new Error('League Client is not running');
  }

  const url = `https://127.0.0.1:${lcuCredentials.port}${endpoint}`;
  const options = {
    method,
    headers: {
      'Authorization': getAuthHeader(lcuCredentials),
      'Accept': 'application/json',
      'Content-Type': 'application/json',
    },
    agent: httpsAgent,
  };

  if (body) {
    options.body = typeof body === 'string' ? body : JSON.stringify(body);
  }

  const response = await fetch(url, options);
  if (!response.ok && response.status !== 204) {
    const text = await response.text();
    let parsed;
    try { parsed = JSON.parse(text); } catch { parsed = { message: text }; }
    throw new Error(parsed.message || `LCU HTTP ${response.status}`);
  }

  if (response.status === 204) return { success: true };
  return await response.json();
}

const candidateLockfiles = [
  'E:\\Riot Games\\League of Legends\\lockfile',
  'F:\\Riot Games\\League of Legends (PBE)\\lockfile',
  'C:\\Riot Games\\League of Legends\\lockfile',
  'D:\\Riot Games\\League of Legends\\lockfile',
  'F:\\Riot Games\\League of Legends\\lockfile',
];

function isPidAlive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (e) {
    return false;
  }
}

function detectLCUProcess() {
  return new Promise((resolve) => {
    // 1. Instantaneous direct lockfile inspection (0ms)
    for (const lf of candidateLockfiles) {
      if (fs.existsSync(lf)) {
        try {
          const content = fs.readFileSync(lf, 'utf8').trim();
          const parts = content.split(':');
          if (parts.length >= 5) {
            const pid = parseInt(parts[1], 10);
            if (isPidAlive(pid)) {
              return resolve({
                port: parts[2],
                password: parts[3],
                installDir: path.dirname(lf),
                method: 'lockfile'
              });
            }
          }
        } catch {}
      }
    }

    // 2. Fallback to WMI process command line
    const cmd = `powershell -Command "Get-CimInstance Win32_Process -Filter \\"name = 'LeagueClientUx.exe'\\" | Select-Object -ExpandProperty CommandLine"`;
    exec(cmd, (error, stdout) => {
      if (error || !stdout || stdout.trim() === '') {
        return resolve(null);
      }

      const portMatch = stdout.match(/--app-port=([0-9]+)/);
      const tokenMatch = stdout.match(/--remoting-auth-token=([^ "]+)/);
      const installDirMatch = stdout.match(/--install-directory=([^ "]+)/);

      if (portMatch && tokenMatch) {
        resolve({
          port: portMatch[1],
          password: tokenMatch[1],
          installDir: installDirMatch ? installDirMatch[1] : null,
          method: 'wmi'
        });
      } else {
        resolve(null);
      }
    });
  });
}

function connectWebSocket(credentials) {
  if (lcuWebSocket) {
    try { lcuWebSocket.close(); } catch {}
  }

  const wsUrl = `wss://127.0.0.1:${credentials.port}/`;
  const ws = new WebSocket(wsUrl, {
    headers: {
      Authorization: getAuthHeader(credentials),
    },
    rejectUnauthorized: false
  });

  ws.on('open', () => {
    console.log('[RiftCore] LCU WebSocket Connected');
    ws.send(JSON.stringify([5, 'OnJsonApiEvent']));
    notifyAllRenderers('lcu-status', { connected: true, credentials });
  });

  ws.on('message', (raw) => {
    try {
      if (!raw || raw.length === 0) return;
      const parsed = JSON.parse(raw.toString());
      if (Array.isArray(parsed) && parsed.length >= 3) {
        const [eventCode, eventName, payload] = parsed;
        if (eventCode === 8 && eventName === 'OnJsonApiEvent') {
          handleLCUEvent(payload);
        }
      }
    } catch {}
  });

  ws.on('error', (err) => {
    console.warn('[RiftCore] LCU WebSocket error:', err.message);
  });

  ws.on('close', () => {
    console.log('[RiftCore] LCU WebSocket Closed');
    notifyAllRenderers('lcu-status', { connected: false });
    champSelectSession = null;
    notifyAllRenderers('champ-select-session', null);
    lcuCredentials = null;
    lcuWebSocket = null;
  });

  lcuWebSocket = ws;
}

function handleLCUEvent(event) {
  if (!event || !event.uri) return;
  const { uri, eventType, data } = event;

  // 1. Gameflow Phase
  if (uri === '/lol-gameflow/v1/gameflow-phase') {
    gameflowPhase = data;
    notifyAllRenderers('gameflow-phase', data);

    if (data === 'ChampSelect') {
      requestLCU('/lol-champ-select/v1/session')
        .then((sess) => {
          if (sess && !sess.errorCode) {
            champSelectSession = sess;
            notifyAllRenderers('champ-select-session', sess);
          }
        })
        .catch(() => {});
    } else {
      champSelectSession = null;
      notifyAllRenderers('champ-select-session', null);
    }

    if (data === 'InProgress' || data === 'GameStart') {
      showOverlay();
      startLiveGamePolling();
    } else {
      hideOverlay();
      stopLiveGamePolling();
    }
  }

  // 2. Ready Check (Queue Pop)
  if (uri === '/lol-matchmaking/v1/ready-check') {
    notifyAllRenderers('ready-check', data);
    if (appSettings.autoAccept && data && data.state === 'InProgress' && data.playerResponse === 'None') {
      console.log('[RiftCore] Auto-accepting queue match!');
      requestLCU('/lol-matchmaking/v1/ready-check/accept', 'POST')
        .catch(err => console.error('Auto accept error:', err));
    }
  }

  // 3. Champ Select Session
  if (uri === '/lol-champ-select/v1/session') {
    if (eventType === 'Delete') {
      champSelectSession = null;
      notifyAllRenderers('champ-select-session', null);
    } else {
      champSelectSession = data;
      notifyAllRenderers('champ-select-session', data);
    }
  }
}

// Live In-Game polling (Riot Live Client Data API on port 2999)
function startLiveGamePolling() {
  if (liveGameInterval) clearInterval(liveGameInterval);
  liveGameInterval = setInterval(async () => {
    try {
      const res = await fetch('https://127.0.0.1:2999/liveclientdata/allgamedata', {
        agent: httpsAgent,
        signal: AbortSignal.timeout(1200)
      });
      if (res.ok) {
        const liveData = await res.json();
        lastLiveData = liveData;
        notifyAllRenderers('live-game-update', liveData);
        if (overlayWindow && !overlayWindow.isVisible() && isLeagueInForeground) {
          showOverlay();
        }
      }
    } catch {}
  }, 1000);
}

function stopLiveGamePolling() {
  if (liveGameInterval) {
    clearInterval(liveGameInterval);
    liveGameInterval = null;
  }
  lastLiveData = null;
}

// Background poller to discover League client
async function pollForClient() {
  if (!lcuCredentials) {
    const creds = await detectLCUProcess();
    if (creds) {
      console.log('[RiftCore] Detected League Client on port:', creds.port);
      lcuCredentials = creds;
      connectWebSocket(creds);

      try {
        const phase = await requestLCU('/lol-gameflow/v1/gameflow-phase');
        gameflowPhase = phase;
        notifyAllRenderers('gameflow-phase', phase);
      } catch {}
    }
  }
}

setInterval(pollForClient, 3000);

// -------------------------------------------------------------
// IPC HANDLERS
// -------------------------------------------------------------

ipcMain.handle('check-client-status', async () => {
  if (!lcuCredentials) {
    await pollForClient();
  }
  return {
    connected: !!lcuCredentials,
    gameflowPhase,
    credentials: lcuCredentials ? { port: lcuCredentials.port } : null
  };
});

ipcMain.handle('lcu-request', async (_event, { endpoint, method, body }) => {
  if (!lcuCredentials) return null;
  try {
    return await requestLCU(endpoint, method, body);
  } catch {
    return null;
  }
});

ipcMain.handle('import-runes', async (_event, pageData) => {
  if (!lcuCredentials) return { success: false, message: 'League Client not running' };

  const pages = await requestLCU('/lol-perks/v1/pages');
  let targetPage = pages.find(p => p.name && (p.name.includes('Rift') || p.name.includes('RiftCore')));
  if (!targetPage) {
    targetPage = pages.find(p => p.isDeletable);
  }
  if (!targetPage) {
    targetPage = pages.find(p => p.isEditable);
  }

  const rawName = pageData.name || 'RiftCore Build';
  const cleanName = rawName.replace(/[^a-zA-Z0-9 ]/g, '').trim() || 'RiftCore Build';

  const payload = {
    name: cleanName,
    primaryStyleId: pageData.primaryStyleId,
    subStyleId: pageData.subStyleId,
    selectedPerkIds: pageData.selectedPerkIds,
    current: true,
  };

  if (targetPage) {
    try {
      const updated = await requestLCU(`/lol-perks/v1/pages/${targetPage.id}`, 'PUT', { ...payload, id: targetPage.id });
      try { await requestLCU('/lol-perks/v1/currentpage', 'PUT', targetPage.id); } catch {}
      return updated;
    } catch {
      if (targetPage.isDeletable) {
        try { await requestLCU(`/lol-perks/v1/pages/${targetPage.id}`, 'DELETE'); } catch {}
      }
      const created = await requestLCU('/lol-perks/v1/pages', 'POST', payload);
      if (created && created.id) {
        try { await requestLCU('/lol-perks/v1/currentpage', 'PUT', created.id); } catch {}
      }
      return created;
    }
  } else {
    const created = await requestLCU('/lol-perks/v1/pages', 'POST', payload);
    if (created && created.id) {
      try { await requestLCU('/lol-perks/v1/currentpage', 'PUT', created.id); } catch {}
    }
    return created;
  }
});

ipcMain.handle('import-spells', async (_event, { spell1Id, spell2Id }) => {
  if (!lcuCredentials) return { success: false, message: 'League Client not running' };
  return await requestLCU('/lol-champ-select/v1/session/my-selection', 'PATCH', {
    spell1Id,
    spell2Id,
  });
});

ipcMain.handle('export-item-set', async (_event, { itemSet, championKey }) => {
  if (!lcuCredentials) return { success: false, message: 'League Client not running' };

  const itemSetPayload = {
    title: itemSet.title || 'RiftCore Build',
    associatedChampions: championKey ? [parseInt(championKey, 10)] : [],
    associatedMaps: [11, 12],
    blocks: itemSet.blocks || [],
    sortorder: 0,
    type: 'custom',
  };

  const summoner = await requestLCU('/lol-summoner/v1/current-summoner');
  if (!summoner) throw new Error('Summoner not found');
  const accountId = summoner.accountId || summoner.summonerId;
  const summonerId = summoner.summonerId || summoner.accountId;

  let setsData = { itemSets: [], timestamp: Date.now() };
  try {
    const existing = await requestLCU(`/lol-item-sets/v1/item-sets/${accountId}/sets`);
    if (existing && Array.isArray(existing.itemSets)) {
      setsData = existing;
    }
  } catch {
    try {
      const existing2 = await requestLCU(`/lol-item-sets/v1/item-sets/${summonerId}/sets`);
      if (existing2 && Array.isArray(existing2.itemSets)) {
        setsData = existing2;
      }
    } catch {}
  }

  const champId = itemSetPayload.associatedChampions?.[0];
  setsData.itemSets = setsData.itemSets.filter(s => {
    if (s.title && s.title.includes('RiftCore')) {
      if (champId && Array.isArray(s.associatedChampions) && s.associatedChampions.includes(champId)) {
        return false;
      }
    }
    return true;
  });

  setsData.itemSets.push(itemSetPayload);
  try {
    return await requestLCU(`/lol-item-sets/v1/item-sets/${accountId}/sets`, 'PUT', setsData);
  } catch {
    return await requestLCU(`/lol-item-sets/v1/item-sets/${summonerId}/sets`, 'PUT', setsData);
  }
});

ipcMain.handle('accept-match', async () => {
  if (!lcuCredentials) throw new Error('League Client not running');
  return await requestLCU('/lol-matchmaking/v1/ready-check/accept', 'POST');
});

ipcMain.handle('get-live-client-data', async () => {
  try {
    const res = await fetch('https://127.0.0.1:2999/liveclientdata/allgamedata', {
      agent: httpsAgent,
      signal: AbortSignal.timeout(1000)
    });
    if (res.ok) return await res.json();
  } catch {}
  return lastLiveData;
});

ipcMain.handle('get-settings', () => {
  return appSettings;
});

ipcMain.handle('save-settings', (_event, newSettings) => {
  saveSettings(newSettings);
  return appSettings;
});

ipcMain.handle('riot-api-request', async (_event, { url, key }) => {
  try {
    const headers = key ? { 'X-Riot-Token': key } : {};
    const res = await fetch(url, { headers });
    const data = await res.json().catch(() => ({}));
    return { status: res.status, data };
  } catch (err) {
    return { status: 500, data: { error: err.message } };
  }
});

// Window controls & Overlay Mode
ipcMain.handle('window-control', (_event, action) => {
  if (!mainWindow) return;

  switch (action) {
    case 'minimize':
      mainWindow.minimize();
      break;
    case 'maximize':
      if (mainWindow.isMaximized()) {
        mainWindow.unmaximize();
      } else {
        mainWindow.maximize();
      }
      break;
    case 'close':
      if (appSettings.closeToTray) {
        mainWindow.hide();
      } else {
        mainWindow.close();
      }
      break;
    case 'minimize-to-tray':
      mainWindow.hide();
      break;
    case 'exit-app':
      app.quit();
      break;
    case 'set-pin-on':
      isPinned = true;
      mainWindow.setAlwaysOnTop(true, 'screen-saver');
      return { isPinned, isCompactMode };
    case 'set-pin-off':
      isPinned = false;
      mainWindow.setAlwaysOnTop(false);
      return { isPinned, isCompactMode };
    case 'toggle-pin':
      isPinned = !isPinned;
      mainWindow.setAlwaysOnTop(isPinned, 'screen-saver');
      return { isPinned, isCompactMode };
    case 'toggle-compact':
      isCompactMode = !isCompactMode;
      if (isCompactMode) {
        showOverlay();
      } else {
        hideOverlay();
        if (mainWindow) {
          mainWindow.show();
          mainWindow.focus();
        }
      }
      notifyAllRenderers('rift-compact-hud', isCompactMode);
      return { isPinned, isCompactMode };
  }
});

ipcMain.handle('set-click-through', (_event, enabled) => {
  overlayClickThrough = Boolean(enabled);
  if (overlayWindow && !overlayWindow.isDestroyed()) {
    if (overlayClickThrough) {
      overlayWindow.setIgnoreMouseEvents(true, { forward: true });
    } else {
      overlayWindow.setIgnoreMouseEvents(false);
    }
  }
  return { clickThrough: overlayClickThrough };
});

ipcMain.on('set-ignore-mouse-events', (event, ignore, options) => {
  const win = BrowserWindow.fromWebContents(event.sender);
  if (win && !win.isDestroyed()) {
    if (ignore) {
      win.setIgnoreMouseEvents(true, { forward: true });
    } else {
      win.setIgnoreMouseEvents(false);
    }
  }
});

ipcMain.on('update-interactive-regions', (_event, regions) => {
  interactiveRegions = Array.isArray(regions) ? regions : [];
});

ipcMain.handle('set-layout-edit-mode', (_event, editMode) => {
  isLayoutEditMode = Boolean(editMode);
  if (overlayWindow && !overlayWindow.isDestroyed()) {
    if (isLayoutEditMode) {
      isCurrentlyInteractive = true;
      overlayWindow.setIgnoreMouseEvents(false);
    } else {
      isCurrentlyInteractive = false;
      overlayWindow.setIgnoreMouseEvents(true, { forward: true });
    }
  }
  return { editMode: isLayoutEditMode };
});

ipcMain.handle('set-hud-mode', (_event, mode) => {
  currentHudMode = mode;
  if (!overlayWindow || overlayWindow.isDestroyed()) return { mode };

  const primaryDisplay = screen.getPrimaryDisplay();
  const { width: sw, height: sh } = primaryDisplay.bounds;

  if (mode === 'integrated') {
    overlayWindow.setBounds({ x: 0, y: 0, width: sw, height: sh });
    overlayWindow.setIgnoreMouseEvents(overlayClickThrough, { forward: true });
  } else if (mode === 'card') {
    overlayWindow.setIgnoreMouseEvents(false);
    overlayWindow.setBounds({ x: Math.max(0, sw - 340 - 24), y: 60, width: 340, height: 270 });
  } else if (mode === 'bar') {
    overlayWindow.setIgnoreMouseEvents(false);
    overlayWindow.setBounds({ x: Math.max(0, Math.floor((sw - 620) / 2)), y: 12, width: 620, height: 42 });
  } else if (mode === 'pill') {
    overlayWindow.setIgnoreMouseEvents(false);
    overlayWindow.setBounds({ x: Math.max(0, sw - 280 - 24), y: 60, width: 280, height: 38 });
  }
  return { mode };
});

ipcMain.handle('set-always-on-top', (_event, alwaysOnTop) => {
  isPinned = Boolean(alwaysOnTop);
  if (mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.setAlwaysOnTop(isPinned, 'screen-saver');
  }
  return { isPinned };
});

ipcMain.handle('set-compact-hud', (_event, compact) => {
  isCompactMode = Boolean(compact);
  if (isCompactMode) {
    showOverlay();
  } else {
    hideOverlay();
    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.show();
      mainWindow.focus();
    }
  }
  notifyAllRenderers('rift-compact-hud', isCompactMode);
  return { isPinned, isCompactMode };
});

ipcMain.handle('focus-window', () => {
  if (mainWindow && !mainWindow.isDestroyed()) {
    if (mainWindow.isMinimized()) mainWindow.restore();
    mainWindow.show();
    mainWindow.focus();
    return { success: true };
  }
  return { success: false };
});

ipcMain.handle('launch-league', async () => {
  const riotClientPaths = [
    'C:\\Riot Games\\Riot Client\\RiotClientServices.exe',
    'D:\\Riot Games\\Riot Client\\RiotClientServices.exe',
    'E:\\Riot Games\\Riot Client\\RiotClientServices.exe',
    'F:\\Riot Games\\Riot Client\\RiotClientServices.exe',
  ];

  for (const exe of riotClientPaths) {
    if (fs.existsSync(exe)) {
      exec(`"${exe}" --launch-product=league_of_legends --launch-patchline=live`);
      return { success: true };
    }
  }
  return { success: false, message: 'RiotClientServices.exe not found' };
});

ipcMain.handle('get-league-window-mode', async () => {
  return parseLeagueGameCfg();
});

ipcMain.handle('set-league-borderless', async () => {
  const cfgPath = findLeagueGameCfg();
  if (!cfgPath) return { success: false };

  try {
    let content = fs.readFileSync(cfgPath, 'utf8');
    if (content.includes('WindowMode=')) {
      content = content.replace(/WindowMode=\d+/g, 'WindowMode=2');
    } else {
      content += '\r\nWindowMode=2\r\n';
    }
    fs.writeFileSync(cfgPath, content, 'utf8');
    return { success: true };
  } catch {
    return { success: false };
  }
});

ipcMain.handle('set-league-fullscreen', async () => {
  const cfgPath = findLeagueGameCfg();
  if (!cfgPath) return { success: false };

  try {
    let content = fs.readFileSync(cfgPath, 'utf8');
    if (content.includes('WindowMode=')) {
      content = content.replace(/WindowMode=\d+/g, 'WindowMode=0');
    } else {
      content += '\r\nWindowMode=0\r\n';
    }
    fs.writeFileSync(cfgPath, content, 'utf8');
    return { success: true };
  } catch {
    return { success: false };
  }
});

ipcMain.handle('check-app-update', async () => {
  try {
    const res = await fetch('https://api.github.com/repos/kotsioskl2/riftcore-companion/releases/latest', {
      headers: { 'User-Agent': 'RiftCore-App' }
    });
    if (!res.ok) return { hasUpdate: false };
    const data = await res.json();
    const latestVersion = (data.tag_name || '').replace(/^v/, '');
    const currentVersion = app.getVersion();
    const hasUpdate = Boolean(latestVersion && latestVersion !== currentVersion);
    return {
      hasUpdate,
      currentVersion,
      latestVersion,
      releaseNotes: data.body || '',
      downloadUrl: data.html_url || ''
    };
  } catch {
    return { hasUpdate: false };
  }
});

ipcMain.handle('download-app-update', async (_event, { downloadUrl }) => {
  if (downloadUrl) shell.openExternal(downloadUrl);
  return { success: true };
});

// -------------------------------------------------------------
// APP LIFECYCLE & GLOBAL SHORTCUTS
// -------------------------------------------------------------

function toggleOverlayHotkey() {
  if (overlayWindow && overlayWindow.isVisible()) {
    hideOverlay();
    if (!isOverlayOnly && mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.show();
      mainWindow.focus();
    }
  } else {
    showOverlay();
  }
}

app.whenReady().then(() => {
  startFocusWatcher();

  if (isOverlayOnly) {
    createOverlayWindow();
    showOverlay();
    pollForClient();
    startLiveGamePolling();

    globalShortcut.register('CommandOrControl+Space', toggleOverlayHotkey);
    globalShortcut.register('Alt+X', toggleOverlayHotkey);
    globalShortcut.register('Alt+G', toggleOverlayHotkey);
    globalShortcut.register('F10', toggleOverlayHotkey);
    return;
  }

  createMainWindow();
  createOverlayWindow();
  pollForClient();

  // Register Global Hotkeys
  globalShortcut.register('CommandOrControl+Space', toggleOverlayHotkey);
  globalShortcut.register('Alt+X', toggleOverlayHotkey);
  globalShortcut.register('Alt+G', toggleOverlayHotkey);
  globalShortcut.register('F10', toggleOverlayHotkey);

  // System Tray
  try {
    const iconPath = path.join(__dirname, '../public/icon.png');
    if (fs.existsSync(iconPath)) {
      tray = new Tray(iconPath);
      const contextMenu = Menu.buildFromTemplate([
        { label: 'RiftCore Companion', enabled: false },
        { type: 'separator' },
        {
          label: 'Open Dashboard',
          click: () => {
            if (mainWindow) {
              mainWindow.show();
              mainWindow.focus();
            }
          }
        },
        {
          label: 'Toggle In-Game HUD',
          click: toggleOverlayHotkey
        },
        { type: 'separator' },
        {
          label: 'Exit RiftCore',
          click: () => app.quit()
        }
      ]);
      tray.setToolTip('RiftCore Companion');
      tray.setContextMenu(contextMenu);
      tray.on('click', () => {
        if (mainWindow) {
          mainWindow.show();
          mainWindow.focus();
        }
      });
    }
  } catch (e) {
    console.warn('[RiftCore] Tray init warning:', e);
  }

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createMainWindow();
  });
});

app.on('will-quit', () => {
  stopFocusWatcher();
  stopMouseWatcher();
  globalShortcut.unregisterAll();
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});
