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
        background: 'rgba(2, 5, 11, 0.82)',
        backdropFilter: 'blur(16px)',
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
          maxWidth: '450px',
          padding: '22px 24px',
          border: '1px solid rgba(0, 210, 255, 0.3)',
          borderTop: '1px solid rgba(255, 255, 255, 0.2)',
          boxShadow: '0 16px 40px -4px rgba(0, 0, 0, 0.8), 0 0 30px rgba(0, 210, 255, 0.2)',
          display: 'flex',
          flexDirection: 'column',
          gap: '14px',
          position: 'relative',
          animation: 'modalPop 0.22s cubic-bezier(0.16, 1, 0.3, 1)',
          borderRadius: 'var(--radius-lg)',
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
            padding: '4px',
            borderRadius: '4px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            transition: 'color 0.15s ease',
          }}
          title="Cancel"
        >
          <X size={15} />
        </button>

        {/* Header */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <div
            style={{
              width: '40px',
              height: '40px',
              borderRadius: '10px',
              background: 'var(--blue-subtle)',
              border: '1px solid var(--border-blue)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              boxShadow: '0 0 16px var(--blue-glow-subtle)',
              flexShrink: 0,
            }}
          >
            <ShieldCheck size={20} color="var(--blue-primary)" />
          </div>
          <div>
            <h3 style={{ fontSize: '15px', fontWeight: 800, color: '#fff', letterSpacing: '0.2px' }}>
              Keep Running in Background?
            </h3>
            <span style={{ fontSize: '10.5px', color: '#94a3b8', fontWeight: 600 }}>
              Windows System Tray Active
            </span>
          </div>
        </div>

        {/* Content explanation */}
        <div style={{ fontSize: '12.5px', color: '#cbd5e1', lineHeight: '1.55' }}>
          Minimizing to the <strong>system tray</strong> keeps <strong>Auto-Accept</strong> and the <strong>In-Game HUD Overlay</strong> active while keeping your desktop clean:
          <ul style={{ marginTop: '8px', paddingLeft: '18px', fontSize: '11.5px', color: '#94a3b8' }}>
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
            fontSize: '11.5px',
            color: '#94a3b8',
            cursor: 'pointer',
            userSelect: 'none',
            marginTop: '2px',
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
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginTop: '6px' }}>
          <button
            onClick={onExitApp}
            style={{
              background: 'none',
              border: 'none',
              color: '#ff2a5f',
              fontSize: '11.5px',
              fontWeight: 700,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              padding: '6px 10px',
              borderRadius: 'var(--radius-sm)',
              transition: 'all 0.15s ease',
            }}
          >
            <Power size={13} />
            <span>Exit Application</span>
          </button>

          <button
            className="btn-primary"
            onClick={() => onMinimizeToTray(remember)}
            style={{
              padding: '7px 18px',
              fontSize: '12px',
              display: 'flex',
              alignItems: 'center',
              gap: '7px',
            }}
          >
            <Minimize2 size={14} />
            <span>Minimize to Tray</span>
          </button>
        </div>
      </div>
    </div>
  );
};
