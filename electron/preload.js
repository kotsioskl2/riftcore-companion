const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('riftAPI', {
  isElectron: true,

  // Event Listeners
  onLCUStatusChange: (callback) => {
    const handler = (_event, status) => callback(status);
    ipcRenderer.on('lcu-status', handler);
    return () => ipcRenderer.removeListener('lcu-status', handler);
  },

  onGameflowPhase: (callback) => {
    const handler = (_event, phase) => callback(phase);
    ipcRenderer.on('gameflow-phase', handler);
    return () => ipcRenderer.removeListener('gameflow-phase', handler);
  },

  onChampSelectSession: (callback) => {
    const handler = (_event, session) => callback(session);
    ipcRenderer.on('champ-select-session', handler);
    return () => ipcRenderer.removeListener('champ-select-session', handler);
  },

  onReadyCheck: (callback) => {
    const handler = (_event, data) => callback(data);
    ipcRenderer.on('ready-check', handler);
    return () => ipcRenderer.removeListener('ready-check', handler);
  },

  onLiveGameUpdate: (callback) => {
    const handler = (_event, data) => callback(data);
    ipcRenderer.on('live-game-update', handler);
    return () => ipcRenderer.removeListener('live-game-update', handler);
  },

  // Actions
  lcuRequest: (endpoint, method = 'GET', body = null) => {
    return ipcRenderer.invoke('lcu-request', { endpoint, method, body });
  },

  importRunes: (pageData) => {
    return ipcRenderer.invoke('import-runes', pageData);
  },

  importSpells: (spell1Id, spell2Id) => {
    return ipcRenderer.invoke('import-spells', { spell1Id, spell2Id });
  },

  exportItemSet: (itemSet, championKey) => {
    return ipcRenderer.invoke('export-item-set', { itemSet, championKey });
  },

  acceptMatch: () => {
    return ipcRenderer.invoke('accept-match');
  },

  getLiveClientData: () => {
    return ipcRenderer.invoke('get-live-client-data');
  },

  windowControl: (action) => {
    return ipcRenderer.invoke('window-control', action);
  },

  setAlwaysOnTop: (alwaysOnTop) => {
    return ipcRenderer.invoke('set-always-on-top', alwaysOnTop);
  },

  setCompactHud: (compact) => {
    return ipcRenderer.invoke('set-compact-hud', compact);
  },

  focusWindow: () => {
    return ipcRenderer.invoke('focus-window');
  },

  getSettings: () => {
    return ipcRenderer.invoke('get-settings');
  },

  saveSettings: (settings) => {
    return ipcRenderer.invoke('save-settings', settings);
  },

  checkClientStatus: () => {
    return ipcRenderer.invoke('check-client-status');
  },

  launchLeague: () => {
    return ipcRenderer.invoke('launch-league');
  },

  getLeagueWindowMode: () => {
    return ipcRenderer.invoke('get-league-window-mode');
  },

  setLeagueFullscreen: () => {
    return ipcRenderer.invoke('set-league-fullscreen');
  },

  setLeagueBorderless: () => {
    return ipcRenderer.invoke('set-league-borderless');
  },

  resizeWindow: (width, height) => {
    return ipcRenderer.invoke('resize-window', { width, height });
  },

  snapOverlay: (position, width, height) => {
    return ipcRenderer.invoke('snap-overlay', { position, width, height });
  },

  setHudMode: (mode) => {
    return ipcRenderer.invoke('set-hud-mode', mode);
  },

  setClickThrough: (enabled) => {
    return ipcRenderer.invoke('set-click-through', enabled);
  },

  setIgnoreMouseEvents: (ignore, options) => {
    ipcRenderer.send('set-ignore-mouse-events', ignore, options);
  },

  updateInteractiveRegions: (regions) => {
    ipcRenderer.send('update-interactive-regions', regions);
  },

  setLayoutEditMode: (editMode) => {
    return ipcRenderer.invoke('set-layout-edit-mode', editMode);
  },

  riotApiRequest: (url, key) => {
    return ipcRenderer.invoke('riot-api-request', { url, key });
  },

  checkForAppUpdate: () => {
    return ipcRenderer.invoke('check-app-update');
  },

  downloadAppUpdate: (downloadUrl) => {
    return ipcRenderer.invoke('download-app-update', { downloadUrl });
  },

  onUpdateProgress: (callback) => {
    const handler = (_event, data) => callback(data);
    ipcRenderer.on('rift-update-progress', handler);
    return () => ipcRenderer.removeListener('rift-update-progress', handler);
  }
});
