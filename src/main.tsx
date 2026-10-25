import React from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './App';
import './index.css';
import { initRustBridge } from './rustBridge';

initRustBridge();

// Permanently block browser right-click context menu (blocks Edge Print/Inspect popup)
window.addEventListener('contextmenu', (e) => {
  e.preventDefault();
  e.stopPropagation();
  return false;
}, true);

// Disable browser hotkeys: Print (Ctrl+P), Save (Ctrl+S), View Source (Ctrl+U), Open (Ctrl+O), Reload (F5 / Ctrl+R)
window.addEventListener('keydown', (e) => {
  const key = e.key ? e.key.toLowerCase() : '';
  const ctrl = e.ctrlKey || e.metaKey;
  if (ctrl && (key === 'p' || e.keyCode === 80)) { e.preventDefault(); e.stopPropagation(); return false; }
  if (ctrl && (key === 's' || e.keyCode === 83)) { e.preventDefault(); e.stopPropagation(); return false; }
  if (ctrl && (key === 'u' || e.keyCode === 85)) { e.preventDefault(); e.stopPropagation(); return false; }
  if (ctrl && (key === 'o' || e.keyCode === 79)) { e.preventDefault(); e.stopPropagation(); return false; }
  if (key === 'f5' || e.keyCode === 116 || (ctrl && (key === 'r' || e.keyCode === 82))) { e.preventDefault(); e.stopPropagation(); return false; }
  if (key === 'f12' || e.keyCode === 123 || (ctrl && e.shiftKey && (key === 'i' || e.keyCode === 73))) { e.preventDefault(); e.stopPropagation(); return false; }
}, true);

// Stub window.print
window.print = () => {
  console.warn('[RiftCore] Printing disabled in desktop app.');
};

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
