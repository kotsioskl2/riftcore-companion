const { app, BrowserWindow, ipcMain, shell, globalShortcut } = require('electron');
const path = require('path');
const fs = require('fs');
const https = require('https');
const { exec } = require('child_process');
const WebSocket = require('ws');

// Disable TLS verification for local self-signed Riot certificates
process.env["NODE_TLS_REJECT_UNAUTHORIZED"] = "0";

const httpsAgent = new https.Agent({ rejectUnauthorized: false });

let mainWindow = null;
let isCompactMode = false;
let isPinned = false;

// LCU State
let lcuCredentials = null;
let lcuWebSocket = null;
let gameflowPhase = 'None';
let champSelectSession = null;
let liveGameInterval = null;

// Settings path
const settingsPath = path.join(app.getPath('userData'), 'hexcore_settings.json');
let appSettings = {
  autoAccept: true,
  autoImportRunes: false,
  autoImportSpells: false,
  soundAlerts: true,
  theme: 'hextech',
  alwaysOnTop: false,
  closeToTray: true,
};

function loadSettings() {
  try {
    if (fs.existsSync(settingsPath)) {
      const data = fs.readFileSync(settingsPath, 'utf8');
      appSettings = { ...appSettings, ...JSON.parse(data) };
    }
  } catch (err) {
    console.error('Error loading settings:', err);
  }
}

function saveSettings(newSettings) {
  try {
    appSettings = { ...appSettings, ...newSettings };
    fs.writeFileSync(settingsPath, JSON.stringify(appSettings, null, 2));
  } catch (err) {
    console.error('Error saving settings:', err);
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

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1240,
    height: 820,
    minWidth: 340,
    minHeight: 240,
    frame: false,
    transparent: true,
    backgroundColor: '#00000000',
    hasShadow: true,
    icon: path.join(__dirname, '../public/icon.png'),
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      nodeIntegration: false,
      contextIsolation: true,
      webSecurity: false // Allows accessing local Riot assets
    }
  });

  const isDev = process.env.NODE_ENV !== 'production' && !app.isPackaged;
  if (isDev) {
    mainWindow.loadURL('http://localhost:5173');
  } else {
    mainWindow.loadFile(path.join(__dirname, '../dist/index.html'));
  }

  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    if (url.startsWith('http://') || url.startsWith('https://')) {
      shell.openExternal(url);
    }
    return { action: 'deny' };
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
    if (liveGameInterval) clearInterval(liveGameInterval);
    if (lcuWebSocket) lcuWebSocket.close();
  });
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

function detectLCUProcess() {
  return new Promise((resolve) => {
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
    notifyRenderer('lcu-status', { connected: true, credentials });
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
    notifyRenderer('lcu-status', { connected: false });
    champSelectSession = null;
    notifyRenderer('champ-select-session', null);
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
    notifyRenderer('gameflow-phase', data);

    if (data === 'ChampSelect') {
      requestLCU('/lol-champ-select/v1/session')
        .then((sess) => {
          if (sess && !sess.errorCode) {
            champSelectSession = sess;
            notifyRenderer('champ-select-session', sess);
          }
        })
        .catch(() => {});
    } else {
      champSelectSession = null;
      notifyRenderer('champ-select-session', null);
    }

    if (data === 'InProgress' || data === 'GameStart') {
      startLiveGamePolling();
    } else {
      stopLiveGamePolling();
    }
  }

  // 2. Ready Check (Queue Pop)
  if (uri === '/lol-matchmaking/v1/ready-check') {
    notifyRenderer('ready-check', data);
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
      notifyRenderer('champ-select-session', null);
    } else {
      champSelectSession = data;
      notifyRenderer('champ-select-session', data);
    }
  }
}

function notifyRenderer(channel, payload) {
  if (mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.webContents.send(channel, payload);
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
        notifyRenderer('live-game-update', liveData);
      }
    } catch {}
  }, 1000);
}

function stopLiveGamePolling() {
  if (liveGameInterval) {
    clearInterval(liveGameInterval);
    liveGameInterval = null;
  }
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
        notifyRenderer('gameflow-phase', phase);
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
  return await requestLCU(endpoint, method, body);
});

