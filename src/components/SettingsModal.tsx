import React from 'react';
import { Settings, Shield, Volume2, Zap, CheckCircle2, RotateCw, Monitor, Sparkles } from 'lucide-react';
import { AppSettings } from '../types';
import { soundService } from '../services/soundService';

interface SettingsModalProps {
  settings: AppSettings;
  onUpdateSetting: (key: keyof AppSettings, value: boolean) => void;
  isConnected: boolean;
  onReconnect: () => void;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  settings,
  onUpdateSetting,
  isConnected,
  onReconnect,
}) => {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px', maxWidth: '780px' }}>
      <div className="glass-card">
        <div className="card-header">
          <div className="card-title">
            <Settings size={18} color="var(--gold-200)" />
            <span>HexCore Preferences & Automation</span>
          </div>
          <span
            style={{
              fontSize: '11px',
              fontWeight: 700,
              color: '#25d366',
              background: 'rgba(37, 211, 102, 0.1)',
              padding: '2px 8px',
              borderRadius: '4px',
              border: '1px solid rgba(37, 211, 102, 0.3)',
            }}
          >
            100% Ad-Free Pro Edition
          </span>
        </div>

        {/* Setting Items */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {/* Auto Accept */}
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingBottom: '14px', borderBottom: '1px solid var(--border-subtle)' }}>
            <div>
              <div style={{ fontSize: '14px', fontWeight: 600, color: '#fff' }}>
                Auto Accept Ready Check (Queue Pop)
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
                Automatically clicks "Accept" when a match is found in queue. Never miss a queue pop again.
              </div>
            </div>
            <div
              className={`toggle-switch ${settings.autoAccept ? 'on' : ''}`}
              onClick={() => onUpdateSetting('autoAccept', !settings.autoAccept)}
            >
              <div className="toggle-knob" />
            </div>
          </div>

          {/* Auto Import Runes */}
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingBottom: '14px', borderBottom: '1px solid var(--border-subtle)' }}>
            <div>
              <div style={{ fontSize: '14px', fontWeight: 600, color: '#fff' }}>
                Auto Import Runes on Champion Lock
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
                Instantly writes the highest winrate rune page to your client as soon as you lock in a champion.
              </div>
            </div>
            <div
              className={`toggle-switch ${settings.autoImportRunes ? 'on' : ''}`}
              onClick={() => onUpdateSetting('autoImportRunes', !settings.autoImportRunes)}
            >
              <div className="toggle-knob" />
            </div>
          </div>

          {/* Auto Import Spells */}
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingBottom: '14px', borderBottom: '1px solid var(--border-subtle)' }}>
            <div>
              <div style={{ fontSize: '14px', fontWeight: 600, color: '#fff' }}>
                Auto Import Summoner Spells
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
                Equips Flash + Teleport / Ignite / Smite automatically based on lane assignment.
              </div>
            </div>
            <div
              className={`toggle-switch ${settings.autoImportSpells ? 'on' : ''}`}
              onClick={() => onUpdateSetting('autoImportSpells', !settings.autoImportSpells)}
            >
              <div className="toggle-knob" />
            </div>
          </div>

          {/* Audio Alerts */}
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', paddingBottom: '14px', borderBottom: '1px solid var(--border-subtle)' }}>
            <div>
              <div style={{ fontSize: '14px', fontWeight: 600, color: '#fff' }}>
                Hextech Audio Cues
              </div>
              <div style={{ fontSize: '12px', color: '#94a3b8', marginTop: '2px' }}>
                Play crystal chimes on Queue Pop, Rune Injection, and Enemy Spell Cooldown resets.
              </div>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <button
                className="btn-secondary"
                style={{ fontSize: '11px', padding: '4px 8px' }}
                onClick={() => soundService.playQueuePop()}
              >
                <Volume2 size={12} />
                <span>Test Sound</span>
              </button>
              <div
                className={`toggle-switch ${settings.soundAlerts ? 'on' : ''}`}
                onClick={() => {
                  const newVal = !settings.soundAlerts;
                  onUpdateSetting('soundAlerts', newVal);
                  soundService.setEnabled(newVal);
                }}
              >
                <div className="toggle-knob" />
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Diagnostics Card */}
      <div className="glass-card">
        <div className="card-header">
          <div className="card-title">
            <Monitor size={18} color="var(--hex-blue)" />
            <span>Connection Diagnostics</span>
          </div>
          <button className="btn-secondary" onClick={onReconnect}>
            <RotateCw size={13} />
            <span>Scan League Process</span>
          </button>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '10px', fontSize: '13px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between' }}>
            <span style={{ color: '#94a3b8' }}>League Client LCU Status:</span>
            <strong style={{ color: isConnected ? '#25d366' : '#ff4655' }}>
              {isConnected ? 'Connected (Authenticated via Lockfile)' : 'Disconnected (Launch League Client)'}
            </strong>
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between' }}>
            <span style={{ color: '#94a3b8' }}>Live Client Data Port (2999):</span>
            <strong style={{ color: '#f0e6d2' }}>Active during in-game matches</strong>
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between' }}>
            <span style={{ color: '#94a3b8' }}>Overlay Memory Footprint:</span>
            <strong style={{ color: '#0ac8b9' }}>~65 MB (Ultra Lightweight)</strong>
          </div>
        </div>
      </div>
    </div>
  );
};
