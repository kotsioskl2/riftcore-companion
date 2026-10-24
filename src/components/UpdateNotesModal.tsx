import React from 'react';
import {
  Sparkles,
  X,
  Maximize2,
  Coins,
  Timer,
  Target,
  ArrowUpCircle,
  ShieldCheck,
  Check,
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
        background: 'rgba(2, 5, 11, 0.85)',
        backdropFilter: 'blur(16px)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '20px',
      }}
    >
      <div
        className="glass-card custom-scrollbar"
        style={{
          width: '100%',
          maxWidth: '540px',
          maxHeight: '90vh',
          overflowY: 'auto',
          padding: '24px 26px',
          border: '1px solid rgba(0, 210, 255, 0.35)',
          borderTop: '1px solid rgba(255, 255, 255, 0.3)',
          boxShadow: '0 20px 50px -4px rgba(0, 0, 0, 0.85), 0 0 35px rgba(0, 210, 255, 0.2)',
          display: 'flex',
          flexDirection: 'column',
          gap: '14px',
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
                Hey! Welcome to v1.1.2
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
                UPDATE v1.1.2
              </span>
            </div>
            <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
              Here is what is new in this release:
            </div>
          </div>
        </div>

        {/* Highlights List */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '9px' }}>
          {/* Highlight 0: In-App Updates & Pure Rust */}
          <div
            style={{
              padding: '11px 14px',
              background: 'linear-gradient(135deg, rgba(16, 185, 129, 0.12) 0%, rgba(0, 210, 255, 0.08) 100%)',
              border: '1px solid rgba(16, 185, 129, 0.3)',
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
                background: 'rgba(16, 185, 129, 0.2)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                flexShrink: 0,
                marginTop: '1px',
              }}
            >
              <Sparkles size={15} color="var(--emerald-primary)" />
            </div>
            <div style={{ flex: 1 }}>
              <span style={{ fontSize: '13px', fontWeight: 700, color: '#6ee7b7' }}>
                In-App Auto-Updates & 100% Native Rust Architecture
              </span>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px', lineHeight: '1.4' }}>
                Never miss an update! RiftCore scans GitHub Releases automatically and installs with 1 click. Completely purged Electron bloat—installer is now only ~5MB!
              </div>
            </div>
          </div>
          {/* Highlight 1: Full Screen In-Game HUD */}
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
                background: 'rgba(0, 210, 255, 0.14)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                flexShrink: 0,
                marginTop: '1px',
              }}
            >
              <Maximize2 size={15} color="var(--blue-primary)" />
            </div>
            <div style={{ flex: 1 }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '8px' }}>
                <span style={{ fontSize: '13px', fontWeight: 700, color: '#fff' }}>
                  Full Screen In-Game HUD
                </span>
                <span style={{ fontSize: '10px', color: 'var(--blue-primary)', background: 'rgba(0, 210, 255, 0.1)', padding: '1px 6px', borderRadius: '3px', fontWeight: 600 }}>
                  Alt + G
                </span>
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px', lineHeight: '1.4' }}>
                Works natively over League of Legends in Full Screen mode! Zero mouse lag, clicks pass straight into the game. Hit <kbd style={{ background: '#1e293b', border: '1px solid #334155', padding: '1px 5px', borderRadius: '3px', color: '#00c8ff' }}>Alt + G</kbd> in-game anytime to show or hide it.
              </div>
            </div>
          </div>

          {/* Highlight 2: Tab Section Gold Difference */}
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
                background: 'rgba(234, 179, 8, 0.14)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                flexShrink: 0,
                marginTop: '1px',
              }}
            >
              <Coins size={15} color="#eab308" />
            </div>
            <div style={{ flex: 1 }}>
              <span style={{ fontSize: '13px', fontWeight: 700, color: '#fff' }}>
                Tab Section Gold Difference
              </span>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px', lineHeight: '1.4' }}>
                Shows real-time team gold lead/deficit and lane matchup difference right at the top center scoreboard.
              </div>
            </div>
          </div>

          {/* Highlight 3: Objective & Camp Timers */}
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
                background: 'rgba(16, 185, 129, 0.14)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                flexShrink: 0,
                marginTop: '1px',
              }}
            >
              <Timer size={15} color="#10b981" />
            </div>
            <div style={{ flex: 1 }}>
              <span style={{ fontSize: '13px', fontWeight: 700, color: '#fff' }}>
                Objective & Camp Timers
              </span>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px', lineHeight: '1.4' }}>
                Separate floating timers on the map for Dragon, Baron, Voidgrubs, Blue Buff, and Red Buff.
              </div>
            </div>
          </div>

          {/* Highlight 4: Small CS Score Tracker */}
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
                background: 'rgba(168, 85, 247, 0.14)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                flexShrink: 0,
                marginTop: '1px',
              }}
            >
              <Target size={15} color="#a855f7" />
            </div>
            <div style={{ flex: 1 }}>
              <span style={{ fontSize: '13px', fontWeight: 700, color: '#fff' }}>
                Small CS Score Tracker
              </span>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px', lineHeight: '1.4' }}>
                Discreet CS score and CS/min pill beside your game clock with live matchup comparison.
              </div>
            </div>
          </div>

          {/* Highlight 5: Skill Level-Up Indicator */}
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
                background: 'rgba(249, 115, 22, 0.14)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                flexShrink: 0,
                marginTop: '1px',
              }}
            >
              <ArrowUpCircle size={15} color="#f97316" />
            </div>
            <div style={{ flex: 1 }}>
              <span style={{ fontSize: '13px', fontWeight: 700, color: '#fff' }}>
                Skill Level-Up Indicator
              </span>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px', lineHeight: '1.4' }}>
                Arrow directly above your ability bar showing your optimal skill upgrade for the next level.
              </div>
            </div>
          </div>
        </div>

        {/* Vanguard Safe Footer */}
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
            Enjoy the climb! Simple, lightweight, and <strong style={{ color: '#10b981' }}>100% Vanguard-safe</strong>.
          </span>
        </div>

        {/* Modal Actions */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'flex-end',
            paddingTop: '4px',
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
