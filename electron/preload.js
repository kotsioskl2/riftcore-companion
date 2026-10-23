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

  exportItemSet: (itemSet) => {
    return ipcRenderer.invoke('export-item-set', itemSet);
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

  getSettings: () => {
    return ipcRenderer.invoke('get-settings');
  },

  saveSettings: (settings) => {
    return ipcRenderer.invoke('save-settings', settings);
  },

  checkClientStatus: () => {
    return ipcRenderer.invoke('check-client-status');
  },

  riotApiRequest: (url, key) => {
    return ipcRenderer.invoke('riot-api-request', { url, key });
  }
});