ipcMain.handle('import-runes', async (_event, pageData) => {
  if (!lcuCredentials) throw new Error('League Client not running');

  const pages = await requestLCU('/lol-perks/v1/pages');
  let targetPage = pages.find(p => p.isEditable && p.isDeletable);

  if (!targetPage) {
    targetPage = pages.find(p => p.isEditable);
  }

  const payload = {
    name: pageData.name || '⚡ RiftCore Build',
    primaryStyleId: pageData.primaryStyleId,
    subStyleId: pageData.subStyleId,
    selectedPerkIds: pageData.selectedPerkIds,
    current: true,
    isActive: true
  };

  if (targetPage) {
    try {
      await requestLCU(`/lol-perks/v1/pages/${targetPage.id}`, 'DELETE');
      return await requestLCU('/lol-perks/v1/pages', 'POST', payload);
    } catch {
      return await requestLCU(`/lol-perks/v1/pages/${targetPage.id}`, 'PUT', payload);
    }
  } else {
    return await requestLCU('/lol-perks/v1/pages', 'POST', payload);
  }
});

ipcMain.handle('import-spells', async (_event, { spell1Id, spell2Id }) => {
  if (!lcuCredentials) throw new Error('League Client not running');
  return await requestLCU('/lol-champ-select/v1/session/my-selection', 'PATCH', {
    spell1Id,
    spell2Id
  });
});

ipcMain.handle('export-item-set', async (_event, itemSetPayload) => {
  if (!lcuCredentials) throw new Error('League Client not running');

  const summoner = await requestLCU('/lol-summoner/v1/current-summoner');
  if (!summoner || !summoner.summonerId) throw new Error('Summoner not found');
  const summonerId = summoner.summonerId;

  let setsData = { itemSets: [], timestamp: Date.now() };
  try {
    const existing = await requestLCU(`/lol-item-sets/v1/item-sets/${summonerId}/sets`);
    if (existing && Array.isArray(existing.itemSets)) {
      setsData = existing;
    }
  } catch {}

  const champId = itemSetPayload.associatedChampions?.[0];
  setsData.itemSets = setsData.itemSets.filter(s => {
    if (s.title && s.title.startsWith('⚡ RiftCore:')) {
      if (champId && Array.isArray(s.associatedChampions) && s.associatedChampions.includes(champId)) {
        return false;
      }
    }
    return true;
  });

  setsData.itemSets.push(itemSetPayload);
  return await requestLCU(`/lol-item-sets/v1/item-sets/${summonerId}/sets`, 'PUT', setsData);
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
  return null;
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
        mainWindow.minimize();
      } else {
        mainWindow.close();
      }
      break;
    case 'minimize-to-tray':
      mainWindow.minimize();
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
        mainWindow.setMinimumSize(240, 36);
        mainWindow.setSize(340, 270);
        mainWindow.setAlwaysOnTop(true, 'screen-saver');
        isPinned = true;
      } else {
        mainWindow.setMinimumSize(900, 640);
        mainWindow.setSize(1240, 820);
        mainWindow.setAlwaysOnTop(isPinned, 'screen-saver');
        mainWindow.center();
      }
      notifyRenderer('rift-compact-hud', isCompactMode);
      return { isPinned, isCompactMode };
  }
});

ipcMain.handle('resize-window', (_event, { width, height }) => {
  if (!mainWindow) return;
  mainWindow.setSize(width, height);
  return { width, height };
});

ipcMain.handle('snap-overlay', (_event, { position, width = 340, height = 270 }) => {
  if (!mainWindow) return;
  const { screen } = require('electron');
  const primaryDisplay = screen.getPrimaryDisplay();
  const { width: sw } = primaryDisplay.workAreaSize;
  let x = Math.max(0, sw - width - 24);
  let y = 60;
  if (position === 'top-center') {
    x = Math.max(0, Math.floor((sw - width) / 2));
    y = 12;
  } else if (position === 'top-left') {
    x = 24;
    y = 60;
  }
  mainWindow.setBounds({ x, y, width, height });
  return { x, y, width, height };
});

ipcMain.handle('set-click-through', (_event, enabled) => {
  if (!mainWindow) return { clickThrough: false };
  mainWindow.setIgnoreMouseEvents(Boolean(enabled), { forward: true });
  return { clickThrough: Boolean(enabled) };
});

