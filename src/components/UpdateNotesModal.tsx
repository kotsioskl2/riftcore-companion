import React from 'react';
import {
  Sparkles,
  X,
  Zap,
  Swords,
  Timer,
  Check,
  ShieldCheck,
  Flame,
} from 'lucide-react';

interface UpdateNotesModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const UpdateNotesModal: React.FC<UpdateNotesModalProps> = ({ isOpen, onClose }) => {
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
          maxWidth: '500px',
          padding: '24px 26px',
          border: '1px solid rgba(0, 210, 255, 0.35)',
          borderTop: '1px solid rgba(255, 255, 255, 0.3)',
          boxShadow: '0 20px 50px -4px rgba(0, 0, 0, 0.85), 0 0 35px rgba(0, 210, 255, 0.2)',
          display: 'flex',
          flexDirection: 'column',
          gap: '16px',
          position: 'relative',
          animation: 'modalPop 0.22s cubic-bezier(0.16, 1, 0.3, 1)',
          borderRadius: 'var(--radius-lg)',
        }}
      >
        {/* Close button */}
        <button
          onClick={onClose}
          style={{
            position: 'absolute',
            top: '16px',
            right: '16px',
            background: 'none',
            border: 'none',
            color: '#94a3b8',
            cursor: 'pointer',
            padding: '6px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            borderRadius: '6px',
            transition: 'color 0.15s ease',
          }}
          title="Close"
        >
          <X size={18} />
        </button>

        {/* Modal Header */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '14px' }}>
          <div
            style={{
              width: '42px',
              height: '42px',
              borderRadius: '12px',
              background: 'rgba(0, 210, 255, 0.12)',
              border: '1px solid rgba(0, 210, 255, 0.35)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              boxShadow: '0 0 16px rgba(0, 210, 255, 0.25)',
              flexShrink: 0,
            }}
          >
            <Sparkles size={20} color="var(--blue-primary)" />
          </div>
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <span style={{ fontSize: '18px', fontWeight: 800, color: '#fff' }}>
                Hey! Here's What's New
              </span>
              <span
                style={{
                  fontSize: '10px',
                  fontWeight: 800,
                  padding: '2px 7px',
                  borderRadius: '4px',
                  background: 'rgba(0, 210, 255, 0.15)',
                  border: '1px solid rgba(0, 210, 255, 0.35)',
                  color: 'var(--blue-primary)',
                  letterSpacing: '0.4px',
                }}
              >
                v1.1.1
              </span>
            </div>
            <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
              Full native fullscreen overlay, uncluttered tier list & mode cleanup.
            </div>
          </div>
        </div>

        {/* Highlights */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
          {/* Highlight 1: Native Fullscreen */}
          <div
            style={{
              padding: '11px 14px',
              background: 'rgba(255, 255, 255, 0.03)',
              border: '1px solid var(--border-subtle)',
              borderRadius: 'var(--radius-md)',
              display: 'flex',
              gap: '12px',
              alignItems: 'flex-start',
            }}
          >
            <div
              style={{
                width: '28px',
                height: '28px',
                borderRadius: '8px',
                background: 'rgba(0, 210, 255, 0.12)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                flexShrink: 0,
                marginTop: '1px',
              }}
            >
              <Zap size={15} color="var(--blue-primary)" />
            </div>
            <div>
              <div style={{ fontSize: '13px', fontWeight: 700, color: '#fff' }}>
                ⚡ True Native Fullscreen Overlay
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px', lineHeight: '1.4' }}>
                No borderless requirement! RiftCore stays seamlessly layered over League in Native Fullscreen (<code style={{ color: 'var(--blue-primary)', background: 'rgba(0, 210, 255, 0.12)', padding: '1px 5px', borderRadius: '3px' }}>WindowMode=0</code>) with zero stutter, zero jitter, and 100% Vanguard safety.
              </div>
            </div>
          </div>

          {/* Highlight 2: Overhauled Tier List */}
          <div
            style={{
              padding: '11px 14px',
              background: 'rgba(255, 255, 255, 0.03)',
              border: '1px solid var(--border-subtle)',
              borderRadius: 'var(--radius-md)',
              display: 'flex',
              gap: '12px',
              alignItems: 'flex-start',
            }}
          >
            <div
              style={{
                width: '28px',
                height: '28px',
                borderRadius: '8px',
                background: 'rgba(16, 185, 129, 0.12)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                flexShrink: 0,
                marginTop: '1px',
              }}
            >
              <Flame size={15} color="#10b981" />
            </div>
            <div>
              <div style={{ fontSize: '13px', fontWeight: 700, color: '#fff' }}>
                👑 Uncluttered Pro Tier List
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px', lineHeight: '1.4' }}>
                Fixed cluttered boxes with a clean Leaderboard Table & Tier Brackets toggle. Fixed role filters (Illaoi is strictly TOP, Amumu is JUNGLE) and unlocked authentic patch-calibrated win/pick/ban rates.
              </div>
            </div>
          </div>

          {/* Highlight 3: KIWI Mode Translation */}
          <div
            style={{
              padding: '11px 14px',
              background: 'rgba(255, 255, 255, 0.03)',
              border: '1px solid var(--border-subtle)',
              borderRadius: 'var(--radius-md)',
              display: 'flex',
              gap: '12px',
              alignItems: 'flex-start',
            }}
          >
            <div
              style={{
                width: '28px',
                height: '28px',
                borderRadius: '8px',
                background: 'rgba(245, 158, 11, 0.12)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                flexShrink: 0,
                marginTop: '1px',
              }}
            >
              <Timer size={15} color="#f59e0b" />
            </div>
            <div>
              <div style={{ fontSize: '13px', fontWeight: 700, color: '#fff' }}>
                🎮 Clean Game Mode Names (KIWI → ARAM)
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px', lineHeight: '1.4' }}>
                Translated Riot's internal mode codenames (KIWI is Riot's code for ARAM, CHERRY for Arena) so match history displays clean, human-readable game types.
              </div>
            </div>
          </div>

          {/* Highlight 4: Live Objective Timers */}
          <div
            style={{
              padding: '11px 14px',
              background: 'rgba(255, 255, 255, 0.03)',
              border: '1px solid var(--border-subtle)',
              borderRadius: 'var(--radius-md)',
              display: 'flex',
              gap: '12px',
              alignItems: 'flex-start',
            }}
          >
            <div
              style={{
                width: '28px',
                height: '28px',
                borderRadius: '8px',
                background: 'rgba(239, 68, 68, 0.12)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                flexShrink: 0,
                marginTop: '1px',
              }}
            >
              <Swords size={15} color="var(--red-primary)" />
            </div>
            <div>
              <div style={{ fontSize: '13px', fontWeight: 700, color: '#fff' }}>
                🐉 Dynamic Live Objective Countdown
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px', lineHeight: '1.4' }}>
                Neutral epic monsters (Dragon, Voidgrubs, Herald, Baron) and jungle camps now calculate exact countdowns in real-time from active game telemetry.
              </div>
            </div>
          </div>
        </div>

        {/* Friendly Tip Box */}
        <div
          style={{
            padding: '10px 12px',
            background: 'rgba(0, 210, 255, 0.06)',
            border: '1px solid rgba(0, 210, 255, 0.2)',
            borderRadius: 'var(--radius-md)',
            display: 'flex',
            alignItems: 'center',
            gap: '10px',
            fontSize: '11.5px',
            color: '#94a3b8',
          }}
        >
          <ShieldCheck size={16} color="var(--blue-primary)" style={{ flexShrink: 0 }} />
          <span>
            <strong>In-Game Hotkey:</strong> Press <strong style={{ color: '#00c8ff' }}>Ctrl + Space</strong> or <strong style={{ color: '#00c8ff' }}>Alt + ~</strong> anytime to bring up the tactical overlay directly over Fullscreen League.
          </span>
        </div>

        {/* Modal Actions */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'flex-end',
            paddingTop: '6px',
          }}
        >
          <button
            onClick={onClose}
            className="btn-primary"
            style={{
              padding: '9px 24px',
              fontSize: '13px',
              fontWeight: 800,
              letterSpacing: '0.3px',
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
              cursor: 'pointer',
            }}
          >
            <Check size={16} />
            <span>Awesome, let's play</span>
          </button>
        </div>
      </div>
    </div>
  );
};
