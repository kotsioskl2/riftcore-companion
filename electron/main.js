const { app, BrowserWindow, ipcMain, shell } = require('electron');
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

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1240,
    height: 820,
    minWidth: 900,
    minHeight: 640,
    frame: false,
    transparent: false,
    backgroundColor: '#070b12',
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
    // Open devtools in detached mode if debugging is needed
    // mainWindow.webContents.openDevTools({ mode: 'detach' });
  } else {
    mainWindow.loadFile(path.join(__dirname, '../dist/index.html'));
  }

  // Open external links (such as revolut.me or Riot Dev Portal) in default web browser
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
    console.log('[HexCore] LCU WebSocket Connected');
    // Subscribe to all json api events
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
    } catch (err) {
      // ignore parse heartbeat noise
    }
  });

  ws.on('error', (err) => {
    console.warn('[HexCore] LCU WebSocket error:', err.message);
  });

  ws.on('close', () => {
    console.log('[HexCore] LCU WebSocket Closed');
    notifyRenderer('lcu-status', { connected: false });
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
    }

    // If entering InProgress, start polling Live Client Data
    if (data === 'InProgress') {
      startLiveGamePolling();
    } else {
      stopLiveGamePolling();
    }
  }

  // 2. Ready Check (Queue Pop)
  if (uri === '/lol-matchmaking/v1/ready-check') {
    notifyRenderer('ready-check', data);
    if (appSettings.autoAccept && data && data.state === 'InProgress' && data.playerResponse === 'None') {
      console.log('[HexCore] Auto-accepting queue match!');
      requestLCU('/lol-matchmaking/v1/ready-check/accept', 'POST')
        .catch(err => console.error('Auto accept error:', err));
    }
  }

  // 3. Champ Select Session
  if (uri === '/lol-champ-select/v1/session') {
    champSelectSession = data;
    notifyRenderer('champ-select-session', data);
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
        signal: AbortSignal.timeout(1500)
      });
      if (res.ok) {
        const liveData = await res.json();
        notifyRenderer('live-game-update', liveData);
      }
    } catch {
      // Game may still be loading or port not ready
    }
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
      console.log('[HexCore] Detected League Client on port:', creds.port);
      lcuCredentials = creds;
      connectWebSocket(creds);

      // Fetch initial gameflow phase
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

  // 1. Get current rune pages
  const pages = await requestLCU('/lol-perks/v1/pages');
  let targetPage = pages.find(p => p.isEditable && p.isDeletable);

  if (!targetPage) {
    targetPage = pages.find(p => p.isEditable);
  }

  const payload = {
    name: pageData.name || '⚡ HexCore Build',
    primaryStyleId: pageData.primaryStyleId,
    subStyleId: pageData.subStyleId,
    selectedPerkIds: pageData.selectedPerkIds,
    current: true,
    isActive: true
  };

  if (targetPage) {
    // Delete and recreate or PUT to replace cleanly
    try {
      await requestLCU(`/lol-perks/v1/pages/${targetPage.id}`, 'DELETE');
      return await requestLCU('/lol-perks/v1/pages', 'POST', payload);
    } catch {
      // Fallback to PUT
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

  // 1. Get summonerId
  const summoner = await requestLCU('/lol-summoner/v1/current-summoner');
  if (!summoner || !summoner.summonerId) throw new Error('Summoner not found');
  const summonerId = summoner.summonerId;

  // 2. Fetch existing item sets
  let setsData = { itemSets: [], timestamp: Date.now() };
  try {
    const existing = await requestLCU(`/lol-item-sets/v1/item-sets/${summonerId}/sets`);
    if (existing && Array.isArray(existing.itemSets)) {
      setsData = existing;
    }
  } catch {}

  // 3. Remove old RiftCore set for this specific champion
  const champId = itemSetPayload.associatedChampions?.[0];
  setsData.itemSets = setsData.itemSets.filter(s => {
    if (s.title && s.title.startsWith('⚡ RiftCore:')) {
      if (champId && Array.isArray(s.associatedChampions) && s.associatedChampions.includes(champId)) {
        return false;
      }
    }
    return true;
  });

  // 4. Append new item set
  setsData.itemSets.push(itemSetPayload);

  // 5. Save back via PUT
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
      mainWindow.close();
      break;
    case 'toggle-pin':
      isPinned = !isPinned;
      mainWindow.setAlwaysOnTop(isPinned);
      return { isPinned, isCompactMode };
    case 'toggle-compact':
      isCompactMode = !isCompactMode;
      if (isCompactMode) {
        mainWindow.setSize(440, 750);
        mainWindow.setAlwaysOnTop(true);
        mainWindow.setResizable(false);
        isPinned = true;
      } else {
        mainWindow.setSize(1240, 820);
        mainWindow.setAlwaysOnTop(isPinned);
        mainWindow.setResizable(true);
      }
      return { isPinned, isCompactMode };
  }
});

// App Lifecycle
app.whenReady().then(() => {
  createWindow();
  pollForClient();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});
