import React, { useState } from 'react';
import { ShieldCheck, X, Minimize2, Power } from 'lucide-react';

interface CloseConfirmationModalProps {
  isOpen: boolean;
  onMinimizeToTray: (remember: boolean) => void;
  onExitApp: () => void;
  onCancel: () => void;
}

export const CloseConfirmationModal: React.FC<CloseConfirmationModalProps> = ({
  isOpen,
  onMinimizeToTray,
  onExitApp,
  onCancel,
}) => {
  const [remember, setRemember] = useState(true);

  if (!isOpen) return null;

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 99999,
        background: 'rgba(3, 6, 12, 0.78)',
        backdropFilter: 'blur(8px)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '20px',
      }}
    >
      <div
        className="glass-card"
        style={{
          width: '100%',
          maxWidth: '460px',
          padding: '24px',
          border: '1px solid var(--border-blue)',
          boxShadow: '0 0 35px rgba(0, 200, 255, 0.22)',
          display: 'flex',
          flexDirection: 'column',
          gap: '16px',
          position: 'relative',
          animation: 'modalPop 0.22s cubic-bezier(0.16, 1, 0.3, 1)',
        }}
      >
        {/* Close dialog button */}
        <button
          onClick={onCancel}
          style={{
            position: 'absolute',
            top: '16px',
            right: '16px',
            background: 'none',
            border: 'none',
            color: '#64748b',
            cursor: 'pointer',
          }}
          title="Cancel"
        >
          <X size={16} />
        </button>

        {/* Header */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <div
            style={{
              width: '42px',
              height: '42px',
              borderRadius: '10px',
              background: 'var(--blue-subtle)',
              border: '1px solid var(--border-blue)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              boxShadow: '0 0 14px var(--blue-glow)',
            }}
          >
            <ShieldCheck size={22} color="var(--blue-primary)" />
          </div>
          <div>
            <h3 style={{ fontSize: '16px', fontWeight: 800, color: '#fff' }}>
              Keep Running in Background?
            </h3>
            <span style={{ fontSize: '11px', color: '#94a3b8' }}>
              Windows System Tray Active
            </span>
          </div>
        </div>

        {/* Content explanation */}
        <div style={{ fontSize: '13px', color: '#cbd5e1', lineHeight: '1.6' }}>
          Minimizing to the <strong>system tray</strong> keeps <strong>Auto-Accept</strong> and the <strong>In-Game HUD Overlay</strong> active while keeping your desktop clean:
          <ul style={{ marginTop: '8px', paddingLeft: '20px', fontSize: '12px', color: '#94a3b8' }}>
            <li>Left click or double-click the tray icon to restore RiftCore.</li>
            <li>Right click the tray icon to exit completely at any time.</li>
          </ul>
        </div>

        {/* Remember choice checkbox */}
        <label
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            fontSize: '12px',
            color: '#94a3b8',
            cursor: 'pointer',
            userSelect: 'none',
            marginTop: '4px',
          }}
        >
          <input
            type="checkbox"
            checked={remember}
            onChange={(e) => setRemember(e.target.checked)}
            style={{ accentColor: 'var(--blue-primary)', cursor: 'pointer' }}
          />
          <span>Don't show this message again (Minimize to Tray on Close)</span>
        </label>

        {/* Actions */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: '8px' }}>
          <button
            onClick={onExitApp}
            style={{
              background: 'none',
              border: 'none',
              color: '#ef4444',
              fontSize: '12px',
              fontWeight: 700,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '6px 10px',
              borderRadius: '6px',
            }}
          >
            <Power size={14} />
            <span>Exit Application</span>
          </button>

          <button
            className="btn-primary"
            onClick={() => onMinimizeToTray(remember)}
            style={{
              padding: '8px 20px',
              fontSize: '13px',
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
            }}
          >
            <Minimize2 size={15} />
            <span>Minimize to Tray</span>
          </button>
        </div>
      </div>
    </div>
  );
};
