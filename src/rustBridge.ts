// Rust Wry IPC Bridge for RiftCore
// Provides window.riftAPI interface when running inside native Rust runtime

import { AppSettings, ChampSelectSession, GameflowPhase, LiveGameData } from './types';

interface PendingRequest {
  resolve: (val: any) => void;
  reject: (err: any) => void;
}

const pendingRequests = new Map<number, PendingRequest>();
let reqIdCounter = 1;

// Global callback receiver invoked by Rust webview.evaluate_script
(window as any).__riftIpcReply = (id: number, error: string | null, data: any) => {
  const req = pendingRequests.get(id);
  if (req) {
    pendingRequests.delete(id);
    if (error) {
      req.reject(new Error(error));
    } else {
      req.resolve(data);
    }
  }
};

function sendRustIpc<T = any>(action: string, payload: Record<string, any> = {}): Promise<T> {
  return new Promise((resolve, reject) => {
    const id = reqIdCounter++;
    pendingRequests.set(id, { resolve, reject });

    const msg = JSON.stringify({ id, action, ...payload });

    if ((window as any).ipc && typeof (window as any).ipc.postMessage === 'function') {
      (window as any).ipc.postMessage(msg);
    } else {
      // If outside both Electron and Rust (e.g. browser preview), fallback gracefully
      console.warn(`[RustBridge] IPC not available for action: ${action}`);
      pendingRequests.delete(id);
      resolve(null as any);
    }
  });
}

const DEFAULT_SETTINGS: AppSettings = {
  autoAccept: true,
  autoImportRunes: false,
  autoImportSpells: false,
  autoImportItemSets: true,
  soundAlerts: true,
  theme: 'hextech',
  alwaysOnTop: false,
  closeToTray: true,
  riotApiKey: 'RGAPI-184ff7f6-57c0-4ae5-a5ce-722df7c55cc9',
  riotRegion: 'eun1',
  hudGoldLead: true,
  hudCsBenchmark: true,
  hudObjectiveTimers: true,
  hudSpellTimers: true,
  hudSkillOrder: true,
  hudItemGoal: true,
  hudMayhemCards: true,
  hudJunglePathing: true,
};

export function initRustBridge() {
  if (window.riftAPI) {
    // Already running in Electron with preload script
    return;
  }

  console.log('[RustBridge] Initializing RiftCore Native Rust IPC Bridge...');

  // Enable window dragging on .titlebar mousedown
  window.addEventListener('mousedown', (e) => {
    const target = e.target as HTMLElement | null;
    if (!target) return;

    // Check if clicked element or its parent is the titlebar and NOT a button/interactive element
    const titlebar = target.closest('.titlebar');
    const isControl = target.closest('button') || target.closest('.control-btn') || target.closest('input');

    if (titlebar && !isControl && (window as any).ipc) {
      (window as any).ipc.postMessage(JSON.stringify({ action: 'drag-window' }));
    }
  });

  window.riftAPI = {
    isElectron: false,

    onLCUStatusChange: (callback) => {
      const handler = (e: Event) => {
        const detail = (e as CustomEvent).detail;
        callback(detail);
      };
      window.addEventListener('rift-lcu-status', handler);
      return () => window.removeEventListener('rift-lcu-status', handler);
    },

    onGameflowPhase: (callback) => {
      const handler = (e: Event) => {
        const detail = (e as CustomEvent).detail as GameflowPhase;
        callback(detail);
      };
      window.addEventListener('rift-phase', handler);
      return () => window.removeEventListener('rift-phase', handler);
    },

    onChampSelectSession: (callback) => {
      const handler = (e: Event) => {
        const detail = (e as CustomEvent).detail as ChampSelectSession;
        callback(detail);
      };
      window.addEventListener('rift-champ-select', handler);
      return () => window.removeEventListener('rift-champ-select', handler);
    },

    onReadyCheck: (callback) => {
      const handler = (e: Event) => {
        const detail = (e as CustomEvent).detail;
        callback(detail);
      };
      window.addEventListener('rift-ready-check', handler);
      return () => window.removeEventListener('rift-ready-check', handler);
    },

    onLiveGameUpdate: (callback) => {
      const handler = (e: Event) => {
        const detail = (e as CustomEvent).detail as LiveGameData;
        callback(detail);
      };
      window.addEventListener('rift-live-data', handler);
      return () => window.removeEventListener('rift-live-data', handler);
    },

    lcuRequest: (endpoint: string, method: string = 'GET', body: any = null) => {
      return sendRustIpc('lcu-request', { endpoint, method, body });
    },

    importRunes: (pageData: any) => {
      return sendRustIpc('import-runes', { pageData });
    },

    importSpells: (spell1Id: number, spell2Id: number) => {
      return sendRustIpc('import-spells', { spell1Id, spell2Id });
    },

    exportItemSet: (itemSet: any, championKey?: string) => {
      return sendRustIpc('export-item-set', { itemSet, championKey });
    },

    acceptMatch: () => {
      return sendRustIpc('accept-match');
    },

    getLiveClientData: () => {
      return sendRustIpc('get-live-client-data');
    },

    windowControl: (action: 'minimize' | 'maximize' | 'close' | 'toggle-pin' | 'toggle-compact' | 'set-pin-on' | 'set-pin-off' | 'minimize-to-tray' | 'exit-app') => {
      return sendRustIpc('window-control', { subAction: action });
    },

    setAlwaysOnTop: (alwaysOnTop: boolean) => {
      return sendRustIpc('set-always-on-top', { alwaysOnTop });
    },

    setCompactHud: (compact: boolean) => {
      return sendRustIpc('set-compact-hud', { compact });
    },

    focusWindow: () => {
      return sendRustIpc('focus-window');
    },

    getSettings: async () => {
      try {
        const raw = localStorage.getItem('riftcore_settings');
        if (raw) return JSON.parse(raw);
      } catch (err) {
        console.error('Failed reading settings:', err);
      }
      return DEFAULT_SETTINGS;
    },

    saveSettings: async (settings: Partial<AppSettings>) => {
      try {
        const existing = await window.riftAPI!.getSettings();
        const updated = { ...existing, ...settings };
        localStorage.setItem('riftcore_settings', JSON.stringify(updated));
        return updated;
      } catch (err) {
        console.error('Failed saving settings:', err);
        return DEFAULT_SETTINGS;
      }
    },

    checkClientStatus: async () => {
      return sendRustIpc('check-client-status');
    },

    launchLeague: async () => {
      return sendRustIpc('launch-league');
    },

    riotApiRequest: async (url: string, key?: string) => {
      return sendRustIpc('riot-web-api', { url, key });
    },

    getLeagueWindowMode: async () => {
      return sendRustIpc('get-league-window-mode');
    },

    setLeagueFullscreen: async () => {
      return sendRustIpc('set-league-fullscreen');
    },

    setLeagueBorderless: async () => {
      return sendRustIpc('set-league-fullscreen');
    },

    resizeWindow: async (width: number, height: number) => {
      return sendRustIpc('resize-window', { width, height });
    },

    snapOverlay: async (position: string, width?: number, height?: number) => {
      return sendRustIpc('snap-overlay', { position, width, height });
    },

    setHudMode: async (mode: 'integrated' | 'card' | 'bar') => {
      return sendRustIpc('set-hud-mode', { mode });
    },

    setClickThrough: async (enabled: boolean) => {
      return sendRustIpc('set-click-through', { enabled });
    },
  };
}