ipcMain.handle('set-hud-mode', (_event, mode) => {
  if (!mainWindow) return { mode };
  const { screen } = require('electron');
  const primaryDisplay = screen.getPrimaryDisplay();
  const { width: sw, height: sh } = primaryDisplay.bounds;

  if (mode === 'integrated') {
    mainWindow.setBounds({ x: 0, y: 0, width: sw, height: sh });
    mainWindow.setIgnoreMouseEvents(true, { forward: true });
    mainWindow.setAlwaysOnTop(true, 'screen-saver');
  } else if (mode === 'card') {
    mainWindow.setIgnoreMouseEvents(false);
    mainWindow.setBounds({ x: Math.max(0, sw - 340 - 24), y: 60, width: 340, height: 270 });
    mainWindow.setAlwaysOnTop(true, 'screen-saver');
  } else if (mode === 'bar') {
    mainWindow.setIgnoreMouseEvents(false);
    mainWindow.setBounds({ x: Math.max(0, Math.floor((sw - 620) / 2)), y: 12, width: 620, height: 42 });
    mainWindow.setAlwaysOnTop(true, 'screen-saver');
  }
  return { mode };
});

ipcMain.handle('set-always-on-top', (_event, alwaysOnTop) => {
  if (!mainWindow) return { isPinned: false };
  isPinned = Boolean(alwaysOnTop);
  mainWindow.setAlwaysOnTop(isPinned, 'screen-saver');
  return { isPinned };
});

ipcMain.handle('set-compact-hud', (_event, compact) => {
  if (!mainWindow) return { isCompactMode, isPinned };
  isCompactMode = Boolean(compact);
  if (isCompactMode) {
    mainWindow.setMinimumSize(240, 36);
    mainWindow.setSize(340, 270);
    mainWindow.setAlwaysOnTop(true, 'screen-saver');
    isPinned = true;
  } else {
    mainWindow.setMinimumSize(900, 640);
    mainWindow.setSize(1240, 820);
    mainWindow.setAlwaysOnTop(isPinned, 'screen-saver');
    mainWindow.center();
  }
  notifyRenderer('rift-compact-hud', isCompactMode);
  return { isPinned, isCompactMode };
});

ipcMain.handle('focus-window', () => {
  if (mainWindow) {
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
  const cfgPath = findLeagueGameCfg();
  if (!cfgPath) return { found: false, windowMode: 0, path: '' };

  try {
    const content = fs.readFileSync(cfgPath, 'utf8');
    let windowMode = 0;
    for (const line of content.split('\n')) {
      const trimmed = line.trim();
      if (trimmed.startsWith('WindowMode=')) {
        windowMode = parseInt(trimmed.replace('WindowMode=', ''), 10) || 0;
        break;
      }
    }
    return { found: true, windowMode, path: cfgPath };
  } catch {
    return { found: false, windowMode: 0, path: cfgPath };
  }
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

function toggleOverlayAction() {
  if (!mainWindow) return;

  if (mainWindow.isMinimized() || !mainWindow.isVisible()) {
    mainWindow.restore();
    mainWindow.show();
    mainWindow.focus();
    if (!isCompactMode) {
      isCompactMode = true;
      mainWindow.setMinimumSize(340, 240);
      mainWindow.setSize(380, 680);
      mainWindow.setAlwaysOnTop(true, 'screen-saver');
      isPinned = true;
      notifyRenderer('rift-compact-hud', true);
    }
    return;
  }

  // If already visible in compact mode: toggle minimize
  if (isCompactMode) {
    mainWindow.minimize();
  } else {
    // Switch to compact overlay mode
    isCompactMode = true;
    mainWindow.setMinimumSize(340, 240);
    mainWindow.setSize(380, 680);
    mainWindow.setAlwaysOnTop(true, 'screen-saver');
    isPinned = true;
    notifyRenderer('rift-compact-hud', true);
  }
}

// App Lifecycle
app.whenReady().then(() => {
  createWindow();
  pollForClient();

  // Register Global Overlay Shortcuts
  const registerShortcut = (accelerator, handler) => {
    try {
      globalShortcut.register(accelerator, handler);
    } catch (e) {
      console.warn(`Failed to register global shortcut ${accelerator}:`, e);
    }
  };

  registerShortcut('Alt+G', toggleOverlayAction);
  registerShortcut('CommandOrControl+Space', toggleOverlayAction);
  registerShortcut('Alt+X', toggleOverlayAction);

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('will-quit', () => {
  globalShortcut.unregisterAll();
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});

